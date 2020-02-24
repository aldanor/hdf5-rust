use crate::internal_prelude::*;

#[cfg(hdf5_1_8_11)]
extern "C" {
    pub fn H5DOwrite_chunk(
        dset_id: hid_t, dxpl_id: hid_t, filter_mask: u32, offset: *mut hsize_t, data_size: size_t,
        buf: *const c_void,
    ) -> herr_t;
}

#[cfg(hdf5_1_10_0)]
extern "C" {
    pub fn H5DOappend(
        dset_id: hid_t, dxpl_id: hid_t, index: c_uint, num_elem: size_t, memtype: hid_t,
        buffer: *const c_void,
    ) -> herr_t;
}

#[cfg(hdf5_1_10_2)]
extern "C" {
    pub fn H5DOread_chunk(
        dset_id: hid_t, dxpl_id: hid_t, offset: *const hsize_t, filter_mask: *mut u32,
        buf: *mut c_void,
    ) -> herr_t;
}
