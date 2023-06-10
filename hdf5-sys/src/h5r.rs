//! Creating and manipulating references to specific objects and data regions in an HDF5 file
pub use self::H5R_type_t::*;
#[cfg(not(feature = "1.10.0"))]
pub use H5Rdereference1 as H5Rdereference;
#[cfg(feature = "1.10.0")]
pub use H5Rdereference2 as H5Rdereference;

use crate::internal_prelude::*;

use crate::h5g::H5G_obj_t;
use crate::h5o::H5O_type_t;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug)]
#[cfg(not(feature = "1.12.0"))]
pub enum H5R_type_t {
    H5R_BADTYPE = -1,
    H5R_OBJECT = 0,
    H5R_DATASET_REGION = 1,
    H5R_MAXTYPE = 2,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug)]
#[cfg(feature = "1.12.0")]
pub enum H5R_type_t {
    H5R_BADTYPE = -1,
    H5R_OBJECT1 = 0,
    H5R_DATASET_REGION1 = 1,
    H5R_OBJECT2 = 2,
    H5R_DATASET_REGION2 = 3,
    H5R_ATTR = 4,
    H5R_MAXTYPE = 5,
}

pub type hobj_ref_t = haddr_t;
pub type hdset_reg_ref_t = [c_uchar; 12usize];

extern "C" {
    pub fn H5Rcreate(
        ref_: *mut c_void, loc_id: hid_t, name: *const c_char, ref_type: H5R_type_t,
        space_id: hid_t,
    ) -> herr_t;
    pub fn H5Rget_region(dataset: hid_t, ref_type: H5R_type_t, ref_: *const c_void) -> hid_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Rget_obj_type2")]
    pub fn H5Rget_obj_type1(id: hid_t, ref_type: H5R_type_t, ref_: *const c_void) -> H5G_obj_t;
    pub fn H5Rget_obj_type2(
        id: hid_t, ref_type: H5R_type_t, ref_: *const c_void, obj_type: *mut H5O_type_t,
    ) -> herr_t;
    pub fn H5Rget_name(
        loc_id: hid_t, ref_type: H5R_type_t, ref_: *const c_void, name: *mut c_char, size: size_t,
    ) -> ssize_t;
}

extern "C" {
    #[cfg_attr(
        feature = "1.10.0",
        deprecated(note = "deprecated in HDF5 1.10.0, use H5Rdereference2")
    )]
    #[cfg_attr(not(feature = "1.10.0"), link_name = "H5Rdereference")]
    pub fn H5Rdereference1(obj_id: hid_t, ref_type: H5R_type_t, ref_: *const c_void) -> hid_t;
    #[cfg(feature = "1.10.0")]
    pub fn H5Rdereference2(
        obj_id: hid_t, oapl_id: hid_t, ref_type: H5R_type_t, ref_: *const c_void,
    ) -> hid_t;
}

#[cfg(feature = "1.12.0")]
pub const H5R_REF_BUF_SIZE: usize = 64;

#[cfg(feature = "1.12.0")]
#[repr(C)]
#[derive(Copy, Clone)]
pub union H5R_ref_t_u {
    __data: [u8; H5R_REF_BUF_SIZE],
    align: i64,
}

#[cfg(feature = "1.12.0")]
impl Default for H5R_ref_t_u {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[cfg(feature = "1.12.0")]
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct H5R_ref_t {
    u: H5R_ref_t_u,
}

#[cfg(feature = "1.12.0")]
extern "C" {
    pub fn H5Rcopy(src_ref_ptr: *const H5R_ref_t, dst_ref_ptr: *mut H5R_ref_t) -> herr_t;
    pub fn H5Rcreate_attr(
        loc_id: hid_t, name: *const c_char, attr_name: *const c_char, oapl_id: hid_t,
        ref_ptr: *mut H5R_ref_t,
    ) -> herr_t;
    pub fn H5Rcreate_object(
        loc_id: hid_t, name: *const c_char, oapl_id: hid_t, ref_ptr: *mut H5R_ref_t,
    ) -> herr_t;
    pub fn H5Rcreate_region(
        loc_id: hid_t, name: *const c_char, space_id: hid_t, oapl_id: hid_t,
        ref_ptr: *mut H5R_ref_t,
    ) -> herr_t;
    pub fn H5Rdestroy(ref_ptr: *mut H5R_ref_t) -> herr_t;
    pub fn H5Requal(ref1_ptr: *const H5R_ref_t, ref2_ptr: *const H5R_ref_t) -> htri_t;
    pub fn H5Rget_attr_name(ref_ptr: *const H5R_ref_t, name: *mut c_char, size: size_t) -> ssize_t;
    pub fn H5Rget_file_name(ref_ptr: *const H5R_ref_t, name: *mut c_char, size: size_t) -> ssize_t;
    pub fn H5Rget_obj_name(
        ref_ptr: *const H5R_ref_t, rapl_id: hid_t, name: *mut c_char, size: size_t,
    ) -> ssize_t;
    pub fn H5Rget_obj_type3(
        ref_ptr: *const H5R_ref_t, rapl_id: hid_t, obj_type: *mut H5O_type_t,
    ) -> herr_t;
    pub fn H5Rget_type(ref_ptr: *const H5R_ref_t) -> H5R_type_t;
    pub fn H5Ropen_attr(ref_ptr: *const H5R_ref_t, rapl_id: hid_t, aapl_id: hid_t) -> hid_t;
    pub fn H5Ropen_object(ref_ptr: *const H5R_ref_t, rapl_id: hid_t, oapl_id: hid_t) -> hid_t;
    pub fn H5Ropen_region(ref_ptr: *const H5R_ref_t, rapl_id: hid_t, oapl_id: hid_t) -> hid_t;
}

#[cfg(feature = "1.14.0")]
extern "C" {
    pub fn H5Ropen_attr_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint,
        ref_ptr: *mut H5R_ref_t, rapl_id: hid_t, aapl_id: hid_t, es_id: hid_t,
    ) -> hid_t;
    pub fn H5Ropen_object_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint,
        ref_ptr: *mut H5R_ref_t, rapl_id: hid_t, oapl_id: hid_t, es_id: hid_t,
    ) -> hid_t;
    pub fn H5Ropen_region_async(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint,
        ref_ptr: *mut H5R_ref_t, rapl_id: hid_t, oapl_id: hid_t, es_id: hid_t,
    ) -> hid_t;
}
