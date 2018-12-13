use std::fmt::{self, Debug};
use std::ops::Deref;
use std::ptr;

use libhdf5_sys::h5p::{
    H5Pcopy, H5Pequal, H5Pexist, H5Pget_class, H5Pget_class_name, H5Pget_nprops, H5Piterate,
};

use crate::internal_prelude::*;

/// Represents the HDF5 property list.
#[repr(transparent)]
pub struct PropertyList(Handle);

impl ObjectClass for PropertyList {
    const NAME: &'static str = "property list";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_GENPROP_LST];

    fn from_handle(handle: Handle) -> Self {
        PropertyList(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    // TODO: short_repr()
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

impl Clone for PropertyList {
    fn clone(&self) -> PropertyList {
        let id = h5call!(H5Pcopy(self.id())).unwrap_or(H5I_INVALID_HID);
        PropertyList::from_id(id).ok().unwrap_or_else(PropertyList::invalid)
    }
}

impl PartialEq for PropertyList {
    fn eq(&self, other: &PropertyList) -> bool {
        h5call!(H5Pequal(self.id(), other.id())).unwrap_or(0) == 1
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyListClass {
    AttributeCreate,
    DatasetAccess,
    DatasetCreate,
    DataTransfer,
    DatatypeAccess,
    DatatypeCreate,
    FileAccess,
    FileCreate,
    FileMount,
    GroupAccess,
    GroupCreate,
    LinkAccess,
    LinkCreate,
    ObjectCopy,
    ObjectCreate,
    StringCreate,
}

impl PropertyListClass {
    pub fn to_string(&self) -> String {
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

    pub fn from_str(class_name: &str) -> Result<PropertyListClass> {
        match class_name {
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
            _ => fail!(format!("invalid property list class: {}", class_name)),
        }
    }
}

impl Into<String> for PropertyListClass {
    fn into(self) -> String {
        self.to_string()
    }
}

impl PropertyList {
    pub fn has(&self, property: &str) -> bool {
        to_cstring(property)
            .ok()
            .and_then(|property| h5call!(H5Pexist(self.id(), property.as_ptr())).ok())
            .map(|r| r > 0)
            .unwrap_or(false)
    }

    pub fn properties(&self) -> Vec<String> {
        extern "C" fn callback(_: hid_t, name: *const c_char, data: *mut c_void) -> herr_t {
            unsafe {
                let data = &mut *(data as *mut Vec<String>);
                let name = string_from_cstr(name);
                if !name.is_empty() {
                    data.push(name);
                }
                0
            }
        }

        let mut data = Vec::new();
        let data_ptr: *mut c_void = &mut data as *mut _ as *mut _;

        h5lock!(H5Piterate(self.id(), ptr::null_mut(), Some(callback), data_ptr));
        data
    }

    pub fn len(&self) -> usize {
        h5get_d!(H5Pget_nprops(self.id()): size_t)
    }

    pub fn class(&self) -> Result<PropertyListClass> {
        h5lock!({
            let class_id = h5check(H5Pget_class(self.id()))?;
            let buf = H5Pget_class_name(class_id);
            if buf.is_null() {
                return Err(Error::query().unwrap_or_else(|| "invalid property class".into()));
            }
            let name = string_from_cstr(buf);
            libc::free(buf as _);
            PropertyListClass::from_str(&name)
        })
    }
}

#[cfg(test)]
pub mod tests {
    use libhdf5_sys::h5p::H5Pcreate;

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
        let fapl_c = fapl.clone();
        assert!(fapl.is_valid());
        assert!(fapl_c.is_valid());
        assert_eq!(fapl.refcount(), 1);
        assert_eq!(fapl_c.refcount(), 1);
        assert_eq!(fapl, fapl_c);
        assert_ne!(fapl.id(), fapl_c.id());
    }
}
