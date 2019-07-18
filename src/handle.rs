use std::collections::HashMap;
use std::sync::Arc;

use lazy_static::lazy_static;
use parking_lot::{Mutex, RwLock};

use hdf5_sys::h5i::{H5I_type_t, H5Idec_ref, H5Iget_type, H5Iinc_ref, H5Iis_valid};

use crate::internal_prelude::*;

pub fn get_id_type(id: hid_t) -> H5I_type_t {
    h5lock!({
        let tp = h5lock!(H5Iget_type(id));
        let valid = id > 0 && tp > H5I_BADID && tp < H5I_NTYPES;
        if valid {
            tp
        } else {
            H5I_BADID
        }
    })
}

pub fn is_valid_id(id: hid_t) -> bool {
    h5lock!({
        let tp = get_id_type(id);
        tp > H5I_BADID && tp < H5I_NTYPES
    })
}

pub fn is_valid_user_id(id: hid_t) -> bool {
    h5lock!({ H5Iis_valid(id) == 1 })
}

struct Registry {
    registry: Mutex<HashMap<hid_t, Arc<RwLock<hid_t>>>>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    pub fn new() -> Self {
        Self { registry: Mutex::new(HashMap::new()) }
    }

    pub fn new_handle(&self, id: hid_t) -> Arc<RwLock<hid_t>> {
        let mut registry = self.registry.lock();
        let handle = registry.entry(id).or_insert_with(|| Arc::new(RwLock::new(id)));
        if *handle.read() != id {
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
    pub fn try_new(id: hid_t) -> Result<Self> {
        lazy_static! {
            static ref REGISTRY: Registry = Registry::new();
        }
        h5lock!({
            if is_valid_user_id(id) {
                Ok(Self { id: REGISTRY.new_handle(id) })
            } else {
                Err(From::from(format!("Invalid handle id: {}", id)))
            }
        })
    }

    pub fn invalid() -> Self {
        Self { id: Arc::new(RwLock::new(H5I_INVALID_HID)) }
    }

    pub fn id(&self) -> hid_t {
        *self.id.read()
    }

    pub fn invalidate(&self) {
        *self.id.write() = H5I_INVALID_HID;
    }

    pub fn incref(&self) {
        if is_valid_user_id(self.id()) {
            h5lock!(H5Iinc_ref(self.id()));
        }
    }

    pub fn decref(&self) {
        h5lock!({
            if self.is_valid_id() {
                H5Idec_ref(self.id());
            }
            // must invalidate all linked IDs because the library reuses them internally
            if !self.is_valid_user_id() && !self.is_valid_id() {
                self.invalidate();
            }
        })
    }

    /// Returns `true` if the object has a valid unlocked identifier (`false` for pre-defined
    /// locked identifiers like property list classes).
    pub fn is_valid_user_id(&self) -> bool {
        is_valid_user_id(self.id())
    }

    pub fn is_valid_id(&self) -> bool {
        is_valid_id(self.id())
    }

    pub fn decref_full(&self) {
        while self.is_valid_user_id() {
            self.decref();
        }
    }
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        h5lock!({
            self.incref();
            Self::try_new(self.id()).unwrap_or_else(|_| Self::invalid())
        })
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        h5lock!(self.decref());
    }
}
