//! Dataset creation properties.

use std::fmt::{self, Debug};
use std::ops::Deref;
use std::ptr;

use bitflags::bitflags;

use hdf5_sys::h5d::{H5D_alloc_time_t, H5D_layout_t};
use hdf5_sys::h5f::H5F_UNLIMITED;
use hdf5_sys::h5p::{
    H5Pall_filters_avail, H5Pcreate, H5Pget_alloc_time, H5Pget_chunk, H5Pget_external,
    H5Pget_external_count, H5Pget_layout, H5Pset_alloc_time, H5Pset_chunk, H5Pset_external,
    H5Pset_layout,
};
#[cfg(hdf5_1_10_0)]
use hdf5_sys::{
    h5d::H5D_CHUNK_DONT_FILTER_PARTIAL_CHUNKS,
    h5p::{H5Pget_chunk_opts, H5Pset_chunk_opts},
};

use crate::globals::H5P_DATASET_CREATE;
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
        formatter.field("alloc_time", &self.alloc_time());
        formatter.field("chunk", &self.chunk());
        formatter.field("layout", &self.layout());
        #[cfg(hdf5_1_10_0)]
        formatter.field("chunk_opts", &self.chunk_opts());
        formatter.field("external", &self.external());
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalFile {
    pub name: String,
    pub offset: usize,
    pub size: usize,
}

/// Builder used to create dataset creation property list.
#[derive(Clone, Debug, Default)]
pub struct DatasetCreateBuilder {
    alloc_time: Option<Option<AllocTime>>,
    chunk: Option<Vec<usize>>,
    layout: Option<Layout>,
    #[cfg(hdf5_1_10_0)]
    chunk_opts: Option<ChunkOpts>,
    external: Vec<ExternalFile>,
}

impl DatasetCreateBuilder {
    /// Creates a new dataset creation property list builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new builder from an existing property list.
    pub fn from_plist(plist: &DatasetCreate) -> Result<Self> {
        let mut builder = Self::default();
        builder.alloc_time(Some(plist.get_alloc_time()?));
        if let Some(v) = plist.get_chunk()? {
            builder.chunk(&v);
        }
        builder.layout(plist.get_layout()?);
        #[cfg(hdf5_1_10_0)]
        {
            if let Some(v) = plist.get_chunk_opts()? {
                builder.chunk_opts(v);
            }
        }
        for external in &plist.get_external()? {
            builder.external(&external.name, external.offset, external.size);
        }
        Ok(builder)
    }

    pub fn alloc_time(&mut self, alloc_time: Option<AllocTime>) -> &mut Self {
        self.alloc_time = Some(alloc_time);
        self
    }

    pub fn chunk(&mut self, dims: &[usize]) -> &mut Self {
        self.chunk = Some(dims.to_vec());
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

    fn populate_plist(&self, id: hid_t) -> Result<()> {
        if let Some(v) = self.alloc_time {
            let v = v.map(Into::into).unwrap_or(H5D_alloc_time_t::H5D_ALLOC_TIME_DEFAULT);
            h5try!(H5Pset_alloc_time(id, v));
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
        }
        for external in &self.external {
            let name = to_cstring(external.name.as_str())?;
            let size = if external.size != 0 { external.size as _ } else { H5F_UNLIMITED as _ };
            h5try!(H5Pset_external(id, name.as_ptr(), external.offset as _, size));
        }
        Ok(())
    }

    pub fn finish(&self) -> Result<DatasetCreate> {
        h5lock!({
            let plist = DatasetCreate::try_new()?;
            self.populate_plist(plist.id())?;
            Ok(plist)
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
    pub fn get_alloc_time(&self) -> Result<AllocTime> {
        h5get!(H5Pget_alloc_time(self.id()): H5D_alloc_time_t).map(Into::into)
    }

    pub fn alloc_time(&self) -> AllocTime {
        self.get_alloc_time().unwrap_or(AllocTime::Late)
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
}
