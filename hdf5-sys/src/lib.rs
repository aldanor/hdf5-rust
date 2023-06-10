//! Rust bindings to the `hdf5` library for reading and writing data to and from storage
#![allow(non_camel_case_types, non_snake_case, dead_code, deprecated)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::unreadable_literal))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::missing_safety_doc))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::cognitive_complexity))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::upper_case_acronyms))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::wildcard_imports))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::module_name_repetitions))]
#![cfg_attr(docsrs, feature(doc_cfg))]

macro_rules! extern_static {
    ($dest:ident, $src:ident) => {
        extern "C" {
            static $src: id_t;
        }
        pub static $dest: &'static id_t = unsafe { &$src };
    };
}

#[cfg(all(feature = "mpio", not(feature = "have-parallel")))]
compile_error!("Enabling \"mpio\" feature requires HDF5 library built with MPI support");

#[cfg(all(feature = "mpio", feature = "static"))]
compile_error!("\"mpio\" and \"static\" are incompatible features");

pub mod h5;
pub mod h5a;
pub mod h5ac;
pub mod h5c;
pub mod h5d;
pub mod h5e;
pub mod h5f;
pub mod h5fd;
pub mod h5g;
pub mod h5i;
pub mod h5l;
pub mod h5mm;
pub mod h5o;
pub mod h5p;
pub mod h5r;
pub mod h5s;
pub mod h5t;
pub mod h5vl;
pub mod h5z;

#[cfg(feature = "1.8.15")]
pub mod h5pl;

#[cfg(feature = "1.14.0")]
pub mod h5es;

#[allow(non_camel_case_types)]
mod internal_prelude {
    pub use crate::h5::{
        haddr_t, hbool_t, herr_t, hsize_t, hssize_t, htri_t, H5_ih_info_t, H5_index_t,
        H5_iter_order_t,
    };
    pub use crate::h5i::hid_t;
    pub use crate::h5t::H5T_cset_t;
    pub use libc::{int64_t, off_t, size_t, ssize_t, time_t, uint32_t, uint64_t, FILE};
    pub use std::os::raw::{
        c_char, c_double, c_float, c_int, c_long, c_longlong, c_uchar, c_uint, c_ulong,
        c_ulonglong, c_void,
    };
}

#[cfg(test)]
mod tests {
    use super::h5::H5open;
    use super::h5p::H5P_CLS_ROOT;

    #[test]
    pub fn test_smoke() {
        unsafe {
            H5open();
            assert!(*H5P_CLS_ROOT > 0);
        }
    }
}
