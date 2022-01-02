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

#[cfg(feature = "1.13.0")]
#[repr(C)]
pub struct H5VL_optional_args_t {
    op_type: c_int,
    args: *mut c_void,
}

#[cfg(feature = "1.13.0")]
extern "C" {
    pub fn H5VLattr_optional_op(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, attr_id: hid_t,
        args: *mut H5VL_optional_args_t, dxpl_id: hid_t, es_id: hid_t,
    ) -> herr_t;
    pub fn H5VLdataset_optional_op(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, dset_id: hid_t,
        args: *mut H5VL_optional_args_t, dxpl_id: hid_t, es_id: hid_t,
    ) -> herr_t;
    pub fn H5VLdatatype_optional_op(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, type_id: hid_t,
        args: *mut H5VL_optional_args_t, dxpl_id: hid_t, es_id: hid_t,
    ) -> herr_t;
    pub fn H5VLfile_optional_op(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, file_id: hid_t,
        args: *mut H5VL_optional_args_t, dxpl_id: hid_t, es_id: hid_t,
    ) -> herr_t;
    pub fn H5VLfind_opt_operation(
        subcls: H5VL_subclass_t, op_name: *const c_char, op_val: *mut c_int,
    ) -> herr_t;
    pub fn H5VLgroup_optional_op(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, group_id: hid_t,
        args: *mut H5VL_optional_args_t, dxpl_id: hid_t, es_id: hid_t,
    ) -> herr_t;
    pub fn H5VLlink_optional_op(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, loc_id: hid_t,
        name: *const c_char, lapl_id: hid_t, args: *mut H5VL_optional_args_t, dxpl_id: hid_t,
        es_id: hid_t,
    ) -> herr_t;
    pub fn H5VLobject_optional_op(
        app_file: *const c_char, app_func: *const c_char, app_line: c_uint, loc_id: hid_t,
        name: *const c_char, lapl_id: hid_t, args: *mut H5VL_optional_args_t, dxpl_id: hid_t,
        es_id: hid_t,
    ) -> herr_t;
    pub fn H5VLregister_opt_operation(
        subcls: H5VL_subclass_t, op_name: *const c_char, op_val: *mut c_int,
    ) -> herr_t;
    pub fn H5VLrequest_optional_op(
        req: *mut c_void, connector_id: hid_t, args: *mut H5VL_optional_args_t,
    ) -> herr_t;
    pub fn H5VLunregister_opt_operation(subcls: H5VL_subclass_t, op_name: *const c_char) -> herr_t;
}

#[cfg(feature = "1.13.0")]
extern "C" {
    pub fn H5VLfinish_lib_state() -> herr_t;
    pub fn H5VLintrospect_get_cap_flags(
        info: *const c_void, connector_id: hid_t, cap_flags: *mut c_uint,
    ) -> herr_t;
    pub fn H5VLstart_lib_state() -> herr_t;
}

#[cfg(feature = "1.13.0")]
extern "C" {
    pub fn H5VLobject_is_native(obj_id: hid_t, is_native: *mut hbool_t) -> herr_t;
}
