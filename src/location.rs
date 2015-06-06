use ffi::h5o::{H5Oget_comment, H5Oset_comment};
use ffi::h5i::{H5Iget_name, H5Iget_file_id, H5I_INVALID_HID};
use ffi::h5f::{H5Fget_name, H5Fflush, H5F_SCOPE_LOCAL, H5F_SCOPE_GLOBAL};

use error::Result;
use file::File;
use object::Object;
use util::{get_h5_str, to_cstring};

use std::ptr;

pub trait Location: Object {
    /// Returns the name of the named object within the file.
    fn name(&self) -> String {
        // TODO: should this return Result<String> or an empty string if it fails?
        h5lock!(get_h5_str(|m, s| { H5Iget_name(self.id(), m, s) }).unwrap_or("".to_string()))
    }

    /// Returns the name of the file containing the named object (or the file itself).
    fn filename(&self) -> String {
        // TODO: should this return Result<String> or an empty string if it fails?
        h5lock!(get_h5_str(|m, s| { H5Fget_name(self.id(), m, s) }).unwrap_or("".to_string()))
    }

    /// Returns a handle to the file containing the named object (or the file itself).
    fn file(&self) -> File {
        // Result<File> or File::from_id(H5I_INVALID_HID)?
        File::from_id(h5call!(H5Iget_file_id(self.id())).unwrap_or(H5I_INVALID_HID))
    }

    /// Flushes the file containing the named object to storage medium.
    fn flush(&self, global: bool) -> Result<()> {
        let scope = if global { H5F_SCOPE_GLOBAL } else { H5F_SCOPE_LOCAL };
        h5call!(H5Fflush(self.id(), scope)).and(Ok(()))
    }

    /// Returns the commment attached to the named object, if any.
    fn comment(&self) -> Option<String> {
        // TODO: should this return Result<Option<String>> or fail silently?
        let comment = h5lock!(get_h5_str(|m, s| { H5Oget_comment(self.id(), m, s) }).ok());
        comment.and_then(|c| if c.len() == 0 { None } else { Some(c) })
    }

    /// Set or the commment attached to the named object.
    fn set_comment<S: Into<String>>(&self, comment: S) -> Result<()> {
        let c: String = comment.into();
        h5call!(H5Oset_comment(self.id(), to_cstring(c.as_ref()).as_ptr())).and(Ok(()))
    }

    /// Clear the commment attached to the named object.
    fn clear_comment(&self) -> Result<()> {
        h5call!(H5Oset_comment(self.id(), ptr::null_mut())).and(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::Location;
    use file::File;
    use object::Object;
    use test::{with_tmp_path, with_tmp_file};

    use std::fs;

    #[test]
    pub fn test_filename() {
        with_tmp_path("foo.h5", |path| {
            assert_eq!(File::open(&path, "w").unwrap().filename(), path.to_str().unwrap());
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
            assert_eq!(file.file().id(), file.id());
        })
    }

    #[test]
    pub fn test_flush() {
        with_tmp_file(|file| {
            assert!(file.size() > 0);
            assert_eq!(fs::metadata(file.filename()).unwrap().len(), 0);
            assert!(file.flush(false).is_ok());
            assert!(file.size() > 0);
            assert_eq!(file.size(), fs::metadata(file.filename()).unwrap().len());
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
