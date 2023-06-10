//! Dataset access properties.

/*
Not implemented:
- H5P{set,get}_append_flush (due to having to deal with raw C extern callbacks)
*/

use std::fmt::{self, Debug};
use std::ops::Deref;

use hdf5_sys::h5p::{H5Pcreate, H5Pget_chunk_cache, H5Pset_chunk_cache};
#[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
use hdf5_sys::h5p::{H5Pget_all_coll_metadata_ops, H5Pset_all_coll_metadata_ops};
#[cfg(feature = "1.8.17")]
use hdf5_sys::h5p::{H5Pget_efile_prefix, H5Pset_efile_prefix};
#[cfg(feature = "1.10.0")]
use hdf5_sys::{
    h5d::H5D_vds_view_t,
    h5p::{
        H5Pget_virtual_printf_gap, H5Pget_virtual_view, H5Pset_virtual_printf_gap,
        H5Pset_virtual_view,
    },
};

pub use super::file_access::ChunkCache;
use crate::globals::H5P_DATASET_ACCESS;
use crate::internal_prelude::*;

/// Dataset access properties.
#[repr(transparent)]
pub struct DatasetAccess(Handle);

impl ObjectClass for DatasetAccess {
    const NAME: &'static str = "dataset access property list";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_GENPROP_LST];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    fn validate(&self) -> Result<()> {
        ensure!(
            self.is_class(PropertyListClass::DatasetAccess),
            "expected dataset access property list, got {:?}",
            self.class()
        );
        Ok(())
    }
}

impl Debug for DatasetAccess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut formatter = f.debug_struct("DatasetAccess");
        formatter.field("chunk_cache", &self.chunk_cache());
        #[cfg(feature = "1.8.17")]
        formatter.field("efile_prefix", &self.efile_prefix());
        #[cfg(feature = "1.10.0")]
        {
            formatter.field("virtual_view", &self.virtual_view());
            formatter.field("virtual_printf_gap", &self.virtual_printf_gap());
        }
        #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
        formatter.field("all_coll_metadata_ops", &self.all_coll_metadata_ops());
        formatter.finish()
    }
}

impl Deref for DatasetAccess {
    type Target = PropertyList;

    fn deref(&self) -> &PropertyList {
        unsafe { self.transmute() }
    }
}

impl PartialEq for DatasetAccess {
    fn eq(&self, other: &Self) -> bool {
        <PropertyList as PartialEq>::eq(self, other)
    }
}

impl Eq for DatasetAccess {}

impl Clone for DatasetAccess {
    fn clone(&self) -> Self {
        unsafe { self.deref().clone().cast_unchecked() }
    }
}

#[cfg(feature = "1.10.0")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VirtualView {
    FirstMissing,
    LastAvailable,
}

#[cfg(feature = "1.10.0")]
impl Default for VirtualView {
    fn default() -> Self {
        Self::LastAvailable
    }
}

#[cfg(feature = "1.10.0")]
impl From<H5D_vds_view_t> for VirtualView {
    fn from(view: H5D_vds_view_t) -> Self {
        match view {
            H5D_vds_view_t::H5D_VDS_FIRST_MISSING => Self::FirstMissing,
            _ => Self::LastAvailable,
        }
    }
}

#[cfg(feature = "1.10.0")]
impl From<VirtualView> for H5D_vds_view_t {
    fn from(v: VirtualView) -> Self {
        match v {
            VirtualView::FirstMissing => Self::H5D_VDS_FIRST_MISSING,
            VirtualView::LastAvailable => Self::H5D_VDS_LAST_AVAILABLE,
        }
    }
}

/// Builder used to create dataset access property list.
#[derive(Clone, Debug, Default)]
pub struct DatasetAccessBuilder {
    chunk_cache: Option<ChunkCache>,
    #[cfg(feature = "1.8.17")]
    efile_prefix: Option<String>,
    #[cfg(feature = "1.10.0")]
    virtual_view: Option<VirtualView>,
    #[cfg(feature = "1.10.0")]
    virtual_printf_gap: Option<usize>,
    #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
    all_coll_metadata_ops: Option<bool>,
}

impl DatasetAccessBuilder {
    /// Creates a new dataset access property list builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new builder from an existing property list.
    pub fn from_plist(plist: &DatasetAccess) -> Result<Self> {
        let mut builder = Self::default();
        let v = plist.get_chunk_cache()?;
        builder.chunk_cache(v.nslots, v.nbytes, v.w0);
        #[cfg(feature = "1.8.17")]
        {
            let v = plist.get_efile_prefix()?;
            builder.efile_prefix(&v);
        }
        #[cfg(feature = "1.10.0")]
        {
            builder.virtual_view(plist.get_virtual_view()?);
            builder.virtual_printf_gap(plist.get_virtual_printf_gap()?);
        }
        #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
        builder.all_coll_metadata_ops(plist.get_all_coll_metadata_ops()?);
        Ok(builder)
    }

    pub fn chunk_cache(&mut self, nslots: usize, nbytes: usize, w0: f64) -> &mut Self {
        self.chunk_cache = Some(ChunkCache { nslots, nbytes, w0 });
        self
    }

    #[cfg(feature = "1.8.17")]
    pub fn efile_prefix(&mut self, prefix: &str) -> &mut Self {
        self.efile_prefix = Some(prefix.into());
        self
    }

    #[cfg(feature = "1.10.0")]
    pub fn virtual_view(&mut self, view: VirtualView) -> &mut Self {
        self.virtual_view = Some(view);
        self
    }

    #[cfg(feature = "1.10.0")]
    pub fn virtual_printf_gap(&mut self, gap_size: usize) -> &mut Self {
        self.virtual_printf_gap = Some(gap_size);
        self
    }

    #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
    pub fn all_coll_metadata_ops(&mut self, is_collective: bool) -> &mut Self {
        self.all_coll_metadata_ops = Some(is_collective);
        self
    }

    fn populate_plist(&self, id: hid_t) -> Result<()> {
        if let Some(v) = self.chunk_cache {
            h5try!(H5Pset_chunk_cache(id, v.nslots as _, v.nbytes as _, v.w0 as _));
        }
        #[cfg(feature = "1.8.17")]
        {
            if let Some(ref v) = self.efile_prefix {
                let v = to_cstring(v.as_ref())?;
                h5try!(H5Pset_efile_prefix(id, v.as_ptr()));
            }
        }
        #[cfg(feature = "1.10.0")]
        {
            if let Some(v) = self.virtual_view {
                h5try!(H5Pset_virtual_view(id, v.into()));
            }
            if let Some(v) = self.virtual_printf_gap {
                h5try!(H5Pset_virtual_printf_gap(id, v as _));
            }
        }
        #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
        {
            if let Some(v) = self.all_coll_metadata_ops {
                h5try!(H5Pset_all_coll_metadata_ops(id, v as _));
            }
        }
        Ok(())
    }

    pub fn apply(&self, plist: &mut DatasetAccess) -> Result<()> {
        h5lock!(self.populate_plist(plist.id()))
    }

    pub fn finish(&self) -> Result<DatasetAccess> {
        h5lock!({
            let mut plist = DatasetAccess::try_new()?;
            self.apply(&mut plist).map(|_| plist)
        })
    }
}

/// Dataset access property list.
impl DatasetAccess {
    pub fn try_new() -> Result<Self> {
        Self::from_id(h5try!(H5Pcreate(*H5P_DATASET_ACCESS)))
    }

    pub fn copy(&self) -> Self {
        unsafe { self.deref().copy().cast_unchecked() }
    }

    pub fn build() -> DatasetAccessBuilder {
        DatasetAccessBuilder::new()
    }

    #[doc(hidden)]
    pub fn get_chunk_cache(&self) -> Result<ChunkCache> {
        h5get!(H5Pget_chunk_cache(self.id()): size_t, size_t, c_double).map(
            |(nslots, nbytes, w0)| ChunkCache {
                nslots: nslots as _,
                nbytes: nbytes as _,
                w0: w0 as _,
            },
        )
    }

    pub fn chunk_cache(&self) -> ChunkCache {
        self.get_chunk_cache().unwrap_or_else(|_| ChunkCache::default())
    }

    #[cfg(feature = "1.8.17")]
    #[doc(hidden)]
    pub fn get_efile_prefix(&self) -> Result<String> {
        h5lock!(get_h5_str(|m, s| H5Pget_efile_prefix(self.id(), m, s)))
    }

    #[cfg(feature = "1.8.17")]
    pub fn efile_prefix(&self) -> String {
        self.get_efile_prefix().ok().unwrap_or_default()
    }

    #[cfg(feature = "1.10.0")]
    #[doc(hidden)]
    pub fn get_virtual_view(&self) -> Result<VirtualView> {
        h5get!(H5Pget_virtual_view(self.id()): H5D_vds_view_t).map(Into::into)
    }

    #[cfg(feature = "1.10.0")]
    pub fn virtual_view(&self) -> VirtualView {
        self.get_virtual_view().ok().unwrap_or_default()
    }

    #[cfg(feature = "1.10.0")]
    #[doc(hidden)]
    pub fn get_virtual_printf_gap(&self) -> Result<usize> {
        h5get!(H5Pget_virtual_printf_gap(self.id()): hsize_t).map(|x| x as _)
    }

    #[cfg(feature = "1.10.0")]
    pub fn virtual_printf_gap(&self) -> usize {
        self.get_virtual_printf_gap().unwrap_or(0)
    }

    #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
    #[doc(hidden)]
    pub fn get_all_coll_metadata_ops(&self) -> Result<bool> {
        h5get!(H5Pget_all_coll_metadata_ops(self.id()): hbool_t).map(|x| x > 0)
    }

    #[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
    pub fn all_coll_metadata_ops(&self) -> bool {
        self.get_all_coll_metadata_ops().unwrap_or(false)
    }
}
