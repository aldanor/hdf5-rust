pub use self::H5R_type_t::*;

use libc::{c_void, c_char, c_uchar, size_t, ssize_t};

use std::mem::size_of;

use ffi::types::{hid_t, herr_t, haddr_t};
use ffi::h5o::H5O_type_t;

lazy_static! {
    pub static ref H5R_OBJ_REF_BUF_SIZE: usize = { size_of::<haddr_t>() };
    pub static ref H5R_DSET_REG_REF_BUF_SIZE: usize = { size_of::<haddr_t>() + 4 };
}

#[test]
fn test_ref_buf_size() {
    assert_eq!(*H5R_OBJ_REF_BUF_SIZE, size_of::<haddr_t>());
    assert_eq!(*H5R_DSET_REG_REF_BUF_SIZE, size_of::<haddr_t>() + 4);
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5R_type_t {
    H5R_BADTYPE        = -1,
    H5R_OBJECT         = 0,
    H5R_DATASET_REGION = 1,
    H5R_MAXTYPE        = 2,
}

pub type hobj_ref_t      = haddr_t;
pub type hdset_reg_ref_t = [c_uchar; 12usize];

#[link(name = "hdf5")]
extern {
    pub fn H5Rcreate(_ref: *mut c_void, loc_id: hid_t, name: *const c_char, ref_type: H5R_type_t,
                     space_id: hid_t) -> herr_t;
    pub fn H5Rdereference(dataset: hid_t, ref_type: H5R_type_t, _ref: *const c_void) -> hid_t;
    pub fn H5Rget_region(dataset: hid_t, ref_type: H5R_type_t, _ref: *const c_void) -> hid_t;
    pub fn H5Rget_obj_type2(id: hid_t, ref_type: H5R_type_t, _ref: *const c_void, obj_type: *mut
                            H5O_type_t) -> herr_t;
    pub fn H5Rget_name(loc_id: hid_t, ref_type: H5R_type_t, _ref: *const c_void, name: *mut c_char,
                       size: size_t) -> ssize_t;
}
