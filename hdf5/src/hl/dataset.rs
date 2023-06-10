use std::fmt::{self, Debug};
use std::ops::Deref;

use ndarray::{self, ArrayView};

use hdf5_sys::h5::HADDR_UNDEF;
use hdf5_sys::h5d::{
    H5Dcreate2, H5Dcreate_anon, H5Dget_access_plist, H5Dget_create_plist, H5Dget_offset,
    H5Dset_extent,
};
use hdf5_sys::h5l::H5Ldelete;
use hdf5_sys::h5p::H5P_DEFAULT;
use hdf5_sys::h5z::H5Z_filter_t;
use hdf5_types::{OwnedDynValue, TypeDescriptor};

#[cfg(feature = "blosc")]
use crate::hl::filters::{Blosc, BloscShuffle};
use crate::hl::filters::{Filter, SZip, ScaleOffset};
#[cfg(feature = "1.10.0")]
use crate::hl::plist::dataset_access::VirtualView;
use crate::hl::plist::dataset_access::{DatasetAccess, DatasetAccessBuilder};
#[cfg(feature = "1.10.0")]
use crate::hl::plist::dataset_create::ChunkOpts;
use crate::hl::plist::dataset_create::{
    AllocTime, AttrCreationOrder, DatasetCreate, DatasetCreateBuilder, FillTime, Layout,
};
use crate::hl::plist::link_create::{CharEncoding, LinkCreate, LinkCreateBuilder};
use crate::internal_prelude::*;

/// Default chunk size when filters are enabled and the chunk size is not specified.
pub const DEFAULT_CHUNK_SIZE_KB: usize = 64 * 1024;

/// Represents the HDF5 dataset object.
#[repr(transparent)]
#[derive(Clone)]
pub struct Dataset(Handle);

impl ObjectClass for Dataset {
    const NAME: &'static str = "dataset";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_DATASET];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    // TODO: short_repr()
}

impl Debug for Dataset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_fmt(f)
    }
}

impl Deref for Dataset {
    type Target = Container;

    fn deref(&self) -> &Container {
        unsafe { self.transmute() }
    }
}

impl Dataset {
    /// Returns a copy of the dataset access property list.
    pub fn access_plist(&self) -> Result<DatasetAccess> {
        h5lock!(DatasetAccess::from_id(h5try!(H5Dget_access_plist(self.id()))))
    }

    /// A short alias for `access_plist()`.
    pub fn dapl(&self) -> Result<DatasetAccess> {
        self.access_plist()
    }

    /// Returns a copy of the dataset creation property list.
    pub fn create_plist(&self) -> Result<DatasetCreate> {
        h5lock!(DatasetCreate::from_id(h5try!(H5Dget_create_plist(self.id()))))
    }

    /// A short alias for `create_plist()`.
    pub fn dcpl(&self) -> Result<DatasetCreate> {
        self.create_plist()
    }

    /// Returns `true` if this dataset is resizable along at least one axis.
    pub fn is_resizable(&self) -> bool {
        h5lock!(self.space().ok().map_or(false, |s| s.is_resizable()))
    }

    /// Returns `true` if this dataset has a chunked layout.
    pub fn is_chunked(&self) -> bool {
        self.layout() == Layout::Chunked
    }

    /// Returns the dataset layout.
    pub fn layout(&self) -> Layout {
        self.dcpl().map_or(Layout::default(), |pl| pl.layout())
    }

    #[cfg(feature = "1.10.5")]
    /// Returns the number of chunks if the dataset is chunked.
    pub fn num_chunks(&self) -> Option<usize> {
        crate::hl::chunks::get_num_chunks(self)
    }

    #[cfg(feature = "1.10.5")]
    /// Retrieves the chunk information for the chunk specified by its index.
    pub fn chunk_info(&self, index: usize) -> Option<crate::dataset::ChunkInfo> {
        crate::hl::chunks::chunk_info(self, index)
    }

    /// Returns the chunk shape if the dataset is chunked.
    pub fn chunk(&self) -> Option<Vec<Ix>> {
        self.dcpl().map_or(None, |pl| pl.chunk())
    }

    /// Visit all chunks
    #[cfg(feature = "1.14.0")]
    pub fn chunks_visit<F>(&self, callback: F) -> Result<()>
    where
        F: for<'a> FnMut(crate::dataset::ChunkInfoRef<'a>) -> i32,
    {
        crate::hl::chunks::visit(self, callback)
    }

    /// Returns the absolute byte offset of the dataset in the file if such offset is defined
    /// (which is not the case for datasets that are chunked, compact or not allocated yet).
    pub fn offset(&self) -> Option<u64> {
        match h5lock!(H5Dget_offset(self.id())) as haddr_t {
            HADDR_UNDEF => None,
            offset => Some(offset as _),
        }
    }

    /// Returns default fill value for the dataset if such value is set.
    pub fn fill_value(&self) -> Result<Option<OwnedDynValue>> {
        h5lock!(self.dcpl()?.get_fill_value(&self.dtype()?.to_descriptor()?))
    }

    /// Resizes the dataset to a new shape.
    pub fn resize<D: Dimension>(&self, shape: D) -> Result<()> {
        let mut dims: Vec<hsize_t> = vec![];
        for dim in &shape.dims() {
            dims.push(*dim as _);
        }
        h5try!(H5Dset_extent(self.id(), dims.as_ptr()));
        Ok(())
    }

    /// Returns the pipeline of filters used in this dataset.
    pub fn filters(&self) -> Vec<Filter> {
        self.dcpl().map_or(Vec::default(), |pl| pl.filters())
    }
}

pub struct Maybe<T>(Option<T>);

impl<T> Deref for Maybe<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<Maybe<T>> for Option<T> {
    fn from(v: Maybe<T>) -> Self {
        v.0
    }
}

impl<T> From<T> for Maybe<T> {
    fn from(v: T) -> Self {
        Self(Some(v))
    }
}

impl<T> From<Option<T>> for Maybe<T> {
    fn from(v: Option<T>) -> Self {
        Self(v)
    }
}

#[derive(Clone)]
/// A dataset builder
pub struct DatasetBuilder {
    builder: DatasetBuilderInner,
}

impl DatasetBuilder {
    pub fn new(parent: &Group) -> Self {
        Self { builder: DatasetBuilderInner::new(parent) }
    }

    pub fn empty<T: H5Type>(self) -> DatasetBuilderEmpty {
        self.empty_as(&T::type_descriptor())
    }

    pub fn empty_as(self, type_desc: &TypeDescriptor) -> DatasetBuilderEmpty {
        DatasetBuilderEmpty { builder: self.builder, type_desc: type_desc.clone() }
    }

    pub fn with_data<'d, A, T, D>(self, data: A) -> DatasetBuilderData<'d, T, D>
    where
        A: Into<ArrayView<'d, T, D>>,
        T: H5Type,
        D: ndarray::Dimension,
    {
        self.with_data_as::<A, T, D>(data, &T::type_descriptor())
    }

    pub fn with_data_as<'d, A, T, D>(
        self, data: A, type_desc: &TypeDescriptor,
    ) -> DatasetBuilderData<'d, T, D>
    where
        A: Into<ArrayView<'d, T, D>>,
        T: H5Type,
        D: ndarray::Dimension,
    {
        DatasetBuilderData {
            builder: self.builder,
            data: data.into(),
            type_desc: type_desc.clone(),
            conv: Conversion::Soft,
        }
    }
}

#[derive(Clone)]
/// A dataset builder with the type known
pub struct DatasetBuilderEmpty {
    builder: DatasetBuilderInner,
    type_desc: TypeDescriptor,
}

impl DatasetBuilderEmpty {
    pub fn shape<S: Into<Extents>>(self, extents: S) -> DatasetBuilderEmptyShape {
        DatasetBuilderEmptyShape {
            builder: self.builder,
            type_desc: self.type_desc,
            extents: extents.into(),
        }
    }
    pub fn create<'n, T: Into<Maybe<&'n str>>>(self, name: T) -> Result<Dataset> {
        self.shape(()).create(name)
    }
}

#[derive(Clone)]
/// A dataset builder with type and shape known
pub struct DatasetBuilderEmptyShape {
    builder: DatasetBuilderInner,
    type_desc: TypeDescriptor,
    extents: Extents,
}

impl DatasetBuilderEmptyShape {
    pub fn create<'n, T: Into<Maybe<&'n str>>>(&self, name: T) -> Result<Dataset> {
        h5lock!(self.builder.create(&self.type_desc, name.into().into(), &self.extents))
    }
}

#[derive(Clone)]
/// A dataset builder with type, shape, and data known
pub struct DatasetBuilderData<'d, T, D> {
    builder: DatasetBuilderInner,
    data: ArrayView<'d, T, D>,
    type_desc: TypeDescriptor,
    conv: Conversion,
}

impl<'d, T, D> DatasetBuilderData<'d, T, D>
where
    T: H5Type,
    D: ndarray::Dimension,
{
    /// Set maximum allowed conversion level.
    pub fn conversion(mut self, conv: Conversion) -> Self {
        self.conv = conv;
        self
    }

    /// Disallow all conversions.
    pub fn no_convert(mut self) -> Self {
        self.conv = Conversion::NoOp;
        self
    }

    pub fn create<'n, N: Into<Maybe<&'n str>>>(&self, name: N) -> Result<Dataset> {
        ensure!(
            self.data.is_standard_layout(),
            "input array is not in standard layout or is not contiguous"
        ); // TODO: relax this when it's supported in the writer
        let extents = Extents::from(self.data.shape());
        let name = name.into().into();
        h5lock!({
            let dtype_src = Datatype::from_type::<T>()?;
            let dtype_dst = Datatype::from_descriptor(&self.type_desc)?;
            dtype_src.ensure_convertible(&dtype_dst, self.conv)?;
            let ds = self.builder.create(&self.type_desc, name, &extents)?;
            if let Err(err) = ds.write(self.data.view()) {
                self.builder.try_unlink(name);
                Err(err)
            } else {
                Ok(ds)
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Chunk {
    Exact(Vec<Ix>), // exact chunk shape
    MinKB(usize),   // minimum chunk shape in KB
    None,           // leave it unchunked
}

impl Default for Chunk {
    fn default() -> Self {
        Self::None
    }
}

fn compute_chunk_shape(dims: &SimpleExtents, minimum_elements: usize) -> Vec<Ix> {
    let mut chunk_shape = vec![1; dims.ndim()];
    let mut product_cs = 1;

    // For c-order datasets we iterate from the back (fastest iteration order)
    for (extent, cs) in dims.iter().zip(chunk_shape.iter_mut()).rev() {
        if product_cs >= minimum_elements {
            break;
        }
        let wanted_size = minimum_elements / product_cs;
        // If unlimited dimension we just map to wanted_size
        *cs = extent.max.map_or(wanted_size, |maxdim| {
            // If the requested chunk size would result
            // in dividing the chunk in two uneven parts,
            // we instead merge these into the same chunk
            // to prevent having small chunks
            if 2 * wanted_size > maxdim + 1 {
                maxdim
            } else {
                std::cmp::min(wanted_size, maxdim)
            }
        });

        product_cs *= *cs;
    }
    chunk_shape
}

#[derive(Clone)]
/// The true internal dataset builder
struct DatasetBuilderInner {
    parent: Result<Handle>,
    dapl_base: Option<DatasetAccess>,
    dcpl_base: Option<DatasetCreate>,
    lcpl_base: Option<LinkCreate>,
    dapl_builder: DatasetAccessBuilder,
    dcpl_builder: DatasetCreateBuilder,
    lcpl_builder: LinkCreateBuilder,
    packed: bool,
    chunk: Option<Chunk>,
}

impl DatasetBuilderInner {
    pub fn new(parent: &Group) -> Self {
        // same as in h5py, disable time tracking by default and enable intermediate groups
        let mut dcpl = DatasetCreateBuilder::default();
        dcpl.obj_track_times(false);
        let mut lcpl = LinkCreateBuilder::default();
        lcpl.create_intermediate_group(true);

        Self {
            parent: parent.try_borrow(),
            dapl_base: None,
            dcpl_base: None,
            lcpl_base: None,
            dapl_builder: DatasetAccessBuilder::default(),
            dcpl_builder: dcpl,
            lcpl_builder: lcpl,
            packed: false,
            chunk: None,
        }
    }

    pub fn packed(&mut self, packed: bool) {
        self.packed = packed;
    }

    fn build_dapl(&self) -> Result<DatasetAccess> {
        let mut dapl = match &self.dapl_base {
            Some(dapl) => dapl.clone(),
            None => DatasetAccess::try_new()?,
        };
        self.dapl_builder.apply(&mut dapl).map(|_| dapl)
    }

    fn compute_chunk_shape(&self, dtype: &Datatype, extents: &Extents) -> Result<Option<Vec<Ix>>> {
        let extents = if let Extents::Simple(extents) = extents {
            extents
        } else {
            return Ok(None);
        };
        let has_filters = self.dcpl_builder.has_filters()
            || self.dcpl_base.as_ref().map_or(false, DatasetCreate::has_filters);
        let chunking_required = has_filters || extents.is_resizable();
        let chunking_allowed = extents.size() > 0 || extents.is_resizable();

        let chunk = if let Some(chunk) = &self.chunk {
            chunk.clone()
        } else if chunking_required && chunking_allowed {
            Chunk::MinKB(DEFAULT_CHUNK_SIZE_KB)
        } else if extents.size() == 0 {
            Chunk::Exact(vec![1; extents.ndim()])
        } else {
            Chunk::None
        };

        let chunk_shape = match chunk {
            Chunk::Exact(chunk) => Some(chunk),
            Chunk::MinKB(size) => {
                let min_elements = size / dtype.size() * 1024;
                Some(compute_chunk_shape(extents, min_elements))
            }
            Chunk::None => {
                ensure!(!extents.is_resizable(), "Chunking required for resizable datasets");
                ensure!(!has_filters, "Chunking required when filters are present");
                None
            }
        };
        if let Some(ref chunk) = chunk_shape {
            let ndim = extents.ndim();
            ensure!(ndim != 0, "Chunking cannot be enabled for 0-dim datasets");
            ensure!(ndim == chunk.len(), "Expected chunk ndim {}, got {}", ndim, chunk.len());
            let chunk_size = chunk.iter().product::<usize>();
            ensure!(chunk_size > 0, "All chunk dimensions must be positive, got {:?}", chunk);
            let dims_ok = extents.iter().zip(chunk).all(|(e, c)| e.max.is_none() || *c <= e.dim);
            let no_extent = extents.size() == 0;
            ensure!(
                dims_ok || no_extent,
                "Chunk dimensions ({:?}) exceed data shape ({:?})",
                chunk,
                extents
            );
        }
        Ok(chunk_shape)
    }

    fn build_dcpl(&self, dtype: &Datatype, extents: &Extents) -> Result<DatasetCreate> {
        self.dcpl_builder.validate_filters(dtype.id())?;

        let mut dcpl_builder = self.dcpl_builder.clone();
        if let Some(chunk) = self.compute_chunk_shape(dtype, extents)? {
            dcpl_builder.chunk(chunk);
            if !dcpl_builder.has_fill_time() {
                // prevent resize glitch (borrowed from h5py)
                dcpl_builder.fill_time(FillTime::Alloc);
            }
        } else {
            dcpl_builder.no_chunk();
        }

        let mut dcpl = match &self.dcpl_base {
            Some(dcpl) => dcpl.clone(),
            None => DatasetCreate::try_new()?,
        };
        dcpl_builder.apply(&mut dcpl).map(|_| dcpl)
    }

    fn build_lcpl(&self) -> Result<LinkCreate> {
        let mut lcpl = match &self.lcpl_base {
            Some(lcpl) => lcpl.clone(),
            None => LinkCreate::try_new()?,
        };
        self.lcpl_builder.apply(&mut lcpl).map(|_| lcpl)
    }

    fn try_unlink<'n, N: Into<Option<&'n str>>>(&self, name: N) {
        if let Some(name) = name.into() {
            let name = to_cstring(name).unwrap();
            if let Ok(parent) = &self.parent {
                h5lock!(H5Ldelete(parent.id(), name.as_ptr(), H5P_DEFAULT));
            }
        }
    }

    unsafe fn create(
        &self, desc: &TypeDescriptor, name: Option<&str>, extents: &Extents,
    ) -> Result<Dataset> {
        // construct in-file type descriptor; convert to packed representation if needed
        let desc = if self.packed { desc.to_packed_repr() } else { desc.to_c_repr() };
        let dtype = Datatype::from_descriptor(&desc)?;

        // construct DAPL and DCPL, validate filters
        let dapl = self.build_dapl()?;
        let dcpl = self.build_dcpl(&dtype, extents)?;

        // create the dataspace from extents
        let space = Dataspace::try_new(extents)?;

        // extract all ids and create the dataset
        let parent = try_ref_clone!(self.parent);
        let (pid, dtype_id, space_id, dcpl_id, dapl_id) =
            (parent.id(), dtype.id(), space.id(), dcpl.id(), dapl.id());
        let ds_id = if let Some(name) = name {
            // create named dataset
            let lcpl = self.build_lcpl()?;
            let name = to_cstring(name)?;
            H5Dcreate2(pid, name.as_ptr(), dtype_id, space_id, lcpl.id(), dcpl_id, dapl_id)
        } else {
            // create anonymous dataset
            H5Dcreate_anon(pid, dtype_id, space_id, dcpl_id, dapl_id)
        };
        Dataset::from_id(h5check(ds_id)?)
    }

    ////////////////////
    // DatasetAccess  //
    ////////////////////

    pub fn set_access_plist(&mut self, dapl: &DatasetAccess) {
        self.dapl_base = Some(dapl.clone());
    }

    pub fn set_dapl(&mut self, dapl: &DatasetAccess) {
        self.set_access_plist(dapl);
    }

    pub fn access_plist(&mut self) -> &mut DatasetAccessBuilder {
        &mut self.dapl_builder
    }

    pub fn dapl(&mut self) -> &mut DatasetAccessBuilder {
        self.access_plist()
    }

    pub fn with_access_plist<F>(&mut self, func: F)
    where
        F: Fn(&mut DatasetAccessBuilder) -> &mut DatasetAccessBuilder,
    {
        func(&mut self.dapl_builder);
    }

    pub fn with_dapl<F>(&mut self, func: F)
    where
        F: Fn(&mut DatasetAccessBuilder) -> &mut DatasetAccessBuilder,
    {
        self.with_access_plist(func);
    }

    // DAPL properties

    pub fn chunk_cache(&mut self, nslots: usize, nbytes: usize, w0: f64) {
        self.with_dapl(|pl| pl.chunk_cache(nslots, nbytes, w0));
    }

    #[cfg(feature = "1.8.17")]
    pub fn efile_prefix(&mut self, prefix: &str) {
        self.with_dapl(|pl| pl.efile_prefix(prefix));
    }

    #[cfg(feature = "1.10.0")]
    pub fn virtual_view(&mut self, view: VirtualView) {
        self.with_dapl(|pl| pl.virtual_view(view));
    }

    #[cfg(feature = "1.10.0")]
    pub fn virtual_printf_gap(&mut self, gap_size: usize) {
        self.with_dapl(|pl| pl.virtual_printf_gap(gap_size));
    }

    #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
    pub fn all_coll_metadata_ops(&mut self, is_collective: bool) {
        self.with_dapl(|pl| pl.all_coll_metadata_ops(is_collective));
    }

    ////////////////////
    // DatasetCreate  //
    ////////////////////

    pub fn set_create_plist(&mut self, dcpl: &DatasetCreate) {
        self.dcpl_base = Some(dcpl.clone());
    }

    pub fn set_dcpl(&mut self, dcpl: &DatasetCreate) {
        self.set_create_plist(dcpl);
    }

    pub fn create_plist(&mut self) -> &mut DatasetCreateBuilder {
        &mut self.dcpl_builder
    }

    pub fn dcpl(&mut self) -> &mut DatasetCreateBuilder {
        self.create_plist()
    }

    pub fn with_create_plist<F>(&mut self, func: F)
    where
        F: Fn(&mut DatasetCreateBuilder) -> &mut DatasetCreateBuilder,
    {
        func(&mut self.dcpl_builder);
    }

    pub fn with_dcpl<F>(&mut self, func: F)
    where
        F: Fn(&mut DatasetCreateBuilder) -> &mut DatasetCreateBuilder,
    {
        self.with_create_plist(func);
    }

    // DCPL properties

    pub fn set_filters(&mut self, filters: &[Filter]) {
        self.with_dcpl(|pl| pl.set_filters(filters));
    }

    pub fn deflate(&mut self, level: u8) {
        self.with_dcpl(|pl| pl.deflate(level));
    }

    pub fn shuffle(&mut self) {
        self.with_dcpl(DatasetCreateBuilder::shuffle);
    }

    pub fn fletcher32(&mut self) {
        self.with_dcpl(DatasetCreateBuilder::fletcher32);
    }

    pub fn szip(&mut self, coding: SZip, px_per_block: u8) {
        self.with_dcpl(|pl| pl.szip(coding, px_per_block));
    }

    pub fn nbit(&mut self) {
        self.with_dcpl(DatasetCreateBuilder::nbit);
    }

    pub fn scale_offset(&mut self, mode: ScaleOffset) {
        self.with_dcpl(|pl| pl.scale_offset(mode));
    }

    #[cfg(feature = "lzf")]
    /// Apply a `lzf` filter
    ///
    /// This requires the `lzf` crate feature
    pub fn lzf(&mut self) {
        self.with_dcpl(DatasetCreateBuilder::lzf);
    }

    #[cfg(feature = "blosc")]
    /// Apply a `blosc` filter
    ///
    /// This requires the `blosc` crate feature
    pub fn blosc(&mut self, complib: Blosc, clevel: u8, shuffle: impl Into<BloscShuffle>) {
        let shuffle = shuffle.into();
        self.with_dcpl(|pl| pl.blosc(complib, clevel, shuffle));
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_blosclz(&mut self, clevel: u8, shuffle: impl Into<BloscShuffle>) {
        let shuffle = shuffle.into();
        self.with_dcpl(|pl| pl.blosc_blosclz(clevel, shuffle));
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_lz4(&mut self, clevel: u8, shuffle: impl Into<BloscShuffle>) {
        let shuffle = shuffle.into();
        self.with_dcpl(|pl| pl.blosc_lz4(clevel, shuffle));
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_lz4hc(&mut self, clevel: u8, shuffle: impl Into<BloscShuffle>) {
        let shuffle = shuffle.into();
        self.with_dcpl(|pl| pl.blosc_lz4hc(clevel, shuffle));
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_snappy(&mut self, clevel: u8, shuffle: impl Into<BloscShuffle>) {
        let shuffle = shuffle.into();
        self.with_dcpl(|pl| pl.blosc_snappy(clevel, shuffle));
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_zlib(&mut self, clevel: u8, shuffle: impl Into<BloscShuffle>) {
        let shuffle = shuffle.into();
        self.with_dcpl(|pl| pl.blosc_zlib(clevel, shuffle));
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_zstd(&mut self, clevel: u8, shuffle: impl Into<BloscShuffle>) {
        let shuffle = shuffle.into();
        self.with_dcpl(|pl| pl.blosc_zstd(clevel, shuffle));
    }

    pub fn add_filter(&mut self, id: H5Z_filter_t, cdata: &[c_uint]) {
        self.with_dcpl(|pl| pl.add_filter(id, cdata));
    }

    pub fn clear_filters(&mut self) {
        self.with_dcpl(DatasetCreateBuilder::clear_filters);
    }

    pub fn alloc_time(&mut self, alloc_time: Option<AllocTime>) {
        self.with_dcpl(|pl| pl.alloc_time(alloc_time));
    }

    pub fn fill_time(&mut self, fill_time: FillTime) {
        self.with_dcpl(|pl| pl.fill_time(fill_time));
    }

    pub fn fill_value<T: Into<OwnedDynValue>>(&mut self, fill_value: T) {
        self.dcpl_builder.fill_value(fill_value);
    }

    pub fn no_fill_value(&mut self) {
        self.with_dcpl(DatasetCreateBuilder::no_fill_value);
    }

    pub fn chunk<D: Dimension>(&mut self, chunk: D) {
        self.chunk = Some(Chunk::Exact(chunk.dims()));
    }

    pub fn chunk_min_kb(&mut self, size: usize) {
        self.chunk = Some(Chunk::MinKB(size));
    }

    pub fn no_chunk(&mut self) {
        self.chunk = Some(Chunk::None);
    }

    pub fn layout(&mut self, layout: Layout) {
        self.with_dcpl(|pl| pl.layout(layout));
    }

    #[cfg(feature = "1.10.0")]
    pub fn chunk_opts(&mut self, opts: ChunkOpts) {
        self.with_dcpl(|pl| pl.chunk_opts(opts));
    }

    pub fn external(&mut self, name: &str, offset: usize, size: usize) {
        self.with_dcpl(|pl| pl.external(name, offset, size));
    }

    #[cfg(feature = "1.10.0")]
    pub fn virtual_map<F, D, E1, S1, E2, S2>(
        &mut self, src_filename: F, src_dataset: D, src_extents: E1, src_selection: S1,
        vds_extents: E2, vds_selection: S2,
    ) where
        F: AsRef<str>,
        D: AsRef<str>,
        E1: Into<Extents>,
        S1: Into<Selection>,
        E2: Into<Extents>,
        S2: Into<Selection>,
    {
        self.dcpl_builder.virtual_map(
            src_filename,
            src_dataset,
            src_extents,
            src_selection,
            vds_extents,
            vds_selection,
        );
    }

    pub fn obj_track_times(&mut self, track_times: bool) {
        self.with_dcpl(|pl| pl.obj_track_times(track_times));
    }

    pub fn attr_phase_change(&mut self, max_compact: u32, min_dense: u32) {
        self.with_dcpl(|pl| pl.attr_phase_change(max_compact, min_dense));
    }

    pub fn attr_creation_order(&mut self, attr_creation_order: AttrCreationOrder) {
        self.with_dcpl(|pl| pl.attr_creation_order(attr_creation_order));
    }

    ////////////////////
    // LinkCreate     //
    ////////////////////

    pub fn set_link_create_plist(&mut self, lcpl: &LinkCreate) {
        self.lcpl_base = Some(lcpl.clone());
    }

    pub fn set_lcpl(&mut self, lcpl: &LinkCreate) {
        self.set_link_create_plist(lcpl);
    }

    pub fn link_create_plist(&mut self) -> &mut LinkCreateBuilder {
        &mut self.lcpl_builder
    }

    pub fn lcpl(&mut self) -> &mut LinkCreateBuilder {
        self.link_create_plist()
    }

    pub fn with_link_create_plist<F>(&mut self, func: F)
    where
        F: Fn(&mut LinkCreateBuilder) -> &mut LinkCreateBuilder,
    {
        func(&mut self.lcpl_builder);
    }

    pub fn with_lcpl<F>(&mut self, func: F)
    where
        F: Fn(&mut LinkCreateBuilder) -> &mut LinkCreateBuilder,
    {
        self.with_link_create_plist(func);
    }

    // LCPL properties

    pub fn create_intermediate_group(&mut self, create: bool) {
        self.with_lcpl(|pl| pl.create_intermediate_group(create));
    }

    pub fn char_encoding(&mut self, encoding: CharEncoding) {
        self.with_lcpl(|pl| pl.char_encoding(encoding));
    }
}

macro_rules! impl_builder {
    ($plist:ident: $name:ident/$short:ident) => {
        paste::paste! {
            #[inline] #[must_use]
            pub fn [<set_ $name _plist>](mut self, $short: &$plist) -> Self {
                self.builder.[<set_ $name _plist>]($short); self
            }

            #[inline] #[must_use]
            pub fn [<set_ $short>](mut self, $short: &$plist) -> Self {
                self.builder.[<set_ $short>]($short); self
            }

            #[inline]
            pub fn [<$name _plist>](&mut self) -> &mut [<$plist Builder>] {
                self.builder.[<$name _plist>]()
            }

            #[inline]
            pub fn $short(&mut self) -> &mut [<$plist Builder>] {
                self.builder.$short()
            }

            #[inline] #[must_use]
            pub fn [<with_ $name _plist>]<F>(mut self, func: F) -> Self
            where
                F: Fn(&mut [<$plist Builder>]) -> &mut [<$plist Builder>],
            {
                self.builder.[<with_ $name _plist>](func); self
            }

            #[inline] #[must_use]
            pub fn [<with_ $short>]<F>(mut self, func: F) -> Self
            where
                F: Fn(&mut [<$plist Builder>]) -> &mut [<$plist Builder>],
            {
                self.builder.[<with_ $short>](func); self
            }
        }
    };
    (*: $name:ident($($var:ident: $ty:ty),*)) => {
        #[inline] #[must_use]
        pub fn $name(mut self $(, $var: $ty)*) -> Self {
            self.builder.$name($($var),*); self
        }
    };
    (
        $(#[$meta:meta])*
        $plist:ident: $name:ident($($var:ident: $ty:ty),*)
    ) => {
        paste::paste! {
            $(#[$meta])*
            #[inline] #[must_use] #[doc =
                "\u{21b3} [`" $plist "Builder::" $name "`]"
                "(crate::plist::" $plist "Builder::" $name ")"
            ]
            pub fn $name(mut self $(, $var: $ty)*) -> Self {
                self.builder.$name($($var),*); self
            }
        }
    };
    (
        $(#[$meta:meta])*
        $plist:ident: $name:ident<$($gid:ident: $gty:path),+>($($var:ident: $ty:ty),*)
    ) => {
        paste::paste! {
            $(#[$meta])*
            #[inline] #[must_use] #[doc =
                "\u{21b3} [`" $plist "Builder::" $name "`]"
                "(crate::plist::" $plist "Builder::" $name ")"
            ]
            pub fn $name<$($gid: $gty),+>(mut self $(, $var: $ty)*) -> Self {
                self.builder.$name($($var),*); self
            }
        }
    };
}

macro_rules! impl_builder_methods {
    () => {
        impl_builder!(*: packed(packed: bool));

        impl_builder!(DatasetAccess: access/dapl);

        impl_builder!(DatasetAccess: chunk_cache(nslots: usize, nbytes: usize, w0: f64));
        impl_builder!(#[cfg(feature = "1.8.17")] DatasetAccess: efile_prefix(prefix: &str));
        impl_builder!(#[cfg(feature = "1.10.0")] DatasetAccess: virtual_view(view: VirtualView));
        impl_builder!(#[cfg(feature = "1.10.0")] DatasetAccess: virtual_printf_gap(gap_size: usize));
        impl_builder!(
            #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
            DatasetAccess: all_coll_metadata_ops(is_collective: bool)
        );

        impl_builder!(DatasetCreate: create/dcpl);

        impl_builder!(DatasetCreate: set_filters(filters: &[Filter]));
        impl_builder!(DatasetCreate: deflate(level: u8));
        impl_builder!(DatasetCreate: shuffle());
        impl_builder!(DatasetCreate: fletcher32());
        impl_builder!(DatasetCreate: szip(coding: SZip, px_per_block: u8));
        impl_builder!(DatasetCreate: nbit());
        impl_builder!(DatasetCreate: scale_offset(mode: ScaleOffset));
        impl_builder!(#[cfg(feature = "lzf")] DatasetCreate: lzf());
        impl_builder!(
            #[cfg(feature = "blosc")]
            DatasetCreate: blosc(complib: Blosc, clevel: u8, shuffle: impl Into<BloscShuffle>)
        );
        impl_builder!(
            #[cfg(feature = "blosc")]
            DatasetCreate: blosc_blosclz(clevel: u8, shuffle: impl Into<BloscShuffle>)
        );
        impl_builder!(
            #[cfg(feature = "blosc")]
            DatasetCreate: blosc_lz4(clevel: u8, shuffle: impl Into<BloscShuffle>)
        );
        impl_builder!(
            #[cfg(feature = "blosc")]
            DatasetCreate: blosc_lz4hc(clevel: u8, shuffle: impl Into<BloscShuffle>)
        );
        impl_builder!(
            #[cfg(feature = "blosc")]
            DatasetCreate: blosc_snappy(clevel: u8, shuffle: impl Into<BloscShuffle>)
        );
        impl_builder!(
            #[cfg(feature = "blosc")]
            DatasetCreate: blosc_zlib(clevel: u8, shuffle: impl Into<BloscShuffle>)
        );
        impl_builder!(
            #[cfg(feature = "blosc")]
            DatasetCreate: blosc_zstd(clevel: u8, shuffle: impl Into<BloscShuffle>)
        );
        impl_builder!(DatasetCreate: add_filter(id: H5Z_filter_t, cdata: &[c_uint]));
        impl_builder!(DatasetCreate: clear_filters());
        impl_builder!(DatasetCreate: alloc_time(alloc_time: Option<AllocTime>));
        impl_builder!(DatasetCreate: fill_time(fill_time: FillTime));
        impl_builder!(DatasetCreate: fill_value<T: Into<OwnedDynValue>>(fill_value: T));
        impl_builder!(DatasetCreate: no_fill_value());
        impl_builder!(DatasetCreate: chunk<D: Dimension>(chunk: D));
        impl_builder!(*: chunk_min_kb(size: usize));
        impl_builder!(DatasetCreate: no_chunk());
        impl_builder!(DatasetCreate: layout(layout: Layout));
        impl_builder!(#[cfg(feature = "1.10.0")] DatasetCreate: chunk_opts(opts: ChunkOpts));
        impl_builder!(DatasetCreate: external(name: &str, offset: usize, size: usize));
        impl_builder!(
            #[cfg(feature = "1.10.0")]
            DatasetCreate: virtual_map<
                F: AsRef<str>, D: AsRef<str>,
                E1: Into<Extents>, S1: Into<Selection>, E2: Into<Extents>, S2: Into<Selection>
            >(
                src_filename: F, src_dataset: D,
                src_extents: E1, src_selection: S1, vds_extents: E2, vds_selection: S2
            )
        );
        impl_builder!(DatasetCreate: obj_track_times(track_times: bool));
        impl_builder!(DatasetCreate: attr_phase_change(max_compact: u32, min_dense: u32));
        impl_builder!(DatasetCreate: attr_creation_order(attr_creation_order: AttrCreationOrder));

        impl_builder!(LinkCreate: link_create/lcpl);

        impl_builder!(LinkCreate: create_intermediate_group(create: bool));
        impl_builder!(LinkCreate: char_encoding(encoding: CharEncoding));
    };
}

/// These methods are common to all dataset builders
impl DatasetBuilder {
    impl_builder_methods!();
}

/// The following methods are common to all dataset builders.
impl DatasetBuilderEmpty {
    impl_builder_methods!();
}

/// The following methods are common to all dataset builders.
impl DatasetBuilderEmptyShape {
    impl_builder_methods!();
}

/// The following methods are common to all dataset builders.
impl<'d, T2: H5Type, D2: ndarray::Dimension> DatasetBuilderData<'d, T2, D2> {
    impl_builder_methods!();
}

#[cfg(test)]
mod tests {
    use super::{compute_chunk_shape, DatasetBuilder};
    use crate::filters::Filter;
    use crate::test::with_tmp_file;
    use crate::{Extent, Result, SimpleExtents};

    #[cfg(feature = "blosc")]
    use crate::filters::{Blosc, BloscShuffle};

    use ndarray::Array2;

    #[allow(dead_code)]
    fn check_filter(func: impl Fn(DatasetBuilder) -> DatasetBuilder, flt: Filter) {
        let filters = vec![flt];
        with_tmp_file::<Result<_>, _>(|file| {
            let arr = Array2::<i64>::ones((1000, 20));
            func(file.new_dataset_builder()).with_data(&arr).create("foo")?;
            let ds = file.dataset("foo")?;
            assert_eq!(ds.filters(), filters);
            assert_eq!(ds.read_2d::<i64>()?, &arr);
            Ok(())
        })
        .unwrap()
    }

    #[test]
    #[cfg(feature = "blosc")]
    fn test_blosc() {
        check_filter(|d| d.blosc_zstd(9, true), Filter::Blosc(Blosc::ZStd, 9, BloscShuffle::Byte));
    }

    #[test]
    #[cfg(feature = "lzf")]
    fn test_lzf() {
        check_filter(|d| d.lzf(), Filter::LZF);
    }

    #[test]
    fn test_compute_chunk_shape() {
        let e = SimpleExtents::new(&[1, 1]);
        assert_eq!(compute_chunk_shape(&e, 1), vec![1, 1]);
        let e = SimpleExtents::new(&[1, 10]);
        assert_eq!(compute_chunk_shape(&e, 3), vec![1, 3]);
        let e = SimpleExtents::new(&[1, 10]);
        assert_eq!(compute_chunk_shape(&e, 11), vec![1, 10]);

        let e = SimpleExtents::new(&[Extent::from(1), Extent::from(10..)]);
        assert_eq!(compute_chunk_shape(&e, 11), vec![1, 11]);

        let e = SimpleExtents::new(&[Extent::from(1), Extent::from(10..)]);
        assert_eq!(compute_chunk_shape(&e, 9), vec![1, 9]);

        let e = SimpleExtents::new(&[4, 4, 4]);
        // chunk shape should be greedy here, a minimal
        // chunk shape would be (1, 3, 4) + (1, 1, 4)
        assert_eq!(compute_chunk_shape(&e, 12), vec![1, 4, 4]);

        let e = SimpleExtents::new(&[4, 4, 4]);
        assert_eq!(compute_chunk_shape(&e, 100), vec![4, 4, 4]);

        let e = SimpleExtents::new(&[4, 4, 4]);
        assert_eq!(compute_chunk_shape(&e, 9), vec![1, 2, 4]);

        let e = SimpleExtents::new(&[1, 1, 100]);
        assert_eq!(compute_chunk_shape(&e, 51), vec![1, 1, 100]);
    }

    #[test]
    fn test_read_write_scalar() {
        use crate::internal_prelude::*;
        with_tmp_file(|file| {
            if !crate::filters::deflate_available() {
                return;
            }
            let val: f64 = 0.2;
            let dataset = file.new_dataset::<f64>().deflate(3).create("foo");
            assert_err_re!(dataset, "Filter requires dataset to be chunked");

            let dataset = file.new_dataset::<f64>().create("foo").unwrap();
            dataset.write_scalar(&val).unwrap();
            let val_back = dataset.read_scalar().unwrap();
            assert_eq!(val, val_back);
        })
    }
}
