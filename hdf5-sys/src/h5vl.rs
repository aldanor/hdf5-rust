//! Using the Virtual Object Layer
#![cfg(hdf5_1_12_0)]
use crate::internal_prelude::*;

pub type H5VL_class_value_t = c_int;

// Incomplete type
pub type H5VL_class_t = c_void;

extern "C" {
    pub fn H5VLclose(connector_id: hid_t) -> herr_t;
    pub fn H5VLget_connector_id(obj_id: hid_t) -> hid_t;
    pub fn H5VLget_connector_id_by_name(name: *const c_char) -> hid_t;
    pub fn H5VLget_connector_id_by_value(connector_value: H5VL_class_value_t) -> hid_t;
    pub fn H5VLget_connector_name(id: hid_t, name: *mut c_char, size: size_t) -> ssize_t;
    pub fn H5VLis_connector_registered_by_name(name: *const c_char) -> htri_t;
    pub fn H5VLis_connector_registered_by_value(connector_value: H5VL_class_value_t) -> htri_t;
    pub fn H5VLregister_connector(cls: *const H5VL_class_t, vipl_id: hid_t) -> hid_t;
    pub fn H5VLregister_connector_by_name(name: *const c_char, vipl_id: hid_t) -> hid_t;
    pub fn H5VLregister_connector_by_value(
        connector_value: H5VL_class_value_t, vipl_id: hid_t,
    ) -> hid_t;
    pub fn H5VLunregister_connector(vol_id: hid_t) -> herr_t;
}
