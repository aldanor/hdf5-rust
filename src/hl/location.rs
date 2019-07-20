use std::fmt::{self, Debug};
use std::ops::Deref;
use std::ptr;

use hdf5_sys::{
    h5f::H5Fget_name,
    h5i::{H5Iget_file_id, H5Iget_name},
    h5o::{H5Oget_comment, H5Oset_comment},
};

use crate::internal_prelude::*;

/// Named location (file, group, dataset, named datatype).
#[repr(transparent)]
#[derive(Clone)]
pub struct Location(Handle);

impl ObjectClass for Location {
    const NAME: &'static str = "location";
    const VALID_TYPES: &'static [H5I_type_t] =
        &[H5I_FILE, H5I_GROUP, H5I_DATATYPE, H5I_DATASET, H5I_ATTR];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    fn short_repr(&self) -> Option<String> {
        Some(format!("\"{}\"", self.name()))
    }
}

impl Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_fmt(f)
    }
}

impl Deref for Location {
    type Target = Object;

    fn deref(&self) -> &Object {
        unsafe { self.transmute() }
    }
}

impl Location {
    /// Returns the name of the object within the file, or empty string if the object doesn't
    /// have a name (e.g., an anonymous dataset).
    pub fn name(&self) -> String {
        // TODO: should this return Result<String> or an empty string if it fails?
        h5lock!(get_h5_str(|m, s| H5Iget_name(self.id(), m, s)).unwrap_or_else(|_| "".to_string()))
    }

    /// Returns the name of the file containing the named object (or the file itself).
    pub fn filename(&self) -> String {
        // TODO: should this return Result<String> or an empty string if it fails?
        h5lock!(get_h5_str(|m, s| H5Fget_name(self.id(), m, s)).unwrap_or_else(|_| "".to_string()))
    }

    /// Returns a handle to the file containing the named object (or the file itself).
    pub fn file(&self) -> Result<File> {
        File::from_id(h5try!(H5Iget_file_id(self.id())))
    }

    /// Returns the commment attached to the named object, if any.
    pub fn comment(&self) -> Option<String> {
        // TODO: should this return Result<Option<String>> or fail silently?
        let comment = h5lock!(get_h5_str(|m, s| H5Oget_comment(self.id(), m, s)).ok());
        comment.and_then(|c| if c.is_empty() { None } else { Some(c) })
    }

    /// Set or the commment attached to the named object.
    pub fn set_comment(&self, comment: &str) -> Result<()> {
        // TODO: &mut self?
        let comment = to_cstring(comment)?;
        h5call!(H5Oset_comment(self.id(), comment.as_ptr())).and(Ok(()))
    }

    /// Clear the commment attached to the named object.
    pub fn clear_comment(&self) -> Result<()> {
        // TODO: &mut self?
        h5call!(H5Oset_comment(self.id(), ptr::null_mut())).and(Ok(()))
    }
}

#[cfg(test)]
pub mod tests {
    use crate::internal_prelude::*;

    #[test]
    pub fn test_filename() {
        with_tmp_path(|path| {
            assert_eq!(File::create(&path).unwrap().filename(), path.to_str().unwrap());
        })
    }

    #[test]
    pub fn test_name() {
        with_tmp_file(|file| {
            assert_eq!(file.name(), "/");
        })
    }

    #[test]
    pub fn test_file() {
        with_tmp_file(|file| {
            assert_eq!(file.file().unwrap().id(), file.id());
        })
    }

    #[test]
    pub fn test_comment() {
        with_tmp_file(|file| {
            assert!(file.comment().is_none());
            assert!(file.set_comment("foo").is_ok());
            assert_eq!(file.comment().unwrap(), "foo");
            assert!(file.clear_comment().is_ok());
            assert!(file.comment().is_none());
        })
    }
}
