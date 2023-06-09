use std::mem;

use hdf5_sys::h5i::{H5I_type_t, H5Idec_ref, H5Iget_ref, H5Iget_type, H5Iinc_ref, H5Iis_valid};

use crate::internal_prelude::*;

/// A handle to an HDF5 object
#[derive(Debug)]
pub struct Handle {
    id: hid_t,
}

impl Handle {
    /// Create a handle from object ID, taking ownership of it
    pub fn try_new(id: hid_t) -> Result<Self> {
        let handle = Self { id };
        if handle.is_valid_user_id() {
            Ok(handle)
        } else {
            // Drop on an invalid handle could cause closing an unrelated object
            // in the destructor, hence it's important to prevent the drop here.
            mem::forget(handle);
            Err(From::from(format!("Invalid handle id: {id}")))
        }
    }

    /// Create a handle from object ID by cloning it
    pub fn try_borrow(id: hid_t) -> Result<Self> {
        // It's ok to just call try_new() since it may not decref the object
        let handle = Self::try_new(id)?;
        handle.incref();
        Ok(handle)
    }

    pub const fn invalid() -> Self {
        Self { id: H5I_INVALID_HID }
    }

    pub const fn id(&self) -> hid_t {
        self.id
    }

    /// Increment the reference count of the handle
    pub fn incref(&self) {
        if self.is_valid_user_id() {
            h5lock!(H5Iinc_ref(self.id));
        }
    }

    /// Decrease the reference count of the handle
    ///
    /// Note: This function should only be used if `incref` has been
    /// previously called.
    pub fn decref(&self) {
        h5lock!({
            if self.is_valid_id() {
                H5Idec_ref(self.id);
            }
        });
    }

    /// Returns `true` if the object has a valid unlocked identifier (`false` for pre-defined
    /// locked identifiers like property list classes).
    pub fn is_valid_user_id(&self) -> bool {
        h5lock!(H5Iis_valid(self.id)) == 1
    }

    pub fn is_valid_id(&self) -> bool {
        matches!(self.id_type(), tp if tp > H5I_BADID && tp < H5I_NTYPES)
    }

    /// Return the reference count of the object
    pub fn refcount(&self) -> u32 {
        h5call!(H5Iget_ref(self.id)).map(|x| x as _).unwrap_or(0) as _
    }

    /// Get HDF5 object type as a native enum.
    pub fn id_type(&self) -> H5I_type_t {
        if self.id <= 0 {
            H5I_BADID
        } else {
            match h5lock!(H5Iget_type(self.id)) {
                tp if tp > H5I_BADID && tp < H5I_NTYPES => tp,
                _ => H5I_BADID,
            }
        }
    }
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        Self::try_borrow(self.id).unwrap_or_else(|_| Self::invalid())
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        h5lock!(self.decref());
    }
}
