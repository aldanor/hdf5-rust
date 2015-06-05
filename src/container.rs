use ffi::h5g::{H5G_info_t, H5Gget_info, H5Gcreate2, H5Gopen2};
use ffi::h5l::{H5Lmove, H5L_SAME_LOC};
use ffi::h5p::H5P_DEFAULT;

use error::Result;
use group::Group;
use location::Location;
use object::Object;
use util::string_to_cstr;

use std::default::Default;

pub trait Container: Location {
    #[doc(hidden)]
    fn group_info(&self) -> Result<H5G_info_t> {
        let info: *mut H5G_info_t = &mut H5G_info_t::default();
        h5call!(H5Gget_info(self.id(), info)).and(Ok(unsafe { *info }))
    }

    /// Returns the number of objects in the container (0 if the container is invalid).
    fn len(&self) -> u64 {
        self.group_info().map(|info| info.nlinks).unwrap_or(0)
    }

    /// Create a new group in a container which can be a file or another group.
    fn create_group<S: Into<String>>(&self, name: S) -> Result<Group> {
        Ok(Group::from_id(h5try!(H5Gcreate2(
            self.id(), string_to_cstr(name), H5P_DEFAULT, 0, H5P_DEFAULT))))
    }

    /// Opens an existing group in a container which can be a file or another group.
    fn group<S: Into<String>>(&self, name: S) -> Result<Group> {
        Ok(Group::from_id(h5try!(H5Gopen2(
            self.id(), string_to_cstr(name), H5P_DEFAULT))))
    }

    /// Relinks an object. Note: `from` and `to` are relative to the current object.
    fn relink<S1: Into<String>, S2: Into<String>>(&self, name: S1, path: S2) -> Result<()> {
        h5call!(H5Lmove(self.id(), string_to_cstr(name), H5L_SAME_LOC,
                        string_to_cstr(path), H5P_DEFAULT, H5P_DEFAULT)).and(Ok(()))
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
            assert_err!(file.group("a"), "unable to open group: object 'a' doesn't exist");
            file.create_group("a");
            let a = file.group("a").unwrap();
            assert!(a.name() == "/a");
            assert!(a.file().id() == file.id());
            a.create_group("b");
            let b = file.group("/a/b").unwrap();
            assert!(b.name() == "/a/b");
            assert!(b.file().id() == file.id());
        })
    }

    #[test]
    pub fn test_len() {
        with_tmp_file(|file| {
            assert_eq!(file.len(), 0);
            file.create_group("foo").unwrap();
            assert_eq!(file.len(), 1);
            assert_eq!(file.group("foo").unwrap().len(), 0);
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
                        "unable to open group: object 'test' doesn't exist");
        })
    }
}
