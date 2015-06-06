pub use self::H5T_class_t::*;
pub use self::H5T_order_t::*;
pub use self::H5T_sign_t::*;
pub use self::H5T_norm_t::*;
pub use self::H5T_cset_t::*;
pub use self::H5T_str_t::*;
pub use self::H5T_pad_t::*;
pub use self::H5T_cmd_t::*;
pub use self::H5T_bkg_t::*;
pub use self::H5T_pers_t::*;
pub use self::H5T_direction_t::*;
pub use self::H5T_conv_except_t::*;
pub use self::H5T_conv_ret_t::*;

use libc::{c_int, c_uint, c_void, c_char, size_t};

use h5::{herr_t, htri_t, hsize_t, hbool_t};
use h5i::hid_t;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5T_class_t {
    H5T_NO_CLASS = -1,
    H5T_INTEGER = 0,
    H5T_FLOAT = 1,
    H5T_TIME = 2,
    H5T_STRING = 3,
    H5T_BITFIELD = 4,
    H5T_OPAQUE = 5,
    H5T_COMPOUND = 6,
    H5T_REFERENCE = 7,
    H5T_ENUM = 8,
    H5T_VLEN = 9,
    H5T_ARRAY = 10,
    H5T_NCLASSES = 11,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5T_order_t {
    H5T_ORDER_ERROR = -1,
    H5T_ORDER_LE = 0,
    H5T_ORDER_BE = 1,
    H5T_ORDER_VAX = 2,
    H5T_ORDER_MIXED = 3,
    H5T_ORDER_NONE = 4,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5T_sign_t {
    H5T_SGN_ERROR = -1,
    H5T_SGN_NONE = 0,
    H5T_SGN_2 = 1,
    H5T_NSGN = 2,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5T_norm_t {
    H5T_NORM_ERROR = -1,
    H5T_NORM_IMPLIED = 0,
    H5T_NORM_MSBSET = 1,
    H5T_NORM_NONE = 2,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5T_cset_t {
    H5T_CSET_ERROR = -1,
    H5T_CSET_ASCII = 0,
    H5T_CSET_UTF8 = 1,
    H5T_CSET_RESERVED_2 = 2,
    H5T_CSET_RESERVED_3 = 3,
    H5T_CSET_RESERVED_4 = 4,
    H5T_CSET_RESERVED_5 = 5,
    H5T_CSET_RESERVED_6 = 6,
    H5T_CSET_RESERVED_7 = 7,
    H5T_CSET_RESERVED_8 = 8,
    H5T_CSET_RESERVED_9 = 9,
    H5T_CSET_RESERVED_10 = 10,
    H5T_CSET_RESERVED_11 = 11,
    H5T_CSET_RESERVED_12 = 12,
    H5T_CSET_RESERVED_13 = 13,
    H5T_CSET_RESERVED_14 = 14,
    H5T_CSET_RESERVED_15 = 15,
}

pub const H5T_NCSET: H5T_cset_t = H5T_CSET_RESERVED_2;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5T_str_t {
    H5T_STR_ERROR = -1,
    H5T_STR_NULLTERM = 0,
    H5T_STR_NULLPAD = 1,
    H5T_STR_SPACEPAD = 2,
    H5T_STR_RESERVED_3 = 3,
    H5T_STR_RESERVED_4 = 4,
    H5T_STR_RESERVED_5 = 5,
    H5T_STR_RESERVED_6 = 6,
    H5T_STR_RESERVED_7 = 7,
    H5T_STR_RESERVED_8 = 8,
    H5T_STR_RESERVED_9 = 9,
    H5T_STR_RESERVED_10 = 10,
    H5T_STR_RESERVED_11 = 11,
    H5T_STR_RESERVED_12 = 12,
    H5T_STR_RESERVED_13 = 13,
    H5T_STR_RESERVED_14 = 14,
    H5T_STR_RESERVED_15 = 15,
}

pub const H5T_NSTR: H5T_str_t = H5T_STR_RESERVED_3;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5T_pad_t {
    H5T_PAD_ERROR = -1,
    H5T_PAD_ZERO = 0,
    H5T_PAD_ONE = 1,
    H5T_PAD_BACKGROUND = 2,
    H5T_NPAD = 3,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5T_cmd_t {
    H5T_CONV_INIT = 0,
    H5T_CONV_CONV = 1,
    H5T_CONV_FREE = 2,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5T_bkg_t {
    H5T_BKG_NO = 0,
    H5T_BKG_TEMP = 1,
    H5T_BKG_YES = 2,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct H5T_cdata_t {
    pub command: H5T_cmd_t,
    pub need_bkg: H5T_bkg_t,
    pub recalc: hbool_t,
    pub _priv: *mut c_void,
}

impl ::std::default::Default for H5T_cdata_t {
    fn default() -> H5T_cdata_t { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5T_pers_t {
    H5T_PERS_DONTCARE = -1,
    H5T_PERS_HARD = 0,
    H5T_PERS_SOFT = 1,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5T_direction_t {
    H5T_DIR_DEFAULT = 0,
    H5T_DIR_ASCEND = 1,
    H5T_DIR_DESCEND = 2,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5T_conv_except_t {
    H5T_CONV_EXCEPT_RANGE_HI = 0,
    H5T_CONV_EXCEPT_RANGE_LOW = 1,
    H5T_CONV_EXCEPT_PRECISION = 2,
    H5T_CONV_EXCEPT_TRUNCATE = 3,
    H5T_CONV_EXCEPT_PINF = 4,
    H5T_CONV_EXCEPT_NINF = 5,
    H5T_CONV_EXCEPT_NAN = 6,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5T_conv_ret_t {
    H5T_CONV_ABORT = -1,
    H5T_CONV_UNHANDLED = 0,
    H5T_CONV_HANDLED = 1,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct hvl_t {
    pub len: size_t,
    pub p: *mut c_void,
}

impl ::std::default::Default for hvl_t {
    fn default() -> hvl_t { unsafe { ::std::mem::zeroed() } }
}

pub const H5T_VARIABLE: size_t = !0;

pub const H5T_OPAQUE_TAG_MAX: c_uint = 256;

pub type H5T_conv_t = Option<extern fn (src_id: hid_t, dst_id: hid_t, cdata: *mut H5T_cdata_t,
                                        nelmts: size_t, buf_stride: size_t, bkg_stride: size_t, buf:
                                        *mut c_void, bkg: *mut c_void, dset_xfer_plist: hid_t) ->
                                        herr_t>;
pub type H5T_conv_except_func_t = Option<extern fn (except_type: H5T_conv_except_t, src_id:
                                                    hid_t, dst_id: hid_t, src_buf: *mut c_void,
                                                    dst_buf: *mut c_void, user_data: *mut c_void) ->
                                                    H5T_conv_ret_t>;

extern {
    pub fn H5Tcreate(_type: H5T_class_t, size: size_t) -> hid_t;
    pub fn H5Tcopy(type_id: hid_t) -> hid_t;
    pub fn H5Tclose(type_id: hid_t) -> herr_t;
    pub fn H5Tequal(type1_id: hid_t, type2_id: hid_t) -> htri_t;
    pub fn H5Tlock(type_id: hid_t) -> herr_t;
    pub fn H5Tcommit2(loc_id: hid_t, name: *const c_char, type_id: hid_t, lcpl_id: hid_t, tcpl_id:
                      hid_t, tapl_id: hid_t) -> herr_t;
    pub fn H5Topen2(loc_id: hid_t, name: *const c_char, tapl_id: hid_t) -> hid_t;
    pub fn H5Tcommit_anon(loc_id: hid_t, type_id: hid_t, tcpl_id: hid_t, tapl_id: hid_t) -> herr_t;
    pub fn H5Tget_create_plist(type_id: hid_t) -> hid_t;
    pub fn H5Tcommitted(type_id: hid_t) -> htri_t;
    pub fn H5Tencode(obj_id: hid_t, buf: *mut c_void, nalloc: *mut size_t) -> herr_t;
    pub fn H5Tdecode(buf: *const c_void) -> hid_t;
    pub fn H5Tinsert(parent_id: hid_t, name: *const c_char, offset: size_t, member_id: hid_t) ->
                     herr_t;
    pub fn H5Tpack(type_id: hid_t) -> herr_t;
    pub fn H5Tenum_create(base_id: hid_t) -> hid_t;
    pub fn H5Tenum_insert(_type: hid_t, name: *const c_char, value: *const c_void) -> herr_t;
    pub fn H5Tenum_nameof(_type: hid_t, value: *const c_void, name: *mut c_char, size: size_t) ->
                          herr_t;
    pub fn H5Tenum_valueof(_type: hid_t, name: *const c_char, value: *mut c_void) -> herr_t;
    pub fn H5Tvlen_create(base_id: hid_t) -> hid_t;
    pub fn H5Tarray_create2(base_id: hid_t, ndims: c_uint, dim: *const hsize_t) -> hid_t;
    pub fn H5Tget_array_ndims(type_id: hid_t) -> c_int;
    pub fn H5Tget_array_dims2(type_id: hid_t, dims: *mut hsize_t) -> c_int;
    pub fn H5Tset_tag(_type: hid_t, tag: *const c_char) -> herr_t;
    pub fn H5Tget_tag(_type: hid_t) -> *mut c_char;
    pub fn H5Tget_super(_type: hid_t) -> hid_t;
    pub fn H5Tget_class(type_id: hid_t) -> H5T_class_t;
    pub fn H5Tdetect_class(type_id: hid_t, cls: H5T_class_t) -> htri_t;
    pub fn H5Tget_size(type_id: hid_t) -> size_t;
    pub fn H5Tget_order(type_id: hid_t) -> H5T_order_t;
    pub fn H5Tget_precision(type_id: hid_t) -> size_t;
    pub fn H5Tget_offset(type_id: hid_t) -> c_int;
    pub fn H5Tget_pad(type_id: hid_t, lsb: *mut H5T_pad_t, msb: *mut H5T_pad_t) -> herr_t;
    pub fn H5Tget_sign(type_id: hid_t) -> H5T_sign_t;
    pub fn H5Tget_fields(type_id: hid_t, spos: *mut size_t, epos: *mut size_t, esize: *mut size_t,
                         mpos: *mut size_t, msize: *mut size_t) -> herr_t;
    pub fn H5Tget_ebias(type_id: hid_t) -> size_t;
    pub fn H5Tget_norm(type_id: hid_t) -> H5T_norm_t;
    pub fn H5Tget_inpad(type_id: hid_t) -> H5T_pad_t;
    pub fn H5Tget_strpad(type_id: hid_t) -> H5T_str_t;
    pub fn H5Tget_nmembers(type_id: hid_t) -> c_int;
    pub fn H5Tget_member_name(type_id: hid_t, membno: c_uint) -> *mut c_char;
    pub fn H5Tget_member_index(type_id: hid_t, name: *const c_char) -> c_int;
    pub fn H5Tget_member_offset(type_id: hid_t, membno: c_uint) -> size_t;
    pub fn H5Tget_member_class(type_id: hid_t, membno: c_uint) -> H5T_class_t;
    pub fn H5Tget_member_type(type_id: hid_t, membno: c_uint) -> hid_t;
    pub fn H5Tget_member_value(type_id: hid_t, membno: c_uint, value: *mut c_void) -> herr_t;
    pub fn H5Tget_cset(type_id: hid_t) -> H5T_cset_t;
    pub fn H5Tis_variable_str(type_id: hid_t) -> htri_t;
    pub fn H5Tget_native_type(type_id: hid_t, direction: H5T_direction_t) -> hid_t;
    pub fn H5Tset_size(type_id: hid_t, size: size_t) -> herr_t;
    pub fn H5Tset_order(type_id: hid_t, order: H5T_order_t) -> herr_t;
    pub fn H5Tset_precision(type_id: hid_t, prec: size_t) -> herr_t;
    pub fn H5Tset_offset(type_id: hid_t, offset: size_t) -> herr_t;
    pub fn H5Tset_pad(type_id: hid_t, lsb: H5T_pad_t, msb: H5T_pad_t) -> herr_t;
    pub fn H5Tset_sign(type_id: hid_t, sign: H5T_sign_t) -> herr_t;
    pub fn H5Tset_fields(type_id: hid_t, spos: size_t, epos: size_t, esize: size_t, mpos: size_t,
                         msize: size_t) -> herr_t;
    pub fn H5Tset_ebias(type_id: hid_t, ebias: size_t) -> herr_t;
    pub fn H5Tset_norm(type_id: hid_t, norm: H5T_norm_t) -> herr_t;
    pub fn H5Tset_inpad(type_id: hid_t, pad: H5T_pad_t) -> herr_t;
    pub fn H5Tset_cset(type_id: hid_t, cset: H5T_cset_t) -> herr_t;
    pub fn H5Tset_strpad(type_id: hid_t, strpad: H5T_str_t) -> herr_t;
    pub fn H5Tregister(pers: H5T_pers_t, name: *const c_char, src_id: hid_t, dst_id: hid_t, func:
                       H5T_conv_t) -> herr_t;
    pub fn H5Tunregister(pers: H5T_pers_t, name: *const c_char, src_id: hid_t, dst_id: hid_t, func:
                         H5T_conv_t) -> herr_t;
    pub fn H5Tfind(src_id: hid_t, dst_id: hid_t, pcdata: *mut *mut H5T_cdata_t) -> H5T_conv_t;
    pub fn H5Tcompiler_conv(src_id: hid_t, dst_id: hid_t) -> htri_t;
    pub fn H5Tconvert(src_id: hid_t, dst_id: hid_t, nelmts: size_t, buf: *mut c_void, background:
                      *mut c_void, plist_id: hid_t) -> herr_t;
}

extern {
    // Datatypes
    pub static H5T_IEEE_F32BE_g: hid_t;
    pub static H5T_IEEE_F32LE_g: hid_t;
    pub static H5T_IEEE_F64BE_g: hid_t;
    pub static H5T_IEEE_F64LE_g: hid_t;
    pub static H5T_STD_I8BE_g: hid_t;
    pub static H5T_STD_I8LE_g: hid_t;
    pub static H5T_STD_I16BE_g: hid_t;
    pub static H5T_STD_I16LE_g: hid_t;
    pub static H5T_STD_I32BE_g: hid_t;
    pub static H5T_STD_I32LE_g: hid_t;
    pub static H5T_STD_I64BE_g: hid_t;
    pub static H5T_STD_I64LE_g: hid_t;
    pub static H5T_STD_U8BE_g: hid_t;
    pub static H5T_STD_U8LE_g: hid_t;
    pub static H5T_STD_U16BE_g: hid_t;
    pub static H5T_STD_U16LE_g: hid_t;
    pub static H5T_STD_U32BE_g: hid_t;
    pub static H5T_STD_U32LE_g: hid_t;
    pub static H5T_STD_U64BE_g: hid_t;
    pub static H5T_STD_U64LE_g: hid_t;
    pub static H5T_STD_B8BE_g: hid_t;
    pub static H5T_STD_B8LE_g: hid_t;
    pub static H5T_STD_B16BE_g: hid_t;
    pub static H5T_STD_B16LE_g: hid_t;
    pub static H5T_STD_B32BE_g: hid_t;
    pub static H5T_STD_B32LE_g: hid_t;
    pub static H5T_STD_B64BE_g: hid_t;
    pub static H5T_STD_B64LE_g: hid_t;
    pub static H5T_STD_REF_OBJ_g: hid_t;
    pub static H5T_STD_REF_DSETREG_g: hid_t;
    pub static H5T_UNIX_D32BE_g: hid_t;
    pub static H5T_UNIX_D32LE_g: hid_t;
    pub static H5T_UNIX_D64BE_g: hid_t;
    pub static H5T_UNIX_D64LE_g: hid_t;
    pub static H5T_C_S1_g: hid_t;
    pub static H5T_FORTRAN_S1_g: hid_t;
    pub static H5T_VAX_F32_g: hid_t;
    pub static H5T_VAX_F64_g: hid_t;
    pub static H5T_NATIVE_SCHAR_g: hid_t;
    pub static H5T_NATIVE_UCHAR_g: hid_t;
    pub static H5T_NATIVE_SHORT_g: hid_t;
    pub static H5T_NATIVE_USHORT_g: hid_t;
    pub static H5T_NATIVE_INT_g: hid_t;
    pub static H5T_NATIVE_UINT_g: hid_t;
    pub static H5T_NATIVE_LONG_g: hid_t;
    pub static H5T_NATIVE_ULONG_g: hid_t;
    pub static H5T_NATIVE_LLONG_g: hid_t;
    pub static H5T_NATIVE_ULLONG_g: hid_t;
    pub static H5T_NATIVE_FLOAT_g: hid_t;
    pub static H5T_NATIVE_DOUBLE_g: hid_t;
    pub static H5T_NATIVE_LDOUBLE_g: hid_t;
    pub static H5T_NATIVE_B8_g: hid_t;
    pub static H5T_NATIVE_B16_g: hid_t;
    pub static H5T_NATIVE_B32_g: hid_t;
    pub static H5T_NATIVE_B64_g: hid_t;
    pub static H5T_NATIVE_OPAQUE_g: hid_t;
    pub static H5T_NATIVE_HADDR_g: hid_t;
    pub static H5T_NATIVE_HSIZE_g: hid_t;
    pub static H5T_NATIVE_HSSIZE_g: hid_t;
    pub static H5T_NATIVE_HERR_g: hid_t;
    pub static H5T_NATIVE_HBOOL_g: hid_t;
    pub static H5T_NATIVE_INT8_g: hid_t;
    pub static H5T_NATIVE_UINT8_g: hid_t;
    pub static H5T_NATIVE_INT_LEAST8_g: hid_t;
    pub static H5T_NATIVE_UINT_LEAST8_g: hid_t;
    pub static H5T_NATIVE_INT_FAST8_g: hid_t;
    pub static H5T_NATIVE_UINT_FAST8_g: hid_t;
    pub static H5T_NATIVE_INT16_g: hid_t;
    pub static H5T_NATIVE_UINT16_g: hid_t;
    pub static H5T_NATIVE_INT_LEAST16_g: hid_t;
    pub static H5T_NATIVE_UINT_LEAST16_g: hid_t;
    pub static H5T_NATIVE_INT_FAST16_g: hid_t;
    pub static H5T_NATIVE_UINT_FAST16_g: hid_t;
    pub static H5T_NATIVE_INT32_g: hid_t;
    pub static H5T_NATIVE_UINT32_g: hid_t;
    pub static H5T_NATIVE_INT_LEAST32_g: hid_t;
    pub static H5T_NATIVE_UINT_LEAST32_g: hid_t;
    pub static H5T_NATIVE_INT_FAST32_g: hid_t;
    pub static H5T_NATIVE_UINT_FAST32_g: hid_t;
    pub static H5T_NATIVE_INT64_g: hid_t;
    pub static H5T_NATIVE_UINT64_g: hid_t;
    pub static H5T_NATIVE_INT_LEAST64_g: hid_t;
    pub static H5T_NATIVE_UINT_LEAST64_g: hid_t;
    pub static H5T_NATIVE_INT_FAST64_g: hid_t;
    pub static H5T_NATIVE_UINT_FAST64_g: hid_t;
}
