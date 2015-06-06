use ffi::h5i::{hid_t, H5I_type_t, H5Iget_type, H5Iis_valid, H5Iinc_ref, H5Idec_ref,
               H5Iget_ref, H5I_INVALID_HID};
use ffi::h5i::H5I_type_t::*;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

fn get_id_type(id: hid_t) -> H5I_type_t {
    let tp = h5lock!(H5Iget_type(id));
    let valid = id > 0 && tp > H5I_BADID && tp < H5I_NTYPES;
    if valid { tp } else { H5I_BADID }
}

fn is_valid_id(id: hid_t) -> bool {
    let tp = get_id_type(id);
    tp > H5I_BADID && tp < H5I_NTYPES
}

fn is_valid_user_id(id: hid_t) -> bool {
    h5lock!(H5Iis_valid(id)) == 1
}

pub struct Handle {
    id: Arc<RwLock<hid_t>>,
}

impl Handle {
    pub fn new(id: hid_t) -> Handle {
        lazy_static! {
            static ref REGISTRY: Registry = Registry::new();
        }
        Handle {
            // if the id is not registered with the library, do not share it
            id: match is_valid_user_id(id) {
                false => Arc::new(RwLock::new(id)),
                true  => REGISTRY.new_handle(id),
            }
        }
    }

    pub fn id(&self) -> hid_t {
        *self.id.read().unwrap()
    }

    pub fn invalidate(&self) {
        *self.id.write().unwrap() = H5I_INVALID_HID;
    }

    fn incref(&self) {
        if self.is_valid() {
            h5lock!(H5Iinc_ref(self.id()));
        }
    }

    fn decref(&self) {
        h5lock!({
            if self.is_valid() {
                H5Idec_ref(self.id());
            }
            // must invalidate all linked IDs because the library reuses them internally
            if !self.is_valid() && !is_valid_id(self.id()) {
                self.invalidate();
            }
        })
    }
}

impl Clone for Handle {
    fn clone(&self) -> Handle {
        self.incref();
        Handle::new(self.id())
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        h5lock_s!(self.decref());
    }
}

struct Registry {
    registry: Mutex<HashMap<hid_t, Arc<RwLock<hid_t>>>>,
}

impl Registry {
    pub fn new() -> Registry {
        Registry { registry: Mutex::new(HashMap::new()) }
    }

    pub fn new_handle(&self, id: hid_t) -> Arc<RwLock<hid_t>> {
        let mut registry = self.registry.lock().unwrap();
        let handle = registry.entry(id).or_insert(Arc::new(RwLock::new(id)));
        if *handle.read().unwrap() != id {
            // an id may be left dangling by previous invalidation of a linked handle
            *handle = Arc::new(RwLock::new(id));
        }
        handle.clone()
    }
}

pub trait Object {
    fn id(&self) -> hid_t;
    fn from_id(id: hid_t) -> Self;

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

impl Object for Handle {
    fn id(&self) -> hid_t {
        self.id()
    }

    fn from_id(id: hid_t) -> Handle {
        Handle::new(id)
    }
}

#[test]
pub fn test_handle() {
    use ffi::h5i::H5I_INVALID_HID;
    use ffi::h5p::H5Pcreate;
    use globals::{H5P_ROOT, H5P_FILE_ACCESS};

    #[derive(Clone)]
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

    impl Object for TestObject {
        fn id(&self) -> hid_t {
            self.handle.id()
        }

        fn from_id(id: hid_t) -> TestObject {
            TestObject { handle: Handle::new(id) }
        }
    }

    // invalid id
    let mut obj = TestObject::from_id(H5I_INVALID_HID);
    assert_eq!(obj.id(), H5I_INVALID_HID);
    assert!(!obj.is_valid());
    assert!(!is_valid_id(obj.id()));
    assert!(!is_valid_user_id(obj.id()));
    assert_eq!(obj.id_type(), H5I_BADID);
    assert_eq!(obj.refcount(), 0);

    // existing generic id
    obj = TestObject::from_id(*H5P_ROOT);
    assert_eq!(obj.id(), *H5P_ROOT);
    assert!(is_valid_id(obj.id()));
    assert!(!is_valid_user_id(obj.id()));
    assert!(!obj.is_valid());
    assert!(obj.is_plist_class());
    assert_eq!(obj.refcount(), 0);
    obj.decref();
    assert!(is_valid_id(obj.id()));

    // new user id
    obj = TestObject::from_id(h5call!(H5Pcreate(*H5P_FILE_ACCESS)).unwrap());
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

    // cloning and dropping
    obj = TestObject::from_id(h5call!(H5Pcreate(*H5P_FILE_ACCESS)).unwrap());
    let obj_id = obj.id();
    obj = TestObject::from_id(h5call!(H5Pcreate(*H5P_FILE_ACCESS)).unwrap());
    assert!(!is_valid_id(obj_id));
    assert!(!is_valid_user_id(obj_id));
    assert!(obj.id() > 0);
    assert!(obj.is_valid());
    assert!(is_valid_id(obj.id()));
    assert!(is_valid_user_id(obj.id()));
    assert_eq!(obj.refcount(), 1);
    let mut obj2 = obj.clone();
    assert_eq!(obj.refcount(), 2);
    assert_eq!(obj2.refcount(), 2);
    drop(obj2);
    assert!(obj.is_valid());
    assert_eq!(obj.refcount(), 1);
    obj2 = obj.clone();
    obj.decref();
    obj.decref();
    assert_eq!(obj.id(), H5I_INVALID_HID);
    assert_eq!(obj2.id(), H5I_INVALID_HID);
}
