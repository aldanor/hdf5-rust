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

pub(crate) fn refcount(id: hid_t) -> Result<hsize_t> {
    h5call!(hdf5_sys::h5i::H5Iget_ref(id)).map(|x| x as _)
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

/// A handle to a `hdf5` object
pub struct Handle {
    id: hid_t,
}

impl Handle {
    /// Take ownership of the object id
    pub fn try_new(id: hid_t) -> Result<Self> {
        h5lock!({
            if is_valid_user_id(id) {
                Ok(Self { id })
            } else {
                Err(From::from(format!("Invalid handle id: {}", id)))
            }
        })
    }

    pub fn invalid() -> Self {
        Self { id: H5I_INVALID_HID }
    }

    pub fn id(&self) -> hid_t {
        self.id
    }

    pub fn incref(&self) {
        if is_valid_user_id(self.id()) {
            h5lock!(H5Iinc_ref(self.id()));
        }
    }

    /// An object should not be decreffed unless it has an
    /// associated incref
    pub unsafe fn decref(&self) {
        h5lock!({
            if self.is_valid_id() {
                H5Idec_ref(self.id());
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

    pub(crate) fn refcount(&self) -> u32 {
        refcount(self.id).unwrap_or(0) as u32
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
