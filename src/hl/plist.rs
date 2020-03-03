use std::fmt::{self, Debug};
use std::ops::Deref;
use std::ptr;
use std::str::FromStr;

use hdf5_sys::h5p::{
    H5Pcopy, H5Pequal, H5Pexist, H5Pget_class, H5Pget_class_name, H5Pget_nprops, H5Piterate,
};

use crate::internal_prelude::*;

pub mod dataset_access;
pub mod file_access;
pub mod file_create;

/// Represents the HDF5 property list.
#[repr(transparent)]
#[derive(Clone)]
pub struct PropertyList(Handle);

impl ObjectClass for PropertyList {
    const NAME: &'static str = "property list";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_GENPROP_LST];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    fn short_repr(&self) -> Option<String> {
        Some(self.class().ok().map_or_else(|| "unknown class".into(), |c| c.into()))
    }
}

impl Debug for PropertyList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_fmt(f)
    }
}

impl Deref for PropertyList {
    type Target = Object;

    fn deref(&self) -> &Object {
        unsafe { self.transmute() }
    }
}

impl PartialEq for PropertyList {
    fn eq(&self, other: &Self) -> bool {
        h5call!(H5Pequal(self.id(), other.id())).unwrap_or(0) == 1
    }
}

impl Eq for PropertyList {}

/// Property list class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyListClass {
    /// Properties for attribute creation.
    AttributeCreate,
    /// Properties for dataset access.
    DatasetAccess,
    /// Properties for dataset creation.
    DatasetCreate,
    /// Properties for raw data transfer.
    DataTransfer,
    /// Properties for datatype access.
    DatatypeAccess,
    /// Properties for datatype creation.
    DatatypeCreate,
    /// Properties for file access.
    FileAccess,
    /// Properties for file creation.
    FileCreate,
    /// Properties for file mounting.
    FileMount,
    /// Properties for group access.
    GroupAccess,
    /// Properties for group creation.
    GroupCreate,
    /// Properties for link traversal when accessing objects.
    LinkAccess,
    /// Properties for link creation.
    LinkCreate,
    /// Properties for object copying process.
    ObjectCopy,
    /// Properties for object creatio.
    ObjectCreate,
    /// Properties for character encoding.
    StringCreate,
}

impl PropertyListClass {
    /// Converts the property list class to a string, e.g. "file create".
    pub fn to_string(self) -> String {
        match self {
            PropertyListClass::AttributeCreate => "attribute create",
            PropertyListClass::DatasetAccess => "dataset access",
            PropertyListClass::DatasetCreate => "dataset create",
            PropertyListClass::DataTransfer => "data transfer",
            PropertyListClass::DatatypeAccess => "datatype access",
            PropertyListClass::DatatypeCreate => "datatype create",
            PropertyListClass::FileAccess => "file access",
            PropertyListClass::FileCreate => "file create",
            PropertyListClass::FileMount => "file mount",
            PropertyListClass::GroupAccess => "group access",
            PropertyListClass::GroupCreate => "group create",
            PropertyListClass::LinkAccess => "link access",
            PropertyListClass::LinkCreate => "link create",
            PropertyListClass::ObjectCopy => "object copy",
            PropertyListClass::ObjectCreate => "object create",
            PropertyListClass::StringCreate => "string create",
        }
        .to_owned()
    }
}

impl Into<String> for PropertyListClass {
    fn into(self) -> String {
        self.to_string()
    }
}

impl FromStr for PropertyListClass {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "attribute create" => Ok(PropertyListClass::AttributeCreate),
            "dataset access" => Ok(PropertyListClass::DatasetAccess),
            "dataset create" => Ok(PropertyListClass::DatasetCreate),
            "data transfer" => Ok(PropertyListClass::DataTransfer),
            "datatype access" => Ok(PropertyListClass::DatatypeAccess),
            "datatype create" => Ok(PropertyListClass::DatatypeCreate),
            "file access" => Ok(PropertyListClass::FileAccess),
            "file create" => Ok(PropertyListClass::FileCreate),
            "file mount" => Ok(PropertyListClass::FileMount),
            "group access" => Ok(PropertyListClass::GroupAccess),
            "group create" => Ok(PropertyListClass::GroupCreate),
            "link access" => Ok(PropertyListClass::LinkAccess),
            "link create" => Ok(PropertyListClass::LinkCreate),
            "object copy" => Ok(PropertyListClass::ObjectCopy),
            "object create" => Ok(PropertyListClass::ObjectCreate),
            "string create" => Ok(PropertyListClass::StringCreate),
            _ => fail!(format!("invalid property list class: {}", s)),
        }
    }
}

#[allow(clippy::len_without_is_empty)]
impl PropertyList {
    /// Copies the property list.
    pub fn copy(&self) -> Self {
        Self::from_id(h5lock!(H5Pcopy(self.id()))).unwrap_or_else(|_| Self::invalid())
    }

    /// Queries whether a property name exists in the property list.
    pub fn has(&self, property: &str) -> bool {
        to_cstring(property)
            .ok()
            .and_then(|property| h5call!(H5Pexist(self.id(), property.as_ptr())).ok())
            .map_or(false, |r| r > 0)
    }

    /// Iterates over properties in the property list, returning their names.
    pub fn properties(&self) -> Vec<String> {
        extern "C" fn callback(_: hid_t, name: *const c_char, data: *mut c_void) -> herr_t {
            let data = unsafe { &mut *(data as *mut Vec<String>) };
            let name = string_from_cstr(name);
            if !name.is_empty() {
                data.push(name);
            }
            0
        }

        let mut data = Vec::new();
        let data_ptr: *mut c_void = &mut data as *mut _ as *mut _;

        h5lock!(H5Piterate(self.id(), ptr::null_mut(), Some(callback), data_ptr));
        data
    }

    /// Returns the current number of properties in the property list.
    pub fn len(&self) -> usize {
        h5get_d!(H5Pget_nprops(self.id()): size_t)
    }

    /// Returns the class of the property list.
    pub fn class(&self) -> Result<PropertyListClass> {
        h5lock!({
            let class_id = h5check(H5Pget_class(self.id()))?;
            let buf = H5Pget_class_name(class_id);
            if buf.is_null() {
                return Err(Error::query().unwrap_or_else(|| "invalid property class".into()));
            }
            let name = string_from_cstr(buf);
            h5_free_memory(buf as _);
            PropertyListClass::from_str(&name)
        })
    }
}

#[cfg(test)]
pub mod tests {
    use hdf5_sys::h5p::H5Pcreate;

    use crate::globals::{H5P_FILE_ACCESS, H5P_FILE_CREATE};
    use crate::internal_prelude::*;

    use super::{PropertyList, PropertyListClass};

    fn make_plists() -> (PropertyList, PropertyList) {
        let fapl = PropertyList::from_id(h5call!(H5Pcreate(*H5P_FILE_ACCESS)).unwrap()).unwrap();
        let fcpl = PropertyList::from_id(h5call!(H5Pcreate(*H5P_FILE_CREATE)).unwrap()).unwrap();
        (fapl, fcpl)
    }

    #[test]
    pub fn test_class() {
        let (fapl, fcpl) = make_plists();
        assert_eq!(fapl.class().unwrap(), PropertyListClass::FileAccess);
        assert_eq!(fcpl.class().unwrap(), PropertyListClass::FileCreate);
    }

    #[test]
    pub fn test_len() {
        let (fapl, fcpl) = make_plists();
        assert!(fapl.len() > 1);
        assert!(fcpl.len() > 1);
        assert_ne!(fapl.len(), fcpl.len());
    }

    #[test]
    pub fn test_eq_ne() {
        let (fapl, fcpl) = make_plists();
        assert_eq!(fapl, fapl);
        assert_eq!(fcpl, fcpl);
        assert_ne!(fapl, fcpl);
    }

    #[test]
    pub fn test_clone() {
        let (fapl, _) = make_plists();
        assert!(fapl.is_valid());
        let fapl_c = fapl.copy();
        assert!(fapl.is_valid());
        assert!(fapl_c.is_valid());
        assert_eq!(fapl.refcount(), 1);
        assert_eq!(fapl_c.refcount(), 1);
        assert_eq!(fapl, fapl_c);
        assert_ne!(fapl.id(), fapl_c.id());
    }

    #[test]
    pub fn test_debug() {
        let (fapl, fcpl) = make_plists();
        assert_eq!(format!("{:?}", fapl), "<HDF5 property list: file access>");
        assert_eq!(format!("{:?}", fcpl), "<HDF5 property list: file create>");
    }
}
