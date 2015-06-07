use ffi::h5i::{H5I_type_t, H5Iget_ref};
use ffi::h5i::H5I_type_t::*;

use handle::{ID, is_valid_user_id, get_id_type};

pub trait Object: ID {
    /// Returns reference count if the handle is valid, 0 otherwise.
    fn refcount(&self) -> u32 {
        if self.is_valid() {
            match h5call!(H5Iget_ref(self.id())) {
                Ok(count) if count >= 0 => count as u32,
                _                       => 0,
            }
        } else {
            0
        }
    }

    fn is_valid(&self) -> bool {
        is_valid_user_id(self.id())
    }

    fn id_type(&self) -> H5I_type_t {
        get_id_type(self.id())
    }

    fn is_file(&self) -> bool          { self.id_type() == H5I_FILE }
    fn is_group(&self) -> bool         { self.id_type() == H5I_GROUP }
    fn is_datatype(&self) -> bool      { self.id_type() == H5I_DATATYPE }
    fn is_dataspace(&self) -> bool     { self.id_type() == H5I_DATASPACE }
    fn is_dataset(&self) -> bool       { self.id_type() == H5I_DATASET }
    fn is_attribute(&self) -> bool     { self.id_type() == H5I_ATTR }
    fn is_reference(&self) -> bool     { self.id_type() == H5I_REFERENCE }
    fn is_vfl(&self) -> bool           { self.id_type() == H5I_VFL }
    fn is_plist_class(&self) -> bool   { self.id_type() == H5I_GENPROP_CLS }
    fn is_plist(&self) -> bool         { self.id_type() == H5I_GENPROP_LST }
    fn is_error_class(&self) -> bool   { self.id_type() == H5I_ERROR_CLASS }
    fn is_error_message(&self) -> bool { self.id_type() == H5I_ERROR_MSG }
    fn is_error_stack(&self) -> bool   { self.id_type() == H5I_ERROR_STACK }
}

#[cfg(test)]
mod tests {
    use ffi::h5i::{H5I_INVALID_HID, hid_t};
    use ffi::h5p::{H5P_DEFAULT, H5Pcreate};
    use globals::H5P_FILE_ACCESS;

    use super::Object;
    use error::Result;
    use handle::{Handle, ID, is_valid_id, is_valid_user_id};

    struct TestObject {
        handle: Handle,
    }

    impl TestObject {
        fn incref(&self) {
            self.handle.incref()
        }

        fn decref(&self) {
            self.handle.decref()
        }
    }

    impl ID for TestObject {
        fn id(&self) -> hid_t {
            self.handle.id()
        }

        fn from_id(id: hid_t) -> Result<TestObject> {
            Ok(TestObject { handle: try!(Handle::new(id)) })
        }
    }

    impl Object for TestObject {}

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

        assert!(!obj.is_file());
        assert!(!obj.is_group());
        assert!(!obj.is_datatype());
        assert!(!obj.is_dataspace());
        assert!(!obj.is_dataset());
        assert!(!obj.is_attribute());
        assert!(!obj.is_reference());
        assert!(!obj.is_vfl());
        assert!(!obj.is_plist_class());
        assert!(obj.is_plist());
        assert!(!obj.is_error_class());
        assert!(!obj.is_error_message());
        assert!(!obj.is_error_stack());

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
        assert!(obj_id != obj.id());
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
