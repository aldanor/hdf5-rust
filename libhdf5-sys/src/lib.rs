#![allow(non_camel_case_types, non_snake_case, dead_code)]

extern crate libc;

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

#[cfg(hdf5_1_8_15)]
pub mod h5pl;

#[cfg(test)]
mod tests {
    use super::h5::H5open;
    #[cfg(not(hdf5_1_8_14))] use super::h5p::H5P_CLS_ROOT_g as H5P_ROOT;
    #[cfg(hdf5_1_8_14)] use super::h5p::H5P_CLS_ROOT_ID_g as H5P_ROOT;

    #[test]
    pub fn test_smoke() {
        unsafe {
            H5open();
            assert!(H5P_ROOT > 0);
        }
    }
}
