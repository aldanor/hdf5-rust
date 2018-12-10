pub use self::H5R_type_t::*;

use crate::internal_prelude::*;

use crate::h5o::H5O_type_t;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5R_type_t {
    H5R_BADTYPE = -1,
    H5R_OBJECT = 0,
    H5R_DATASET_REGION = 1,
    H5R_MAXTYPE = 2,
}

pub type hobj_ref_t = haddr_t;
pub type hdset_reg_ref_t = [c_uchar; 12usize];

#[cfg(not(hdf5_1_10_0))]
extern "C" {
    pub fn H5Rdereference(dataset: hid_t, ref_type: H5R_type_t, ref_: *const c_void) -> hid_t;
}

extern "C" {
    pub fn H5Rcreate(
        ref_: *mut c_void, loc_id: hid_t, name: *const c_char, ref_type: H5R_type_t,
        space_id: hid_t,
    ) -> herr_t;
    pub fn H5Rget_region(dataset: hid_t, ref_type: H5R_type_t, ref_: *const c_void) -> hid_t;
    pub fn H5Rget_obj_type2(
        id: hid_t, ref_type: H5R_type_t, ref_: *const c_void, obj_type: *mut H5O_type_t,
    ) -> herr_t;
    pub fn H5Rget_name(
        loc_id: hid_t, ref_type: H5R_type_t, ref_: *const c_void, name: *mut c_char, size: size_t,
    ) -> ssize_t;
}

#[cfg(hdf5_1_10_0)]
extern "C" {
    #[deprecated(note = "deprecated in HDF5 1.10.0, use H5Rdereference2()")]
    pub fn H5Rdereference1(obj_id: hid_t, ref_type: H5R_type_t, ref_: *const c_void) -> hid_t;
    pub fn H5Rdereference2(
        obj_id: hid_t, oapl_id: hid_t, ref_type: H5R_type_t, ref_: *const c_void,
    ) -> hid_t;
}

#[cfg(hdf5_1_10_0)]
pub use self::H5Rdereference1 as H5Rdereference;
