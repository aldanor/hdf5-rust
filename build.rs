extern crate libhdf5_sys as ffi;

use ffi::h5;
use std::os::raw::c_uint;

fn main() {
    let mut version: (c_uint, c_uint, c_uint) = (0, 0, 0);
    unsafe {
        assert_eq!(h5::H5open(), 0);
        assert_eq!(h5::H5get_libversion(&mut version.0, &mut version.1, &mut version.2), 0);
    }
    assert!(version >= (1, 8, 0));
    if version >= (1, 8, 14) {
        println!("cargo:rustc-cfg=hdf5_1_8_14");
    }
}
