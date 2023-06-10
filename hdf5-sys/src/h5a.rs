//! Creating and manipulating HDF5 attributes
use std::mem;

use crate::internal_prelude::*;

use crate::h5o::H5O_msg_crt_idx_t;
pub use {
    H5A_operator2_t as H5A_operator_t, H5A_operator2_t as H5A_operator_r, H5Acreate2 as H5Acreate,
    H5Aiterate2 as H5Aiterate,
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct H5A_info_t {
    pub corder_valid: hbool_t,
    pub corder: H5O_msg_crt_idx_t,
    pub cset: H5T_cset_t,
    pub data_size: hsize_t,
}

impl Default for H5A_info_t {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

#[deprecated(note = "deprecated in HDF5 1.8.0, use H5A_operator2_t")]
pub type H5A_operator1_t = Option<
    extern "C" fn(
        location_id: hid_t,
        attr_name: *const c_char,
        operator_data: *mut c_void,
    ) -> herr_t,
>;

pub type H5A_operator2_t = Option<
    extern "C" fn(
        location_id: hid_t,
        attr_name: *const c_char,
        ainfo: *const H5A_info_t,
        op_data: *mut c_void,
    ) -> herr_t,
>;

extern "C" {
    pub fn H5Acreate2(
        loc_id: hid_t, attr_name: *const c_char, type_id: hid_t, space_id: hid_t, acpl_id: hid_t,
        aapl_id: hid_t,
    ) -> hid_t;
    pub fn H5Acreate_by_name(
        loc_id: hid_t, obj_name: *const c_char, attr_name: *const c_char, type_id: hid_t,
        space_id: hid_t, acpl_id: hid_t, aapl_id: hid_t, lapl_id: hid_t,
    ) -> hid_t;
    pub fn H5Aopen(obj_id: hid_t, attr_name: *const c_char, aapl_id: hid_t) -> hid_t;
    pub fn H5Aopen_by_name(
        loc_id: hid_t, obj_name: *const c_char, attr_name: *const c_char, aapl_id: hid_t,
        lapl_id: hid_t,
    ) -> hid_t;
    pub fn H5Aopen_by_idx(
        loc_id: hid_t, obj_name: *const c_char, idx_type: H5_index_t, order: H5_iter_order_t,
        n: hsize_t, aapl_id: hid_t, lapl_id: hid_t,
    ) -> hid_t;
    pub fn H5Awrite(attr_id: hid_t, type_id: hid_t, buf: *const c_void) -> herr_t;
    pub fn H5Aread(attr_id: hid_t, type_id: hid_t, buf: *mut c_void) -> herr_t;
    pub fn H5Aclose(attr_id: hid_t) -> herr_t;
    pub fn H5Aget_space(attr_id: hid_t) -> hid_t;
    pub fn H5Aget_type(attr_id: hid_t) -> hid_t;
    pub fn H5Aget_create_plist(attr_id: hid_t) -> hid_t;
    pub fn H5Aget_name(attr_id: hid_t, buf_size: size_t, buf: *mut c_char) -> ssize_t;
    pub fn H5Aget_name_by_idx(
        loc_id: hid_t, obj_name: *const c_char, idx_type: H5_index_t, order: H5_iter_order_t,
        n: hsize_t, name: *mut c_char, size: size_t, lapl_id: hid_t,
    ) -> ssize_t;
    pub fn H5Aget_storage_size(attr_id: hid_t) -> hsize_t;
    pub fn H5Aget_info(attr_id: hid_t, ainfo: *mut H5A_info_t) -> herr_t;
    pub fn H5Aget_info_by_name(
        loc_id: hid_t, obj_name: *const c_char, attr_name: *const c_char, ainfo: *mut H5A_info_t,
        lapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Aget_info_by_idx(
        loc_id: hid_t, obj_name: *const c_char, idx_type: H5_index_t, order: H5_iter_order_t,
        n: hsize_t, ainfo: *mut H5A_info_t, lapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Arename(loc_id: hid_t, old_name: *const c_char, new_name: *const c_char) -> herr_t;
    pub fn H5Arename_by_name(
        loc_id: hid_t, obj_name: *const c_char, old_attr_name: *const c_char,
        new_attr_name: *const c_char, lapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Aiterate2(
        loc_id: hid_t, idx_type: H5_index_t, order: H5_iter_order_t, idx: *mut hsize_t,
        op: H5A_operator2_t, op_data: *mut c_void,
    ) -> herr_t;
    pub fn H5Aiterate_by_name(
        loc_id: hid_t, obj_name: *const c_char, idx_type: H5_index_t, order: H5_iter_order_t,
        idx: *mut hsize_t, op: H5A_operator2_t, op_data: *mut c_void, lapd_id: hid_t,
    ) -> herr_t;
    pub fn H5Adelete(loc_id: hid_t, name: *const c_char) -> herr_t;
    pub fn H5Adelete_by_name(
        loc_id: hid_t, obj_name: *const c_char, attr_name: *const c_char, lapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Adelete_by_idx(
        loc_id: hid_t, obj_name: *const c_char, idx_type: H5_index_t, order: H5_iter_order_t,
        n: hsize_t, lapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Aexists(obj_id: hid_t, attr_name: *const c_char) -> htri_t;
    pub fn H5Aexists_by_name(
        obj_id: hid_t, obj_name: *const c_char, attr_name: *const c_char, lapl_id: hid_t,
    ) -> htri_t;

    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Aget_info")]
    pub fn H5Aget_num_attrs(loc_id: hid_t) -> c_int;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Aopen_by_idx")]
    pub fn H5Aopen_idx(loc_id: hid_t, idx: c_uint) -> hid_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Aopen_by_name")]
    pub fn H5Aopen_name(loc_id: hid_t, name: *const c_char) -> hid_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Acreate2")]
    pub fn H5Acreate1(
        loc_id: hid_t, name: *const c_char, type_id: hid_t, space_id: hid_t, acpl_id: hid_t,
    ) -> hid_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Aiterate2")]
    pub fn H5Aiterate1(
        loc_id: hid_t, attr_num: *mut c_uint, op: H5A_operator1_t, op_data: *mut c_void,
    ) -> herr_t;
}

#[cfg(feature = "1.14.0")]
extern "C" {
    pub fn H5Aclose_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, attr_id: hid_t,
        es_id: hid_t,
    ) -> herr_t;
    pub fn H5Acreate_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, loc_id: hid_t,
        attr_name: *const c_char, type_id: hid_t, space_id: hid_t, acpl_id: hid_t, aapl_id: hid_t,
        es_id: hid_t,
    ) -> hid_t;
    pub fn H5Acreate_by_name_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, loc_id: hid_t,
        obj_name: *const c_char, attr_name: *const c_char, type_id: hid_t, space_id: hid_t,
        acpl_id: hid_t, aapl_id: hid_t, lapl_id: hid_t, es_id: hid_t,
    ) -> hid_t;
    pub fn H5Aexists_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, obj_id: hid_t,
        attr_name: *const c_char, exists: *mut hbool_t, es_id: hid_t,
    ) -> herr_t;
    pub fn H5Aexists_by_name_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, loc_id: hid_t,
        obj_name: *const c_char, attr_name: *const c_char, exists: *mut hbool_t, lapl_id: hid_t,
        es_id: hid_t,
    ) -> herr_t;
    pub fn H5Aopen_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, obj_id: hid_t,
        attr_name: *const c_char, aapl_id: hid_t, es_id: hid_t,
    ) -> hid_t;
    pub fn H5Aopen_by_idx_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, loc_id: hid_t,
        obj_name: *const c_char, idx_type: H5_index_t, order: H5_iter_order_t, n: c_ulong,
        aapl_id: hid_t, lapl_id: hid_t, es_id: hid_t,
    ) -> hid_t;
    pub fn H5Aopen_by_name_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, loc_id: hid_t,
        obj_name: *const c_char, attr_name: *const c_char, aapl_id: hid_t, lapl_id: hid_t,
        es_id: hid_t,
    ) -> hid_t;
    pub fn H5Aread_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, attr_id: hid_t,
        dtype_id: hid_t, buf: *mut c_void, es_id: hid_t,
    ) -> herr_t;
    pub fn H5Arename_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, loc_id: hid_t,
        old_name: *const c_char, new_name: *const c_char, es_id: hid_t,
    ) -> herr_t;
    pub fn H5Arename_by_name_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, loc_id: hid_t,
        obj_name: *const c_char, old_attr_name: *const c_char, new_attr_name: *const c_char,
        lapl_id: hid_t, es_id: hid_t,
    ) -> herr_t;
    pub fn H5Awrite_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, attr_id: hid_t,
        type_id: hid_t, buf: *const c_void, es_id: hid_t,
    ) -> herr_t;
}
