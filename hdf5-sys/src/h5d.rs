pub use self::H5D_alloc_time_t::*;
pub use self::H5D_fill_time_t::*;
pub use self::H5D_fill_value_t::*;
pub use self::H5D_layout_t::*;
pub use self::H5D_mpio_actual_chunk_opt_mode_t::*;
pub use self::H5D_mpio_actual_io_mode_t::*;
pub use self::H5D_mpio_no_collective_cause_t::*;
pub use self::H5D_space_status_t::*;

use crate::internal_prelude::*;

pub const H5D_CHUNK_CACHE_NSLOTS_DEFAULT: size_t = !0;
pub const H5D_CHUNK_CACHE_NBYTES_DEFAULT: size_t = !0;

pub const H5D_CHUNK_CACHE_W0_DEFAULT: c_float = -1.0;

#[cfg(not(hdf5_1_10_0))]
#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5D_layout_t {
    H5D_LAYOUT_ERROR = -1,
    H5D_COMPACT = 0,
    H5D_CONTIGUOUS = 1,
    H5D_CHUNKED = 2,
    H5D_NLAYOUTS = 3,
}

impl Default for H5D_layout_t {
    fn default() -> Self {
        H5D_layout_t::H5D_CONTIGUOUS
    }
}

pub type H5D_chunk_index_t = c_uint;

pub const H5D_CHUNK_BTREE: H5D_chunk_index_t = 0;
pub const H5D_CHUNK_IDX_BTREE: H5D_chunk_index_t = H5D_CHUNK_BTREE;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5D_alloc_time_t {
    H5D_ALLOC_TIME_ERROR = -1,
    H5D_ALLOC_TIME_DEFAULT = 0,
    H5D_ALLOC_TIME_EARLY = 1,
    H5D_ALLOC_TIME_LATE = 2,
    H5D_ALLOC_TIME_INCR = 3,
}

impl Default for H5D_alloc_time_t {
    fn default() -> Self {
        H5D_alloc_time_t::H5D_ALLOC_TIME_DEFAULT
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5D_space_status_t {
    H5D_SPACE_STATUS_ERROR = -1,
    H5D_SPACE_STATUS_NOT_ALLOCATED = 0,
    H5D_SPACE_STATUS_PART_ALLOCATED = 1,
    H5D_SPACE_STATUS_ALLOCATED = 2,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5D_fill_time_t {
    H5D_FILL_TIME_ERROR = -1,
    H5D_FILL_TIME_ALLOC = 0,
    H5D_FILL_TIME_NEVER = 1,
    H5D_FILL_TIME_IFSET = 2,
}

impl Default for H5D_fill_time_t {
    fn default() -> Self {
        H5D_fill_time_t::H5D_FILL_TIME_IFSET
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5D_fill_value_t {
    H5D_FILL_VALUE_ERROR = -1,
    H5D_FILL_VALUE_UNDEFINED = 0,
    H5D_FILL_VALUE_DEFAULT = 1,
    H5D_FILL_VALUE_USER_DEFINED = 2,
}

impl Default for H5D_fill_value_t {
    fn default() -> Self {
        H5D_fill_value_t::H5D_FILL_VALUE_DEFAULT
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5D_mpio_actual_chunk_opt_mode_t {
    H5D_MPIO_NO_CHUNK_OPTIMIZATION = 0,
    H5D_MPIO_LINK_CHUNK = 1,
    H5D_MPIO_MULTI_CHUNK = 2,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5D_mpio_actual_io_mode_t {
    H5D_MPIO_NO_COLLECTIVE = 0,
    H5D_MPIO_CHUNK_INDEPENDENT = 1,
    H5D_MPIO_CHUNK_COLLECTIVE = 2,
    H5D_MPIO_CHUNK_MIXED = 3,
    H5D_MPIO_CONTIGUOUS_COLLECTIVE = 4,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5D_mpio_no_collective_cause_t {
    H5D_MPIO_COLLECTIVE = 0,
    H5D_MPIO_SET_INDEPENDENT = 1,
    H5D_MPIO_DATATYPE_CONVERSION = 2,
    H5D_MPIO_DATA_TRANSFORMS = 4,
    H5D_MPIO_MPI_OPT_TYPES_ENV_VAR_DISABLED = 8,
    H5D_MPIO_NOT_SIMPLE_OR_SCALAR_DATASPACES = 16,
    H5D_MPIO_NOT_CONTIGUOUS_OR_CHUNKED_DATASET = 32,
    H5D_MPIO_FILTERS = 64,
}

pub type H5D_operator_t = Option<
    extern "C" fn(
        elem: *mut c_void,
        type_id: hid_t,
        ndim: c_uint,
        point: *const hsize_t,
        operator_data: *mut c_void,
    ) -> herr_t,
>;

#[cfg(hdf5_1_8_11)]
pub type H5D_scatter_func_t = Option<
    extern "C" fn(
        src_buf: *mut *const c_void,
        src_buf_bytes_used: *mut size_t,
        op_data: *mut c_void,
    ) -> herr_t,
>;
#[cfg(hdf5_1_8_11)]
pub type H5D_gather_func_t = Option<
    extern "C" fn(
        dst_buf: *const c_void,
        dst_buf_bytes_used: size_t,
        op_data: *mut c_void,
    ) -> herr_t,
>;

extern "C" {
    pub fn H5Dcreate2(
        loc_id: hid_t, name: *const c_char, type_id: hid_t, space_id: hid_t, lcpl_id: hid_t,
        dcpl_id: hid_t, dapl_id: hid_t,
    ) -> hid_t;
    pub fn H5Dcreate_anon(
        file_id: hid_t, type_id: hid_t, space_id: hid_t, plist_id: hid_t, dapl_id: hid_t,
    ) -> hid_t;
    pub fn H5Dopen2(file_id: hid_t, name: *const c_char, dapl_id: hid_t) -> hid_t;
    pub fn H5Dclose(dset_id: hid_t) -> herr_t;
    pub fn H5Dget_space(dset_id: hid_t) -> hid_t;
    pub fn H5Dget_space_status(dset_id: hid_t, allocation: *mut H5D_space_status_t) -> herr_t;
    pub fn H5Dget_type(dset_id: hid_t) -> hid_t;
    pub fn H5Dget_create_plist(dset_id: hid_t) -> hid_t;
    pub fn H5Dget_access_plist(dset_id: hid_t) -> hid_t;
    pub fn H5Dget_storage_size(dset_id: hid_t) -> hsize_t;
    pub fn H5Dget_offset(dset_id: hid_t) -> haddr_t;
    pub fn H5Dread(
        dset_id: hid_t, mem_type_id: hid_t, mem_space_id: hid_t, file_space_id: hid_t,
        plist_id: hid_t, buf: *mut c_void,
    ) -> herr_t;
    pub fn H5Dwrite(
        dset_id: hid_t, mem_type_id: hid_t, mem_space_id: hid_t, file_space_id: hid_t,
        plist_id: hid_t, buf: *const c_void,
    ) -> herr_t;
    pub fn H5Diterate(
        buf: *mut c_void, type_id: hid_t, space_id: hid_t, op: H5D_operator_t,
        operator_data: *mut c_void,
    ) -> herr_t;
    pub fn H5Dvlen_reclaim(
        type_id: hid_t, space_id: hid_t, plist_id: hid_t, buf: *mut c_void,
    ) -> herr_t;
    pub fn H5Dvlen_get_buf_size(
        dataset_id: hid_t, type_id: hid_t, space_id: hid_t, size: *mut hsize_t,
    ) -> herr_t;
    pub fn H5Dfill(
        fill: *const c_void, fill_type: hid_t, buf: *mut c_void, buf_type: hid_t, space: hid_t,
    ) -> herr_t;
    pub fn H5Dset_extent(dset_id: hid_t, size: *const hsize_t) -> herr_t;
    pub fn H5Ddebug(dset_id: hid_t) -> herr_t;
}

#[cfg(hdf5_1_8_11)]
extern "C" {
    pub fn H5Dscatter(
        op: H5D_scatter_func_t, op_data: *mut c_void, type_id: hid_t, dst_space_id: hid_t,
        dst_buf: *mut c_void,
    ) -> herr_t;
    pub fn H5Dgather(
        src_space_id: hid_t, src_buf: *const c_void, type_id: hid_t, dst_buf_size: size_t,
        dst_buf: *mut c_void, op: H5D_gather_func_t, op_data: *mut c_void,
    ) -> herr_t;
}

#[cfg(hdf5_1_10_0)]
mod hdf5_1_10_0 {
    use super::*;

    #[repr(C)]
    #[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
    pub enum H5D_layout_t {
        H5D_LAYOUT_ERROR = -1,
        H5D_COMPACT = 0,
        H5D_CONTIGUOUS = 1,
        H5D_CHUNKED = 2,
        H5D_VIRTUAL = 3,
        H5D_NLAYOUTS = 4,
    }

    #[repr(C)]
    #[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
    pub enum H5D_vds_view_t {
        H5D_VDS_ERROR = -1,
        H5D_VDS_FIRST_MISSING = 0,
        H5D_VDS_LAST_AVAILABLE = 1,
    }

    impl Default for H5D_vds_view_t {
        fn default() -> Self {
            H5D_vds_view_t::H5D_VDS_LAST_AVAILABLE
        }
    }

    pub const H5D_CHUNK_DONT_FILTER_PARTIAL_CHUNKS: c_uint = 0x0002;

    pub type H5D_append_cb_t = Option<
        unsafe extern "C" fn(
            dataset_id: hid_t,
            cur_dims: *mut hsize_t,
            op_data: *mut c_void,
        ) -> herr_t,
    >;

    extern "C" {
        pub fn H5Dflush(dset_id: hid_t) -> herr_t;
        pub fn H5Drefresh(dset_id: hid_t) -> herr_t;
        pub fn H5Dformat_convert(dset_id: hid_t) -> herr_t;
        pub fn H5Dget_chunk_index_type(did: hid_t, idx_type: *mut H5D_chunk_index_t) -> herr_t;
    }
}

#[cfg(hdf5_1_10_0)]
pub use self::hdf5_1_10_0::*;

#[cfg(hdf5_1_10_3)]
extern "C" {
    pub fn H5Dread_chunk(
        dset_id: hid_t, dxpl_id: hid_t, offset: *const hsize_t, filters: *mut u32, buf: *mut c_void,
    ) -> herr_t;
    pub fn H5Dwrite_chunk(
        dset_id: hid_t, dxpl_id: hid_t, filters: u32, offset: *const hsize_t, data_size: size_t,
        buf: *const c_void,
    ) -> herr_t;
}

#[cfg(hdf5_1_10_5)]
extern "C" {
    pub fn H5Dget_chunk_info(
        dset_id: hid_t, fspace_id: hid_t, index: hsize_t, offset: *mut hsize_t,
        filter_mask: *mut c_uint, addr: *mut haddr_t, size: *mut hsize_t,
    ) -> herr_t;
    pub fn H5Dget_chunk_info_by_coord(
        dset_id: hid_t, offset: *const hsize_t, filter_mask: *mut c_uint, addr: *mut haddr_t,
        size: *mut hsize_t,
    ) -> herr_t;
    pub fn H5Dget_num_chunks(dset_id: hid_t, fspace_id: hid_t, nchunks: *mut hsize_t) -> herr_t;
}
