use std::fmt::{self, Debug};
use std::ops::Deref;
use std::panic;

use hdf5_sys::{
    h5::{hsize_t, H5_index_t, H5_iter_order_t},
    h5d::H5Dopen2,
    h5g::{H5G_info_t, H5Gcreate2, H5Gget_info, H5Gopen2},
    h5l::{
        H5L_info_t, H5L_iterate_t, H5Lcreate_hard, H5Lcreate_soft, H5Ldelete, H5Lexists,
        H5Literate, H5Lmove, H5L_SAME_LOC,
    },
    h5p::{H5Pcreate, H5Pset_create_intermediate_group},
};

use crate::globals::H5P_LINK_CREATE;
use crate::internal_prelude::*;

/// Represents the HDF5 group object.
#[repr(transparent)]
#[derive(Clone)]
pub struct Group(Handle);

impl ObjectClass for Group {
    const NAME: &'static str = "group";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_GROUP, H5I_FILE];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    fn short_repr(&self) -> Option<String> {
        let members = match self.len() {
            0 => "empty".to_owned(),
            1 => "1 member".to_owned(),
            x => format!("{} members", x),
        };
        Some(format!("\"{}\" ({})", self.name(), members))
    }
}

impl Debug for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_fmt(f)
    }
}

impl Deref for Group {
    type Target = Location;

    fn deref(&self) -> &Location {
        unsafe { self.transmute() }
    }
}

fn group_info(id: hid_t) -> Result<H5G_info_t> {
    let info: *mut H5G_info_t = &mut H5G_info_t::default();
    h5call!(H5Gget_info(id, info)).and(Ok(unsafe { *info }))
}

fn make_lcpl() -> Result<PropertyList> {
    h5lock!({
        let lcpl = PropertyList::from_id(h5try!(H5Pcreate(*H5P_LINK_CREATE)))?;
        h5call!(H5Pset_create_intermediate_group(lcpl.id(), 1)).and(Ok(lcpl))
    })
}

impl Group {
    /// Returns the number of objects in the container (or 0 if the container is invalid).
    pub fn len(&self) -> u64 {
        group_info(self.id()).map(|info| info.nlinks).unwrap_or(0)
    }

    /// Returns true if the container has no linked objects (or if the container is invalid).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Create a new group in a file or group.
    pub fn create_group(&self, name: &str) -> Result<Self> {
        // TODO: &mut self?
        h5lock!({
            let lcpl = make_lcpl()?;
            let name = to_cstring(name)?;
            Self::from_id(h5try!(H5Gcreate2(
                self.id(),
                name.as_ptr(),
                lcpl.id(),
                H5P_DEFAULT,
                H5P_DEFAULT
            )))
        })
    }

    /// Opens an existing group in a file or group.
    pub fn group(&self, name: &str) -> Result<Self> {
        let name = to_cstring(name)?;
        Self::from_id(h5try!(H5Gopen2(self.id(), name.as_ptr(), H5P_DEFAULT)))
    }

    /// Creates a soft link. Note: `src` and `dst` are relative to the current object.
    pub fn link_soft(&self, src: &str, dst: &str) -> Result<()> {
        // TODO: &mut self?
        h5lock!({
            let lcpl = make_lcpl()?;
            let src = to_cstring(src)?;
            let dst = to_cstring(dst)?;
            h5call!(H5Lcreate_soft(src.as_ptr(), self.id(), dst.as_ptr(), lcpl.id(), H5P_DEFAULT))
                .and(Ok(()))
        })
    }

    /// Creates a hard link. Note: `src` and `dst` are relative to the current object.
    pub fn link_hard(&self, src: &str, dst: &str) -> Result<()> {
        // TODO: &mut self?
        let src = to_cstring(src)?;
        let dst = to_cstring(dst)?;
        h5call!(H5Lcreate_hard(
            self.id(),
            src.as_ptr(),
            H5L_SAME_LOC,
            dst.as_ptr(),
            H5P_DEFAULT,
            H5P_DEFAULT
        ))
        .and(Ok(()))
    }

    /// Relinks an object. Note: `name` and `path` are relative to the current object.
    pub fn relink(&self, name: &str, path: &str) -> Result<()> {
        // TODO: &mut self?
        let name = to_cstring(name)?;
        let path = to_cstring(path)?;
        h5call!(H5Lmove(
            self.id(),
            name.as_ptr(),
            H5L_SAME_LOC,
            path.as_ptr(),
            H5P_DEFAULT,
            H5P_DEFAULT
        ))
        .and(Ok(()))
    }

    /// Removes a link to an object from this file or group.
    pub fn unlink(&self, name: &str) -> Result<()> {
        // TODO: &mut self?
        let name = to_cstring(name)?;
        h5call!(H5Ldelete(self.id(), name.as_ptr(), H5P_DEFAULT)).and(Ok(()))
    }

    /// Check if a link with a given name exists in this file or group.
    pub fn link_exists(&self, name: &str) -> bool {
        (|| -> Result<bool> {
            let name = to_cstring(name)?;
            Ok(h5call!(H5Lexists(self.id(), name.as_ptr(), H5P_DEFAULT))? > 0)
        })()
        .unwrap_or(false)
    }

    /// Instantiates a new dataset builder.
    pub fn new_dataset<T: H5Type>(&self) -> DatasetBuilder<T> {
        DatasetBuilder::<T>::new(self)
    }

    /// Opens an existing dataset in the file or group.
    pub fn dataset(&self, name: &str) -> Result<Dataset> {
        let name = to_cstring(name)?;
        Dataset::from_id(h5try!(H5Dopen2(self.id(), name.as_ptr(), H5P_DEFAULT)))
    }

    /// Returns names of all the members in the group, non-recursively.
    pub fn member_names(&self) -> Result<Vec<String>> {
        extern "C" fn members_callback(
            _id: hid_t, name: *const c_char, _info: *const H5L_info_t, op_data: *mut c_void,
        ) -> herr_t {
            panic::catch_unwind(|| {
                let other_data: &mut Vec<String> = unsafe { &mut *(op_data as *mut Vec<String>) };

                other_data.push(string_from_cstr(name));

                0 // Continue iteration
            })
            .unwrap_or(-1)
        }

        let callback_fn: H5L_iterate_t = Some(members_callback);
        let iteration_position: *mut hsize_t = &mut { 0_u64 };
        let mut result: Vec<String> = Vec::new();
        let other_data: *mut c_void = &mut result as *mut _ as *mut c_void;

        h5call!(H5Literate(
            self.id(),
            H5_index_t::H5_INDEX_NAME,
            H5_iter_order_t::H5_ITER_INC,
            iteration_position,
            callback_fn,
            other_data
        ))?;

        Ok(result)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::internal_prelude::*;

    #[test]
    pub fn test_debug() {
        with_tmp_file(|file| {
            file.create_group("a/b/c").unwrap();
            file.create_group("/a/d").unwrap();
            let a = file.group("a").unwrap();
            let ab = file.group("/a/b").unwrap();
            let abc = file.group("./a/b/c/").unwrap();
            assert_eq!(format!("{:?}", a), "<HDF5 group: \"/a\" (2 members)>");
            assert_eq!(format!("{:?}", ab), "<HDF5 group: \"/a/b\" (1 member)>");
            assert_eq!(format!("{:?}", abc), "<HDF5 group: \"/a/b/c\" (empty)>");
            file.close();
            assert_eq!(format!("{:?}", a), "<HDF5 group: invalid id>");
        })
    }

    #[test]
    pub fn test_group() {
        with_tmp_file(|file| {
            assert_err_re!(file.group("a"), "unable to open group: object.+doesn't exist");
            file.create_group("a").unwrap();
            let a = file.group("a").unwrap();
            assert_eq!(a.name(), "/a");
            assert_eq!(a.file().unwrap().id(), file.id());
            a.create_group("b").unwrap();
            let b = file.group("/a/b").unwrap();
            assert_eq!(b.name(), "/a/b");
            assert_eq!(b.file().unwrap().id(), file.id());
            file.create_group("/foo/bar").unwrap();
            file.group("foo").unwrap().group("bar").unwrap();
            file.create_group("x/y/").unwrap();
            file.group("/x").unwrap().group("./y/").unwrap();
        })
    }

    #[test]
    pub fn test_clone() {
        with_tmp_file(|file| {
            file.create_group("a").unwrap();
            let a = file.group("a").unwrap();
            assert_eq!(a.name(), "/a");
            assert_eq!(a.file().unwrap().id(), file.id());
            assert_eq!(a.refcount(), 1);
            let b = a.clone();
            assert_eq!(b.name(), "/a");
            assert_eq!(b.file().unwrap().id(), file.id());
            assert_eq!(b.refcount(), 2);
            assert_eq!(a.refcount(), 2);
            drop(a);
            assert_eq!(b.refcount(), 1);
            assert!(b.is_valid());
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
        with_tmp_file(|file| {
            file.create_group("foo/test/inner").unwrap();
            file.link_hard("/foo/test", "/foo/hard").unwrap();
            file.group("foo/test/inner").unwrap();
            file.group("/foo/hard/inner").unwrap();
            assert_err_re!(
                file.link_hard("foo/test", "/foo/test/inner"),
                "unable to create (?:hard )?link: name already exists"
            );
            assert_err_re!(
                file.link_hard("foo/bar", "/foo/baz"),
                "unable to create (?:hard )?link: object.+doesn't exist"
            );
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
    pub fn test_link_exists() {
        with_tmp_file(|file| {
            file.create_group("a/b/c").unwrap();
            file.link_soft("/a/b", "a/soft").unwrap();
            file.group("/a/soft/c").unwrap();
            assert!(file.link_exists("a"));
            assert!(file.link_exists("a/b"));
            assert!(file.link_exists("a/b/c"));
            assert!(file.link_exists("a/soft"));
            assert!(file.link_exists("a/soft/c"));
            assert!(!file.link_exists("b"));
            assert!(!file.link_exists("soft"));
            let group = file.group("a/soft").unwrap();
            assert!(group.link_exists("c"));
            assert!(!group.link_exists("a"));
            assert!(!group.link_exists("soft"));
            #[cfg(not(hdf5_1_10_0))]
            assert!(!group.link_exists("/"));
            #[cfg(hdf5_1_10_0)]
            assert!(group.link_exists("/"));
        })
    }

    #[test]
    pub fn test_relink() {
        with_tmp_file(|file| {
            file.create_group("test").unwrap();
            file.group("test").unwrap();
            assert_err!(
                file.relink("test", "foo/test"),
                "unable to move link: component not found"
            );
            file.create_group("foo").unwrap();
            assert_err!(file.relink("bar", "/baz"), "unable to move link: name doesn't exist");
            file.relink("test", "/foo/test").unwrap();
            file.group("/foo/test").unwrap();
            assert_err_re!(file.group("test"), "unable to open group: object.+doesn't exist");
        })
    }

    #[test]
    pub fn test_unlink() {
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

    #[test]
    pub fn test_get_member_names() {
        with_tmp_file(|file| {
            file.create_group("a").unwrap();
            file.create_group("b").unwrap();
            let group_a = file.group("a").unwrap();
            let group_b = file.group("b").unwrap();
            file.new_dataset::<u32>().no_chunk().create("a/foo", (10, 20)).unwrap();
            file.new_dataset::<u32>().no_chunk().create("a/123", (10, 20)).unwrap();
            file.new_dataset::<u32>().no_chunk().create("a/bar", (10, 20)).unwrap();
            assert_eq!(group_a.member_names().unwrap(), vec!["123", "bar", "foo"]);
            assert_eq!(group_b.member_names().unwrap().len(), 0);
            assert_eq!(file.member_names().unwrap(), vec!["a", "b"]);
        })
    }
}
