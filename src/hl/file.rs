use std::fmt::{self, Debug};
use std::ops::Deref;
use std::path::Path;

use hdf5_sys::{
    h5f::{
        H5Fclose, H5Fcreate, H5Fflush, H5Fget_access_plist, H5Fget_create_plist, H5Fget_filesize,
        H5Fget_freespace, H5Fget_intent, H5Fget_obj_count, H5Fget_obj_ids, H5Fopen,
        H5F_ACC_DEFAULT, H5F_ACC_EXCL, H5F_ACC_RDONLY, H5F_ACC_RDWR, H5F_ACC_TRUNC, H5F_OBJ_ALL,
        H5F_OBJ_FILE, H5F_SCOPE_LOCAL,
    },
    h5p::{H5Pcreate, H5Pset_fapl_core, H5Pset_fapl_sec2, H5Pset_fapl_stdio, H5Pset_userblock},
};

use crate::globals::{H5P_FILE_ACCESS, H5P_FILE_CREATE};
use crate::hl::plist::file_access::FileAccess;
use crate::hl::plist::file_create::FileCreate;
use crate::internal_prelude::*;

/// Represents the HDF5 file object.
#[repr(transparent)]
#[derive(Clone)]
pub struct File(Handle);

impl ObjectClass for File {
    const NAME: &'static str = "file";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_FILE];

    fn from_handle(handle: Handle) -> Self {
        File(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    fn short_repr(&self) -> Option<String> {
        let basename = match Path::new(&self.filename()).file_name() {
            Some(s) => s.to_string_lossy().into_owned(),
            None => "".to_owned(),
        };
        let mode = if self.is_read_only() { "read-only" } else { "read/write" };
        Some(format!("\"{}\" ({})", basename, mode))
    }
}

impl Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_fmt(f)
    }
}

impl Deref for File {
    type Target = Group;

    fn deref(&self) -> &Group {
        unsafe { self.transmute() }
    }
}

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
    pub fn open<P: AsRef<Path>>(filename: P, mode: &str) -> Result<Self> {
        FileBuilder::new().mode(mode).open(filename)
    }

    pub fn with_options() -> FileBuilder {
        FileBuilder::new()
    }

    /// Returns the file size in bytes (or 0 if the file handle is invalid).
    pub fn size(&self) -> u64 {
        h5get_d!(H5Fget_filesize(self.id()): hsize_t) as _
    }

    /// Returns the free space in the file in bytes (or 0 if the file handle is invalid).
    pub fn free_space(&self) -> u64 {
        h5lock!(H5Fget_freespace(self.id())).max(0) as _
    }

    /// Returns true if the file was opened in a read-only mode.
    pub fn is_read_only(&self) -> bool {
        h5get!(H5Fget_intent(self.id()): c_uint).unwrap_or(H5F_ACC_DEFAULT) != H5F_ACC_RDWR
    }

    /// Returns the userblock size in bytes (or 0 if the file handle is invalid).
    pub fn userblock(&self) -> u64 {
        h5lock!(self.fcpl().map(|p| p.userblock()).unwrap_or(0))
    }

    /// Flushes the file to the storage medium.
    pub fn flush(&self) -> Result<()> {
        // TODO: &mut self?
        h5call!(H5Fflush(self.id(), H5F_SCOPE_LOCAL)).and(Ok(()))
    }

    /// Get objects ids of the contained objects. Note: these are borrowed references.
    fn get_obj_ids(&self, types: c_uint) -> Vec<hid_t> {
        h5lock!({
            let count = h5call!(H5Fget_obj_count(self.id(), types)).unwrap_or(0) as size_t;
            if count > 0 {
                let mut ids: Vec<hid_t> = Vec::with_capacity(count as _);
                unsafe {
                    ids.set_len(count as _);
                }
                if h5call!(H5Fget_obj_ids(self.id(), types, count, ids.as_mut_ptr())).is_ok() {
                    ids.retain(|id| *id != self.id());
                    return ids;
                }
            }
            Vec::new()
        })
    }

    /// Closes the file and invalidates all open handles for contained objects.
    pub fn close(self) {
        h5lock!({
            let file_ids = self.get_obj_ids(H5F_OBJ_FILE);
            let object_ids = self.get_obj_ids(H5F_OBJ_ALL & !H5F_OBJ_FILE);
            for file_id in &file_ids {
                if let Ok(handle) = Handle::try_new(*file_id) {
                    handle.decref_full();
                }
            }
            for object_id in &object_ids {
                if let Ok(handle) = Handle::try_new(*object_id) {
                    handle.decref_full();
                }
            }
            H5Fclose(self.id());
            while self.is_valid() {
                self.0.decref();
            }
            self.0.decref();
        })
    }

    /// Returns a copy of the file access property list.
    pub fn get_access_plist(&self) -> Result<FileAccess> {
        h5lock!(FileAccess::from_id(h5try!(H5Fget_access_plist(self.id()))))
    }

    /// A short alias for `get_access_plist()`.
    pub fn fapl(&self) -> Result<FileAccess> {
        self.get_access_plist()
    }

    /// Returns a copy of the file creation property list.
    pub fn get_create_plist(&self) -> Result<FileCreate> {
        h5lock!(FileCreate::from_id(h5try!(H5Fget_create_plist(self.id()))))
    }

    /// A short alias for `get_create_plist()`.
    pub fn fcpl(&self) -> Result<FileCreate> {
        self.get_create_plist()
    }
}

pub struct FileBuilder {
    driver: String,
    mode: String,
    userblock: u64,
    filebacked: bool,
    increment: u32,
}

impl Default for FileBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FileBuilder {
    pub fn new() -> Self {
        Self {
            driver: "sec2".to_owned(),
            mode: "r".to_owned(),
            userblock: 0,
            filebacked: false,
            increment: 64 * 1024 * 1024,
        }
    }

    pub fn driver(&mut self, driver: &str) -> &mut Self {
        self.driver = driver.into();
        self
    }

    pub fn mode(&mut self, mode: &str) -> &mut Self {
        self.mode = mode.into();
        self
    }

    pub fn userblock(&mut self, userblock: u64) -> &mut Self {
        self.userblock = userblock;
        self
    }

    pub fn filebacked(&mut self, filebacked: bool) -> &mut Self {
        self.filebacked = filebacked;
        self
    }

    #[allow(dead_code)]
    pub fn increment(&mut self, increment: u32) -> &mut Self {
        self.increment = increment;
        self
    }

    fn make_fapl(&self) -> Result<PropertyList> {
        h5lock!({
            let fapl = PropertyList::from_id(h5try!(H5Pcreate(*H5P_FILE_ACCESS)))?;
            match self.driver.as_ref() {
                "sec2" => h5try!(H5Pset_fapl_sec2(fapl.id())),
                "stdio" => h5try!(H5Pset_fapl_stdio(fapl.id())),
                "core" => {
                    h5try!(H5Pset_fapl_core(fapl.id(), self.increment as _, self.filebacked as _))
                }
                _ => fail!(format!("Invalid file driver: {}", self.driver)),
            };
            Ok(fapl)
        })
    }

    fn open_file<P: AsRef<Path>>(&self, filename: P, write: bool) -> Result<File> {
        ensure!(self.userblock == 0, "Cannot specify userblock when opening a file");
        h5lock!({
            let fapl = self.make_fapl()?;
            let flags = if write { H5F_ACC_RDWR } else { H5F_ACC_RDONLY };
            let filename = filename.as_ref();
            match filename.to_str() {
                Some(filename) => {
                    let filename = to_cstring(filename)?;
                    File::from_id(h5try!(H5Fopen(filename.as_ptr(), flags, fapl.id())))
                }
                None => fail!("Invalid UTF-8 in file name: {:?}", filename),
            }
        })
    }

    fn create_file<P: AsRef<Path>>(&self, filename: P, exclusive: bool) -> Result<File> {
        h5lock!({
            let fcpl = PropertyList::from_id(h5try!(H5Pcreate(*H5P_FILE_CREATE)))?;
            h5try!(H5Pset_userblock(fcpl.id(), self.userblock));
            let fapl = self.make_fapl()?;
            let flags = if exclusive { H5F_ACC_EXCL } else { H5F_ACC_TRUNC };
            let filename = filename.as_ref();
            match filename.to_str() {
                Some(filename) => {
                    let filename = to_cstring(filename)?;
                    File::from_id(h5try!(H5Fcreate(filename.as_ptr(), flags, fcpl.id(), fapl.id())))
                }
                None => fail!("Invalid UTF-8 in file name: {:?}", filename),
            }
        })
    }

    pub fn open<P: AsRef<Path>>(&self, filename: P) -> Result<File> {
        match self.mode.as_ref() {
            "r" => self.open_file(&filename, false),
            "r+" => self.open_file(&filename, true),
            "w" => self.create_file(&filename, false),
            "w-" | "x" => self.create_file(&filename, true),
            "a" => match self.open_file(&filename, true) {
                Ok(file) => Ok(file),
                _ => self.create_file(&filename, true),
            },
            _ => fail!("Invalid file access mode, expected r|r+|w|w-|x|a"),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::hdf5_version;
    use crate::internal_prelude::*;
    use std::fs;
    use std::io::{Read, Write};

    #[test]
    pub fn test_invalid_mode() {
        with_tmp_dir(|dir| {
            assert_err!(File::open(&dir, "foo"), "Invalid file access mode");
        })
    }

    #[test]
    pub fn test_is_read_only() {
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
            assert_err!(
                file.create_group("bar"),
                "unable to create group: no write intent on file"
            );
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
            let orig_size = fs::metadata(file.filename()).unwrap().len();
            assert!(file.size() > orig_size);
            if hdf5_version() >= (1, 10, 0) {
                assert_ne!(orig_size, 0);
            } else {
                assert_eq!(orig_size, 0);
            }
            assert!(file.flush().is_ok());
            assert!(file.size() > 0);
            let new_size = fs::metadata(file.filename()).unwrap().len();
            assert!(new_size > orig_size);
            assert_eq!(file.size(), new_size);
        })
    }

    #[test]
    pub fn test_userblock() {
        with_tmp_file(|file| {
            assert_eq!(file.userblock(), 0);
        });
        with_tmp_path(|path| {
            assert_err!(
                FileBuilder::new().userblock(512).mode("r").open(&path),
                "Cannot specify userblock when opening a file"
            );
            assert_err!(
                FileBuilder::new().userblock(512).mode("r+").open(&path),
                "Cannot specify userblock when opening a file"
            );
            assert_err!(
                FileBuilder::new().userblock(1).mode("w").open(&path),
                "userblock size is non-zero and less than 512"
            );
            FileBuilder::new().userblock(512).mode("w").open(&path).unwrap();
            assert_eq!(File::open(&path, "r").unwrap().userblock(), 512);

            // writing to userblock doesn't corrupt the file
            File::open(&path, "r+").unwrap().create_group("foo").unwrap();
            {
                let mut file = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(false)
                    .open(&path)
                    .unwrap();
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
                for (i, item) in data.iter().cloned().enumerate().take(512) {
                    assert_eq!(item, (i % 256) as u8);
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
            assert!(!group.is_valid());
            assert!(!file_copy.is_valid());
        })
    }

    #[test]
    pub fn test_core_fd_non_filebacked() {
        with_tmp_path(|path| {
            let file =
                FileBuilder::new().driver("core").filebacked(false).mode("w").open(&path).unwrap();
            file.create_group("x").unwrap();
            assert!(file.is_valid());
            file.close();
            assert!(fs::metadata(&path).is_err());
            assert_err!(
                FileBuilder::new().driver("core").mode("r").open(&path),
                "unable to open file"
            );
        })
    }

    #[test]
    pub fn test_core_fd_filebacked() {
        with_tmp_path(|path| {
            let file =
                FileBuilder::new().driver("core").filebacked(true).mode("w").open(&path).unwrap();
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
            FileBuilder::new()
                .driver("sec2")
                .mode("w")
                .open(&path)
                .unwrap()
                .create_group("foo")
                .unwrap();
            FileBuilder::new().driver("sec2").mode("r").open(&path).unwrap().group("foo").unwrap();
        })
    }

    #[test]
    pub fn test_stdio_fd() {
        with_tmp_path(|path| {
            FileBuilder::new()
                .driver("stdio")
                .mode("w")
                .open(&path)
                .unwrap()
                .create_group("qwe")
                .unwrap();
            FileBuilder::new().driver("stdio").mode("r").open(&path).unwrap().group("qwe").unwrap();
        })
    }

    #[test]
    pub fn test_debug() {
        with_tmp_dir(|dir| {
            let path = dir.join("qwe.h5");
            let file = File::open(&path, "w").unwrap();
            assert_eq!(format!("{:?}", file), "<HDF5 file: \"qwe.h5\" (read/write)>");
            let root = file.file().unwrap();
            file.close();
            assert_eq!(format!("{:?}", root), "<HDF5 file: invalid id>");
            let file = File::open(&path, "r").unwrap();
            assert_eq!(format!("{:?}", file), "<HDF5 file: \"qwe.h5\" (read-only)>");
        })
    }
}
