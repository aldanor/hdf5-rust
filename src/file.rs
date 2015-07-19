use ffi::h5::{hsize_t, hbool_t};
use ffi::h5i::{H5I_FILE, hid_t};
use ffi::h5p::{H5Pcreate, H5Pset_userblock, H5Pget_userblock};
use ffi::h5f::{
    H5F_ACC_RDONLY, H5F_ACC_RDWR, H5F_ACC_EXCL, H5F_ACC_TRUNC, H5F_SCOPE_LOCAL,
    H5F_OBJ_FILE, H5F_OBJ_ALL, H5Fopen, H5Fcreate, H5Fget_filesize, H5Fget_freespace,
    H5Fflush, H5Fget_obj_ids, H5Fget_access_plist, H5Fget_create_plist, H5Fget_intent,
    H5Fget_obj_count, H5Fclose
};
use ffi::h5fd::{H5Pset_fapl_sec2, H5Pset_fapl_stdio, H5Pset_fapl_core};

use globals::{H5P_FILE_CREATE, H5P_FILE_ACCESS};

use container::Container;
use error::Result;
use location::Location;
use object::Object;
use handle::{Handle, ID, FromID, get_id_type};
use plist::PropertyList;
use util::to_cstring;

use std::fmt;
use std::path::Path;
use std::process::Command;

use libc::{size_t, c_uint};

/// Represents the HDF5 file object.
pub struct File {
    handle: Handle,
}

#[doc(hidden)]
impl ID for File {
    fn id(&self) -> hid_t {
        self.handle.id()
    }
}

#[doc(hidden)]
impl FromID for File {
    fn from_id(id: hid_t) -> Result<File> {
        match get_id_type(id) {
            H5I_FILE => Ok(File { handle: try!(Handle::new(id)) }),
            _ => Err(From::from(format!("Invalid file id: {}", id))),
        }
    }
}

impl Object for File {}

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

    /// Returns the file size in bytes (or 0 if the file handle is invalid).
    pub fn size(&self) -> u64 {
        unsafe {
            let size: *mut hsize_t = &mut 0;
            h5lock_s!(H5Fget_filesize(self.id(), size));
            if *size > 0 { *size as u64 } else { 0 }
        }
    }

    /// Returns the free space in the file in bytes (or 0 if the file handle is invalid).
    pub fn free_space(&self) -> u64 {
        match h5call!(H5Fget_freespace(self.id())) {
            Ok(size) => size as u64,
            _ => 0,
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
    fn fapl(&self) -> Result<PropertyList> {
        PropertyList::from_id(h5try!(H5Fget_access_plist(self.id())))
    }

    fn fcpl(&self) -> Result<PropertyList> {
        PropertyList::from_id(h5try!(H5Fget_create_plist(self.id())))
    }

    /// Returns the output of the `h5dump` tool. Note that this wouldn't work with core driver.
    pub fn dump(&self) -> Option<String> {
        self.flush().ok().and(
            Command::new("h5dump").arg(self.filename()).output().ok()
                                  .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
        )
    }

    /// Returns the userblock size in bytes (or 0 if the file handle is invalid).
    pub fn userblock(&self) -> size_t {
        unsafe {
            let userblock: *mut hsize_t = &mut 0;
            if let Ok(fcpl) = self.fcpl() {
                h5lock_s!(H5Pget_userblock(fcpl.id(), userblock));
            }
            *userblock as size_t
        }
    }

    /// Flushes the file to the storage medium.
    pub fn flush(&self) -> Result<()> {
        h5call!(H5Fflush(self.id(), H5F_SCOPE_LOCAL)).and(Ok(()))
    }

    /// Get objects ids of the contained objects. Note: these are borrowed references.
    fn get_obj_ids(&self, types: c_uint) -> Vec<hid_t> {
        h5lock_s!({
            let count = h5call!(H5Fget_obj_count(self.id(), types)).unwrap_or(0) as size_t;
            if count > 0 {
                let mut ids: Vec<hid_t> = Vec::with_capacity(count as usize);
                unsafe { ids.set_len(count as usize); }
                if h5call!(H5Fget_obj_ids(self.id(), types, count, ids.as_mut_ptr())).is_ok() {
                    ids.retain(|id| *id != self.id());
                    return ids;
                }
            }
            Vec::new()
        })
    }

    /// Closes the file and invalidates all open handles for contained objects.
    pub fn close(&self) {
        h5lock_s!({
            let file_ids = self.get_obj_ids(H5F_OBJ_FILE);
            let object_ids = self.get_obj_ids(H5F_OBJ_ALL & !H5F_OBJ_FILE);
            for file_id in file_ids.iter() {
                let handle = Handle::from_id(*file_id);
                if let Ok(handle) = handle {
                    while handle.is_valid() {
                        handle.decref();
                    }
                }
            }
            for object_id in object_ids.iter() {
                let handle = Handle::from_id(*object_id);
                if let Ok(handle) = handle {
                    while handle.is_valid() {
                        handle.decref();
                    }
                }
            }
            unsafe { H5Fclose(self.id()); }
            while self.is_valid() {
                self.handle.decref();
            }
            self.handle.decref();
        })
    }
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.is_valid() {
            return "<HDF5 file: invalid id>".fmt(f);
        }
        let basename = match Path::new(&self.filename()).file_name() {
            Some(s) => s.to_string_lossy().into_owned(),
            None    => "".to_string(),
        };
        let mode = if self.is_read_only() { "read-only" } else { "read/write" };
        format!("<HDF5 file: \"{}\" ({})>", basename, mode).fmt(f)
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
            let fapl = try!(PropertyList::from_id(h5try!(H5Pcreate(*H5P_FILE_ACCESS))));
            match self.driver.as_ref() {
                "sec2"  => h5try!(H5Pset_fapl_sec2(fapl.id())),
                "stdio" => h5try!(H5Pset_fapl_stdio(fapl.id())),
                "core"  => h5try!(H5Pset_fapl_core(
                               fapl.id(), self.increment, self.filebacked as hbool_t
                           )),
                _ => fail!(format!("Invalid file driver: {}", self.driver)),
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
                    let c_filename = to_cstring(filename).as_ptr();
                    File::from_id(h5try!(H5Fopen(c_filename, flags, fapl.id())))
                },
                None => fail!("Invalid UTF-8 in file name: {:?}", filename)
            }
        })
    }

    fn create_file<P: AsRef<Path>>(&self, filename: P, exclusive: bool) -> Result<File> {
        h5lock_s!({
            let fcpl = try!(PropertyList::from_id(h5try!(H5Pcreate(*H5P_FILE_CREATE))));
            h5try!(H5Pset_userblock(fcpl.id(), self.userblock));
            let fapl = try!(self.make_fapl());
            let flags = if exclusive { H5F_ACC_EXCL } else { H5F_ACC_TRUNC };
            let filename = filename.as_ref();
            match filename.to_str() {
                Some(filename) => {
                    let c_filename = to_cstring(filename).as_ptr();
                    File::from_id(h5try!(H5Fcreate(c_filename, flags, fcpl.id(), fapl.id())))
                },
                None => fail!("Invalid UTF-8 in file name: {:?}", filename)
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
                            _ => self.create_file(&filename, true),
                          },
            _ => fail!("Invalid file access mode, expected r|r+|w|w-|x|a"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{File, FileBuilder};
    use container::Container;
    use error::silence_errors;
    use location::Location;
    use object::Object;
    use test::{with_tmp_dir, with_tmp_path, with_tmp_file};

    use std::fs;
    use std::io::{Read, Write};

    #[test]
    pub fn test_invalid_mode() {
        silence_errors();
        with_tmp_dir(|dir| {
            assert_err!(File::open(&dir, "foo"), "Invalid file access mode");
        })
    }

    #[test]
    pub fn test_is_read_only() {
        silence_errors();
        with_tmp_path(|path| {
            assert!(!File::open(&path, "w").unwrap().is_read_only());
            assert!(File::open(&path, "r").unwrap().is_read_only());
            assert!(!File::open(&path, "r+").unwrap().is_read_only());
            assert!(!File::open(&path, "a").unwrap().is_read_only());
        });
        with_tmp_path(|path| {
            assert!(!File::open(&path, "a").unwrap().is_read_only());
        });
        with_tmp_path(|path| {
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

        with_tmp_path(|path| {
            fs::File::create(&path).unwrap().write_all(b"foo").unwrap();
            assert!(fs::metadata(&path).is_ok());
            assert_err!(File::open(&path, "r"), "unable to open file");
        })
    }

    #[test]
    pub fn test_access_modes() {
        silence_errors();

        // "w" means overwrite
        with_tmp_path(|path| {
            File::open(&path, "w").unwrap().create_group("foo").unwrap();
            assert_err!(File::open(&path, "w").unwrap().group("foo"), "unable to open group");
        });

        // "w-"/"x-" means exclusive write
        with_tmp_path(|path| {
            File::open(&path, "w-").unwrap();
            assert_err!(File::open(&path, "w-"), "unable to create file");
        });
        with_tmp_path(|path| {
            File::open(&path, "x").unwrap();
            assert_err!(File::open(&path, "x"), "unable to create file");
        });

        // "a" means append
        with_tmp_path(|path| {
            File::open(&path, "a").unwrap().create_group("foo").unwrap();
            File::open(&path, "a").unwrap().group("foo").unwrap();
        });

        // "r" means read-only
        with_tmp_path(|path| {
            File::open(&path, "w").unwrap().create_group("foo").unwrap();
            let file = File::open(&path, "r").unwrap();
            file.group("foo").unwrap();
            assert_err!(file.create_group("bar"),
                "unable to create group: no write intent on file");
            assert_err!(File::open("/foo/bar/baz", "r"), "unable to open file");
        });

        // "r+" means read-write
        with_tmp_path(|path| {
            File::open(&path, "w").unwrap().create_group("foo").unwrap();
            let file = File::open(&path, "r+").unwrap();
            file.group("foo").unwrap();
            file.create_group("bar").unwrap();
            assert_err!(File::open("/foo/bar/baz", "r+"), "unable to open file");
        });
    }

    #[test]
    pub fn test_flush() {
        with_tmp_file(|file| {
            assert!(file.size() > 0);
            assert_eq!(fs::metadata(file.filename()).unwrap().len(), 0);
            assert!(file.flush().is_ok());
            assert!(file.size() > 0);
            assert_eq!(file.size(), fs::metadata(file.filename()).unwrap().len());
        })
    }

    #[test]
    pub fn test_userblock() {
        silence_errors();

        with_tmp_file(|file| {
            assert_eq!(file.userblock(), 0);
        });
        with_tmp_path(|path| {
            assert_err!(FileBuilder::new().userblock(512).mode("r").open(&path),
                "Cannot specify userblock when opening a file");
            assert_err!(FileBuilder::new().userblock(512).mode("r+").open(&path),
                "Cannot specify userblock when opening a file");
            assert_err!(FileBuilder::new().userblock(1).mode("w").open(&path),
                "userblock size is non-zero and less than 512");
            FileBuilder::new().userblock(512).mode("w").open(&path).unwrap();
            assert_eq!(File::open(&path, "r").unwrap().userblock(), 512);

            // writing to userblock doesn't corrupt the file
            File::open(&path, "r+").unwrap().create_group("foo").unwrap();
            {
                let mut file = fs::OpenOptions::new().read(true).write(true)
                                                     .create(false).open(&path).unwrap();
                for i in 0usize..512usize {
                    file.write_all(&[(i % 256) as u8]).unwrap();
                }
                file.flush().unwrap();
            }
            File::open(&path, "r").unwrap().group("foo").unwrap();

            // writing to file doesn't corrupt the userblock
            File::open(&path, "r+").unwrap().create_group("foo/bar").unwrap();
            {
                let mut reader = fs::File::open(&path).unwrap().take(512);
                let mut data: Vec<u8> = Vec::new();
                assert_eq!(reader.read_to_end(&mut data).unwrap(), 512);
                for i in 0usize..512usize {
                    assert_eq!(data[i], (i % 256) as u8);
                }
            }
            File::open(&path, "r").unwrap().group("foo/bar").unwrap();
        })
    }

    #[test]
    pub fn test_close_automatic() {
        // File going out of scope should just close its own handle
        with_tmp_path(|path| {
            let file = File::open(&path, "w").unwrap();
            let group = file.create_group("foo").unwrap();
            let file_copy = group.file().unwrap();
            drop(file);
            assert!(group.is_valid());
            assert!(file_copy.is_valid());
        });
    }

    #[test]
    pub fn test_close_manual() {
        // File::close() should close handles of all related objects
        with_tmp_path(|path| {
            let file = File::open(&path, "w").unwrap();
            let group = file.create_group("foo").unwrap();
            let file_copy = group.file().unwrap();
            file.close();
            assert!(!file.is_valid());
            assert!(!group.is_valid());
            assert!(!file_copy.is_valid());
        })
    }

    #[test]
    pub fn test_core_fd_non_filebacked() {
        silence_errors();
        with_tmp_path(|path| {
            let file = FileBuilder::new().driver("core").filebacked(false).mode("w")
                                         .open(&path).unwrap();
            file.create_group("x").unwrap();
            assert!(file.is_valid());
            file.close();
            assert!(fs::metadata(&path).is_err());
            assert_err!(FileBuilder::new().driver("core").mode("r").open(&path),
                "unable to open file");
        })
    }

    #[test]
    pub fn test_core_fd_filebacked() {
        with_tmp_path(|path| {
            let file = FileBuilder::new().driver("core").filebacked(true).mode("w")
                                         .open(&path).unwrap();
            assert!(file.is_valid());
            file.create_group("bar").unwrap();
            file.close();
            assert!(fs::metadata(&path).is_ok());
            File::open(&path, "r").unwrap().group("bar").unwrap();
        })
    }

    #[test]
    pub fn test_core_fd_existing_file() {
        with_tmp_path(|path| {
            File::open(&path, "w").unwrap().create_group("baz").unwrap();
            FileBuilder::new().driver("core").mode("r").open(&path).unwrap().group("baz").unwrap();
        })
    }

    #[test]
    pub fn test_sec2_fd() {
        with_tmp_path(|path| {
            FileBuilder::new().driver("sec2").mode("w").open(&path).unwrap()
                              .create_group("foo").unwrap();
            FileBuilder::new().driver("sec2").mode("r").open(&path).unwrap()
                              .group("foo").unwrap();
        })
    }

    #[test]
    pub fn test_stdio_fd() {
        with_tmp_path(|path| {
            FileBuilder::new().driver("stdio").mode("w").open(&path).unwrap()
                              .create_group("qwe").unwrap();
            FileBuilder::new().driver("stdio").mode("r").open(&path).unwrap()
                              .group("qwe").unwrap();
        })
    }

    #[test]
    pub fn test_debug_display() {
        with_tmp_dir(|dir| {
            let path = dir.join("qwe.h5");
            let file = File::open(&path, "w").unwrap();
            assert_eq!(format!("{}", file), "<HDF5 file: \"qwe.h5\" (read/write)>");
            assert_eq!(format!("{:?}", file), "<HDF5 file: \"qwe.h5\" (read/write)>");
            file.close();
            assert_eq!(format!("{}", file), "<HDF5 file: invalid id>");
            assert_eq!(format!("{:?}", file), "<HDF5 file: invalid id>");
            drop(file);
            let file = File::open(&path, "r").unwrap();
            assert_eq!(format!("{}", file), "<HDF5 file: \"qwe.h5\" (read-only)>");
            assert_eq!(format!("{:?}", file), "<HDF5 file: \"qwe.h5\" (read-only)>");
        })
    }
}
