use std::fmt::{self, Debug};
use std::mem;
use std::ops::Deref;
use std::path::Path;

use hdf5_sys::h5f::{
    H5Fclose, H5Fcreate, H5Fflush, H5Fget_access_plist, H5Fget_create_plist, H5Fget_filesize,
    H5Fget_freespace, H5Fget_intent, H5Fget_obj_count, H5Fget_obj_ids, H5Fopen, H5F_ACC_DEFAULT,
    H5F_ACC_EXCL, H5F_ACC_RDONLY, H5F_ACC_RDWR, H5F_ACC_TRUNC, H5F_SCOPE_LOCAL,
};

use crate::hl::plist::{
    file_access::{FileAccess, FileAccessBuilder},
    file_create::{FileCreate, FileCreateBuilder},
};
use crate::internal_prelude::*;

/// File opening mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenMode {
    /// Open a file as read-only, file must exist.
    Read,
    /// Open a file as read/write, file must exist.
    ReadWrite,
    /// Create a file, truncate if exists.
    Create,
    /// Create a file, fail if exists.
    CreateExcl,
    /// Open a file as read/write if exists, create otherwise.
    Append,
}

/// HDF5 file object.
#[repr(transparent)]
#[derive(Clone)]
pub struct File(Handle);

impl ObjectClass for File {
    const NAME: &'static str = "file";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_FILE];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    fn short_repr(&self) -> Option<String> {
        let basename = match Path::new(&self.filename()).file_name() {
            Some(s) => s.to_string_lossy().into_owned(),
            None => String::new(),
        };
        let mode = if self.is_read_only() { "read-only" } else { "read/write" };
        Some(format!("\"{basename}\" ({mode})"))
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
    /// Opens a file as read-only, file must exist.
    pub fn open<P: AsRef<Path>>(filename: P) -> Result<Self> {
        Self::open_as(filename, OpenMode::Read)
    }

    /// Opens a file as read/write, file must exist.
    pub fn open_rw<P: AsRef<Path>>(filename: P) -> Result<Self> {
        Self::open_as(filename, OpenMode::ReadWrite)
    }

    /// Creates a file, truncates if exists.
    pub fn create<P: AsRef<Path>>(filename: P) -> Result<Self> {
        Self::open_as(filename, OpenMode::Create)
    }

    /// Creates a file, fails if exists.
    pub fn create_excl<P: AsRef<Path>>(filename: P) -> Result<Self> {
        Self::open_as(filename, OpenMode::CreateExcl)
    }

    /// Opens a file as read/write if exists, creates otherwise.
    pub fn append<P: AsRef<Path>>(filename: P) -> Result<Self> {
        Self::open_as(filename, OpenMode::Append)
    }

    /// Opens a file in a given mode.
    pub fn open_as<P: AsRef<Path>>(filename: P, mode: OpenMode) -> Result<Self> {
        FileBuilder::new().open_as(filename, mode)
    }

    /// Opens a file with custom file-access and file-creation options.
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
        h5call!(H5Fflush(self.id(), H5F_SCOPE_LOCAL)).and(Ok(()))
    }

    /// Returns objects IDs of the contained objects. NOTE: these are borrowed references.
    #[allow(unused)]
    fn get_obj_ids(&self, types: c_uint) -> Vec<hid_t> {
        h5lock!({
            let count = h5call!(H5Fget_obj_count(self.id(), types)).unwrap_or(0) as size_t;
            if count > 0 {
                let mut ids: Vec<hid_t> = Vec::with_capacity(count as _);
                if h5call!(H5Fget_obj_ids(self.id(), types, count, ids.as_mut_ptr())).is_ok() {
                    unsafe {
                        ids.set_len(count as _);
                    }
                    ids.retain(|id| *id != self.id());
                    return ids;
                }
            }
            Vec::new()
        })
    }

    /// Closes the file and invalidates all open handles for contained objects.
    pub fn close(self) -> Result<()> {
        let id = self.id();
        // Ensure we only decref once
        mem::forget(self.0);
        h5call!(H5Fclose(id)).map(|_| ())
    }

    /// Returns a copy of the file access property list.
    pub fn access_plist(&self) -> Result<FileAccess> {
        h5lock!(FileAccess::from_id(h5try!(H5Fget_access_plist(self.id()))))
    }

    /// A short alias for `access_plist()`.
    pub fn fapl(&self) -> Result<FileAccess> {
        self.access_plist()
    }

    /// Returns a copy of the file creation property list.
    pub fn create_plist(&self) -> Result<FileCreate> {
        h5lock!(FileCreate::from_id(h5try!(H5Fget_create_plist(self.id()))))
    }

    /// A short alias for `create_plist()`.
    pub fn fcpl(&self) -> Result<FileCreate> {
        self.create_plist()
    }
}

/// File builder allowing to customize file access/creation property lists.
#[derive(Default, Clone, Debug)]
pub struct FileBuilder {
    fapl: FileAccessBuilder,
    fcpl: FileCreateBuilder,
}

impl FileBuilder {
    /// Creates a new file builder with default property lists.
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens a file as read-only, file must exist.
    pub fn open<P: AsRef<Path>>(&self, filename: P) -> Result<File> {
        self.open_as(filename, OpenMode::Read)
    }

    /// Opens a file as read/write, file must exist.
    pub fn open_rw<P: AsRef<Path>>(&self, filename: P) -> Result<File> {
        self.open_as(filename, OpenMode::ReadWrite)
    }

    /// Creates a file, truncates if exists.
    pub fn create<P: AsRef<Path>>(&self, filename: P) -> Result<File> {
        self.open_as(filename, OpenMode::Create)
    }

    /// Creates a file, fails if exists.
    pub fn create_excl<P: AsRef<Path>>(&self, filename: P) -> Result<File> {
        self.open_as(filename, OpenMode::CreateExcl)
    }

    /// Opens a file as read/write if exists, creates otherwise.
    pub fn append<P: AsRef<Path>>(&self, filename: P) -> Result<File> {
        self.open_as(filename, OpenMode::Append)
    }

    /// Opens a file in a given mode.
    pub fn open_as<P: AsRef<Path>>(&self, filename: P, mode: OpenMode) -> Result<File> {
        let filename = filename.as_ref();
        if mode == OpenMode::Append {
            if let Ok(file) = self.open_as(filename, OpenMode::ReadWrite) {
                return Ok(file);
            }
        }
        let filename = to_cstring(
            filename.to_str().ok_or_else(|| format!("Invalid UTF-8 in file name: {filename:?}"))?,
        )?;
        let flags = match mode {
            OpenMode::Read => H5F_ACC_RDONLY,
            OpenMode::ReadWrite => H5F_ACC_RDWR,
            OpenMode::Create => H5F_ACC_TRUNC,
            OpenMode::CreateExcl | OpenMode::Append => H5F_ACC_EXCL,
        };
        let fname_ptr = filename.as_ptr();
        h5lock!({
            let fapl = self.fapl.finish()?;
            match mode {
                OpenMode::Read | OpenMode::ReadWrite => {
                    File::from_id(h5try!(H5Fopen(fname_ptr, flags, fapl.id())))
                }
                _ => {
                    let fcpl = self.fcpl.finish()?;
                    File::from_id(h5try!(H5Fcreate(fname_ptr, flags, fcpl.id(), fapl.id())))
                }
            }
        })
    }

    // File Access Property List

    /// Sets current file access property list to a given one.
    pub fn set_access_plist(&mut self, fapl: &FileAccess) -> Result<&mut Self> {
        FileAccessBuilder::from_plist(fapl).map(|fapl| {
            self.fapl = fapl;
            self
        })
    }

    /// A short alias for `set_access_plist()`.
    pub fn set_fapl(&mut self, fapl: &FileAccess) -> Result<&mut Self> {
        self.set_access_plist(fapl)
    }

    /// Returns the builder object for the file access property list.
    pub fn access_plist(&mut self) -> &mut FileAccessBuilder {
        &mut self.fapl
    }

    /// A short alias for `access_plist()`.
    pub fn fapl(&mut self) -> &mut FileAccessBuilder {
        self.access_plist()
    }

    /// Allows accessing the builder object for the file access property list.
    pub fn with_access_plist<F>(&mut self, func: F) -> &mut Self
    where
        F: Fn(&mut FileAccessBuilder) -> &mut FileAccessBuilder,
    {
        func(&mut self.fapl);
        self
    }

    /// A short alias for `with_access_plist()`.
    pub fn with_fapl<F>(&mut self, func: F) -> &mut Self
    where
        F: Fn(&mut FileAccessBuilder) -> &mut FileAccessBuilder,
    {
        self.with_access_plist(func)
    }

    // File Creation Property List

    /// Sets current file creation property list to a given one.
    pub fn set_create_plist(&mut self, fcpl: &FileCreate) -> Result<&mut Self> {
        FileCreateBuilder::from_plist(fcpl).map(|fcpl| {
            self.fcpl = fcpl;
            self
        })
    }

    /// A short alias for `set_create_plist()`.
    pub fn set_fcpl(&mut self, fcpl: &FileCreate) -> Result<&mut Self> {
        self.set_create_plist(fcpl)
    }

    /// Returns the builder object for the file creation property list.
    pub fn create_plist(&mut self) -> &mut FileCreateBuilder {
        &mut self.fcpl
    }

    /// A short alias for `create_plist()`.
    pub fn fcpl(&mut self) -> &mut FileCreateBuilder {
        self.create_plist()
    }

    /// Allows accessing the builder object for the file creation property list.
    pub fn with_create_plist<F>(&mut self, func: F) -> &mut Self
    where
        F: Fn(&mut FileCreateBuilder) -> &mut FileCreateBuilder,
    {
        func(&mut self.fcpl);
        self
    }

    /// A short alias for `with_create_plist()`.
    pub fn with_fcpl<F>(&mut self, func: F) -> &mut Self
    where
        F: Fn(&mut FileCreateBuilder) -> &mut FileCreateBuilder,
    {
        self.with_create_plist(func)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::internal_prelude::*;
    use std::fs;
    use std::io::{Read, Write};

    #[test]
    pub fn test_is_read_only() {
        with_tmp_path(|path| {
            assert!(!File::create(&path).unwrap().is_read_only());
            assert!(File::open(&path).unwrap().is_read_only());
            assert!(!File::open_rw(&path).unwrap().is_read_only());
            assert!(!File::append(&path).unwrap().is_read_only());
        });
        with_tmp_path(|path| {
            assert!(!File::append(&path).unwrap().is_read_only());
        });
        with_tmp_path(|path| {
            assert!(!File::create_excl(&path).unwrap().is_read_only());
        });
    }

    #[test]
    pub fn test_unable_to_open() {
        with_tmp_dir(|dir| {
            assert_err_re!(File::open(&dir), "unable to (?:synchronously )?open file");
            assert_err_re!(File::open_rw(&dir), "unable to (?:synchronously )?open file");
            assert_err_re!(File::create_excl(&dir), "unable to (?:synchronously )?create file");
            assert_err_re!(File::create(&dir), "unable to (?:synchronously )?create file");
            assert_err_re!(File::append(&dir), "unable to (?:synchronously )?create file");
        });
        with_tmp_path(|path| {
            fs::File::create(&path).unwrap().write_all(b"foo").unwrap();
            assert!(fs::metadata(&path).is_ok());
            assert_err_re!(File::open(&path), "unable to (?:synchronously )?open file");
        })
    }

    #[test]
    pub fn test_file_create() {
        with_tmp_path(|path| {
            File::create(&path).unwrap().create_group("foo").unwrap();
            assert_err_re!(
                File::create(&path).unwrap().group("foo"),
                "unable to (?:synchronously )?open group"
            );
        });
    }

    #[test]
    pub fn test_file_create_excl() {
        with_tmp_path(|path| {
            File::create_excl(&path).unwrap();
            assert_err_re!(File::create_excl(&path), "unable to (?:synchronously )?create file");
        });
    }

    #[test]
    pub fn test_file_append() {
        with_tmp_path(|path| {
            File::append(&path).unwrap().create_group("foo").unwrap();
            File::append(&path).unwrap().group("foo").unwrap();
        });
    }

    #[test]
    pub fn test_file_open() {
        with_tmp_path(|path| {
            File::create(&path).unwrap().create_group("foo").unwrap();
            let file = File::open(&path).unwrap();
            file.group("foo").unwrap();
            assert_err_re!(
                file.create_group("bar"),
                "unable to (?:synchronously )?create group: no write intent on file"
            );
            assert_err!(File::open("/foo/bar/baz"), "unable to open file");
        });
    }

    #[test]
    pub fn test_file_open_rw() {
        with_tmp_path(|path| {
            File::create(&path).unwrap().create_group("foo").unwrap();
            let file = File::open_rw(&path).unwrap();
            file.group("foo").unwrap();
            file.create_group("bar").unwrap();
            assert_err!(File::open_rw("/foo/bar/baz"), "unable to open file");
        });
    }

    #[test]
    pub fn test_flush() {
        with_tmp_file(|file| {
            assert!(file.size() > 0);
            let orig_size = fs::metadata(file.filename()).unwrap().len();
            assert!(file.size() > orig_size);
            #[cfg(feature = "1.10.0")]
            assert_ne!(orig_size, 0);
            #[cfg(not(feature = "1.10.0"))]
            assert_eq!(orig_size, 0);
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
                FileBuilder::new().with_fcpl(|p| p.userblock(1)).create(&path),
                "userblock size is non-zero and less than 512"
            );
            FileBuilder::new().with_fcpl(|p| p.userblock(512)).create(&path).unwrap();
            assert_eq!(File::open(&path).unwrap().userblock(), 512);

            // writing to userblock doesn't corrupt the file
            File::open_rw(&path).unwrap().create_group("foo").unwrap();
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
            File::open(&path).unwrap().group("foo").unwrap();

            // writing to file doesn't corrupt the userblock
            File::open_rw(&path).unwrap().create_group("foo/bar").unwrap();
            {
                let mut reader = fs::File::open(&path).unwrap().take(512);
                let mut data: Vec<u8> = Vec::new();
                assert_eq!(reader.read_to_end(&mut data).unwrap(), 512);
                for (i, item) in data.iter().cloned().enumerate().take(512) {
                    assert_eq!(item, (i % 256) as u8);
                }
            }
            File::open(&path).unwrap().group("foo/bar").unwrap();
        })
    }

    fn rc(id: hid_t) -> Result<hsize_t> {
        h5call!(hdf5_sys::h5i::H5Iget_ref(id)).map(|x| x as _)
    }

    #[test]
    fn test_strong_close() {
        use crate::hl::plist::file_access::FileCloseDegree;
        with_tmp_path(|path| {
            let file = File::with_options()
                .with_fapl(|fapl| fapl.fclose_degree(FileCloseDegree::Strong))
                .create(&path)
                .unwrap();
            assert_eq!(file.refcount(), 1);
            let fileid = file.id();

            let group = file.create_group("foo").unwrap();
            assert_eq!(file.refcount(), 1);
            assert_eq!(group.refcount(), 1);

            let file_copy = group.file().unwrap();
            assert_eq!(group.refcount(), 1);
            assert_eq!(file.refcount(), 2);
            assert_eq!(file_copy.refcount(), 2);

            drop(file);
            assert_eq!(rc(fileid).unwrap(), 1);
            assert_eq!(group.refcount(), 1);
            assert_eq!(file_copy.refcount(), 1);

            h5lock!({
                // Lock to ensure fileid does not get overwritten
                let groupid = group.id();
                drop(file_copy);
                assert!(rc(fileid).is_err());
                assert!(rc(groupid).is_err());
                assert!(!group.is_valid());
                drop(group);
            });
        });
    }

    #[test]
    fn test_weak_close() {
        use crate::hl::plist::file_access::FileCloseDegree;
        with_tmp_path(|path| {
            let file = File::with_options()
                .with_fapl(|fapl| fapl.fclose_degree(FileCloseDegree::Weak))
                .create(&path)
                .unwrap();
            assert_eq!(file.refcount(), 1);
            let fileid = file.id();

            let group = file.create_group("foo").unwrap();
            assert_eq!(file.refcount(), 1);
            assert_eq!(group.refcount(), 1);

            let file_copy = group.file().unwrap();
            assert_eq!(group.refcount(), 1);
            assert_eq!(file.refcount(), 2);
            assert_eq!(file_copy.refcount(), 2);

            drop(file);
            assert_eq!(rc(fileid).unwrap(), 1);
            assert_eq!(group.refcount(), 1);
            assert_eq!(file_copy.refcount(), 1);

            h5lock!({
                // Lock to ensure fileid does not get overwritten
                drop(file_copy);
                assert!(rc(fileid).is_err());
            });
            assert_eq!(group.refcount(), 1);
        });
    }

    #[test]
    pub fn test_close_automatic() {
        // File going out of scope should just close its own handle
        with_tmp_path(|path| {
            let file = File::create(&path).unwrap();
            let group = file.create_group("foo").unwrap();
            let file_copy = group.file().unwrap();
            drop(file);
            assert!(group.is_valid());
            assert!(file_copy.is_valid());
        });
    }

    #[test]
    pub fn test_core_fd_non_filebacked() {
        with_tmp_path(|path| {
            let file =
                FileBuilder::new().with_fapl(|p| p.core_filebacked(false)).create(&path).unwrap();
            file.create_group("x").unwrap();
            assert!(file.is_valid());
            file.close().unwrap();
            assert!(fs::metadata(&path).is_err());
            assert_err!(
                FileBuilder::new().with_fapl(|p| p.core()).open(&path),
                "unable to open file"
            );
        })
    }

    #[test]
    pub fn test_core_fd_filebacked() {
        with_tmp_path(|path| {
            let file =
                FileBuilder::new().with_fapl(|p| p.core_filebacked(true)).create(&path).unwrap();
            assert!(file.is_valid());
            file.create_group("bar").unwrap();
            file.close().unwrap();
            assert!(fs::metadata(&path).is_ok());
            File::open(&path).unwrap().group("bar").unwrap();
        })
    }

    #[test]
    pub fn test_core_fd_existing_file() {
        with_tmp_path(|path| {
            File::create(&path).unwrap().create_group("baz").unwrap();
            FileBuilder::new().with_fapl(|p| p.core()).open(&path).unwrap().group("baz").unwrap();
        })
    }

    #[test]
    pub fn test_sec2_fd() {
        with_tmp_path(|path| {
            FileBuilder::new()
                .with_fapl(|p| p.sec2())
                .create(&path)
                .unwrap()
                .create_group("foo")
                .unwrap();
            FileBuilder::new().with_fapl(|p| p.sec2()).open(&path).unwrap().group("foo").unwrap();
        })
    }

    #[test]
    pub fn test_stdio_fd() {
        with_tmp_path(|path| {
            FileBuilder::new()
                .with_fapl(|p| p.stdio())
                .create(&path)
                .unwrap()
                .create_group("qwe")
                .unwrap();
            FileBuilder::new().with_fapl(|p| p.stdio()).open(&path).unwrap().group("qwe").unwrap();
        })
    }

    #[test]
    pub fn test_debug() {
        with_tmp_dir(|dir| {
            let path = dir.join("qwe.h5");
            let file = File::create(&path).unwrap();
            assert_eq!(format!("{:?}", file), "<HDF5 file: \"qwe.h5\" (read/write)>");
            file.close().unwrap();
            let root = File::from_handle(Handle::invalid());
            assert_eq!(format!("{:?}", root), "<HDF5 file: invalid id>");
            let file = File::open(&path).unwrap();
            assert_eq!(format!("{:?}", file), "<HDF5 file: \"qwe.h5\" (read-only)>");
        })
    }
}
