use ffi::h5g::{H5G_info_t, H5Gget_info, H5Gcreate2, H5Gopen2};
use ffi::h5l::{H5Lmove, H5L_SAME_LOC};
use ffi::h5i::hid_t;
use ffi::h5p::{H5Pcreate, H5Pset_create_intermediate_group, H5P_DEFAULT};
use globals::H5P_LINK_CREATE;

use error::Result;
use group::Group;
use location::Location;
use object::Object;
use plist::PropertyList;
use util::to_cstring;

use std::default::Default;

fn group_info(id: hid_t) -> Result<H5G_info_t> {
    let info: *mut H5G_info_t = &mut H5G_info_t::default();
    h5call!(H5Gget_info(id, info)).and(Ok(unsafe { *info }))
}

fn make_lcpl() -> Result<PropertyList> {
    h5lock_s!({
        let lcpl = PropertyList::from_id(h5try!(H5Pcreate(*H5P_LINK_CREATE)));
        h5call!(H5Pset_create_intermediate_group(lcpl.id(), 1)).and(Ok(lcpl))
    })
}

pub trait Container: Location {
    /// Returns the number of objects in the container (or 0 if the container is invalid).
    fn len(&self) -> u64 {
        group_info(self.id()).map(|info| info.nlinks).unwrap_or(0)
    }

    /// Returns true if the container has no linked objects (or if the container is invalid).
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Create a new group in a container which can be a file or another group.
    fn create_group<S: Into<String>>(&self, name: S) -> Result<Group> {
        h5lock_s!({
            let lcpl = try!(make_lcpl());
            Ok(Group::from_id(h5try!(H5Gcreate2(
                self.id(), to_cstring(name).as_ptr(), lcpl.id(), H5P_DEFAULT, H5P_DEFAULT))))
        })
    }

    /// Opens an existing group in a container which can be a file or another group.
    fn group<S: Into<String>>(&self, name: S) -> Result<Group> {
        Ok(Group::from_id(h5try!(H5Gopen2(
            self.id(), to_cstring(name).as_ptr(), H5P_DEFAULT))))
    }

    /// Relinks an object. Note: `from` and `to` are relative to the current object.
    fn relink<S1: Into<String>, S2: Into<String>>(&self, name: S1, path: S2) -> Result<()> {
        h5call!(H5Lmove(self.id(), to_cstring(name).as_ptr(), H5L_SAME_LOC,
                        to_cstring(path).as_ptr(), H5P_DEFAULT, H5P_DEFAULT)).and(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use error::silence_errors;
    use test::with_tmp_file;
    use super::Container;
    use location::Location;
    use object::Object;

    #[test]
    pub fn test_group() {
        silence_errors();
        with_tmp_file(|file| {
            assert_err!(file.group("a"), "unable to open group: object.+doesn't exist");
            file.create_group("a").unwrap();
            let a = file.group("a").unwrap();
            assert!(a.name() == "/a");
            assert!(a.file().id() == file.id());
            a.create_group("b").unwrap();
            let b = file.group("/a/b").unwrap();
            assert!(b.name() == "/a/b");
            assert!(b.file().id() == file.id());
            file.create_group("/foo/bar").unwrap();
            file.group("foo").unwrap().group("bar").unwrap();
            file.create_group("x/y/").unwrap();
            file.group("/x").unwrap().group("./y/").unwrap();
        })
    }

    #[test]
    pub fn test_len() {
        with_tmp_file(|file| {
            assert_eq!(file.len(), 0);
            assert!(file.is_empty());
            file.create_group("foo").unwrap();
            assert_eq!(file.len(), 1);
            assert!(!file.is_empty());
            assert_eq!(file.group("foo").unwrap().len(), 0);
            assert!(file.group("foo").unwrap().is_empty());
            file.create_group("bar").unwrap().create_group("baz").unwrap();
            assert_eq!(file.len(), 2);
            assert_eq!(file.group("bar").unwrap().len(), 1);
            assert_eq!(file.group("/bar/baz").unwrap().len(), 0);
        })
    }

    #[test]
    pub fn test_relink() {
        silence_errors();
        with_tmp_file(|file| {
            file.create_group("test").unwrap();
            file.group("test").unwrap();
            assert_err!(file.relink("test", "foo/test"),
                        "unable to move link: component not found");
            file.create_group("foo").unwrap();
            assert_err!(file.relink("bar", "/baz"),
                        "unable to move link: name doesn't exist");
            file.relink("test", "/foo/test").unwrap();
            file.group("/foo/test").unwrap();
            assert_err!(file.group("test"),
                        "unable to open group: object.+doesn't exist");
        })
    }
}
