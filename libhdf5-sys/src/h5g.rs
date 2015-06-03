pub use self::H5G_storage_type_t::*;

use libc::{c_uint, c_char, int64_t};

use h5::{herr_t, hbool_t, hsize_t, H5_index_t, H5_iter_order_t};
use h5i::hid_t;
use h5l::{H5L_SAME_LOC, H5L_TYPE_ERROR, H5L_TYPE_HARD, H5L_TYPE_SOFT, H5L_type_t};

pub const H5G_SAME_LOC: hid_t = H5L_SAME_LOC;

pub const H5G_LINK_ERROR: H5L_type_t = H5L_TYPE_ERROR;
pub const H5G_LINK_HARD:  H5L_type_t = H5L_TYPE_HARD;
pub const H5G_LINK_SOFT:  H5L_type_t = H5L_TYPE_SOFT;

pub type H5G_link_t = H5L_type_t;

pub const H5G_NTYPES:     c_uint = 256;
pub const H5G_NLIBTYPES:  c_uint = 8;
pub const H5G_NUSERTYPES: c_uint = H5G_NTYPES - H5G_NLIBTYPES;

pub fn H5G_USERTYPE(X: c_uint) -> c_uint { 8 + X }

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5G_storage_type_t {
    H5G_STORAGE_TYPE_UNKNOWN      = -1,
    H5G_STORAGE_TYPE_SYMBOL_TABLE = 0,
    H5G_STORAGE_TYPE_COMPACT      = 1,
    H5G_STORAGE_TYPE_DENSE        = 2,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct H5G_info_t {
    pub storage_type: H5G_storage_type_t,
    pub nlinks: hsize_t,
    pub max_corder: int64_t,
    pub mounted: hbool_t,
}

impl ::std::default::Default for H5G_info_t {
    fn default() -> H5G_info_t { unsafe { ::std::mem::zeroed() } }
}

extern {
    pub fn H5Gcreate2(loc_id: hid_t, name: *const c_char, lcpl_id: hid_t, gcpl_id: hid_t, gapl_id:
                      hid_t) -> hid_t;
    pub fn H5Gcreate_anon(loc_id: hid_t, gcpl_id: hid_t, gapl_id: hid_t) -> hid_t;
    pub fn H5Gopen2(loc_id: hid_t, name: *const c_char, gapl_id: hid_t) -> hid_t;
    pub fn H5Gget_create_plist(group_id: hid_t) -> hid_t;
    pub fn H5Gget_info(loc_id: hid_t, ginfo: *mut H5G_info_t) -> herr_t;
    pub fn H5Gget_info_by_name(loc_id: hid_t, name: *const c_char, ginfo: *mut H5G_info_t, lapl_id:
                               hid_t) -> herr_t;
    pub fn H5Gget_info_by_idx(loc_id: hid_t, group_name: *const c_char, idx_type: H5_index_t, order:
                              H5_iter_order_t, n: hsize_t, ginfo: *mut H5G_info_t, lapl_id: hid_t)
                              -> herr_t;
    pub fn H5Gclose(group_id: hid_t) -> herr_t;
}
