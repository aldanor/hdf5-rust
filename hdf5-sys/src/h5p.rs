//! Creating and manipulating property lists to control HDF5 library behaviour
use crate::internal_prelude::*;

use crate::h5ac::H5AC_cache_config_t;
use crate::h5d::{H5D_alloc_time_t, H5D_fill_time_t, H5D_fill_value_t, H5D_layout_t};
use crate::h5f::{H5F_close_degree_t, H5F_libver_t};
use crate::h5fd::H5FD_mem_t;
use crate::h5l::H5L_elink_traverse_t;
use crate::h5mm::{H5MM_allocate_t, H5MM_free_t};
use crate::h5t::{H5T_conv_except_func_t, H5T_cset_t};
use crate::h5z::{H5Z_EDC_t, H5Z_SO_scale_type_t, H5Z_filter_func_t, H5Z_filter_t};

#[cfg(feature = "1.8.9")]
use crate::h5fd::H5FD_file_image_callbacks_t;
#[cfg(feature = "1.8.9")]
use crate::h5o::H5O_mcdt_search_cb_t;
#[cfg(feature = "1.10.1")]
use crate::{h5ac::H5AC_cache_image_config_t, h5f::H5F_fspace_strategy_t};

#[cfg(feature = "1.14.0")]
use crate::{h5fd::H5FD_class_value_t, h5s::H5S_seloper_t};

pub const H5P_CRT_ORDER_TRACKED: c_uint = 0x0001;
pub const H5P_CRT_ORDER_INDEXED: c_uint = 0x0002;

pub const H5P_DEFAULT: hid_t = 0;

pub type H5P_cls_create_func_t =
    Option<extern "C" fn(prop_id: hid_t, create_data: *mut c_void) -> herr_t>;
pub type H5P_cls_copy_func_t =
    Option<extern "C" fn(new_prop_id: hid_t, old_prop_id: hid_t, copy_data: *mut c_void) -> herr_t>;
pub type H5P_cls_close_func_t =
    Option<extern "C" fn(prop_id: hid_t, close_data: *mut c_void) -> herr_t>;
pub type H5P_prp_cb1_t =
    Option<extern "C" fn(name: *const c_char, size: size_t, value: *mut c_void) -> herr_t>;
pub type H5P_prp_cb2_t = Option<
    extern "C" fn(prop_id: hid_t, name: *const c_char, size: size_t, value: *mut c_void) -> herr_t,
>;
pub type H5P_prp_create_func_t = H5P_prp_cb1_t;
pub type H5P_prp_set_func_t = H5P_prp_cb2_t;
pub type H5P_prp_get_func_t = H5P_prp_cb2_t;
pub type H5P_prp_delete_func_t = H5P_prp_cb2_t;
pub type H5P_prp_copy_func_t = H5P_prp_cb1_t;
pub type H5P_prp_compare_func_t =
    Option<extern "C" fn(value1: *const c_void, value2: *const c_void, size: size_t) -> c_int>;
pub type H5P_prp_close_func_t = H5P_prp_cb1_t;
pub type H5P_iterate_t =
    Option<extern "C" fn(id: hid_t, name: *const c_char, iter_data: *mut c_void) -> herr_t>;

pub use self::globals::*;

#[cfg(all(not(feature = "1.8.14"), not(all(target_env = "msvc", not(feature = "static")))))]
mod globals {
    pub use crate::h5i::hid_t as id_t;

    // Property list classes
    extern_static!(H5P_CLS_ROOT, H5P_CLS_ROOT_g);
    extern_static!(H5P_CLS_OBJECT_CREATE, H5P_CLS_OBJECT_CREATE_g);
    extern_static!(H5P_CLS_FILE_CREATE, H5P_CLS_FILE_CREATE_g);
    extern_static!(H5P_CLS_FILE_ACCESS, H5P_CLS_FILE_ACCESS_g);
    extern_static!(H5P_CLS_DATASET_CREATE, H5P_CLS_DATASET_CREATE_g);
    extern_static!(H5P_CLS_DATASET_ACCESS, H5P_CLS_DATASET_ACCESS_g);
    extern_static!(H5P_CLS_DATASET_XFER, H5P_CLS_DATASET_XFER_g);
    extern_static!(H5P_CLS_FILE_MOUNT, H5P_CLS_FILE_MOUNT_g);
    extern_static!(H5P_CLS_GROUP_CREATE, H5P_CLS_GROUP_CREATE_g);
    extern_static!(H5P_CLS_GROUP_ACCESS, H5P_CLS_GROUP_ACCESS_g);
    extern_static!(H5P_CLS_DATATYPE_CREATE, H5P_CLS_DATATYPE_CREATE_g);
    extern_static!(H5P_CLS_DATATYPE_ACCESS, H5P_CLS_DATATYPE_ACCESS_g);
    extern_static!(H5P_CLS_STRING_CREATE, H5P_CLS_STRING_CREATE_g);
    extern_static!(H5P_CLS_ATTRIBUTE_CREATE, H5P_CLS_ATTRIBUTE_CREATE_g);
    extern_static!(H5P_CLS_OBJECT_COPY, H5P_CLS_OBJECT_COPY_g);
    extern_static!(H5P_CLS_LINK_CREATE, H5P_CLS_LINK_CREATE_g);
    extern_static!(H5P_CLS_LINK_ACCESS, H5P_CLS_LINK_ACCESS_g);

    // Default property lists
    extern_static!(H5P_LST_FILE_CREATE, H5P_LST_FILE_CREATE_g);
    extern_static!(H5P_LST_FILE_ACCESS, H5P_LST_FILE_ACCESS_g);
    extern_static!(H5P_LST_DATASET_CREATE, H5P_LST_DATASET_CREATE_g);
    extern_static!(H5P_LST_DATASET_ACCESS, H5P_LST_DATASET_ACCESS_g);
    extern_static!(H5P_LST_DATASET_XFER, H5P_LST_DATASET_XFER_g);
    extern_static!(H5P_LST_FILE_MOUNT, H5P_LST_FILE_MOUNT_g);
    extern_static!(H5P_LST_GROUP_CREATE, H5P_LST_GROUP_CREATE_g);
    extern_static!(H5P_LST_GROUP_ACCESS, H5P_LST_GROUP_ACCESS_g);
    extern_static!(H5P_LST_DATATYPE_CREATE, H5P_LST_DATATYPE_CREATE_g);
    extern_static!(H5P_LST_DATATYPE_ACCESS, H5P_LST_DATATYPE_ACCESS_g);
    extern_static!(H5P_LST_ATTRIBUTE_CREATE, H5P_LST_ATTRIBUTE_CREATE_g);
    extern_static!(H5P_LST_OBJECT_COPY, H5P_LST_OBJECT_COPY_g);
    extern_static!(H5P_LST_LINK_CREATE, H5P_LST_LINK_CREATE_g);
    extern_static!(H5P_LST_LINK_ACCESS, H5P_LST_LINK_ACCESS_g);
}

#[cfg(all(feature = "1.8.14", not(all(target_env = "msvc", not(feature = "static")))))]
mod globals {
    pub use crate::h5i::hid_t as id_t;

    // Property list classes
    extern_static!(H5P_CLS_ROOT, H5P_CLS_ROOT_ID_g);
    extern_static!(H5P_CLS_OBJECT_CREATE, H5P_CLS_OBJECT_CREATE_ID_g);
    extern_static!(H5P_CLS_FILE_CREATE, H5P_CLS_FILE_CREATE_ID_g);
    extern_static!(H5P_CLS_FILE_ACCESS, H5P_CLS_FILE_ACCESS_ID_g);
    extern_static!(H5P_CLS_DATASET_CREATE, H5P_CLS_DATASET_CREATE_ID_g);
    extern_static!(H5P_CLS_DATASET_ACCESS, H5P_CLS_DATASET_ACCESS_ID_g);
    extern_static!(H5P_CLS_DATASET_XFER, H5P_CLS_DATASET_XFER_ID_g);
    extern_static!(H5P_CLS_FILE_MOUNT, H5P_CLS_FILE_MOUNT_ID_g);
    extern_static!(H5P_CLS_GROUP_CREATE, H5P_CLS_GROUP_CREATE_ID_g);
    extern_static!(H5P_CLS_GROUP_ACCESS, H5P_CLS_GROUP_ACCESS_ID_g);
    extern_static!(H5P_CLS_DATATYPE_CREATE, H5P_CLS_DATATYPE_CREATE_ID_g);
    extern_static!(H5P_CLS_DATATYPE_ACCESS, H5P_CLS_DATATYPE_ACCESS_ID_g);
    extern_static!(H5P_CLS_STRING_CREATE, H5P_CLS_STRING_CREATE_ID_g);
    extern_static!(H5P_CLS_ATTRIBUTE_CREATE, H5P_CLS_ATTRIBUTE_CREATE_ID_g);
    extern_static!(H5P_CLS_OBJECT_COPY, H5P_CLS_OBJECT_COPY_ID_g);
    extern_static!(H5P_CLS_LINK_CREATE, H5P_CLS_LINK_CREATE_ID_g);
    extern_static!(H5P_CLS_LINK_ACCESS, H5P_CLS_LINK_ACCESS_ID_g);

    // Default property lists
    extern_static!(H5P_LST_FILE_CREATE, H5P_LST_FILE_CREATE_ID_g);
    extern_static!(H5P_LST_FILE_ACCESS, H5P_LST_FILE_ACCESS_ID_g);
    extern_static!(H5P_LST_DATASET_CREATE, H5P_LST_DATASET_CREATE_ID_g);
    extern_static!(H5P_LST_DATASET_ACCESS, H5P_LST_DATASET_ACCESS_ID_g);
    extern_static!(H5P_LST_DATASET_XFER, H5P_LST_DATASET_XFER_ID_g);
    extern_static!(H5P_LST_FILE_MOUNT, H5P_LST_FILE_MOUNT_ID_g);
    extern_static!(H5P_LST_GROUP_CREATE, H5P_LST_GROUP_CREATE_ID_g);
    extern_static!(H5P_LST_GROUP_ACCESS, H5P_LST_GROUP_ACCESS_ID_g);
    extern_static!(H5P_LST_DATATYPE_CREATE, H5P_LST_DATATYPE_CREATE_ID_g);
    extern_static!(H5P_LST_DATATYPE_ACCESS, H5P_LST_DATATYPE_ACCESS_ID_g);
    extern_static!(H5P_LST_ATTRIBUTE_CREATE, H5P_LST_ATTRIBUTE_CREATE_ID_g);
    extern_static!(H5P_LST_OBJECT_COPY, H5P_LST_OBJECT_COPY_ID_g);
    extern_static!(H5P_LST_LINK_CREATE, H5P_LST_LINK_CREATE_ID_g);
    extern_static!(H5P_LST_LINK_ACCESS, H5P_LST_LINK_ACCESS_ID_g);

    #[cfg(feature = "1.12.0")]
    #[allow(clippy::module_inception)]
    pub mod globals {
        use super::*;
        extern_static!(H5P_MAP_CREATE, H5P_CLS_MAP_CREATE_ID_g);
        extern_static!(H5P_MAP_ACCESS, H5P_CLS_MAP_ACCESS_ID_g);
        extern_static!(H5P_VOL_INITIALIZE, H5P_CLS_VOL_INITIALIZE_ID_g);
        extern_static!(H5P_REFERENCE_ACCESS, H5P_CLS_REFERENCE_ACCESS_ID_g);
        extern_static!(H5P_MAP_CREATE_DEFAULT, H5P_LST_MAP_CREATE_ID_g);
        extern_static!(H5P_MAP_ACCESS_DEFAULT, H5P_LST_MAP_ACCESS_ID_g);
        extern_static!(H5P_VOL_INITIALIZE_DEFAULT, H5P_LST_VOL_INITIALIZE_ID_g);
        extern_static!(H5P_REFERENCE_ACCESS_DEFAULT, H5P_LST_REFERENCE_ACCESS_ID_g);
    }
    #[cfg(feature = "1.12.0")]
    pub use globals::*;
}

#[cfg(all(not(feature = "1.8.14"), all(target_env = "msvc", not(feature = "static"))))]
mod globals {
    // dllimport hack
    pub type id_t = usize;

    // Property list classes
    extern_static!(H5P_CLS_ROOT, __imp_H5P_CLS_ROOT_g);
    extern_static!(H5P_CLS_OBJECT_CREATE, __imp_H5P_CLS_OBJECT_CREATE_g);
    extern_static!(H5P_CLS_FILE_CREATE, __imp_H5P_CLS_FILE_CREATE_g);
    extern_static!(H5P_CLS_FILE_ACCESS, __imp_H5P_CLS_FILE_ACCESS_g);
    extern_static!(H5P_CLS_DATASET_CREATE, __imp_H5P_CLS_DATASET_CREATE_g);
    extern_static!(H5P_CLS_DATASET_ACCESS, __imp_H5P_CLS_DATASET_ACCESS_g);
    extern_static!(H5P_CLS_DATASET_XFER, __imp_H5P_CLS_DATASET_XFER_g);
    extern_static!(H5P_CLS_FILE_MOUNT, __imp_H5P_CLS_FILE_MOUNT_g);
    extern_static!(H5P_CLS_GROUP_CREATE, __imp_H5P_CLS_GROUP_CREATE_g);
    extern_static!(H5P_CLS_GROUP_ACCESS, __imp_H5P_CLS_GROUP_ACCESS_g);
    extern_static!(H5P_CLS_DATATYPE_CREATE, __imp_H5P_CLS_DATATYPE_CREATE_g);
    extern_static!(H5P_CLS_DATATYPE_ACCESS, __imp_H5P_CLS_DATATYPE_ACCESS_g);
    extern_static!(H5P_CLS_STRING_CREATE, __imp_H5P_CLS_STRING_CREATE_g);
    extern_static!(H5P_CLS_ATTRIBUTE_CREATE, __imp_H5P_CLS_ATTRIBUTE_CREATE_g);
    extern_static!(H5P_CLS_OBJECT_COPY, __imp_H5P_CLS_OBJECT_COPY_g);
    extern_static!(H5P_CLS_LINK_CREATE, __imp_H5P_CLS_LINK_CREATE_g);
    extern_static!(H5P_CLS_LINK_ACCESS, __imp_H5P_CLS_LINK_ACCESS_g);

    // Default property lists
    extern_static!(H5P_LST_FILE_CREATE, __imp_H5P_LST_FILE_CREATE_g);
    extern_static!(H5P_LST_FILE_ACCESS, __imp_H5P_LST_FILE_ACCESS_g);
    extern_static!(H5P_LST_DATASET_CREATE, __imp_H5P_LST_DATASET_CREATE_g);
    extern_static!(H5P_LST_DATASET_ACCESS, __imp_H5P_LST_DATASET_ACCESS_g);
    extern_static!(H5P_LST_DATASET_XFER, __imp_H5P_LST_DATASET_XFER_g);
    extern_static!(H5P_LST_FILE_MOUNT, __imp_H5P_LST_FILE_MOUNT_g);
    extern_static!(H5P_LST_GROUP_CREATE, __imp_H5P_LST_GROUP_CREATE_g);
    extern_static!(H5P_LST_GROUP_ACCESS, __imp_H5P_LST_GROUP_ACCESS_g);
    extern_static!(H5P_LST_DATATYPE_CREATE, __imp_H5P_LST_DATATYPE_CREATE_g);
    extern_static!(H5P_LST_DATATYPE_ACCESS, __imp_H5P_LST_DATATYPE_ACCESS_g);
    extern_static!(H5P_LST_ATTRIBUTE_CREATE, __imp_H5P_LST_ATTRIBUTE_CREATE_g);
    extern_static!(H5P_LST_OBJECT_COPY, __imp_H5P_LST_OBJECT_COPY_g);
    extern_static!(H5P_LST_LINK_CREATE, __imp_H5P_LST_LINK_CREATE_g);
    extern_static!(H5P_LST_LINK_ACCESS, __imp_H5P_LST_LINK_ACCESS_g);
}

#[cfg(all(feature = "1.8.14", all(target_env = "msvc", not(feature = "static"))))]
mod globals {
    // dllimport hack
    pub type id_t = usize;

    // Property list classes
    extern_static!(H5P_CLS_ROOT, __imp_H5P_CLS_ROOT_ID_g);
    extern_static!(H5P_CLS_OBJECT_CREATE, __imp_H5P_CLS_OBJECT_CREATE_ID_g);
    extern_static!(H5P_CLS_FILE_CREATE, __imp_H5P_CLS_FILE_CREATE_ID_g);
    extern_static!(H5P_CLS_FILE_ACCESS, __imp_H5P_CLS_FILE_ACCESS_ID_g);
    extern_static!(H5P_CLS_DATASET_CREATE, __imp_H5P_CLS_DATASET_CREATE_ID_g);
    extern_static!(H5P_CLS_DATASET_ACCESS, __imp_H5P_CLS_DATASET_ACCESS_ID_g);
    extern_static!(H5P_CLS_DATASET_XFER, __imp_H5P_CLS_DATASET_XFER_ID_g);
    extern_static!(H5P_CLS_FILE_MOUNT, __imp_H5P_CLS_FILE_MOUNT_ID_g);
    extern_static!(H5P_CLS_GROUP_CREATE, __imp_H5P_CLS_GROUP_CREATE_ID_g);
    extern_static!(H5P_CLS_GROUP_ACCESS, __imp_H5P_CLS_GROUP_ACCESS_ID_g);
    extern_static!(H5P_CLS_DATATYPE_CREATE, __imp_H5P_CLS_DATATYPE_CREATE_ID_g);
    extern_static!(H5P_CLS_DATATYPE_ACCESS, __imp_H5P_CLS_DATATYPE_ACCESS_ID_g);
    extern_static!(H5P_CLS_STRING_CREATE, __imp_H5P_CLS_STRING_CREATE_ID_g);
    extern_static!(H5P_CLS_ATTRIBUTE_CREATE, __imp_H5P_CLS_ATTRIBUTE_CREATE_ID_g);
    extern_static!(H5P_CLS_OBJECT_COPY, __imp_H5P_CLS_OBJECT_COPY_ID_g);
    extern_static!(H5P_CLS_LINK_CREATE, __imp_H5P_CLS_LINK_CREATE_ID_g);
    extern_static!(H5P_CLS_LINK_ACCESS, __imp_H5P_CLS_LINK_ACCESS_ID_g);

    // Default property lists
    extern_static!(H5P_LST_FILE_CREATE, __imp_H5P_LST_FILE_CREATE_ID_g);
    extern_static!(H5P_LST_FILE_ACCESS, __imp_H5P_LST_FILE_ACCESS_ID_g);
    extern_static!(H5P_LST_DATASET_CREATE, __imp_H5P_LST_DATASET_CREATE_ID_g);
    extern_static!(H5P_LST_DATASET_ACCESS, __imp_H5P_LST_DATASET_ACCESS_ID_g);
    extern_static!(H5P_LST_DATASET_XFER, __imp_H5P_LST_DATASET_XFER_ID_g);
    extern_static!(H5P_LST_FILE_MOUNT, __imp_H5P_LST_FILE_MOUNT_ID_g);
    extern_static!(H5P_LST_GROUP_CREATE, __imp_H5P_LST_GROUP_CREATE_ID_g);
    extern_static!(H5P_LST_GROUP_ACCESS, __imp_H5P_LST_GROUP_ACCESS_ID_g);
    extern_static!(H5P_LST_DATATYPE_CREATE, __imp_H5P_LST_DATATYPE_CREATE_ID_g);
    extern_static!(H5P_LST_DATATYPE_ACCESS, __imp_H5P_LST_DATATYPE_ACCESS_ID_g);
    extern_static!(H5P_LST_ATTRIBUTE_CREATE, __imp_H5P_LST_ATTRIBUTE_CREATE_ID_g);
    extern_static!(H5P_LST_OBJECT_COPY, __imp_H5P_LST_OBJECT_COPY_ID_g);
    extern_static!(H5P_LST_LINK_CREATE, __imp_H5P_LST_LINK_CREATE_ID_g);
    extern_static!(H5P_LST_LINK_ACCESS, __imp_H5P_LST_LINK_ACCESS_ID_g);

    #[cfg(feature = "1.12.0")]
    #[allow(clippy::module_inception)]
    pub mod globals {
        use super::*;
        extern_static!(H5P_MAP_CREATE, __imp_H5P_CLS_MAP_CREATE_ID_g);
        extern_static!(H5P_MAP_ACCESS, __imp_H5P_CLS_MAP_ACCESS_ID_g);
        extern_static!(H5P_VOL_INITIALIZE, __imp_H5P_CLS_VOL_INITIALIZE_ID_g);
        extern_static!(H5P_REFERENCE_ACCESS, __imp_H5P_CLS_REFERENCE_ACCESS_ID_g);
        extern_static!(H5P_MAP_CREATE_DEFAULT, __imp_H5P_LST_MAP_CREATE_ID_g);
        extern_static!(H5P_MAP_ACCESS_DEFAULT, __imp_H5P_LST_MAP_ACCESS_ID_g);
        extern_static!(H5P_VOL_INITIALIZE_DEFAULT, __imp_H5P_LST_VOL_INITIALIZE_ID_g);
        extern_static!(H5P_REFERENCE_ACCESS_DEFAULT, __imp_H5P_LST_REFERENCE_ACCESS_ID_g);
    }
    #[cfg(feature = "1.12.0")]
    pub use globals::*;
}

extern "C" {
    pub fn H5Pcreate_class(
        parent: hid_t, name: *const c_char, cls_create: H5P_cls_create_func_t,
        create_data: *mut c_void, cls_copy: H5P_cls_copy_func_t, copy_data: *mut c_void,
        cls_close: H5P_cls_close_func_t, close_data: *mut c_void,
    ) -> hid_t;
    pub fn H5Pget_class_name(pclass_id: hid_t) -> *mut c_char;
    pub fn H5Pcreate(cls_id: hid_t) -> hid_t;
    pub fn H5Pregister2(
        cls_id: hid_t, name: *const c_char, size: size_t, def_value: *mut c_void,
        prp_create: H5P_prp_create_func_t, prp_set: H5P_prp_set_func_t,
        prp_get: H5P_prp_get_func_t, prp_del: H5P_prp_delete_func_t, prp_copy: H5P_prp_copy_func_t,
        prp_cmp: H5P_prp_compare_func_t, prp_close: H5P_prp_close_func_t,
    ) -> herr_t;
    pub fn H5Pinsert2(
        plist_id: hid_t, name: *const c_char, size: size_t, value: *mut c_void,
        prp_set: H5P_prp_set_func_t, prp_get: H5P_prp_get_func_t,
        prp_delete: H5P_prp_delete_func_t, prp_copy: H5P_prp_copy_func_t,
        prp_cmp: H5P_prp_compare_func_t, prp_close: H5P_prp_close_func_t,
    ) -> herr_t;
    pub fn H5Pset(plist_id: hid_t, name: *const c_char, value: *mut c_void) -> herr_t;
    pub fn H5Pexist(plist_id: hid_t, name: *const c_char) -> htri_t;
    pub fn H5Pget_size(id: hid_t, name: *const c_char, size: *mut size_t) -> herr_t;
    pub fn H5Pget_nprops(id: hid_t, nprops: *mut size_t) -> herr_t;
    pub fn H5Pget_class(plist_id: hid_t) -> hid_t;
    pub fn H5Pget_class_parent(pclass_id: hid_t) -> hid_t;
    pub fn H5Pget(plist_id: hid_t, name: *const c_char, value: *mut c_void) -> herr_t;
    pub fn H5Pequal(id1: hid_t, id2: hid_t) -> htri_t;
    pub fn H5Pisa_class(plist_id: hid_t, pclass_id: hid_t) -> htri_t;
    pub fn H5Piterate(
        id: hid_t, idx: *mut c_int, iter_func: H5P_iterate_t, iter_data: *mut c_void,
    ) -> c_int;
    pub fn H5Pcopy_prop(dst_id: hid_t, src_id: hid_t, name: *const c_char) -> herr_t;
    pub fn H5Premove(plist_id: hid_t, name: *const c_char) -> herr_t;
    pub fn H5Punregister(pclass_id: hid_t, name: *const c_char) -> herr_t;
    pub fn H5Pclose_class(plist_id: hid_t) -> herr_t;
    pub fn H5Pclose(plist_id: hid_t) -> herr_t;
    pub fn H5Pcopy(plist_id: hid_t) -> hid_t;
    pub fn H5Pset_attr_phase_change(
        plist_id: hid_t, max_compact: c_uint, min_dense: c_uint,
    ) -> herr_t;
    pub fn H5Pget_attr_phase_change(
        plist_id: hid_t, max_compact: *mut c_uint, min_dense: *mut c_uint,
    ) -> herr_t;
    pub fn H5Pset_attr_creation_order(plist_id: hid_t, crt_order_flags: c_uint) -> herr_t;
    pub fn H5Pget_attr_creation_order(plist_id: hid_t, crt_order_flags: *mut c_uint) -> herr_t;
    pub fn H5Pset_obj_track_times(plist_id: hid_t, track_times: hbool_t) -> herr_t;
    pub fn H5Pget_obj_track_times(plist_id: hid_t, track_times: *mut hbool_t) -> herr_t;
    pub fn H5Pmodify_filter(
        plist_id: hid_t, filter: H5Z_filter_t, flags: c_uint, cd_nelmts: size_t,
        cd_values: *const c_uint,
    ) -> herr_t;
    pub fn H5Pset_filter(
        plist_id: hid_t, filter: H5Z_filter_t, flags: c_uint, cd_nelmts: size_t,
        c_values: *const c_uint,
    ) -> herr_t;
    pub fn H5Pget_nfilters(plist_id: hid_t) -> c_int;
    pub fn H5Pget_filter2(
        plist_id: hid_t, filter: c_uint, flags: *mut c_uint, cd_nelmts: *mut size_t,
        cd_values: *mut c_uint, namelen: size_t, name: *mut c_char, filter_config: *mut c_uint,
    ) -> H5Z_filter_t;
    pub fn H5Pget_filter_by_id2(
        plist_id: hid_t, id: H5Z_filter_t, flags: *mut c_uint, cd_nelmts: *mut size_t,
        cd_values: *mut c_uint, namelen: size_t, name: *mut c_char, filter_config: *mut c_uint,
    ) -> herr_t;
    pub fn H5Pall_filters_avail(plist_id: hid_t) -> htri_t;
    pub fn H5Premove_filter(plist_id: hid_t, filter: H5Z_filter_t) -> herr_t;
    pub fn H5Pset_deflate(plist_id: hid_t, aggression: c_uint) -> herr_t;
    pub fn H5Pset_fletcher32(plist_id: hid_t) -> herr_t;
    pub fn H5Pset_userblock(plist_id: hid_t, size: hsize_t) -> herr_t;
    pub fn H5Pget_userblock(plist_id: hid_t, size: *mut hsize_t) -> herr_t;
    pub fn H5Pset_sizes(plist_id: hid_t, sizeof_addr: size_t, sizeof_size: size_t) -> herr_t;
    pub fn H5Pget_sizes(
        plist_id: hid_t, sizeof_addr: *mut size_t, sizeof_size: *mut size_t,
    ) -> herr_t;
    pub fn H5Pset_sym_k(plist_id: hid_t, ik: c_uint, lk: c_uint) -> herr_t;
    pub fn H5Pget_sym_k(plist_id: hid_t, ik: *mut c_uint, lk: *mut c_uint) -> herr_t;
    pub fn H5Pset_istore_k(plist_id: hid_t, ik: c_uint) -> herr_t;
    pub fn H5Pget_istore_k(plist_id: hid_t, ik: *mut c_uint) -> herr_t;
    pub fn H5Pset_shared_mesg_nindexes(plist_id: hid_t, nindexes: c_uint) -> herr_t;
    pub fn H5Pget_shared_mesg_nindexes(plist_id: hid_t, nindexes: *mut c_uint) -> herr_t;
    pub fn H5Pset_shared_mesg_index(
        plist_id: hid_t, index_num: c_uint, mesg_type_flags: c_uint, min_mesg_size: c_uint,
    ) -> herr_t;
    pub fn H5Pget_shared_mesg_index(
        plist_id: hid_t, index_num: c_uint, mesg_type_flags: *mut c_uint,
        min_mesg_size: *mut c_uint,
    ) -> herr_t;
    pub fn H5Pset_shared_mesg_phase_change(
        plist_id: hid_t, max_list: c_uint, min_btree: c_uint,
    ) -> herr_t;
    pub fn H5Pget_shared_mesg_phase_change(
        plist_id: hid_t, max_list: *mut c_uint, min_btree: *mut c_uint,
    ) -> herr_t;
    pub fn H5Pset_alignment(fapl_id: hid_t, threshold: hsize_t, alignment: hsize_t) -> herr_t;
    pub fn H5Pget_alignment(
        fapl_id: hid_t, threshold: *mut hsize_t, alignment: *mut hsize_t,
    ) -> herr_t;
    pub fn H5Pset_driver(plist_id: hid_t, driver_id: hid_t, driver_info: *const c_void) -> herr_t;
    pub fn H5Pget_driver(plist_id: hid_t) -> hid_t;
    pub fn H5Pget_driver_info(plist_id: hid_t) -> *mut c_void;
    pub fn H5Pset_cache(
        plist_id: hid_t, mdc_nelmts: c_int, rdcc_nslots: size_t, rdcc_nbytes: size_t,
        rdcc_w0: c_double,
    ) -> herr_t;
    pub fn H5Pget_cache(
        plist_id: hid_t, mdc_nelmts: *mut c_int, rdcc_nslots: *mut size_t,
        rdcc_nbytes: *mut size_t, rdcc_w0: *mut c_double,
    ) -> herr_t;
    pub fn H5Pset_mdc_config(plist_id: hid_t, config_ptr: *const H5AC_cache_config_t) -> herr_t;
    pub fn H5Pget_mdc_config(plist_id: hid_t, config_ptr: *mut H5AC_cache_config_t) -> herr_t;
    pub fn H5Pset_gc_references(fapl_id: hid_t, gc_ref: c_uint) -> herr_t;
    pub fn H5Pget_gc_references(fapl_id: hid_t, gc_ref: *mut c_uint) -> herr_t;
    pub fn H5Pset_fclose_degree(fapl_id: hid_t, degree: H5F_close_degree_t) -> herr_t;
    pub fn H5Pget_fclose_degree(fapl_id: hid_t, degree: *mut H5F_close_degree_t) -> herr_t;
    pub fn H5Pset_meta_block_size(fapl_id: hid_t, size: hsize_t) -> herr_t;
    pub fn H5Pget_meta_block_size(fapl_id: hid_t, size: *mut hsize_t) -> herr_t;
    pub fn H5Pset_sieve_buf_size(fapl_id: hid_t, size: size_t) -> herr_t;
    pub fn H5Pget_sieve_buf_size(fapl_id: hid_t, size: *mut size_t) -> herr_t;
    pub fn H5Pset_small_data_block_size(fapl_id: hid_t, size: hsize_t) -> herr_t;
    pub fn H5Pget_small_data_block_size(fapl_id: hid_t, size: *mut hsize_t) -> herr_t;
    pub fn H5Pset_libver_bounds(plist_id: hid_t, low: H5F_libver_t, high: H5F_libver_t) -> herr_t;
    pub fn H5Pget_libver_bounds(
        plist_id: hid_t, low: *mut H5F_libver_t, high: *mut H5F_libver_t,
    ) -> herr_t;
    pub fn H5Pset_layout(plist_id: hid_t, layout: H5D_layout_t) -> herr_t;
    pub fn H5Pget_layout(plist_id: hid_t) -> H5D_layout_t;
    pub fn H5Pset_chunk(plist_id: hid_t, ndims: c_int, dim: *const hsize_t) -> herr_t;
    pub fn H5Pget_chunk(plist_id: hid_t, max_ndims: c_int, dim: *mut hsize_t) -> c_int;
    pub fn H5Pset_external(
        plist_id: hid_t, name: *const c_char, offset: off_t, size: hsize_t,
    ) -> herr_t;
    pub fn H5Pget_external_count(plist_id: hid_t) -> c_int;
    pub fn H5Pget_external(
        plist_id: hid_t, idx: c_uint, name_size: size_t, name: *mut c_char, offset: *mut off_t,
        size: *mut hsize_t,
    ) -> herr_t;
    pub fn H5Pset_szip(plist_id: hid_t, options_mask: c_uint, pixels_per_block: c_uint) -> herr_t;
    pub fn H5Pset_shuffle(plist_id: hid_t) -> herr_t;
    pub fn H5Pset_nbit(plist_id: hid_t) -> herr_t;
    pub fn H5Pset_scaleoffset(
        plist_id: hid_t, scale_type: H5Z_SO_scale_type_t, scale_factor: c_int,
    ) -> herr_t;
    pub fn H5Pset_fill_value(plist_id: hid_t, type_id: hid_t, value: *const c_void) -> herr_t;
    pub fn H5Pget_fill_value(plist_id: hid_t, type_id: hid_t, value: *mut c_void) -> herr_t;
    pub fn H5Pfill_value_defined(plist: hid_t, status: *mut H5D_fill_value_t) -> herr_t;
    pub fn H5Pset_alloc_time(plist_id: hid_t, alloc_time: H5D_alloc_time_t) -> herr_t;
    pub fn H5Pget_alloc_time(plist_id: hid_t, alloc_time: *mut H5D_alloc_time_t) -> herr_t;
    pub fn H5Pset_fill_time(plist_id: hid_t, fill_time: H5D_fill_time_t) -> herr_t;
    pub fn H5Pget_fill_time(plist_id: hid_t, fill_time: *mut H5D_fill_time_t) -> herr_t;
    pub fn H5Pset_chunk_cache(
        dapl_id: hid_t, rdcc_nslots: size_t, rdcc_nbytes: size_t, rdcc_w0: c_double,
    ) -> herr_t;
    pub fn H5Pget_chunk_cache(
        dapl_id: hid_t, rdcc_nslots: *mut size_t, rdcc_nbytes: *mut size_t, rdcc_w0: *mut c_double,
    ) -> herr_t;
    pub fn H5Pset_data_transform(plist_id: hid_t, expression: *const c_char) -> herr_t;
    pub fn H5Pget_data_transform(plist_id: hid_t, expression: *mut c_char, size: size_t)
        -> ssize_t;
    pub fn H5Pset_buffer(
        plist_id: hid_t, size: size_t, tconv: *mut c_void, bkg: *mut c_void,
    ) -> herr_t;
    pub fn H5Pget_buffer(plist_id: hid_t, tconv: *mut *mut c_void, bkg: *mut *mut c_void)
        -> size_t;
    pub fn H5Pset_edc_check(plist_id: hid_t, check: H5Z_EDC_t) -> herr_t;
    pub fn H5Pget_edc_check(plist_id: hid_t) -> H5Z_EDC_t;
    pub fn H5Pset_filter_callback(
        plist_id: hid_t, func: H5Z_filter_func_t, op_data: *mut c_void,
    ) -> herr_t;
    pub fn H5Pset_btree_ratios(
        plist_id: hid_t, left: c_double, middle: c_double, right: c_double,
    ) -> herr_t;
    pub fn H5Pget_btree_ratios(
        plist_id: hid_t, left: *mut c_double, middle: *mut c_double, right: *mut c_double,
    ) -> herr_t;
    pub fn H5Pset_vlen_mem_manager(
        plist_id: hid_t, alloc_func: H5MM_allocate_t, alloc_info: *mut c_void,
        free_func: H5MM_free_t, free_info: *mut c_void,
    ) -> herr_t;
    pub fn H5Pget_vlen_mem_manager(
        plist_id: hid_t, alloc_func: *mut H5MM_allocate_t, alloc_info: *mut *mut c_void,
        free_func: *mut H5MM_free_t, free_info: *mut *mut c_void,
    ) -> herr_t;
    pub fn H5Pset_hyper_vector_size(fapl_id: hid_t, size: size_t) -> herr_t;
    pub fn H5Pget_hyper_vector_size(fapl_id: hid_t, size: *mut size_t) -> herr_t;
    pub fn H5Pset_type_conv_cb(
        dxpl_id: hid_t, op: H5T_conv_except_func_t, operate_data: *mut c_void,
    ) -> herr_t;
    pub fn H5Pget_type_conv_cb(
        dxpl_id: hid_t, op: *mut H5T_conv_except_func_t, operate_data: *mut *mut c_void,
    ) -> herr_t;
    pub fn H5Pset_create_intermediate_group(plist_id: hid_t, crt_intmd: c_uint) -> herr_t;
    pub fn H5Pget_create_intermediate_group(plist_id: hid_t, crt_intmd: *mut c_uint) -> herr_t;
    pub fn H5Pset_local_heap_size_hint(plist_id: hid_t, size_hint: size_t) -> herr_t;
    pub fn H5Pget_local_heap_size_hint(plist_id: hid_t, size_hint: *mut size_t) -> herr_t;
    pub fn H5Pset_link_phase_change(
        plist_id: hid_t, max_compact: c_uint, min_dense: c_uint,
    ) -> herr_t;
    pub fn H5Pget_link_phase_change(
        plist_id: hid_t, max_compact: *mut c_uint, min_dense: *mut c_uint,
    ) -> herr_t;
    pub fn H5Pset_est_link_info(
        plist_id: hid_t, est_num_entries: c_uint, est_name_len: c_uint,
    ) -> herr_t;
    pub fn H5Pget_est_link_info(
        plist_id: hid_t, est_num_entries: *mut c_uint, est_name_len: *mut c_uint,
    ) -> herr_t;
    pub fn H5Pset_link_creation_order(plist_id: hid_t, crt_order_flags: c_uint) -> herr_t;
    pub fn H5Pget_link_creation_order(plist_id: hid_t, crt_order_flags: *mut c_uint) -> herr_t;
    pub fn H5Pset_char_encoding(plist_id: hid_t, encoding: H5T_cset_t) -> herr_t;
    pub fn H5Pget_char_encoding(plist_id: hid_t, encoding: *mut H5T_cset_t) -> herr_t;
    pub fn H5Pset_nlinks(plist_id: hid_t, nlinks: size_t) -> herr_t;
    pub fn H5Pget_nlinks(plist_id: hid_t, nlinks: *mut size_t) -> herr_t;
    pub fn H5Pset_elink_prefix(plist_id: hid_t, prefix: *const c_char) -> herr_t;
    pub fn H5Pget_elink_prefix(plist_id: hid_t, prefix: *mut c_char, size: size_t) -> ssize_t;
    pub fn H5Pget_elink_fapl(lapl_id: hid_t) -> hid_t;
    pub fn H5Pset_elink_fapl(lapl_id: hid_t, fapl_id: hid_t) -> herr_t;
    pub fn H5Pset_elink_acc_flags(lapl_id: hid_t, flags: c_uint) -> herr_t;
    pub fn H5Pget_elink_acc_flags(lapl_id: hid_t, flags: *mut c_uint) -> herr_t;
    pub fn H5Pset_elink_cb(
        lapl_id: hid_t, func: H5L_elink_traverse_t, op_data: *mut c_void,
    ) -> herr_t;
    pub fn H5Pget_elink_cb(
        lapl_id: hid_t, func: *mut H5L_elink_traverse_t, op_data: *mut *mut c_void,
    ) -> herr_t;
    pub fn H5Pset_copy_object(plist_id: hid_t, crt_intmd: c_uint) -> herr_t;
    pub fn H5Pget_copy_object(plist_id: hid_t, crt_intmd: *mut c_uint) -> herr_t;
    #[cfg_attr(
        feature = "1.10.0",
        deprecated(note = "deprecated in HDF5 1.10.0, use H5Fget_info2()")
    )]
    pub fn H5Pget_version(
        plist_id: hid_t, boot: *mut c_uint, freelist: *mut c_uint, stab: *mut c_uint,
        shhdr: *mut c_uint,
    ) -> herr_t;
}

// drivers
extern "C" {
    // sec2
    pub fn H5Pset_fapl_sec2(fapl_id: hid_t) -> herr_t;

    // core
    pub fn H5Pset_fapl_core(fapl_id: hid_t, increment: size_t, backing_store: hbool_t) -> herr_t;
    pub fn H5Pget_fapl_core(
        fapl_id: hid_t, increment: *mut size_t, backing_store: *mut hbool_t,
    ) -> herr_t;

    // stdio
    pub fn H5Pset_fapl_stdio(fapl_id: hid_t) -> herr_t;

    // family
    pub fn H5Pset_fapl_family(fapl_id: hid_t, memb_size: hsize_t, memb_fapl_id: hid_t) -> herr_t;
    pub fn H5Pget_fapl_family(
        fapl_id: hid_t, memb_size: *mut hsize_t, memb_fapl_id: *mut hid_t,
    ) -> herr_t;
    pub fn H5Pset_family_offset(fapl_id: hid_t, offset: hsize_t) -> herr_t;
    pub fn H5Pget_family_offset(fapl_id: hid_t, offset: *mut hsize_t) -> herr_t;

    // multi/split
    pub fn H5Pset_fapl_multi(
        fapl_id: hid_t, memb_map: *const H5FD_mem_t, memb_fapl: *const hid_t,
        memb_name: *const *const c_char, memb_addr: *const haddr_t, relax: hbool_t,
    ) -> herr_t;
    pub fn H5Pget_fapl_multi(
        fapl_id: hid_t, memb_map: *mut H5FD_mem_t, memb_fapl: *mut hid_t,
        memb_name: *mut *const c_char, memb_addr: *mut haddr_t, relax: *mut hbool_t,
    ) -> herr_t;
    pub fn H5Pset_multi_type(fapl_id: hid_t, type_: H5FD_mem_t) -> herr_t;
    pub fn H5Pget_multi_type(fapl_id: hid_t, type_: *mut H5FD_mem_t) -> herr_t;
    pub fn H5Pset_fapl_split(
        fapl_id: hid_t, meta_ext: *const c_char, meta_plist_id: hid_t, raw_ext: *const c_char,
        raw_plist_id: hid_t,
    ) -> herr_t;

    // log
    pub fn H5Pset_fapl_log(
        fapl_id: hid_t, logfile: *const c_char, flags: c_ulonglong, buf_size: size_t,
    ) -> herr_t;

    // mpi-io
    #[cfg(feature = "mpio")]
    pub fn H5Pset_fapl_mpio(
        fapl_id: hid_t, comm: mpi_sys::MPI_Comm, info: mpi_sys::MPI_Info,
    ) -> herr_t;
    #[cfg(feature = "mpio")]
    pub fn H5Pget_fapl_mpio(
        fapl_id: hid_t, comm: *mut mpi_sys::MPI_Comm, info: *mut mpi_sys::MPI_Info,
    ) -> herr_t;

    // direct
    #[cfg(feature = "have-direct")]
    pub fn H5Pset_fapl_direct(
        fapl_id: hid_t, alignment: size_t, block_size: size_t, cbuf_size: size_t,
    ) -> herr_t;
    #[cfg(feature = "have-direct")]
    pub fn H5Pget_fapl_direct(
        fapl_id: hid_t, alignment: *mut size_t, block_size: *mut size_t, cbuf_size: *mut size_t,
    ) -> herr_t;
}

#[cfg(feature = "have-parallel")]
mod mpio {
    use crate::internal_prelude::*;

    pub const H5D_ONE_LINK_CHUNK_IO_THRESHOLD: c_uint = 0;
    pub const H5D_MULTI_CHUNK_IO_COL_THRESHOLD: c_uint = 60;

    #[repr(C)]
    #[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
    pub enum H5FD_mpio_xfer_t {
        H5FD_MPIO_INDEPENDENT = 0,
        H5FD_MPIO_COLLECTIVE = 1,
    }

    #[repr(C)]
    #[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
    pub enum H5FD_mpio_chunk_opt_t {
        H5FD_MPIO_CHUNK_DEFAULT = 0,
        H5FD_MPIO_CHUNK_ONE_IO = 1,
        H5FD_MPIO_CHUNK_MULTI_IO = 2,
    }

    #[repr(C)]
    #[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
    pub enum H5FD_mpio_collective_opt_t {
        H5FD_MPIO_COLLECTIVE_IO = 0,
        H5FD_MPIO_INDIVIDUAL_IO = 1,
    }

    extern "C" {
        pub fn H5Pset_dxpl_mpio(dxpl_id: hid_t, xfer_mode: H5FD_mpio_xfer_t) -> herr_t;
        pub fn H5Pget_dxpl_mpio(dxpl_id: hid_t, xfer_mode: *mut H5FD_mpio_xfer_t) -> herr_t;
        pub fn H5Pset_dxpl_mpio_collective_opt(
            dxpl_id: hid_t, opt_mode: H5FD_mpio_collective_opt_t,
        ) -> herr_t;
        pub fn H5Pset_dxpl_mpio_chunk_opt(
            dxpl_id: hid_t, opt_mode: H5FD_mpio_chunk_opt_t,
        ) -> herr_t;
        pub fn H5Pset_dxpl_mpio_chunk_opt_num(dxpl_id: hid_t, num_chunk_per_proc: c_uint)
            -> herr_t;
        pub fn H5Pset_dxpl_mpio_chunk_opt_ratio(
            dxpl_id: hid_t, percent_num_proc_per_chunk: c_uint,
        ) -> herr_t;
    }
}

#[cfg(feature = "have-parallel")]
pub use self::mpio::*;

#[cfg(target_os = "windows")]
extern "C" {
    pub fn H5Pset_fapl_windows(fapl_id: hid_t) -> herr_t;
}

#[cfg(feature = "1.8.7")]
extern "C" {
    pub fn H5Pset_elink_file_cache_size(plist_id: hid_t, efc_size: c_uint) -> herr_t;
    pub fn H5Pget_elink_file_cache_size(plist_id: hid_t, efc_size: *mut c_uint) -> herr_t;
}

#[cfg(feature = "1.8.9")]
extern "C" {
    pub fn H5Pset_file_image(fapl_id: hid_t, buf_ptr: *mut c_void, buf_len: size_t) -> herr_t;
    pub fn H5Pget_file_image(
        fapl_id: hid_t, buf_ptr_ptr: *mut *mut c_void, buf_len_ptr: *mut size_t,
    ) -> herr_t;
    pub fn H5Pset_file_image_callbacks(
        fapl_id: hid_t, callbacks_ptr: *mut H5FD_file_image_callbacks_t,
    ) -> herr_t;
    pub fn H5Pget_file_image_callbacks(
        fapl_id: hid_t, callbacks_ptr: *mut H5FD_file_image_callbacks_t,
    ) -> herr_t;
    pub fn H5Pset_mcdt_search_cb(
        plist_id: hid_t, func: H5O_mcdt_search_cb_t, op_data: *mut c_void,
    ) -> herr_t;
    pub fn H5Pget_mcdt_search_cb(
        plist_id: hid_t, func: *mut H5O_mcdt_search_cb_t, op_data: *mut *mut c_void,
    ) -> herr_t;
    pub fn H5Padd_merge_committed_dtype_path(plist_id: hid_t, path: *const c_char) -> herr_t;
    pub fn H5Pfree_merge_committed_dtype_paths(plist_id: hid_t) -> herr_t;
}

#[cfg(feature = "1.8.13")]
extern "C" {
    pub fn H5Pset_core_write_tracking(
        fapl_id: hid_t, is_enabled: hbool_t, page_size: size_t,
    ) -> herr_t;
    pub fn H5Pget_core_write_tracking(
        fapl_id: hid_t, is_enabled: *mut hbool_t, page_size: *mut size_t,
    ) -> herr_t;
}

#[cfg(feature = "1.8.17")]
extern "C" {
    pub fn H5Pset_efile_prefix(dapl_id: hid_t, prefix: *const c_char) -> herr_t;
    pub fn H5Pget_efile_prefix(dapl_id: hid_t, prefix: *const c_char, size: size_t) -> ssize_t;
}

#[cfg(feature = "1.10.0")]
use crate::{
    h5d::{H5D_append_cb_t, H5D_vds_view_t},
    h5f::{H5F_file_space_type_t, H5F_flush_cb_t},
};

#[cfg(feature = "1.10.0")]
extern "C" {
    pub fn H5Pset_append_flush(
        plist_id: hid_t, ndims: c_uint, boundary: *const hsize_t, func: H5D_append_cb_t,
        udata: *mut c_void,
    ) -> herr_t;
    pub fn H5Pget_append_flush(
        plist_id: hid_t, dims: c_uint, boundary: *mut hsize_t, func: *mut H5D_append_cb_t,
        udata: *mut *mut c_void,
    ) -> herr_t;
    pub fn H5Pset_object_flush_cb(
        plist_id: hid_t, func: H5F_flush_cb_t, udata: *mut c_void,
    ) -> herr_t;
    pub fn H5Pget_object_flush_cb(
        plist_id: hid_t, func: *mut H5F_flush_cb_t, udata: *mut *mut c_void,
    ) -> herr_t;
    pub fn H5Pget_metadata_read_attempts(plist_id: hid_t, attempts: *mut c_uint) -> herr_t;
    pub fn H5Pset_metadata_read_attempts(plist_id: hid_t, attempts: c_uint) -> herr_t;
    pub fn H5Pset_mdc_log_options(
        plist_id: hid_t, is_enabled: hbool_t, location: *const c_char, start_on_access: hbool_t,
    ) -> herr_t;
    pub fn H5Pget_mdc_log_options(
        plist_id: hid_t, is_enabled: *mut hbool_t, location: *mut c_char,
        location_size: *mut size_t, start_on_access: *mut hbool_t,
    ) -> herr_t;
    pub fn H5Fget_mdc_logging_status(
        file_id: hid_t, is_enabled: *mut hbool_t, is_currently_logging: *mut hbool_t,
    ) -> herr_t;
    pub fn H5Pset_virtual(
        dcpl_id: hid_t, vspace_id: hid_t, src_file_name: *const c_char,
        src_dset_name: *const c_char, src_space_id: hid_t,
    ) -> herr_t;
    pub fn H5Pget_virtual_count(dcpl_id: hid_t, count: *mut size_t) -> herr_t;
    pub fn H5Pget_virtual_vspace(dcpl_id: hid_t, index: size_t) -> hid_t;
    pub fn H5Pget_virtual_srcspace(dcpl_id: hid_t, index: size_t) -> hid_t;
    pub fn H5Pget_virtual_dsetname(
        dcpl_id: hid_t, index: size_t, name: *mut c_char, size: size_t,
    ) -> ssize_t;
    pub fn H5Pget_virtual_filename(
        dcpl_id: hid_t, index: size_t, name: *mut c_char, size: size_t,
    ) -> ssize_t;
    pub fn H5Pget_virtual_printf_gap(plist_id: hid_t, gap_size: *mut hsize_t) -> herr_t;
    pub fn H5Pset_virtual_printf_gap(plist_id: hid_t, gap_size: hsize_t) -> herr_t;
    pub fn H5Pget_virtual_view(plist_id: hid_t, view: *mut H5D_vds_view_t) -> herr_t;
    pub fn H5Pset_virtual_view(plist_id: hid_t, view: H5D_vds_view_t) -> herr_t;
    pub fn H5Pget_chunk_opts(plist_id: hid_t, opts: *mut c_uint) -> herr_t;
    pub fn H5Pset_chunk_opts(plist_id: hid_t, opts: c_uint) -> herr_t;
    #[cfg_attr(
        feature = "1.12.0",
        deprecated(note = "deprecated in HDF5 1.12.0, use H5Pencode2()")
    )]
    #[cfg_attr(not(feature = "1.12.0"), link_name = "H5Pencode")]
    pub fn H5Pencode1(plist_id: hid_t, buf: *mut c_void, nalloc: *mut size_t) -> herr_t;
    #[cfg(feature = "1.12.0")]
    pub fn H5Pencode2(
        plist_id: hid_t, buf: *mut c_void, nalloc: *mut size_t, fapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Pdecode(buf: *const c_void) -> hid_t;
    #[cfg_attr(
        hdf5_1_10_1,
        deprecated(note = "deprecated in HDF5 1.10.1, use H5Pset_file_space_strategy()")
    )]
    pub fn H5Pset_file_space(
        plist_id: hid_t, strategy: H5F_file_space_type_t, threshold: hsize_t,
    ) -> herr_t;
    #[cfg_attr(
        hdf5_1_10_1,
        deprecated(note = "deprecated in HDF5 1.10.1, use H5Pget_file_space_strategy()")
    )]
    pub fn H5Pget_file_space(
        plist_id: hid_t, strategy: *mut H5F_file_space_type_t, threshold: *mut hsize_t,
    ) -> herr_t;
}

#[cfg(all(feature = "1.10.0", not(feature = "1.12.0")))]
pub use self::H5Pencode1 as H5Pencode;
#[cfg(feature = "1.12.0")]
pub use self::H5Pencode2 as H5Pencode;

#[cfg(all(feature = "1.10.0", feature = "have-parallel"))]
extern "C" {
    pub fn H5Pset_coll_metadata_write(fapl_id: hid_t, is_collective: hbool_t) -> herr_t;
    pub fn H5Pget_coll_metadata_write(fapl_id: hid_t, is_collective: *mut hbool_t) -> herr_t;
    pub fn H5Pset_all_coll_metadata_ops(accpl_id: hid_t, is_collective: hbool_t) -> herr_t;
    pub fn H5Pget_all_coll_metadata_ops(accpl_id: hid_t, is_collective: *mut hbool_t) -> herr_t;
}

#[cfg(feature = "1.10.1")]
extern "C" {
    pub fn H5Pset_evict_on_close(fapl_id: hid_t, evict_on_close: hbool_t) -> herr_t;
    pub fn H5Pget_evict_on_close(fapl_id: hid_t, evict_on_close: *mut hbool_t) -> herr_t;
    pub fn H5Pset_mdc_image_config(
        plist_id: hid_t, config_ptr: *const H5AC_cache_image_config_t,
    ) -> herr_t;
    pub fn H5Pget_mdc_image_config(
        plist_id: hid_t, config_ptr: *mut H5AC_cache_image_config_t,
    ) -> herr_t;
    pub fn H5Pset_page_buffer_size(
        plist_id: hid_t, buf_size: size_t, min_meta_per: c_uint, min_raw_per: c_uint,
    ) -> herr_t;
    pub fn H5Pget_page_buffer_size(
        plist_id: hid_t, buf_size: *mut size_t, min_meta_per: *mut c_uint, min_raw_per: *mut c_uint,
    ) -> herr_t;
    pub fn H5Pset_file_space_strategy(
        plist_id: hid_t, strategy: H5F_fspace_strategy_t, persist: hbool_t, threshold: hsize_t,
    ) -> herr_t;
    pub fn H5Pget_file_space_strategy(
        plist_id: hid_t, strategy: *mut H5F_fspace_strategy_t, persist: *mut hbool_t,
        threshold: *mut hsize_t,
    ) -> herr_t;
    pub fn H5Pset_file_space_page_size(plist_id: hid_t, fsp_size: hsize_t) -> herr_t;
    pub fn H5Pget_file_space_page_size(plist_id: hid_t, fsp_size: *mut hsize_t) -> herr_t;
}

#[cfg(feature = "1.10.2")]
extern "C" {
    pub fn H5Pset_virtual_prefix(dapl_id: hid_t, prefix: *const c_char) -> herr_t;
    pub fn H5Pget_virtual_prefix(dapl_id: hid_t, prefix: *mut c_char, size: size_t) -> ssize_t;
}

#[cfg(feature = "1.10.5")]
extern "C" {
    pub fn H5Pget_dset_no_attrs_hint(dcpl_id: hid_t, minimize: *mut hbool_t) -> herr_t;
    pub fn H5Pset_dset_no_attrs_hint(dcpl_id: hid_t, minimize: hbool_t) -> herr_t;
}

#[cfg(any(all(feature = "1.10.7", not(feature = "1.12.0")), feature = "1.12.1"))]
extern "C" {
    pub fn H5Pget_file_locking(
        fapl_id: hid_t, use_file_locking: *mut hbool_t, ignore_when_disable: *mut hbool_t,
    ) -> herr_t;
    pub fn H5Pset_file_locking(
        fapl_id: hid_t, use_file_locking: hbool_t, ignore_when_disable: hbool_t,
    ) -> herr_t;

}

#[cfg(feature = "1.12.0")]
extern "C" {
    pub fn H5Pget_vol_id(plist_id: hid_t, vol_id: *mut hid_t) -> herr_t;
    pub fn H5Pget_vol_info(plist_id: hid_t, vol_info: *mut *mut c_void) -> herr_t;
    pub fn H5Pset_vol(plist_id: hid_t, new_vol_id: hid_t, new_vol_id: *const c_void) -> herr_t;
}

#[cfg(feature = "1.14.0")]
extern "C" {
    pub fn H5Pget_driver_config_str(
        fapl_id: hid_t, config_buf: *mut c_char, buf_size: size_t,
    ) -> ssize_t;
    pub fn H5Pget_vol_cap_flags(plist_id: hid_t, cap_flags: *mut c_uint) -> herr_t;
    pub fn H5Pset_dataset_io_hyperslab_selection(
        plist_id: hid_t, rank: c_uint, op: H5S_seloper_t, start: *const hsize_t,
        stride: *const hsize_t, count: *const hsize_t, block: *const hsize_t,
    ) -> herr_t;
    pub fn H5Pset_driver_by_name(
        plist_id: hid_t, driver_name: *const c_char, driver_config: *const c_char,
    ) -> herr_t;
    pub fn H5Pset_driver_by_value(
        plist_id: hid_t, driver_value: H5FD_class_value_t, driver_config: *const c_char,
    ) -> herr_t;
}
