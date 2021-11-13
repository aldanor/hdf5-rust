//! Using the Virtual Object Layer
#![cfg(feature = "1.12.0")]
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

#[cfg(feature = "1.12.1")]
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum H5VL_subclass_t {
    H5VL_SUBCLS_NONE,
    H5VL_SUBCLS_INFO,
    H5VL_SUBCLS_WRAP,
    H5VL_SUBCLS_ATTR,
    H5VL_SUBCLS_DATASET,
    H5VL_SUBCLS_DATATYPE,
    H5VL_SUBCLS_FILE,
    H5VL_SUBCLS_GROUP,
    H5VL_SUBCLS_LINK,
    H5VL_SUBCLS_OBJECT,
    H5VL_SUBCLS_REQUEST,
    H5VL_SUBCLS_BLOB,
    H5VL_SUBCLS_TOKEN,
}

#[cfg(feature = "1.12.1")]
extern "C" {
    pub fn H5VLquery_optional(
        obj_id: hid_t, subcls: H5VL_subclass_t, opt_type: c_int, supported: *mut hbool_t,
    ) -> herr_t;
}
