//! Dataset access properties.

use std::fmt::{self, Debug};
use std::ops::Deref;

use hdf5_sys::h5p::H5Pcreate;

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
pub struct DatasetAccessBuilder {}

impl DatasetAccessBuilder {
    /// Creates a new dataset access property list builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new builder from an existing property list.
    pub fn from_plist(plist: &DatasetAccess) -> Result<Self> {
        let mut builder = Self::default();
        Ok(builder)
    }

    fn populate_plist(&self, id: hid_t) -> Result<()> {
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
}
