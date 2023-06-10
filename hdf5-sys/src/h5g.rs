//! Creating and manipulating groups of objects inside an HDF5 file
use std::mem;

pub use self::H5G_storage_type_t::*;
pub use {H5Gcreate2 as H5Gcreate, H5Gopen2 as H5Gopen};

use crate::internal_prelude::*;

use crate::h5l::{H5L_type_t, H5L_SAME_LOC, H5L_TYPE_ERROR, H5L_TYPE_HARD, H5L_TYPE_SOFT};
use crate::h5o::H5O_stat_t;

pub const H5G_SAME_LOC: hid_t = H5L_SAME_LOC;

pub const H5G_LINK_ERROR: H5L_type_t = H5L_TYPE_ERROR;
pub const H5G_LINK_HARD: H5L_type_t = H5L_TYPE_HARD;
pub const H5G_LINK_SOFT: H5L_type_t = H5L_TYPE_SOFT;

pub type H5G_link_t = H5L_type_t;

pub const H5G_NTYPES: c_uint = 256;
pub const H5G_NLIBTYPES: c_uint = 8;
pub const H5G_NUSERTYPES: c_uint = H5G_NTYPES - H5G_NLIBTYPES;

pub const fn H5G_USERTYPE(X: c_uint) -> c_uint {
    8 + X
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug)]
pub enum H5G_storage_type_t {
    H5G_STORAGE_TYPE_UNKNOWN = -1,
    H5G_STORAGE_TYPE_SYMBOL_TABLE = 0,
    H5G_STORAGE_TYPE_COMPACT = 1,
    H5G_STORAGE_TYPE_DENSE = 2,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5G_info_t {
    pub storage_type: H5G_storage_type_t,
    pub nlinks: hsize_t,
    pub max_corder: int64_t,
    pub mounted: hbool_t,
}

impl Default for H5G_info_t {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

extern "C" {
    pub fn H5Gcreate2(
        loc_id: hid_t, name: *const c_char, lcpl_id: hid_t, gcpl_id: hid_t, gapl_id: hid_t,
    ) -> hid_t;
    pub fn H5Gcreate_anon(loc_id: hid_t, gcpl_id: hid_t, gapl_id: hid_t) -> hid_t;
    pub fn H5Gopen2(loc_id: hid_t, name: *const c_char, gapl_id: hid_t) -> hid_t;
    pub fn H5Gget_create_plist(group_id: hid_t) -> hid_t;
    pub fn H5Gget_info(loc_id: hid_t, ginfo: *mut H5G_info_t) -> herr_t;
    pub fn H5Gget_info_by_name(
        loc_id: hid_t, name: *const c_char, ginfo: *mut H5G_info_t, lapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Gget_info_by_idx(
        loc_id: hid_t, group_name: *const c_char, idx_type: H5_index_t, order: H5_iter_order_t,
        n: hsize_t, ginfo: *mut H5G_info_t, lapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Gclose(group_id: hid_t) -> herr_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Gcreate2")]
    pub fn H5Gcreate1(loc_id: hid_t, name: *const c_char, size_hint: size_t) -> hid_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Oget_comment")]
    pub fn H5Gget_comment(
        loc_id: hid_t, name: *const c_char, bufsize: size_t, buf: *mut c_char,
    ) -> c_int;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Lget_val")]
    pub fn H5Gget_linkval(loc_id: hid_t, name: *const c_char, comment: *const c_char) -> herr_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Gget_info")]
    pub fn H5Gget_num_objs(loc_id: hid_t, num_objs: *mut hsize_t) -> herr_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Oget_info or H5Lget_info")]
    pub fn H5Gget_objinfo(
        loc_id: hid_t, name: *const c_char, follow_link: hbool_t, statubuf: *mut H5G_stat_t,
    ) -> herr_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Lget_name_by_idx")]
    pub fn H5Gget_objname_by_idx(
        loc_id: hid_t, idx: hsize_t, name: *mut c_char, size: size_t,
    ) -> ssize_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Oget_info")]
    pub fn H5Gget_objtype_by_idx(loc_id: hid_t, idx: hsize_t) -> H5G_obj_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Lcreate_hard or H5Lcreate_soft")]
    pub fn H5Glink(
        cur_loc_id: hid_t, type_: H5G_link_t, cur_name: *const c_char, new_name: *const c_char,
    ) -> herr_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Lcreate_hard or H5Lcreate_soft")]
    pub fn H5Glink2(
        cur_loc_id: hid_t, cur_name: *const c_char, type_: H5G_link_t, new_loc_id: hid_t,
        new_name: *const c_char,
    ) -> herr_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Lmove")]
    pub fn H5Gmove(src_loc_id: hid_t, src_name: *const c_char, dst_name: *const c_char) -> herr_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Lmove")]
    pub fn H5Gmove2(
        src_loc_id: hid_t, src_name: *const c_char, dst_loc_id: hid_t, dst_name: *const c_char,
    ) -> herr_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Gopen2")]
    pub fn H5Gopen1(loc_id: hid_t, name: *const c_char) -> hid_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Ldelete")]
    pub fn H5Gunlink(loc_id: hid_t, name: *const c_char) -> herr_t;
}

#[cfg(feature = "1.10.0")]
extern "C" {
    pub fn H5Gflush(group_id: hid_t) -> herr_t;
    pub fn H5Grefresh(group_id: hid_t) -> herr_t;
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum H5G_obj_t {
    H5G_UNKNOwN = -1,
    H5G_GROUP,
    H5G_DATASET,
    H5G_TYPE,
    H5G_LINK,
    H5G_UDLINK,
    H5G_RESERVED_5,
    H5G_RESERVED_6,
    H5G_RESERVED_7,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5G_stat_t {
    fileno: [c_ulong; 2],
    objno: [c_ulong; 2],
    nlink: c_uint,
    type_: H5G_obj_t,
    mtime: time_t,
    linklen: size_t,
    ohdr: H5O_stat_t,
}

#[cfg(feature = "1.14.0")]
extern "C" {
    pub fn H5Gclose_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, group_id: hid_t,
        es_id: hid_t,
    ) -> herr_t;
    pub fn H5Gcreate_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, loc_id: hid_t,
        name: *const c_char, lcpl_id: hid_t, gcpl_id: hid_t, gapl_id: hid_t, es_id: hid_t,
    ) -> hid_t;
    pub fn H5Gget_info_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, loc_id: hid_t,
        ginfo: *mut H5G_info_t, es_id: hid_t,
    ) -> herr_t;
    pub fn H5Gget_info_by_idx_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, loc_id: hid_t,
        group_name: *const c_char, idx_type: H5_index_t, order: H5_iter_order_t, n: c_ulong,
        ginfo: *mut H5G_info_t, lapl_id: hid_t, es_id: hid_t,
    ) -> herr_t;
    pub fn H5Gget_info_by_name_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, loc_id: hid_t,
        name: *const c_char, ginfo: *mut H5G_info_t, lapl_id: hid_t, es_id: hid_t,
    ) -> herr_t;
    pub fn H5Gopen_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, loc_id: hid_t,
        name: *const c_char, gapl_id: hid_t, es_id: hid_t,
    ) -> hid_t;
}
