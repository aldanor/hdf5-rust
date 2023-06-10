//! File drivers
use std::mem;

pub use self::H5FD_file_image_op_t::*;
pub use super::h5f::H5F_mem_t::*;

use crate::internal_prelude::*;

use crate::h5f::{H5F_close_degree_t, H5F_mem_t};

#[cfg(not(feature = "1.14.0"))]
pub const H5_HAVE_VFL: c_uint = 1;

pub const H5FD_VFD_DEFAULT: c_uint = 0;

pub type H5FD_mem_t = H5F_mem_t;

pub const H5FD_MEM_FHEAP_HDR: H5FD_mem_t = H5FD_MEM_OHDR;
pub const H5FD_MEM_FHEAP_IBLOCK: H5FD_mem_t = H5FD_MEM_OHDR;
pub const H5FD_MEM_FHEAP_DBLOCK: H5FD_mem_t = H5FD_MEM_LHEAP;
pub const H5FD_MEM_FHEAP_HUGE_OBJ: H5FD_mem_t = H5FD_MEM_DRAW;

pub const H5FD_MEM_FSPACE_HDR: H5FD_mem_t = H5FD_MEM_OHDR;
pub const H5FD_MEM_FSPACE_SINFO: H5FD_mem_t = H5FD_MEM_LHEAP;

pub const H5FD_MEM_SOHM_TABLE: H5FD_mem_t = H5FD_MEM_OHDR;
pub const H5FD_MEM_SOHM_INDEX: H5FD_mem_t = H5FD_MEM_BTREE;

pub static H5FD_FLMAP_SINGLE: [H5FD_mem_t; 7] = [
    H5FD_MEM_SUPER,
    H5FD_MEM_SUPER,
    H5FD_MEM_SUPER,
    H5FD_MEM_SUPER,
    H5FD_MEM_SUPER,
    H5FD_MEM_SUPER,
    H5FD_MEM_SUPER,
];

pub static H5FD_FLMAP_DICHOTOMY: [H5FD_mem_t; 7] = [
    H5FD_MEM_SUPER,
    H5FD_MEM_SUPER,
    H5FD_MEM_SUPER,
    H5FD_MEM_DRAW,
    H5FD_MEM_DRAW,
    H5FD_MEM_SUPER,
    H5FD_MEM_SUPER,
];

pub static H5FD_FLMAP_DEFAULT: [H5FD_mem_t; 7] = [
    H5FD_MEM_DEFAULT,
    H5FD_MEM_DEFAULT,
    H5FD_MEM_DEFAULT,
    H5FD_MEM_DEFAULT,
    H5FD_MEM_DEFAULT,
    H5FD_MEM_DEFAULT,
    H5FD_MEM_DEFAULT,
];

pub const H5FD_FEAT_AGGREGATE_METADATA: c_uint = 0x00000001;
pub const H5FD_FEAT_ACCUMULATE_METADATA_WRITE: c_uint = 0x00000002;
pub const H5FD_FEAT_ACCUMULATE_METADATA_READ: c_uint = 0x00000004;
pub const H5FD_FEAT_ACCUMULATE_METADATA: c_uint =
    H5FD_FEAT_ACCUMULATE_METADATA_WRITE | H5FD_FEAT_ACCUMULATE_METADATA_READ;
pub const H5FD_FEAT_DATA_SIEVE: c_uint = 0x00000008;
pub const H5FD_FEAT_AGGREGATE_SMALLDATA: c_uint = 0x00000010;
pub const H5FD_FEAT_IGNORE_DRVRINFO: c_uint = 0x00000020;
pub const H5FD_FEAT_DIRTY_SBLK_LOAD: c_uint = 0x00000040;
pub const H5FD_FEAT_POSIX_COMPAT_HANDLE: c_uint = 0x00000080;
pub const H5FD_FEAT_ALLOW_FILE_IMAGE: c_uint = 0x00000400;
pub const H5FD_FEAT_CAN_USE_FILE_IMAGE_CALLBACKS: c_uint = 0x00000800;
#[cfg(feature = "1.10.2")]
pub const H5FD_FEAT_DEFAULT_VFD_COMPATIBLE: c_uint = 0x00008000;

/* Flags for H5Pset_fapl_log() */
/* Flags for tracking 'meta' operations (truncate) */
pub const H5FD_LOG_TRUNCATE: c_ulonglong = 0x00000001;
pub const H5FD_LOG_META_IO: c_ulonglong = H5FD_LOG_TRUNCATE;
/* Flags for tracking where reads/writes/seeks occur */
pub const H5FD_LOG_LOC_READ: c_ulonglong = 0x00000002;
pub const H5FD_LOG_LOC_WRITE: c_ulonglong = 0x00000004;
pub const H5FD_LOG_LOC_SEEK: c_ulonglong = 0x00000008;
pub const H5FD_LOG_LOC_IO: c_ulonglong = H5FD_LOG_LOC_READ | H5FD_LOG_LOC_WRITE | H5FD_LOG_LOC_SEEK;
/* Flags for tracking number of times each byte is read/written */
pub const H5FD_LOG_FILE_READ: c_ulonglong = 0x00000010;
pub const H5FD_LOG_FILE_WRITE: c_ulonglong = 0x00000020;
pub const H5FD_LOG_FILE_IO: c_ulonglong = H5FD_LOG_FILE_READ | H5FD_LOG_FILE_WRITE;
/* Flag for tracking "flavor" (type) of information stored at each byte */
pub const H5FD_LOG_FLAVOR: c_ulonglong = 0x00000040;
/* Flags for tracking total number of reads/writes/seeks/truncates */
pub const H5FD_LOG_NUM_READ: c_ulonglong = 0x00000080;
pub const H5FD_LOG_NUM_WRITE: c_ulonglong = 0x00000100;
pub const H5FD_LOG_NUM_SEEK: c_ulonglong = 0x00000200;
pub const H5FD_LOG_NUM_TRUNCATE: c_ulonglong = 0x00000400;
pub const H5FD_LOG_NUM_IO: c_ulonglong =
    H5FD_LOG_NUM_READ | H5FD_LOG_NUM_WRITE | H5FD_LOG_NUM_SEEK | H5FD_LOG_NUM_TRUNCATE;
/* Flags for tracking time spent in open/stat/read/write/seek/truncate/close */
pub const H5FD_LOG_TIME_OPEN: c_ulonglong = 0x00000800;
pub const H5FD_LOG_TIME_STAT: c_ulonglong = 0x00001000;
pub const H5FD_LOG_TIME_READ: c_ulonglong = 0x00002000;
pub const H5FD_LOG_TIME_WRITE: c_ulonglong = 0x00004000;
pub const H5FD_LOG_TIME_SEEK: c_ulonglong = 0x00008000;
pub const H5FD_LOG_TIME_TRUNCATE: c_ulonglong = 0x00010000;
pub const H5FD_LOG_TIME_CLOSE: c_ulonglong = 0x00020000;
pub const H5FD_LOG_TIME_IO: c_ulonglong = H5FD_LOG_TIME_OPEN
    | H5FD_LOG_TIME_STAT
    | H5FD_LOG_TIME_READ
    | H5FD_LOG_TIME_WRITE
    | H5FD_LOG_TIME_SEEK
    | H5FD_LOG_TIME_TRUNCATE
    | H5FD_LOG_TIME_CLOSE;
/* Flags for tracking allocation/release of space in file */
pub const H5FD_LOG_ALLOC: c_ulonglong = 0x00040000;
pub const H5FD_LOG_FREE: c_ulonglong = 0x00080000;
pub const H5FD_LOG_ALL: c_ulonglong = H5FD_LOG_FREE
    | H5FD_LOG_ALLOC
    | H5FD_LOG_TIME_IO
    | H5FD_LOG_NUM_IO
    | H5FD_LOG_FLAVOR
    | H5FD_LOG_FILE_IO
    | H5FD_LOG_LOC_IO
    | H5FD_LOG_META_IO;

pub type H5FD_class_value_t = c_int;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5FD_class_t {
    #[cfg(feature = "1.14.0")]
    pub value: H5FD_class_value_t,
    pub name: *const c_char,
    pub maxaddr: haddr_t,
    pub fc_degree: H5F_close_degree_t,
    #[cfg(feature = "1.14.0")]
    pub terminate: Option<extern "C" fn() -> herr_t>,
    pub sb_size: Option<extern "C" fn(file: *mut H5FD_t) -> hsize_t>,
    pub sb_encode:
        Option<extern "C" fn(file: *mut H5FD_t, name: *mut c_char, p: *mut c_uchar) -> herr_t>,
    pub sb_decode:
        Option<extern "C" fn(f: *mut H5FD_t, name: *const c_char, p: *const c_uchar) -> herr_t>,
    pub fapl_size: size_t,
    pub fapl_get: Option<extern "C" fn(file: *mut H5FD_t) -> *mut c_void>,
    pub fapl_copy: Option<extern "C" fn(fapl: *const c_void) -> *mut c_void>,
    pub fapl_free: Option<extern "C" fn(fapl: *mut c_void) -> herr_t>,
    pub dxpl_size: size_t,
    pub dxpl_copy: Option<extern "C" fn(dxpl: *const c_void) -> *mut c_void>,
    pub dxpl_free: Option<extern "C" fn(dxpl: *mut c_void) -> herr_t>,
    pub open: Option<
        extern "C" fn(
            name: *const c_char,
            flags: c_uint,
            fapl: hid_t,
            maxaddr: haddr_t,
        ) -> *mut H5FD_t,
    >,
    pub close: Option<extern "C" fn(file: *mut H5FD_t) -> herr_t>,
    pub cmp: Option<extern "C" fn(f1: *const H5FD_t, f2: *const H5FD_t) -> c_int>,
    pub query: Option<extern "C" fn(f1: *const H5FD_t, flags: *mut c_ulong) -> herr_t>,
    pub get_type_map:
        Option<extern "C" fn(file: *const H5FD_t, type_map: *mut H5FD_mem_t) -> herr_t>,
    pub alloc: Option<
        extern "C" fn(
            file: *mut H5FD_t,
            type_: H5FD_mem_t,
            dxpl_id: hid_t,
            size: hsize_t,
        ) -> haddr_t,
    >,
    pub free: Option<
        extern "C" fn(
            file: *mut H5FD_t,
            type_: H5FD_mem_t,
            dxpl_id: hid_t,
            addr: haddr_t,
            size: hsize_t,
        ) -> herr_t,
    >,
    pub get_eoa: Option<extern "C" fn(file: *const H5FD_t, type_: H5FD_mem_t) -> haddr_t>,
    pub set_eoa:
        Option<extern "C" fn(file: *mut H5FD_t, type_: H5FD_mem_t, addr: haddr_t) -> herr_t>,
    pub get_eof: Option<extern "C" fn(file: *const H5FD_t) -> haddr_t>,
    pub get_handle: Option<
        extern "C" fn(file: *mut H5FD_t, fapl: hid_t, file_handle: *mut *mut c_void) -> herr_t,
    >,
    pub read: Option<
        extern "C" fn(
            file: *mut H5FD_t,
            type_: H5FD_mem_t,
            dxpl: hid_t,
            addr: haddr_t,
            size: size_t,
            buffer: *mut c_void,
        ) -> herr_t,
    >,
    pub write: Option<
        extern "C" fn(
            file: *mut H5FD_t,
            type_: H5FD_mem_t,
            dxpl: hid_t,
            addr: haddr_t,
            size: size_t,
            buffer: *const c_void,
        ) -> herr_t,
    >,
    pub flush: Option<extern "C" fn(file: *mut H5FD_t, dxpl_id: hid_t, closing: c_uint) -> herr_t>,
    pub truncate:
        Option<extern "C" fn(file: *mut H5FD_t, dxpl_id: hid_t, closing: hbool_t) -> herr_t>,
    pub lock: Option<
        extern "C" fn(
            file: *mut H5FD_t,
            oid: *mut c_uchar,
            lock_type: c_uint,
            last: hbool_t,
        ) -> herr_t,
    >,
    pub unlock:
        Option<extern "C" fn(file: *mut H5FD_t, oid: *mut c_uchar, last: hbool_t) -> herr_t>,
    #[cfg(feature = "1.14.0")]
    pub del: Option<extern "C" fn(name: *const c_char, fapl: hid_t) -> herr_t>,
    #[cfg(feature = "1.14.0")]
    pub ctl: Option<
        extern "C" fn(
            file: *mut H5FD_t,
            op_code: u64,
            flags: u64,
            input: *const c_char,
            output: *mut *mut c_void,
        ) -> herr_t,
    >,
    pub fl_map: [H5FD_mem_t; 7usize],
}

impl Default for H5FD_class_t {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5FD_free_t {
    pub addr: haddr_t,
    pub size: hsize_t,
    pub next: *mut Self,
}

impl Default for H5FD_free_t {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5FD_t {
    pub driver_id: hid_t,
    pub cls: *const H5FD_class_t,
    pub fileno: c_ulong,
    pub feature_flags: c_ulong,
    pub maxaddr: haddr_t,
    pub base_addr: haddr_t,
    pub threshold: hsize_t,
    pub alignment: hsize_t,
}

impl Default for H5FD_t {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug)]
pub enum H5FD_file_image_op_t {
    H5FD_FILE_IMAGE_OP_NO_OP = 0,
    H5FD_FILE_IMAGE_OP_PROPERTY_LIST_SET = 1,
    H5FD_FILE_IMAGE_OP_PROPERTY_LIST_COPY = 2,
    H5FD_FILE_IMAGE_OP_PROPERTY_LIST_GET = 3,
    H5FD_FILE_IMAGE_OP_PROPERTY_LIST_CLOSE = 4,
    H5FD_FILE_IMAGE_OP_FILE_OPEN = 5,
    H5FD_FILE_IMAGE_OP_FILE_RESIZE = 6,
    H5FD_FILE_IMAGE_OP_FILE_CLOSE = 7,
}

#[cfg(feature = "1.8.9")]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5FD_file_image_callbacks_t {
    pub image_malloc: Option<
        extern "C" fn(
            size: size_t,
            file_image_op: H5FD_file_image_op_t,
            udata: *mut c_void,
        ) -> *mut c_void,
    >,
    pub image_memcpy: Option<
        extern "C" fn(
            dest: *mut c_void,
            src: *const c_void,
            size: size_t,
            file_image_op: H5FD_file_image_op_t,
            udata: *mut c_void,
        ) -> *mut c_void,
    >,
    pub image_realloc: Option<
        extern "C" fn(
            ptr: *mut c_void,
            size: size_t,
            file_image_op: H5FD_file_image_op_t,
            udata: *mut c_void,
        ) -> *mut c_void,
    >,
    pub image_free: Option<
        extern "C" fn(
            ptr: *mut c_void,
            file_image_op: H5FD_file_image_op_t,
            udata: *mut c_void,
        ) -> herr_t,
    >,
    pub udata_copy: Option<extern "C" fn(udata: *mut c_void) -> *mut c_void>,
    pub udata_free: Option<extern "C" fn(udata: *mut c_void) -> herr_t>,
    pub udata: *mut c_void,
}

#[cfg(feature = "1.8.9")]
impl Default for H5FD_file_image_callbacks_t {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

extern "C" {
    pub fn H5FDregister(cls: *const H5FD_class_t) -> hid_t;
    pub fn H5FDunregister(driver_id: hid_t) -> herr_t;
    pub fn H5FDopen(
        name: *const c_char, flags: c_uint, fapl_id: hid_t, maxaddr: haddr_t,
    ) -> *mut H5FD_t;
    pub fn H5FDclose(file: *mut H5FD_t) -> herr_t;
    pub fn H5FDcmp(f1: *const H5FD_t, f2: *const H5FD_t) -> c_int;
    pub fn H5FDquery(f: *const H5FD_t, flags: *mut c_ulong) -> c_int;
    pub fn H5FDalloc(
        file: *mut H5FD_t, type_: H5FD_mem_t, dxpl_id: hid_t, size: hsize_t,
    ) -> haddr_t;
    pub fn H5FDfree(
        file: *mut H5FD_t, type_: H5FD_mem_t, dxpl_id: hid_t, addr: haddr_t, size: hsize_t,
    ) -> herr_t;
    pub fn H5FDget_eoa(file: *mut H5FD_t, type_: H5FD_mem_t) -> haddr_t;
    pub fn H5FDset_eoa(file: *mut H5FD_t, type_: H5FD_mem_t, eoa: haddr_t) -> herr_t;
    pub fn H5FDget_eof(file: *mut H5FD_t) -> haddr_t;
    pub fn H5FDget_vfd_handle(
        file: *mut H5FD_t, fapl: hid_t, file_handle: *mut *mut c_void,
    ) -> herr_t;
    pub fn H5FDread(
        file: *mut H5FD_t, type_: H5FD_mem_t, dxpl_id: hid_t, addr: haddr_t, size: size_t,
        buf: *mut c_void,
    ) -> herr_t;
    pub fn H5FDwrite(
        file: *mut H5FD_t, type_: H5FD_mem_t, dxpl_id: hid_t, addr: haddr_t, size: size_t,
        buf: *const c_void,
    ) -> herr_t;
    pub fn H5FDflush(file: *mut H5FD_t, dxpl_id: hid_t, closing: c_uint) -> herr_t;
    pub fn H5FDtruncate(file: *mut H5FD_t, dxpl_id: hid_t, closing: hbool_t) -> herr_t;
}

// drivers
extern "C" {
    pub fn H5FD_sec2_init() -> hid_t;
    pub fn H5FD_core_init() -> hid_t;
    pub fn H5FD_stdio_init() -> hid_t;
    pub fn H5FD_family_init() -> hid_t;
    pub fn H5FD_log_init() -> hid_t;
    pub fn H5FD_multi_init() -> hid_t;
}

#[cfg(feature = "have-parallel")]
extern "C" {
    pub fn H5FD_mpio_init() -> hid_t;
}

#[cfg(feature = "have-direct")]
extern "C" {
    pub fn H5FD_direct_init() -> hid_t;
}

#[cfg(feature = "1.10.0")]
extern "C" {
    pub fn H5FDlock(file: *mut H5FD_t, rw: hbool_t) -> herr_t;
    pub fn H5FDunlock(file: *mut H5FD_t) -> herr_t;
}

#[cfg(all(feature = "1.10.6", not(feature = "1.14.0")))]
pub mod hdfs {
    use super::*;
    pub const H5FD__CURR_HDFS_FAPL_T_VERSION: c_uint = 1;
    pub const H5FD__HDFS_NODE_NAME_SPACE: c_uint = 128;
    pub const H5FD__HDFS_USER_NAME_SPACE: c_uint = 128;
    pub const H5FD__HDFS_KERB_CACHE_PATH_SPACE: c_uint = 128;

    #[repr(C)]
    pub struct H5FD_hdfs_fapl_t {
        version: i32,
        namenode_name: [c_char; H5FD__HDFS_NODE_NAME_SPACE as usize + 1],
        namenode_port: i32,
        user_name: [c_char; H5FD__HDFS_USER_NAME_SPACE as usize + 1],
        kerberos_ticket_cache: [c_char; H5FD__HDFS_KERB_CACHE_PATH_SPACE as usize + 1],
        stream_buffer_size: i32,
    }

    extern "C" {
        pub fn H5FD_hdfs_init() -> hid_t;
        pub fn H5Pget_fapl_hdfs(fapl_id: hid_t, fa: *mut H5FD_hdfs_fapl_t) -> herr_t;
        pub fn H5Pset_fapl_hdfs(fapl_id: hid_t, fa: *mut H5FD_hdfs_fapl_t) -> herr_t;
    }
}

#[cfg(feature = "1.10.6")]
pub mod ros3 {
    use super::*;
    pub const H5FD_CURR_ROS3_FAPL_T_VERSION: c_uint = 1;
    pub const H5FD_ROS3_MAX_REGION_LEN: c_uint = 128;
    pub const H5FD_ROS3_MAX_SECRET_ID_LEN: c_uint = 128;
    pub const H5FD_ROS3_MAX_SECRET_KEY_LEN: c_uint = 128;

    #[repr(C)]
    pub struct H5FD_ros3_fapl_t {
        version: i32,
        authenticate: hbool_t,
        aws_region: [c_char; H5FD_ROS3_MAX_REGION_LEN as usize + 1],
        secret_id: [c_char; H5FD_ROS3_MAX_SECRET_ID_LEN as usize + 1],
        secret_key: [c_char; H5FD_ROS3_MAX_SECRET_KEY_LEN as usize + 1],
    }

    extern "C" {
        pub fn H5FD_ros3_init() -> hid_t;
        pub fn H5Pget_fapl_ros3(fapl_id: hid_t, fa: *mut H5FD_ros3_fapl_t) -> herr_t;
        pub fn H5Pset_fapl_ros3(fapl_id: hid_t, fa: *mut H5FD_ros3_fapl_t) -> herr_t;
    }
}

#[cfg(any(all(feature = "1.10.7", not(feature = "1.12.0")), feature = "1.12.1"))]
pub mod splitter {
    use super::*;

    pub const H5FD_CURR_SPLITTER_VFD_CONFIG_VERSION: c_uint = 1;
    pub const H5FD_SPLITTER_PATH_MAX: c_uint = 4096;
    pub const H5FD_SPLITTER_MAGIC: c_uint = 0x2B916880;

    #[repr(C)]
    pub struct H5FD_splitter_vfg_config_t {
        magic: i32,
        version: c_uint,
        rw_fapl_id: hid_t,
        wo_fapl_id: hid_t,
        wo_path: [c_char; H5FD_SPLITTER_PATH_MAX as usize + 1],
        log_file_path: [c_char; H5FD_SPLITTER_PATH_MAX as usize + 1],
        ignore_wo_errs: hbool_t,
    }

    extern "C" {
        pub fn H5FD_splitter_init() -> hid_t;
        pub fn H5Pget_fapl_splitter(
            fapl_id: hid_t, config_ptr: *mut H5FD_splitter_vfg_config_t,
        ) -> herr_t;
        pub fn H5Pset_fapl_splitter(
            fapl_id: hid_t, config_ptr: *mut H5FD_splitter_vfg_config_t,
        ) -> herr_t;
    }
}

#[cfg(feature = "1.10.2")]
extern "C" {
    pub fn H5FDdriver_query(driver_id: hid_t, flags: *mut c_ulong) -> herr_t;
}

#[cfg(feature = "1.14.0")]
type H5FD_perform_init_func_t = Option<extern "C" fn() -> hid_t>;

#[cfg(feature = "1.14.0")]
extern "C" {
    pub fn H5FDctl(
        file: *mut H5FD_t, op_cod: u64, flags: u64, input: *const c_void, output: *mut *mut c_void,
    ) -> herr_t;
    pub fn H5FDdelete(name: *const c_char, fapl_id: hid_t) -> herr_t;
    pub fn H5FDis_driver_registered_by_name(driver_name: *const c_char) -> htri_t;
    pub fn H5FDis_driver_registered_by_value(driver_value: H5FD_class_value_t) -> htri_t;
    pub fn H5FDperform_init(p: H5FD_perform_init_func_t) -> hid_t;
}
