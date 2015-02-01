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
pub mod h5z;

pub mod util;

pub mod types {
    pub use ffi::h5::{herr_t, hbool_t, htri_t, hsize_t, hssize_t, haddr_t};
    pub use ffi::h5i::hid_t;
    pub use ffi::h5r::hobj_ref_t;
}
