use ffi::h5i::{H5I_GENPROP_LST, hid_t};

use error::Result;
use handle::{Handle, ID, get_id_type};
use object::Object;

pub struct PropertyList {
    handle: Handle,
}

impl ID for PropertyList {
    fn id(&self) -> hid_t {
        self.handle.id()
    }

    fn from_id(id: hid_t) -> Result<PropertyList> {
        match get_id_type(id) {
            H5I_GENPROP_LST => Ok(PropertyList { handle: try!(Handle::new(id)) }),
            _               => Err(From::from(format!("Invalid property list id: {}", id))),
        }
    }
}

impl Object for PropertyList {}
