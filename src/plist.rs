use ffi::types::hid_t;
use ffi::h5p::H5Pclose;

use object::{Handle, Object};

#[derive(Clone)]
pub struct PropertyList {
    handle: Handle,
}

impl Object for PropertyList {
    fn id(&self) -> hid_t {
        self.handle.id()
    }

    fn from_id(id: hid_t) -> PropertyList {
        PropertyList { handle: Handle::new(id) }
    }
}

impl Drop for PropertyList {
    fn drop(&mut self) {
        if self.refcount() == 1 {
            h5lock!(H5Pclose(self.id()));
        }
    }
}
