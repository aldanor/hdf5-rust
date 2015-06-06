pub use self::H5E_type_t::*;
pub use self::H5E_direction_t::*;

use libc::{c_uint, c_void, c_char, size_t, ssize_t, FILE};

use h5::herr_t;
use h5i::hid_t;

pub const H5E_DEFAULT: hid_t = 0;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5E_type_t {
    H5E_MAJOR = 0,
    H5E_MINOR = 1,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct H5E_error2_t {
    pub cls_id: hid_t,
    pub maj_num: hid_t,
    pub min_num: hid_t,
    pub line: c_uint,
    pub func_name: *const c_char,
    pub file_name: *const c_char,
    pub desc: *const c_char,
}

impl ::std::default::Default for H5E_error2_t {
    fn default() -> H5E_error2_t { unsafe { ::std::mem::zeroed() } }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum H5E_direction_t {
    H5E_WALK_UPWARD = 0,
    H5E_WALK_DOWNWARD = 1,
}

pub type H5E_walk2_t = Option<extern fn (n: c_uint, err_desc: *const H5E_error2_t, client_data:
                                         *mut c_void) -> herr_t>;
pub type H5E_auto2_t = Option<extern fn (estack: hid_t, client_data: *mut c_void) -> herr_t>;

extern {
    pub fn H5Eregister_class(cls_name: *const c_char, lib_name: *const c_char, version: *const
                             c_char) -> hid_t;
    pub fn H5Eunregister_class(class_id: hid_t) -> herr_t;
    pub fn H5Eclose_msg(err_id: hid_t) -> herr_t;
    pub fn H5Ecreate_msg(cls: hid_t, msg_type: H5E_type_t, msg: *const c_char) -> hid_t;
    pub fn H5Ecreate_stack() -> hid_t;
    pub fn H5Eget_current_stack() -> hid_t;
    pub fn H5Eclose_stack(stack_id: hid_t) -> herr_t;
    pub fn H5Eget_class_name(class_id: hid_t, name: *mut c_char, size: size_t) -> ssize_t;
    pub fn H5Eset_current_stack(err_stack_id: hid_t) -> herr_t;
    pub fn H5Epush2(err_stack: hid_t, file: *const c_char, func: *const c_char, line: c_uint,
                    cls_id: hid_t, maj_id: hid_t, min_id: hid_t, msg: *const c_char, ...) -> herr_t;
    pub fn H5Epop(err_stack: hid_t, count: size_t) -> herr_t;
    pub fn H5Eprint2(err_stack: hid_t, stream: *mut FILE) -> herr_t;
    pub fn H5Ewalk2(err_stack: hid_t, direction: H5E_direction_t, func: H5E_walk2_t, client_data:
                    *mut c_void) -> herr_t;
    pub fn H5Eget_auto2(estack_id: hid_t, func: *mut H5E_auto2_t, client_data: *mut *mut c_void) ->
                        herr_t;
    pub fn H5Eset_auto2(estack_id: hid_t, func: H5E_auto2_t, client_data: *mut c_void) -> herr_t;
    pub fn H5Eclear2(err_stack: hid_t) -> herr_t;
    pub fn H5Eauto_is_v2(err_stack: hid_t, is_stack: *mut c_uint) -> herr_t;
    pub fn H5Eget_msg(msg_id: hid_t, _type: *mut H5E_type_t, msg: *mut c_char, size: size_t) ->
                      ssize_t;
    pub fn H5Eget_num(error_stack_id: hid_t) -> ssize_t;
}

extern {
    // Error class
    pub static H5E_ERR_CLS_g: hid_t;

    // Errors
    pub static H5E_DATASET_g: hid_t;
    pub static H5E_FUNC_g: hid_t;
    pub static H5E_STORAGE_g: hid_t;
    pub static H5E_FILE_g: hid_t;
    pub static H5E_SOHM_g: hid_t;
    pub static H5E_SYM_g: hid_t;
    pub static H5E_PLUGIN_g: hid_t;
    pub static H5E_VFL_g: hid_t;
    pub static H5E_INTERNAL_g: hid_t;
    pub static H5E_BTREE_g: hid_t;
    pub static H5E_REFERENCE_g: hid_t;
    pub static H5E_DATASPACE_g: hid_t;
    pub static H5E_RESOURCE_g: hid_t;
    pub static H5E_PLIST_g: hid_t;
    pub static H5E_LINK_g: hid_t;
    pub static H5E_DATATYPE_g: hid_t;
    pub static H5E_RS_g: hid_t;
    pub static H5E_HEAP_g: hid_t;
    pub static H5E_OHDR_g: hid_t;
    pub static H5E_ATOM_g: hid_t;
    pub static H5E_ATTR_g: hid_t;
    pub static H5E_NONE_MAJOR_g: hid_t;
    pub static H5E_IO_g: hid_t;
    pub static H5E_SLIST_g: hid_t;
    pub static H5E_EFL_g: hid_t;
    pub static H5E_TST_g: hid_t;
    pub static H5E_ARGS_g: hid_t;
    pub static H5E_ERROR_g: hid_t;
    pub static H5E_PLINE_g: hid_t;
    pub static H5E_FSPACE_g: hid_t;
    pub static H5E_CACHE_g: hid_t;
    pub static H5E_SEEKERROR_g: hid_t;
    pub static H5E_READERROR_g: hid_t;
    pub static H5E_WRITEERROR_g: hid_t;
    pub static H5E_CLOSEERROR_g: hid_t;
    pub static H5E_OVERFLOW_g: hid_t;
    pub static H5E_FCNTL_g: hid_t;
    pub static H5E_NOSPACE_g: hid_t;
    pub static H5E_CANTALLOC_g: hid_t;
    pub static H5E_CANTCOPY_g: hid_t;
    pub static H5E_CANTFREE_g: hid_t;
    pub static H5E_ALREADYEXISTS_g: hid_t;
    pub static H5E_CANTLOCK_g: hid_t;
    pub static H5E_CANTUNLOCK_g: hid_t;
    pub static H5E_CANTGC_g: hid_t;
    pub static H5E_CANTGETSIZE_g: hid_t;
    pub static H5E_OBJOPEN_g: hid_t;
    pub static H5E_CANTRESTORE_g: hid_t;
    pub static H5E_CANTCOMPUTE_g: hid_t;
    pub static H5E_CANTEXTEND_g: hid_t;
    pub static H5E_CANTATTACH_g: hid_t;
    pub static H5E_CANTUPDATE_g: hid_t;
    pub static H5E_CANTOPERATE_g: hid_t;
    pub static H5E_CANTINIT_g: hid_t;
    pub static H5E_ALREADYINIT_g: hid_t;
    pub static H5E_CANTRELEASE_g: hid_t;
    pub static H5E_CANTGET_g: hid_t;
    pub static H5E_CANTSET_g: hid_t;
    pub static H5E_DUPCLASS_g: hid_t;
    pub static H5E_SETDISALLOWED_g: hid_t;
    pub static H5E_CANTMERGE_g: hid_t;
    pub static H5E_CANTREVIVE_g: hid_t;
    pub static H5E_CANTSHRINK_g: hid_t;
    pub static H5E_LINKCOUNT_g: hid_t;
    pub static H5E_VERSION_g: hid_t;
    pub static H5E_ALIGNMENT_g: hid_t;
    pub static H5E_BADMESG_g: hid_t;
    pub static H5E_CANTDELETE_g: hid_t;
    pub static H5E_BADITER_g: hid_t;
    pub static H5E_CANTPACK_g: hid_t;
    pub static H5E_CANTRESET_g: hid_t;
    pub static H5E_CANTRENAME_g: hid_t;
    pub static H5E_SYSERRSTR_g: hid_t;
    pub static H5E_NOFILTER_g: hid_t;
    pub static H5E_CALLBACK_g: hid_t;
    pub static H5E_CANAPPLY_g: hid_t;
    pub static H5E_SETLOCAL_g: hid_t;
    pub static H5E_NOENCODER_g: hid_t;
    pub static H5E_CANTFILTER_g: hid_t;
    pub static H5E_CANTOPENOBJ_g: hid_t;
    pub static H5E_CANTCLOSEOBJ_g: hid_t;
    pub static H5E_COMPLEN_g: hid_t;
    pub static H5E_PATH_g: hid_t;
    pub static H5E_NONE_MINOR_g: hid_t;
    pub static H5E_OPENERROR_g: hid_t;
    pub static H5E_FILEEXISTS_g: hid_t;
    pub static H5E_FILEOPEN_g: hid_t;
    pub static H5E_CANTCREATE_g: hid_t;
    pub static H5E_CANTOPENFILE_g: hid_t;
    pub static H5E_CANTCLOSEFILE_g: hid_t;
    pub static H5E_NOTHDF5_g: hid_t;
    pub static H5E_BADFILE_g: hid_t;
    pub static H5E_TRUNCATED_g: hid_t;
    pub static H5E_MOUNT_g: hid_t;
    pub static H5E_BADATOM_g: hid_t;
    pub static H5E_BADGROUP_g: hid_t;
    pub static H5E_CANTREGISTER_g: hid_t;
    pub static H5E_CANTINC_g: hid_t;
    pub static H5E_CANTDEC_g: hid_t;
    pub static H5E_NOIDS_g: hid_t;
    pub static H5E_CANTFLUSH_g: hid_t;
    pub static H5E_CANTSERIALIZE_g: hid_t;
    pub static H5E_CANTLOAD_g: hid_t;
    pub static H5E_PROTECT_g: hid_t;
    pub static H5E_NOTCACHED_g: hid_t;
    pub static H5E_SYSTEM_g: hid_t;
    pub static H5E_CANTINS_g: hid_t;
    pub static H5E_CANTPROTECT_g: hid_t;
    pub static H5E_CANTUNPROTECT_g: hid_t;
    pub static H5E_CANTPIN_g: hid_t;
    pub static H5E_CANTUNPIN_g: hid_t;
    pub static H5E_CANTMARKDIRTY_g: hid_t;
    pub static H5E_CANTDIRTY_g: hid_t;
    pub static H5E_CANTEXPUNGE_g: hid_t;
    pub static H5E_CANTRESIZE_g: hid_t;
    pub static H5E_TRAVERSE_g: hid_t;
    pub static H5E_NLINKS_g: hid_t;
    pub static H5E_NOTREGISTERED_g: hid_t;
    pub static H5E_CANTMOVE_g: hid_t;
    pub static H5E_CANTSORT_g: hid_t;
    pub static H5E_MPI_g: hid_t;
    pub static H5E_MPIERRSTR_g: hid_t;
    pub static H5E_CANTRECV_g: hid_t;
    pub static H5E_CANTCLIP_g: hid_t;
    pub static H5E_CANTCOUNT_g: hid_t;
    pub static H5E_CANTSELECT_g: hid_t;
    pub static H5E_CANTNEXT_g: hid_t;
    pub static H5E_BADSELECT_g: hid_t;
    pub static H5E_CANTCOMPARE_g: hid_t;
    pub static H5E_UNINITIALIZED_g: hid_t;
    pub static H5E_UNSUPPORTED_g: hid_t;
    pub static H5E_BADTYPE_g: hid_t;
    pub static H5E_BADRANGE_g: hid_t;
    pub static H5E_BADVALUE_g: hid_t;
    pub static H5E_NOTFOUND_g: hid_t;
    pub static H5E_EXISTS_g: hid_t;
    pub static H5E_CANTENCODE_g: hid_t;
    pub static H5E_CANTDECODE_g: hid_t;
    pub static H5E_CANTSPLIT_g: hid_t;
    pub static H5E_CANTREDISTRIBUTE_g: hid_t;
    pub static H5E_CANTSWAP_g: hid_t;
    pub static H5E_CANTINSERT_g: hid_t;
    pub static H5E_CANTLIST_g: hid_t;
    pub static H5E_CANTMODIFY_g: hid_t;
    pub static H5E_CANTREMOVE_g: hid_t;
    pub static H5E_CANTCONVERT_g: hid_t;
    pub static H5E_BADSIZE_g: hid_t;
}
