use libc::{c_int, c_char, c_long, c_double, size_t};

use h5::hbool_t;
use h5c::{H5C_cache_incr_mode, H5C_cache_decr_mode, H5C_cache_flash_incr_mode};

pub const H5AC__CURR_CACHE_CONFIG_VERSION: c_int = 1;
pub const H5AC__MAX_TRACE_FILE_NAME_LEN:   c_int = 1024;

pub const H5AC_METADATA_WRITE_STRATEGY__PROCESS_0_ONLY: c_int = 0;
pub const H5AC_METADATA_WRITE_STRATEGY__DISTRIBUTED:    c_int = 1;

#[repr(C)]
#[derive(Copy)]
pub struct H5AC_cache_config_t {
    pub version: c_int,
    pub rpt_fcn_enabled: hbool_t,
    pub open_trace_file: hbool_t,
    pub close_trace_file: hbool_t,
    pub trace_file_name: [c_char; 1025usize],
    pub evictions_enabled: hbool_t,
    pub set_initial_size: hbool_t,
    pub initial_size: size_t,
    pub min_clean_fraction: c_double,
    pub max_size: size_t,
    pub min_size: size_t,
    pub epoch_length: c_long,
    pub incr_mode: H5C_cache_incr_mode,
    pub lower_hr_threshold: c_double,
    pub increment: c_double,
    pub apply_max_increment: hbool_t,
    pub max_increment: size_t,
    pub flash_incr_mode: H5C_cache_flash_incr_mode,
    pub flash_multiple: c_double,
    pub flash_threshold: c_double,
    pub decr_mode: H5C_cache_decr_mode,
    pub upper_hr_threshold: c_double,
    pub decrement: c_double,
    pub apply_max_decrement: hbool_t,
    pub max_decrement: size_t,
    pub epochs_before_eviction: c_int,
    pub apply_empty_reserve: hbool_t,
    pub empty_reserve: c_double,
    pub dirty_bytes_threshold: c_int,
    pub metadata_write_strategy: c_int,
}

impl ::std::default::Default for H5AC_cache_config_t {
    fn default() -> H5AC_cache_config_t { unsafe { ::std::mem::zeroed() } }
}

impl Clone for H5AC_cache_config_t {
    fn clone(&self) -> H5AC_cache_config_t { *self }
}
