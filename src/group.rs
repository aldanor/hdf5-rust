use ffi::h5i::{H5I_GROUP, hid_t};

use error::Result;
use handle::{Handle, ID, FromID, get_id_type};
use object::Object;
use container::Container;
use location::Location;

use std::fmt;

/// Represents the HDF5 group object.
pub struct Group {
    handle: Handle,
}

impl fmt::Debug for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.is_valid() {
            return "<HDF5 group: invalid id>".fmt(f);
        }
        let members = match self.len() {
            0 => "empty".to_string(),
            1 => "1 member".to_string(),
            x => format!("{} members", x),
        };
        format!("<HDF5 group: \"{}\" ({})>", self.name(), members).fmt(f)
    }
}

#[doc(hidden)]
impl ID for Group {
    fn id(&self) -> hid_t {
        self.handle.id()
    }
}

#[doc(hidden)]
impl FromID for Group {
    fn from_id(id: hid_t) -> Result<Group> {
        match get_id_type(id) {
            H5I_GROUP => Ok(Group { handle: try!(Handle::new(id)) }),
            _ => Err(From::from(format!("Invalid group id: {}", id))),
        }
    }
}

impl Object for Group {}

impl Location for Group {}

impl Container for Group {}

#[cfg(test)]
mod tests {
    use container::Container;
    use test::with_tmp_file;

    #[test]
    pub fn test_debug_display() {
        with_tmp_file(|file| {
            file.create_group("a/b/c").unwrap();
            file.create_group("/a/d").unwrap();
            let a = file.group("a").unwrap();
            let ab = file.group("/a/b").unwrap();
            let abc = file.group("./a/b/c/").unwrap();
            assert_eq!(format!("{}", a), "<HDF5 group: \"/a\" (2 members)>");
            assert_eq!(format!("{:?}", a), "<HDF5 group: \"/a\" (2 members)>");
            assert_eq!(format!("{}", ab), "<HDF5 group: \"/a/b\" (1 member)>");
            assert_eq!(format!("{:?}", ab), "<HDF5 group: \"/a/b\" (1 member)>");
            assert_eq!(format!("{}", abc), "<HDF5 group: \"/a/b/c\" (empty)>");
            assert_eq!(format!("{:?}", abc), "<HDF5 group: \"/a/b/c\" (empty)>");
            file.close();
            assert_eq!(format!("{}", a), "<HDF5 group: invalid id>");
            assert_eq!(format!("{:?}", a), "<HDF5 group: invalid id>");
        })
    }
}
