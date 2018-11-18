use std::fmt;

use libhdf5_sys::h5i::H5I_type_t;

use crate::internal_prelude::*;

/// Represents the HDF5 group object.
define_object_type!(Group: Container, "group", |id_type| id_type == H5I_type_t::H5I_GROUP);

impl fmt::Debug for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.is_valid() {
            f.write_str("<HDF5 group: invalid id>")
        } else {
            let members = match self.len() {
                0 => "empty".to_owned(),
                1 => "1 member".to_owned(),
                x => format!("{} members", x),
            };
            write!(f, "<HDF5 group: \"{}\" ({})>", self.name(), members)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::internal_prelude::*;

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
