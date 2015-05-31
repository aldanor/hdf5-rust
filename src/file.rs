use ffi::types::{hid_t, hsize_t, hbool_t};
use ffi::h5i::H5I_INVALID_HID;
use ffi::h5p::{H5P_FILE_CREATE, H5P_FILE_ACCESS, H5Pcreate, H5Pset_userblock};
use ffi::h5f::{H5F_ACC_RDONLY, H5F_ACC_RDWR, H5F_ACC_EXCL, H5F_ACC_TRUNC,
               H5Fopen, H5Fcreate, H5Fclose, H5Fget_filesize, H5Fget_intent,
               H5Fget_access_plist, H5Fget_create_plist};
use ffi::drivers::{H5Pset_fapl_sec2, H5Pset_fapl_stdio, H5Pset_fapl_core};

use object::{Handle, Object};
use error::Result;
use plist::PropertyList;
use ffi::util::string_to_cstr;
use std::path::{Path, PathBuf};

use libc::{size_t, c_uint};

#[derive(Clone)]
pub struct File {
    handle: Handle,
}

impl Object for File {
    fn id(&self) -> hid_t {
        self.handle.id()
    }

    fn from_id(id: hid_t) -> File {
        File { handle: Handle::new(id) }
    }
}

impl File {
    pub fn size(&self) -> u64 {
        unsafe {
            let size: *mut hsize_t = &mut 0;
            h5lock_s!(H5Fget_filesize(self.id(), size));
            if *size > 0 { *size as u64 } else { 0 }
        }
    }

    pub fn is_read_only(&self) -> bool {
        unsafe {
            let mode: *mut c_uint = &mut 0;
            h5lock_s!(H5Fget_intent(self.id(), mode));
            *mode != H5F_ACC_RDWR
        }
    }

    pub fn open<P: AsRef<Path>, S: Into<String>>(filename: P, mode: S) -> Result<File> {
        FileBuilder::new(filename).mode(mode).open()
    }

    fn fapl(&self) -> PropertyList {
        PropertyList::from_id(h5call!(H5Fget_access_plist(self.id())).unwrap_or(H5I_INVALID_HID))
    }

    fn fcpl(&self) -> PropertyList {
        PropertyList::from_id(h5call!(H5Fget_create_plist(self.id())).unwrap_or(H5I_INVALID_HID))
    }
}

impl Drop for File {
    fn drop(&mut self) {
        if self.refcount() == 1 {
            h5lock!(H5Fclose(self.id()));
        }
    }
}

pub struct FileBuilder {
    filename: PathBuf,
    driver: String,
    mode: String,
    userblock: size_t,
    filebacked: bool,
    increment: size_t,
}


impl FileBuilder {
    pub fn new<P: AsRef<Path>>(filename: P) -> FileBuilder {
        FileBuilder {
            filename: filename.as_ref().to_path_buf(),
            driver: "sec2".to_string(),
            mode: "r".to_string(),
            userblock: 0,
            filebacked: false,
            increment: 64 * 1024 * 1024,
        }
    }

    pub fn driver<S: Into<String>>(&mut self, driver: S) -> &mut FileBuilder {
        self.driver = driver.into(); self
    }

    pub fn mode<S: Into<String>>(&mut self, mode: S) -> &mut FileBuilder {
        self.mode = mode.into(); self
    }

    pub fn userblock(&mut self, userblock: size_t) -> &mut FileBuilder {
        self.userblock = userblock; self
    }

    pub fn filebacked(&mut self, filebacked: bool) -> &mut FileBuilder {
        self.filebacked = filebacked; self
    }

    pub fn increment(&mut self, increment: size_t) -> &mut FileBuilder {
        self.increment = increment; self
    }

    fn make_fapl(&self) -> Result<PropertyList> {
        h5lock_s!({
            let fapl = PropertyList::from_id(h5try!(H5Pcreate(*H5P_FILE_ACCESS)));
            match self.driver.as_ref() {
                "sec2"  => h5try!(H5Pset_fapl_sec2(fapl.id())),
                "stdio" => h5try!(H5Pset_fapl_stdio(fapl.id())),
                "core"  => h5try!(H5Pset_fapl_core(fapl.id(), self.increment,
                                                   self.filebacked as hbool_t)),
                _       => fail!(format!("Invalid file driver: {}", self.driver)),
            };
            Ok(fapl)
        })
    }

    fn open_file(&self, write: bool) -> Result<File> {
        ensure!(self.userblock == 0, "Cannot specify userblock when opening a file");
        h5lock_s!({
            let fapl = try!(self.make_fapl());
            let flags = if write { H5F_ACC_RDWR } else { H5F_ACC_RDONLY };
            match self.filename.to_str() {
                Some(filename) => {
                    let c_filename = string_to_cstr(filename);
                    let file = File::from_id(h5try!(H5Fopen(c_filename, flags, fapl.id())));
                    ensure!(file.is_valid(), "Invalid id for opened file");
                    Ok(file)
                },
                None          => fail!("Invalid UTF-8 in file name: {:?}", self.filename)
            }
        })
    }

    fn create_file(&self, exclusive: bool) -> Result<File> {
        h5lock_s!({
            let fcpl = PropertyList::from_id(h5try!(H5Pcreate(*H5P_FILE_CREATE)));
            h5try!(H5Pset_userblock(fcpl.id(), self.userblock));
            let fapl = try!(self.make_fapl());
            let flags = if exclusive { H5F_ACC_EXCL } else { H5F_ACC_TRUNC };
            match self.filename.to_str() {
                Some(filename) => {
                    let c_filename = string_to_cstr(filename);
                    let file = File::from_id(h5try!(H5Fcreate(c_filename, flags,
                                                              fcpl.id(), fapl.id())));
                    ensure!(file.is_valid(), "Invalid id for created file");
                    Ok(file)
                },
                None          => fail!("Invalid UTF-8 in file name: {:?}", self.filename)
            }
        })
    }

    pub fn open(&self) -> Result<File> {
        match self.mode.as_ref() {
            "r"        => self.open_file(false),
            "r+"       => self.open_file(true),
            "w"        => self.create_file(false),
            "w-" | "x" => self.create_file(true),
            "a"        => match self.open_file(true) {
                            Ok(file) => Ok(file),
                            _        => self.create_file(true),
                          },
            _          => fail!("Invalid file access mode, expected r|r+|w|w-|x|a"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::File;
    use std::path::PathBuf;
    use tempdir::TempDir;

    fn with_tmpdir<F: Fn(PathBuf)>(f: F) {
        let dir = TempDir::new_in(".", "tmp").unwrap();
        let path = dir.path().to_path_buf();
        f(path);
    }

    #[test]
    pub fn test_invalid_mode() {
        with_tmpdir(|dir| {
            assert_err!(File::open(&dir, "foo"), "Invalid file access mode");
        })
    }

    #[test]
    pub fn test_is_read_only() {
        with_tmpdir(|dir| {
            let path = dir.join("foo.h5");
            assert!(!File::open(&path, "w").unwrap().is_read_only());
            assert!(File::open(&path, "r").unwrap().is_read_only());
            assert!(!File::open(&path, "r+").unwrap().is_read_only());
            assert!(!File::open(&path, "a").unwrap().is_read_only());
        });
        with_tmpdir(|dir| {
            assert!(!File::open(dir.join("foo.h"), "a").unwrap().is_read_only());
        });
        with_tmpdir(|dir| {
            assert!(!File::open(dir.join("foo.h"), "x").unwrap().is_read_only());
        });
    }

    #[test]
    pub fn test_invalid_filename() {
        with_tmpdir(|dir| {
            assert_err!(File::open(&dir, "r"), "unable to open file");
            assert_err!(File::open(&dir, "r+"), "unable to open file");
            assert_err!(File::open(&dir, "x"), "unable to create file");
            assert_err!(File::open(&dir, "w-"), "unable to create file");
            assert_err!(File::open(&dir, "w"), "unable to create file");
            assert_err!(File::open(&dir, "a"), "unable to create file");
        });
    }
}
