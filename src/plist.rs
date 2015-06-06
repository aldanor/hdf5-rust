use ffi::h5i::hid_t;

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
