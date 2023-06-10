//! Using the Virtual Object Layer
#![cfg(feature = "1.12.0")]

use crate::internal_prelude::*;

pub type H5VL_class_value_t = c_int;

// Incomplete type
#[cfg(all(feature = "1.12.0", not(feature = "1.14.0")))]
pub type H5VL_class_t = c_void;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[cfg(feature = "1.14.0")]
pub struct H5VL_class_t {
    pub version: c_uint,
    pub value: H5VL_class_value_t,
    pub name: *const c_char,
    pub conn_version: c_uint,
    pub cap_flags: c_uint,
    pub initialize: Option<extern "C" fn(vipl_id: hid_t) -> herr_t>,
    pub terminate: Option<extern "C" fn() -> herr_t>,

    pub info_cls: H5VL_info_class_t,
    pub wrap_cls: H5VL_wrap_class_t,

    pub attr_cls: H5VL_attr_class_t,
    pub dataset_cls: H5VL_dataset_class_t,
    pub datatype_cls: H5VL_datatype_class_t,
    pub file_cls: H5VL_file_class_t,
    pub group_cls: H5VL_group_class_t,
    pub link_cls: H5VL_link_class_t,
    pub object_cls: H5VL_object_class_t,

    pub introspect_cls: H5VL_introspect_class_t,
    pub request_cls: H5VL_request_class_t,
    pub blob_cls: H5VL_blob_class_t,
    pub token_cls: H5VL_token_class_t,

    pub optional: Option<
        extern "C" fn(
            obj: *mut c_void,
            args: *mut H5VL_optional_args_t,
            dxpl_id: hid_t,
            req: *mut *mut c_void,
        ) -> herr_t,
    >,
}

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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[cfg(feature = "1.14.0")]
pub use v1_14_0::*;
#[cfg(feature = "1.14.0")]
mod v1_14_0 {
    use std::fmt::{self, Debug};
    use std::mem::ManuallyDrop;

    use crate::{
        h5a::{H5A_info_t, H5A_operator2_t},
        h5d::H5D_space_status_t,
        h5f::H5F_scope_t,
        h5g::H5G_info_t,
        h5i::H5I_type_t,
        h5l::{H5L_info2_t, H5L_iterate2_t, H5L_type_t},
        h5o::{H5O_info2_t, H5O_iterate2_t, H5O_token_t, H5O_type_t},
    };

    use super::*;

    macro_rules! impl_debug_args {
        ($ty:ty, $tag:ident, $args:ty, {$($variant:ident => $func:expr),+$(,)*}) => {
            #[allow(unreachable_patterns)]
            impl std::fmt::Debug for $ty {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    let mut s = f.debug_struct(stringify!($ty));
                    s.field("op_type", &self.op_type);
                    match self.op_type {
                        $($tag::$variant => {
                            s.field("args", &($func as fn($args) -> _)(self.args));
                        })+
                        _ => {}
                    }
                    s.finish()
                }
            }
        };
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_info_class_t {
        pub size: size_t,
        pub copy: Option<extern "C" fn(info: *const c_void) -> *mut c_void>,
        pub cmp: Option<
            extern "C" fn(
                cmp_value: *mut c_int,
                info1: *const c_void,
                info2: *const c_void,
            ) -> herr_t,
        >,
        pub free: Option<extern "C" fn(info: *mut c_void) -> herr_t>,
        pub to_str: Option<extern "C" fn(info: *const c_void, str: *mut *mut c_char) -> herr_t>,
        pub from_str: Option<extern "C" fn(str: *const c_char, info: *mut *mut c_void) -> herr_t>,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_wrap_class_t {
        pub get_object: Option<extern "C" fn(obj: *const c_void) -> *mut c_void>,
        pub get_wrap_ctx:
            Option<extern "C" fn(obj: *const c_void, wrap_ctx: *mut *mut c_void) -> herr_t>,
        pub wrap_object: Option<
            extern "C" fn(
                obj: *mut c_void,
                obj_type: H5I_type_t,
                wrap_ctx: *mut c_void,
            ) -> *mut c_void,
        >,
        pub unwrap_object: Option<extern "C" fn(obj: *mut c_void) -> *mut c_void>,
        pub free_wrap_ctx: Option<extern "C" fn(wrap_ctx: *mut c_void) -> herr_t>,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_attr_get_t {
        H5VL_ATTR_GET_ACPL,
        H5VL_ATTR_GET_INFO,
        H5VL_ATTR_GET_NAME,
        H5VL_ATTR_GET_SPACE,
        H5VL_ATTR_GET_STORAGE_SIZE,
        H5VL_ATTR_GET_TYPE,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_attr_get_args_t_union_get_acpl {
        pub acpl_id: hid_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_attr_get_args_t_union_get_space {
        pub space_id: hid_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_attr_get_args_t_union_get_storage_size {
        pub data_size: *mut hsize_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_attr_get_args_t_union_get_type {
        pub type_id: hid_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_attr_get_name_args_t {
        pub loc_params: H5VL_loc_params_t,
        pub buf_size: size_t,
        pub buf: *mut c_char,
        pub attr_name_len: *mut size_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_attr_get_info_args_t {
        pub loc_params: H5VL_loc_params_t,
        pub attr_name: *const c_char,
        pub ainfo: *mut H5A_info_t,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union H5VL_attr_get_args_t_union {
        pub get_acpl: ManuallyDrop<H5VL_attr_get_args_t_union_get_acpl>,
        pub get_info: ManuallyDrop<H5VL_attr_get_info_args_t>,
        pub get_name: ManuallyDrop<H5VL_attr_get_name_args_t>,
        pub get_space: ManuallyDrop<H5VL_attr_get_args_t_union_get_space>,
        pub get_storage_size: ManuallyDrop<H5VL_attr_get_args_t_union_get_storage_size>,
        pub get_type: ManuallyDrop<H5VL_attr_get_args_t_union_get_type>,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct H5VL_attr_get_args_t {
        pub op_type: H5VL_attr_get_t,
        pub args: H5VL_attr_get_args_t_union,
    }

    impl_debug_args!(
        H5VL_attr_get_args_t,
        H5VL_attr_get_t,
        H5VL_attr_get_args_t_union,
        {
            H5VL_ATTR_GET_ACPL => |args| unsafe { args.get_acpl },
            H5VL_ATTR_GET_INFO => |args| unsafe { args.get_info },
            H5VL_ATTR_GET_NAME => |args| unsafe { args.get_name },
            H5VL_ATTR_GET_SPACE => |args| unsafe { args.get_space },
            H5VL_ATTR_GET_STORAGE_SIZE => |args| unsafe { args.get_storage_size },
            H5VL_ATTR_GET_TYPE => |args| unsafe { args.get_type },
        }
    );

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_attr_specific_t {
        H5VL_ATTR_DELETE,
        H5VL_ATTR_DELETE_BY_IDX,
        H5VL_ATTR_EXISTS,
        H5VL_ATTR_ITER,
        H5VL_ATTR_RENAME,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_attr_specific_args_t_union_del {
        pub name: *const c_char,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_attr_specific_args_t_union_exists {
        pub name: *const c_char,
        pub exists: *mut hbool_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_attr_specific_args_t_union_rename {
        pub old_name: *const c_char,
        pub new_name: *const c_char,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_attr_iterate_args_t {
        pub idx_type: H5_index_t,
        pub order: H5_iter_order_t,
        pub idx: *mut hsize_t,
        pub op: H5A_operator2_t,
        pub op_data: *mut c_void,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_attr_delete_by_idx_args_t {
        pub idx_type: H5_index_t,
        pub order: H5_iter_order_t,
        pub n: hsize_t,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union H5VL_attr_specific_args_t_union {
        pub del: ManuallyDrop<H5VL_attr_specific_args_t_union_del>,
        pub delete_by_idx: ManuallyDrop<H5VL_attr_delete_by_idx_args_t>,
        pub exists: ManuallyDrop<H5VL_attr_specific_args_t_union_exists>,
        pub iterate: ManuallyDrop<H5VL_attr_iterate_args_t>,
        pub rename: ManuallyDrop<H5VL_attr_specific_args_t_union_rename>,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct H5VL_attr_specific_args_t {
        pub op_type: H5VL_attr_specific_t,
        pub args: H5VL_attr_specific_args_t_union,
    }

    impl_debug_args!(
        H5VL_attr_specific_args_t,
        H5VL_attr_specific_t,
        H5VL_attr_specific_args_t_union,
        {
            H5VL_ATTR_DELETE => |args| unsafe { args.del },
            H5VL_ATTR_DELETE_BY_IDX => |args| unsafe { args.delete_by_idx },
            H5VL_ATTR_EXISTS => |args| unsafe { args.exists },
            H5VL_ATTR_ITER => |args| unsafe { args.iterate },
            H5VL_ATTR_RENAME => |args| unsafe { args.rename },
        }
    );

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_attr_class_t {
        pub create: Option<
            extern "C" fn(
                obj: *mut c_void,
                loc_params: *const H5VL_loc_params_t,
                attr_name: *const c_char,
                type_id: hid_t,
                space_id: hid_t,
                acpl_id: hid_t,
                aapl_id: hid_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> *mut c_void,
        >,
        pub open: Option<
            extern "C" fn(
                obj: *mut c_void,
                loc_params: *const H5VL_loc_params_t,
                attr_name: *const c_char,
                aapl_id: hid_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> *mut c_void,
        >,
        pub read: Option<
            extern "C" fn(
                attr: *mut c_void,
                mem_type_id: hid_t,
                buf: *mut c_void,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub write: Option<
            extern "C" fn(
                attr: *mut c_void,
                mem_type_id: hid_t,
                buf: *const c_void,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub get: Option<
            extern "C" fn(
                obj: *mut c_void,
                args: *mut H5VL_attr_get_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub specific: Option<
            extern "C" fn(
                obj: *mut c_void,
                loc_params: *const H5VL_loc_params_t,
                args: *mut H5VL_attr_specific_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub optional: Option<
            extern "C" fn(
                obj: *mut c_void,
                args: *mut H5VL_optional_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub close: Option<
            extern "C" fn(attr: *mut c_void, dxpl_id: hid_t, req: *mut *mut c_void) -> herr_t,
        >,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_dataset_specific_t {
        H5VL_DATASET_SET_EXTENT,
        H5VL_DATASET_FLUSH,
        H5VL_DATASET_REFRESH,
    }

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct H5VL_dataset_specific_args_t_union_set_extent {
        pub size: *const hsize_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_dataset_specific_args_t_union_flush {
        pub dset_id: hid_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_dataset_specific_args_t_union_refresh {
        pub dset_id: hid_t,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union H5VL_dataset_specific_args_t_union {
        pub set_extent: ManuallyDrop<H5VL_dataset_specific_args_t_union_set_extent>,
        pub flush: ManuallyDrop<H5VL_dataset_specific_args_t_union_flush>,
        pub refresh: ManuallyDrop<H5VL_dataset_specific_args_t_union_refresh>,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct H5VL_dataset_specific_args_t {
        pub op_type: H5VL_dataset_specific_t,
        pub args: H5VL_dataset_specific_args_t_union,
    }

    impl_debug_args!(
        H5VL_dataset_specific_args_t,
        H5VL_dataset_specific_t,
        H5VL_dataset_specific_args_t_union,
        {
            H5VL_DATASET_SET_EXTENT => |args| unsafe { args.set_extent },
            H5VL_DATASET_FLUSH => |args| unsafe { args.flush },
            H5VL_DATASET_REFRESH => |args| unsafe { args.refresh },
        }
    );

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_dataset_get_t {
        H5VL_DATASET_GET_DAPL,
        H5VL_DATASET_GET_DCPL,
        H5VL_DATASET_GET_SPACE,
        H5VL_DATASET_GET_SPACE_STATUS,
        H5VL_DATASET_GET_STORAGE_SIZE,
        H5VL_DATASET_GET_TYPE,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_dataset_get_args_t_union_get_dapl {
        pub dapl_id: hid_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_dataset_get_args_t_union_get_dcpl {
        pub dcpl_id: hid_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_dataset_get_args_t_union_get_space {
        pub space_id: hid_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_dataset_get_args_t_union_get_space_status {
        pub status: *mut H5D_space_status_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_dataset_get_args_t_union_get_storage_size {
        pub storage_size: *mut hsize_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_dataset_get_args_t_union_get_type {
        pub type_id: hid_t,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union H5VL_dataset_get_args_t_union {
        pub get_dapl: ManuallyDrop<H5VL_dataset_get_args_t_union_get_dapl>,
        pub get_dcpl: ManuallyDrop<H5VL_dataset_get_args_t_union_get_dcpl>,
        pub get_space: ManuallyDrop<H5VL_dataset_get_args_t_union_get_space>,
        pub get_space_status: ManuallyDrop<H5VL_dataset_get_args_t_union_get_space_status>,
        pub get_storage_size: ManuallyDrop<H5VL_dataset_get_args_t_union_get_storage_size>,
        pub get_type: ManuallyDrop<H5VL_dataset_get_args_t_union_get_type>,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct H5VL_dataset_get_args_t {
        pub op_type: H5VL_dataset_get_t,
        pub args: H5VL_dataset_get_args_t_union,
    }

    impl_debug_args!(
        H5VL_dataset_get_args_t,
        H5VL_dataset_get_t,
        H5VL_dataset_get_args_t_union,
        {
            H5VL_DATASET_GET_DAPL => |args| unsafe { args.get_dapl },
            H5VL_DATASET_GET_DCPL => |args| unsafe { args.get_dcpl },
            H5VL_DATASET_GET_SPACE => |args| unsafe { args.get_space },
            H5VL_DATASET_GET_SPACE_STATUS => |args| unsafe { args.get_space_status },
            H5VL_DATASET_GET_STORAGE_SIZE => |args| unsafe { args.get_storage_size },
            H5VL_DATASET_GET_TYPE => |args| unsafe { args.get_type },
        }
    );

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_dataset_class_t {
        pub create: Option<
            extern "C" fn(
                obj: *mut c_void,
                loc_params: *const H5VL_loc_params_t,
                name: *const c_char,
                lcpl_id: hid_t,
                type_id: hid_t,
                space_id: hid_t,
                dcpl_id: hid_t,
                dapl_id: hid_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> *mut c_void,
        >,
        pub open: Option<
            extern "C" fn(
                obj: *mut c_void,
                loc_params: *const H5VL_loc_params_t,
                name: *const c_char,
                dapl_id: hid_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> *mut c_void,
        >,
        pub read: Option<
            extern "C" fn(
                dset: *mut c_void,
                mem_type_id: hid_t,
                mem_space_id: hid_t,
                file_space_id: hid_t,
                dxpl_id: hid_t,
                buf: *mut c_void,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub write: Option<
            extern "C" fn(
                dset: *mut c_void,
                mem_type_id: hid_t,
                mem_space_id: hid_t,
                file_space_id: hid_t,
                dxpl_id: hid_t,
                buf: *const c_void,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub get: Option<
            extern "C" fn(
                obj: *mut c_void,
                args: *mut H5VL_dataset_get_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub specific: Option<
            extern "C" fn(
                obj: *mut c_void,
                args: *mut H5VL_dataset_specific_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub optional: Option<
            extern "C" fn(
                obj: *mut c_void,
                args: *mut H5VL_optional_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub close: Option<
            extern "C" fn(dset: *mut c_void, dxpl_id: hid_t, req: *mut *mut c_void) -> herr_t,
        >,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_file_specific_t {
        H5VL_FILE_FLUSH,
        H5VL_FILE_REOPEN,
        H5VL_FILE_IS_ACCESSIBLE,
        H5VL_FILE_DELETE,
        H5VL_FILE_IS_EQUAL,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_datatype_specific_args_t_union_flush {
        pub type_id: hid_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_datatype_specific_args_t_union_refresh {
        pub type_id: hid_t,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union H5VL_datatype_specific_args_t_union {
        pub flush: ManuallyDrop<H5VL_datatype_specific_args_t_union_flush>,
        pub refresh: ManuallyDrop<H5VL_datatype_specific_args_t_union_refresh>,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_datatype_specific_t {
        H5VL_DATATYPE_FLUSH,
        H5VL_DATATYPE_REFRESH,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct H5VL_datatype_specific_args_t {
        pub op_type: H5VL_datatype_specific_t,
        pub args: H5VL_datatype_specific_args_t_union,
    }

    impl_debug_args!(
        H5VL_datatype_specific_args_t,
        H5VL_datatype_specific_t,
        H5VL_datatype_specific_args_t_union,
        {
            H5VL_DATATYPE_FLUSH => |args| unsafe { args.flush },
            H5VL_DATATYPE_REFRESH => |args| unsafe { args.refresh },
        }
    );

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_datatype_get_t {
        H5VL_DATATYPE_GET_BINARY_SIZE,
        H5VL_DATATYPE_GET_BINARY,
        H5VL_DATATYPE_GET_TCPL,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_datatype_get_args_t_union_get_binary_size {
        pub size: *mut size_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_datatype_get_args_t_union_get_binary {
        pub buf: *mut c_void,
        pub buf_size: size_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_datatype_get_args_t_union_get_tcpl {
        pub tcpl_id: hid_t,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union H5VL_datatype_get_args_t_union {
        pub get_binary_size: ManuallyDrop<H5VL_datatype_get_args_t_union_get_binary_size>,
        pub get_binary: ManuallyDrop<H5VL_datatype_get_args_t_union_get_binary>,
        pub get_tcpl: ManuallyDrop<H5VL_datatype_get_args_t_union_get_tcpl>,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct H5VL_datatype_get_args_t {
        pub op_type: H5VL_datatype_get_t,
        pub args: H5VL_datatype_get_args_t_union,
    }

    impl_debug_args!(
        H5VL_datatype_get_args_t,
        H5VL_datatype_get_t,
        H5VL_datatype_get_args_t_union,
        {
            H5VL_DATATYPE_GET_BINARY_SIZE => |args| unsafe { args.get_binary_size },
            H5VL_DATATYPE_GET_BINARY => |args| unsafe { args.get_binary },
            H5VL_DATATYPE_GET_TCPL => |args| unsafe { args.get_tcpl },
        }
    );

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_datatype_class_t {
        pub commit: Option<
            extern "C" fn(
                obj: *mut c_void,
                loc_params: *const H5VL_loc_params_t,
                name: *const c_char,
                type_id: hid_t,
                lcpl_id: hid_t,
                tcpl_id: hid_t,
                tapl_id: hid_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> *mut c_void,
        >,
        pub open: Option<
            extern "C" fn(
                obj: *mut c_void,
                loc_params: *const H5VL_loc_params_t,
                name: *const c_char,
                tapl_id: hid_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> *mut c_void,
        >,
        pub get: Option<
            extern "C" fn(
                obj: *mut c_void,
                args: *mut H5VL_datatype_get_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub specific: Option<
            extern "C" fn(
                obj: *mut c_void,
                args: *mut H5VL_datatype_specific_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub optional: Option<
            extern "C" fn(
                obj: *mut c_void,
                args: *mut H5VL_optional_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub close:
            Option<extern "C" fn(dt: *mut c_void, dxpl_id: hid_t, req: *mut *mut c_void) -> herr_t>,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_file_specific_args_t_union_flush {
        pub obj_type: H5I_type_t,
        pub scope: H5F_scope_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_file_specific_args_t_union_reopen {
        pub file: *mut *mut c_void,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_file_specific_args_t_union_is_accessible {
        pub filename: *const c_char,
        pub fapl_id: hid_t,
        pub accessible: *mut hbool_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_file_specific_args_t_union_del {
        pub filename: *const c_char,
        pub fapl_id: hid_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_file_specific_args_t_union_is_equal {
        pub obj2: *mut c_void,
        pub same_file: *mut hbool_t,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union H5VL_file_specific_args_t_union {
        pub flush: ManuallyDrop<H5VL_file_specific_args_t_union_flush>,
        pub reopen: ManuallyDrop<H5VL_file_specific_args_t_union_reopen>,
        pub is_accessible: ManuallyDrop<H5VL_file_specific_args_t_union_is_accessible>,
        pub del: ManuallyDrop<H5VL_file_specific_args_t_union_del>,
        pub is_equal: ManuallyDrop<H5VL_file_specific_args_t_union_is_equal>,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct H5VL_file_specific_args_t {
        pub op_type: H5VL_file_specific_t,
        pub args: H5VL_file_specific_args_t_union,
    }

    impl_debug_args!(
        H5VL_file_specific_args_t,
        H5VL_file_specific_t,
        H5VL_file_specific_args_t_union,
        {
            H5VL_FILE_FLUSH => |args| unsafe { args.flush },
            H5VL_FILE_REOPEN => |args| unsafe { args.reopen },
            H5VL_FILE_IS_ACCESSIBLE => |args| unsafe { args.is_accessible },
            H5VL_FILE_DELETE => |args| unsafe { args.del },
            H5VL_FILE_IS_EQUAL => |args| unsafe { args.is_equal },
        }
    );

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_file_cont_info_t {
        pub version: c_uint,
        pub feature_flags: u64,
        pub token_size: size_t,
        pub blob_id_size: size_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_file_get_args_t_union_get_cont_info {
        pub info: *mut H5VL_file_cont_info_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_file_get_args_t_union_get_fapl {
        pub fapl_id: hid_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_file_get_args_t_union_get_fcpl {
        pub fcpl_id: hid_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_file_get_args_t_union_get_fileno {
        pub fileno: *mut c_ulong,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_file_get_args_t_union_get_intent {
        pub flags: *mut c_uint,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_file_get_args_t_union_get_obj_count {
        pub types: c_uint,
        pub count: *mut size_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_file_get_obj_ids_args_t {
        pub types: c_uint,
        pub max_objs: size_t,
        pub old_list: *mut hid_t,
        pub count: *mut size_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_file_get_name_args_t {
        pub r#type: H5I_type_t,
        pub buf_size: size_t,
        pub buf: *mut c_char,
        pub file_name_len: *mut size_t,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union H5VL_file_get_args_t_union {
        pub get_cont_info: ManuallyDrop<H5VL_file_get_args_t_union_get_cont_info>,
        pub get_fapl: ManuallyDrop<H5VL_file_get_args_t_union_get_fapl>,
        pub get_fcpl: ManuallyDrop<H5VL_file_get_args_t_union_get_fcpl>,
        pub get_fileno: ManuallyDrop<H5VL_file_get_args_t_union_get_fileno>,
        pub get_intent: ManuallyDrop<H5VL_file_get_args_t_union_get_intent>,
        pub get_name: ManuallyDrop<H5VL_file_get_name_args_t>,
        pub get_obj_count: ManuallyDrop<H5VL_file_get_args_t_union_get_obj_count>,
        pub get_obj_ids: ManuallyDrop<H5VL_file_get_obj_ids_args_t>,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_file_get_t {
        H5VL_FILE_GET_CONT_INFO,
        H5VL_FILE_GET_FAPL,
        H5VL_FILE_GET_FCPL,
        H5VL_FILE_GET_FILENO,
        H5VL_FILE_GET_INTENT,
        H5VL_FILE_GET_NAME,
        H5VL_FILE_GET_OBJ_COUNT,
        H5VL_FILE_GET_OBJ_IDS,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct H5VL_file_get_args_t {
        pub op_type: H5VL_file_get_t,
        pub args: H5VL_file_get_args_t_union,
    }

    impl_debug_args!(
        H5VL_file_get_args_t,
        H5VL_file_get_t,
        H5VL_file_get_args_t_union,
        {
            H5VL_FILE_GET_CONT_INFO => |args| unsafe { args.get_cont_info },
            H5VL_FILE_GET_FAPL => |args| unsafe { args.get_fapl },
            H5VL_FILE_GET_FCPL => |args| unsafe { args.get_fcpl },
            H5VL_FILE_GET_FILENO => |args| unsafe { args.get_fileno },
            H5VL_FILE_GET_INTENT => |args| unsafe { args.get_intent },
            H5VL_FILE_GET_NAME => |args| unsafe { args.get_name },
            H5VL_FILE_GET_OBJ_COUNT => |args| unsafe { args.get_obj_count },
            H5VL_FILE_GET_OBJ_IDS => |args| unsafe { args.get_obj_ids },
        }
    );

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_file_class_t {
        pub create: Option<
            extern "C" fn(
                name: *const c_char,
                flags: c_uint,
                fcpl_id: hid_t,
                fapl_id: hid_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> *mut c_void,
        >,
        pub open: Option<
            extern "C" fn(
                name: *const c_char,
                flags: c_uint,
                fapl_id: hid_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> *mut c_void,
        >,
        pub get: Option<
            extern "C" fn(
                obj: *mut c_void,
                args: *mut H5VL_file_get_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub specific: Option<
            extern "C" fn(
                obj: *mut c_void,
                args: *mut H5VL_file_specific_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub optional: Option<
            extern "C" fn(
                obj: *mut c_void,
                args: *mut H5VL_optional_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub close: Option<
            extern "C" fn(file: *mut c_void, dxpl_id: hid_t, req: *mut *mut c_void) -> herr_t,
        >,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_group_specific_t {
        H5VL_GROUP_MOUNT,
        H5VL_GROUP_UNMOUNT,
        H5VL_GROUP_FLUSH,
        H5VL_GROUP_REFRESH,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_group_spec_mount_args_t {
        pub name: *const c_char,
        pub child_file: *mut c_void,
        pub fmpl_id: hid_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_group_specific_args_t_union_unmount {
        pub name: *const c_char,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_group_specific_args_t_union_flush {
        pub grp_id: hid_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_group_specific_args_t_union_refresh {
        pub grp_id: hid_t,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union H5VL_group_specific_args_t_union {
        pub mount: ManuallyDrop<H5VL_group_spec_mount_args_t>,
        pub unmount: ManuallyDrop<H5VL_group_specific_args_t_union_unmount>,
        pub flush: ManuallyDrop<H5VL_group_specific_args_t_union_flush>,
        pub refresh: ManuallyDrop<H5VL_group_specific_args_t_union_refresh>,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct H5VL_group_specific_args_t {
        pub op_type: H5VL_group_specific_t,
        pub args: H5VL_group_specific_args_t_union,
    }

    impl_debug_args!(
        H5VL_group_specific_args_t,
        H5VL_group_specific_t,
        H5VL_group_specific_args_t_union,
        {
            H5VL_GROUP_MOUNT => |args| unsafe { args.mount },
            H5VL_GROUP_UNMOUNT => |args| unsafe { args.unmount },
            H5VL_GROUP_FLUSH => |args| unsafe { args.flush },
            H5VL_GROUP_REFRESH => |args| unsafe { args.refresh },
        }
    );

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_group_get_info_args_t {
        pub loc_params: H5VL_loc_params_t,
        pub ginfo: *mut H5G_info_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_group_get_args_t_union_get_gcpl {
        pub gcpl_id: hid_t,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union H5VL_group_get_args_t_union {
        pub get_gcpl: ManuallyDrop<H5VL_group_get_args_t_union_get_gcpl>,
        pub get_info: ManuallyDrop<H5VL_group_get_info_args_t>,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_group_get_t {
        H5VL_GROUP_GET_GCPL,
        H5VL_GROUP_GET_INFO,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct H5VL_group_get_args_t {
        pub op_type: H5VL_group_get_t,
        pub args: H5VL_group_get_args_t_union,
    }

    impl_debug_args!(
        H5VL_group_get_args_t,
        H5VL_group_get_t,
        H5VL_group_get_args_t_union,
        {
            H5VL_GROUP_GET_GCPL => |args| unsafe { args.get_gcpl },
            H5VL_GROUP_GET_INFO => |args| unsafe { args.get_info },
        }
    );

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_group_class_t {
        pub create: Option<
            extern "C" fn(
                obj: *mut c_void,
                loc_params: *const H5VL_loc_params_t,
                name: *const c_char,
                lcpl_id: hid_t,
                gcpl_id: hid_t,
                gapl_id: hid_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> *mut c_void,
        >,
        pub open: Option<
            extern "C" fn(
                obj: *mut c_void,
                loc_params: *const H5VL_loc_params_t,

                name: *const c_char,
                gapl_id: hid_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> *mut c_void,
        >,
        pub get: Option<
            extern "C" fn(
                obj: *mut c_void,
                args: *mut H5VL_group_get_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub specific: Option<
            extern "C" fn(
                obj: *mut c_void,
                args: *mut H5VL_group_specific_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub optional: Option<
            extern "C" fn(
                obj: *mut c_void,
                args: *mut H5VL_optional_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub close: Option<
            extern "C" fn(grp: *mut c_void, dxpl_id: hid_t, req: *mut *mut c_void) -> herr_t,
        >,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_link_iterate_args_t {
        pub recursive: hbool_t,
        pub idx_type: H5_index_t,
        pub order: H5_iter_order_t,
        pub idx_p: *mut hsize_t,
        pub op: H5L_iterate2_t,
        pub op_data: *mut c_void,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_link_specific_args_t_union_exists {
        pub exists: *mut hbool_t,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union H5VL_link_specific_args_t_union {
        pub exists: ManuallyDrop<H5VL_link_specific_args_t_union_exists>,
        pub iterate: ManuallyDrop<H5VL_link_iterate_args_t>,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_link_specific_t {
        H5VL_LINK_DELETE,
        H5VL_LINK_EXISTS,
        H5VL_LINK_ITER,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct H5VL_link_specific_args_t {
        pub op_type: H5VL_link_specific_t,
        pub args: H5VL_link_specific_args_t_union,
    }

    impl_debug_args!(
        H5VL_link_specific_args_t,
        H5VL_link_specific_t,
        H5VL_link_specific_args_t_union,
        {
            H5VL_LINK_EXISTS => |args| unsafe { args.exists },
            H5VL_LINK_ITER => |args| unsafe { args.iterate },
        }
    );

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_link_get_args_t_union_get_info {
        pub linfo: *mut H5L_info2_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_link_get_args_t_union_get_name {
        pub name_size: size_t,
        pub name: *mut c_char,
        pub name_len: *mut size_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_link_get_args_t_union_get_val {
        pub buf_size: size_t,
        pub buf: *mut c_void,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union H5VL_link_get_args_t_union {
        pub get_info: ManuallyDrop<H5VL_link_get_args_t_union_get_info>,
        pub get_name: ManuallyDrop<H5VL_link_get_args_t_union_get_name>,
        pub get_val: ManuallyDrop<H5VL_link_get_args_t_union_get_val>,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_link_get_t {
        H5VL_LINK_GET_INFO,
        H5VL_LINK_GET_NAME,
        H5VL_LINK_GET_VAL,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct H5VL_link_get_args_t {
        pub op_type: H5VL_link_get_t,
        pub args: H5VL_link_get_args_t_union,
    }

    impl_debug_args!(
        H5VL_link_get_args_t,
        H5VL_link_get_t,
        H5VL_link_get_args_t_union,
        {
            H5VL_LINK_GET_INFO => |args| unsafe { args.get_info },
            H5VL_LINK_GET_NAME => |args| unsafe { args.get_name },
            H5VL_LINK_GET_VAL => |args| unsafe { args.get_val },
        }
    );

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_link_create_t {
        H5VL_LINK_CREATE_HARD,
        H5VL_LINK_CREATE_SOFT,
        H5VL_LINK_CREATE_UD,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_link_create_args_t_union_hard {
        pub curr_obj: *mut c_void,
        pub curr_loc_params: H5VL_loc_params_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_link_create_args_t_union_soft {
        pub target: *const c_char,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_link_create_args_t_union_ud {
        pub r#type: H5L_type_t,
        pub buf: *const c_void,
        pub buf_size: size_t,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union H5VL_link_create_args_t_union {
        pub hard: ManuallyDrop<H5VL_link_create_args_t_union_hard>,
        pub soft: ManuallyDrop<H5VL_link_create_args_t_union_soft>,
        pub ud: ManuallyDrop<H5VL_link_create_args_t_union_ud>,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct H5VL_link_create_args_t {
        pub op_type: H5VL_link_create_t,
        pub args: H5VL_link_create_args_t_union,
    }

    impl_debug_args!(
        H5VL_link_create_args_t,
        H5VL_link_create_t,
        H5VL_link_create_args_t_union,
        {
            H5VL_LINK_CREATE_HARD => |args| unsafe { args.hard },
            H5VL_LINK_CREATE_SOFT => |args| unsafe { args.soft },
            H5VL_LINK_CREATE_UD => |args| unsafe { args.ud },
        }
    );

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_link_class_t {
        pub create: Option<
            extern "C" fn(
                args: *mut H5VL_link_create_args_t,
                obj: *mut c_void,
                loc_params: *const H5VL_loc_params_t,
                lcpl_id: hid_t,
                lapl_id: hid_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub copy: Option<
            extern "C" fn(
                src_obj: *mut c_void,
                loc_params1: *const H5VL_loc_params_t,
                dest_obj: *mut c_void,
                loc_params2: *const H5VL_loc_params_t,
                lcpl_id: hid_t,
                lapl_id: hid_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub r#move: Option<
            extern "C" fn(
                src_obj: *mut c_void,
                loc_params1: *const H5VL_loc_params_t,
                dest_obj: *mut c_void,
                loc_params2: *const H5VL_loc_params_t,
                lcpl_id: hid_t,
                lapl_id: hid_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub get: Option<
            extern "C" fn(
                obj: *mut c_void,
                loc_params: *const H5VL_loc_params_t,
                args: *mut H5VL_link_get_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub specific: Option<
            extern "C" fn(
                obj: *mut c_void,
                loc_params: *const H5VL_loc_params_t,
                args: *mut H5VL_link_specific_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub optional: Option<
            extern "C" fn(
                obj: *mut c_void,
                loc_params: *const H5VL_loc_params_t,
                args: *mut H5VL_optional_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_object_visit_args_t {
        pub idx_type: H5_index_t,
        pub order: H5_iter_order_t,
        pub fields: c_uint,
        pub op: H5O_iterate2_t,
        pub op_data: *mut c_void,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_object_specific_args_t_union_change_rc {
        pub delta: c_int,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_object_specific_args_t_union_exists {
        pub exists: *mut hbool_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_object_specific_args_t_union_lookup {
        pub token_ptr: *mut H5O_token_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_object_specific_args_t_union_flush {
        pub obj_id: hid_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_object_specific_args_t_union_refresh {
        pub obj_id: hid_t,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union H5VL_object_specific_args_t_union {
        pub change_rc: ManuallyDrop<H5VL_object_specific_args_t_union_change_rc>,
        pub exists: ManuallyDrop<H5VL_object_specific_args_t_union_exists>,
        pub lookup: ManuallyDrop<H5VL_object_specific_args_t_union_lookup>,
        pub visit: ManuallyDrop<H5VL_object_visit_args_t>,
        pub flush: ManuallyDrop<H5VL_object_specific_args_t_union_flush>,
        pub refresh: ManuallyDrop<H5VL_object_specific_args_t_union_refresh>,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_object_specific_t {
        H5VL_OBJECT_CHANGE_REF_COUNT,
        H5VL_OBJECT_EXISTS,
        H5VL_OBJECT_LOOKUP,
        H5VL_OBJECT_VISIT,
        H5VL_OBJECT_FLUSH,
        H5VL_OBJECT_REFRESH,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct H5VL_object_specific_args_t {
        pub op_type: H5VL_object_specific_t,
        pub args: H5VL_object_specific_args_t_union,
    }

    impl_debug_args!(
        H5VL_object_specific_args_t,
        H5VL_object_specific_t,
        H5VL_object_specific_args_t_union,
        {
            H5VL_OBJECT_CHANGE_REF_COUNT => |args| unsafe { args.change_rc },
            H5VL_OBJECT_EXISTS => |args| unsafe { args.exists },
            H5VL_OBJECT_LOOKUP => |args| unsafe { args.lookup },
            H5VL_OBJECT_VISIT => |args| unsafe { args.visit },
            H5VL_OBJECT_FLUSH => |args| unsafe { args.flush },
            H5VL_OBJECT_REFRESH => |args| unsafe { args.refresh },
        }
    );

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_object_get_t {
        H5VL_OBJECT_GET_FILE,
        H5VL_OBJECT_GET_NAME,
        H5VL_OBJECT_GET_TYPE,
        H5VL_OBJECT_GET_INFO,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_object_get_args_t_union_get_file {
        pub file: *mut *mut c_void,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_object_get_args_t_union_get_name {
        pub buf_size: size_t,
        pub buf: *mut c_char,
        pub name_len: *mut size_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_object_get_args_t_union_get_type {
        pub obj_type: *mut H5O_type_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_object_get_args_t_union_get_info {
        pub fields: c_uint,
        pub oinfo: *mut H5O_info2_t,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union H5VL_object_get_args_t_union {
        pub get_file: ManuallyDrop<H5VL_object_get_args_t_union_get_file>,
        pub get_name: ManuallyDrop<H5VL_object_get_args_t_union_get_name>,
        pub get_type: ManuallyDrop<H5VL_object_get_args_t_union_get_type>,
        pub get_info: ManuallyDrop<H5VL_object_get_args_t_union_get_info>,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct H5VL_object_get_args_t {
        pub op_type: H5VL_object_get_t,
        pub args: H5VL_object_get_args_t_union,
    }

    impl_debug_args!(
        H5VL_object_get_args_t,
        H5VL_object_get_t,
        H5VL_object_get_args_t_union,
        {
            H5VL_OBJECT_GET_FILE => |args| unsafe { args.get_file },
            H5VL_OBJECT_GET_NAME => |args| unsafe { args.get_name },
            H5VL_OBJECT_GET_TYPE => |args| unsafe { args.get_type },
            H5VL_OBJECT_GET_INFO => |args| unsafe { args.get_info },
        }
    );

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_object_class_t {
        pub open: Option<
            extern "C" fn(
                obj: *mut c_void,
                loc_params: *const H5VL_loc_params_t,
                opened_type: *mut H5I_type_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> *mut c_void,
        >,
        pub copy: Option<
            extern "C" fn(
                src_obj: *mut c_void,
                loc_params1: *const H5VL_loc_params_t,
                src_name: *const c_char,
                dest_obj: *mut c_void,
                loc_params2: *const H5VL_loc_params_t,
                dst_name: *const c_char,
                ocpypl_id: hid_t,
                lcpl_id: hid_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub get: Option<
            extern "C" fn(
                obj: *mut c_void,
                loc_params: *const H5VL_loc_params_t,
                args: *mut H5VL_object_get_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub specific: Option<
            extern "C" fn(
                obj: *mut c_void,
                loc_params: *const H5VL_loc_params_t,
                args: *mut H5VL_object_specific_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
        pub optional: Option<
            extern "C" fn(
                obj: *mut c_void,
                loc_params: *const H5VL_loc_params_t,
                args: *mut H5VL_optional_args_t,
                dxpl_id: hid_t,
                req: *mut *mut c_void,
            ) -> herr_t,
        >,
    }
    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_get_conn_lvl_t {
        H5VL_GET_CONN_LVL_CURR,
        H5VL_GET_CONN_LVL_TERM,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_introspect_class_t {
        pub get_conn_cls: Option<
            extern "C" fn(
                obj: *mut c_void,
                lvl: H5VL_get_conn_lvl_t,
                conn_cls: *const *const H5VL_class_t,
            ) -> herr_t,
        >,
        pub get_cap_flags:
            Option<extern "C" fn(info: *const c_void, cap_flags: *mut c_uint) -> herr_t>,
        pub opt_query: Option<
            extern "C" fn(
                obj: *mut c_void,
                cls: H5VL_subclass_t,
                opt_type: c_int,
                flags: *mut u64,
            ) -> herr_t,
        >,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_request_status_t {
        H5VL_REQUEST_STATUS_IN_PROGRESS,
        H5VL_REQUEST_STATUS_SUCCEED,
        H5VL_REQUEST_STATUS_FAIL,
        H5VL_REQUEST_STATUS_CANT_CANCEL,
        H5VL_REQUEST_STATUS_CANCELED,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_request_specific_t {
        H5VL_REQUEST_GET_ERR_STACK,
        H5VL_REQUEST_GET_EXEC_TIME,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_request_specific_args_t_union_get_err_stack {
        pub err_stack_id: hid_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_request_specific_args_t_union_get_exec_time {
        pub exec_ts: *mut u64,
        pub exec_time: *mut u64,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union H5VL_request_specific_args_t_union {
        pub get_err_stack: ManuallyDrop<H5VL_request_specific_args_t_union_get_err_stack>,
        pub get_exec_time: ManuallyDrop<H5VL_request_specific_args_t_union_get_exec_time>,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct H5VL_request_specific_args_t {
        pub op_type: H5VL_request_specific_t,
        pub args: H5VL_request_specific_args_t_union,
    }

    impl_debug_args!(
        H5VL_request_specific_args_t,
        H5VL_request_specific_t,
        H5VL_request_specific_args_t_union,
        {
            H5VL_REQUEST_GET_ERR_STACK => |args| unsafe { args.get_err_stack },
            H5VL_REQUEST_GET_EXEC_TIME => |args| unsafe { args.get_exec_time },
        }
    );

    pub type H5VL_request_notify_t =
        Option<extern "C" fn(ctx: *mut c_void, status: H5VL_request_status_t) -> herr_t>;

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_request_class_t {
        pub wait: Option<
            extern "C" fn(
                req: *mut c_void,
                timeout: u64,
                status: *mut H5VL_request_status_t,
            ) -> herr_t,
        >,
        pub notify: Option<
            extern "C" fn(req: *mut c_void, cb: H5VL_request_notify_t, ctx: *mut c_void) -> herr_t,
        >,
        pub cancel:
            Option<extern "C" fn(req: *mut c_void, status: *mut H5VL_request_status_t) -> herr_t>,
        pub specific: Option<
            extern "C" fn(req: *mut c_void, args: *mut H5VL_request_specific_args_t) -> herr_t,
        >,
        pub optional:
            Option<extern "C" fn(req: *mut c_void, args: *mut H5VL_optional_args_t) -> herr_t>,
        pub free: Option<extern "C" fn(req: *mut c_void) -> herr_t>,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_blob_specific_t {
        H5VL_BLOB_DELETE,
        H5VL_BLOB_ISNULL,
        H5VL_BLOB_SETNULL,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_blob_specific_args_t_union_is_null {
        pub isnull: *mut hbool_t,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union H5VL_blob_specific_args_t_union {
        pub is_null: ManuallyDrop<H5VL_blob_specific_args_t_union_is_null>,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct H5VL_blob_specific_args_t {
        pub op_type: H5VL_blob_specific_t,
        pub args: H5VL_blob_specific_args_t_union,
    }

    impl_debug_args!(
        H5VL_blob_specific_args_t,
        H5VL_blob_specific_t,
        H5VL_blob_specific_args_t_union,
        {
            H5VL_BLOB_ISNULL => |args| unsafe { args.is_null },
        }
    );

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_blob_class_t {
        pub put: Option<
            extern "C" fn(
                obj: *mut c_void,
                buf: *const c_void,
                size: size_t,
                blob_id: *mut c_void,
                ctx: *mut c_void,
            ) -> herr_t,
        >,
        pub get: Option<
            extern "C" fn(
                obj: *mut c_void,
                blob_id: *const c_void,
                buf: *mut c_void,
                size: size_t,
                ctx: *mut c_void,
            ) -> herr_t,
        >,
        pub specific: Option<
            extern "C" fn(
                obj: *mut c_void,
                blob_id: *mut c_void,
                args: *mut H5VL_blob_specific_args_t,
            ) -> herr_t,
        >,
        pub optional: Option<
            extern "C" fn(
                obj: *mut c_void,
                blob_id: *mut c_void,
                args: *mut H5VL_optional_args_t,
            ) -> herr_t,
        >,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_token_class_t {
        pub cmp: Option<
            extern "C" fn(
                obj: *mut c_void,
                token1: *const H5O_token_t,
                token2: *const H5O_token_t,
                cmp_value: *mut c_int,
            ) -> herr_t,
        >,
        pub to_str: Option<
            extern "C" fn(
                obj: *mut c_void,
                obj_type: H5I_type_t,
                token: *const H5O_token_t,
                token_str: *mut *mut c_char,
            ) -> herr_t,
        >,
        pub from_str: Option<
            extern "C" fn(
                obj: *mut c_void,
                obj_type: H5I_type_t,
                token_str: *const c_char,
                token: *mut H5O_token_t,
            ) -> herr_t,
        >,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum H5VL_loc_type_t {
        H5VL_OBJECT_BY_SELF,
        H5VL_OBJECT_BY_NAME,
        H5VL_OBJECT_BY_IDX,
        H5VL_OBJECT_BY_TOKEN,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_loc_by_name_t {
        pub name: *const c_char,
        pub lapl_id: hid_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_loc_by_idx_t {
        pub name: *const c_char,
        pub idx_type: H5_index_t,
        pub order: H5_iter_order_t,
        pub n: hsize_t,
        pub lapl_id: hid_t,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_loc_by_token_t {
        pub token: *mut H5O_token_t,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub union H5VL_loc_params_t_union {
        pub loc_by_token: ManuallyDrop<H5VL_loc_by_token_t>,
        pub loc_by_name: ManuallyDrop<H5VL_loc_by_name_t>,
        pub loc_by_idx: ManuallyDrop<H5VL_loc_by_idx_t>,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct H5VL_loc_params_t {
        pub obj_type: H5I_type_t,
        pub type_: H5VL_loc_type_t,
        pub loc_data: H5VL_loc_params_t_union,
    }

    impl Debug for H5VL_loc_params_t {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut s = f.debug_struct("H5VL_lov_params_t");
            s.field("obj_type", &self.obj_type).field("type", &self.type_);
            unsafe {
                match self.type_ {
                    H5VL_loc_type_t::H5VL_OBJECT_BY_SELF => {}
                    H5VL_loc_type_t::H5VL_OBJECT_BY_NAME => {
                        s.field("loc_data", &self.loc_data.loc_by_name);
                    }
                    H5VL_loc_type_t::H5VL_OBJECT_BY_IDX => {
                        s.field("loc_data", &self.loc_data.loc_by_idx);
                    }
                    H5VL_loc_type_t::H5VL_OBJECT_BY_TOKEN => {
                        s.field("loc_data", &self.loc_data.loc_by_token);
                    }
                }
            };
            s.finish()
        }
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct H5VL_optional_args_t {
        pub op_type: c_int,
        pub args: *mut c_void,
    }

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
        pub fn H5VLunregister_opt_operation(
            subcls: H5VL_subclass_t, op_name: *const c_char,
        ) -> herr_t;
    }

    extern "C" {
        pub fn H5VLfinish_lib_state() -> herr_t;
        pub fn H5VLintrospect_get_cap_flags(
            info: *const c_void, connector_id: hid_t, cap_flags: *mut c_uint,
        ) -> herr_t;
        pub fn H5VLstart_lib_state() -> herr_t;
    }

    extern "C" {
        pub fn H5VLobject_is_native(obj_id: hid_t, is_native: *mut hbool_t) -> herr_t;
    }
}
