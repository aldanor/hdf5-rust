use std::fmt::{self, Debug};
use std::ops::Deref;

use ndarray::{self, ArrayView};

use hdf5_sys::h5::HADDR_UNDEF;
use hdf5_sys::h5d::{
    H5Dcreate2, H5Dcreate_anon, H5Dget_access_plist, H5Dget_create_plist, H5Dget_offset,
    H5Dset_extent,
};
#[cfg(hdf5_1_10_5)]
use hdf5_sys::h5d::{H5Dget_chunk_info, H5Dget_num_chunks};
use hdf5_sys::h5l::H5Ldelete;
use hdf5_sys::h5p::H5P_DEFAULT;
use hdf5_sys::h5z::H5Z_filter_t;
use hdf5_types::{OwnedDynValue, TypeDescriptor};

#[cfg(feature = "blosc")]
use crate::hl::filters::{Blosc, BloscShuffle};
use crate::hl::filters::{Filter, SZip, ScaleOffset};
#[cfg(hdf5_1_10_0)]
use crate::hl::plist::dataset_access::VirtualView;
use crate::hl::plist::dataset_access::{DatasetAccess, DatasetAccessBuilder};
#[cfg(hdf5_1_10_0)]
use crate::hl::plist::dataset_create::ChunkOpts;
use crate::hl::plist::dataset_create::{
    AllocTime, AttrCreationOrder, DatasetCreate, DatasetCreateBuilder, FillTime, Layout,
};
use crate::hl::plist::link_create::{CharEncoding, LinkCreate, LinkCreateBuilder};
use crate::internal_prelude::*;

/// Default chunk size when filters are enabled and the chunk size is not specified.
/// This is the same value that netcdf uses by default.
pub const DEFAULT_CHUNK_SIZE_KB: usize = 4096;

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

#[cfg(hdf5_1_10_5)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChunkInfo {
    /// Array with a size equal to the dataset’s rank whose elements contain 0-based
    /// logical positions of the chunk’s first element in each dimension.
    pub offset: Vec<u64>,
    /// Filter mask that indicates which filters were used with the chunk when written.
    /// A zero value indicates that all enabled filters are applied on the chunk.
    /// A filter is skipped if the bit corresponding to the filter’s position in
    /// the pipeline (0 ≤ position < 32) is turned on.
    pub filter_mask: u32,
    /// Chunk address in the file.
    pub addr: u64,
    /// Chunk size in bytes.
    pub size: u64,
}

#[cfg(hdf5_1_10_5)]
impl ChunkInfo {
    pub(crate) fn new(ndim: usize) -> Self {
        let mut offset = Vec::with_capacity(ndim);
        unsafe { offset.set_len(ndim) };
        Self { offset, filter_mask: 0, addr: 0, size: 0 }
    }

    /// Returns positional indices of disabled filters.
    pub fn disabled_filters(&self) -> Vec<usize> {
        (0..32)
            .filter_map(|i| if self.filter_mask & (1 << i) != 0 { Some(i) } else { None })
            .collect()
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
        self.dcpl().map_or(Default::default(), |pl| pl.layout())
    }

    #[cfg(hdf5_1_10_5)]
    /// Returns the number of chunks if the dataset is chunked.
    pub fn num_chunks(&self) -> Option<usize> {
        if !self.is_chunked() {
            return None;
        }
        h5lock!(self.space().map_or(None, |s| {
            let mut n: hsize_t = 0;
            h5check(H5Dget_num_chunks(self.id(), s.id(), &mut n)).map(|_| n as _).ok()
        }))
    }

    #[cfg(hdf5_1_10_5)]
    /// Retrieves the chunk information for the chunk specified by its index.
    pub fn chunk_info(&self, index: usize) -> Option<ChunkInfo> {
        if !self.is_chunked() {
            return None;
        }
        h5lock!(self.space().map_or(None, |s| {
            let mut chunk_info = ChunkInfo::new(self.ndim());
            h5check(H5Dget_chunk_info(
                self.id(),
                s.id(),
                index as _,
                chunk_info.offset.as_mut_ptr(),
                &mut chunk_info.filter_mask,
                &mut chunk_info.addr,
                &mut chunk_info.size,
            ))
            .map(|_| chunk_info)
            .ok()
        }))
    }

    /// Returns the chunk shape if the dataset is chunked.
    pub fn chunk(&self) -> Option<Vec<Ix>> {
        self.dcpl().map_or(None, |pl| pl.chunk())
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
        self.dcpl().map_or(Default::default(), |pl| pl.filters())
    }
}

pub struct Maybe<T>(Option<T>);

impl<T> Deref for Maybe<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Into<Option<T>> for Maybe<T> {
    fn into(self) -> Option<T> {
        self.0
    }
}

impl<T> From<T> for Maybe<T> {
    fn from(v: T) -> Maybe<T> {
        Self(Some(v))
    }
}

impl<T> From<Option<T>> for Maybe<T> {
    fn from(v: Option<T>) -> Maybe<T> {
        Self(v)
    }
}

#[derive(Clone)]
pub struct DatasetBuilderEmpty<'a> {
    builder: &'a DatasetBuilder,
    type_desc: TypeDescriptor,
}

impl<'a> DatasetBuilderEmpty<'a> {
    pub fn shape<S: Into<Extents>>(&'a self, extents: S) -> DatasetBuilderEmptyShape<'a> {
        DatasetBuilderEmptyShape {
            builder: self.builder,
            type_desc: &self.type_desc,
            extents: extents.into(),
        }
    }
}

#[derive(Clone)]
pub struct DatasetBuilderEmptyShape<'a> {
    builder: &'a DatasetBuilder,
    type_desc: &'a TypeDescriptor,
    extents: Extents,
}

impl<'a> DatasetBuilderEmptyShape<'a> {
    pub fn create<'n, T: Into<Maybe<&'n str>>>(&self, name: T) -> Result<Dataset> {
        h5lock!(self.builder.create(&self.type_desc, name.into().into(), &self.extents))
    }
}

#[derive(Clone)]
pub struct DatasetBuilderData<'a, 'b, T, D> {
    builder: &'a DatasetBuilder,
    data: ArrayView<'b, T, D>,
    type_desc: TypeDescriptor,
    resizable: bool,
}

impl<'a, 'b, T, D> DatasetBuilderData<'a, 'b, T, D>
where
    T: H5Type,
    D: ndarray::Dimension,
{
    pub fn resizable(&mut self, resizable: bool) -> &mut Self {
        self.resizable = resizable;
        self
    }

    pub fn create<'n, N: Into<Maybe<&'n str>>>(&self, name: N) -> Result<Dataset> {
        ensure!(
            self.data.is_standard_layout(),
            "input array is not in standard layout or is not contiguous"
        ); // TODO: relax this when it's supported in the writer
        let extents = Extents::from(self.data.shape());
        let extents = if self.resizable { extents.resizable() } else { extents };
        let name = name.into().into();
        h5lock!({
            let dtype_src = Datatype::from_type::<T>()?;
            let dtype_dst = Datatype::from_descriptor(&self.type_desc)?;
            // TODO: soft conversion? hard? user-specifiable?
            dtype_src.ensure_convertible(&dtype_dst, Conversion::Soft)?;
            let ds = self.builder.create(&self.type_desc, name, &extents)?;
            if let Err(err) = ds.write(self.data.view()) {
                let _ = self.builder.try_unlink(name);
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

pub(crate) fn compute_chunk_shape(type_size: usize, dims: &[Ix], min_kb: usize) -> Vec<Ix> {}

#[derive(Clone)]
pub struct DatasetBuilder {
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

impl DatasetBuilder {
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

    pub fn packed(&mut self, packed: bool) -> &mut Self {
        self.packed = packed;
        self
    }

    pub fn empty<T: H5Type>(&self) -> DatasetBuilderEmpty {
        self.empty_as(&T::type_descriptor())
    }

    pub fn empty_as(&self, type_desc: &TypeDescriptor) -> DatasetBuilderEmpty {
        DatasetBuilderEmpty { builder: self, type_desc: type_desc.clone() }
    }

    pub fn with_data<'a, 'b, A, T, D>(&'a self, data: A) -> DatasetBuilderData<'a, 'b, T, D>
    where
        A: Into<ArrayView<'b, T, D>>,
        T: H5Type,
        D: ndarray::Dimension,
    {
        self.with_data_as::<A, T, D>(data, &T::type_descriptor())
    }

    pub fn with_data_as<'a, 'b, A, T, D>(
        &'a self, data: A, type_desc: &TypeDescriptor,
    ) -> DatasetBuilderData<'a, 'b, T, D>
    where
        A: Into<ArrayView<'b, T, D>>,
        T: H5Type,
        D: ndarray::Dimension,
    {
        DatasetBuilderData {
            builder: self,
            data: data.into(),
            type_desc: type_desc.clone(),
            resizable: false,
        }
    }

    fn build_dapl(&self) -> Result<DatasetAccess> {
        let mut dapl = match &self.dapl_base {
            Some(dapl) => dapl.clone(),
            None => DatasetAccess::try_new()?,
        };
        self.dapl_builder.apply(&mut dapl).map(|_| dapl)
    }

    fn compute_chunk_shape(&self, extents: &Extents) -> Result<Option<Vec<Ix>>> {
        let has_filters =
            self.dcpl_builder.has_filters() || self.dcpl_base.map_or(false, |pl| pl.has_filters());
        let chunking_required = has_filters || extents.is_resizable();
        let chunking_allowed = extents.ndim() > 0 && (extents.size() > 0 || extents.is_resizable());

        let chunk = if let Some(chunk) = *self.chunk {
            chunk
        } else if chunking_required && chunking_allowed {
            Chunk::MinKB(DEFAULT_CHUNK_SIZE_KB)
        } else {
            Chunk::None
        };

        let chunk_shape = match chunk {
            Chunk::Exact(chunk) => Some(chunk),
            Chunk::MinKB(size) => Some(compute_chunk_shape(type_size, extents, size)),
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
            let chunk_size = chunk.iter().product();
            ensure!(chunk_size > 0, "All chunk dimensions must be positive, got {:?}", chunk);
            let dims_ok =
                extents.iter().zip(chunk).map(|(e, c)| e.max.is_none() || *c <= e.dim).all();
            ensure!(dims_ok, "Chunk dimensions ({:?}) exceed data shape ({:?})", chunk, extents);
        }
        Ok(chunk_shape)
    }

    fn build_dcpl(&self, dtype: &Datatype, extents: &Extents) -> Result<DatasetCreate> {
        self.dcpl_builder.validate_filters(dtype.id())?;

        let mut dcpl_builder = self.dcpl_builder.clone();
        if let Some(chunk) = self.compute_chunk_shape(extents)? {
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
            let name = to_cstring(name)?;
            let parent = try_ref_clone!(self.parent);
            h5lock!(H5Ldelete(parent.id(), name.as_ptr(), H5P_DEFAULT));
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
        let dcpl = self.build_dcpl(&dtype, &extents.dims())?;

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

    pub fn set_access_plist(&mut self, dapl: &DatasetAccess) -> &mut Self {
        self.dapl_base = Some(dapl.clone());
        self
    }

    pub fn set_dapl(&mut self, dapl: &DatasetAccess) -> &mut Self {
        self.set_access_plist(dapl)
    }

    pub fn access_plist(&mut self) -> &mut DatasetAccessBuilder {
        &mut self.dapl_builder
    }

    pub fn dapl(&mut self) -> &mut DatasetAccessBuilder {
        self.access_plist()
    }

    pub fn with_access_plist<F>(&mut self, func: F) -> &mut Self
    where
        F: Fn(&mut DatasetAccessBuilder) -> &mut DatasetAccessBuilder,
    {
        func(&mut self.dapl_builder);
        self
    }

    pub fn with_dapl<F>(&mut self, func: F) -> &mut Self
    where
        F: Fn(&mut DatasetAccessBuilder) -> &mut DatasetAccessBuilder,
    {
        self.with_access_plist(func)
    }

    // DAPL properties

    pub fn chunk_cache(&mut self, nslots: usize, nbytes: usize, w0: f64) -> &mut Self {
        self.with_dapl(|pl| pl.chunk_cache(nslots, nbytes, w0))
    }

    #[cfg(hdf5_1_8_17)]
    pub fn efile_prefix(&mut self, prefix: &str) -> &mut Self {
        self.with_dapl(|pl| pl.efile_prefix(prefix))
    }

    #[cfg(hdf5_1_10_0)]
    pub fn virtual_view(&mut self, view: VirtualView) -> &mut Self {
        self.with_dapl(|pl| pl.virtual_view(view))
    }

    #[cfg(hdf5_1_10_0)]
    pub fn virtual_printf_gap(&mut self, gap_size: usize) -> &mut Self {
        self.with_dapl(|pl| pl.virtual_printf_gap(gap_size))
    }

    #[cfg(all(hdf5_1_10_0, h5_have_parallel))]
    pub fn all_coll_metadata_ops(&mut self, is_collective: bool) -> &mut Self {
        self.with_dapl(|pl| pl.all_coll_metadata_ops(is_collective))
    }

    ////////////////////
    // DatasetCreate  //
    ////////////////////

    pub fn set_create_plist(&mut self, dcpl: &DatasetCreate) -> &mut Self {
        self.dcpl_base = Some(dcpl.clone());
        self
    }

    pub fn set_dcpl(&mut self, dcpl: &DatasetCreate) -> &mut Self {
        self.set_create_plist(dcpl)
    }

    pub fn create_plist(&mut self) -> &mut DatasetCreateBuilder {
        &mut self.dcpl_builder
    }

    pub fn dcpl(&mut self) -> &mut DatasetCreateBuilder {
        self.create_plist()
    }

    pub fn with_create_plist<F>(&mut self, func: F) -> &mut Self
    where
        F: Fn(&mut DatasetCreateBuilder) -> &mut DatasetCreateBuilder,
    {
        func(&mut self.dcpl_builder);
        self
    }

    pub fn with_dcpl<F>(&mut self, func: F) -> &mut Self
    where
        F: Fn(&mut DatasetCreateBuilder) -> &mut DatasetCreateBuilder,
    {
        self.with_create_plist(func)
    }

    // DCPL properties

    pub fn set_filters(&mut self, filters: &[Filter]) -> &mut Self {
        self.with_dcpl(|pl| pl.set_filters(filters))
    }

    pub fn deflate(&mut self, level: u8) -> &mut Self {
        self.with_dcpl(|pl| pl.deflate(level))
    }

    pub fn shuffle(&mut self) -> &mut Self {
        self.with_dcpl(|pl| pl.shuffle())
    }

    pub fn fletcher32(&mut self) -> &mut Self {
        self.with_dcpl(|pl| pl.fletcher32())
    }

    pub fn szip(&mut self, coding: SZip, px_per_block: u8) -> &mut Self {
        self.with_dcpl(|pl| pl.szip(coding, px_per_block))
    }

    pub fn nbit(&mut self) -> &mut Self {
        self.with_dcpl(|pl| pl.nbit())
    }

    pub fn scale_offset(&mut self, mode: ScaleOffset) -> &mut Self {
        self.with_dcpl(|pl| pl.scale_offset(mode))
    }

    #[cfg(feature = "lzf")]
    pub fn lzf(&mut self) -> &mut Self {
        self.with_dcpl(|pl| pl.lzf())
    }

    #[cfg(feature = "blosc")]
    pub fn blosc<T>(&mut self, complib: Blosc, clevel: u8, shuffle: T) -> &mut Self
    where
        T: Into<BloscShuffle>,
    {
        // TODO: add all the blosc_*() variants here as well?
        self.with_dcpl(|pl| pl.blosc(complib, clevel, shuffle))
    }

    pub fn add_filter(&mut self, id: H5Z_filter_t, cdata: &[c_uint]) -> &mut Self {
        self.with_dcpl(|pl| pl.add_filter(id, cdata))
    }

    pub fn clear_filters(&mut self) -> &mut Self {
        self.with_dcpl(|pl| pl.clear_filters())
    }

    pub fn alloc_time(&mut self, alloc_time: Option<AllocTime>) -> &mut Self {
        self.with_dcpl(|pl| pl.alloc_time(alloc_time))
    }

    pub fn fill_time(&mut self, fill_time: FillTime) -> &mut Self {
        self.with_dcpl(|pl| pl.fill_time(fill_time))
    }

    pub fn fill_value<T: Into<OwnedDynValue>>(&mut self, fill_value: T) -> &mut Self {
        self.dcpl_builder.fill_value(fill_value);
        self
    }

    pub fn no_fill_value(&mut self) -> &mut Self {
        self.with_dcpl(|pl| pl.no_fill_value())
    }

    pub fn chunk<D: Dimension>(&mut self, chunk: D) -> &mut Self {
        self.chunk = Some(Chunk::Exact(chunk.dims()));
        self
    }

    pub fn chunk_min_kb(&mut self, size: usize) -> &mut Self {
        self.chunk = Some(Chunk::MinKB(size));
        self
    }

    pub fn no_chunk(&mut self) -> &mut Self {
        self.chunk = Some(Chunk::None);
        self
    }

    pub fn layout(&mut self, layout: Layout) -> &mut Self {
        self.with_dcpl(|pl| pl.layout(layout))
    }

    #[cfg(hdf5_1_10_0)]
    pub fn chunk_opts(&mut self, opts: ChunkOpts) -> &mut Self {
        self.with_dcpl(|pl| pl.chunk_opts(opts))
    }

    pub fn external(&mut self, name: &str, offset: usize, size: usize) -> &mut Self {
        self.with_dcpl(|pl| pl.external(name, offset, size))
    }

    #[cfg(hdf5_1_10_0)]
    pub fn virtual_map<F, D, E1, S1, E2, S2>(
        &mut self, src_filename: F, src_dataset: D, src_extents: E1, src_selection: S1,
        vds_extents: E2, vds_selection: S2,
    ) -> &mut Self
    where
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
        self
    }

    pub fn obj_track_times(&mut self, track_times: bool) -> &mut Self {
        self.with_dcpl(|pl| pl.obj_track_times(track_times))
    }

    pub fn attr_phase_change(&mut self, max_compact: u32, min_dense: u32) -> &mut Self {
        self.with_dcpl(|pl| pl.attr_phase_change(max_compact, min_dense))
    }

    pub fn attr_creation_order(&mut self, attr_creation_order: AttrCreationOrder) -> &mut Self {
        self.with_dcpl(|pl| pl.attr_creation_order(attr_creation_order))
    }

    ////////////////////
    // LinkCreate     //
    ////////////////////

    pub fn set_link_create_plist(&mut self, lcpl: &LinkCreate) -> &mut Self {
        self.lcpl_base = Some(lcpl.clone());
        self
    }

    pub fn set_lcpl(&mut self, lcpl: &LinkCreate) -> &mut Self {
        self.set_link_create_plist(lcpl)
    }

    pub fn link_create_plist(&mut self) -> &mut LinkCreateBuilder {
        &mut self.lcpl_builder
    }

    pub fn lcpl(&mut self) -> &mut LinkCreateBuilder {
        self.link_create_plist()
    }

    pub fn with_link_create_plist<F>(&mut self, func: F) -> &mut Self
    where
        F: Fn(&mut LinkCreateBuilder) -> &mut LinkCreateBuilder,
    {
        func(&mut self.lcpl_builder);
        self
    }

    pub fn with_lcpl<F>(&mut self, func: F) -> &mut Self
    where
        F: Fn(&mut LinkCreateBuilder) -> &mut LinkCreateBuilder,
    {
        self.with_link_create_plist(func)
    }

    // LCPL properties

    pub fn create_intermediate_group(&mut self, create: bool) -> &mut Self {
        self.with_lcpl(|pl| pl.create_intermediate_group(create))
    }

    pub fn char_encoding(&mut self, encoding: CharEncoding) -> &mut Self {
        self.with_lcpl(|pl| pl.char_encoding(encoding))
    }
}
