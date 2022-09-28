//! Creating and manipulating dataspaces in which to store elements of a dataset
pub use self::H5S_class_t::*;
pub use self::H5S_sel_type::*;
pub use self::H5S_seloper_t::*;
#[cfg(not(feature = "1.12.0"))]
pub use self::H5Sencode1 as H5Sencode;
#[cfg(feature = "1.12.0")]
pub use self::H5Sencode2 as H5Sencode;

use crate::internal_prelude::*;

pub const H5S_ALL: hid_t = 0;

pub const H5S_UNLIMITED: hsize_t = !0;

pub const H5S_MAX_RANK: c_uint = 32;

pub const H5S_SEL_ITER_GET_SEQ_LIST_SORTED: c_uint = 0x0001;
pub const H5S_SEL_ITER_SHARE_WITH_DATASPACE: c_uint = 0x0002;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug)]
pub enum H5S_class_t {
    H5S_NO_CLASS = -1,
    H5S_SCALAR = 0,
    H5S_SIMPLE = 1,
    H5S_NULL = 2,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug)]
pub enum H5S_seloper_t {
    H5S_SELECT_NOOP = -1,
    H5S_SELECT_SET = 0,
    H5S_SELECT_OR = 1,
    H5S_SELECT_AND = 2,
    H5S_SELECT_XOR = 3,
    H5S_SELECT_NOTB = 4,
    H5S_SELECT_NOTA = 5,
    H5S_SELECT_APPEND = 6,
    H5S_SELECT_PREPEND = 7,
    H5S_SELECT_INVALID = 8,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug)]
pub enum H5S_sel_type {
    H5S_SEL_ERROR = -1,
    H5S_SEL_NONE = 0,
    H5S_SEL_POINTS = 1,
    H5S_SEL_HYPERSLABS = 2,
    H5S_SEL_ALL = 3,
    H5S_SEL_N = 4,
}

extern "C" {
    pub fn H5Screate(type_: H5S_class_t) -> hid_t;
    pub fn H5Screate_simple(rank: c_int, dims: *const hsize_t, maxdims: *const hsize_t) -> hid_t;
    pub fn H5Sset_extent_simple(
        space_id: hid_t, rank: c_int, dims: *const hsize_t, max: *const hsize_t,
    ) -> herr_t;
    pub fn H5Scopy(space_id: hid_t) -> hid_t;
    pub fn H5Sclose(space_id: hid_t) -> herr_t;
    #[cfg_attr(
        feature = "1.12.0",
        deprecated(note = "deprecated in HDF5 1.12.0, use H5Sencode2()")
    )]
    #[cfg_attr(not(feature = "1.12.0"), link_name = "H5Sencode")]
    pub fn H5Sencode1(obj_id: hid_t, buf: *mut c_void, nalloc: *mut size_t) -> herr_t;
    #[cfg(feature = "1.12.0")]
    pub fn H5Sencode2(
        obj_id: hid_t, buf: *mut c_void, nalloc: *mut size_t, fapl_id: hid_t,
    ) -> herr_t;
    pub fn H5Sdecode(buf: *const c_void) -> hid_t;
    pub fn H5Sget_simple_extent_npoints(space_id: hid_t) -> hssize_t;
    pub fn H5Sget_simple_extent_ndims(space_id: hid_t) -> c_int;
    pub fn H5Sget_simple_extent_dims(
        space_id: hid_t, dims: *mut hsize_t, maxdims: *mut hsize_t,
    ) -> c_int;
    pub fn H5Sis_simple(space_id: hid_t) -> htri_t;
    pub fn H5Sget_select_npoints(spaceid: hid_t) -> hssize_t;
    pub fn H5Sselect_hyperslab(
        space_id: hid_t, op: H5S_seloper_t, start: *const hsize_t, _stride: *const hsize_t,
        count: *const hsize_t, _block: *const hsize_t,
    ) -> herr_t;
    pub fn H5Sselect_elements(
        space_id: hid_t, op: H5S_seloper_t, num_elem: size_t, coord: *const hsize_t,
    ) -> herr_t;
    pub fn H5Sget_simple_extent_type(space_id: hid_t) -> H5S_class_t;
    pub fn H5Sset_extent_none(space_id: hid_t) -> herr_t;
    pub fn H5Sextent_copy(dst_id: hid_t, src_id: hid_t) -> herr_t;
    pub fn H5Sextent_equal(sid1: hid_t, sid2: hid_t) -> htri_t;
    pub fn H5Sselect_all(spaceid: hid_t) -> herr_t;
    pub fn H5Sselect_none(spaceid: hid_t) -> herr_t;
    pub fn H5Soffset_simple(space_id: hid_t, offset: *const hssize_t) -> herr_t;
    pub fn H5Sselect_valid(spaceid: hid_t) -> htri_t;
    pub fn H5Sget_select_hyper_nblocks(spaceid: hid_t) -> hssize_t;
    pub fn H5Sget_select_elem_npoints(spaceid: hid_t) -> hssize_t;
    pub fn H5Sget_select_hyper_blocklist(
        spaceid: hid_t, startblock: hsize_t, numblocks: hsize_t, buf: *mut hsize_t,
    ) -> herr_t;
    pub fn H5Sget_select_elem_pointlist(
        spaceid: hid_t, startpoint: hsize_t, numpoints: hsize_t, buf: *mut hsize_t,
    ) -> herr_t;
    pub fn H5Sget_select_bounds(spaceid: hid_t, start: *mut hsize_t, end: *mut hsize_t) -> herr_t;
    pub fn H5Sget_select_type(spaceid: hid_t) -> H5S_sel_type;
}

#[cfg(feature = "1.10.0")]
extern "C" {
    pub fn H5Sis_regular_hyperslab(spaceid: hid_t) -> htri_t;
    pub fn H5Sget_regular_hyperslab(
        spaceid: hid_t, start: *mut hsize_t, stride: *mut hsize_t, count: *mut hsize_t,
        block: *mut hsize_t,
    ) -> htri_t;
}

#[cfg(any(feature = "1.12.0", feature = "1.10.7"))]
extern "C" {
    pub fn H5Scombine_hyperslab(
        space_id: hid_t, op: H5S_seloper_t, start: *const hsize_t, stride: *const hsize_t,
        count: *const hsize_t, block: *const hsize_t,
    ) -> hid_t;
    pub fn H5Scombine_select(space1_id: hid_t, op: H5S_seloper_t, space2_id: hid_t) -> hid_t;
    pub fn H5Smodify_select(space1_id: hid_t, op: H5S_seloper_t, space2_id: hid_t) -> herr_t;
    pub fn H5Sselect_adjust(space_id: hid_t, offset: *const hssize_t) -> herr_t;
    pub fn H5Sselect_copy(dst_id: hid_t, src_id: hid_t) -> herr_t;
    pub fn H5Sselect_intersect_block(
        space_id: hid_t, start: *const hsize_t, end: *const hsize_t,
    ) -> htri_t;
    pub fn H5Sselect_project_intersection(
        src_space_id: hid_t, dst_space_id: hid_t, src_intersect_space_id: hid_t,
    ) -> hid_t;
    pub fn H5Sselect_shape_same(space1_id: hid_t, space2_id: hid_t) -> htri_t;
}

#[cfg(feature = "1.12.0")]
extern "C" {
    pub fn H5Ssel_iter_close(sel_iter_id: hid_t) -> herr_t;
    pub fn H5Ssel_iter_create(space_id: hid_t, elmt_size: size_t, flags: c_uint) -> hid_t;
    pub fn H5Ssel_iter_get_seq_list(
        sel_iter_id: hid_t, maxseq: size_t, maxbytes: size_t, nseq: *mut size_t,
        nbytes: *mut size_t, off: *mut hsize_t, len: *mut size_t,
    ) -> herr_t;
}

#[cfg(feature = "1.12.1")]
extern "C" {
    pub fn H5Ssel_iter_reset(sel_iter_id: hid_t, space_id: hid_t) -> hid_t;
}
