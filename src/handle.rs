use hdf5_sys::h5i::{H5I_type_t, H5Idec_ref, H5Iget_ref, H5Iget_type, H5Iinc_ref, H5Iis_valid};

use crate::internal_prelude::*;

pub(crate) fn get_id_type(id: hid_t) -> H5I_type_t {
    if id <= 0 {
        H5I_BADID
    } else {
        match h5lock!(H5Iget_type(id)) {
            tp if tp > H5I_BADID && tp < H5I_NTYPES => tp,
            _ => H5I_BADID,
        }
    }
}

pub(crate) fn refcount(id: hid_t) -> Result<hsize_t> {
    h5call!(H5Iget_ref(id)).map(|x| x as _)
}

pub fn is_valid_id(id: hid_t) -> bool {
    match h5lock!(get_id_type(id)) {
        tp if tp > H5I_BADID && tp < H5I_NTYPES => true,
        _ => false,
    }
}

pub fn is_valid_user_id(id: hid_t) -> bool {
    h5lock!({ H5Iis_valid(id) == 1 })
}

/// A handle to an HDF5 object
#[derive(Debug)]
pub struct Handle {
    id: hid_t,
}

impl Handle {
    /// Create a handle from object ID, taking ownership of it
    pub fn try_new(id: hid_t) -> Result<Self> {
        h5lock!({
            if is_valid_user_id(id) {
                Ok(Self { id })
            } else {
                Err(From::from(format!("Invalid handle id: {}", id)))
            }
        })
    }

    /// Create a handle from object ID by cloning it
    pub fn try_borrow(id: hid_t) -> Result<Self> {
        h5lock!({
            if is_valid_user_id(id) {
                h5call!(H5Iinc_ref(id))?;
                Ok(Self { id })
            } else {
                Err(From::from(format!("Invalid handle id: {}", id)))
            }
        })
    }

    pub const fn invalid() -> Self {
        Self { id: H5I_INVALID_HID }
    }

    pub const fn id(&self) -> hid_t {
        self.id
    }

    /// Increment the reference count of the handle
    pub fn incref(&self) {
        if is_valid_user_id(self.id()) {
            h5lock!(H5Iinc_ref(self.id()));
        }
    }

    /// Decrease the reference count of the handle
    ///
    /// Note: This function should only be used if `incref` has been
    /// previously called.
    pub fn decref(&self) {
        h5lock!({
            if self.is_valid_id() {
                H5Idec_ref(self.id());
            }
        });
    }

    /// Returns `true` if the object has a valid unlocked identifier (`false` for pre-defined
    /// locked identifiers like property list classes).
    pub fn is_valid_user_id(&self) -> bool {
        is_valid_user_id(self.id())
    }

    pub fn is_valid_id(&self) -> bool {
        is_valid_id(self.id())
    }

    /// Return the reference count of the object
    pub fn refcount(&self) -> u32 {
        refcount(self.id).unwrap_or(0) as _
    }

    /// Get HDF5 object type as a native enum.
    pub fn id_type(&self) -> H5I_type_t {
        get_id_type(self.id)
    }
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        Self::try_borrow(self.id()).unwrap_or_else(|_| Self::invalid())
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        h5lock!(self.decref());
    }
}
