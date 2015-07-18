use ffi::h5i::{hid_t, H5I_type_t, H5Iget_type, H5Iis_valid, H5Iinc_ref, H5Idec_ref,
               H5I_INVALID_HID};
use ffi::h5i::H5I_type_t::*;

use error::Result;
use object::Object;

use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;

pub fn get_id_type(id: hid_t) -> H5I_type_t {
    h5lock_s!({
        let tp = h5lock!(H5Iget_type(id));
        let valid = id > 0 && tp > H5I_BADID && tp < H5I_NTYPES;
        if valid { tp } else { H5I_BADID }
    })
}

pub fn is_valid_id(id: hid_t) -> bool {
    h5lock_s!({
        let tp = get_id_type(id);
        tp > H5I_BADID && tp < H5I_NTYPES
    })
}

pub fn is_valid_user_id(id: hid_t) -> bool {
    h5lock!({
        H5Iis_valid(id) == 1
    })
}

pub trait ID: Sized {
    fn id(&self) -> hid_t;
}

pub trait FromID: Sized {
    fn from_id(id: hid_t) -> Result<Self>;
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

pub struct Handle {
    id: Arc<RwLock<hid_t>>,
}

impl Handle {
    pub fn new(id: hid_t) -> Result<Handle> {
        lazy_static! {
            static ref REGISTRY: Registry = Registry::new();
        }
        h5lock_s!({
            match is_valid_user_id(id) {
                false => Err(From::from(format!("Invalid handle id: {}", id))),
                true  => Ok(Handle{ id: REGISTRY.new_handle(id) })
            }
        })
    }

    pub fn invalid() -> Handle {
        Handle { id: Arc::new(RwLock::new(H5I_INVALID_HID)) }
    }

    pub fn id(&self) -> hid_t {
        *self.id.read().unwrap()
    }

    pub fn invalidate(&self) {
        *self.id.write().unwrap() = H5I_INVALID_HID;
    }

    #[allow(dead_code)]  // FIXME: spurious rustc warning
    pub fn incref(&self) {
        if is_valid_user_id(self.id()) {
            h5lock!(H5Iinc_ref(self.id()));
        }
    }

    pub fn decref(&self) {
        h5lock!({
            if is_valid_user_id(self.id()) {
                H5Idec_ref(self.id());
            }
            // must invalidate all linked IDs because the library reuses them internally
            if !is_valid_user_id(self.id()) && !is_valid_id(self.id()) {
                self.invalidate();
            }
        })
    }
}

impl Clone for Handle {
    fn clone(&self) -> Handle {
        h5lock_s!({
            self.incref();
            Handle::from_id(self.id()).unwrap_or(Handle::invalid())
        })
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        h5lock_s!(self.decref());
    }
}

impl ID for Handle {
    fn id(&self) -> hid_t {
        self.id()
    }
}

impl FromID for Handle {
    fn from_id(id: hid_t) -> Result<Handle> {
        Handle::new(id)
    }
}

impl Object for Handle {}
