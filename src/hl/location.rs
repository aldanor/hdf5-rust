use std::fmt::{self, Debug};
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::ptr;

#[allow(deprecated)]
use hdf5_sys::h5o::H5Oset_comment;
#[cfg(hdf5_1_10_3)]
use hdf5_sys::h5o::H5O_INFO_BASIC;
#[cfg(hdf5_1_12_0)]
use hdf5_sys::h5o::{
    H5O_info2_t, H5O_token_t, H5Oget_info3, H5Oget_info_by_name3, H5Oopen_by_token,
};
#[cfg(not(hdf5_1_10_3))]
use hdf5_sys::h5o::{H5Oget_info1, H5Oget_info_by_name1};
#[cfg(all(hdf5_1_10_3, not(hdf5_1_12_0)))]
use hdf5_sys::h5o::{H5Oget_info2, H5Oget_info_by_name2};
#[cfg(not(hdf5_1_12_0))]
use hdf5_sys::{h5::haddr_t, h5o::H5O_info1_t, h5o::H5Oopen_by_addr};
use hdf5_sys::{
    h5a::H5Aopen,
    h5f::H5Fget_name,
    h5i::{H5Iget_file_id, H5Iget_name},
    h5o::{H5O_type_t, H5Oget_comment},
};

use crate::internal_prelude::*;

use super::attribute::AttributeBuilderEmpty;

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
    #[deprecated(note = "attributes are preferred to comments")]
    pub fn set_comment(&self, comment: &str) -> Result<()> {
        // TODO: &mut self?
        let comment = to_cstring(comment)?;
        #[allow(deprecated)]
        h5call!(H5Oset_comment(self.id(), comment.as_ptr())).and(Ok(()))
    }

    /// Clear the commment attached to the named object.
    #[deprecated(note = "attributes are preferred to comments")]
    pub fn clear_comment(&self) -> Result<()> {
        // TODO: &mut self?
        #[allow(deprecated)]
        h5call!(H5Oset_comment(self.id(), ptr::null_mut())).and(Ok(()))
    }

    pub fn new_attr<T: H5Type>(&self) -> AttributeBuilderEmpty {
        AttributeBuilder::new(self).empty::<T>()
    }

    pub fn new_attr_builder(&self) -> AttributeBuilder {
        AttributeBuilder::new(self)
    }

    pub fn attr(&self, name: &str) -> Result<Attribute> {
        let name = to_cstring(name)?;
        Attribute::from_id(h5try!(H5Aopen(self.id(), name.as_ptr(), H5P_DEFAULT)))
    }

    pub fn attr_names(&self) -> Result<Vec<String>> {
        Attribute::attr_names(self)
    }

    pub fn get_info(&self) -> Result<LocationInfo> {
        H5O_get_info(self.id())
    }

    pub fn loc_type(&self) -> Result<LocationType> {
        Ok(self.get_info()?.loc_type)
    }

    pub fn get_info_by_name(&self, name: &str) -> Result<LocationInfo> {
        let name = to_cstring(name)?;
        H5O_get_info_by_name(self.id(), name.as_ptr())
    }

    pub fn open_by_token(&self, token: LocationToken) -> Result<Self> {
        H5O_open_by_token(self.id(), token)
    }
}

#[cfg(hdf5_1_12_0)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocationToken(H5O_token_t);

#[cfg(not(hdf5_1_12_0))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocationToken(haddr_t);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocationType {
    Group,
    Dataset,
    NamedDatatype,
    #[cfg(hdf5_1_12_0)]
    TypeMap,
}

impl From<H5O_type_t> for LocationType {
    fn from(loc_type: H5O_type_t) -> Self {
        // we're assuming here that if a C API call returns H5O_TYPE_UNKNOWN (-1), then
        // an error has occured anyway and has been pushed on the error stack so we'll
        // catch it, and the value of -1 will never reach this conversion function
        match loc_type {
            H5O_type_t::H5O_TYPE_DATASET => Self::Dataset,
            H5O_type_t::H5O_TYPE_NAMED_DATATYPE => Self::NamedDatatype,
            #[cfg(hdf5_1_12_0)]
            H5O_type_t::H5O_TYPE_MAP => Self::TypeMap,
            _ => Self::Group, // see the comment above
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocationInfo {
    pub fileno: u64,
    pub token: LocationToken,
    pub loc_type: LocationType,
    pub refcount: usize,
    pub atime: i64,
    pub mtime: i64,
    pub ctime: i64,
    pub btime: i64,
    pub num_attrs: usize,
}

#[cfg(not(hdf5_1_12_0))]
impl From<H5O_info1_t> for LocationInfo {
    fn from(info: H5O_info1_t) -> Self {
        Self {
            fileno: info.fileno as _,
            token: LocationToken(info.addr),
            loc_type: info.type_.into(),
            refcount: info.rc as _,
            atime: info.atime as _,
            mtime: info.mtime as _,
            ctime: info.ctime as _,
            btime: info.btime as _,
            num_attrs: info.num_attrs as _,
        }
    }
}

#[cfg(hdf5_1_12_0)]
impl From<H5O_info2_t> for LocationInfo {
    fn from(info: H5O_info2_t) -> Self {
        Self {
            fileno: info.fileno as _,
            token: LocationToken(info.token),
            loc_type: info.type_.into(),
            refcount: info.rc as _,
            atime: info.atime as _,
            mtime: info.mtime as _,
            ctime: info.ctime as _,
            btime: info.btime as _,
            num_attrs: info.num_attrs as _,
        }
    }
}

#[allow(non_snake_case)]
fn H5O_get_info(loc_id: hid_t) -> Result<LocationInfo> {
    let mut info_buf = MaybeUninit::uninit();
    let info_ptr = info_buf.as_mut_ptr();
    #[cfg(hdf5_1_12_0)]
    h5call!(H5Oget_info3(loc_id, info_ptr, H5O_INFO_BASIC))?;
    #[cfg(all(hdf5_1_10_3, not(hdf5_1_12_0)))]
    h5call!(H5Oget_info2(loc_id, info_ptr, H5O_INFO_BASIC))?;
    #[cfg(not(hdf5_1_10_3))]
    h5call!(H5Oget_info1(loc_id, info_ptr))?;
    let info = unsafe { info_buf.assume_init() };
    Ok(info.into())
}

#[allow(non_snake_case)]
fn H5O_get_info_by_name(loc_id: hid_t, name: *const c_char) -> Result<LocationInfo> {
    let mut info_buf = MaybeUninit::uninit();
    let info_ptr = info_buf.as_mut_ptr();
    #[cfg(hdf5_1_12_0)]
    h5call!(H5Oget_info_by_name3(loc_id, name, info_ptr, H5O_INFO_BASIC, H5P_DEFAULT))?;
    #[cfg(all(hdf5_1_10_3, not(hdf5_1_12_0)))]
    h5call!(H5Oget_info_by_name2(loc_id, name, info_ptr, H5O_INFO_BASIC, H5P_DEFAULT))?;
    #[cfg(not(hdf5_1_10_3))]
    h5call!(H5Oget_info_by_name1(loc_id, name, info_ptr, H5P_DEFAULT))?;
    let info = unsafe { info_buf.assume_init() };
    Ok(info.into())
}

#[allow(non_snake_case)]
fn H5O_open_by_token(loc_id: hid_t, token: LocationToken) -> Result<Location> {
    #[cfg(not(hdf5_1_12_0))]
    {
        Location::from_id(h5call!(H5Oopen_by_addr(loc_id, token.0))?)
    }
    #[cfg(hdf5_1_12_0)]
    {
        Location::from_id(h5call!(H5Oopen_by_token(loc_id, token.0))?)
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
        #[allow(deprecated)]
        with_tmp_file(|file| {
            assert!(file.comment().is_none());
            assert!(file.set_comment("foo").is_ok());
            assert_eq!(file.comment().unwrap(), "foo");
            assert!(file.clear_comment().is_ok());
            assert!(file.comment().is_none());
        })
    }
}
