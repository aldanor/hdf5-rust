pub use self::H5_iter_order_t::*;
pub use self::H5_index_t::*;

use libc::{c_int, c_uint, c_void, c_ulonglong, c_longlong, uint64_t};

pub const H5_VERS_MAJOR:      c_uint = 1;
pub const H5_VERS_MINOR:      c_uint = 8;
pub const H5_VERS_RELEASE:    c_uint = 14;
pub const H5_VERS_SUBRELEASE: &'static str = "";
pub const H5_VERS_INFO:       &'static str = "HDF5; library version: 1.8.14";

pub type herr_t   = c_int;
pub type hbool_t  = c_uint;
pub type htri_t   = c_int;
pub type hsize_t  = c_ulonglong;
pub type hssize_t = c_longlong;
pub type haddr_t  = uint64_t;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5_iter_order_t {
    H5_ITER_UNKNOWN = -1,
    H5_ITER_INC     = 0,
    H5_ITER_DEC     = 1,
    H5_ITER_NATIVE  = 2,
    H5_ITER_N       = 3,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5_index_t {
    H5_INDEX_UNKNOWN   = -1,
    H5_INDEX_NAME      = 0,
    H5_INDEX_CRT_ORDER = 1,
    H5_INDEX_N         = 2,
}

pub const H5_ITER_ERROR: c_int = -1;
pub const H5_ITER_CONT:  c_int = 0;
pub const H5_ITER_STOP:  c_int = -1;

pub const HADDR_UNDEF: haddr_t = !0;
pub const HADDR_MAX:   haddr_t = HADDR_UNDEF - 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct H5_ih_info_t {
    pub index_size: hsize_t,
    pub heap_size: hsize_t,
}

impl ::std::default::Default for H5_ih_info_t {
    fn default() -> H5_ih_info_t { unsafe { ::std::mem::zeroed() } }
}

extern {
    pub fn H5open() -> herr_t;
    pub fn H5close() -> herr_t;
    pub fn H5dont_atexit() -> herr_t;
    pub fn H5garbage_collect() -> herr_t;
    pub fn H5set_free_list_limits(reg_global_lim: c_int, reg_list_lim: c_int, arr_global_lim: c_int,
                                  arr_list_lim: c_int, blk_global_lim: c_int, blk_list_lim: c_int)
                                  -> herr_t;
    pub fn H5get_libversion(majnum: *mut c_uint, minnum: *mut c_uint, relnum: *mut c_uint) ->
                            herr_t;
    pub fn H5check_version(majnum: c_uint, minnum: c_uint, relnum: c_uint) -> herr_t;
    pub fn H5free_memory(mem: *mut c_void) -> herr_t;
}
