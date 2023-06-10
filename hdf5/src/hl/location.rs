use std::fmt::{self, Debug};
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::ptr;

#[allow(deprecated)]
use hdf5_sys::h5o::H5Oset_comment;
#[cfg(feature = "1.12.0")]
use hdf5_sys::h5o::{
    H5O_info2_t, H5O_token_t, H5Oget_info3, H5Oget_info_by_name3, H5Oopen_by_token,
};
#[cfg(not(feature = "1.10.3"))]
use hdf5_sys::h5o::{H5Oget_info1, H5Oget_info_by_name1};
#[cfg(all(feature = "1.10.3", not(feature = "1.12.0")))]
use hdf5_sys::h5o::{H5Oget_info2, H5Oget_info_by_name2};
#[cfg(feature = "1.10.3")]
use hdf5_sys::h5o::{H5O_INFO_BASIC, H5O_INFO_NUM_ATTRS, H5O_INFO_TIME};
#[cfg(not(feature = "1.12.0"))]
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
        h5lock!(get_h5_str(|m, s| H5Iget_name(self.id(), m, s)).unwrap_or_else(|_| String::new()))
    }

    /// Returns the name of the file containing the named object (or the file itself).
    pub fn filename(&self) -> String {
        // TODO: should this return Result<String> or an empty string if it fails?
        h5lock!(get_h5_str(|m, s| H5Fget_name(self.id(), m, s)).unwrap_or_else(|_| String::new()))
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

    pub fn loc_info(&self) -> Result<LocationInfo> {
        H5O_get_info(self.id(), true)
    }

    pub fn loc_type(&self) -> Result<LocationType> {
        Ok(H5O_get_info(self.id(), false)?.loc_type)
    }

    pub fn loc_info_by_name(&self, name: &str) -> Result<LocationInfo> {
        let name = to_cstring(name)?;
        H5O_get_info_by_name(self.id(), name.as_ptr(), true)
    }

    pub fn loc_type_by_name(&self, name: &str) -> Result<LocationType> {
        let name = to_cstring(name)?;
        Ok(H5O_get_info_by_name(self.id(), name.as_ptr(), false)?.loc_type)
    }

    pub fn open_by_token(&self, token: LocationToken) -> Result<Self> {
        H5O_open_by_token(self.id(), token)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocationToken(
    #[cfg(not(feature = "1.12.0"))] haddr_t,
    #[cfg(feature = "1.12.0")] H5O_token_t,
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocationType {
    Group,
    Dataset,
    NamedDatatype,
    #[cfg(feature = "1.12.0")]
    #[cfg_attr(docrs, doc(cfg(feature = "1.12.0")))]
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
            #[cfg(feature = "1.12.0")]
            H5O_type_t::H5O_TYPE_MAP => Self::TypeMap,
            _ => Self::Group, // see the comment above
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Metadata information describing a [`Location`]
///
/// # Notes
///
/// In order for all timestamps to be filled out, a few conditions must hold:
///
/// - Minimum HDF5 library version is 1.10.3.
/// - Library version lower bound in the file access plist must be set to a least 1.10. This
///   can be done via `FileAccessBuilder::libver_v110` or `FileAccessBuilder::libver_latest`.
/// - For datasets, additionally, time tracking must be enabled (which is disabled
///   by default to improve access performance). This can be done via
///   `DatasetBuilder::track_times`. If tracking is enabled, ctime timestamp will likely be
///   filled out even if library version lower bound is not set), but the other three will
///   be zero.
pub struct LocationInfo {
    /// Number of file where the object is located
    pub fileno: u64,
    /// Object address in file, or a token identifier
    pub token: LocationToken,
    /// Basic location type of the object
    pub loc_type: LocationType,
    /// Number of hard links to the object
    pub num_links: usize,
    /// Access time
    pub atime: i64,
    /// Modification time
    pub mtime: i64,
    /// Change time
    pub ctime: i64,
    /// Birth time
    pub btime: i64,
    /// Number of attributes attached to the object
    pub num_attrs: usize,
}

#[cfg(not(feature = "1.12.0"))]
impl From<H5O_info1_t> for LocationInfo {
    fn from(info: H5O_info1_t) -> Self {
        Self {
            fileno: info.fileno as _,
            token: LocationToken(info.addr),
            loc_type: info.type_.into(),
            num_links: info.rc as _,
            atime: info.atime as _,
            mtime: info.mtime as _,
            ctime: info.ctime as _,
            btime: info.btime as _,
            num_attrs: info.num_attrs as _,
        }
    }
}

#[cfg(feature = "1.12.0")]
impl From<H5O_info2_t> for LocationInfo {
    fn from(info: H5O_info2_t) -> Self {
        Self {
            fileno: info.fileno as _,
            token: LocationToken(info.token),
            loc_type: info.type_.into(),
            num_links: info.rc as _,
            atime: info.atime as _,
            mtime: info.mtime as _,
            ctime: info.ctime as _,
            btime: info.btime as _,
            num_attrs: info.num_attrs as _,
        }
    }
}

#[cfg(feature = "1.10.3")]
fn info_fields(full: bool) -> c_uint {
    if full {
        H5O_INFO_BASIC | H5O_INFO_NUM_ATTRS | H5O_INFO_TIME
    } else {
        H5O_INFO_BASIC
    }
}

#[allow(non_snake_case, unused_variables)]
fn H5O_get_info(loc_id: hid_t, full: bool) -> Result<LocationInfo> {
    let mut info_buf = MaybeUninit::uninit();
    let info_ptr = info_buf.as_mut_ptr();
    #[cfg(feature = "1.12.0")]
    h5call!(H5Oget_info3(loc_id, info_ptr, info_fields(full)))?;
    #[cfg(all(feature = "1.10.3", not(feature = "1.12.0")))]
    h5call!(H5Oget_info2(loc_id, info_ptr, info_fields(full)))?;
    #[cfg(not(feature = "1.10.3"))]
    h5call!(H5Oget_info1(loc_id, info_ptr))?;
    let info = unsafe { info_buf.assume_init() };
    Ok(info.into())
}

#[allow(non_snake_case, unused_variables)]
fn H5O_get_info_by_name(loc_id: hid_t, name: *const c_char, full: bool) -> Result<LocationInfo> {
    let mut info_buf = MaybeUninit::uninit();
    let info_ptr = info_buf.as_mut_ptr();
    #[cfg(feature = "1.12.0")]
    h5call!(H5Oget_info_by_name3(loc_id, name, info_ptr, info_fields(full), H5P_DEFAULT))?;
    #[cfg(all(feature = "1.10.3", not(feature = "1.12.0")))]
    h5call!(H5Oget_info_by_name2(loc_id, name, info_ptr, info_fields(full), H5P_DEFAULT))?;
    #[cfg(not(feature = "1.10.3"))]
    h5call!(H5Oget_info_by_name1(loc_id, name, info_ptr, H5P_DEFAULT))?;
    let info = unsafe { info_buf.assume_init() };
    Ok(info.into())
}

#[allow(non_snake_case)]
fn H5O_open_by_token(loc_id: hid_t, token: LocationToken) -> Result<Location> {
    #[cfg(not(feature = "1.12.0"))]
    {
        Location::from_id(h5call!(H5Oopen_by_addr(loc_id, token.0))?)
    }
    #[cfg(feature = "1.12.0")]
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

    #[test]
    pub fn test_location_info() {
        let new_file = |path| {
            cfg_if::cfg_if! {
                if #[cfg(feature = "1.10.2")] {
                    File::with_options().with_fapl(|p| p.libver_v110()).create(path)
                } else {
                    File::create(path)
                }
            }
        };
        with_tmp_path(|path| {
            let file = new_file(path).unwrap();
            let token = {
                let group = file.create_group("group").unwrap();
                assert_eq!(file.loc_type_by_name("group").unwrap(), LocationType::Group);
                let info = group.loc_info().unwrap();
                assert_eq!(info.num_links, 1);
                assert_eq!(info.loc_type, LocationType::Group);
                cfg_if::cfg_if! {
                    if #[cfg(feature = "1.10.2")] {
                        assert!(info.btime > 0);
                    } else {
                        assert_eq!(info.btime, 0);
                    }
                }
                assert_eq!(info.btime == 0, info.mtime == 0);
                assert_eq!(info.btime == 0, info.ctime == 0);
                assert_eq!(info.btime == 0, info.atime == 0);
                assert_eq!(info.num_attrs, 0);
                info.token
            };
            let group = file.open_by_token(token).unwrap().as_group().unwrap();
            assert_eq!(group.name(), "/group");
            let token = {
                let var = group
                    .new_dataset_builder()
                    .obj_track_times(true)
                    .empty::<i8>()
                    .create("var")
                    .unwrap();
                var.new_attr::<i16>().create("attr1").unwrap();
                var.new_attr::<i32>().create("attr2").unwrap();
                group.link_hard("var", "hard1").unwrap();
                group.link_hard("var", "hard2").unwrap();
                group.link_hard("var", "hard3").unwrap();
                group.link_hard("var", "hard4").unwrap();
                group.link_hard("var", "hard5").unwrap();
                group.link_soft("var", "soft1").unwrap();
                group.link_soft("var", "soft2").unwrap();
                group.link_soft("var", "soft3").unwrap();
                assert_eq!(file.loc_type_by_name("/group/var").unwrap(), LocationType::Dataset);
                let info = var.loc_info().unwrap();
                assert_eq!(info.num_links, 6); // 1 + 5
                assert_eq!(info.loc_type, LocationType::Dataset);
                assert!(info.ctime > 0);
                cfg_if::cfg_if! {
                    if #[cfg(feature = "1.10.2")] {
                        assert!(info.btime > 0);
                    } else {
                        assert_eq!(info.btime, 0);
                    }
                }
                assert_eq!(info.btime == 0, info.mtime == 0);
                assert_eq!(info.btime == 0, info.atime == 0);
                assert_eq!(info.num_attrs, 2);
                info.token
            };
            let var = file.open_by_token(token).unwrap();
            // will open either the first or the last hard-linked object
            assert!(var.name().starts_with("/group/hard"));

            let info = file.loc_info_by_name("group").unwrap();
            let group = file.open_by_token(info.token).unwrap();
            assert_eq!(group.name(), "/group");
            let info = file.loc_info_by_name("/group/var").unwrap();
            let var = file.open_by_token(info.token).unwrap();
            assert!(var.name().starts_with("/group/hard"));

            assert!(file.loc_info_by_name("gibberish").is_err());
        })
    }
}
