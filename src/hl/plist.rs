use std::fmt::{self, Debug, Display};
use std::ops::Deref;
use std::panic;
use std::ptr::{self, addr_of_mut};
use std::str::FromStr;

use hdf5_sys::h5p::{
    H5Pcopy, H5Pequal, H5Pexist, H5Pget_class, H5Pget_class_name, H5Pget_nprops, H5Pisa_class,
    H5Piterate, H5Pset_vlen_mem_manager,
};

use crate::internal_prelude::*;

pub mod common;
pub mod dataset_access;
pub mod dataset_create;
pub mod file_access;
pub mod file_create;
pub mod link_create;

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
        Some(self.class().ok().map_or_else(|| "unknown class".into(), Into::into))
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

impl Display for PropertyListClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::AttributeCreate => "attribute create",
            Self::DatasetAccess => "dataset access",
            Self::DatasetCreate => "dataset create",
            Self::DataTransfer => "data transfer",
            Self::DatatypeAccess => "datatype access",
            Self::DatatypeCreate => "datatype create",
            Self::FileAccess => "file access",
            Self::FileCreate => "file create",
            Self::FileMount => "file mount",
            Self::GroupAccess => "group access",
            Self::GroupCreate => "group create",
            Self::LinkAccess => "link access",
            Self::LinkCreate => "link create",
            Self::ObjectCopy => "object copy",
            Self::ObjectCreate => "object create",
            Self::StringCreate => "string create",
        })
    }
}

impl From<PropertyListClass> for String {
    fn from(v: PropertyListClass) -> Self {
        format!("{v}")
    }
}

impl FromStr for PropertyListClass {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "attribute create" => Ok(Self::AttributeCreate),
            "dataset access" => Ok(Self::DatasetAccess),
            "dataset create" => Ok(Self::DatasetCreate),
            "data transfer" => Ok(Self::DataTransfer),
            "datatype access" => Ok(Self::DatatypeAccess),
            "datatype create" => Ok(Self::DatatypeCreate),
            "file access" => Ok(Self::FileAccess),
            "file create" => Ok(Self::FileCreate),
            "file mount" => Ok(Self::FileMount),
            "group access" => Ok(Self::GroupAccess),
            "group create" => Ok(Self::GroupCreate),
            "link access" => Ok(Self::LinkAccess),
            "link create" => Ok(Self::LinkCreate),
            "object copy" => Ok(Self::ObjectCopy),
            "object create" => Ok(Self::ObjectCreate),
            "string create" => Ok(Self::StringCreate),
            _ => fail!(format!("invalid property list class: {s}")),
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
            panic::catch_unwind(|| {
                let data = unsafe { &mut *(data.cast::<Vec<String>>()) };
                let name = string_from_cstr(name);
                if !name.is_empty() {
                    data.push(name);
                }
                0
            })
            .unwrap_or(-1)
        }

        let mut data = Vec::new();
        let data_ptr: *mut c_void = addr_of_mut!(data).cast();

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
                return Err(Error::query().unwrap_or_else(|_| "invalid property class".into()));
            }
            let name = string_from_cstr(buf);
            h5_free_memory(buf.cast());
            PropertyListClass::from_str(&name)
        })
    }

    pub fn is_class(&self, class: PropertyListClass) -> bool {
        use crate::globals::*;
        h5lock!({
            let class = match class {
                PropertyListClass::FileCreate => *H5P_FILE_CREATE,
                PropertyListClass::AttributeCreate => *H5P_ATTRIBUTE_CREATE,
                PropertyListClass::DatasetAccess => *H5P_DATASET_ACCESS,
                PropertyListClass::DatasetCreate => *H5P_DATASET_CREATE,
                PropertyListClass::DataTransfer => *H5P_DATASET_XFER,
                PropertyListClass::DatatypeAccess => *H5P_DATATYPE_ACCESS,
                PropertyListClass::DatatypeCreate => *H5P_DATATYPE_CREATE,
                PropertyListClass::FileAccess => *H5P_FILE_ACCESS,
                PropertyListClass::FileMount => *H5P_FILE_MOUNT,
                PropertyListClass::GroupAccess => *H5P_GROUP_ACCESS,
                PropertyListClass::GroupCreate => *H5P_GROUP_CREATE,
                PropertyListClass::LinkAccess => *H5P_LINK_ACCESS,
                PropertyListClass::LinkCreate => *H5P_LINK_CREATE,
                PropertyListClass::ObjectCopy => *H5P_OBJECT_COPY,
                PropertyListClass::ObjectCreate => *H5P_OBJECT_CREATE,
                PropertyListClass::StringCreate => *H5P_STRING_CREATE,
            };
            let tri = H5Pisa_class(self.id(), class);

            tri == 1
        })
    }
}

/// Set the memory manager for variable length items to
/// the same allocator as is in use by hdf5-types
// TODO: move this to dataset_transfer module when DatasetTransfer plist is implemented
pub fn set_vlen_manager_libc(plist: hid_t) -> Result<()> {
    extern "C" fn alloc(size: size_t, _info: *mut c_void) -> *mut c_void {
        panic::catch_unwind(|| unsafe { libc::malloc(size) }).unwrap_or(ptr::null_mut())
    }
    extern "C" fn free(ptr: *mut c_void, _info: *mut libc::c_void) {
        let _p = panic::catch_unwind(|| unsafe {
            libc::free(ptr);
        });
    }
    h5try!(H5Pset_vlen_mem_manager(
        plist,
        Some(alloc),
        ptr::null_mut(),
        Some(free),
        ptr::null_mut()
    ));
    Ok(())
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
