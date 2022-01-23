//! Event set module
use crate::internal_prelude::*;

#[repr(C)]
#[derive(Debug)]
pub struct H5ES_err_info_t {
    api_cname: *mut c_char,
    api_args: *mut c_char,

    app_file_name: *mut c_char,
    app_func_name: *mut c_char,
    app_line_num: c_uint,

    op_ins_count: u64,
    op_ins_ts: u64,
    op_exec_ts: u64,
    op_exec_time: u64,

    err_stack_id: hid_t,
}

#[repr(C)]
#[derive(Debug)]
pub struct H5ES_op_info_t {
    api_cname: *const c_char,
    api_args: *mut c_char,

    app_file_name: *const c_char,
    app_func_name: *const c_char,
    app_line_num: c_uint,

    op_ins_count: u64,
    op_ins_ts: u64,
    op_exec_ts: u64,
    op_exec_time: u64,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum H5ES_status_t {
    H5ES_STATUS_IN_PROGRESS,
    H5ES_STATUS_SUCCEED,
    H5ES_STATUS_CANCELED,
    H5ES_STATUS_FAIL,
}

pub type H5ES_event_complete_func_t = Option<
    extern "C" fn(
        op_info: *mut H5ES_op_info_t,
        status: H5ES_status_t,
        err_stack: hid_t,
        ctx: *mut c_void,
    ) -> c_int,
>;

pub type H5ES_event_insert_func_t =
    Option<extern "C" fn(op_info: *const H5ES_op_info_t, ctx: *mut c_void) -> c_int>;

extern "C" {
    pub fn H5ESinsert_request(es_id: hid_t, connector_id: hid_t, request: *mut c_void) -> herr_t;
}

extern "C" {
    pub fn H5EScancel(
        es_id: hid_t, num_not_canceled: *mut size_t, err_occured: *mut hbool_t,
    ) -> herr_t;
    pub fn H5ESclose(es_id: hid_t) -> herr_t;
    pub fn H5EScreate() -> hid_t;
    pub fn H5ESfree_err_info(num_err_info: size_t, err_info: *mut H5ES_err_info_t) -> herr_t;
    pub fn H5ESget_count(es_id: hid_t, count: *mut size_t) -> herr_t;
    pub fn H5ESget_err_count(es_id: hid_t, num_errs: *mut size_t) -> herr_t;
    pub fn H5ESget_err_info(
        es_id: hid_t, num_err_info: size_t, err_info: *mut H5ES_err_info_t,
        err_cleared: *mut size_t,
    ) -> herr_t;
    pub fn H5ESget_err_status(es_id: hid_t, err_occured: *mut hbool_t) -> herr_t;
    pub fn H5ESget_op_counter(es_id: hid_t, counter: *mut u64) -> herr_t;
    pub fn H5ESregister_complete_func(
        es_id: hid_t, func: H5ES_event_complete_func_t, ctx: *mut c_void,
    ) -> herr_t;
    pub fn H5ESregister_insert_func(
        es_id: hid_t, func: H5ES_event_insert_func_t, ctx: *mut c_void,
    ) -> herr_t;
    pub fn H5ESwait(
        es_id: hid_t, timeout: u64, num_in_progress: *mut size_t, err_occured: *mut hbool_t,
    ) -> herr_t;
}
