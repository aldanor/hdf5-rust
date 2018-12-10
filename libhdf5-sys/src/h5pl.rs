use crate::internal_prelude::*;

#[cfg(hdf5_1_8_15)]
mod hdf5_1_8_15 {
    use super::*;

    #[repr(C)]
    #[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
    pub enum H5PL_type_t {
        H5PL_TYPE_ERROR = -1,
        H5PL_TYPE_FILTER = 0,
        H5PL_TYPE_NONE = 1,
    }

    pub use self::H5PL_type_t::*;

    pub const H5PL_FILTER_PLUGIN: c_uint = 0x0001;
    pub const H5PL_ALL_PLUGIN: c_uint = 0xffff;

    extern "C" {
        pub fn H5PLget_loading_state(plugin_flags: *mut c_int) -> herr_t;
        pub fn H5PLset_loading_state(plugin_flags: *mut c_int) -> herr_t;
    }
}

#[cfg(hdf5_1_8_15)]
pub use self::hdf5_1_8_15::*;

#[cfg(hdf5_1_10_1)]
extern "C" {
    pub fn H5PLappend(search_path: *const c_char) -> herr_t;
    pub fn H5PLprepend(search_path: *const c_char) -> herr_t;
    pub fn H5PLreplace(search_path: *const c_char, index: c_uint) -> herr_t;
    pub fn H5PLinsert(search_path: *const c_char, index: c_uint) -> herr_t;
    pub fn H5PLremove(index: c_uint) -> herr_t;
    pub fn H5PLget(index: c_uint, path_buf: *mut c_char, buf_size: size_t) -> ssize_t;
    pub fn H5PLsize(num_paths: *mut c_uint) -> herr_t;
}
