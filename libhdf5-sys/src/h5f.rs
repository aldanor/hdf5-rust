pub use self::H5F_scope_t::*;
pub use self::H5F_close_degree_t::*;
pub use self::H5F_mem_t::*;
pub use self::H5F_libver_t::*;

use libc::{c_int, c_uint, c_void, c_char, c_double, size_t, ssize_t};

use h5::{herr_t, hsize_t, htri_t, hssize_t, H5_ih_info_t};
use h5i::hid_t;
use h5ac::H5AC_cache_config_t;

/* these flags call H5check() in the C library */
pub const H5F_ACC_RDONLY:   c_uint = 0x0000;
pub const H5F_ACC_RDWR:     c_uint = 0x0001;
pub const H5F_ACC_TRUNC:    c_uint = 0x0002;
pub const H5F_ACC_EXCL:     c_uint = 0x0004;
pub const H5F_ACC_DEBUG:    c_uint = 0x0008;
pub const H5F_ACC_CREAT:    c_uint = 0x0010;
pub const H5F_ACC_DEFAULT:  c_uint = 0xffff;

pub const H5F_OBJ_FILE:     c_uint = 0x0001;
pub const H5F_OBJ_DATASET:  c_uint = 0x0002;
pub const H5F_OBJ_GROUP:    c_uint = 0x0004;
pub const H5F_OBJ_DATATYPE: c_uint = 0x0008;
pub const H5F_OBJ_ATTR:     c_uint = 0x0010;
pub const H5F_OBJ_ALL:      c_uint = H5F_OBJ_FILE |
                                     H5F_OBJ_DATASET |
                                     H5F_OBJ_GROUP |
                                     H5F_OBJ_DATATYPE |
                                     H5F_OBJ_ATTR;
pub const H5F_OBJ_LOCAL:    c_uint = 0x0020;

pub const H5F_FAMILY_DEFAULT: hsize_t = 0;

pub const H5F_MPIO_DEBUG_KEY: &'static str = "H5F_mpio_debug_key";

pub const H5F_UNLIMITED: hsize_t = !0;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5F_scope_t {
    H5F_SCOPE_LOCAL  = 0,
    H5F_SCOPE_GLOBAL = 1,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5F_close_degree_t {
    H5F_CLOSE_DEFAULT = 0,
    H5F_CLOSE_WEAK    = 1,
    H5F_CLOSE_SEMI    = 2,
    H5F_CLOSE_STRONG  = 3,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct H5F_info_t {
    pub super_ext_size: hsize_t,
    pub sohm: __H5F_info_t__sohm,
}

impl ::std::default::Default for H5F_info_t {
    fn default() -> H5F_info_t { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct __H5F_info_t__sohm {
    pub hdr_size: hsize_t,
    pub msgs_info: H5_ih_info_t,
}

impl ::std::default::Default for __H5F_info_t__sohm {
    fn default() -> __H5F_info_t__sohm { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5F_mem_t {
    H5FD_MEM_NOLIST  = -1,
    H5FD_MEM_DEFAULT = 0,
    H5FD_MEM_SUPER   = 1,
    H5FD_MEM_BTREE   = 2,
    H5FD_MEM_DRAW    = 3,
    H5FD_MEM_GHEAP   = 4,
    H5FD_MEM_LHEAP   = 5,
    H5FD_MEM_OHDR    = 6,
    H5FD_MEM_NTYPES  = 7,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5F_libver_t {
    H5F_LIBVER_EARLIEST = 0,
    H5F_LIBVER_LATEST   = 1,
}

pub const H5F_LIBVER_18: H5F_libver_t = H5F_LIBVER_LATEST;

extern {
    pub fn H5Fis_hdf5(filename: *const c_char) -> htri_t;
    pub fn H5Fcreate(filename: *const c_char, flags: c_uint, create_plist: hid_t, access_plist:
                     hid_t) -> hid_t;
    pub fn H5Fopen(filename: *const c_char, flags: c_uint, access_plist: hid_t) -> hid_t;
    pub fn H5Freopen(file_id: hid_t) -> hid_t;
    pub fn H5Fflush(object_id: hid_t, scope: H5F_scope_t) -> herr_t;
    pub fn H5Fclose(file_id: hid_t) -> herr_t;
    pub fn H5Fget_create_plist(file_id: hid_t) -> hid_t;
    pub fn H5Fget_access_plist(file_id: hid_t) -> hid_t;
    pub fn H5Fget_intent(file_id: hid_t, intent: *mut c_uint) -> herr_t;
    pub fn H5Fget_obj_count(file_id: hid_t, types: c_uint) -> ssize_t;
    pub fn H5Fget_obj_ids(file_id: hid_t, types: c_uint, max_objs: size_t, obj_id_list: *mut hid_t)
                          -> ssize_t;
    pub fn H5Fget_vfd_handle(file_id: hid_t, fapl: hid_t, file_handle: *mut *mut c_void) -> herr_t;
    pub fn H5Fmount(loc: hid_t, name: *const c_char, child: hid_t, plist: hid_t) -> herr_t;
    pub fn H5Funmount(loc: hid_t, name: *const c_char) -> herr_t;
    pub fn H5Fget_freespace(file_id: hid_t) -> hssize_t;
    pub fn H5Fget_filesize(file_id: hid_t, size: *mut hsize_t) -> herr_t;
    pub fn H5Fget_file_image(file_id: hid_t, buf_ptr: *mut c_void, buf_len: size_t) -> ssize_t;
    pub fn H5Fget_mdc_config(file_id: hid_t, config_ptr: *mut H5AC_cache_config_t) -> herr_t;
    pub fn H5Fset_mdc_config(file_id: hid_t, config_ptr: *mut H5AC_cache_config_t) -> herr_t;
    pub fn H5Fget_mdc_hit_rate(file_id: hid_t, hit_rate_ptr: *mut c_double) -> herr_t;
    pub fn H5Fget_mdc_size(file_id: hid_t, max_size_ptr: *mut size_t, min_clean_size_ptr: *mut
                           size_t, cur_size_ptr: *mut size_t, cur_num_entries_ptr: *mut c_int) ->
                           herr_t;
    pub fn H5Freset_mdc_hit_rate_stats(file_id: hid_t) -> herr_t;
    pub fn H5Fget_name(obj_id: hid_t, name: *mut c_char, size: size_t) -> ssize_t;
    pub fn H5Fget_info(obj_id: hid_t, bh_info: *mut H5F_info_t) -> herr_t;
    pub fn H5Fclear_elink_file_cache(file_id: hid_t) -> herr_t;
}
