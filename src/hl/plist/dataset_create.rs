//! Dataset creation properties.

use std::fmt::{self, Debug};
use std::mem;
use std::ops::Deref;
use std::ptr;

use bitflags::bitflags;

use hdf5_sys::h5d::{H5D_alloc_time_t, H5D_fill_time_t, H5D_fill_value_t, H5D_layout_t};
use hdf5_sys::h5f::H5F_UNLIMITED;
use hdf5_sys::h5p::{
    H5Pall_filters_avail, H5Pcreate, H5Pfill_value_defined, H5Pget_alloc_time,
    H5Pget_attr_creation_order, H5Pget_attr_phase_change, H5Pget_chunk, H5Pget_external,
    H5Pget_external_count, H5Pget_fill_time, H5Pget_fill_value, H5Pget_layout,
    H5Pget_obj_track_times, H5Pset_alloc_time, H5Pset_attr_creation_order,
    H5Pset_attr_phase_change, H5Pset_chunk, H5Pset_external, H5Pset_fill_time, H5Pset_fill_value,
    H5Pset_layout, H5Pset_obj_track_times,
};
use hdf5_sys::h5t::H5Tget_class;
use hdf5_sys::h5z::H5Z_filter_t;
#[cfg(hdf5_1_10_0)]
use hdf5_sys::{
    h5d::H5D_CHUNK_DONT_FILTER_PARTIAL_CHUNKS,
    h5p::{
        H5Pget_chunk_opts, H5Pget_virtual_count, H5Pget_virtual_dsetname, H5Pget_virtual_filename,
        H5Pget_virtual_srcspace, H5Pget_virtual_vspace, H5Pset_chunk_opts, H5Pset_virtual,
    },
};
use hdf5_types::{OwnedDynValue, TypeDescriptor};

use crate::dim::Dimension;
use crate::globals::H5P_DATASET_CREATE;
use crate::hl::datatype::Datatype;
use crate::hl::filters::{validate_filters, Filter, SZip, ScaleOffset};
#[cfg(feature = "blosc")]
use crate::hl::filters::{Blosc, BloscShuffle};
pub use crate::hl::plist::common::{AttrCreationOrder, AttrPhaseChange};
use crate::internal_prelude::*;

/// Dataset creation properties.
#[repr(transparent)]
pub struct DatasetCreate(Handle);

impl ObjectClass for DatasetCreate {
    const NAME: &'static str = "dataset creation property list";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_GENPROP_LST];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    fn validate(&self) -> Result<()> {
        let class = self.class()?;
        if class != PropertyListClass::DatasetCreate {
            fail!("expected dataset creation property list, got {:?}", class);
        }
        Ok(())
    }
}

impl Debug for DatasetCreate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let _e = silence_errors();
        let mut formatter = f.debug_struct("DatasetCreate");
        formatter.field("filters", &self.filters());
        formatter.field("alloc_time", &self.alloc_time());
        formatter.field("fill_time", &self.fill_time());
        formatter.field("fill_value", &self.fill_value_defined());
        formatter.field("chunk", &self.chunk());
        formatter.field("layout", &self.layout());
        #[cfg(hdf5_1_10_0)]
        formatter.field("chunk_opts", &self.chunk_opts());
        formatter.field("external", &self.external());
        #[cfg(hdf5_1_10_0)]
        formatter.field("virtual_map", &self.virtual_map());
        formatter.field("obj_track_times", &self.obj_track_times());
        formatter.field("attr_phase_change", &self.attr_phase_change());
        formatter.field("attr_creation_order", &self.attr_creation_order());
        formatter.finish()
    }
}

impl Deref for DatasetCreate {
    type Target = PropertyList;

    fn deref(&self) -> &PropertyList {
        unsafe { self.transmute() }
    }
}

impl PartialEq for DatasetCreate {
    fn eq(&self, other: &Self) -> bool {
        <PropertyList as PartialEq>::eq(self, other)
    }
}

impl Eq for DatasetCreate {}

impl Clone for DatasetCreate {
    fn clone(&self) -> Self {
        unsafe { self.deref().clone().cast() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Compact,
    Contiguous,
    Chunked,
    #[cfg(hdf5_1_10_0)]
    Virtual,
}

impl Default for Layout {
    fn default() -> Self {
        Layout::Contiguous
    }
}

impl From<H5D_layout_t> for Layout {
    fn from(layout: H5D_layout_t) -> Self {
        match layout {
            H5D_layout_t::H5D_COMPACT => Layout::Compact,
            H5D_layout_t::H5D_CHUNKED => Layout::Chunked,
            #[cfg(hdf5_1_10_0)]
            H5D_layout_t::H5D_VIRTUAL => Layout::Virtual,
            _ => Layout::Contiguous,
        }
    }
}

impl From<Layout> for H5D_layout_t {
    fn from(layout: Layout) -> Self {
        match layout {
            Layout::Compact => H5D_layout_t::H5D_COMPACT,
            Layout::Chunked => H5D_layout_t::H5D_CHUNKED,
            #[cfg(hdf5_1_10_0)]
            Layout::Virtual => H5D_layout_t::H5D_VIRTUAL,
            _ => H5D_layout_t::H5D_CONTIGUOUS,
        }
    }
}

#[cfg(hdf5_1_10_0)]
bitflags! {
    pub struct ChunkOpts: u32 {
        const DONT_FILTER_PARTIAL_CHUNKS = H5D_CHUNK_DONT_FILTER_PARTIAL_CHUNKS;
    }
}

#[cfg(hdf5_1_10_0)]
impl Default for ChunkOpts {
    fn default() -> Self {
        ChunkOpts::DONT_FILTER_PARTIAL_CHUNKS
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllocTime {
    Early,
    Incr,
    Late,
}

impl From<H5D_alloc_time_t> for AllocTime {
    fn from(alloc_time: H5D_alloc_time_t) -> Self {
        match alloc_time {
            H5D_alloc_time_t::H5D_ALLOC_TIME_EARLY => AllocTime::Early,
            H5D_alloc_time_t::H5D_ALLOC_TIME_INCR => AllocTime::Incr,
            _ => AllocTime::Late,
        }
    }
}

impl From<AllocTime> for H5D_alloc_time_t {
    fn from(alloc_time: AllocTime) -> Self {
        match alloc_time {
            AllocTime::Early => H5D_alloc_time_t::H5D_ALLOC_TIME_EARLY,
            AllocTime::Incr => H5D_alloc_time_t::H5D_ALLOC_TIME_INCR,
            _ => H5D_alloc_time_t::H5D_ALLOC_TIME_LATE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillTime {
    IfSet,
    Alloc,
    Never,
}

impl Default for FillTime {
    fn default() -> Self {
        FillTime::IfSet
    }
}

impl From<H5D_fill_time_t> for FillTime {
    fn from(fill_time: H5D_fill_time_t) -> Self {
        match fill_time {
            H5D_fill_time_t::H5D_FILL_TIME_IFSET => FillTime::IfSet,
            H5D_fill_time_t::H5D_FILL_TIME_ALLOC => FillTime::Alloc,
            _ => FillTime::Never,
        }
    }
}

impl From<FillTime> for H5D_fill_time_t {
    fn from(fill_time: FillTime) -> Self {
        match fill_time {
            FillTime::IfSet => H5D_fill_time_t::H5D_FILL_TIME_IFSET,
            FillTime::Alloc => H5D_fill_time_t::H5D_FILL_TIME_ALLOC,
            _ => H5D_fill_time_t::H5D_FILL_TIME_NEVER,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillValue {
    Undefined,
    Default,
    UserDefined,
}

impl Default for FillValue {
    fn default() -> Self {
        FillValue::Default
    }
}

impl From<H5D_fill_value_t> for FillValue {
    fn from(fill_value: H5D_fill_value_t) -> Self {
        match fill_value {
            H5D_fill_value_t::H5D_FILL_VALUE_DEFAULT => FillValue::Default,
            H5D_fill_value_t::H5D_FILL_VALUE_USER_DEFINED => FillValue::UserDefined,
            _ => FillValue::Undefined,
        }
    }
}

impl From<FillValue> for H5D_fill_value_t {
    fn from(fill_value: FillValue) -> Self {
        match fill_value {
            FillValue::Default => H5D_fill_value_t::H5D_FILL_VALUE_DEFAULT,
            FillValue::UserDefined => H5D_fill_value_t::H5D_FILL_VALUE_USER_DEFINED,
            _ => H5D_fill_value_t::H5D_FILL_VALUE_UNDEFINED,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalFile {
    pub name: String,
    pub offset: usize,
    pub size: usize,
}

#[cfg(hdf5_1_10_0)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VirtualMapping {
    pub src_filename: String,
    pub src_dataset: String,
    pub src_extents: Extents,
    pub src_selection: Selection,
    pub vds_extents: Extents,
    pub vds_selection: Selection,
}

#[cfg(hdf5_1_10_0)]
impl VirtualMapping {
    pub fn new<F, D, E1, S1, E2, S2>(
        src_filename: F, src_dataset: D, src_extents: E1, src_selection: S1, vds_extents: E2,
        vds_selection: S2,
    ) -> Self
    where
        F: AsRef<str>,
        D: AsRef<str>,
        E1: Into<Extents>,
        S1: Into<Selection>,
        E2: Into<Extents>,
        S2: Into<Selection>,
    {
        Self {
            src_filename: src_filename.as_ref().into(),
            src_dataset: src_dataset.as_ref().into(),
            src_extents: src_extents.into(),
            src_selection: src_selection.into(),
            vds_extents: vds_extents.into(),
            vds_selection: vds_selection.into(),
        }
    }
}

/// Builder used to create dataset creation property list.
#[derive(Clone, Debug, Default)]
pub struct DatasetCreateBuilder {
    filters: Vec<Filter>,
    alloc_time: Option<Option<AllocTime>>,
    fill_time: Option<FillTime>,
    fill_value: Option<OwnedDynValue>,
    chunk: Option<Vec<usize>>,
    layout: Option<Layout>,
    #[cfg(hdf5_1_10_0)]
    chunk_opts: Option<ChunkOpts>,
    external: Vec<ExternalFile>,
    #[cfg(hdf5_1_10_0)]
    virtual_map: Vec<VirtualMapping>,
    obj_track_times: Option<bool>,
    attr_phase_change: Option<AttrPhaseChange>,
    attr_creation_order: Option<AttrCreationOrder>,
}

impl DatasetCreateBuilder {
    /// Creates a new dataset creation property list builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new builder from an existing property list.
    ///
    /// **Note**: the fill value is not copied over (due to its type not being
    /// exposed in the property list API).
    pub fn from_plist(plist: &DatasetCreate) -> Result<Self> {
        let mut builder = Self::default();
        builder.set_filters(&plist.get_filters()?);
        builder.alloc_time(Some(plist.get_alloc_time()?));
        builder.fill_time(plist.get_fill_time()?);
        if let Some(v) = plist.get_chunk()? {
            builder.chunk(&v);
        }
        let layout = plist.get_layout()?;
        builder.layout(layout);
        #[cfg(hdf5_1_10_0)]
        {
            if let Some(v) = plist.get_chunk_opts()? {
                builder.chunk_opts(v);
            }
            if layout == Layout::Virtual {
                for mapping in &plist.get_virtual_map()? {
                    builder.virtual_map(
                        &mapping.src_filename,
                        &mapping.src_dataset,
                        &mapping.src_extents,
                        &mapping.src_selection,
                        &mapping.vds_extents,
                        &mapping.vds_selection,
                    );
                }
            }
        }
        for external in &plist.get_external()? {
            builder.external(&external.name, external.offset, external.size);
        }
        builder.obj_track_times(plist.get_obj_track_times()?);
        let apc = plist.get_attr_phase_change()?;
        builder.attr_phase_change(apc.max_compact, apc.min_dense);
        builder.attr_creation_order(plist.get_attr_creation_order()?);
        Ok(builder)
    }

    pub fn set_filters(&mut self, filters: &[Filter]) -> &mut Self {
        self.filters = filters.to_owned();
        self
    }

    pub fn deflate(&mut self, level: u8) -> &mut Self {
        self.filters.push(Filter::deflate(level));
        self
    }

    pub fn shuffle(&mut self) -> &mut Self {
        self.filters.push(Filter::shuffle());
        self
    }

    pub fn fletcher32(&mut self) -> &mut Self {
        self.filters.push(Filter::fletcher32());
        self
    }

    pub fn szip(&mut self, coding: SZip, px_per_block: u8) -> &mut Self {
        self.filters.push(Filter::szip(coding, px_per_block));
        self
    }

    pub fn nbit(&mut self) -> &mut Self {
        self.filters.push(Filter::nbit());
        self
    }

    pub fn scale_offset(&mut self, mode: ScaleOffset) -> &mut Self {
        self.filters.push(Filter::scale_offset(mode));
        self
    }

    #[cfg(feature = "lzf")]
    pub fn lzf(&mut self) -> &mut Self {
        self.filters.push(Filter::lzf());
        self
    }

    #[cfg(feature = "blosc")]
    pub fn blosc<T>(&mut self, complib: Blosc, clevel: u8, shuffle: T) -> &mut Self
    where
        T: Into<BloscShuffle>,
    {
        self.filters.push(Filter::blosc(complib, clevel, shuffle));
        self
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_blosclz<T>(&mut self, clevel: u8, shuffle: T) -> &mut Self
    where
        T: Into<BloscShuffle>,
    {
        self.filters.push(Filter::blosc_blosclz(clevel, shuffle));
        self
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_lz4<T>(&mut self, clevel: u8, shuffle: T) -> &mut Self
    where
        T: Into<BloscShuffle>,
    {
        self.filters.push(Filter::blosc_lz4(clevel, shuffle));
        self
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_lz4hc<T>(&mut self, clevel: u8, shuffle: T) -> &mut Self
    where
        T: Into<BloscShuffle>,
    {
        self.filters.push(Filter::blosc_lz4hc(clevel, shuffle));
        self
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_snappy<T>(&mut self, clevel: u8, shuffle: T) -> &mut Self
    where
        T: Into<BloscShuffle>,
    {
        self.filters.push(Filter::blosc_snappy(clevel, shuffle));
        self
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_zlib<T>(&mut self, clevel: u8, shuffle: T) -> &mut Self
    where
        T: Into<BloscShuffle>,
    {
        self.filters.push(Filter::blosc_zlib(clevel, shuffle));
        self
    }

    #[cfg(feature = "blosc")]
    pub fn blosc_zstd<T>(&mut self, clevel: u8, shuffle: T) -> &mut Self
    where
        T: Into<BloscShuffle>,
    {
        self.filters.push(Filter::blosc_zstd(clevel, shuffle));
        self
    }

    pub fn add_filter(&mut self, id: H5Z_filter_t, cdata: &[c_uint]) -> &mut Self {
        self.filters.push(Filter::user(id, cdata));
        self
    }

    pub fn clear_filters(&mut self) -> &mut Self {
        self.filters.clear();
        self
    }

    pub fn alloc_time(&mut self, alloc_time: Option<AllocTime>) -> &mut Self {
        self.alloc_time = Some(alloc_time);
        self
    }

    pub fn fill_time(&mut self, fill_time: FillTime) -> &mut Self {
        self.fill_time = Some(fill_time);
        self
    }

    pub fn fill_value<T: Into<OwnedDynValue>>(&mut self, fill_value: T) -> &mut Self {
        self.fill_value = Some(fill_value.into());
        self
    }

    pub fn no_fill_value(&mut self) -> &mut Self {
        self.fill_value = None;
        self
    }

    pub fn chunk<D: Dimension>(&mut self, chunk: D) -> &mut Self {
        self.chunk = Some(chunk.dims().to_vec());
        self
    }

    pub fn no_chunk(&mut self) -> &mut Self {
        self.chunk = None;
        self
    }

    pub fn layout(&mut self, layout: Layout) -> &mut Self {
        self.layout = Some(layout);
        self
    }

    #[cfg(hdf5_1_10_0)]
    pub fn chunk_opts(&mut self, opts: ChunkOpts) -> &mut Self {
        self.chunk_opts = Some(opts);
        self
    }

    pub fn external(&mut self, name: &str, offset: usize, size: usize) -> &mut Self {
        self.external.push(ExternalFile { name: name.to_owned(), offset, size });
        self
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
        self.virtual_map.push(VirtualMapping {
            src_filename: src_filename.as_ref().into(),
            src_dataset: src_dataset.as_ref().into(),
            src_extents: src_extents.into(),
            src_selection: src_selection.into(),
            vds_extents: vds_extents.into(),
            vds_selection: vds_selection.into(),
        });
        self
    }

    pub fn obj_track_times(&mut self, track_times: bool) -> &mut Self {
        self.obj_track_times = Some(track_times);
        self
    }

    pub fn attr_phase_change(&mut self, max_compact: u32, min_dense: u32) -> &mut Self {
        self.attr_phase_change = Some(AttrPhaseChange { max_compact, min_dense });
        self
    }

    pub fn attr_creation_order(&mut self, attr_creation_order: AttrCreationOrder) -> &mut Self {
        self.attr_creation_order = Some(attr_creation_order);
        self
    }

    fn populate_plist(&self, id: hid_t) -> Result<()> {
        for filter in &self.filters {
            filter.apply_to_plist(id)?;
        }
        if let Some(v) = self.alloc_time {
            let v = v.map(Into::into).unwrap_or(H5D_alloc_time_t::H5D_ALLOC_TIME_DEFAULT);
            h5try!(H5Pset_alloc_time(id, v));
        }
        if let Some(v) = self.fill_time {
            h5try!(H5Pset_fill_time(id, v.into()));
        }
        if let Some(ref v) = self.fill_value {
            let dtype = Datatype::from_descriptor(v.type_descriptor())?;
            h5try!(H5Pset_fill_value(id, dtype.id(), v.get_buf().as_ptr() as *const _));
        }
        if let Some(v) = self.layout {
            h5try!(H5Pset_layout(id, v.into()));
        }
        if let Some(ref v) = self.chunk {
            let v = v.iter().map(|&x| x as _).collect::<Vec<_>>();
            h5try!(H5Pset_chunk(id, v.len() as _, v.as_ptr()));
        }
        #[cfg(hdf5_1_10_0)]
        {
            if let Some(v) = self.chunk_opts {
                h5try!(H5Pset_chunk_opts(id, v.bits() as _));
            }
            for v in &self.virtual_map {
                let src_filename = to_cstring(v.src_filename.as_str())?;
                let src_dataset = to_cstring(v.src_dataset.as_str())?;
                let src_space = Dataspace::try_new(&v.src_extents)?.select(&v.src_selection)?;
                let vds_space = Dataspace::try_new(&v.vds_extents)?.select(&v.vds_selection)?;
                h5try!(H5Pset_virtual(
                    id,
                    vds_space.id(),
                    src_filename.as_ptr(),
                    src_dataset.as_ptr(),
                    src_space.id()
                ));
            }
        }
        for external in &self.external {
            let name = to_cstring(external.name.as_str())?;
            let size = if external.size != 0 { external.size as _ } else { H5F_UNLIMITED as _ };
            h5try!(H5Pset_external(id, name.as_ptr(), external.offset as _, size));
        }
        if let Some(v) = self.obj_track_times {
            h5try!(H5Pset_obj_track_times(id, v as _));
        }
        if let Some(v) = self.attr_phase_change {
            h5try!(H5Pset_attr_phase_change(id, v.max_compact as _, v.min_dense as _));
        }
        if let Some(v) = self.attr_creation_order {
            h5try!(H5Pset_attr_creation_order(id, v.bits() as _));
        }
        Ok(())
    }

    pub(crate) fn validate_filters(&self, datatype_id: hid_t) -> Result<()> {
        validate_filters(&self.filters, h5lock!(H5Tget_class(datatype_id)))
    }

    pub fn apply(&self, plist: &mut DatasetCreate) -> Result<()> {
        h5lock!(self.populate_plist(plist.id()))
    }

    pub fn finish(&self) -> Result<DatasetCreate> {
        h5lock!({
            let mut plist = DatasetCreate::try_new()?;
            self.apply(&mut plist).map(|_| plist)
        })
    }
}

/// Dataset creation property list.
impl DatasetCreate {
    pub fn try_new() -> Result<Self> {
        Self::from_id(h5try!(H5Pcreate(*H5P_DATASET_CREATE)))
    }

    pub fn copy(&self) -> Self {
        unsafe { self.deref().copy().cast() }
    }

    pub fn build() -> DatasetCreateBuilder {
        DatasetCreateBuilder::new()
    }

    pub fn all_filters_avail(&self) -> bool {
        h5lock!(H5Pall_filters_avail(self.id())) > 0
    }

    #[doc(hidden)]
    pub fn get_filters(&self) -> Result<Vec<Filter>> {
        Filter::extract_pipeline(self.id())
    }

    pub fn filters(&self) -> Vec<Filter> {
        self.get_filters().unwrap_or_default()
    }

    #[doc(hidden)]
    pub fn get_alloc_time(&self) -> Result<AllocTime> {
        h5get!(H5Pget_alloc_time(self.id()): H5D_alloc_time_t).map(Into::into)
    }

    pub fn alloc_time(&self) -> AllocTime {
        self.get_alloc_time().unwrap_or(AllocTime::Late)
    }

    #[doc(hidden)]
    pub fn get_fill_time(&self) -> Result<FillTime> {
        h5get!(H5Pget_fill_time(self.id()): H5D_fill_time_t).map(Into::into)
    }

    pub fn fill_time(&self) -> FillTime {
        self.get_fill_time().unwrap_or_default()
    }

    #[doc(hidden)]
    pub fn get_fill_value_defined(&self) -> Result<FillValue> {
        h5get!(H5Pfill_value_defined(self.id()): H5D_fill_value_t).map(Into::into)
    }

    pub fn fill_value_defined(&self) -> FillValue {
        self.get_fill_value_defined().unwrap_or(FillValue::Undefined)
    }

    #[doc(hidden)]
    pub fn get_fill_value(&self, tp: &TypeDescriptor) -> Result<Option<OwnedDynValue>> {
        match self.get_fill_value_defined()? {
            FillValue::Default | FillValue::UserDefined => {
                let dtype = Datatype::from_descriptor(&tp)?;
                let mut buf: Vec<u8> = Vec::with_capacity(tp.size());
                unsafe {
                    buf.set_len(tp.size());
                }
                h5try!(H5Pget_fill_value(self.id(), dtype.id(), buf.as_mut_ptr() as *mut _));
                Ok(Some(unsafe { OwnedDynValue::from_raw(tp.clone(), buf) }))
            }
            _ => Ok(None),
        }
    }

    pub fn fill_value(&self, tp: &TypeDescriptor) -> Option<OwnedDynValue> {
        self.get_fill_value(tp).unwrap_or_default()
    }

    #[doc(hidden)]
    pub fn get_fill_value_as<T: H5Type>(&self) -> Result<Option<T>> {
        let dtype = Datatype::from_type::<T>()?;
        Ok(self.get_fill_value(&dtype.to_descriptor()?)?.map(|value| unsafe {
            let mut out: T = mem::zeroed();
            let buf = value.get_buf();
            ptr::copy_nonoverlapping(buf.as_ptr(), &mut out as *mut _ as *mut _, buf.len());
            mem::forget(value);
            out
        }))
    }

    pub fn fill_value_as<T: H5Type>(&self) -> Option<T> {
        self.get_fill_value_as::<T>().unwrap_or_default()
    }

    #[doc(hidden)]
    pub fn get_chunk(&self) -> Result<Option<Vec<usize>>> {
        if let Layout::Chunked = self.get_layout()? {
            let ndims = h5try!(H5Pget_chunk(self.id(), 0, ptr::null_mut()));
            let mut buf = vec![0 as hsize_t; ndims as usize];
            h5try!(H5Pget_chunk(self.id(), ndims, buf.as_mut_ptr()));
            Ok(Some(buf.into_iter().map(|x| x as _).collect()))
        } else {
            Ok(None)
        }
    }

    pub fn chunk(&self) -> Option<Vec<usize>> {
        self.get_chunk().unwrap_or_default()
    }

    #[doc(hidden)]
    pub fn get_layout(&self) -> Result<Layout> {
        let layout = h5lock!(H5Pget_layout(self.id()));
        h5check(layout as c_int)?;
        Ok(layout.into())
    }

    pub fn layout(&self) -> Layout {
        self.get_layout().unwrap_or_default()
    }

    #[cfg(hdf5_1_10_0)]
    #[doc(hidden)]
    pub fn get_chunk_opts(&self) -> Result<Option<ChunkOpts>> {
        if let Layout::Chunked = self.get_layout()? {
            let opts = h5get!(H5Pget_chunk_opts(self.id()): c_uint)?;
            Ok(Some(ChunkOpts::from_bits_truncate(opts as _)))
        } else {
            Ok(None)
        }
    }

    #[cfg(hdf5_1_10_0)]
    pub fn chunk_opts(&self) -> Option<ChunkOpts> {
        self.get_chunk_opts().unwrap_or_default()
    }

    #[doc(hidden)]
    pub fn get_external(&self) -> Result<Vec<ExternalFile>> {
        h5lock!({
            let mut external = Vec::new();
            let count = h5try!(H5Pget_external_count(self.id()));
            const NAME_LEN: usize = 1024;
            let mut name = vec![0 as c_char; NAME_LEN + 1];
            for idx in 0..count {
                let mut offset: libc::off_t = 0;
                let mut size: hsize_t = 0;
                h5try!(H5Pget_external(
                    self.id(),
                    idx as _,
                    NAME_LEN as _,
                    name.as_mut_ptr(),
                    &mut offset as *mut _,
                    &mut size as *mut _,
                ));
                external.push(ExternalFile {
                    name: string_from_cstr(name.as_ptr()),
                    offset: offset as _,
                    size: if size >= H5F_UNLIMITED { 0 } else { size as _ },
                })
            }
            Ok(external)
        })
    }

    pub fn external(&self) -> Vec<ExternalFile> {
        self.get_external().unwrap_or_default()
    }

    #[cfg(hdf5_1_10_0)]
    #[doc(hidden)]
    pub fn get_virtual_map(&self) -> Result<Vec<VirtualMapping>> {
        sync(|| unsafe {
            let id = self.id();
            let n_virtual = h5get!(H5Pget_virtual_count(id): size_t)? as _;
            let mut virtual_map = Vec::with_capacity(n_virtual);

            for i in 0..n_virtual {
                let src_filename = get_h5_str(|s, n| H5Pget_virtual_filename(id, i, s, n))?;
                let src_dataset = get_h5_str(|s, n| H5Pget_virtual_dsetname(id, i, s, n))?;

                let src_space_id = h5check(H5Pget_virtual_srcspace(id, i))?;
                let src_space = Dataspace::from_id(src_space_id)?;
                let src_extents = src_space.extents()?;
                let src_selection = src_space.get_selection()?;

                let vds_space_id = h5check(H5Pget_virtual_vspace(id, i))?;
                let vds_space = Dataspace::from_id(vds_space_id)?;
                let vds_extents = vds_space.extents()?;
                let vds_selection = vds_space.get_selection()?;

                virtual_map.push(VirtualMapping {
                    src_filename,
                    src_dataset,
                    src_extents,
                    src_selection,
                    vds_extents,
                    vds_selection,
                })
            }

            Ok(virtual_map)
        })
    }

    #[cfg(hdf5_1_10_0)]
    pub fn virtual_map(&self) -> Vec<VirtualMapping> {
        self.get_virtual_map().unwrap_or_default()
    }

    #[doc(hidden)]
    pub fn get_obj_track_times(&self) -> Result<bool> {
        h5get!(H5Pget_obj_track_times(self.id()): hbool_t).map(|x| x > 0)
    }

    pub fn obj_track_times(&self) -> bool {
        self.get_obj_track_times().unwrap_or(true)
    }

    #[doc(hidden)]
    pub fn get_attr_phase_change(&self) -> Result<AttrPhaseChange> {
        h5get!(H5Pget_attr_phase_change(self.id()): c_uint, c_uint)
            .map(|(mc, md)| AttrPhaseChange { max_compact: mc as _, min_dense: md as _ })
    }

    pub fn attr_phase_change(&self) -> AttrPhaseChange {
        self.get_attr_phase_change().unwrap_or_default()
    }

    #[doc(hidden)]
    pub fn get_attr_creation_order(&self) -> Result<AttrCreationOrder> {
        h5get!(H5Pget_attr_creation_order(self.id()): c_uint)
            .map(AttrCreationOrder::from_bits_truncate)
    }

    pub fn attr_creation_order(&self) -> AttrCreationOrder {
        self.get_attr_creation_order().unwrap_or_default()
    }
}
