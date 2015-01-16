pub use self::H5E_type_t::*;
pub use self::H5E_direction_t::*;

use libc::{c_uint, c_void, c_char, size_t, ssize_t, FILE};

use ffi::types::{hid_t, herr_t};
use ffi::h5::H5open;

pub const H5E_DEFAULT: hid_t = 0;

#[repr(C)]
#[derive(Copy)]
pub enum H5E_type_t {
    H5E_MAJOR = 0,
    H5E_MINOR = 1,
}

#[repr(C)]
#[derive(Copy)]
pub struct H5E_error2_t {
    pub cls_id: hid_t,
    pub maj_num: hid_t,
    pub min_num: hid_t,
    pub line: c_uint,
    pub func_name: *const c_char,
    pub file_name: *const c_char,
    pub desc: *const c_char,
}

#[repr(C)]
#[derive(Copy)]
pub enum H5E_direction_t {
    H5E_WALK_UPWARD = 0,
    H5E_WALK_DOWNWARD = 1,
}

pub type H5E_walk2_t = Option<extern fn (n: c_uint, err_desc: *const H5E_error2_t, client_data:
                                             *mut c_void) -> herr_t>;
pub type H5E_auto2_t = Option<extern fn (estack: hid_t, client_data: *mut c_void) -> herr_t>;

#[link(name = "hdf5")]
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

#[test]
pub fn test_globals() {
    use ffi::h5i::H5I_INVALID_HID;

    assert!(*H5E_ERR_CLS != H5I_INVALID_HID);
    assert!(*H5E_FUNC != H5I_INVALID_HID);
}

register_h5open!(H5E_ERR_CLS, H5E_ERR_CLS_g);

register_h5open!(H5E_DATASET, H5E_DATASET_g);
register_h5open!(H5E_FUNC, H5E_FUNC_g);
register_h5open!(H5E_STORAGE, H5E_STORAGE_g);
register_h5open!(H5E_FILE, H5E_FILE_g);
register_h5open!(H5E_SOHM, H5E_SOHM_g);
register_h5open!(H5E_SYM, H5E_SYM_g);
register_h5open!(H5E_PLUGIN, H5E_PLUGIN_g);
register_h5open!(H5E_VFL, H5E_VFL_g);
register_h5open!(H5E_INTERNAL, H5E_INTERNAL_g);
register_h5open!(H5E_BTREE, H5E_BTREE_g);
register_h5open!(H5E_REFERENCE, H5E_REFERENCE_g);
register_h5open!(H5E_DATASPACE, H5E_DATASPACE_g);
register_h5open!(H5E_RESOURCE, H5E_RESOURCE_g);
register_h5open!(H5E_PLIST, H5E_PLIST_g);
register_h5open!(H5E_LINK, H5E_LINK_g);
register_h5open!(H5E_DATATYPE, H5E_DATATYPE_g);
register_h5open!(H5E_RS, H5E_RS_g);
register_h5open!(H5E_HEAP, H5E_HEAP_g);
register_h5open!(H5E_OHDR, H5E_OHDR_g);
register_h5open!(H5E_ATOM, H5E_ATOM_g);
register_h5open!(H5E_ATTR, H5E_ATTR_g);
register_h5open!(H5E_NONE_MAJOR, H5E_NONE_MAJOR_g);
register_h5open!(H5E_IO, H5E_IO_g);
register_h5open!(H5E_SLIST, H5E_SLIST_g);
register_h5open!(H5E_EFL, H5E_EFL_g);
register_h5open!(H5E_TST, H5E_TST_g);
register_h5open!(H5E_ARGS, H5E_ARGS_g);
register_h5open!(H5E_ERROR, H5E_ERROR_g);
register_h5open!(H5E_PLINE, H5E_PLINE_g);
register_h5open!(H5E_FSPACE, H5E_FSPACE_g);
register_h5open!(H5E_CACHE, H5E_CACHE_g);
register_h5open!(H5E_SEEKERROR, H5E_SEEKERROR_g);
register_h5open!(H5E_READERROR, H5E_READERROR_g);
register_h5open!(H5E_WRITEERROR, H5E_WRITEERROR_g);
register_h5open!(H5E_CLOSEERROR, H5E_CLOSEERROR_g);
register_h5open!(H5E_OVERFLOW, H5E_OVERFLOW_g);
register_h5open!(H5E_FCNTL, H5E_FCNTL_g);
register_h5open!(H5E_NOSPACE, H5E_NOSPACE_g);
register_h5open!(H5E_CANTALLOC, H5E_CANTALLOC_g);
register_h5open!(H5E_CANTCOPY, H5E_CANTCOPY_g);
register_h5open!(H5E_CANTFREE, H5E_CANTFREE_g);
register_h5open!(H5E_ALREADYEXISTS, H5E_ALREADYEXISTS_g);
register_h5open!(H5E_CANTLOCK, H5E_CANTLOCK_g);
register_h5open!(H5E_CANTUNLOCK, H5E_CANTUNLOCK_g);
register_h5open!(H5E_CANTGC, H5E_CANTGC_g);
register_h5open!(H5E_CANTGETSIZE, H5E_CANTGETSIZE_g);
register_h5open!(H5E_OBJOPEN, H5E_OBJOPEN_g);
register_h5open!(H5E_CANTRESTORE, H5E_CANTRESTORE_g);
register_h5open!(H5E_CANTCOMPUTE, H5E_CANTCOMPUTE_g);
register_h5open!(H5E_CANTEXTEND, H5E_CANTEXTEND_g);
register_h5open!(H5E_CANTATTACH, H5E_CANTATTACH_g);
register_h5open!(H5E_CANTUPDATE, H5E_CANTUPDATE_g);
register_h5open!(H5E_CANTOPERATE, H5E_CANTOPERATE_g);
register_h5open!(H5E_CANTINIT, H5E_CANTINIT_g);
register_h5open!(H5E_ALREADYINIT, H5E_ALREADYINIT_g);
register_h5open!(H5E_CANTRELEASE, H5E_CANTRELEASE_g);
register_h5open!(H5E_CANTGET, H5E_CANTGET_g);
register_h5open!(H5E_CANTSET, H5E_CANTSET_g);
register_h5open!(H5E_DUPCLASS, H5E_DUPCLASS_g);
register_h5open!(H5E_SETDISALLOWED, H5E_SETDISALLOWED_g);
register_h5open!(H5E_CANTMERGE, H5E_CANTMERGE_g);
register_h5open!(H5E_CANTREVIVE, H5E_CANTREVIVE_g);
register_h5open!(H5E_CANTSHRINK, H5E_CANTSHRINK_g);
register_h5open!(H5E_LINKCOUNT, H5E_LINKCOUNT_g);
register_h5open!(H5E_VERSION, H5E_VERSION_g);
register_h5open!(H5E_ALIGNMENT, H5E_ALIGNMENT_g);
register_h5open!(H5E_BADMESG, H5E_BADMESG_g);
register_h5open!(H5E_CANTDELETE, H5E_CANTDELETE_g);
register_h5open!(H5E_BADITER, H5E_BADITER_g);
register_h5open!(H5E_CANTPACK, H5E_CANTPACK_g);
register_h5open!(H5E_CANTRESET, H5E_CANTRESET_g);
register_h5open!(H5E_CANTRENAME, H5E_CANTRENAME_g);
register_h5open!(H5E_SYSERRSTR, H5E_SYSERRSTR_g);
register_h5open!(H5E_NOFILTER, H5E_NOFILTER_g);
register_h5open!(H5E_CALLBACK, H5E_CALLBACK_g);
register_h5open!(H5E_CANAPPLY, H5E_CANAPPLY_g);
register_h5open!(H5E_SETLOCAL, H5E_SETLOCAL_g);
register_h5open!(H5E_NOENCODER, H5E_NOENCODER_g);
register_h5open!(H5E_CANTFILTER, H5E_CANTFILTER_g);
register_h5open!(H5E_CANTOPENOBJ, H5E_CANTOPENOBJ_g);
register_h5open!(H5E_CANTCLOSEOBJ, H5E_CANTCLOSEOBJ_g);
register_h5open!(H5E_COMPLEN, H5E_COMPLEN_g);
register_h5open!(H5E_PATH, H5E_PATH_g);
register_h5open!(H5E_NONE_MINOR, H5E_NONE_MINOR_g);
register_h5open!(H5E_OPENERROR, H5E_OPENERROR_g);
register_h5open!(H5E_FILEEXISTS, H5E_FILEEXISTS_g);
register_h5open!(H5E_FILEOPEN, H5E_FILEOPEN_g);
register_h5open!(H5E_CANTCREATE, H5E_CANTCREATE_g);
register_h5open!(H5E_CANTOPENFILE, H5E_CANTOPENFILE_g);
register_h5open!(H5E_CANTCLOSEFILE, H5E_CANTCLOSEFILE_g);
register_h5open!(H5E_NOTHDF5, H5E_NOTHDF5_g);
register_h5open!(H5E_BADFILE, H5E_BADFILE_g);
register_h5open!(H5E_TRUNCATED, H5E_TRUNCATED_g);
register_h5open!(H5E_MOUNT, H5E_MOUNT_g);
register_h5open!(H5E_BADATOM, H5E_BADATOM_g);
register_h5open!(H5E_BADGROUP, H5E_BADGROUP_g);
register_h5open!(H5E_CANTREGISTER, H5E_CANTREGISTER_g);
register_h5open!(H5E_CANTINC, H5E_CANTINC_g);
register_h5open!(H5E_CANTDEC, H5E_CANTDEC_g);
register_h5open!(H5E_NOIDS, H5E_NOIDS_g);
register_h5open!(H5E_CANTFLUSH, H5E_CANTFLUSH_g);
register_h5open!(H5E_CANTSERIALIZE, H5E_CANTSERIALIZE_g);
register_h5open!(H5E_CANTLOAD, H5E_CANTLOAD_g);
register_h5open!(H5E_PROTECT, H5E_PROTECT_g);
register_h5open!(H5E_NOTCACHED, H5E_NOTCACHED_g);
register_h5open!(H5E_SYSTEM, H5E_SYSTEM_g);
register_h5open!(H5E_CANTINS, H5E_CANTINS_g);
register_h5open!(H5E_CANTPROTECT, H5E_CANTPROTECT_g);
register_h5open!(H5E_CANTUNPROTECT, H5E_CANTUNPROTECT_g);
register_h5open!(H5E_CANTPIN, H5E_CANTPIN_g);
register_h5open!(H5E_CANTUNPIN, H5E_CANTUNPIN_g);
register_h5open!(H5E_CANTMARKDIRTY, H5E_CANTMARKDIRTY_g);
register_h5open!(H5E_CANTDIRTY, H5E_CANTDIRTY_g);
register_h5open!(H5E_CANTEXPUNGE, H5E_CANTEXPUNGE_g);
register_h5open!(H5E_CANTRESIZE, H5E_CANTRESIZE_g);
register_h5open!(H5E_TRAVERSE, H5E_TRAVERSE_g);
register_h5open!(H5E_NLINKS, H5E_NLINKS_g);
register_h5open!(H5E_NOTREGISTERED, H5E_NOTREGISTERED_g);
register_h5open!(H5E_CANTMOVE, H5E_CANTMOVE_g);
register_h5open!(H5E_CANTSORT, H5E_CANTSORT_g);
register_h5open!(H5E_MPI, H5E_MPI_g);
register_h5open!(H5E_MPIERRSTR, H5E_MPIERRSTR_g);
register_h5open!(H5E_CANTRECV, H5E_CANTRECV_g);
register_h5open!(H5E_CANTCLIP, H5E_CANTCLIP_g);
register_h5open!(H5E_CANTCOUNT, H5E_CANTCOUNT_g);
register_h5open!(H5E_CANTSELECT, H5E_CANTSELECT_g);
register_h5open!(H5E_CANTNEXT, H5E_CANTNEXT_g);
register_h5open!(H5E_BADSELECT, H5E_BADSELECT_g);
register_h5open!(H5E_CANTCOMPARE, H5E_CANTCOMPARE_g);
register_h5open!(H5E_UNINITIALIZED, H5E_UNINITIALIZED_g);
register_h5open!(H5E_UNSUPPORTED, H5E_UNSUPPORTED_g);
register_h5open!(H5E_BADTYPE, H5E_BADTYPE_g);
register_h5open!(H5E_BADRANGE, H5E_BADRANGE_g);
register_h5open!(H5E_BADVALUE, H5E_BADVALUE_g);
register_h5open!(H5E_NOTFOUND, H5E_NOTFOUND_g);
register_h5open!(H5E_EXISTS, H5E_EXISTS_g);
register_h5open!(H5E_CANTENCODE, H5E_CANTENCODE_g);
register_h5open!(H5E_CANTDECODE, H5E_CANTDECODE_g);
register_h5open!(H5E_CANTSPLIT, H5E_CANTSPLIT_g);
register_h5open!(H5E_CANTREDISTRIBUTE, H5E_CANTREDISTRIBUTE_g);
register_h5open!(H5E_CANTSWAP, H5E_CANTSWAP_g);
register_h5open!(H5E_CANTINSERT, H5E_CANTINSERT_g);
register_h5open!(H5E_CANTLIST, H5E_CANTLIST_g);
register_h5open!(H5E_CANTMODIFY, H5E_CANTMODIFY_g);
register_h5open!(H5E_CANTREMOVE, H5E_CANTREMOVE_g);
register_h5open!(H5E_CANTCONVERT, H5E_CANTCONVERT_g);
register_h5open!(H5E_BADSIZE, H5E_BADSIZE_g);
