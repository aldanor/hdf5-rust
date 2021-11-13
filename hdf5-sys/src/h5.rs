//! General purpose library functions
use std::mem;

pub use self::H5_index_t::*;
pub use self::H5_iter_order_t::*;

use crate::internal_prelude::*;

pub type herr_t = c_int;
pub type htri_t = c_int;
pub type hsize_t = c_ulonglong;
pub type hssize_t = c_longlong;
pub type haddr_t = uint64_t;

#[cfg(all(feature = "1.10.0", have_stdbool_h))]
pub type hbool_t = u8;
#[cfg(any(not(feature = "1.10.0"), not(have_stdbool_h)))]
pub type hbool_t = c_uint;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5_iter_order_t {
    H5_ITER_UNKNOWN = -1,
    H5_ITER_INC = 0,
    H5_ITER_DEC = 1,
    H5_ITER_NATIVE = 2,
    H5_ITER_N = 3,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5_index_t {
    H5_INDEX_UNKNOWN = -1,
    H5_INDEX_NAME = 0,
    H5_INDEX_CRT_ORDER = 1,
    H5_INDEX_N = 2,
}

pub const H5_ITER_ERROR: c_int = -1;
pub const H5_ITER_CONT: c_int = 0;
pub const H5_ITER_STOP: c_int = -1;

pub const HADDR_UNDEF: haddr_t = !0;
pub const HADDR_MAX: haddr_t = HADDR_UNDEF - 1;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5_ih_info_t {
    pub index_size: hsize_t,
    pub heap_size: hsize_t,
}

impl Default for H5_ih_info_t {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

extern "C" {
    pub fn H5open() -> herr_t;
    pub fn H5close() -> herr_t;
    pub fn H5dont_atexit() -> herr_t;
    pub fn H5garbage_collect() -> herr_t;
    pub fn H5set_free_list_limits(
        reg_global_lim: c_int, reg_list_lim: c_int, arr_global_lim: c_int, arr_list_lim: c_int,
        blk_global_lim: c_int, blk_list_lim: c_int,
    ) -> herr_t;
    pub fn H5get_libversion(
        majnum: *mut c_uint, minnum: *mut c_uint, relnum: *mut c_uint,
    ) -> herr_t;
    pub fn H5check_version(majnum: c_uint, minnum: c_uint, relnum: c_uint) -> herr_t;
}

#[cfg(feature = "1.8.13")]
extern "C" {
    pub fn H5free_memory(mem: *mut c_void) -> herr_t;
}

#[cfg(feature = "1.8.15")]
extern "C" {
    pub fn H5allocate_memory(size: size_t, clear: hbool_t) -> *mut c_void;
    pub fn H5resize_memory(mem: *mut c_void, size: size_t) -> *mut c_void;
}

#[cfg(feature = "1.8.16")]
extern "C" {
    pub fn H5is_library_threadsafe(is_ts: *mut hbool_t) -> herr_t;
}

#[cfg(any(all(feature = "1.10.7", not(feature = "1.12.0")), feature = "1.12.1"))]
#[repr(C)]
pub struct H5_alloc_stats_t {
    total_alloc_bytes: c_ulonglong,
    curr_alloc_bytes: size_t,
    peak_alloc_bytes: size_t,
    max_block_size: size_t,
    total_alloc_blocks_count: size_t,
    curr_alloc_blocks_count: size_t,
    peak_alloc_blocks_count: size_t,
}

#[cfg(any(all(feature = "1.10.7", not(feature = "1.12.0")), feature = "1.12.1"))]
extern "C" {
    pub fn H5get_alloc_stats(stats: *mut H5_alloc_stats_t) -> herr_t;
    pub fn H5get_free_list_sizes(
        reg_size: *mut size_t, arr_size: *mut size_t, blk_size: *mut size_t, fac_size: *mut size_t,
    ) -> herr_t;
}
