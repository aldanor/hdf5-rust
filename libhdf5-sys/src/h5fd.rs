pub use super::h5f::H5F_mem_t::*;
pub use self::H5FD_file_image_op_t::*;

use libc::{c_int, c_uint, c_void, c_char, c_uchar, c_ulong, size_t};

use h5::{herr_t, haddr_t, hsize_t, hbool_t};
use h5f::{H5F_mem_t, H5F_close_degree_t};
use h5i::hid_t;

pub const H5_HAVE_VFL: c_uint = 1;

pub const H5FD_VFD_DEFAULT: c_uint = 0;

pub type H5FD_mem_t = H5F_mem_t;

pub const H5FD_MEM_FHEAP_HDR:      H5FD_mem_t = H5FD_MEM_OHDR;
pub const H5FD_MEM_FHEAP_IBLOCK:   H5FD_mem_t = H5FD_MEM_OHDR;
pub const H5FD_MEM_FHEAP_DBLOCK:   H5FD_mem_t = H5FD_MEM_LHEAP;
pub const H5FD_MEM_FHEAP_HUGE_OBJ: H5FD_mem_t = H5FD_MEM_DRAW;

pub const H5FD_MEM_FSPACE_HDR:     H5FD_mem_t = H5FD_MEM_OHDR;
pub const H5FD_MEM_FSPACE_SINFO:   H5FD_mem_t = H5FD_MEM_LHEAP;

pub const H5FD_MEM_SOHM_TABLE:     H5FD_mem_t = H5FD_MEM_OHDR;
pub const H5FD_MEM_SOHM_INDEX:     H5FD_mem_t = H5FD_MEM_BTREE;

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

pub const H5FD_FEAT_AGGREGATE_METADATA:           c_uint = 0x00000001;
pub const H5FD_FEAT_ACCUMULATE_METADATA_WRITE:    c_uint = 0x00000002;
pub const H5FD_FEAT_ACCUMULATE_METADATA_READ:     c_uint = 0x00000004;
pub const H5FD_FEAT_ACCUMULATE_METADATA:          c_uint = H5FD_FEAT_ACCUMULATE_METADATA_WRITE |
                                                           H5FD_FEAT_ACCUMULATE_METADATA_READ;
pub const H5FD_FEAT_DATA_SIEVE:                   c_uint = 0x00000008;
pub const H5FD_FEAT_AGGREGATE_SMALLDATA:          c_uint = 0x00000010;
pub const H5FD_FEAT_IGNORE_DRVRINFO:              c_uint = 0x00000020;
pub const H5FD_FEAT_DIRTY_SBLK_LOAD:              c_uint = 0x00000040;
pub const H5FD_FEAT_POSIX_COMPAT_HANDLE:          c_uint = 0x00000080;
pub const H5FD_FEAT_ALLOW_FILE_IMAGE:             c_uint = 0x00000400;
pub const H5FD_FEAT_CAN_USE_FILE_IMAGE_CALLBACKS: c_uint = 0x00000800;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct H5FD_class_t {
    pub name: *const c_char,
    pub maxaddr: haddr_t,
    pub fc_degree: H5F_close_degree_t,
    pub sb_size: Option<extern fn (file: *mut H5FD_t) -> hsize_t>,
    pub sb_encode: Option<extern fn (file: *mut H5FD_t, name: *mut c_char, p: *mut c_uchar) ->
                                     herr_t>,
    pub sb_decode: Option<extern fn (f: *mut H5FD_t, name: *const c_char, p: *const c_uchar) ->
                                     herr_t>,
    pub fapl_size: size_t,
    pub fapl_get: Option<extern fn (file: *mut H5FD_t) -> *mut c_void>,
    pub fapl_copy: Option<extern fn (fapl: *const c_void) -> *mut c_void>,
    pub fapl_free: Option<extern fn (fapl: *mut c_void) -> herr_t>,
    pub dxpl_size: size_t,
    pub dxpl_copy: Option<extern fn (dxpl: *const c_void) -> *mut c_void>,
    pub dxpl_free: Option<extern fn (dxpl: *mut c_void) -> herr_t>,
    pub open: Option<extern fn (name: *const c_char, flags: c_uint, fapl: hid_t, maxaddr: haddr_t)
                                -> *mut H5FD_t>,
    pub close: Option<extern fn (file: *mut H5FD_t) -> herr_t>,
    pub cmp: Option<extern fn (f1: *const H5FD_t, f2: *const H5FD_t) -> c_int>,
    pub query: Option<extern fn (f1: *const H5FD_t, flags: *mut c_ulong) -> herr_t>,
    pub get_type_map: Option<extern fn (file: *const H5FD_t, type_map: *mut H5FD_mem_t) -> herr_t>,
    pub alloc: Option<extern fn (file: *mut H5FD_t, _type: H5FD_mem_t, dxpl_id: hid_t, size:
                                 hsize_t) -> haddr_t>,
    pub free: Option<extern fn (file: *mut H5FD_t, _type: H5FD_mem_t, dxpl_id: hid_t, addr: haddr_t,
                                size: hsize_t) -> herr_t>,
    pub get_eoa: Option<extern fn (file: *const H5FD_t, _type: H5FD_mem_t) -> haddr_t>,
    pub set_eoa: Option<extern fn (file: *mut H5FD_t, _type: H5FD_mem_t, addr: haddr_t) -> herr_t>,
    pub get_eof: Option<extern fn (file: *const H5FD_t) -> haddr_t>,
    pub get_handle: Option<extern fn (file: *mut H5FD_t, fapl: hid_t, file_handle: *mut *mut c_void)
                                      -> herr_t>,
    pub read: Option<extern fn (file: *mut H5FD_t, _type: H5FD_mem_t, dxpl: hid_t, addr: haddr_t,
                                size: size_t, buffer: *mut c_void) -> herr_t>,
    pub write: Option<extern fn (file: *mut H5FD_t, _type: H5FD_mem_t, dxpl: hid_t, addr: haddr_t,
                                 size: size_t, buffer: *const c_void) -> herr_t>,
    pub flush: Option<extern fn (file: *mut H5FD_t, dxpl_id: hid_t, closing: c_uint) -> herr_t>,
    pub truncate: Option<extern fn (file: *mut H5FD_t, dxpl_id: hid_t, closing: hbool_t) -> herr_t>,
    pub lock: Option<extern fn (file: *mut H5FD_t, oid: *mut c_uchar, lock_type: c_uint, last:
                                hbool_t) -> herr_t>,
    pub unlock: Option<extern fn (file: *mut H5FD_t, oid: *mut c_uchar, last: hbool_t) -> herr_t>,
    pub fl_map: [H5FD_mem_t; 7usize],
}

impl ::std::default::Default for H5FD_class_t {
    fn default() -> H5FD_class_t { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct H5FD_free_t {
    pub addr: haddr_t,
    pub size: hsize_t,
    pub next: *mut H5FD_free_t,
}

impl ::std::default::Default for H5FD_free_t {
    fn default() -> H5FD_free_t { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy, Clone)]
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

impl ::std::default::Default for H5FD_t {
    fn default() -> H5FD_t { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5FD_file_image_op_t {
    H5FD_FILE_IMAGE_OP_NO_OP               = 0,
    H5FD_FILE_IMAGE_OP_PROPERTY_LIST_SET   = 1,
    H5FD_FILE_IMAGE_OP_PROPERTY_LIST_COPY  = 2,
    H5FD_FILE_IMAGE_OP_PROPERTY_LIST_GET   = 3,
    H5FD_FILE_IMAGE_OP_PROPERTY_LIST_CLOSE = 4,
    H5FD_FILE_IMAGE_OP_FILE_OPEN           = 5,
    H5FD_FILE_IMAGE_OP_FILE_RESIZE         = 6,
    H5FD_FILE_IMAGE_OP_FILE_CLOSE          = 7,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct H5FD_file_image_callbacks_t {
    pub image_malloc: Option<extern fn (size: size_t, file_image_op: H5FD_file_image_op_t, udata:
                                        *mut c_void) -> *mut c_void>,
    pub image_memcpy: Option<extern fn (dest: *mut c_void, src: *const c_void, size: size_t,
                                        file_image_op: H5FD_file_image_op_t, udata: *mut c_void) ->
                                        *mut c_void>,
    pub image_realloc: Option<extern fn (ptr: *mut c_void, size: size_t, file_image_op:
                                         H5FD_file_image_op_t, udata: *mut c_void) -> *mut c_void>,
    pub image_free: Option<extern fn (ptr: *mut c_void, file_image_op: H5FD_file_image_op_t, udata:
                                      *mut c_void) -> herr_t>,
    pub udata_copy: Option<extern fn (udata: *mut c_void) -> *mut c_void>,
    pub udata_free: Option<extern fn (udata: *mut c_void) -> herr_t>,
    pub udata: *mut c_void,
}

impl ::std::default::Default for H5FD_file_image_callbacks_t {
    fn default() -> H5FD_file_image_callbacks_t { unsafe { ::std::mem::zeroed() } }
}

extern {
    pub fn H5FDregister(cls: *const H5FD_class_t) -> hid_t;
    pub fn H5FDunregister(driver_id: hid_t) -> herr_t;
    pub fn H5FDopen(name: *const c_char, flags: c_uint, fapl_id: hid_t, maxaddr:
                    haddr_t) -> *mut H5FD_t;
    pub fn H5FDclose(file: *mut H5FD_t) -> herr_t;
    pub fn H5FDcmp(f1: *const H5FD_t, f2: *const H5FD_t) -> c_int;
    pub fn H5FDquery(f: *const H5FD_t, flags: *mut c_ulong) -> c_int;
    pub fn H5FDalloc(file: *mut H5FD_t, _type: H5FD_mem_t, dxpl_id: hid_t, size: hsize_t) ->
                     haddr_t;
    pub fn H5FDfree(file: *mut H5FD_t, _type: H5FD_mem_t, dxpl_id: hid_t, addr: haddr_t, size:
                    hsize_t) -> herr_t;
    pub fn H5FDget_eoa(file: *mut H5FD_t, _type: H5FD_mem_t) -> haddr_t;
    pub fn H5FDset_eoa(file: *mut H5FD_t, _type: H5FD_mem_t, eoa: haddr_t) -> herr_t;
    pub fn H5FDget_eof(file: *mut H5FD_t) -> haddr_t;
    pub fn H5FDget_vfd_handle(file: *mut H5FD_t, fapl: hid_t, file_handle: *mut *mut c_void)
                              -> herr_t;
    pub fn H5FDread(file: *mut H5FD_t, _type: H5FD_mem_t, dxpl_id: hid_t, addr: haddr_t, size:
                    size_t, buf: *mut c_void) -> herr_t;
    pub fn H5FDwrite(file: *mut H5FD_t, _type: H5FD_mem_t, dxpl_id: hid_t, addr: haddr_t, size:
                     size_t, buf: *const c_void) -> herr_t;
    pub fn H5FDflush(file: *mut H5FD_t, dxpl_id: hid_t, closing: c_uint) -> herr_t;
    pub fn H5FDtruncate(file: *mut H5FD_t, dxpl_id: hid_t, closing: hbool_t) -> herr_t;
}

// sec2 driver
extern {
    pub fn H5FD_sec2_init() -> hid_t;
    pub fn H5FD_sec2_term();
    pub fn H5Pset_fapl_sec2(fapl_id: hid_t) -> herr_t;
}

// core driver
extern {
    pub fn H5FD_core_init() -> hid_t;
    pub fn H5FD_core_term();
    pub fn H5Pset_fapl_core(fapl_id: hid_t, increment: size_t, backing_store: hbool_t) -> herr_t;
    pub fn H5Pget_fapl_core(fapl_id: hid_t, increment: *mut size_t,
                            backing_store: *mut hbool_t) -> herr_t;
}

// stdio driver
extern {
    pub fn H5FD_stdio_init() -> hid_t;
    pub fn H5FD_stdio_term();
    pub fn H5Pset_fapl_stdio(fapl_id: hid_t) -> herr_t;
}
