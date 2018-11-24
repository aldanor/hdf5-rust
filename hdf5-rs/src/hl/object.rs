use libhdf5_sys::h5i::H5Iget_ref;

use crate::internal_prelude::*;

object_class! {
    /// Any HDF5 object that can be referenced through an identifier.
    pub struct Object {
        name: "object",
        types: None,
        repr: |_| None,
    }
}

impl Object {
    pub(crate) fn id(&self) -> hid_t {
        self.handle.id()
    }

    /// Returns reference count if the handle is valid and 0 otherwise.
    pub fn refcount(&self) -> u32 {
        if self.is_valid() {
            h5call!(H5Iget_ref(self.id())).unwrap_or(0) as _
        } else {
            0
        }
    }

    /// Returns `true` if the object has a valid unlocked identifier (`false` for pre-defined
    /// locked identifiers like property list classes).
    pub fn is_valid(&self) -> bool {
        is_valid_user_id(self.id())
    }

    /// Returns type of the object.
    pub fn id_type(&self) -> H5I_type_t {
        get_id_type(self.id())
    }
}

#[cfg(test)]
pub mod tests {
    use libhdf5_sys::{h5i::H5I_type_t, h5p::H5Pcreate};

    use crate::globals::H5P_FILE_ACCESS;
    use crate::handle::{is_valid_id, is_valid_user_id};
    use crate::internal_prelude::*;

    object_class! {
        pub struct TestObject: Object {
            name: "test object",
            types: None,
            repr: |_| None,
        }
    }

    impl TestObject {
        fn incref(&self) {
            self.handle.incref()
        }

        fn decref(&self) {
            self.handle.decref()
        }
    }

    #[test]
    pub fn test_not_a_valid_user_id() {
        assert_err!(TestObject::from_id(H5I_INVALID_HID), "Invalid handle id");
        assert_err!(TestObject::from_id(H5P_DEFAULT), "Invalid handle id");
    }

    #[test]
    pub fn test_new_user_id() {
        let obj = TestObject::from_id(h5call!(H5Pcreate(*H5P_FILE_ACCESS)).unwrap()).unwrap();
        assert!(obj.id() > 0);
        assert!(obj.is_valid());
        assert!(is_valid_id(obj.id()));
        assert!(is_valid_user_id(obj.id()));
        assert_eq!(obj.id_type(), H5I_type_t::H5I_GENPROP_LST);

        assert_eq!(obj.refcount(), 1);
        obj.incref();
        assert_eq!(obj.refcount(), 2);
        obj.decref();
        assert_eq!(obj.refcount(), 1);
        obj.decref();
        obj.decref();
        assert_eq!(obj.refcount(), 0);
        assert!(!obj.is_valid());
        assert!(!is_valid_user_id(obj.id()));
        assert!(!is_valid_id(obj.id()));
    }

    #[test]
    pub fn test_incref_decref_drop() {
        let mut obj = TestObject::from_id(h5call!(H5Pcreate(*H5P_FILE_ACCESS)).unwrap()).unwrap();
        let obj_id = obj.id();
        obj = TestObject::from_id(h5call!(H5Pcreate(*H5P_FILE_ACCESS)).unwrap()).unwrap();
        assert_ne!(obj_id, obj.id());
        assert!(obj.id() > 0);
        assert!(obj.is_valid());
        assert!(is_valid_id(obj.id()));
        assert!(is_valid_user_id(obj.id()));
        assert_eq!(obj.refcount(), 1);
        let mut obj2 = TestObject::from_id(obj.id()).unwrap();
        obj2.incref();
        assert_eq!(obj.refcount(), 2);
        assert_eq!(obj2.refcount(), 2);
        drop(obj2);
        assert!(obj.is_valid());
        assert_eq!(obj.refcount(), 1);
        obj2 = TestObject::from_id(obj.id()).unwrap();
        obj2.incref();
        obj.decref();
        obj.decref();
        assert_eq!(obj.id(), H5I_INVALID_HID);
        assert_eq!(obj2.id(), H5I_INVALID_HID);
    }
}
