pub use self::H5Z_SO_scale_type_t::*;
pub use self::H5Z_EDC_t::*;
pub use self::H5Z_cb_return_t::*;

use libc::{c_int, c_uint, c_void, c_char, size_t};

use h5::{herr_t, htri_t};
use h5i::hid_t;

pub const H5Z_FILTER_ERROR:       hid_t = -1;
pub const H5Z_FILTER_NONE:        hid_t = 0;
pub const H5Z_FILTER_DEFLATE:     hid_t = 1;
pub const H5Z_FILTER_SHUFFLE:     hid_t = 2;
pub const H5Z_FILTER_FLETCHER32:  hid_t = 3;
pub const H5Z_FILTER_SZIP:        hid_t = 4;
pub const H5Z_FILTER_NBIT:        hid_t = 5;
pub const H5Z_FILTER_SCALEOFFSET: hid_t = 6;
pub const H5Z_FILTER_RESERVED:    hid_t = 256;

pub const H5Z_FILTER_MAX: c_uint = 65535;

pub const H5Z_FILTER_ALL: c_uint = 0;

pub const H5Z_MAX_NFILTERS: c_uint = 32;

pub const H5Z_FLAG_DEFMASK:   c_uint = 0x00ff;
pub const H5Z_FLAG_MANDATORY: c_uint = 0x0000;
pub const H5Z_FLAG_OPTIONAL:  c_uint = 0x0001;

pub const H5Z_FLAG_INVMASK:   c_uint = 0xff00;
pub const H5Z_FLAG_REVERSE:   c_uint = 0x0100;
pub const H5Z_FLAG_SKIP_EDC:  c_uint = 0x0200;

pub const H5_SZIP_ALLOW_K13_OPTION_MASK: c_uint = 1;
pub const H5_SZIP_CHIP_OPTION_MASK:      c_uint = 2;
pub const H5_SZIP_EC_OPTION_MASK:        c_uint = 4;
pub const H5_SZIP_NN_OPTION_MASK:        c_uint = 32;
pub const H5_SZIP_MAX_PIXELS_PER_BLOCK:  c_uint = 32;

pub const H5Z_SHUFFLE_USER_NPARMS:  c_uint = 0;
pub const H5Z_SHUFFLE_TOTAL_NPARMS: c_uint = 1;

pub const H5Z_SZIP_USER_NPARMS:  c_uint = 2;
pub const H5Z_SZIP_TOTAL_NPARMS: c_uint = 4;
pub const H5Z_SZIP_PARM_MASK:    c_uint = 0;
pub const H5Z_SZIP_PARM_PPB:     c_uint = 1;
pub const H5Z_SZIP_PARM_BPP:     c_uint = 2;
pub const H5Z_SZIP_PARM_PPS:     c_uint = 3;

pub const H5Z_NBIT_USER_NPARMS: c_uint = 0;

pub const H5Z_SCALEOFFSET_USER_NPARMS: c_uint = 2;

pub const H5Z_SO_INT_MINBITS_DEFAULT: c_uint = 0;

pub const H5Z_CLASS_T_VERS: c_uint = 1;

pub const H5Z_FILTER_CONFIG_ENCODE_ENABLED: c_uint = 0x0001;
pub const H5Z_FILTER_CONFIG_DECODE_ENABLED: c_uint = 0x0002;

pub type H5Z_filter_t = c_int;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5Z_SO_scale_type_t {
    H5Z_SO_FLOAT_DSCALE = 0,
    H5Z_SO_FLOAT_ESCALE = 1,
    H5Z_SO_INT          = 2,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5Z_EDC_t {
    H5Z_ERROR_EDC   = -1,
    H5Z_DISABLE_EDC = 0,
    H5Z_ENABLE_EDC  = 1,
    H5Z_NO_EDC      = 2,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5Z_cb_return_t {
    H5Z_CB_ERROR = -1,
    H5Z_CB_FAIL  = 0,
    H5Z_CB_CONT  = 1,
    H5Z_CB_NO    = 2,
}

pub type H5Z_filter_func_t = Option<extern fn (filter: H5Z_filter_t, buf: *mut c_void, buf_size:
                                               size_t, op_data: *mut c_void) -> H5Z_cb_return_t>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct H5Z_cb_t {
    pub func: H5Z_filter_func_t,
    pub op_data: *mut c_void,
}

impl ::std::default::Default for H5Z_cb_t {
    fn default() -> H5Z_cb_t { unsafe { ::std::mem::zeroed() } }
}

pub type H5Z_can_apply_func_t = Option<extern fn (dcpl_id: hid_t, type_id: hid_t, space_id: hid_t)
                                                  -> htri_t>;
pub type H5Z_set_local_func_t = Option<extern fn (dcpl_id: hid_t, type_id: hid_t, space_id: hid_t)
                                                  -> herr_t>;
pub type H5Z_func_t = Option<extern fn (flags: c_uint, cd_nelmts: size_t, cd_values: *const c_uint,
                                        nbytes: size_t, buf_size: *mut size_t, buf: *mut *mut
                                        c_void) -> size_t>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct H5Z_class2_t {
    pub version: c_int,
    pub id: H5Z_filter_t,
    pub encoder_present: c_uint,
    pub decoder_present: c_uint,
    pub name: *const c_char,
    pub can_apply: H5Z_can_apply_func_t,
    pub set_local: H5Z_set_local_func_t,
    pub filter: H5Z_func_t,
}

impl ::std::default::Default for H5Z_class2_t {
    fn default() -> H5Z_class2_t { unsafe { ::std::mem::zeroed() } }
}

extern {
    pub fn H5Zregister(cls: *const c_void) -> herr_t;
    pub fn H5Zunregister(id: H5Z_filter_t) -> herr_t;
    pub fn H5Zfilter_avail(id: H5Z_filter_t) -> htri_t;
    pub fn H5Zget_filter_info(filter: H5Z_filter_t, filter_config_flags: *mut c_uint) -> herr_t;
}
