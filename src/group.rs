use ffi::h5i::{H5I_GROUP, hid_t};

use error::Result;
use handle::{Handle, ID, get_id_type};
use object::Object;
use container::Container;
use location::Location;

pub struct Group {
    handle: Handle,
}

impl ID for Group {
    fn id(&self) -> hid_t {
        self.handle.id()
    }

    fn from_id(id: hid_t) -> Result<Group> {
        match get_id_type(id) {
            H5I_GROUP => Ok(Group { handle: try!(Handle::new(id)) }),
            _         => Err(From::from(format!("Invalid group id: {}", id))),
        }
    }
}

impl Object for Group {}

impl Location for Group {}

impl Container for Group {}

#[cfg(test)]
mod tests {
}
