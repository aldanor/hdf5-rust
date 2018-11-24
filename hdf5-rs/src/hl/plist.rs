use std::ptr;

use libhdf5_sys::h5p::{H5P_iterate_t, H5Pcopy, H5Pequal, H5Pexist, H5Piterate};

use crate::internal_prelude::*;

object_class! {
    /// Represents the HDF5 property list.
    pub struct PropertyList: Object {
        name: "property list",
        types: H5I_GENPROP_LST,
        repr: |_| None,
    }
}

impl Clone for PropertyList {
    fn clone(&self) -> PropertyList {
        let id = h5call!(H5Pcopy(self.id())).unwrap_or(H5I_INVALID_HID);
        PropertyList::from_id(id).unwrap_or(PropertyList { handle: Handle::invalid() })
    }
}

impl PartialEq for PropertyList {
    fn eq(&self, other: &PropertyList) -> bool {
        h5call!(H5Pequal(self.id(), other.id())).unwrap_or(0) == 1
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
}

#[cfg(test)]
pub mod tests {
    use libhdf5_sys::h5p::H5Pcreate;

    use crate::globals::{H5P_FILE_ACCESS, H5P_FILE_CREATE};
    use crate::internal_prelude::*;

    use super::PropertyList;

    #[test]
    pub fn test_clone_eq() {
        let fapl = PropertyList::from_id(h5call!(H5Pcreate(*H5P_FILE_ACCESS)).unwrap()).unwrap();
        let fcpl = PropertyList::from_id(h5call!(H5Pcreate(*H5P_FILE_CREATE)).unwrap()).unwrap();
        assert!(fapl.is_valid());
        assert!(fcpl.is_valid());
        assert_ne!(fapl, fcpl);
        let fapl_c = fapl.clone();
        assert!(fapl.is_valid());
        assert!(fapl_c.is_valid());
        assert_eq!(fapl.refcount(), 1);
        assert_eq!(fapl_c.refcount(), 1);
        assert_eq!(fapl, fapl_c);
        assert_ne!(fapl.id(), fapl_c.id());
        assert_ne!(fcpl, fapl_c);
    }
}
