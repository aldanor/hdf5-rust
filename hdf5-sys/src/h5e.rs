//! Functions for handling errors that occur within HDF5
use std::mem;

pub use self::H5E_direction_t::*;
pub use self::H5E_type_t::*;
pub use {
    H5E_auto2_t as H5E_auto_t, H5E_error2_t as H5E_error_t, H5E_walk2_t as H5E_walk_t,
    H5Eclear2 as H5Eclear, H5Eget_auto2 as H5Eget_auto, H5Eprint2 as H5Eprint, H5Epush2 as H5Epush,
    H5Eset_auto2 as H5Eset_auto, H5Ewalk2 as H5Ewalk,
};

use crate::internal_prelude::*;

pub const H5E_DEFAULT: hid_t = 0;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug)]
pub enum H5E_type_t {
    H5E_MAJOR = 0,
    H5E_MINOR = 1,
}

pub type H5E_major_t = hid_t;
pub type H5E_minor_t = hid_t;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[deprecated(note = "deprecated in HDF5 1.8.0, use H5E_error2_t")]
pub struct H5E_error1_t {
    maj_num: H5E_major_t,
    min_num: H5E_minor_t,
    func_name: *const c_char,
    file_name: *const c_char,
    line: c_uint,
    desc: *const c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5E_error2_t {
    pub cls_id: hid_t,
    pub maj_num: hid_t,
    pub min_num: hid_t,
    pub line: c_uint,
    pub func_name: *const c_char,
    pub file_name: *const c_char,
    pub desc: *const c_char,
}

impl Default for H5E_error2_t {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug)]
pub enum H5E_direction_t {
    H5E_WALK_UPWARD = 0,
    H5E_WALK_DOWNWARD = 1,
}

#[deprecated(note = "deprecated in HDF5 1.8.0, use H5E_walk2_t")]
pub type H5E_walk1_t = Option<
    unsafe extern "C" fn(n: c_int, err_desc: *mut H5E_error1_t, client_data: *mut c_void) -> herr_t,
>;
#[deprecated(note = "deprecated in HDF5 1.8.0, use H5E_auto2_t")]
pub type H5E_auto1_t = Option<unsafe extern "C" fn(client_data: *mut c_void) -> herr_t>;

pub type H5E_walk2_t = Option<
    unsafe extern "C" fn(
        n: c_uint,
        err_desc: *const H5E_error2_t,
        client_data: *mut c_void,
    ) -> herr_t,
>;
pub type H5E_auto2_t =
    Option<unsafe extern "C" fn(estack: hid_t, client_data: *mut c_void) -> herr_t>;

extern "C" {
    pub fn H5Eregister_class(
        cls_name: *const c_char, lib_name: *const c_char, version: *const c_char,
    ) -> hid_t;
    pub fn H5Eunregister_class(class_id: hid_t) -> herr_t;
    pub fn H5Eclose_msg(err_id: hid_t) -> herr_t;
    pub fn H5Ecreate_msg(cls: hid_t, msg_type: H5E_type_t, msg: *const c_char) -> hid_t;
    pub fn H5Ecreate_stack() -> hid_t;
    pub fn H5Eget_current_stack() -> hid_t;
    pub fn H5Eclose_stack(stack_id: hid_t) -> herr_t;
    pub fn H5Eget_class_name(class_id: hid_t, name: *mut c_char, size: size_t) -> ssize_t;
    pub fn H5Eset_current_stack(err_stack_id: hid_t) -> herr_t;
    pub fn H5Epush2(
        err_stack: hid_t, file: *const c_char, func: *const c_char, line: c_uint, cls_id: hid_t,
        maj_id: hid_t, min_id: hid_t, msg: *const c_char, ...
    ) -> herr_t;
    pub fn H5Epop(err_stack: hid_t, count: size_t) -> herr_t;
    pub fn H5Eprint2(err_stack: hid_t, stream: *mut FILE) -> herr_t;
    pub fn H5Ewalk2(
        err_stack: hid_t, direction: H5E_direction_t, func: H5E_walk2_t, client_data: *mut c_void,
    ) -> herr_t;
    pub fn H5Eget_auto2(
        estack_id: hid_t, func: *mut H5E_auto2_t, client_data: *mut *mut c_void,
    ) -> herr_t;
    pub fn H5Eset_auto2(estack_id: hid_t, func: H5E_auto2_t, client_data: *mut c_void) -> herr_t;
    pub fn H5Eclear2(err_stack: hid_t) -> herr_t;
    pub fn H5Eauto_is_v2(err_stack: hid_t, is_stack: *mut c_uint) -> herr_t;
    pub fn H5Eget_msg(
        msg_id: hid_t, type_: *mut H5E_type_t, msg: *mut c_char, size: size_t,
    ) -> ssize_t;
    pub fn H5Eget_num(error_stack_id: hid_t) -> ssize_t;

    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Epush2")]
    pub fn H5Epush1(
        file: *const c_char, func: *const c_char, line: c_uint, maj: H5E_major_t, min: H5E_minor_t,
        str_: *const c_char,
    ) -> herr_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Eprint2")]
    pub fn H5Eprint1(stream: *mut FILE) -> herr_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Ewalk2")]
    pub fn H5Ewalk1(
        direction: H5E_direction_t, func: H5E_walk1_t, client_data: *mut c_void,
    ) -> herr_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Eget_auto2")]
    pub fn H5Eget_auto1(func: *mut H5E_auto1_t, client_data: *mut *mut c_void) -> herr_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Eset_auto2")]
    pub fn H5Eset_auto1(func: H5E_auto1_t, client_data: *mut c_void) -> herr_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Eclear2")]
    pub fn H5Eclear1() -> herr_t;
    #[deprecated(note = "deprecated in HDF5 1.8.0, use H5Eget_msg")]
    pub fn H5Eget_major(maj: H5E_major_t) -> *mut c_char;
    #[deprecated(note = "deprecated in HDF5 1.8.0")]
    pub fn H5Eget_minor(min: H5E_minor_t) -> *mut c_char;

    #[cfg(feature = "1.14.0")]
    pub fn H5Eappend_stack(
        dst_stack_id: hid_t, src_stack_id: hid_t, close_source_stack: hbool_t,
    ) -> herr_t;
}

pub use self::globals::*;

#[cfg(not(all(target_env = "msvc", not(feature = "static"))))]
mod globals {
    pub use crate::h5i::hid_t as id_t;

    // Error class
    extern_static!(H5E_ERR_CLS, H5E_ERR_CLS_g);

    // Errors
    extern_static!(H5E_DATASET, H5E_DATASET_g);
    extern_static!(H5E_FUNC, H5E_FUNC_g);
    extern_static!(H5E_STORAGE, H5E_STORAGE_g);
    extern_static!(H5E_FILE, H5E_FILE_g);
    extern_static!(H5E_SOHM, H5E_SOHM_g);
    extern_static!(H5E_SYM, H5E_SYM_g);
    extern_static!(H5E_PLUGIN, H5E_PLUGIN_g);
    extern_static!(H5E_VFL, H5E_VFL_g);
    extern_static!(H5E_INTERNAL, H5E_INTERNAL_g);
    extern_static!(H5E_BTREE, H5E_BTREE_g);
    extern_static!(H5E_REFERENCE, H5E_REFERENCE_g);
    extern_static!(H5E_DATASPACE, H5E_DATASPACE_g);
    extern_static!(H5E_RESOURCE, H5E_RESOURCE_g);
    extern_static!(H5E_PLIST, H5E_PLIST_g);
    extern_static!(H5E_LINK, H5E_LINK_g);
    extern_static!(H5E_DATATYPE, H5E_DATATYPE_g);
    extern_static!(H5E_RS, H5E_RS_g);
    extern_static!(H5E_HEAP, H5E_HEAP_g);
    extern_static!(H5E_OHDR, H5E_OHDR_g);
    #[cfg(not(feature = "1.14.0"))]
    extern_static!(H5E_ATOM, H5E_ATOM_g);
    extern_static!(H5E_ATTR, H5E_ATTR_g);
    extern_static!(H5E_NONE_MAJOR, H5E_NONE_MAJOR_g);
    extern_static!(H5E_IO, H5E_IO_g);
    extern_static!(H5E_SLIST, H5E_SLIST_g);
    extern_static!(H5E_EFL, H5E_EFL_g);
    extern_static!(H5E_TST, H5E_TST_g);
    extern_static!(H5E_ARGS, H5E_ARGS_g);
    extern_static!(H5E_ERROR, H5E_ERROR_g);
    extern_static!(H5E_PLINE, H5E_PLINE_g);
    extern_static!(H5E_FSPACE, H5E_FSPACE_g);
    extern_static!(H5E_CACHE, H5E_CACHE_g);
    extern_static!(H5E_SEEKERROR, H5E_SEEKERROR_g);
    extern_static!(H5E_READERROR, H5E_READERROR_g);
    extern_static!(H5E_WRITEERROR, H5E_WRITEERROR_g);
    extern_static!(H5E_CLOSEERROR, H5E_CLOSEERROR_g);
    extern_static!(H5E_OVERFLOW, H5E_OVERFLOW_g);
    extern_static!(H5E_FCNTL, H5E_FCNTL_g);
    extern_static!(H5E_NOSPACE, H5E_NOSPACE_g);
    extern_static!(H5E_CANTALLOC, H5E_CANTALLOC_g);
    extern_static!(H5E_CANTCOPY, H5E_CANTCOPY_g);
    extern_static!(H5E_CANTFREE, H5E_CANTFREE_g);
    extern_static!(H5E_ALREADYEXISTS, H5E_ALREADYEXISTS_g);
    extern_static!(H5E_CANTLOCK, H5E_CANTLOCK_g);
    extern_static!(H5E_CANTUNLOCK, H5E_CANTUNLOCK_g);
    extern_static!(H5E_CANTGC, H5E_CANTGC_g);
    extern_static!(H5E_CANTGETSIZE, H5E_CANTGETSIZE_g);
    extern_static!(H5E_OBJOPEN, H5E_OBJOPEN_g);
    extern_static!(H5E_CANTRESTORE, H5E_CANTRESTORE_g);
    extern_static!(H5E_CANTCOMPUTE, H5E_CANTCOMPUTE_g);
    extern_static!(H5E_CANTEXTEND, H5E_CANTEXTEND_g);
    extern_static!(H5E_CANTATTACH, H5E_CANTATTACH_g);
    extern_static!(H5E_CANTUPDATE, H5E_CANTUPDATE_g);
    extern_static!(H5E_CANTOPERATE, H5E_CANTOPERATE_g);
    extern_static!(H5E_CANTINIT, H5E_CANTINIT_g);
    extern_static!(H5E_ALREADYINIT, H5E_ALREADYINIT_g);
    extern_static!(H5E_CANTRELEASE, H5E_CANTRELEASE_g);
    extern_static!(H5E_CANTGET, H5E_CANTGET_g);
    extern_static!(H5E_CANTSET, H5E_CANTSET_g);
    extern_static!(H5E_DUPCLASS, H5E_DUPCLASS_g);
    extern_static!(H5E_SETDISALLOWED, H5E_SETDISALLOWED_g);
    extern_static!(H5E_CANTMERGE, H5E_CANTMERGE_g);
    extern_static!(H5E_CANTREVIVE, H5E_CANTREVIVE_g);
    extern_static!(H5E_CANTSHRINK, H5E_CANTSHRINK_g);
    extern_static!(H5E_LINKCOUNT, H5E_LINKCOUNT_g);
    extern_static!(H5E_VERSION, H5E_VERSION_g);
    extern_static!(H5E_ALIGNMENT, H5E_ALIGNMENT_g);
    extern_static!(H5E_BADMESG, H5E_BADMESG_g);
    extern_static!(H5E_CANTDELETE, H5E_CANTDELETE_g);
    extern_static!(H5E_BADITER, H5E_BADITER_g);
    extern_static!(H5E_CANTPACK, H5E_CANTPACK_g);
    extern_static!(H5E_CANTRESET, H5E_CANTRESET_g);
    extern_static!(H5E_CANTRENAME, H5E_CANTRENAME_g);
    extern_static!(H5E_SYSERRSTR, H5E_SYSERRSTR_g);
    extern_static!(H5E_NOFILTER, H5E_NOFILTER_g);
    extern_static!(H5E_CALLBACK, H5E_CALLBACK_g);
    extern_static!(H5E_CANAPPLY, H5E_CANAPPLY_g);
    extern_static!(H5E_SETLOCAL, H5E_SETLOCAL_g);
    extern_static!(H5E_NOENCODER, H5E_NOENCODER_g);
    extern_static!(H5E_CANTFILTER, H5E_CANTFILTER_g);
    extern_static!(H5E_CANTOPENOBJ, H5E_CANTOPENOBJ_g);
    extern_static!(H5E_CANTCLOSEOBJ, H5E_CANTCLOSEOBJ_g);
    extern_static!(H5E_COMPLEN, H5E_COMPLEN_g);
    extern_static!(H5E_PATH, H5E_PATH_g);
    extern_static!(H5E_NONE_MINOR, H5E_NONE_MINOR_g);
    extern_static!(H5E_OPENERROR, H5E_OPENERROR_g);
    extern_static!(H5E_FILEEXISTS, H5E_FILEEXISTS_g);
    extern_static!(H5E_FILEOPEN, H5E_FILEOPEN_g);
    extern_static!(H5E_CANTCREATE, H5E_CANTCREATE_g);
    extern_static!(H5E_CANTOPENFILE, H5E_CANTOPENFILE_g);
    extern_static!(H5E_CANTCLOSEFILE, H5E_CANTCLOSEFILE_g);
    extern_static!(H5E_NOTHDF5, H5E_NOTHDF5_g);
    extern_static!(H5E_BADFILE, H5E_BADFILE_g);
    extern_static!(H5E_TRUNCATED, H5E_TRUNCATED_g);
    extern_static!(H5E_MOUNT, H5E_MOUNT_g);
    #[cfg(not(feature = "1.14.0"))]
    extern_static!(H5E_BADATOM, H5E_BADATOM_g);
    extern_static!(H5E_BADGROUP, H5E_BADGROUP_g);
    extern_static!(H5E_CANTREGISTER, H5E_CANTREGISTER_g);
    extern_static!(H5E_CANTINC, H5E_CANTINC_g);
    extern_static!(H5E_CANTDEC, H5E_CANTDEC_g);
    extern_static!(H5E_NOIDS, H5E_NOIDS_g);
    extern_static!(H5E_CANTFLUSH, H5E_CANTFLUSH_g);
    extern_static!(H5E_CANTSERIALIZE, H5E_CANTSERIALIZE_g);
    extern_static!(H5E_CANTLOAD, H5E_CANTLOAD_g);
    extern_static!(H5E_PROTECT, H5E_PROTECT_g);
    extern_static!(H5E_NOTCACHED, H5E_NOTCACHED_g);
    extern_static!(H5E_SYSTEM, H5E_SYSTEM_g);
    extern_static!(H5E_CANTINS, H5E_CANTINS_g);
    extern_static!(H5E_CANTPROTECT, H5E_CANTPROTECT_g);
    extern_static!(H5E_CANTUNPROTECT, H5E_CANTUNPROTECT_g);
    extern_static!(H5E_CANTPIN, H5E_CANTPIN_g);
    extern_static!(H5E_CANTUNPIN, H5E_CANTUNPIN_g);
    extern_static!(H5E_CANTMARKDIRTY, H5E_CANTMARKDIRTY_g);
    extern_static!(H5E_CANTDIRTY, H5E_CANTDIRTY_g);
    extern_static!(H5E_CANTEXPUNGE, H5E_CANTEXPUNGE_g);
    extern_static!(H5E_CANTRESIZE, H5E_CANTRESIZE_g);
    extern_static!(H5E_TRAVERSE, H5E_TRAVERSE_g);
    extern_static!(H5E_NLINKS, H5E_NLINKS_g);
    extern_static!(H5E_NOTREGISTERED, H5E_NOTREGISTERED_g);
    extern_static!(H5E_CANTMOVE, H5E_CANTMOVE_g);
    extern_static!(H5E_CANTSORT, H5E_CANTSORT_g);
    extern_static!(H5E_MPI, H5E_MPI_g);
    extern_static!(H5E_MPIERRSTR, H5E_MPIERRSTR_g);
    extern_static!(H5E_CANTRECV, H5E_CANTRECV_g);
    extern_static!(H5E_CANTCLIP, H5E_CANTCLIP_g);
    extern_static!(H5E_CANTCOUNT, H5E_CANTCOUNT_g);
    extern_static!(H5E_CANTSELECT, H5E_CANTSELECT_g);
    extern_static!(H5E_CANTNEXT, H5E_CANTNEXT_g);
    extern_static!(H5E_BADSELECT, H5E_BADSELECT_g);
    extern_static!(H5E_CANTCOMPARE, H5E_CANTCOMPARE_g);
    extern_static!(H5E_UNINITIALIZED, H5E_UNINITIALIZED_g);
    extern_static!(H5E_UNSUPPORTED, H5E_UNSUPPORTED_g);
    extern_static!(H5E_BADTYPE, H5E_BADTYPE_g);
    extern_static!(H5E_BADRANGE, H5E_BADRANGE_g);
    extern_static!(H5E_BADVALUE, H5E_BADVALUE_g);
    extern_static!(H5E_NOTFOUND, H5E_NOTFOUND_g);
    extern_static!(H5E_EXISTS, H5E_EXISTS_g);
    extern_static!(H5E_CANTENCODE, H5E_CANTENCODE_g);
    extern_static!(H5E_CANTDECODE, H5E_CANTDECODE_g);
    extern_static!(H5E_CANTSPLIT, H5E_CANTSPLIT_g);
    extern_static!(H5E_CANTREDISTRIBUTE, H5E_CANTREDISTRIBUTE_g);
    extern_static!(H5E_CANTSWAP, H5E_CANTSWAP_g);
    extern_static!(H5E_CANTINSERT, H5E_CANTINSERT_g);
    extern_static!(H5E_CANTLIST, H5E_CANTLIST_g);
    extern_static!(H5E_CANTMODIFY, H5E_CANTMODIFY_g);
    extern_static!(H5E_CANTREMOVE, H5E_CANTREMOVE_g);
    extern_static!(H5E_CANTCONVERT, H5E_CANTCONVERT_g);
    extern_static!(H5E_BADSIZE, H5E_BADSIZE_g);
    #[cfg(feature = "1.12.1")]
    extern_static!(H5E_CANTLOCKFILE, H5E_CANTLOCKFILE_g);
    #[cfg(feature = "1.12.1")]
    extern_static!(H5E_CANTUNLOCKFILE, H5E_CANTUNLOCKFILE_g);
    #[cfg(feature = "1.12.1")]
    extern_static!(H5E_LIB, H5E_LIB_g);
    #[cfg(feature = "1.14.0")]
    extern_static!(H5E_BADID, H5E_BADID_g);
    #[cfg(feature = "1.14.0")]
    extern_static!(H5E_CANTCANCEL, H5E_CANTCANCEL_g);
    #[cfg(feature = "1.14.0")]
    extern_static!(H5E_CANTFIND, H5E_CANTFIND_g);
    #[cfg(feature = "1.14.0")]
    extern_static!(H5E_CANTPUT, H5E_CANTPUT_g);
    #[cfg(feature = "1.14.0")]
    extern_static!(H5E_CANTWAIT, H5E_CANTWAIT_g);
    #[cfg(feature = "1.14.0")]
    extern_static!(H5E_EVENTSET, H5E_EVENTSET_g);
    #[cfg(feature = "1.14.0")]
    extern_static!(H5E_ID, H5E_ID_g);
    #[cfg(feature = "1.14.0")]
    extern_static!(H5E_UNMOUNT, H5E_UNMOUNT_g);
}

#[cfg(all(target_env = "msvc", not(feature = "static")))]
mod globals {
    // dllimport hack
    pub type id_t = usize;

    // Error class
    extern_static!(H5E_ERR_CLS, __imp_H5E_ERR_CLS_g);

    // Errors
    extern_static!(H5E_DATASET, __imp_H5E_DATASET_g);
    extern_static!(H5E_FUNC, __imp_H5E_FUNC_g);
    extern_static!(H5E_STORAGE, __imp_H5E_STORAGE_g);
    extern_static!(H5E_FILE, __imp_H5E_FILE_g);
    extern_static!(H5E_SOHM, __imp_H5E_SOHM_g);
    extern_static!(H5E_SYM, __imp_H5E_SYM_g);
    extern_static!(H5E_PLUGIN, __imp_H5E_PLUGIN_g);
    extern_static!(H5E_VFL, __imp_H5E_VFL_g);
    extern_static!(H5E_INTERNAL, __imp_H5E_INTERNAL_g);
    extern_static!(H5E_BTREE, __imp_H5E_BTREE_g);
    extern_static!(H5E_REFERENCE, __imp_H5E_REFERENCE_g);
    extern_static!(H5E_DATASPACE, __imp_H5E_DATASPACE_g);
    extern_static!(H5E_RESOURCE, __imp_H5E_RESOURCE_g);
    extern_static!(H5E_PLIST, __imp_H5E_PLIST_g);
    extern_static!(H5E_LINK, __imp_H5E_LINK_g);
    extern_static!(H5E_DATATYPE, __imp_H5E_DATATYPE_g);
    extern_static!(H5E_RS, __imp_H5E_RS_g);
    extern_static!(H5E_HEAP, __imp_H5E_HEAP_g);
    extern_static!(H5E_OHDR, __imp_H5E_OHDR_g);
    #[cfg(not(feature = "1.14.0"))]
    extern_static!(H5E_ATOM, __imp_H5E_ATOM_g);
    extern_static!(H5E_ATTR, __imp_H5E_ATTR_g);
    extern_static!(H5E_NONE_MAJOR, __imp_H5E_NONE_MAJOR_g);
    extern_static!(H5E_IO, __imp_H5E_IO_g);
    extern_static!(H5E_SLIST, __imp_H5E_SLIST_g);
    extern_static!(H5E_EFL, __imp_H5E_EFL_g);
    extern_static!(H5E_TST, __imp_H5E_TST_g);
    extern_static!(H5E_ARGS, __imp_H5E_ARGS_g);
    extern_static!(H5E_ERROR, __imp_H5E_ERROR_g);
    extern_static!(H5E_PLINE, __imp_H5E_PLINE_g);
    extern_static!(H5E_FSPACE, __imp_H5E_FSPACE_g);
    extern_static!(H5E_CACHE, __imp_H5E_CACHE_g);
    extern_static!(H5E_SEEKERROR, __imp_H5E_SEEKERROR_g);
    extern_static!(H5E_READERROR, __imp_H5E_READERROR_g);
    extern_static!(H5E_WRITEERROR, __imp_H5E_WRITEERROR_g);
    extern_static!(H5E_CLOSEERROR, __imp_H5E_CLOSEERROR_g);
    extern_static!(H5E_OVERFLOW, __imp_H5E_OVERFLOW_g);
    extern_static!(H5E_FCNTL, __imp_H5E_FCNTL_g);
    extern_static!(H5E_NOSPACE, __imp_H5E_NOSPACE_g);
    extern_static!(H5E_CANTALLOC, __imp_H5E_CANTALLOC_g);
    extern_static!(H5E_CANTCOPY, __imp_H5E_CANTCOPY_g);
    extern_static!(H5E_CANTFREE, __imp_H5E_CANTFREE_g);
    extern_static!(H5E_ALREADYEXISTS, __imp_H5E_ALREADYEXISTS_g);
    extern_static!(H5E_CANTLOCK, __imp_H5E_CANTLOCK_g);
    extern_static!(H5E_CANTUNLOCK, __imp_H5E_CANTUNLOCK_g);
    extern_static!(H5E_CANTGC, __imp_H5E_CANTGC_g);
    extern_static!(H5E_CANTGETSIZE, __imp_H5E_CANTGETSIZE_g);
    extern_static!(H5E_OBJOPEN, __imp_H5E_OBJOPEN_g);
    extern_static!(H5E_CANTRESTORE, __imp_H5E_CANTRESTORE_g);
    extern_static!(H5E_CANTCOMPUTE, __imp_H5E_CANTCOMPUTE_g);
    extern_static!(H5E_CANTEXTEND, __imp_H5E_CANTEXTEND_g);
    extern_static!(H5E_CANTATTACH, __imp_H5E_CANTATTACH_g);
    extern_static!(H5E_CANTUPDATE, __imp_H5E_CANTUPDATE_g);
    extern_static!(H5E_CANTOPERATE, __imp_H5E_CANTOPERATE_g);
    extern_static!(H5E_CANTINIT, __imp_H5E_CANTINIT_g);
    extern_static!(H5E_ALREADYINIT, __imp_H5E_ALREADYINIT_g);
    extern_static!(H5E_CANTRELEASE, __imp_H5E_CANTRELEASE_g);
    extern_static!(H5E_CANTGET, __imp_H5E_CANTGET_g);
    extern_static!(H5E_CANTSET, __imp_H5E_CANTSET_g);
    extern_static!(H5E_DUPCLASS, __imp_H5E_DUPCLASS_g);
    extern_static!(H5E_SETDISALLOWED, __imp_H5E_SETDISALLOWED_g);
    extern_static!(H5E_CANTMERGE, __imp_H5E_CANTMERGE_g);
    extern_static!(H5E_CANTREVIVE, __imp_H5E_CANTREVIVE_g);
    extern_static!(H5E_CANTSHRINK, __imp_H5E_CANTSHRINK_g);
    extern_static!(H5E_LINKCOUNT, __imp_H5E_LINKCOUNT_g);
    extern_static!(H5E_VERSION, __imp_H5E_VERSION_g);
    extern_static!(H5E_ALIGNMENT, __imp_H5E_ALIGNMENT_g);
    extern_static!(H5E_BADMESG, __imp_H5E_BADMESG_g);
    extern_static!(H5E_CANTDELETE, __imp_H5E_CANTDELETE_g);
    extern_static!(H5E_BADITER, __imp_H5E_BADITER_g);
    extern_static!(H5E_CANTPACK, __imp_H5E_CANTPACK_g);
    extern_static!(H5E_CANTRESET, __imp_H5E_CANTRESET_g);
    extern_static!(H5E_CANTRENAME, __imp_H5E_CANTRENAME_g);
    extern_static!(H5E_SYSERRSTR, __imp_H5E_SYSERRSTR_g);
    extern_static!(H5E_NOFILTER, __imp_H5E_NOFILTER_g);
    extern_static!(H5E_CALLBACK, __imp_H5E_CALLBACK_g);
    extern_static!(H5E_CANAPPLY, __imp_H5E_CANAPPLY_g);
    extern_static!(H5E_SETLOCAL, __imp_H5E_SETLOCAL_g);
    extern_static!(H5E_NOENCODER, __imp_H5E_NOENCODER_g);
    extern_static!(H5E_CANTFILTER, __imp_H5E_CANTFILTER_g);
    extern_static!(H5E_CANTOPENOBJ, __imp_H5E_CANTOPENOBJ_g);
    extern_static!(H5E_CANTCLOSEOBJ, __imp_H5E_CANTCLOSEOBJ_g);
    extern_static!(H5E_COMPLEN, __imp_H5E_COMPLEN_g);
    extern_static!(H5E_PATH, __imp_H5E_PATH_g);
    extern_static!(H5E_NONE_MINOR, __imp_H5E_NONE_MINOR_g);
    extern_static!(H5E_OPENERROR, __imp_H5E_OPENERROR_g);
    extern_static!(H5E_FILEEXISTS, __imp_H5E_FILEEXISTS_g);
    extern_static!(H5E_FILEOPEN, __imp_H5E_FILEOPEN_g);
    extern_static!(H5E_CANTCREATE, __imp_H5E_CANTCREATE_g);
    extern_static!(H5E_CANTOPENFILE, __imp_H5E_CANTOPENFILE_g);
    extern_static!(H5E_CANTCLOSEFILE, __imp_H5E_CANTCLOSEFILE_g);
    extern_static!(H5E_NOTHDF5, __imp_H5E_NOTHDF5_g);
    extern_static!(H5E_BADFILE, __imp_H5E_BADFILE_g);
    extern_static!(H5E_TRUNCATED, __imp_H5E_TRUNCATED_g);
    extern_static!(H5E_MOUNT, __imp_H5E_MOUNT_g);
    #[cfg(not(feature = "1.14.0"))]
    extern_static!(H5E_BADATOM, __imp_H5E_BADATOM_g);
    extern_static!(H5E_BADGROUP, __imp_H5E_BADGROUP_g);
    extern_static!(H5E_CANTREGISTER, __imp_H5E_CANTREGISTER_g);
    extern_static!(H5E_CANTINC, __imp_H5E_CANTINC_g);
    extern_static!(H5E_CANTDEC, __imp_H5E_CANTDEC_g);
    extern_static!(H5E_NOIDS, __imp_H5E_NOIDS_g);
    extern_static!(H5E_CANTFLUSH, __imp_H5E_CANTFLUSH_g);
    extern_static!(H5E_CANTSERIALIZE, __imp_H5E_CANTSERIALIZE_g);
    extern_static!(H5E_CANTLOAD, __imp_H5E_CANTLOAD_g);
    extern_static!(H5E_PROTECT, __imp_H5E_PROTECT_g);
    extern_static!(H5E_NOTCACHED, __imp_H5E_NOTCACHED_g);
    extern_static!(H5E_SYSTEM, __imp_H5E_SYSTEM_g);
    extern_static!(H5E_CANTINS, __imp_H5E_CANTINS_g);
    extern_static!(H5E_CANTPROTECT, __imp_H5E_CANTPROTECT_g);
    extern_static!(H5E_CANTUNPROTECT, __imp_H5E_CANTUNPROTECT_g);
    extern_static!(H5E_CANTPIN, __imp_H5E_CANTPIN_g);
    extern_static!(H5E_CANTUNPIN, __imp_H5E_CANTUNPIN_g);
    extern_static!(H5E_CANTMARKDIRTY, __imp_H5E_CANTMARKDIRTY_g);
    extern_static!(H5E_CANTDIRTY, __imp_H5E_CANTDIRTY_g);
    extern_static!(H5E_CANTEXPUNGE, __imp_H5E_CANTEXPUNGE_g);
    extern_static!(H5E_CANTRESIZE, __imp_H5E_CANTRESIZE_g);
    extern_static!(H5E_TRAVERSE, __imp_H5E_TRAVERSE_g);
    extern_static!(H5E_NLINKS, __imp_H5E_NLINKS_g);
    extern_static!(H5E_NOTREGISTERED, __imp_H5E_NOTREGISTERED_g);
    extern_static!(H5E_CANTMOVE, __imp_H5E_CANTMOVE_g);
    extern_static!(H5E_CANTSORT, __imp_H5E_CANTSORT_g);
    extern_static!(H5E_MPI, __imp_H5E_MPI_g);
    extern_static!(H5E_MPIERRSTR, __imp_H5E_MPIERRSTR_g);
    extern_static!(H5E_CANTRECV, __imp_H5E_CANTRECV_g);
    extern_static!(H5E_CANTCLIP, __imp_H5E_CANTCLIP_g);
    extern_static!(H5E_CANTCOUNT, __imp_H5E_CANTCOUNT_g);
    extern_static!(H5E_CANTSELECT, __imp_H5E_CANTSELECT_g);
    extern_static!(H5E_CANTNEXT, __imp_H5E_CANTNEXT_g);
    extern_static!(H5E_BADSELECT, __imp_H5E_BADSELECT_g);
    extern_static!(H5E_CANTCOMPARE, __imp_H5E_CANTCOMPARE_g);
    extern_static!(H5E_UNINITIALIZED, __imp_H5E_UNINITIALIZED_g);
    extern_static!(H5E_UNSUPPORTED, __imp_H5E_UNSUPPORTED_g);
    extern_static!(H5E_BADTYPE, __imp_H5E_BADTYPE_g);
    extern_static!(H5E_BADRANGE, __imp_H5E_BADRANGE_g);
    extern_static!(H5E_BADVALUE, __imp_H5E_BADVALUE_g);
    extern_static!(H5E_NOTFOUND, __imp_H5E_NOTFOUND_g);
    extern_static!(H5E_EXISTS, __imp_H5E_EXISTS_g);
    extern_static!(H5E_CANTENCODE, __imp_H5E_CANTENCODE_g);
    extern_static!(H5E_CANTDECODE, __imp_H5E_CANTDECODE_g);
    extern_static!(H5E_CANTSPLIT, __imp_H5E_CANTSPLIT_g);
    extern_static!(H5E_CANTREDISTRIBUTE, __imp_H5E_CANTREDISTRIBUTE_g);
    extern_static!(H5E_CANTSWAP, __imp_H5E_CANTSWAP_g);
    extern_static!(H5E_CANTINSERT, __imp_H5E_CANTINSERT_g);
    extern_static!(H5E_CANTLIST, __imp_H5E_CANTLIST_g);
    extern_static!(H5E_CANTMODIFY, __imp_H5E_CANTMODIFY_g);
    extern_static!(H5E_CANTREMOVE, __imp_H5E_CANTREMOVE_g);
    extern_static!(H5E_CANTCONVERT, __imp_H5E_CANTCONVERT_g);
    extern_static!(H5E_BADSIZE, __imp_H5E_BADSIZE_g);
    #[cfg(feature = "1.12.1")]
    extern_static!(H5E_CANTLOCKFILE, __imp_H5E_CANTLOCKFILE_g);
    #[cfg(feature = "1.12.1")]
    extern_static!(H5E_CANTUNLOCKFILE, __imp_H5E_CANTUNLOCKFILE_g);
    #[cfg(feature = "1.12.1")]
    extern_static!(H5E_LIB, __imp_H5E_LIB_g);
    #[cfg(feature = "1.14.0")]
    extern_static!(H5E_BADID, __imp_H5E_BADID_g);
    #[cfg(feature = "1.14.0")]
    extern_static!(H5E_CANTCANCEL, __imp_H5E_CANTCANCEL_g);
    #[cfg(feature = "1.14.0")]
    extern_static!(H5E_CANTFIND, __imp_H5E_CANTFIND_g);
    #[cfg(feature = "1.14.0")]
    extern_static!(H5E_CANTPUT, __imp_H5E_CANTPUT_g);
    #[cfg(feature = "1.14.0")]
    extern_static!(H5E_CANTWAIT, __imp_H5E_CANTWAIT_g);
    #[cfg(feature = "1.14.0")]
    extern_static!(H5E_EVENTSET, __imp_H5E_EVENTSET_g);
    #[cfg(feature = "1.14.0")]
    extern_static!(H5E_ID, __imp_H5E_ID_g);
    #[cfg(feature = "1.14.0")]
    extern_static!(H5E_UNMOUNT, __imp_H5E_UNMOUNT_g);
}
