use ffi::h5i::hid_t;

use handle::{Handle, ID};
use object::Object;
use container::Container;
use location::Location;

#[derive(Clone)]
pub struct Group {
    handle: Handle,
}

impl ID for Group {
    fn id(&self) -> hid_t {
        self.handle.id()
    }

    fn from_id(id: hid_t) -> Group {
        Group { handle: Handle::new(id) }
    }
}

impl Object for Group {}

impl Location for Group {}

impl Container for Group {}

#[cfg(test)]
mod tests {
}
