pub use self::H5D_layout_t::*;
pub use self::H5D_alloc_time_t::*;
pub use self::H5D_space_status_t::*;
pub use self::H5D_fill_time_t::*;
pub use self::H5D_fill_value_t::*;

use libc::{c_uint, c_void, c_char, c_float, size_t};

use ffi::types::{hid_t, herr_t, hsize_t, haddr_t};

pub const H5D_CHUNK_CACHE_NSLOTS_DEFAULT: size_t = !0;
pub const H5D_CHUNK_CACHE_NBYTES_DEFAULT: size_t = !0;

pub const H5D_CHUNK_CACHE_W0_DEFAULT: c_float = -1.0;

pub const H5D_XFER_DIRECT_CHUNK_WRITE_FLAG_NAME:     &'static str = "direct_chunk_flag";
pub const H5D_XFER_DIRECT_CHUNK_WRITE_FILTERS_NAME:  &'static str = "direct_chunk_filters";
pub const H5D_XFER_DIRECT_CHUNK_WRITE_OFFSET_NAME:   &'static str = "direct_chunk_offset";
pub const H5D_XFER_DIRECT_CHUNK_WRITE_DATASIZE_NAME: &'static str = "direct_chunk_datasize";

#[repr(C)]
#[derive(Copy, Clone)]
pub enum H5D_layout_t {
    H5D_LAYOUT_ERROR = -1,
    H5D_COMPACT      = 0,
    H5D_CONTIGUOUS   = 1,
    H5D_CHUNKED      = 2,
    H5D_NLAYOUTS     = 3,
}

pub type H5D_chunk_index_t = c_uint;
pub const H5D_CHUNK_BTREE: H5D_chunk_index_t = 0;

#[repr(C)]
#[derive(Copy, Clone)]
pub enum H5D_alloc_time_t {
    H5D_ALLOC_TIME_ERROR   = -1,
    H5D_ALLOC_TIME_DEFAULT = 0,
    H5D_ALLOC_TIME_EARLY   = 1,
    H5D_ALLOC_TIME_LATE    = 2,
    H5D_ALLOC_TIME_INCR    = 3,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum H5D_space_status_t {
    H5D_SPACE_STATUS_ERROR          = -1,
    H5D_SPACE_STATUS_NOT_ALLOCATED  = 0,
    H5D_SPACE_STATUS_PART_ALLOCATED = 1,
    H5D_SPACE_STATUS_ALLOCATED      = 2,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum H5D_fill_time_t {
    H5D_FILL_TIME_ERROR = -1,
    H5D_FILL_TIME_ALLOC = 0,
    H5D_FILL_TIME_NEVER = 1,
    H5D_FILL_TIME_IFSET = 2,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum H5D_fill_value_t {
    H5D_FILL_VALUE_ERROR        = -1,
    H5D_FILL_VALUE_UNDEFINED    = 0,
    H5D_FILL_VALUE_DEFAULT      = 1,
    H5D_FILL_VALUE_USER_DEFINED = 2,
}

pub type H5D_operator_t = Option<extern fn (elem: *mut c_void, type_id: hid_t, ndim: c_uint, point:
                                            *const hsize_t, operator_data: *mut c_void) -> herr_t>;
pub type H5D_scatter_func_t = Option<extern fn (src_buf: *mut *const c_void, src_buf_bytes_used:
                                                *mut size_t, op_data: *mut c_void) -> herr_t>;
pub type H5D_gather_func_t = Option<extern fn (dst_buf: *const c_void, dst_buf_bytes_used: size_t,
                                               op_data: *mut c_void) -> herr_t>;

#[link(name = "hdf5")]
extern {
    pub fn H5Dcreate2(loc_id: hid_t, name: *const c_char, type_id: hid_t, space_id: hid_t, lcpl_id:
                      hid_t, dcpl_id: hid_t, dapl_id: hid_t) -> hid_t;
    pub fn H5Dcreate_anon(file_id: hid_t, type_id: hid_t, space_id: hid_t, plist_id: hid_t, dapl_id:
                          hid_t) -> hid_t;
    pub fn H5Dopen2(file_id: hid_t, name: *const c_char, dapl_id: hid_t) -> hid_t;
    pub fn H5Dclose(dset_id: hid_t) -> herr_t;
    pub fn H5Dget_space(dset_id: hid_t) -> hid_t;
    pub fn H5Dget_space_status(dset_id: hid_t, allocation: *mut H5D_space_status_t) -> herr_t;
    pub fn H5Dget_type(dset_id: hid_t) -> hid_t;
    pub fn H5Dget_create_plist(dset_id: hid_t) -> hid_t;
    pub fn H5Dget_access_plist(dset_id: hid_t) -> hid_t;
    pub fn H5Dget_storage_size(dset_id: hid_t) -> hsize_t;
    pub fn H5Dget_offset(dset_id: hid_t) -> haddr_t;
    pub fn H5Dread(dset_id: hid_t, mem_type_id: hid_t, mem_space_id: hid_t, file_space_id: hid_t,
                   plist_id: hid_t, buf: *mut c_void) -> herr_t;
    pub fn H5Dwrite(dset_id: hid_t, mem_type_id: hid_t, mem_space_id: hid_t, file_space_id: hid_t,
                    plist_id: hid_t, buf: *const c_void) -> herr_t;
    pub fn H5Diterate(buf: *mut c_void, type_id: hid_t, space_id: hid_t, op: H5D_operator_t,
                      operator_data: *mut c_void) -> herr_t;
    pub fn H5Dvlen_reclaim(type_id: hid_t, space_id: hid_t, plist_id: hid_t, buf: *mut c_void) ->
                           herr_t;
    pub fn H5Dvlen_get_buf_size(dataset_id: hid_t, type_id: hid_t, space_id: hid_t, size: *mut
                                hsize_t) -> herr_t;
    pub fn H5Dfill(fill: *const c_void, fill_type: hid_t, buf: *mut c_void, buf_type: hid_t, space:
                   hid_t) -> herr_t;
    pub fn H5Dset_extent(dset_id: hid_t, size: *const hsize_t) -> herr_t;
    pub fn H5Dscatter(op: H5D_scatter_func_t, op_data: *mut c_void, type_id: hid_t, dst_space_id:
                      hid_t, dst_buf: *mut c_void) -> herr_t;
    pub fn H5Dgather(src_space_id: hid_t, src_buf: *const c_void, type_id: hid_t, dst_buf_size:
                     size_t, dst_buf: *mut c_void, op: H5D_gather_func_t, op_data: *mut c_void) ->
                     herr_t;
    pub fn H5Ddebug(dset_id: hid_t) -> herr_t;
}
