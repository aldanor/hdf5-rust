pub use self::H5O_type_t::*;
pub use self::H5O_mcdt_search_ret_t::*;

use libc::{c_uint, c_void, c_char, c_ulong, size_t, ssize_t, uint32_t, uint64_t, time_t};

use h5::{herr_t, hsize_t, haddr_t, htri_t, H5_index_t, H5_iter_order_t, H5_ih_info_t};
use h5i::hid_t;

pub const H5O_COPY_SHALLOW_HIERARCHY_FLAG:     c_uint = 0x0001;
pub const H5O_COPY_EXPAND_SOFT_LINK_FLAG:      c_uint = 0x0002;
pub const H5O_COPY_EXPAND_EXT_LINK_FLAG:       c_uint = 0x0004;
pub const H5O_COPY_EXPAND_REFERENCE_FLAG:      c_uint = 0x0008;
pub const H5O_COPY_WITHOUT_ATTR_FLAG:          c_uint = 0x0010;
pub const H5O_COPY_PRESERVE_NULL_FLAG:         c_uint = 0x0020;
pub const H5O_COPY_MERGE_COMMITTED_DTYPE_FLAG: c_uint = 0x0040;
pub const H5O_COPY_ALL:                        c_uint = 0x007F;

pub const H5O_SHMESG_NONE_FLAG:    c_uint = 0x0000;
pub const H5O_SHMESG_SDSPACE_FLAG: c_uint = 1 << 0x0001;
pub const H5O_SHMESG_DTYPE_FLAG:   c_uint = 1 << 0x0003;
pub const H5O_SHMESG_FILL_FLAG:    c_uint = 1 << 0x0005;
pub const H5O_SHMESG_PLINE_FLAG:   c_uint = 1 << 0x000b;
pub const H5O_SHMESG_ATTR_FLAG:    c_uint = 1 << 0x000c;
pub const H5O_SHMESG_ALL_FLAG:     c_uint = H5O_SHMESG_SDSPACE_FLAG |
                                            H5O_SHMESG_DTYPE_FLAG |
                                            H5O_SHMESG_FILL_FLAG |
                                            H5O_SHMESG_PLINE_FLAG |
                                            H5O_SHMESG_ATTR_FLAG;

pub const H5O_HDR_CHUNK0_SIZE:             c_uint = 0x03;
pub const H5O_HDR_ATTR_CRT_ORDER_TRACKED:  c_uint = 0x04;
pub const H5O_HDR_ATTR_CRT_ORDER_INDEXED:  c_uint = 0x08;
pub const H5O_HDR_ATTR_STORE_PHASE_CHANGE: c_uint = 0x10;
pub const H5O_HDR_STORE_TIMES:             c_uint = 0x20;
pub const H5O_HDR_ALL_FLAGS:               c_uint = H5O_HDR_CHUNK0_SIZE |
                                                    H5O_HDR_ATTR_CRT_ORDER_TRACKED |
                                                    H5O_HDR_ATTR_CRT_ORDER_INDEXED |
                                                    H5O_HDR_ATTR_STORE_PHASE_CHANGE |
                                                    H5O_HDR_STORE_TIMES;

pub const H5O_SHMESG_MAX_NINDEXES:  c_uint = 8;
pub const H5O_SHMESG_MAX_LIST_SIZE: c_uint = 5000;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5O_type_t {
    H5O_TYPE_UNKNOWN        = -1,
    H5O_TYPE_GROUP          = 0,
    H5O_TYPE_DATASET        = 1,
    H5O_TYPE_NAMED_DATATYPE = 2,
    H5O_TYPE_NTYPES         = 3,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct H5O_hdr_info_t {
    pub version: c_uint,
    pub nmesgs: c_uint,
    pub nchunks: c_uint,
    pub flags: c_uint,
    pub space: __H5O_hdr_info_t__space,
    pub mesg: __H5O_hdr_info_t__mesg,
}

impl ::std::default::Default for H5O_hdr_info_t {
    fn default() -> H5O_hdr_info_t { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct __H5O_hdr_info_t__space {
    pub total: hsize_t,
    pub meta: hsize_t,
    pub mesg: hsize_t,
    pub free: hsize_t,
}

impl ::std::default::Default for __H5O_hdr_info_t__space {
    fn default() -> __H5O_hdr_info_t__space { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct __H5O_hdr_info_t__mesg {
    pub present: uint64_t,
    pub shared: uint64_t,
}

impl ::std::default::Default for __H5O_hdr_info_t__mesg {
    fn default() -> __H5O_hdr_info_t__mesg { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct H5O_info_t {
    pub fileno: c_ulong,
    pub addr: haddr_t,
    pub _type: H5O_type_t,
    pub rc: c_uint,
    pub atime: time_t,
    pub mtime: time_t,
    pub ctime: time_t,
    pub btime: time_t,
    pub num_attrs: hsize_t,
    pub hdr: H5O_hdr_info_t,
    pub meta_size: __H5O_info_t__meta_size,
}

impl ::std::default::Default for H5O_info_t {
    fn default() -> H5O_info_t { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct __H5O_info_t__meta_size {
    pub obj: H5_ih_info_t,
    pub attr: H5_ih_info_t,
}

impl ::std::default::Default for __H5O_info_t__meta_size {
    fn default() -> __H5O_info_t__meta_size { unsafe { ::std::mem::zeroed() } }
}

pub type H5O_msg_crt_idx_t = uint32_t;

pub type H5O_iterate_t = Option<extern fn (obj: hid_t, name: *const c_char, info: *const H5O_info_t,
                                           op_data: *mut c_void) -> herr_t>;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5O_mcdt_search_ret_t {
    H5O_MCDT_SEARCH_ERROR = -1,
    H5O_MCDT_SEARCH_CONT  = 0,
    H5O_MCDT_SEARCH_STOP  = 1,
}

pub type H5O_mcdt_search_cb_t = Option<extern fn(op_data: *mut c_void) -> H5O_mcdt_search_ret_t>;

extern {
    pub fn H5Oopen(loc_id: hid_t, name: *const c_char, lapl_id: hid_t) -> hid_t;
    pub fn H5Oopen_by_addr(loc_id: hid_t, addr: haddr_t) -> hid_t;
    pub fn H5Oopen_by_idx(loc_id: hid_t, group_name: *const c_char, idx_type: H5_index_t, order:
                          H5_iter_order_t, n: hsize_t, lapl_id: hid_t) -> hid_t;
    pub fn H5Oexists_by_name(loc_id: hid_t, name: *const c_char, lapl_id: hid_t) -> htri_t;
    pub fn H5Oget_info(loc_id: hid_t, oinfo: *mut H5O_info_t) -> herr_t;
    pub fn H5Oget_info_by_name(loc_id: hid_t, name: *const c_char, oinfo: *mut H5O_info_t, lapl_id:
                               hid_t) -> herr_t;
    pub fn H5Oget_info_by_idx(loc_id: hid_t, group_name: *const c_char, idx_type: H5_index_t, order:
                              H5_iter_order_t, n: hsize_t, oinfo: *mut H5O_info_t, lapl_id: hid_t)
                              -> herr_t;
    pub fn H5Olink(obj_id: hid_t, new_loc_id: hid_t, new_name: *const c_char, lcpl_id: hid_t,
                   lapl_id: hid_t) -> herr_t;
    pub fn H5Oincr_refcount(object_id: hid_t) -> herr_t;
    pub fn H5Odecr_refcount(object_id: hid_t) -> herr_t;
    pub fn H5Ocopy(src_loc_id: hid_t, src_name: *const c_char, dst_loc_id: hid_t, dst_name: *const
                   c_char, ocpypl_id: hid_t, lcpl_id: hid_t) -> herr_t;
    pub fn H5Oset_comment(obj_id: hid_t, comment: *const c_char) -> herr_t;
    pub fn H5Oset_comment_by_name(loc_id: hid_t, name: *const c_char, comment: *const c_char,
                                  lapl_id: hid_t) -> herr_t;
    pub fn H5Oget_comment(obj_id: hid_t, comment: *mut c_char, bufsize: size_t) -> ssize_t;
    pub fn H5Oget_comment_by_name(loc_id: hid_t, name: *const c_char, comment: *mut c_char, bufsize:
                                  size_t, lapl_id: hid_t) -> ssize_t;
    pub fn H5Ovisit(obj_id: hid_t, idx_type: H5_index_t, order: H5_iter_order_t, op: H5O_iterate_t,
                    op_data: *mut c_void) -> herr_t;
    pub fn H5Ovisit_by_name(loc_id: hid_t, obj_name: *const c_char, idx_type: H5_index_t, order:
                            H5_iter_order_t, op: H5O_iterate_t, op_data: *mut c_void, lapl_id:
                            hid_t) -> herr_t;
    pub fn H5Oclose(object_id: hid_t) -> herr_t;
}
