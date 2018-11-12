use std::os::raw::{c_int, c_uint};

extern {
    pub fn H5open() -> c_int;
    pub fn H5get_libversion(majnum: *mut c_uint, minnum: *mut c_uint,
                            relnum: *mut c_uint) -> c_int;
}

pub fn hdf5_version() -> Result<(u8, u8, u8), &'static str> {
    let mut v: (c_uint, c_uint, c_uint) = (0, 0, 0);
    unsafe {
        if H5open() != 0 {
            Err("cannot open HDF5 library")
        } else {
            if H5get_libversion(&mut v.0, &mut v.1, &mut v.2) != 0 {
                Err("cannot get HDF5 version")
            } else {
                Ok((v.0 as _, v.1 as _, v.2 as _))
            }
        }
    }
}

pub fn dump_build_flags() {
    let version = hdf5_version().unwrap();
    assert!(version >= (1, 8, 4));
    let mut vs: Vec<_> = (5..=21).map(|v| (1, 8, v)).collect();
    vs.extend((0..=1).map(|v| (1, 10, v)));
    for v in vs.into_iter().filter(|&v| version >= v) {
        println!("cargo:rustc-cfg=hdf5_{}_{}_{}", v.0, v.1, v.2);
    }
}

#[cfg(test)]
pub mod tests {
    use super::hdf5_version;

    #[test]
    fn test_version() {
        assert!(hdf5_version().unwrap() >= (1, 8, 4));
    }
}
