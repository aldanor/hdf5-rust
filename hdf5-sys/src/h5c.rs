//! Cache functionality
pub use self::H5C_cache_decr_mode::*;
pub use self::H5C_cache_flash_incr_mode::*;
pub use self::H5C_cache_incr_mode::*;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug)]
pub enum H5C_cache_incr_mode {
    H5C_incr__off = 0,
    H5C_incr__threshold = 1,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug)]
pub enum H5C_cache_flash_incr_mode {
    H5C_flash_incr__off = 0,
    H5C_flash_incr__add_space = 1,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug)]
pub enum H5C_cache_decr_mode {
    H5C_decr__off = 0,
    H5C_decr__threshold = 1,
    H5C_decr__age_out = 2,
    H5C_decr__age_out_with_threshold = 3,
}
