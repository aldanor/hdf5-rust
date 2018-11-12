use libc::c_int;

use crate::h5::herr_t;

#[cfg(hdf5_1_8_15)]
extern {
    pub fn H5PLget_loading_state(plugin_flags: *mut c_int) -> herr_t;
    pub fn H5PLset_loading_state(plugin_flags: *mut c_int) -> herr_t;
}
