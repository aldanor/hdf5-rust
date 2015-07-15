use ffi::h5::{haddr_t, H5open};
use ffi::h5fd::{H5FD_core_init, H5FD_sec2_init, H5FD_stdio_init};
use ffi::h5i::hid_t;

use ffi::h5e::*;
use ffi::h5t::*;

use std::mem;

pub use self::os::*;

macro_rules! link_hid {
    ($rust_name:ident, $c_name:ident) => {
        lazy_static! {
            pub static ref $rust_name: hid_t = {
                h5lock!(H5open());
                $c_name
            };
        }
    }
}

// Datatypes
link_hid!(H5T_IEEE_F32BE,              H5T_IEEE_F32BE_g);
link_hid!(H5T_IEEE_F32LE,              H5T_IEEE_F32LE_g);
link_hid!(H5T_IEEE_F64BE,              H5T_IEEE_F64BE_g);
link_hid!(H5T_IEEE_F64LE,              H5T_IEEE_F64LE_g);
link_hid!(H5T_STD_I8BE,                H5T_STD_I8BE_g);
link_hid!(H5T_STD_I8LE,                H5T_STD_I8LE_g);
link_hid!(H5T_STD_I16BE,               H5T_STD_I16BE_g);
link_hid!(H5T_STD_I16LE,               H5T_STD_I16LE_g);
link_hid!(H5T_STD_I32BE,               H5T_STD_I32BE_g);
link_hid!(H5T_STD_I32LE,               H5T_STD_I32LE_g);
link_hid!(H5T_STD_I64BE,               H5T_STD_I64BE_g);
link_hid!(H5T_STD_I64LE,               H5T_STD_I64LE_g);
link_hid!(H5T_STD_U8BE,                H5T_STD_U8BE_g);
link_hid!(H5T_STD_U8LE,                H5T_STD_U8LE_g);
link_hid!(H5T_STD_U16BE,               H5T_STD_U16BE_g);
link_hid!(H5T_STD_U16LE,               H5T_STD_U16LE_g);
link_hid!(H5T_STD_U32BE,               H5T_STD_U32BE_g);
link_hid!(H5T_STD_U32LE,               H5T_STD_U32LE_g);
link_hid!(H5T_STD_U64BE,               H5T_STD_U64BE_g);
link_hid!(H5T_STD_U64LE,               H5T_STD_U64LE_g);
link_hid!(H5T_STD_B8BE,                H5T_STD_B8BE_g);
link_hid!(H5T_STD_B8LE,                H5T_STD_B8LE_g);
link_hid!(H5T_STD_B16BE,               H5T_STD_B16BE_g);
link_hid!(H5T_STD_B16LE,               H5T_STD_B16LE_g);
link_hid!(H5T_STD_B32BE,               H5T_STD_B32BE_g);
link_hid!(H5T_STD_B32LE,               H5T_STD_B32LE_g);
link_hid!(H5T_STD_B64BE,               H5T_STD_B64BE_g);
link_hid!(H5T_STD_B64LE,               H5T_STD_B64LE_g);
link_hid!(H5T_STD_REF_OBJ,             H5T_STD_REF_OBJ_g);
link_hid!(H5T_STD_REF_DSETREG,         H5T_STD_REF_DSETREG_g);
link_hid!(H5T_UNIX_D32BE,              H5T_UNIX_D32BE_g);
link_hid!(H5T_UNIX_D32LE,              H5T_UNIX_D32LE_g);
link_hid!(H5T_UNIX_D64BE,              H5T_UNIX_D64BE_g);
link_hid!(H5T_UNIX_D64LE,              H5T_UNIX_D64LE_g);
link_hid!(H5T_C_S1,                    H5T_C_S1_g);
link_hid!(H5T_FORTRAN_S1,              H5T_FORTRAN_S1_g);
link_hid!(H5T_VAX_F32,                 H5T_VAX_F32_g);
link_hid!(H5T_VAX_F64,                 H5T_VAX_F64_g);
link_hid!(H5T_NATIVE_SCHAR,            H5T_NATIVE_SCHAR_g);
link_hid!(H5T_NATIVE_UCHAR,            H5T_NATIVE_UCHAR_g);
link_hid!(H5T_NATIVE_SHORT,            H5T_NATIVE_SHORT_g);
link_hid!(H5T_NATIVE_USHORT,           H5T_NATIVE_USHORT_g);
link_hid!(H5T_NATIVE_INT,              H5T_NATIVE_INT_g);
link_hid!(H5T_NATIVE_UINT,             H5T_NATIVE_UINT_g);
link_hid!(H5T_NATIVE_LONG,             H5T_NATIVE_LONG_g);
link_hid!(H5T_NATIVE_ULONG,            H5T_NATIVE_ULONG_g);
link_hid!(H5T_NATIVE_LLONG,            H5T_NATIVE_LLONG_g);
link_hid!(H5T_NATIVE_ULLONG,           H5T_NATIVE_ULLONG_g);
link_hid!(H5T_NATIVE_FLOAT,            H5T_NATIVE_FLOAT_g);
link_hid!(H5T_NATIVE_DOUBLE,           H5T_NATIVE_DOUBLE_g);
link_hid!(H5T_NATIVE_LDOUBLE,          H5T_NATIVE_LDOUBLE_g);
link_hid!(H5T_NATIVE_B8,               H5T_NATIVE_B8_g);
link_hid!(H5T_NATIVE_B16,              H5T_NATIVE_B16_g);
link_hid!(H5T_NATIVE_B32,              H5T_NATIVE_B32_g);
link_hid!(H5T_NATIVE_B64,              H5T_NATIVE_B64_g);
link_hid!(H5T_NATIVE_OPAQUE,           H5T_NATIVE_OPAQUE_g);
link_hid!(H5T_NATIVE_HADDR,            H5T_NATIVE_HADDR_g);
link_hid!(H5T_NATIVE_HSIZE,            H5T_NATIVE_HSIZE_g);
link_hid!(H5T_NATIVE_HSSIZE,           H5T_NATIVE_HSSIZE_g);
link_hid!(H5T_NATIVE_HERR,             H5T_NATIVE_HERR_g);
link_hid!(H5T_NATIVE_HBOOL,            H5T_NATIVE_HBOOL_g);
link_hid!(H5T_NATIVE_INT8,             H5T_NATIVE_INT8_g);
link_hid!(H5T_NATIVE_UINT8,            H5T_NATIVE_UINT8_g);
link_hid!(H5T_NATIVE_INT_LEAST8,       H5T_NATIVE_INT_LEAST8_g);
link_hid!(H5T_NATIVE_UINT_LEAST8,      H5T_NATIVE_UINT_LEAST8_g);
link_hid!(H5T_NATIVE_INT_FAST8,        H5T_NATIVE_INT_FAST8_g);
link_hid!(H5T_NATIVE_UINT_FAST8,       H5T_NATIVE_UINT_FAST8_g);
link_hid!(H5T_NATIVE_INT16,            H5T_NATIVE_INT16_g);
link_hid!(H5T_NATIVE_UINT16,           H5T_NATIVE_UINT16_g);
link_hid!(H5T_NATIVE_INT_LEAST16,      H5T_NATIVE_INT_LEAST16_g);
link_hid!(H5T_NATIVE_UINT_LEAST16,     H5T_NATIVE_UINT_LEAST16_g);
link_hid!(H5T_NATIVE_INT_FAST16,       H5T_NATIVE_INT_FAST16_g);
link_hid!(H5T_NATIVE_UINT_FAST16,      H5T_NATIVE_UINT_FAST16_g);
link_hid!(H5T_NATIVE_INT32,            H5T_NATIVE_INT32_g);
link_hid!(H5T_NATIVE_UINT32,           H5T_NATIVE_UINT32_g);
link_hid!(H5T_NATIVE_INT_LEAST32,      H5T_NATIVE_INT_LEAST32_g);
link_hid!(H5T_NATIVE_UINT_LEAST32,     H5T_NATIVE_UINT_LEAST32_g);
link_hid!(H5T_NATIVE_INT_FAST32,       H5T_NATIVE_INT_FAST32_g);
link_hid!(H5T_NATIVE_UINT_FAST32,      H5T_NATIVE_UINT_FAST32_g);
link_hid!(H5T_NATIVE_INT64,            H5T_NATIVE_INT64_g);
link_hid!(H5T_NATIVE_UINT64,           H5T_NATIVE_UINT64_g);
link_hid!(H5T_NATIVE_INT_LEAST64,      H5T_NATIVE_INT_LEAST64_g);
link_hid!(H5T_NATIVE_UINT_LEAST64,     H5T_NATIVE_UINT_LEAST64_g);
link_hid!(H5T_NATIVE_INT_FAST64,       H5T_NATIVE_INT_FAST64_g);
link_hid!(H5T_NATIVE_UINT_FAST64,      H5T_NATIVE_UINT_FAST64_g);

#[cfg(target_os = "linux")]
mod os {
    use ffi::h5i::hid_t;
    use ffi::h5::H5open;
    use ffi::h5p::*;

    // Property list classes
    link_hid!(H5P_ROOT,                    H5P_CLS_ROOT_g);
    link_hid!(H5P_OBJECT_CREATE,           H5P_CLS_OBJECT_CREATE_g);
    link_hid!(H5P_FILE_CREATE,             H5P_CLS_FILE_CREATE_g);
    link_hid!(H5P_FILE_ACCESS,             H5P_CLS_FILE_ACCESS_g);
    link_hid!(H5P_DATASET_CREATE,          H5P_CLS_DATASET_CREATE_g);
    link_hid!(H5P_DATASET_ACCESS,          H5P_CLS_DATASET_ACCESS_g);
    link_hid!(H5P_DATASET_XFER,            H5P_CLS_DATASET_XFER_g);
    link_hid!(H5P_FILE_MOUNT,              H5P_CLS_FILE_MOUNT_g);
    link_hid!(H5P_GROUP_CREATE,            H5P_CLS_GROUP_CREATE_g);
    link_hid!(H5P_GROUP_ACCESS,            H5P_CLS_GROUP_ACCESS_g);
    link_hid!(H5P_DATATYPE_CREATE,         H5P_CLS_DATATYPE_CREATE_g);
    link_hid!(H5P_DATATYPE_ACCESS,         H5P_CLS_DATATYPE_ACCESS_g);
    link_hid!(H5P_STRING_CREATE,           H5P_CLS_STRING_CREATE_g);
    link_hid!(H5P_ATTRIBUTE_CREATE,        H5P_CLS_ATTRIBUTE_CREATE_g);
    link_hid!(H5P_OBJECT_COPY,             H5P_CLS_OBJECT_COPY_g);
    link_hid!(H5P_LINK_CREATE,             H5P_CLS_LINK_CREATE_g);
    link_hid!(H5P_LINK_ACCESS,             H5P_CLS_LINK_ACCESS_g);

    // Default property lists
    link_hid!(H5P_LST_FILE_CREATE_ID,      H5P_LST_FILE_CREATE_g);
    link_hid!(H5P_LST_FILE_ACCESS_ID,      H5P_LST_FILE_ACCESS_g);
    link_hid!(H5P_LST_DATASET_CREATE_ID,   H5P_LST_DATASET_CREATE_g);
    link_hid!(H5P_LST_DATASET_ACCESS_ID,   H5P_LST_DATASET_ACCESS_g);
    link_hid!(H5P_LST_DATASET_XFER_ID,     H5P_LST_DATASET_XFER_g);
    link_hid!(H5P_LST_FILE_MOUNT_ID,       H5P_LST_FILE_MOUNT_g);
    link_hid!(H5P_LST_GROUP_CREATE_ID,     H5P_LST_GROUP_CREATE_g);
    link_hid!(H5P_LST_GROUP_ACCESS_ID,     H5P_LST_GROUP_ACCESS_g);
    link_hid!(H5P_LST_DATATYPE_CREATE_ID,  H5P_LST_DATATYPE_CREATE_g);
    link_hid!(H5P_LST_DATATYPE_ACCESS_ID,  H5P_LST_DATATYPE_ACCESS_g);
    link_hid!(H5P_LST_ATTRIBUTE_CREATE_ID, H5P_LST_ATTRIBUTE_CREATE_g);
    link_hid!(H5P_LST_OBJECT_COPY_ID,      H5P_LST_OBJECT_COPY_g);
    link_hid!(H5P_LST_LINK_CREATE_ID,      H5P_LST_LINK_CREATE_g);
    link_hid!(H5P_LST_LINK_ACCESS_ID,      H5P_LST_LINK_ACCESS_g);
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod os {
    use ffi::h5i::hid_t;
    use ffi::h5::H5open;
    use ffi::h5p::*;

    // Property list classes
    link_hid!(H5P_ROOT,                    H5P_CLS_ROOT_ID_g);
    link_hid!(H5P_OBJECT_CREATE,           H5P_CLS_OBJECT_CREATE_ID_g);
    link_hid!(H5P_FILE_CREATE,             H5P_CLS_FILE_CREATE_ID_g);
    link_hid!(H5P_FILE_ACCESS,             H5P_CLS_FILE_ACCESS_ID_g);
    link_hid!(H5P_DATASET_CREATE,          H5P_CLS_DATASET_CREATE_ID_g);
    link_hid!(H5P_DATASET_ACCESS,          H5P_CLS_DATASET_ACCESS_ID_g);
    link_hid!(H5P_DATASET_XFER,            H5P_CLS_DATASET_XFER_ID_g);
    link_hid!(H5P_FILE_MOUNT,              H5P_CLS_FILE_MOUNT_ID_g);
    link_hid!(H5P_GROUP_CREATE,            H5P_CLS_GROUP_CREATE_ID_g);
    link_hid!(H5P_GROUP_ACCESS,            H5P_CLS_GROUP_ACCESS_ID_g);
    link_hid!(H5P_DATATYPE_CREATE,         H5P_CLS_DATATYPE_CREATE_ID_g);
    link_hid!(H5P_DATATYPE_ACCESS,         H5P_CLS_DATATYPE_ACCESS_ID_g);
    link_hid!(H5P_STRING_CREATE,           H5P_CLS_STRING_CREATE_ID_g);
    link_hid!(H5P_ATTRIBUTE_CREATE,        H5P_CLS_ATTRIBUTE_CREATE_ID_g);
    link_hid!(H5P_OBJECT_COPY,             H5P_CLS_OBJECT_COPY_ID_g);
    link_hid!(H5P_LINK_CREATE,             H5P_CLS_LINK_CREATE_ID_g);
    link_hid!(H5P_LINK_ACCESS,             H5P_CLS_LINK_ACCESS_ID_g);

    // Default property lists
    link_hid!(H5P_LST_FILE_CREATE_ID,      H5P_LST_FILE_CREATE_ID_g);
    link_hid!(H5P_LST_FILE_ACCESS_ID,      H5P_LST_FILE_ACCESS_ID_g);
    link_hid!(H5P_LST_DATASET_CREATE_ID,   H5P_LST_DATASET_CREATE_ID_g);
    link_hid!(H5P_LST_DATASET_ACCESS_ID,   H5P_LST_DATASET_ACCESS_ID_g);
    link_hid!(H5P_LST_DATASET_XFER_ID,     H5P_LST_DATASET_XFER_ID_g);
    link_hid!(H5P_LST_FILE_MOUNT_ID,       H5P_LST_FILE_MOUNT_ID_g);
    link_hid!(H5P_LST_GROUP_CREATE_ID,     H5P_LST_GROUP_CREATE_ID_g);
    link_hid!(H5P_LST_GROUP_ACCESS_ID,     H5P_LST_GROUP_ACCESS_ID_g);
    link_hid!(H5P_LST_DATATYPE_CREATE_ID,  H5P_LST_DATATYPE_CREATE_ID_g);
    link_hid!(H5P_LST_DATATYPE_ACCESS_ID,  H5P_LST_DATATYPE_ACCESS_ID_g);
    link_hid!(H5P_LST_ATTRIBUTE_CREATE_ID, H5P_LST_ATTRIBUTE_CREATE_ID_g);
    link_hid!(H5P_LST_OBJECT_COPY_ID,      H5P_LST_OBJECT_COPY_ID_g);
    link_hid!(H5P_LST_LINK_CREATE_ID,      H5P_LST_LINK_CREATE_ID_g);
    link_hid!(H5P_LST_LINK_ACCESS_ID,      H5P_LST_LINK_ACCESS_ID_g);
}

// Error class
link_hid!(H5E_ERR_CLS,                 H5E_ERR_CLS_g);

// Errors
link_hid!(H5E_DATASET,                 H5E_DATASET_g);
link_hid!(H5E_FUNC,                    H5E_FUNC_g);
link_hid!(H5E_STORAGE,                 H5E_STORAGE_g);
link_hid!(H5E_FILE,                    H5E_FILE_g);
link_hid!(H5E_SOHM,                    H5E_SOHM_g);
link_hid!(H5E_SYM,                     H5E_SYM_g);
link_hid!(H5E_PLUGIN,                  H5E_PLUGIN_g);
link_hid!(H5E_VFL,                     H5E_VFL_g);
link_hid!(H5E_INTERNAL,                H5E_INTERNAL_g);
link_hid!(H5E_BTREE,                   H5E_BTREE_g);
link_hid!(H5E_REFERENCE,               H5E_REFERENCE_g);
link_hid!(H5E_DATASPACE,               H5E_DATASPACE_g);
link_hid!(H5E_RESOURCE,                H5E_RESOURCE_g);
link_hid!(H5E_PLIST,                   H5E_PLIST_g);
link_hid!(H5E_LINK,                    H5E_LINK_g);
link_hid!(H5E_DATATYPE,                H5E_DATATYPE_g);
link_hid!(H5E_RS,                      H5E_RS_g);
link_hid!(H5E_HEAP,                    H5E_HEAP_g);
link_hid!(H5E_OHDR,                    H5E_OHDR_g);
link_hid!(H5E_ATOM,                    H5E_ATOM_g);
link_hid!(H5E_ATTR,                    H5E_ATTR_g);
link_hid!(H5E_NONE_MAJOR,              H5E_NONE_MAJOR_g);
link_hid!(H5E_IO,                      H5E_IO_g);
link_hid!(H5E_SLIST,                   H5E_SLIST_g);
link_hid!(H5E_EFL,                     H5E_EFL_g);
link_hid!(H5E_TST,                     H5E_TST_g);
link_hid!(H5E_ARGS,                    H5E_ARGS_g);
link_hid!(H5E_ERROR,                   H5E_ERROR_g);
link_hid!(H5E_PLINE,                   H5E_PLINE_g);
link_hid!(H5E_FSPACE,                  H5E_FSPACE_g);
link_hid!(H5E_CACHE,                   H5E_CACHE_g);
link_hid!(H5E_SEEKERROR,               H5E_SEEKERROR_g);
link_hid!(H5E_READERROR,               H5E_READERROR_g);
link_hid!(H5E_WRITEERROR,              H5E_WRITEERROR_g);
link_hid!(H5E_CLOSEERROR,              H5E_CLOSEERROR_g);
link_hid!(H5E_OVERFLOW,                H5E_OVERFLOW_g);
link_hid!(H5E_FCNTL,                   H5E_FCNTL_g);
link_hid!(H5E_NOSPACE,                 H5E_NOSPACE_g);
link_hid!(H5E_CANTALLOC,               H5E_CANTALLOC_g);
link_hid!(H5E_CANTCOPY,                H5E_CANTCOPY_g);
link_hid!(H5E_CANTFREE,                H5E_CANTFREE_g);
link_hid!(H5E_ALREADYEXISTS,           H5E_ALREADYEXISTS_g);
link_hid!(H5E_CANTLOCK,                H5E_CANTLOCK_g);
link_hid!(H5E_CANTUNLOCK,              H5E_CANTUNLOCK_g);
link_hid!(H5E_CANTGC,                  H5E_CANTGC_g);
link_hid!(H5E_CANTGETSIZE,             H5E_CANTGETSIZE_g);
link_hid!(H5E_OBJOPEN,                 H5E_OBJOPEN_g);
link_hid!(H5E_CANTRESTORE,             H5E_CANTRESTORE_g);
link_hid!(H5E_CANTCOMPUTE,             H5E_CANTCOMPUTE_g);
link_hid!(H5E_CANTEXTEND,              H5E_CANTEXTEND_g);
link_hid!(H5E_CANTATTACH,              H5E_CANTATTACH_g);
link_hid!(H5E_CANTUPDATE,              H5E_CANTUPDATE_g);
link_hid!(H5E_CANTOPERATE,             H5E_CANTOPERATE_g);
link_hid!(H5E_CANTINIT,                H5E_CANTINIT_g);
link_hid!(H5E_ALREADYINIT,             H5E_ALREADYINIT_g);
link_hid!(H5E_CANTRELEASE,             H5E_CANTRELEASE_g);
link_hid!(H5E_CANTGET,                 H5E_CANTGET_g);
link_hid!(H5E_CANTSET,                 H5E_CANTSET_g);
link_hid!(H5E_DUPCLASS,                H5E_DUPCLASS_g);
link_hid!(H5E_SETDISALLOWED,           H5E_SETDISALLOWED_g);
link_hid!(H5E_CANTMERGE,               H5E_CANTMERGE_g);
link_hid!(H5E_CANTREVIVE,              H5E_CANTREVIVE_g);
link_hid!(H5E_CANTSHRINK,              H5E_CANTSHRINK_g);
link_hid!(H5E_LINKCOUNT,               H5E_LINKCOUNT_g);
link_hid!(H5E_VERSION,                 H5E_VERSION_g);
link_hid!(H5E_ALIGNMENT,               H5E_ALIGNMENT_g);
link_hid!(H5E_BADMESG,                 H5E_BADMESG_g);
link_hid!(H5E_CANTDELETE,              H5E_CANTDELETE_g);
link_hid!(H5E_BADITER,                 H5E_BADITER_g);
link_hid!(H5E_CANTPACK,                H5E_CANTPACK_g);
link_hid!(H5E_CANTRESET,               H5E_CANTRESET_g);
link_hid!(H5E_CANTRENAME,              H5E_CANTRENAME_g);
link_hid!(H5E_SYSERRSTR,               H5E_SYSERRSTR_g);
link_hid!(H5E_NOFILTER,                H5E_NOFILTER_g);
link_hid!(H5E_CALLBACK,                H5E_CALLBACK_g);
link_hid!(H5E_CANAPPLY,                H5E_CANAPPLY_g);
link_hid!(H5E_SETLOCAL,                H5E_SETLOCAL_g);
link_hid!(H5E_NOENCODER,               H5E_NOENCODER_g);
link_hid!(H5E_CANTFILTER,              H5E_CANTFILTER_g);
link_hid!(H5E_CANTOPENOBJ,             H5E_CANTOPENOBJ_g);
link_hid!(H5E_CANTCLOSEOBJ,            H5E_CANTCLOSEOBJ_g);
link_hid!(H5E_COMPLEN,                 H5E_COMPLEN_g);
link_hid!(H5E_PATH,                    H5E_PATH_g);
link_hid!(H5E_NONE_MINOR,              H5E_NONE_MINOR_g);
link_hid!(H5E_OPENERROR,               H5E_OPENERROR_g);
link_hid!(H5E_FILEEXISTS,              H5E_FILEEXISTS_g);
link_hid!(H5E_FILEOPEN,                H5E_FILEOPEN_g);
link_hid!(H5E_CANTCREATE,              H5E_CANTCREATE_g);
link_hid!(H5E_CANTOPENFILE,            H5E_CANTOPENFILE_g);
link_hid!(H5E_CANTCLOSEFILE,           H5E_CANTCLOSEFILE_g);
link_hid!(H5E_NOTHDF5,                 H5E_NOTHDF5_g);
link_hid!(H5E_BADFILE,                 H5E_BADFILE_g);
link_hid!(H5E_TRUNCATED,               H5E_TRUNCATED_g);
link_hid!(H5E_MOUNT,                   H5E_MOUNT_g);
link_hid!(H5E_BADATOM,                 H5E_BADATOM_g);
link_hid!(H5E_BADGROUP,                H5E_BADGROUP_g);
link_hid!(H5E_CANTREGISTER,            H5E_CANTREGISTER_g);
link_hid!(H5E_CANTINC,                 H5E_CANTINC_g);
link_hid!(H5E_CANTDEC,                 H5E_CANTDEC_g);
link_hid!(H5E_NOIDS,                   H5E_NOIDS_g);
link_hid!(H5E_CANTFLUSH,               H5E_CANTFLUSH_g);
link_hid!(H5E_CANTSERIALIZE,           H5E_CANTSERIALIZE_g);
link_hid!(H5E_CANTLOAD,                H5E_CANTLOAD_g);
link_hid!(H5E_PROTECT,                 H5E_PROTECT_g);
link_hid!(H5E_NOTCACHED,               H5E_NOTCACHED_g);
link_hid!(H5E_SYSTEM,                  H5E_SYSTEM_g);
link_hid!(H5E_CANTINS,                 H5E_CANTINS_g);
link_hid!(H5E_CANTPROTECT,             H5E_CANTPROTECT_g);
link_hid!(H5E_CANTUNPROTECT,           H5E_CANTUNPROTECT_g);
link_hid!(H5E_CANTPIN,                 H5E_CANTPIN_g);
link_hid!(H5E_CANTUNPIN,               H5E_CANTUNPIN_g);
link_hid!(H5E_CANTMARKDIRTY,           H5E_CANTMARKDIRTY_g);
link_hid!(H5E_CANTDIRTY,               H5E_CANTDIRTY_g);
link_hid!(H5E_CANTEXPUNGE,             H5E_CANTEXPUNGE_g);
link_hid!(H5E_CANTRESIZE,              H5E_CANTRESIZE_g);
link_hid!(H5E_TRAVERSE,                H5E_TRAVERSE_g);
link_hid!(H5E_NLINKS,                  H5E_NLINKS_g);
link_hid!(H5E_NOTREGISTERED,           H5E_NOTREGISTERED_g);
link_hid!(H5E_CANTMOVE,                H5E_CANTMOVE_g);
link_hid!(H5E_CANTSORT,                H5E_CANTSORT_g);
link_hid!(H5E_MPI,                     H5E_MPI_g);
link_hid!(H5E_MPIERRSTR,               H5E_MPIERRSTR_g);
link_hid!(H5E_CANTRECV,                H5E_CANTRECV_g);
link_hid!(H5E_CANTCLIP,                H5E_CANTCLIP_g);
link_hid!(H5E_CANTCOUNT,               H5E_CANTCOUNT_g);
link_hid!(H5E_CANTSELECT,              H5E_CANTSELECT_g);
link_hid!(H5E_CANTNEXT,                H5E_CANTNEXT_g);
link_hid!(H5E_BADSELECT,               H5E_BADSELECT_g);
link_hid!(H5E_CANTCOMPARE,             H5E_CANTCOMPARE_g);
link_hid!(H5E_UNINITIALIZED,           H5E_UNINITIALIZED_g);
link_hid!(H5E_UNSUPPORTED,             H5E_UNSUPPORTED_g);
link_hid!(H5E_BADTYPE,                 H5E_BADTYPE_g);
link_hid!(H5E_BADRANGE,                H5E_BADRANGE_g);
link_hid!(H5E_BADVALUE,                H5E_BADVALUE_g);
link_hid!(H5E_NOTFOUND,                H5E_NOTFOUND_g);
link_hid!(H5E_EXISTS,                  H5E_EXISTS_g);
link_hid!(H5E_CANTENCODE,              H5E_CANTENCODE_g);
link_hid!(H5E_CANTDECODE,              H5E_CANTDECODE_g);
link_hid!(H5E_CANTSPLIT,               H5E_CANTSPLIT_g);
link_hid!(H5E_CANTREDISTRIBUTE,        H5E_CANTREDISTRIBUTE_g);
link_hid!(H5E_CANTSWAP,                H5E_CANTSWAP_g);
link_hid!(H5E_CANTINSERT,              H5E_CANTINSERT_g);
link_hid!(H5E_CANTLIST,                H5E_CANTLIST_g);
link_hid!(H5E_CANTMODIFY,              H5E_CANTMODIFY_g);
link_hid!(H5E_CANTREMOVE,              H5E_CANTREMOVE_g);
link_hid!(H5E_CANTCONVERT,             H5E_CANTCONVERT_g);
link_hid!(H5E_BADSIZE,                 H5E_BADSIZE_g);

// H5R constants
lazy_static! {
    pub static ref H5R_OBJ_REF_BUF_SIZE: usize = { mem::size_of::<haddr_t>() };
    pub static ref H5R_DSET_REG_REF_BUF_SIZE: usize = { mem::size_of::<haddr_t>() + 4 };
}

// File drivers
lazy_static! {
    pub static ref H5FD_CORE: hid_t = unsafe { H5FD_core_init() };
    pub static ref H5FD_SEC2: hid_t = unsafe { H5FD_sec2_init() };
    pub static ref H5FD_STDIO: hid_t = unsafe { H5FD_stdio_init() };
}

#[cfg(test)]
mod tests
{
    use ffi::h5::haddr_t;
    use super::{H5T_IEEE_F32BE, H5T_NATIVE_INT, H5P_ROOT, H5P_LST_LINK_ACCESS_ID,
                H5E_ERR_CLS, H5E_DATASET, H5R_OBJ_REF_BUF_SIZE, H5R_DSET_REG_REF_BUF_SIZE};
    use std::mem;

    #[test]
    pub fn test_lazy_globals() {
        use ffi::h5i::H5I_INVALID_HID;

        assert!(*H5T_IEEE_F32BE != H5I_INVALID_HID);
        assert!(*H5T_NATIVE_INT != H5I_INVALID_HID);

        assert!(*H5P_ROOT != H5I_INVALID_HID);
        assert!(*H5P_LST_LINK_ACCESS_ID != H5I_INVALID_HID);

        assert!(*H5E_ERR_CLS != H5I_INVALID_HID);
        assert!(*H5E_DATASET != H5I_INVALID_HID);

        assert_eq!(*H5R_OBJ_REF_BUF_SIZE, mem::size_of::<haddr_t>());
        assert_eq!(*H5R_DSET_REG_REF_BUF_SIZE, mem::size_of::<haddr_t>() + 4);
    }
}
