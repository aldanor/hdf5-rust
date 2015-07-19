use ffi::h5i::{H5I_GENPROP_LST, H5I_INVALID_HID, hid_t};
use ffi::h5p::{H5Pcopy, H5Pequal};

use error::Result;
use handle::{Handle, ID, FromID, get_id_type};
use object::Object;

pub struct PropertyList {
    handle: Handle,
}

#[doc(hidden)]
impl ID for PropertyList {
    fn id(&self) -> hid_t {
        self.handle.id()
    }
}

#[doc(hidden)]
impl FromID for PropertyList {
    fn from_id(id: hid_t) -> Result<PropertyList> {
        match get_id_type(id) {
            H5I_GENPROP_LST => Ok(PropertyList { handle: try!(Handle::new(id)) }),
            _ => Err(From::from(format!("Invalid property list id: {}", id))),
        }
    }
}

impl Object for PropertyList {}

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

#[cfg(test)]
mod tests {
    use super::PropertyList;
    use globals::{H5P_FILE_ACCESS, H5P_FILE_CREATE};
    use ffi::h5p::H5Pcreate;
    use handle::{ID, FromID};
    use object::Object;

    #[test]
    pub fn test_clone_eq() {
        let fapl = PropertyList::from_id(h5call!(H5Pcreate(*H5P_FILE_ACCESS)).unwrap()).unwrap();
        let fcpl = PropertyList::from_id(h5call!(H5Pcreate(*H5P_FILE_CREATE)).unwrap()).unwrap();
        assert!(fapl.is_valid() && fcpl.is_valid());
        assert!(fapl != fcpl);
        let fapl_c = fapl.clone();
        assert!(fapl.is_valid() && fapl.refcount() == 1);
        assert!(fapl_c.is_valid() && fapl_c.refcount() == 1);
        assert!(fapl == fapl_c && fapl.id() != fapl_c.id());
        assert!(fcpl != fapl_c);
    }
}
