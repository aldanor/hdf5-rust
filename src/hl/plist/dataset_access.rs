//! Dataset access properties.

use std::fmt::{self, Debug};
use std::ops::Deref;

use hdf5_sys::h5p::{H5Pcreate, H5Pget_efile_prefix, H5Pset_efile_prefix};

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
        let class = self.class()?;
        if class != PropertyListClass::DatasetAccess {
            fail!("expected dataset access property list, got {:?}", class);
        }
        Ok(())
    }
}

impl Debug for DatasetAccess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let _e = silence_errors();
        let mut formatter = f.debug_struct("DatasetAccess");
        #[cfg(hdf5_1_8_17)]
        formatter.field("efile_prefix", &self.efile_prefix());
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
        unsafe { self.deref().clone().cast() }
    }
}

/// Builder used to create dataset access property list.
#[derive(Clone, Debug, Default)]
pub struct DatasetAccessBuilder {
    #[cfg(hdf5_1_8_17)]
    efile_prefix: Option<String>,
}

impl DatasetAccessBuilder {
    /// Creates a new dataset access property list builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new builder from an existing property list.
    pub fn from_plist(plist: &DatasetAccess) -> Result<Self> {
        let mut builder = Self::default();
        #[cfg(hdf5_1_8_17)]
        {
            let v = plist.get_efile_prefix()?;
            builder.efile_prefix(&v);
        }
        Ok(builder)
    }

    #[cfg(hdf5_1_8_17)]
    pub fn efile_prefix(&mut self, prefix: &str) -> &mut Self {
        self.efile_prefix = Some(prefix.into());
        self
    }

    fn populate_plist(&self, id: hid_t) -> Result<()> {
        #[cfg(hdf5_1_8_17)]
        {
            if let Some(ref v) = self.efile_prefix {
                let v = to_cstring(v.as_ref())?;
                h5try!(H5Pset_efile_prefix(id, v.as_ptr()));
            }
        }
        Ok(())
    }

    pub fn finish(&self) -> Result<DatasetAccess> {
        h5lock!({
            let plist = DatasetAccess::try_new()?;
            self.populate_plist(plist.id())?;
            Ok(plist)
        })
    }
}

/// Dataset access property list.
impl DatasetAccess {
    pub fn try_new() -> Result<Self> {
        Self::from_id(h5try!(H5Pcreate(*H5P_DATASET_ACCESS)))
    }

    pub fn copy(&self) -> Self {
        unsafe { self.deref().copy().cast() }
    }

    pub fn build() -> DatasetAccessBuilder {
        DatasetAccessBuilder::new()
    }

    #[cfg(hdf5_1_8_17)]
    #[doc(hidden)]
    pub fn get_efile_prefix(&self) -> Result<String> {
        h5lock!(get_h5_str(|m, s| H5Pget_efile_prefix(self.id(), m, s)))
    }

    #[cfg(hdf5_1_8_17)]
    pub fn efile_prefix(&self) -> String {
        self.get_efile_prefix().unwrap_or("".into())
    }
}
