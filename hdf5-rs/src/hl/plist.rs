use libhdf5_sys::h5p::{H5Pcopy, H5Pequal};

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
