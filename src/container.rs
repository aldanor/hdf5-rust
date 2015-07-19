use ffi::h5d::H5Dopen2;
use ffi::h5g::{H5G_info_t, H5Gget_info, H5Gcreate2, H5Gopen2};
use ffi::h5i::hid_t;
use ffi::h5l::{H5Lmove, H5Lcreate_soft, H5Lcreate_hard, H5Ldelete, H5L_SAME_LOC};
use ffi::h5p::{H5Pcreate, H5Pset_create_intermediate_group, H5P_DEFAULT};
use globals::H5P_LINK_CREATE;

use dataset::{Dataset, DatasetBuilder};
use datatype::ToDatatype;
use error::Result;
use group::Group;
use handle::{ID, FromID};
use location::Location;
use plist::PropertyList;
use util::to_cstring;

use std::default::Default;

fn group_info(id: hid_t) -> Result<H5G_info_t> {
    let info: *mut H5G_info_t = &mut H5G_info_t::default();
    h5call!(H5Gget_info(id, info)).and(Ok(unsafe { *info }))
}

fn make_lcpl() -> Result<PropertyList> {
    h5lock_s!({
        let lcpl = try!(PropertyList::from_id(h5try!(H5Pcreate(*H5P_LINK_CREATE))));
        h5call!(H5Pset_create_intermediate_group(lcpl.id(), 1)).and(Ok(lcpl))
    })
}

/// A trait for HDF5 objects that can contain other objects (file, group).
pub trait Container: Location {
    /// Returns the number of objects in the container (or 0 if the container is invalid).
    fn len(&self) -> u64 {
        group_info(self.id()).map(|info| info.nlinks).unwrap_or(0)
    }

    /// Returns true if the container has no linked objects (or if the container is invalid).
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Create a new group in a file or group.
    fn create_group<S: Into<String>>(&self, name: S) -> Result<Group> {
        h5lock_s!({
            let lcpl = try!(make_lcpl());
            Group::from_id(h5try!(H5Gcreate2(
                self.id(), to_cstring(name).as_ptr(), lcpl.id(), H5P_DEFAULT, H5P_DEFAULT
            )))
        })
    }

    /// Opens an existing group in a file or group.
    fn group<S: Into<String>>(&self, name: S) -> Result<Group> {
        Group::from_id(h5try!(H5Gopen2(
            self.id(), to_cstring(name).as_ptr(), H5P_DEFAULT)))
    }

    /// Creates a soft link. Note: `name` and `path` are relative to the current object.
    fn link_soft<S1: Into<String>, S2: Into<String>>(&self, name: S1, path: S2) -> Result<()> {
        h5lock_s!({
            let lcpl = try!(make_lcpl());
            h5call!(H5Lcreate_soft(
                to_cstring(name).as_ptr(), self.id(),
                to_cstring(path).as_ptr(), lcpl.id(), H5P_DEFAULT
            )).and(Ok(()))
        })
    }

    /// Creates a hard link. Note: `name` and `path` are relative to the current object.
    fn link_hard<S1: Into<String>, S2: Into<String>>(&self, name: S1, path: S2) -> Result<()> {
        h5call!(H5Lcreate_hard(
            self.id(), to_cstring(name).as_ptr(), H5L_SAME_LOC,
            to_cstring(path).as_ptr(), H5P_DEFAULT, H5P_DEFAULT
        )).and(Ok(()))
    }

    /// Relinks an object. Note: `name` and `path` are relative to the current object.
    fn relink<S1: Into<String>, S2: Into<String>>(&self, name: S1, path: S2) -> Result<()> {
        h5call!(H5Lmove(
            self.id(), to_cstring(name).as_ptr(), H5L_SAME_LOC,
            to_cstring(path).as_ptr(), H5P_DEFAULT, H5P_DEFAULT
        )).and(Ok(()))
    }

    /// Removes a link to an object from this file or group.
    fn unlink<S: Into<String>>(&self, name: S) -> Result<()> {
        h5call!(H5Ldelete(
            self.id(), to_cstring(name).as_ptr(), H5P_DEFAULT
        )).and(Ok(()))
    }

    /// Instantiates a new dataset builder.
    fn new_dataset<T: ToDatatype>(&self) -> DatasetBuilder<T> {
        DatasetBuilder::<T>::new::<Self>(&self)
    }

    /// Opens an existing dataset in the file or group.
    fn dataset<S: Into<String>>(&self, name: S) -> Result<Dataset> {
        Dataset::from_id(h5try!(H5Dopen2(
            self.id(), to_cstring(name).as_ptr(), H5P_DEFAULT)))
    }
}

#[cfg(test)]
mod tests {
    use error::silence_errors;
    use handle::ID;
    use test::with_tmp_file;
    use super::Container;
    use location::Location;

    #[test]
    pub fn test_group() {
        silence_errors();
        with_tmp_file(|file| {
            assert_err!(file.group("a"), "unable to open group: object.+doesn't exist");
            file.create_group("a").unwrap();
            let a = file.group("a").unwrap();
            assert!(a.name() == "/a");
            assert!(a.file().unwrap().id() == file.id());
            a.create_group("b").unwrap();
            let b = file.group("/a/b").unwrap();
            assert!(b.name() == "/a/b");
            assert!(b.file().unwrap().id() == file.id());
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
    pub fn test_link_hard() {
        silence_errors();
        with_tmp_file(|file| {
            file.create_group("foo/test/inner").unwrap();
            file.link_hard("/foo/test", "/foo/hard").unwrap();
            file.group("foo/test/inner").unwrap();
            file.group("/foo/hard/inner").unwrap();
            assert_err!(file.link_hard("foo/test", "/foo/test/inner"),
                "unable to create link: name already exists");
            assert_err!(file.link_hard("foo/bar", "/foo/baz"),
                "unable to create link: object.+doesn't exist");
            file.relink("/foo/hard", "/foo/hard2").unwrap();
            file.group("/foo/hard2/inner").unwrap();
            file.relink("/foo/test", "/foo/baz").unwrap();
            file.group("/foo/baz/inner").unwrap();
            file.group("/foo/hard2/inner").unwrap();
            file.unlink("/foo/baz").unwrap();
            assert_err!(file.group("/foo/baz"), "unable to open group");
            file.group("/foo/hard2/inner").unwrap();
            file.unlink("/foo/hard2").unwrap();
            assert_err!(file.group("/foo/hard2/inner"), "unable to open group");
        })
    }

    #[test]
    pub fn test_link_soft() {
        silence_errors();
        with_tmp_file(|file| {
            file.create_group("a/b/c").unwrap();
            file.link_soft("/a/b", "a/soft").unwrap();
            file.group("/a/soft/c").unwrap();
            file.relink("/a/soft", "/a/soft2").unwrap();
            file.group("/a/soft2/c").unwrap();
            file.relink("a/b", "/a/d").unwrap();
            assert_err!(file.group("/a/soft2/c"), "unable to open group");
            file.link_soft("/a/bar", "/a/baz").unwrap();
            assert_err!(file.group("/a/baz"), "unable to open group");
            file.create_group("/a/bar").unwrap();
            file.group("/a/baz").unwrap();
            file.unlink("/a/bar").unwrap();
            assert_err!(file.group("/a/bar"), "unable to open group");
            assert_err!(file.group("/a/baz"), "unable to open group");
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

    #[test]
    pub fn test_unlink() {
        silence_errors();
        with_tmp_file(|file| {
            file.create_group("/foo/bar").unwrap();
            file.unlink("foo/bar").unwrap();
            assert_err!(file.group("/foo/bar"), "unable to open group");
            assert!(file.group("foo").unwrap().is_empty());
        })
    }

    #[test]
    pub fn test_dataset() {
        with_tmp_file(|file| {
            file.new_dataset::<u32>().no_chunk().create("/foo/bar", (10, 20)).unwrap();
            file.new_dataset::<f32>().resizable(true).create("baz", (10, 20)).unwrap();
            file.new_dataset::<u8>().resizable(true).create_anon((10, 20)).unwrap();
        });
    }
}
