mod macros;

pub mod h5;
pub mod h5e;
pub mod h5i;

pub mod types {
    pub use libc::{c_int, c_uint, c_void, c_char, c_ulonglong, c_longlong, size_t, ssize_t};

    pub use super::h5i::hid_t;
    pub use super::h5::{herr_t, hbool_t, htri_t, hsize_t, hssize_t, haddr_t};
}
