//! Dataset creation properties.

use std::fmt::{self, Debug};
use std::ops::Deref;

use hdf5_sys::h5p::H5Pcreate;

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

/// Builder used to create dataset creation property list.
#[derive(Clone, Debug, Default)]
pub struct DatasetCreateBuilder {}

impl DatasetCreateBuilder {
    /// Creates a new dataset creation property list builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new builder from an existing property list.
    pub fn from_plist(plist: &DatasetCreate) -> Result<Self> {
        let mut builder = Self::default();
        Ok(builder)
    }

    fn populate_plist(&self, id: hid_t) -> Result<()> {
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
}
