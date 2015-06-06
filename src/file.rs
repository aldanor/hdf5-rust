use ffi::h5::{hsize_t, hbool_t};
use ffi::h5i::{hid_t, H5I_INVALID_HID};
use ffi::h5p::{H5Pcreate, H5Pset_userblock};
use ffi::h5f::{H5F_ACC_RDONLY, H5F_ACC_RDWR, H5F_ACC_EXCL, H5F_ACC_TRUNC,
               H5Fopen, H5Fcreate, H5Fget_filesize, H5Fget_intent,
               H5Fget_access_plist, H5Fget_create_plist, H5Fget_freespace};
use ffi::h5fd::{H5Pset_fapl_sec2, H5Pset_fapl_stdio, H5Pset_fapl_core};

use globals::{H5P_FILE_CREATE, H5P_FILE_ACCESS};

use container::Container;
use error::Result;
use location::Location;
use object::{Handle, Object};
use plist::PropertyList;
use util::string_to_cstr;

use std::path::Path;
use std::process::Command;

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

impl Location for File {}

impl Container for File {}

impl File {
    /// Create a new file object.
    ///
    /// | `mode`    | File access mode
    /// |-----------|-----------------
    /// | `r`       | Read-only, file must exist
    /// | `r+`      | Read/write, file must exist
    /// | `w`       | Create file, truncate if exists
    /// | `w-`, `x` | Create file, fail if exists
    /// | `a`       | Read/write if exists, create otherwise
    pub fn open<P: AsRef<Path>, S: Into<String>>(filename: P, mode: S) -> Result<File> {
        FileBuilder::new().mode(mode).open(filename)
    }

    /// Returns the file size in bytes.
    pub fn size(&self) -> u64 {
        unsafe {
            let size: *mut hsize_t = &mut 0;
            h5lock_s!(H5Fget_filesize(self.id(), size));
            if *size > 0 { *size as u64 } else { 0 }
        }
    }

    /// Returns the free space in the file in bytes.
    pub fn free_space(&self) -> u64 {
        match h5call!(H5Fget_freespace(self.id())) {
            Ok(size) => size as u64,
            _        => 0,
        }
    }

    /// Returns true if the file was opened in a read-only mode.
    pub fn is_read_only(&self) -> bool {
        unsafe {
            let mode: *mut c_uint = &mut 0;
            h5lock_s!(H5Fget_intent(self.id(), mode));
            *mode != H5F_ACC_RDWR
        }
    }

    #[allow(dead_code)]
    fn fapl(&self) -> PropertyList {
        PropertyList::from_id(h5call!(H5Fget_access_plist(self.id())).unwrap_or(H5I_INVALID_HID))
    }

    #[allow(dead_code)]
    fn fcpl(&self) -> PropertyList {
        PropertyList::from_id(h5call!(H5Fget_create_plist(self.id())).unwrap_or(H5I_INVALID_HID))
    }

    /// Returns the output of the `h5dump` tool. Note that this wouldn't work with core driver.
    pub fn dump(&self) -> Option<String> {
        self.flush(true).ok().and(Command::new("h5dump").arg(self.filename()).output().ok()
                                  .map(|out| String::from_utf8_lossy(&out.stdout).to_string()))
    }
}

pub struct FileBuilder {
    driver: String,
    mode: String,
    userblock: size_t,
    filebacked: bool,
    increment: size_t,
}


impl FileBuilder {
    pub fn new() -> FileBuilder {
        FileBuilder {
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

    fn open_file<P: AsRef<Path>>(&self, filename: P, write: bool) -> Result<File> {
        ensure!(self.userblock == 0, "Cannot specify userblock when opening a file");
        h5lock_s!({
            let fapl = try!(self.make_fapl());
            let flags = if write { H5F_ACC_RDWR } else { H5F_ACC_RDONLY };
            let filename = filename.as_ref();
            match filename.to_str() {
                Some(filename) => {
                    let c_filename = string_to_cstr(filename);
                    let file = File::from_id(h5try!(H5Fopen(c_filename, flags, fapl.id())));
                    ensure!(file.is_valid(), "Invalid id for opened file");
                    Ok(file)
                },
                None          => fail!("Invalid UTF-8 in file name: {:?}", filename)
            }
        })
    }

    fn create_file<P: AsRef<Path>>(&self, filename: P, exclusive: bool) -> Result<File> {
        h5lock_s!({
            let fcpl = PropertyList::from_id(h5try!(H5Pcreate(*H5P_FILE_CREATE)));
            h5try!(H5Pset_userblock(fcpl.id(), self.userblock));
            let fapl = try!(self.make_fapl());
            let flags = if exclusive { H5F_ACC_EXCL } else { H5F_ACC_TRUNC };
            let filename = filename.as_ref();
            match filename.to_str() {
                Some(filename) => {
                    let c_filename = string_to_cstr(filename);
                    let file = File::from_id(h5try!(H5Fcreate(c_filename, flags,
                                                              fcpl.id(), fapl.id())));
                    ensure!(file.is_valid(), "Invalid id for created file");
                    Ok(file)
                },
                None          => fail!("Invalid UTF-8 in file name: {:?}", filename)
            }
        })
    }

    pub fn open<P: AsRef<Path>>(&self, filename: P) -> Result<File> {
        match self.mode.as_ref() {
            "r"        => self.open_file(&filename, false),
            "r+"       => self.open_file(&filename, true),
            "w"        => self.create_file(&filename, false),
            "w-" | "x" => self.create_file(&filename, true),
            "a"        => match self.open_file(&filename, true) {
                            Ok(file) => Ok(file),
                            _        => self.create_file(&filename, true),
                          },
            _          => fail!("Invalid file access mode, expected r|r+|w|w-|x|a"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::File;
    use std::fs;
    use std::io::Write;
    use test::{with_tmp_dir, with_tmp_path};
    use error::silence_errors;

    #[test]
    pub fn test_invalid_mode() {
        silence_errors();
        with_tmp_dir(|dir| {
            assert_err!(File::open(&dir, "foo"), "Invalid file access mode");
        })
    }

    #[test]
    pub fn test_non_hdf5_file() {
        silence_errors();
        with_tmp_path("foo.h5", |path| {
            fs::File::create(&path).unwrap().write_all(b"foo").unwrap();
            assert!(fs::metadata(&path).is_ok());
            assert_err!(File::open(&path, "r"), "unable to open file");
        })
    }

    #[test]
    pub fn test_is_read_only() {
        silence_errors();
        with_tmp_path("foo.h5", |path| {
            assert!(!File::open(&path, "w").unwrap().is_read_only());
            assert!(File::open(&path, "r").unwrap().is_read_only());
            assert!(!File::open(&path, "r+").unwrap().is_read_only());
            assert!(!File::open(&path, "a").unwrap().is_read_only());
        });
        with_tmp_path("foo.h5", |path| {
            assert!(!File::open(&path, "a").unwrap().is_read_only());
        });
        with_tmp_path("foo.h5", |path| {
            assert!(!File::open(&path, "x").unwrap().is_read_only());
        });
    }

    #[test]
    pub fn test_unable_to_open() {
        silence_errors();
        with_tmp_dir(|dir| {
            assert_err!(File::open(&dir, "r"), "unable to open file");
            assert_err!(File::open(&dir, "r+"), "unable to open file");
            assert_err!(File::open(&dir, "x"), "unable to create file");
            assert_err!(File::open(&dir, "w-"), "unable to create file");
            assert_err!(File::open(&dir, "w"), "unable to create file");
            assert_err!(File::open(&dir, "a"), "unable to create file");
        });
    }
}
