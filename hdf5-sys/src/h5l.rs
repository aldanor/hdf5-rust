//! Creating and manipulating links within an HDF5 group
use std::mem;

pub use self::H5L_type_t::*;

#[cfg(feature = "1.12.0")]
use crate::h5o::H5O_token_t;
use crate::internal_prelude::*;

pub const H5L_MAX_LINK_NAME_LEN: uint32_t = !0;

pub const H5L_SAME_LOC: hid_t = 0;

pub const H5L_LINK_CLASS_T_VERS: c_uint = 0;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug)]
pub enum H5L_type_t {
    H5L_TYPE_ERROR = -1,
    H5L_TYPE_HARD = 0,
    H5L_TYPE_SOFT = 1,
    H5L_TYPE_EXTERNAL = 64,
    H5L_TYPE_MAX = 255,
}

pub const H5L_TYPE_BUILTIN_MAX: H5L_type_t = H5L_TYPE_SOFT;
pub const H5L_TYPE_UD_MIN: H5L_type_t = H5L_TYPE_EXTERNAL;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5L_info1_t {
    pub type_: H5L_type_t,
    pub corder_valid: hbool_t,
    pub corder: int64_t,
    pub cset: H5T_cset_t,
    pub u: H5L_info1_t__u,
}

impl Default for H5L_info1_t {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5L_info1_t__u {
    value: [u64; 1usize],
}

impl Default for H5L_info1_t__u {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

impl H5L_info1_t__u {
    pub unsafe fn address(&mut self) -> *mut haddr_t {
        &self.value as *const [u64; 1] as *mut haddr_t
    }
    pub unsafe fn val_size(&mut self) -> *mut size_t {
        &self.value as *const [u64; 1] as *mut size_t
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[cfg(feature = "1.12.0")]
pub struct H5L_info2_t {
    pub type_: H5L_type_t,
    pub corder_valid: hbool_t,
    pub corder: int64_t,
    pub cset: H5T_cset_t,
    pub u: H5L_info1_t__u,
}

#[cfg(feature = "1.12.0")]
impl Default for H5L_info2_t {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(feature = "1.12.0")]
pub union H5L_info2_t__u {
    token: H5O_token_t,
    val_size: size_t,
}

#[cfg(feature = "1.12.0")]
impl Default for H5L_info2_t__u {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

pub type H5L_create_func_t = Option<
    extern "C" fn(
        link_name: *const c_char,
        loc_group: hid_t,
        lnkdata: *const c_void,
        lnkdata_size: size_t,
        lcpl_id: hid_t,
    ) -> herr_t,
>;
pub type H5L_move_func_t = Option<
    extern "C" fn(
        new_name: *const c_char,
        new_loc: hid_t,
        lnkdata: *const c_void,
        lnkdata_size: size_t,
    ) -> herr_t,
>;
pub type H5L_copy_func_t = Option<
    extern "C" fn(
        new_name: *const c_char,
        new_loc: hid_t,
        lnkdata: *const c_void,
        lnkdata_size: size_t,
    ) -> herr_t,
>;
pub type H5L_traverse_func_t = Option<
    extern "C" fn(
        link_name: *const c_char,
        cur_group: hid_t,
        lnkdata: *const c_void,
        lnkdata_size: size_t,
        lapl_id: hid_t,
    ) -> hid_t,
>;
pub type H5L_delete_func_t = Option<
    extern "C" fn(
        link_name: *const c_char,
        file: hid_t,
        lnkdata: *const c_void,
        lnkdata_size: size_t,
    ) -> herr_t,
>;
pub type H5L_query_func_t = Option<
    extern "C" fn(
        link_name: *const c_char,
        lnkdata: *const c_void,
        lnkdata_size: size_t,
        buf: *mut c_void,
        buf_size: size_t,
    ) -> ssize_t,
>;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5L_class_t {
    pub version: c_int,
    pub id: H5L_type_t,
    pub comment: *const c_char,
    pub create_func: H5L_create_func_t,
    pub move_func: H5L_move_func_t,
    pub copy_func: H5L_copy_func_t,
    pub trav_func: H5L_traverse_func_t,
    pub del_func: H5L_delete_func_t,
    pub query_func: H5L_query_func_t,
}

impl Default for H5L_class_t {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

pub type H5L_iterate1_t = Option<
    extern "C" fn(
        group: hid_t,
        name: *const c_char,
        info: *const H5L_info1_t,
        op_data: *mut c_void,
    ) -> herr_t,
>;
#[cfg(feature = "1.12.0")]
pub type H5L_iterate2_t = Option<
    extern "C" fn(
        group: hid_t,
        name: *const c_char,
        info: *const H5L_info2_t,
        op_data: *mut c_void,
    ) -> herr_t,
>;
pub type H5L_elink_traverse_t = Option<
    extern "C" fn(
        parent_file_name: *const c_char,
        parent_group_name: *const c_char,
        child_file_name: *const c_char,
        child_object_name: *const c_char,
        acc_flags: *mut c_uint,
        fapl_id: hid_t,
        op_data: *mut c_void,
    ) -> herr_t,
>;

extern "C" {
    pub fn H5Lmove(
        src_loc: hid_t, src_name: *const c_char, dst_loc: hid_t, dst_name: *const c_char,
        lcpl_id: hid_t, lapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Lcopy(
        src_loc: hid_t, src_name: *const c_char, dst_loc: hid_t, dst_name: *const c_char,
        lcpl_id: hid_t, lapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Lcreate_hard(
        cur_loc: hid_t, cur_name: *const c_char, dst_loc: hid_t, dst_name: *const c_char,
        lcpl_id: hid_t, lapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Lcreate_soft(
        link_target: *const c_char, link_loc_id: hid_t, link_name: *const c_char, lcpl_id: hid_t,
        lapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Ldelete(loc_id: hid_t, name: *const c_char, lapl_id: hid_t) -> herr_t;
    pub fn H5Ldelete_by_idx(
        loc_id: hid_t, group_name: *const c_char, idx_type: H5_index_t, order: H5_iter_order_t,
        n: hsize_t, lapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Lget_val(
        loc_id: hid_t, name: *const c_char, buf: *mut c_void, size: size_t, lapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Lget_val_by_idx(
        loc_id: hid_t, group_name: *const c_char, idx_type: H5_index_t, order: H5_iter_order_t,
        n: hsize_t, buf: *mut c_void, size: size_t, lapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Lexists(loc_id: hid_t, name: *const c_char, lapl_id: hid_t) -> htri_t;
    #[cfg_attr(
        feature = "1.12.0",
        deprecated(note = "deprecated in HDF5 1.12.0, use H5Lget_info2()")
    )]
    #[cfg_attr(not(feature = "1.12.0"), link_name = "H5Lget_info")]
    pub fn H5Lget_info1(
        loc_id: hid_t, name: *const c_char, linfo: *mut H5L_info1_t, lapl_id: hid_t,
    ) -> herr_t;
    #[cfg(feature = "1.12.0")]
    pub fn H5Lget_info2(
        loc_id: hid_t, name: *const c_char, linfo: *mut H5L_info2_t, lapl_id: hid_t,
    ) -> herr_t;
    #[cfg_attr(
        hdf5_1_12_0,
        deprecated(note = "deprecated in HDF5 1.12.0, use H5Lget_info_by_idx2()")
    )]
    #[cfg_attr(not(feature = "1.12.0"), link_name = "H5Lget_info_by_idx")]
    pub fn H5Lget_info_by_idx(
        loc_id: hid_t, group_name: *const c_char, idx_type: H5_index_t, order: H5_iter_order_t,
        n: hsize_t, linfo: *mut H5L_info1_t, lapl_id: hid_t,
    ) -> herr_t;
    #[cfg(feature = "1.12.0")]
    pub fn H5Lget_info_by_idx2(
        loc_id: hid_t, group_name: *const c_char, idx_type: H5_index_t, order: H5_iter_order_t,
        n: hsize_t, linfo: *mut H5L_info2_t, lapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Lget_name_by_idx(
        loc_id: hid_t, group_name: *const c_char, idx_type: H5_index_t, order: H5_iter_order_t,
        n: hsize_t, name: *mut c_char, size: size_t, lapl_id: hid_t,
    ) -> ssize_t;
    #[cfg_attr(
        feature = "1.12.0",
        deprecated(note = "deprecated in HDF5 1.12.0, use H5Literate2()")
    )]
    #[cfg_attr(not(feature = "1.12.0"), link_name = "H5Literate")]
    pub fn H5Literate1(
        grp_id: hid_t, idx_type: H5_index_t, order: H5_iter_order_t, idx: *mut hsize_t,
        op: H5L_iterate1_t, op_data: *mut c_void,
    ) -> herr_t;
    #[cfg(feature = "1.12.0")]
    pub fn H5Literate2(
        grp_id: hid_t, idx_type: H5_index_t, order: H5_iter_order_t, idx: *mut hsize_t,
        op: H5L_iterate2_t, op_data: *mut c_void,
    ) -> herr_t;
    #[cfg_attr(
        hdf5_1_12_0,
        deprecated(note = "deprecated in HDF5 1.12.0, use H5Literate_by_name2()")
    )]
    #[cfg_attr(not(feature = "1.12.0"), link_name = "H5Literate_by_name")]
    pub fn H5Literate_by_name1(
        loc_id: hid_t, group_name: *const c_char, idx_type: H5_index_t, order: H5_iter_order_t,
        idx: *mut hsize_t, op: H5L_iterate1_t, op_data: *mut c_void, lapl_id: hid_t,
    ) -> herr_t;
    #[cfg(feature = "1.12.0")]
    pub fn H5Literate_by_name2(
        loc_id: hid_t, group_name: *const c_char, idx_type: H5_index_t, order: H5_iter_order_t,
        idx: *mut hsize_t, op: H5L_iterate2_t, op_data: *mut c_void, lapl_id: hid_t,
    ) -> herr_t;
    #[cfg_attr(feature = "1.12.0", deprecated(note = "deprecated in HDF5 1.12.0, use H5Lvisit2()"))]
    #[cfg_attr(not(feature = "1.12.0"), link_name = "H5Lvisit")]
    pub fn H5Lvisit1(
        grp_id: hid_t, idx_type: H5_index_t, order: H5_iter_order_t, op: H5L_iterate1_t,
        op_data: *mut c_void,
    ) -> herr_t;
    #[cfg(feature = "1.12.0")]
    pub fn H5Lvisit2(
        grp_id: hid_t, idx_type: H5_index_t, order: H5_iter_order_t, op: H5L_iterate2_t,
        op_data: *mut c_void,
    ) -> herr_t;
    #[cfg_attr(
        hdf5_1_12_0,
        deprecated(note = "deprecated in HDF5 1.12.0, use H5Lvisit_by_name2()")
    )]
    #[cfg_attr(not(feature = "1.12.0"), link_name = "H5Lvisit_by_name")]
    pub fn H5Lvisit_by_name1(
        loc_id: hid_t, group_name: *const c_char, idx_type: H5_index_t, order: H5_iter_order_t,
        op: H5L_iterate1_t, op_data: *mut c_void, lapl_id: hid_t,
    ) -> herr_t;
    #[cfg(feature = "1.12.0")]
    pub fn H5Lvisit_by_name2(
        loc_id: hid_t, group_name: *const c_char, idx_type: H5_index_t, order: H5_iter_order_t,
        op: H5L_iterate2_t, op_data: *mut c_void, lapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Lcreate_ud(
        link_loc_id: hid_t, link_name: *const c_char, link_type: H5L_type_t, udata: *const c_void,
        udata_size: size_t, lcpl_id: hid_t, lapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Lregister(cls: *const H5L_class_t) -> herr_t;
    pub fn H5Lunregister(id: H5L_type_t) -> herr_t;
    pub fn H5Lis_registered(id: H5L_type_t) -> htri_t;
    pub fn H5Lunpack_elink_val(
        ext_linkval: *const c_void, link_size: size_t, flags: *mut c_uint,
        filename: *mut *const c_char, obj_path: *mut *const c_char,
    ) -> herr_t;
    pub fn H5Lcreate_external(
        file_name: *const c_char, obj_name: *const c_char, link_loc_id: hid_t,
        link_name: *const c_char, lcpl_id: hid_t, lapl_id: hid_t,
    ) -> herr_t;
}

#[cfg(not(feature = "1.12.0"))]
pub use self::{
    H5L_info1_t as H5L_info_t, H5L_iterate1_t as H5L_iterate_t, H5Literate1 as H5Literate,
};
#[cfg(feature = "1.12.0")]
pub use self::{
    H5L_info2_t as H5L_info_t, H5L_iterate2_t as H5L_iterate_t, H5Literate2 as H5Literate,
};

#[cfg(feature = "1.14.0")]
extern "C" {
    pub fn H5Lcreate_hard_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, cur_loc_id: hid_t,
        cur_name: *const c_char, new_loc_id: hid_t, new_name: *const c_char, lcpl_id: hid_t,
        lapl_id: hid_t, es_id: hid_t,
    ) -> herr_t;
    pub fn H5Lcreate_soft_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint,
        link_target: *const c_char, link_loc_id: hid_t, link_name: *const c_char, lcpl_id: hid_t,
        lapl_id: hid_t, es_id: hid_t,
    ) -> herr_t;
    pub fn H5Ldelete_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, loc_id: hid_t,
        name: *const c_char, lapl_id: hid_t, es_id: hid_t,
    ) -> herr_t;
    pub fn H5Ldelete_by_idx_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, loc_id: hid_t,
        group_name: *const c_char, idx_type: H5_index_t, order: H5_iter_order_t, n: c_ulong,
        lapl_id: hid_t, es_id: hid_t,
    ) -> herr_t;
    pub fn H5Lexists_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, loc_id: hid_t,
        name: *const c_char, exists: *mut hbool_t, lapl_id: hid_t, es_id: hid_t,
    ) -> herr_t;
    pub fn H5Literate_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, group_id: hid_t,
        idx_type: H5_index_t, order: H5_iter_order_t, idx_p: *mut c_ulong, op: H5L_iterate2_t,
        op_data: *mut c_void, es_id: hid_t,
    ) -> herr_t;

}
