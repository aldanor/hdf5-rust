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

impl Default for H5E_error2_t {
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

pub use self::globals::*;

#[cfg(not(target_env = "msvc"))]
mod globals {
    pub use h5i::hid_t as id_t;

    extern {
        // Error class
        static H5E_ERR_CLS_g: id_t;

        // Errors
        static H5E_DATASET_g: id_t;
        static H5E_FUNC_g: id_t;
        static H5E_STORAGE_g: id_t;
        static H5E_FILE_g: id_t;
        static H5E_SOHM_g: id_t;
        static H5E_SYM_g: id_t;
        static H5E_PLUGIN_g: id_t;
        static H5E_VFL_g: id_t;
        static H5E_INTERNAL_g: id_t;
        static H5E_BTREE_g: id_t;
        static H5E_REFERENCE_g: id_t;
        static H5E_DATASPACE_g: id_t;
        static H5E_RESOURCE_g: id_t;
        static H5E_PLIST_g: id_t;
        static H5E_LINK_g: id_t;
        static H5E_DATATYPE_g: id_t;
        static H5E_RS_g: id_t;
        static H5E_HEAP_g: id_t;
        static H5E_OHDR_g: id_t;
        static H5E_ATOM_g: id_t;
        static H5E_ATTR_g: id_t;
        static H5E_NONE_MAJOR_g: id_t;
        static H5E_IO_g: id_t;
        static H5E_SLIST_g: id_t;
        static H5E_EFL_g: id_t;
        static H5E_TST_g: id_t;
        static H5E_ARGS_g: id_t;
        static H5E_ERROR_g: id_t;
        static H5E_PLINE_g: id_t;
        static H5E_FSPACE_g: id_t;
        static H5E_CACHE_g: id_t;
        static H5E_SEEKERROR_g: id_t;
        static H5E_READERROR_g: id_t;
        static H5E_WRITEERROR_g: id_t;
        static H5E_CLOSEERROR_g: id_t;
        static H5E_OVERFLOW_g: id_t;
        static H5E_FCNTL_g: id_t;
        static H5E_NOSPACE_g: id_t;
        static H5E_CANTALLOC_g: id_t;
        static H5E_CANTCOPY_g: id_t;
        static H5E_CANTFREE_g: id_t;
        static H5E_ALREADYEXISTS_g: id_t;
        static H5E_CANTLOCK_g: id_t;
        static H5E_CANTUNLOCK_g: id_t;
        static H5E_CANTGC_g: id_t;
        static H5E_CANTGETSIZE_g: id_t;
        static H5E_OBJOPEN_g: id_t;
        static H5E_CANTRESTORE_g: id_t;
        static H5E_CANTCOMPUTE_g: id_t;
        static H5E_CANTEXTEND_g: id_t;
        static H5E_CANTATTACH_g: id_t;
        static H5E_CANTUPDATE_g: id_t;
        static H5E_CANTOPERATE_g: id_t;
        static H5E_CANTINIT_g: id_t;
        static H5E_ALREADYINIT_g: id_t;
        static H5E_CANTRELEASE_g: id_t;
        static H5E_CANTGET_g: id_t;
        static H5E_CANTSET_g: id_t;
        static H5E_DUPCLASS_g: id_t;
        static H5E_SETDISALLOWED_g: id_t;
        static H5E_CANTMERGE_g: id_t;
        static H5E_CANTREVIVE_g: id_t;
        static H5E_CANTSHRINK_g: id_t;
        static H5E_LINKCOUNT_g: id_t;
        static H5E_VERSION_g: id_t;
        static H5E_ALIGNMENT_g: id_t;
        static H5E_BADMESG_g: id_t;
        static H5E_CANTDELETE_g: id_t;
        static H5E_BADITER_g: id_t;
        static H5E_CANTPACK_g: id_t;
        static H5E_CANTRESET_g: id_t;
        static H5E_CANTRENAME_g: id_t;
        static H5E_SYSERRSTR_g: id_t;
        static H5E_NOFILTER_g: id_t;
        static H5E_CALLBACK_g: id_t;
        static H5E_CANAPPLY_g: id_t;
        static H5E_SETLOCAL_g: id_t;
        static H5E_NOENCODER_g: id_t;
        static H5E_CANTFILTER_g: id_t;
        static H5E_CANTOPENOBJ_g: id_t;
        static H5E_CANTCLOSEOBJ_g: id_t;
        static H5E_COMPLEN_g: id_t;
        static H5E_PATH_g: id_t;
        static H5E_NONE_MINOR_g: id_t;
        static H5E_OPENERROR_g: id_t;
        static H5E_FILEEXISTS_g: id_t;
        static H5E_FILEOPEN_g: id_t;
        static H5E_CANTCREATE_g: id_t;
        static H5E_CANTOPENFILE_g: id_t;
        static H5E_CANTCLOSEFILE_g: id_t;
        static H5E_NOTHDF5_g: id_t;
        static H5E_BADFILE_g: id_t;
        static H5E_TRUNCATED_g: id_t;
        static H5E_MOUNT_g: id_t;
        static H5E_BADATOM_g: id_t;
        static H5E_BADGROUP_g: id_t;
        static H5E_CANTREGISTER_g: id_t;
        static H5E_CANTINC_g: id_t;
        static H5E_CANTDEC_g: id_t;
        static H5E_NOIDS_g: id_t;
        static H5E_CANTFLUSH_g: id_t;
        static H5E_CANTSERIALIZE_g: id_t;
        static H5E_CANTLOAD_g: id_t;
        static H5E_PROTECT_g: id_t;
        static H5E_NOTCACHED_g: id_t;
        static H5E_SYSTEM_g: id_t;
        static H5E_CANTINS_g: id_t;
        static H5E_CANTPROTECT_g: id_t;
        static H5E_CANTUNPROTECT_g: id_t;
        static H5E_CANTPIN_g: id_t;
        static H5E_CANTUNPIN_g: id_t;
        static H5E_CANTMARKDIRTY_g: id_t;
        static H5E_CANTDIRTY_g: id_t;
        static H5E_CANTEXPUNGE_g: id_t;
        static H5E_CANTRESIZE_g: id_t;
        static H5E_TRAVERSE_g: id_t;
        static H5E_NLINKS_g: id_t;
        static H5E_NOTREGISTERED_g: id_t;
        static H5E_CANTMOVE_g: id_t;
        static H5E_CANTSORT_g: id_t;
        static H5E_MPI_g: id_t;
        static H5E_MPIERRSTR_g: id_t;
        static H5E_CANTRECV_g: id_t;
        static H5E_CANTCLIP_g: id_t;
        static H5E_CANTCOUNT_g: id_t;
        static H5E_CANTSELECT_g: id_t;
        static H5E_CANTNEXT_g: id_t;
        static H5E_BADSELECT_g: id_t;
        static H5E_CANTCOMPARE_g: id_t;
        static H5E_UNINITIALIZED_g: id_t;
        static H5E_UNSUPPORTED_g: id_t;
        static H5E_BADTYPE_g: id_t;
        static H5E_BADRANGE_g: id_t;
        static H5E_BADVALUE_g: id_t;
        static H5E_NOTFOUND_g: id_t;
        static H5E_EXISTS_g: id_t;
        static H5E_CANTENCODE_g: id_t;
        static H5E_CANTDECODE_g: id_t;
        static H5E_CANTSPLIT_g: id_t;
        static H5E_CANTREDISTRIBUTE_g: id_t;
        static H5E_CANTSWAP_g: id_t;
        static H5E_CANTINSERT_g: id_t;
        static H5E_CANTLIST_g: id_t;
        static H5E_CANTMODIFY_g: id_t;
        static H5E_CANTREMOVE_g: id_t;
        static H5E_CANTCONVERT_g: id_t;
        static H5E_BADSIZE_g: id_t;
    }

    // Error class
    pub static H5E_ERR_CLS: &'static id_t = &H5E_ERR_CLS_g;

    // Errors
    pub static H5E_DATASET: &'static id_t = &H5E_DATASET_g;
    pub static H5E_FUNC: &'static id_t = &H5E_FUNC_g;
    pub static H5E_STORAGE: &'static id_t = &H5E_STORAGE_g;
    pub static H5E_FILE: &'static id_t = &H5E_FILE_g;
    pub static H5E_SOHM: &'static id_t = &H5E_SOHM_g;
    pub static H5E_SYM: &'static id_t = &H5E_SYM_g;
    pub static H5E_PLUGIN: &'static id_t = &H5E_PLUGIN_g;
    pub static H5E_VFL: &'static id_t = &H5E_VFL_g;
    pub static H5E_INTERNAL: &'static id_t = &H5E_INTERNAL_g;
    pub static H5E_BTREE: &'static id_t = &H5E_BTREE_g;
    pub static H5E_REFERENCE: &'static id_t = &H5E_REFERENCE_g;
    pub static H5E_DATASPACE: &'static id_t = &H5E_DATASPACE_g;
    pub static H5E_RESOURCE: &'static id_t = &H5E_RESOURCE_g;
    pub static H5E_PLIST: &'static id_t = &H5E_PLIST_g;
    pub static H5E_LINK: &'static id_t = &H5E_LINK_g;
    pub static H5E_DATATYPE: &'static id_t = &H5E_DATATYPE_g;
    pub static H5E_RS: &'static id_t = &H5E_RS_g;
    pub static H5E_HEAP: &'static id_t = &H5E_HEAP_g;
    pub static H5E_OHDR: &'static id_t = &H5E_OHDR_g;
    pub static H5E_ATOM: &'static id_t = &H5E_ATOM_g;
    pub static H5E_ATTR: &'static id_t = &H5E_ATTR_g;
    pub static H5E_NONE_MAJOR: &'static id_t = &H5E_NONE_MAJOR_g;
    pub static H5E_IO: &'static id_t = &H5E_IO_g;
    pub static H5E_SLIST: &'static id_t = &H5E_SLIST_g;
    pub static H5E_EFL: &'static id_t = &H5E_EFL_g;
    pub static H5E_TST: &'static id_t = &H5E_TST_g;
    pub static H5E_ARGS: &'static id_t = &H5E_ARGS_g;
    pub static H5E_ERROR: &'static id_t = &H5E_ERROR_g;
    pub static H5E_PLINE: &'static id_t = &H5E_PLINE_g;
    pub static H5E_FSPACE: &'static id_t = &H5E_FSPACE_g;
    pub static H5E_CACHE: &'static id_t = &H5E_CACHE_g;
    pub static H5E_SEEKERROR: &'static id_t = &H5E_SEEKERROR_g;
    pub static H5E_READERROR: &'static id_t = &H5E_READERROR_g;
    pub static H5E_WRITEERROR: &'static id_t = &H5E_WRITEERROR_g;
    pub static H5E_CLOSEERROR: &'static id_t = &H5E_CLOSEERROR_g;
    pub static H5E_OVERFLOW: &'static id_t = &H5E_OVERFLOW_g;
    pub static H5E_FCNTL: &'static id_t = &H5E_FCNTL_g;
    pub static H5E_NOSPACE: &'static id_t = &H5E_NOSPACE_g;
    pub static H5E_CANTALLOC: &'static id_t = &H5E_CANTALLOC_g;
    pub static H5E_CANTCOPY: &'static id_t = &H5E_CANTCOPY_g;
    pub static H5E_CANTFREE: &'static id_t = &H5E_CANTFREE_g;
    pub static H5E_ALREADYEXISTS: &'static id_t = &H5E_ALREADYEXISTS_g;
    pub static H5E_CANTLOCK: &'static id_t = &H5E_CANTLOCK_g;
    pub static H5E_CANTUNLOCK: &'static id_t = &H5E_CANTUNLOCK_g;
    pub static H5E_CANTGC: &'static id_t = &H5E_CANTGC_g;
    pub static H5E_CANTGETSIZE: &'static id_t = &H5E_CANTGETSIZE_g;
    pub static H5E_OBJOPEN: &'static id_t = &H5E_OBJOPEN_g;
    pub static H5E_CANTRESTORE: &'static id_t = &H5E_CANTRESTORE_g;
    pub static H5E_CANTCOMPUTE: &'static id_t = &H5E_CANTCOMPUTE_g;
    pub static H5E_CANTEXTEND: &'static id_t = &H5E_CANTEXTEND_g;
    pub static H5E_CANTATTACH: &'static id_t = &H5E_CANTATTACH_g;
    pub static H5E_CANTUPDATE: &'static id_t = &H5E_CANTUPDATE_g;
    pub static H5E_CANTOPERATE: &'static id_t = &H5E_CANTOPERATE_g;
    pub static H5E_CANTINIT: &'static id_t = &H5E_CANTINIT_g;
    pub static H5E_ALREADYINIT: &'static id_t = &H5E_ALREADYINIT_g;
    pub static H5E_CANTRELEASE: &'static id_t = &H5E_CANTRELEASE_g;
    pub static H5E_CANTGET: &'static id_t = &H5E_CANTGET_g;
    pub static H5E_CANTSET: &'static id_t = &H5E_CANTSET_g;
    pub static H5E_DUPCLASS: &'static id_t = &H5E_DUPCLASS_g;
    pub static H5E_SETDISALLOWED: &'static id_t = &H5E_SETDISALLOWED_g;
    pub static H5E_CANTMERGE: &'static id_t = &H5E_CANTMERGE_g;
    pub static H5E_CANTREVIVE: &'static id_t = &H5E_CANTREVIVE_g;
    pub static H5E_CANTSHRINK: &'static id_t = &H5E_CANTSHRINK_g;
    pub static H5E_LINKCOUNT: &'static id_t = &H5E_LINKCOUNT_g;
    pub static H5E_VERSION: &'static id_t = &H5E_VERSION_g;
    pub static H5E_ALIGNMENT: &'static id_t = &H5E_ALIGNMENT_g;
    pub static H5E_BADMESG: &'static id_t = &H5E_BADMESG_g;
    pub static H5E_CANTDELETE: &'static id_t = &H5E_CANTDELETE_g;
    pub static H5E_BADITER: &'static id_t = &H5E_BADITER_g;
    pub static H5E_CANTPACK: &'static id_t = &H5E_CANTPACK_g;
    pub static H5E_CANTRESET: &'static id_t = &H5E_CANTRESET_g;
    pub static H5E_CANTRENAME: &'static id_t = &H5E_CANTRENAME_g;
    pub static H5E_SYSERRSTR: &'static id_t = &H5E_SYSERRSTR_g;
    pub static H5E_NOFILTER: &'static id_t = &H5E_NOFILTER_g;
    pub static H5E_CALLBACK: &'static id_t = &H5E_CALLBACK_g;
    pub static H5E_CANAPPLY: &'static id_t = &H5E_CANAPPLY_g;
    pub static H5E_SETLOCAL: &'static id_t = &H5E_SETLOCAL_g;
    pub static H5E_NOENCODER: &'static id_t = &H5E_NOENCODER_g;
    pub static H5E_CANTFILTER: &'static id_t = &H5E_CANTFILTER_g;
    pub static H5E_CANTOPENOBJ: &'static id_t = &H5E_CANTOPENOBJ_g;
    pub static H5E_CANTCLOSEOBJ: &'static id_t = &H5E_CANTCLOSEOBJ_g;
    pub static H5E_COMPLEN: &'static id_t = &H5E_COMPLEN_g;
    pub static H5E_PATH: &'static id_t = &H5E_PATH_g;
    pub static H5E_NONE_MINOR: &'static id_t = &H5E_NONE_MINOR_g;
    pub static H5E_OPENERROR: &'static id_t = &H5E_OPENERROR_g;
    pub static H5E_FILEEXISTS: &'static id_t = &H5E_FILEEXISTS_g;
    pub static H5E_FILEOPEN: &'static id_t = &H5E_FILEOPEN_g;
    pub static H5E_CANTCREATE: &'static id_t = &H5E_CANTCREATE_g;
    pub static H5E_CANTOPENFILE: &'static id_t = &H5E_CANTOPENFILE_g;
    pub static H5E_CANTCLOSEFILE: &'static id_t = &H5E_CANTCLOSEFILE_g;
    pub static H5E_NOTHDF5: &'static id_t = &H5E_NOTHDF5_g;
    pub static H5E_BADFILE: &'static id_t = &H5E_BADFILE_g;
    pub static H5E_TRUNCATED: &'static id_t = &H5E_TRUNCATED_g;
    pub static H5E_MOUNT: &'static id_t = &H5E_MOUNT_g;
    pub static H5E_BADATOM: &'static id_t = &H5E_BADATOM_g;
    pub static H5E_BADGROUP: &'static id_t = &H5E_BADGROUP_g;
    pub static H5E_CANTREGISTER: &'static id_t = &H5E_CANTREGISTER_g;
    pub static H5E_CANTINC: &'static id_t = &H5E_CANTINC_g;
    pub static H5E_CANTDEC: &'static id_t = &H5E_CANTDEC_g;
    pub static H5E_NOIDS: &'static id_t = &H5E_NOIDS_g;
    pub static H5E_CANTFLUSH: &'static id_t = &H5E_CANTFLUSH_g;
    pub static H5E_CANTSERIALIZE: &'static id_t = &H5E_CANTSERIALIZE_g;
    pub static H5E_CANTLOAD: &'static id_t = &H5E_CANTLOAD_g;
    pub static H5E_PROTECT: &'static id_t = &H5E_PROTECT_g;
    pub static H5E_NOTCACHED: &'static id_t = &H5E_NOTCACHED_g;
    pub static H5E_SYSTEM: &'static id_t = &H5E_SYSTEM_g;
    pub static H5E_CANTINS: &'static id_t = &H5E_CANTINS_g;
    pub static H5E_CANTPROTECT: &'static id_t = &H5E_CANTPROTECT_g;
    pub static H5E_CANTUNPROTECT: &'static id_t = &H5E_CANTUNPROTECT_g;
    pub static H5E_CANTPIN: &'static id_t = &H5E_CANTPIN_g;
    pub static H5E_CANTUNPIN: &'static id_t = &H5E_CANTUNPIN_g;
    pub static H5E_CANTMARKDIRTY: &'static id_t = &H5E_CANTMARKDIRTY_g;
    pub static H5E_CANTDIRTY: &'static id_t = &H5E_CANTDIRTY_g;
    pub static H5E_CANTEXPUNGE: &'static id_t = &H5E_CANTEXPUNGE_g;
    pub static H5E_CANTRESIZE: &'static id_t = &H5E_CANTRESIZE_g;
    pub static H5E_TRAVERSE: &'static id_t = &H5E_TRAVERSE_g;
    pub static H5E_NLINKS: &'static id_t = &H5E_NLINKS_g;
    pub static H5E_NOTREGISTERED: &'static id_t = &H5E_NOTREGISTERED_g;
    pub static H5E_CANTMOVE: &'static id_t = &H5E_CANTMOVE_g;
    pub static H5E_CANTSORT: &'static id_t = &H5E_CANTSORT_g;
    pub static H5E_MPI: &'static id_t = &H5E_MPI_g;
    pub static H5E_MPIERRSTR: &'static id_t = &H5E_MPIERRSTR_g;
    pub static H5E_CANTRECV: &'static id_t = &H5E_CANTRECV_g;
    pub static H5E_CANTCLIP: &'static id_t = &H5E_CANTCLIP_g;
    pub static H5E_CANTCOUNT: &'static id_t = &H5E_CANTCOUNT_g;
    pub static H5E_CANTSELECT: &'static id_t = &H5E_CANTSELECT_g;
    pub static H5E_CANTNEXT: &'static id_t = &H5E_CANTNEXT_g;
    pub static H5E_BADSELECT: &'static id_t = &H5E_BADSELECT_g;
    pub static H5E_CANTCOMPARE: &'static id_t = &H5E_CANTCOMPARE_g;
    pub static H5E_UNINITIALIZED: &'static id_t = &H5E_UNINITIALIZED_g;
    pub static H5E_UNSUPPORTED: &'static id_t = &H5E_UNSUPPORTED_g;
    pub static H5E_BADTYPE: &'static id_t = &H5E_BADTYPE_g;
    pub static H5E_BADRANGE: &'static id_t = &H5E_BADRANGE_g;
    pub static H5E_BADVALUE: &'static id_t = &H5E_BADVALUE_g;
    pub static H5E_NOTFOUND: &'static id_t = &H5E_NOTFOUND_g;
    pub static H5E_EXISTS: &'static id_t = &H5E_EXISTS_g;
    pub static H5E_CANTENCODE: &'static id_t = &H5E_CANTENCODE_g;
    pub static H5E_CANTDECODE: &'static id_t = &H5E_CANTDECODE_g;
    pub static H5E_CANTSPLIT: &'static id_t = &H5E_CANTSPLIT_g;
    pub static H5E_CANTREDISTRIBUTE: &'static id_t = &H5E_CANTREDISTRIBUTE_g;
    pub static H5E_CANTSWAP: &'static id_t = &H5E_CANTSWAP_g;
    pub static H5E_CANTINSERT: &'static id_t = &H5E_CANTINSERT_g;
    pub static H5E_CANTLIST: &'static id_t = &H5E_CANTLIST_g;
    pub static H5E_CANTMODIFY: &'static id_t = &H5E_CANTMODIFY_g;
    pub static H5E_CANTREMOVE: &'static id_t = &H5E_CANTREMOVE_g;
    pub static H5E_CANTCONVERT: &'static id_t = &H5E_CANTCONVERT_g;
    pub static H5E_BADSIZE: &'static id_t = &H5E_BADSIZE_g;
}

#[cfg(target_env = "msvc")]
mod globals {
    // dllimport hack
    pub type id_t = usize;

    extern {
        // Error class
        static __imp_H5E_ERR_CLS_g: id_t;

        // Errors
        static __imp_H5E_DATASET_g: id_t;
        static __imp_H5E_FUNC_g: id_t;
        static __imp_H5E_STORAGE_g: id_t;
        static __imp_H5E_FILE_g: id_t;
        static __imp_H5E_SOHM_g: id_t;
        static __imp_H5E_SYM_g: id_t;
        static __imp_H5E_PLUGIN_g: id_t;
        static __imp_H5E_VFL_g: id_t;
        static __imp_H5E_INTERNAL_g: id_t;
        static __imp_H5E_BTREE_g: id_t;
        static __imp_H5E_REFERENCE_g: id_t;
        static __imp_H5E_DATASPACE_g: id_t;
        static __imp_H5E_RESOURCE_g: id_t;
        static __imp_H5E_PLIST_g: id_t;
        static __imp_H5E_LINK_g: id_t;
        static __imp_H5E_DATATYPE_g: id_t;
        static __imp_H5E_RS_g: id_t;
        static __imp_H5E_HEAP_g: id_t;
        static __imp_H5E_OHDR_g: id_t;
        static __imp_H5E_ATOM_g: id_t;
        static __imp_H5E_ATTR_g: id_t;
        static __imp_H5E_NONE_MAJOR_g: id_t;
        static __imp_H5E_IO_g: id_t;
        static __imp_H5E_SLIST_g: id_t;
        static __imp_H5E_EFL_g: id_t;
        static __imp_H5E_TST_g: id_t;
        static __imp_H5E_ARGS_g: id_t;
        static __imp_H5E_ERROR_g: id_t;
        static __imp_H5E_PLINE_g: id_t;
        static __imp_H5E_FSPACE_g: id_t;
        static __imp_H5E_CACHE_g: id_t;
        static __imp_H5E_SEEKERROR_g: id_t;
        static __imp_H5E_READERROR_g: id_t;
        static __imp_H5E_WRITEERROR_g: id_t;
        static __imp_H5E_CLOSEERROR_g: id_t;
        static __imp_H5E_OVERFLOW_g: id_t;
        static __imp_H5E_FCNTL_g: id_t;
        static __imp_H5E_NOSPACE_g: id_t;
        static __imp_H5E_CANTALLOC_g: id_t;
        static __imp_H5E_CANTCOPY_g: id_t;
        static __imp_H5E_CANTFREE_g: id_t;
        static __imp_H5E_ALREADYEXISTS_g: id_t;
        static __imp_H5E_CANTLOCK_g: id_t;
        static __imp_H5E_CANTUNLOCK_g: id_t;
        static __imp_H5E_CANTGC_g: id_t;
        static __imp_H5E_CANTGETSIZE_g: id_t;
        static __imp_H5E_OBJOPEN_g: id_t;
        static __imp_H5E_CANTRESTORE_g: id_t;
        static __imp_H5E_CANTCOMPUTE_g: id_t;
        static __imp_H5E_CANTEXTEND_g: id_t;
        static __imp_H5E_CANTATTACH_g: id_t;
        static __imp_H5E_CANTUPDATE_g: id_t;
        static __imp_H5E_CANTOPERATE_g: id_t;
        static __imp_H5E_CANTINIT_g: id_t;
        static __imp_H5E_ALREADYINIT_g: id_t;
        static __imp_H5E_CANTRELEASE_g: id_t;
        static __imp_H5E_CANTGET_g: id_t;
        static __imp_H5E_CANTSET_g: id_t;
        static __imp_H5E_DUPCLASS_g: id_t;
        static __imp_H5E_SETDISALLOWED_g: id_t;
        static __imp_H5E_CANTMERGE_g: id_t;
        static __imp_H5E_CANTREVIVE_g: id_t;
        static __imp_H5E_CANTSHRINK_g: id_t;
        static __imp_H5E_LINKCOUNT_g: id_t;
        static __imp_H5E_VERSION_g: id_t;
        static __imp_H5E_ALIGNMENT_g: id_t;
        static __imp_H5E_BADMESG_g: id_t;
        static __imp_H5E_CANTDELETE_g: id_t;
        static __imp_H5E_BADITER_g: id_t;
        static __imp_H5E_CANTPACK_g: id_t;
        static __imp_H5E_CANTRESET_g: id_t;
        static __imp_H5E_CANTRENAME_g: id_t;
        static __imp_H5E_SYSERRSTR_g: id_t;
        static __imp_H5E_NOFILTER_g: id_t;
        static __imp_H5E_CALLBACK_g: id_t;
        static __imp_H5E_CANAPPLY_g: id_t;
        static __imp_H5E_SETLOCAL_g: id_t;
        static __imp_H5E_NOENCODER_g: id_t;
        static __imp_H5E_CANTFILTER_g: id_t;
        static __imp_H5E_CANTOPENOBJ_g: id_t;
        static __imp_H5E_CANTCLOSEOBJ_g: id_t;
        static __imp_H5E_COMPLEN_g: id_t;
        static __imp_H5E_PATH_g: id_t;
        static __imp_H5E_NONE_MINOR_g: id_t;
        static __imp_H5E_OPENERROR_g: id_t;
        static __imp_H5E_FILEEXISTS_g: id_t;
        static __imp_H5E_FILEOPEN_g: id_t;
        static __imp_H5E_CANTCREATE_g: id_t;
        static __imp_H5E_CANTOPENFILE_g: id_t;
        static __imp_H5E_CANTCLOSEFILE_g: id_t;
        static __imp_H5E_NOTHDF5_g: id_t;
        static __imp_H5E_BADFILE_g: id_t;
        static __imp_H5E_TRUNCATED_g: id_t;
        static __imp_H5E_MOUNT_g: id_t;
        static __imp_H5E_BADATOM_g: id_t;
        static __imp_H5E_BADGROUP_g: id_t;
        static __imp_H5E_CANTREGISTER_g: id_t;
        static __imp_H5E_CANTINC_g: id_t;
        static __imp_H5E_CANTDEC_g: id_t;
        static __imp_H5E_NOIDS_g: id_t;
        static __imp_H5E_CANTFLUSH_g: id_t;
        static __imp_H5E_CANTSERIALIZE_g: id_t;
        static __imp_H5E_CANTLOAD_g: id_t;
        static __imp_H5E_PROTECT_g: id_t;
        static __imp_H5E_NOTCACHED_g: id_t;
        static __imp_H5E_SYSTEM_g: id_t;
        static __imp_H5E_CANTINS_g: id_t;
        static __imp_H5E_CANTPROTECT_g: id_t;
        static __imp_H5E_CANTUNPROTECT_g: id_t;
        static __imp_H5E_CANTPIN_g: id_t;
        static __imp_H5E_CANTUNPIN_g: id_t;
        static __imp_H5E_CANTMARKDIRTY_g: id_t;
        static __imp_H5E_CANTDIRTY_g: id_t;
        static __imp_H5E_CANTEXPUNGE_g: id_t;
        static __imp_H5E_CANTRESIZE_g: id_t;
        static __imp_H5E_TRAVERSE_g: id_t;
        static __imp_H5E_NLINKS_g: id_t;
        static __imp_H5E_NOTREGISTERED_g: id_t;
        static __imp_H5E_CANTMOVE_g: id_t;
        static __imp_H5E_CANTSORT_g: id_t;
        static __imp_H5E_MPI_g: id_t;
        static __imp_H5E_MPIERRSTR_g: id_t;
        static __imp_H5E_CANTRECV_g: id_t;
        static __imp_H5E_CANTCLIP_g: id_t;
        static __imp_H5E_CANTCOUNT_g: id_t;
        static __imp_H5E_CANTSELECT_g: id_t;
        static __imp_H5E_CANTNEXT_g: id_t;
        static __imp_H5E_BADSELECT_g: id_t;
        static __imp_H5E_CANTCOMPARE_g: id_t;
        static __imp_H5E_UNINITIALIZED_g: id_t;
        static __imp_H5E_UNSUPPORTED_g: id_t;
        static __imp_H5E_BADTYPE_g: id_t;
        static __imp_H5E_BADRANGE_g: id_t;
        static __imp_H5E_BADVALUE_g: id_t;
        static __imp_H5E_NOTFOUND_g: id_t;
        static __imp_H5E_EXISTS_g: id_t;
        static __imp_H5E_CANTENCODE_g: id_t;
        static __imp_H5E_CANTDECODE_g: id_t;
        static __imp_H5E_CANTSPLIT_g: id_t;
        static __imp_H5E_CANTREDISTRIBUTE_g: id_t;
        static __imp_H5E_CANTSWAP_g: id_t;
        static __imp_H5E_CANTINSERT_g: id_t;
        static __imp_H5E_CANTLIST_g: id_t;
        static __imp_H5E_CANTMODIFY_g: id_t;
        static __imp_H5E_CANTREMOVE_g: id_t;
        static __imp_H5E_CANTCONVERT_g: id_t;
        static __imp_H5E_BADSIZE_g: id_t;
    }

    // Error class
    pub static H5E_ERR_CLS: &'static id_t = &__imp_H5E_ERR_CLS_g;

    // Errors
    pub static H5E_DATASET: &'static id_t = &__imp_H5E_DATASET_g;
    pub static H5E_FUNC: &'static id_t = &__imp_H5E_FUNC_g;
    pub static H5E_STORAGE: &'static id_t = &__imp_H5E_STORAGE_g;
    pub static H5E_FILE: &'static id_t = &__imp_H5E_FILE_g;
    pub static H5E_SOHM: &'static id_t = &__imp_H5E_SOHM_g;
    pub static H5E_SYM: &'static id_t = &__imp_H5E_SYM_g;
    pub static H5E_PLUGIN: &'static id_t = &__imp_H5E_PLUGIN_g;
    pub static H5E_VFL: &'static id_t = &__imp_H5E_VFL_g;
    pub static H5E_INTERNAL: &'static id_t = &__imp_H5E_INTERNAL_g;
    pub static H5E_BTREE: &'static id_t = &__imp_H5E_BTREE_g;
    pub static H5E_REFERENCE: &'static id_t = &__imp_H5E_REFERENCE_g;
    pub static H5E_DATASPACE: &'static id_t = &__imp_H5E_DATASPACE_g;
    pub static H5E_RESOURCE: &'static id_t = &__imp_H5E_RESOURCE_g;
    pub static H5E_PLIST: &'static id_t = &__imp_H5E_PLIST_g;
    pub static H5E_LINK: &'static id_t = &__imp_H5E_LINK_g;
    pub static H5E_DATATYPE: &'static id_t = &__imp_H5E_DATATYPE_g;
    pub static H5E_RS: &'static id_t = &__imp_H5E_RS_g;
    pub static H5E_HEAP: &'static id_t = &__imp_H5E_HEAP_g;
    pub static H5E_OHDR: &'static id_t = &__imp_H5E_OHDR_g;
    pub static H5E_ATOM: &'static id_t = &__imp_H5E_ATOM_g;
    pub static H5E_ATTR: &'static id_t = &__imp_H5E_ATTR_g;
    pub static H5E_NONE_MAJOR: &'static id_t = &__imp_H5E_NONE_MAJOR_g;
    pub static H5E_IO: &'static id_t = &__imp_H5E_IO_g;
    pub static H5E_SLIST: &'static id_t = &__imp_H5E_SLIST_g;
    pub static H5E_EFL: &'static id_t = &__imp_H5E_EFL_g;
    pub static H5E_TST: &'static id_t = &__imp_H5E_TST_g;
    pub static H5E_ARGS: &'static id_t = &__imp_H5E_ARGS_g;
    pub static H5E_ERROR: &'static id_t = &__imp_H5E_ERROR_g;
    pub static H5E_PLINE: &'static id_t = &__imp_H5E_PLINE_g;
    pub static H5E_FSPACE: &'static id_t = &__imp_H5E_FSPACE_g;
    pub static H5E_CACHE: &'static id_t = &__imp_H5E_CACHE_g;
    pub static H5E_SEEKERROR: &'static id_t = &__imp_H5E_SEEKERROR_g;
    pub static H5E_READERROR: &'static id_t = &__imp_H5E_READERROR_g;
    pub static H5E_WRITEERROR: &'static id_t = &__imp_H5E_WRITEERROR_g;
    pub static H5E_CLOSEERROR: &'static id_t = &__imp_H5E_CLOSEERROR_g;
    pub static H5E_OVERFLOW: &'static id_t = &__imp_H5E_OVERFLOW_g;
    pub static H5E_FCNTL: &'static id_t = &__imp_H5E_FCNTL_g;
    pub static H5E_NOSPACE: &'static id_t = &__imp_H5E_NOSPACE_g;
    pub static H5E_CANTALLOC: &'static id_t = &__imp_H5E_CANTALLOC_g;
    pub static H5E_CANTCOPY: &'static id_t = &__imp_H5E_CANTCOPY_g;
    pub static H5E_CANTFREE: &'static id_t = &__imp_H5E_CANTFREE_g;
    pub static H5E_ALREADYEXISTS: &'static id_t = &__imp_H5E_ALREADYEXISTS_g;
    pub static H5E_CANTLOCK: &'static id_t = &__imp_H5E_CANTLOCK_g;
    pub static H5E_CANTUNLOCK: &'static id_t = &__imp_H5E_CANTUNLOCK_g;
    pub static H5E_CANTGC: &'static id_t = &__imp_H5E_CANTGC_g;
    pub static H5E_CANTGETSIZE: &'static id_t = &__imp_H5E_CANTGETSIZE_g;
    pub static H5E_OBJOPEN: &'static id_t = &__imp_H5E_OBJOPEN_g;
    pub static H5E_CANTRESTORE: &'static id_t = &__imp_H5E_CANTRESTORE_g;
    pub static H5E_CANTCOMPUTE: &'static id_t = &__imp_H5E_CANTCOMPUTE_g;
    pub static H5E_CANTEXTEND: &'static id_t = &__imp_H5E_CANTEXTEND_g;
    pub static H5E_CANTATTACH: &'static id_t = &__imp_H5E_CANTATTACH_g;
    pub static H5E_CANTUPDATE: &'static id_t = &__imp_H5E_CANTUPDATE_g;
    pub static H5E_CANTOPERATE: &'static id_t = &__imp_H5E_CANTOPERATE_g;
    pub static H5E_CANTINIT: &'static id_t = &__imp_H5E_CANTINIT_g;
    pub static H5E_ALREADYINIT: &'static id_t = &__imp_H5E_ALREADYINIT_g;
    pub static H5E_CANTRELEASE: &'static id_t = &__imp_H5E_CANTRELEASE_g;
    pub static H5E_CANTGET: &'static id_t = &__imp_H5E_CANTGET_g;
    pub static H5E_CANTSET: &'static id_t = &__imp_H5E_CANTSET_g;
    pub static H5E_DUPCLASS: &'static id_t = &__imp_H5E_DUPCLASS_g;
    pub static H5E_SETDISALLOWED: &'static id_t = &__imp_H5E_SETDISALLOWED_g;
    pub static H5E_CANTMERGE: &'static id_t = &__imp_H5E_CANTMERGE_g;
    pub static H5E_CANTREVIVE: &'static id_t = &__imp_H5E_CANTREVIVE_g;
    pub static H5E_CANTSHRINK: &'static id_t = &__imp_H5E_CANTSHRINK_g;
    pub static H5E_LINKCOUNT: &'static id_t = &__imp_H5E_LINKCOUNT_g;
    pub static H5E_VERSION: &'static id_t = &__imp_H5E_VERSION_g;
    pub static H5E_ALIGNMENT: &'static id_t = &__imp_H5E_ALIGNMENT_g;
    pub static H5E_BADMESG: &'static id_t = &__imp_H5E_BADMESG_g;
    pub static H5E_CANTDELETE: &'static id_t = &__imp_H5E_CANTDELETE_g;
    pub static H5E_BADITER: &'static id_t = &__imp_H5E_BADITER_g;
    pub static H5E_CANTPACK: &'static id_t = &__imp_H5E_CANTPACK_g;
    pub static H5E_CANTRESET: &'static id_t = &__imp_H5E_CANTRESET_g;
    pub static H5E_CANTRENAME: &'static id_t = &__imp_H5E_CANTRENAME_g;
    pub static H5E_SYSERRSTR: &'static id_t = &__imp_H5E_SYSERRSTR_g;
    pub static H5E_NOFILTER: &'static id_t = &__imp_H5E_NOFILTER_g;
    pub static H5E_CALLBACK: &'static id_t = &__imp_H5E_CALLBACK_g;
    pub static H5E_CANAPPLY: &'static id_t = &__imp_H5E_CANAPPLY_g;
    pub static H5E_SETLOCAL: &'static id_t = &__imp_H5E_SETLOCAL_g;
    pub static H5E_NOENCODER: &'static id_t = &__imp_H5E_NOENCODER_g;
    pub static H5E_CANTFILTER: &'static id_t = &__imp_H5E_CANTFILTER_g;
    pub static H5E_CANTOPENOBJ: &'static id_t = &__imp_H5E_CANTOPENOBJ_g;
    pub static H5E_CANTCLOSEOBJ: &'static id_t = &__imp_H5E_CANTCLOSEOBJ_g;
    pub static H5E_COMPLEN: &'static id_t = &__imp_H5E_COMPLEN_g;
    pub static H5E_PATH: &'static id_t = &__imp_H5E_PATH_g;
    pub static H5E_NONE_MINOR: &'static id_t = &__imp_H5E_NONE_MINOR_g;
    pub static H5E_OPENERROR: &'static id_t = &__imp_H5E_OPENERROR_g;
    pub static H5E_FILEEXISTS: &'static id_t = &__imp_H5E_FILEEXISTS_g;
    pub static H5E_FILEOPEN: &'static id_t = &__imp_H5E_FILEOPEN_g;
    pub static H5E_CANTCREATE: &'static id_t = &__imp_H5E_CANTCREATE_g;
    pub static H5E_CANTOPENFILE: &'static id_t = &__imp_H5E_CANTOPENFILE_g;
    pub static H5E_CANTCLOSEFILE: &'static id_t = &__imp_H5E_CANTCLOSEFILE_g;
    pub static H5E_NOTHDF5: &'static id_t = &__imp_H5E_NOTHDF5_g;
    pub static H5E_BADFILE: &'static id_t = &__imp_H5E_BADFILE_g;
    pub static H5E_TRUNCATED: &'static id_t = &__imp_H5E_TRUNCATED_g;
    pub static H5E_MOUNT: &'static id_t = &__imp_H5E_MOUNT_g;
    pub static H5E_BADATOM: &'static id_t = &__imp_H5E_BADATOM_g;
    pub static H5E_BADGROUP: &'static id_t = &__imp_H5E_BADGROUP_g;
    pub static H5E_CANTREGISTER: &'static id_t = &__imp_H5E_CANTREGISTER_g;
    pub static H5E_CANTINC: &'static id_t = &__imp_H5E_CANTINC_g;
    pub static H5E_CANTDEC: &'static id_t = &__imp_H5E_CANTDEC_g;
    pub static H5E_NOIDS: &'static id_t = &__imp_H5E_NOIDS_g;
    pub static H5E_CANTFLUSH: &'static id_t = &__imp_H5E_CANTFLUSH_g;
    pub static H5E_CANTSERIALIZE: &'static id_t = &__imp_H5E_CANTSERIALIZE_g;
    pub static H5E_CANTLOAD: &'static id_t = &__imp_H5E_CANTLOAD_g;
    pub static H5E_PROTECT: &'static id_t = &__imp_H5E_PROTECT_g;
    pub static H5E_NOTCACHED: &'static id_t = &__imp_H5E_NOTCACHED_g;
    pub static H5E_SYSTEM: &'static id_t = &__imp_H5E_SYSTEM_g;
    pub static H5E_CANTINS: &'static id_t = &__imp_H5E_CANTINS_g;
    pub static H5E_CANTPROTECT: &'static id_t = &__imp_H5E_CANTPROTECT_g;
    pub static H5E_CANTUNPROTECT: &'static id_t = &__imp_H5E_CANTUNPROTECT_g;
    pub static H5E_CANTPIN: &'static id_t = &__imp_H5E_CANTPIN_g;
    pub static H5E_CANTUNPIN: &'static id_t = &__imp_H5E_CANTUNPIN_g;
    pub static H5E_CANTMARKDIRTY: &'static id_t = &__imp_H5E_CANTMARKDIRTY_g;
    pub static H5E_CANTDIRTY: &'static id_t = &__imp_H5E_CANTDIRTY_g;
    pub static H5E_CANTEXPUNGE: &'static id_t = &__imp_H5E_CANTEXPUNGE_g;
    pub static H5E_CANTRESIZE: &'static id_t = &__imp_H5E_CANTRESIZE_g;
    pub static H5E_TRAVERSE: &'static id_t = &__imp_H5E_TRAVERSE_g;
    pub static H5E_NLINKS: &'static id_t = &__imp_H5E_NLINKS_g;
    pub static H5E_NOTREGISTERED: &'static id_t = &__imp_H5E_NOTREGISTERED_g;
    pub static H5E_CANTMOVE: &'static id_t = &__imp_H5E_CANTMOVE_g;
    pub static H5E_CANTSORT: &'static id_t = &__imp_H5E_CANTSORT_g;
    pub static H5E_MPI: &'static id_t = &__imp_H5E_MPI_g;
    pub static H5E_MPIERRSTR: &'static id_t = &__imp_H5E_MPIERRSTR_g;
    pub static H5E_CANTRECV: &'static id_t = &__imp_H5E_CANTRECV_g;
    pub static H5E_CANTCLIP: &'static id_t = &__imp_H5E_CANTCLIP_g;
    pub static H5E_CANTCOUNT: &'static id_t = &__imp_H5E_CANTCOUNT_g;
    pub static H5E_CANTSELECT: &'static id_t = &__imp_H5E_CANTSELECT_g;
    pub static H5E_CANTNEXT: &'static id_t = &__imp_H5E_CANTNEXT_g;
    pub static H5E_BADSELECT: &'static id_t = &__imp_H5E_BADSELECT_g;
    pub static H5E_CANTCOMPARE: &'static id_t = &__imp_H5E_CANTCOMPARE_g;
    pub static H5E_UNINITIALIZED: &'static id_t = &__imp_H5E_UNINITIALIZED_g;
    pub static H5E_UNSUPPORTED: &'static id_t = &__imp_H5E_UNSUPPORTED_g;
    pub static H5E_BADTYPE: &'static id_t = &__imp_H5E_BADTYPE_g;
    pub static H5E_BADRANGE: &'static id_t = &__imp_H5E_BADRANGE_g;
    pub static H5E_BADVALUE: &'static id_t = &__imp_H5E_BADVALUE_g;
    pub static H5E_NOTFOUND: &'static id_t = &__imp_H5E_NOTFOUND_g;
    pub static H5E_EXISTS: &'static id_t = &__imp_H5E_EXISTS_g;
    pub static H5E_CANTENCODE: &'static id_t = &__imp_H5E_CANTENCODE_g;
    pub static H5E_CANTDECODE: &'static id_t = &__imp_H5E_CANTDECODE_g;
    pub static H5E_CANTSPLIT: &'static id_t = &__imp_H5E_CANTSPLIT_g;
    pub static H5E_CANTREDISTRIBUTE: &'static id_t = &__imp_H5E_CANTREDISTRIBUTE_g;
    pub static H5E_CANTSWAP: &'static id_t = &__imp_H5E_CANTSWAP_g;
    pub static H5E_CANTINSERT: &'static id_t = &__imp_H5E_CANTINSERT_g;
    pub static H5E_CANTLIST: &'static id_t = &__imp_H5E_CANTLIST_g;
    pub static H5E_CANTMODIFY: &'static id_t = &__imp_H5E_CANTMODIFY_g;
    pub static H5E_CANTREMOVE: &'static id_t = &__imp_H5E_CANTREMOVE_g;
    pub static H5E_CANTCONVERT: &'static id_t = &__imp_H5E_CANTCONVERT_g;
    pub static H5E_BADSIZE: &'static id_t = &__imp_H5E_BADSIZE_g;
}
