#![allow(dead_code)]

use std::mem;

use lazy_static::lazy_static;

#[cfg(feature = "have-direct")]
use hdf5_sys::h5fd::H5FD_direct_init;
#[cfg(feature = "have-parallel")]
use hdf5_sys::h5fd::H5FD_mpio_init;
use hdf5_sys::h5fd::{
    H5FD_core_init, H5FD_family_init, H5FD_log_init, H5FD_multi_init, H5FD_sec2_init,
    H5FD_stdio_init,
};
use hdf5_sys::{h5e, h5p, h5t};

use crate::internal_prelude::*;

pub struct H5GlobalConstant(
    #[cfg(msvc_dll_indirection)] &'static usize,
    #[cfg(not(msvc_dll_indirection))] &'static hdf5_sys::h5i::hid_t,
);

impl std::ops::Deref for H5GlobalConstant {
    type Target = hdf5_sys::h5i::hid_t;
    fn deref(&self) -> &Self::Target {
        lazy_static::initialize(&crate::sync::LIBRARY_INIT);
        cfg_if::cfg_if! {
            if #[cfg(msvc_dll_indirection)] {
                let dll_ptr = self.0 as *const usize;
                let ptr: *const *const hdf5_sys::h5i::hid_t = dll_ptr.cast();
                unsafe {
                    &**ptr
                }
            } else {
                self.0
            }
        }
    }
}

macro_rules! link_hid {
    ($rust_name:ident, $c_name:path) => {
        pub static $rust_name: H5GlobalConstant = H5GlobalConstant($c_name);
    };
}

// Datatypes
link_hid!(H5T_IEEE_F32BE, h5t::H5T_IEEE_F32BE);
link_hid!(H5T_IEEE_F32LE, h5t::H5T_IEEE_F32LE);
link_hid!(H5T_IEEE_F64BE, h5t::H5T_IEEE_F64BE);
link_hid!(H5T_IEEE_F64LE, h5t::H5T_IEEE_F64LE);
link_hid!(H5T_STD_I8BE, h5t::H5T_STD_I8BE);
link_hid!(H5T_STD_I8LE, h5t::H5T_STD_I8LE);
link_hid!(H5T_STD_I16BE, h5t::H5T_STD_I16BE);
link_hid!(H5T_STD_I16LE, h5t::H5T_STD_I16LE);
link_hid!(H5T_STD_I32BE, h5t::H5T_STD_I32BE);
link_hid!(H5T_STD_I32LE, h5t::H5T_STD_I32LE);
link_hid!(H5T_STD_I64BE, h5t::H5T_STD_I64BE);
link_hid!(H5T_STD_I64LE, h5t::H5T_STD_I64LE);
link_hid!(H5T_STD_U8BE, h5t::H5T_STD_U8BE);
link_hid!(H5T_STD_U8LE, h5t::H5T_STD_U8LE);
link_hid!(H5T_STD_U16BE, h5t::H5T_STD_U16BE);
link_hid!(H5T_STD_U16LE, h5t::H5T_STD_U16LE);
link_hid!(H5T_STD_U32BE, h5t::H5T_STD_U32BE);
link_hid!(H5T_STD_U32LE, h5t::H5T_STD_U32LE);
link_hid!(H5T_STD_U64BE, h5t::H5T_STD_U64BE);
link_hid!(H5T_STD_U64LE, h5t::H5T_STD_U64LE);
link_hid!(H5T_STD_B8BE, h5t::H5T_STD_B8BE);
link_hid!(H5T_STD_B8LE, h5t::H5T_STD_B8LE);
link_hid!(H5T_STD_B16BE, h5t::H5T_STD_B16BE);
link_hid!(H5T_STD_B16LE, h5t::H5T_STD_B16LE);
link_hid!(H5T_STD_B32BE, h5t::H5T_STD_B32BE);
link_hid!(H5T_STD_B32LE, h5t::H5T_STD_B32LE);
link_hid!(H5T_STD_B64BE, h5t::H5T_STD_B64BE);
link_hid!(H5T_STD_B64LE, h5t::H5T_STD_B64LE);
link_hid!(H5T_STD_REF_OBJ, h5t::H5T_STD_REF_OBJ);
link_hid!(H5T_STD_REF_DSETREG, h5t::H5T_STD_REF_DSETREG);
link_hid!(H5T_UNIX_D32BE, h5t::H5T_UNIX_D32BE);
link_hid!(H5T_UNIX_D32LE, h5t::H5T_UNIX_D32LE);
link_hid!(H5T_UNIX_D64BE, h5t::H5T_UNIX_D64BE);
link_hid!(H5T_UNIX_D64LE, h5t::H5T_UNIX_D64LE);
link_hid!(H5T_C_S1, h5t::H5T_C_S1);
link_hid!(H5T_FORTRAN_S1, h5t::H5T_FORTRAN_S1);
link_hid!(H5T_VAX_F32, h5t::H5T_VAX_F32);
link_hid!(H5T_VAX_F64, h5t::H5T_VAX_F64);
link_hid!(H5T_NATIVE_SCHAR, h5t::H5T_NATIVE_SCHAR);
link_hid!(H5T_NATIVE_UCHAR, h5t::H5T_NATIVE_UCHAR);
link_hid!(H5T_NATIVE_SHORT, h5t::H5T_NATIVE_SHORT);
link_hid!(H5T_NATIVE_USHORT, h5t::H5T_NATIVE_USHORT);
link_hid!(H5T_NATIVE_INT, h5t::H5T_NATIVE_INT);
link_hid!(H5T_NATIVE_UINT, h5t::H5T_NATIVE_UINT);
link_hid!(H5T_NATIVE_LONG, h5t::H5T_NATIVE_LONG);
link_hid!(H5T_NATIVE_ULONG, h5t::H5T_NATIVE_ULONG);
link_hid!(H5T_NATIVE_LLONG, h5t::H5T_NATIVE_LLONG);
link_hid!(H5T_NATIVE_ULLONG, h5t::H5T_NATIVE_ULLONG);
link_hid!(H5T_NATIVE_FLOAT, h5t::H5T_NATIVE_FLOAT);
link_hid!(H5T_NATIVE_DOUBLE, h5t::H5T_NATIVE_DOUBLE);
link_hid!(H5T_NATIVE_LDOUBLE, h5t::H5T_NATIVE_LDOUBLE);
link_hid!(H5T_NATIVE_B8, h5t::H5T_NATIVE_B8);
link_hid!(H5T_NATIVE_B16, h5t::H5T_NATIVE_B16);
link_hid!(H5T_NATIVE_B32, h5t::H5T_NATIVE_B32);
link_hid!(H5T_NATIVE_B64, h5t::H5T_NATIVE_B64);
link_hid!(H5T_NATIVE_OPAQUE, h5t::H5T_NATIVE_OPAQUE);
link_hid!(H5T_NATIVE_HADDR, h5t::H5T_NATIVE_HADDR);
link_hid!(H5T_NATIVE_HSIZE, h5t::H5T_NATIVE_HSIZE);
link_hid!(H5T_NATIVE_HSSIZE, h5t::H5T_NATIVE_HSSIZE);
link_hid!(H5T_NATIVE_HERR, h5t::H5T_NATIVE_HERR);
link_hid!(H5T_NATIVE_HBOOL, h5t::H5T_NATIVE_HBOOL);
link_hid!(H5T_NATIVE_INT8, h5t::H5T_NATIVE_INT8);
link_hid!(H5T_NATIVE_UINT8, h5t::H5T_NATIVE_UINT8);
link_hid!(H5T_NATIVE_INT_LEAST8, h5t::H5T_NATIVE_INT_LEAST8);
link_hid!(H5T_NATIVE_UINT_LEAST8, h5t::H5T_NATIVE_UINT_LEAST8);
link_hid!(H5T_NATIVE_INT_FAST8, h5t::H5T_NATIVE_INT_FAST8);
link_hid!(H5T_NATIVE_UINT_FAST8, h5t::H5T_NATIVE_UINT_FAST8);
link_hid!(H5T_NATIVE_INT16, h5t::H5T_NATIVE_INT16);
link_hid!(H5T_NATIVE_UINT16, h5t::H5T_NATIVE_UINT16);
link_hid!(H5T_NATIVE_INT_LEAST16, h5t::H5T_NATIVE_INT_LEAST16);
link_hid!(H5T_NATIVE_UINT_LEAST16, h5t::H5T_NATIVE_UINT_LEAST16);
link_hid!(H5T_NATIVE_INT_FAST16, h5t::H5T_NATIVE_INT_FAST16);
link_hid!(H5T_NATIVE_UINT_FAST16, h5t::H5T_NATIVE_UINT_FAST16);
link_hid!(H5T_NATIVE_INT32, h5t::H5T_NATIVE_INT32);
link_hid!(H5T_NATIVE_UINT32, h5t::H5T_NATIVE_UINT32);
link_hid!(H5T_NATIVE_INT_LEAST32, h5t::H5T_NATIVE_INT_LEAST32);
link_hid!(H5T_NATIVE_UINT_LEAST32, h5t::H5T_NATIVE_UINT_LEAST32);
link_hid!(H5T_NATIVE_INT_FAST32, h5t::H5T_NATIVE_INT_FAST32);
link_hid!(H5T_NATIVE_UINT_FAST32, h5t::H5T_NATIVE_UINT_FAST32);
link_hid!(H5T_NATIVE_INT64, h5t::H5T_NATIVE_INT64);
link_hid!(H5T_NATIVE_UINT64, h5t::H5T_NATIVE_UINT64);
link_hid!(H5T_NATIVE_INT_LEAST64, h5t::H5T_NATIVE_INT_LEAST64);
link_hid!(H5T_NATIVE_UINT_LEAST64, h5t::H5T_NATIVE_UINT_LEAST64);
link_hid!(H5T_NATIVE_INT_FAST64, h5t::H5T_NATIVE_INT_FAST64);
link_hid!(H5T_NATIVE_UINT_FAST64, h5t::H5T_NATIVE_UINT_FAST64);

// Property list classes
link_hid!(H5P_ROOT, h5p::H5P_CLS_ROOT);
link_hid!(H5P_OBJECT_CREATE, h5p::H5P_CLS_OBJECT_CREATE);
link_hid!(H5P_FILE_CREATE, h5p::H5P_CLS_FILE_CREATE);
link_hid!(H5P_FILE_ACCESS, h5p::H5P_CLS_FILE_ACCESS);
link_hid!(H5P_DATASET_CREATE, h5p::H5P_CLS_DATASET_CREATE);
link_hid!(H5P_DATASET_ACCESS, h5p::H5P_CLS_DATASET_ACCESS);
link_hid!(H5P_DATASET_XFER, h5p::H5P_CLS_DATASET_XFER);
link_hid!(H5P_FILE_MOUNT, h5p::H5P_CLS_FILE_MOUNT);
link_hid!(H5P_GROUP_CREATE, h5p::H5P_CLS_GROUP_CREATE);
link_hid!(H5P_GROUP_ACCESS, h5p::H5P_CLS_GROUP_ACCESS);
link_hid!(H5P_DATATYPE_CREATE, h5p::H5P_CLS_DATATYPE_CREATE);
link_hid!(H5P_DATATYPE_ACCESS, h5p::H5P_CLS_DATATYPE_ACCESS);
link_hid!(H5P_STRING_CREATE, h5p::H5P_CLS_STRING_CREATE);
link_hid!(H5P_ATTRIBUTE_CREATE, h5p::H5P_CLS_ATTRIBUTE_CREATE);
link_hid!(H5P_OBJECT_COPY, h5p::H5P_CLS_OBJECT_COPY);
link_hid!(H5P_LINK_CREATE, h5p::H5P_CLS_LINK_CREATE);
link_hid!(H5P_LINK_ACCESS, h5p::H5P_CLS_LINK_ACCESS);

// Default property lists
link_hid!(H5P_LST_FILE_CREATE_ID, h5p::H5P_LST_FILE_CREATE);
link_hid!(H5P_LST_FILE_ACCESS_ID, h5p::H5P_LST_FILE_ACCESS);
link_hid!(H5P_LST_DATASET_CREATE_ID, h5p::H5P_LST_DATASET_CREATE);
link_hid!(H5P_LST_DATASET_ACCESS_ID, h5p::H5P_LST_DATASET_ACCESS);
link_hid!(H5P_LST_DATASET_XFER_ID, h5p::H5P_LST_DATASET_XFER);
link_hid!(H5P_LST_FILE_MOUNT_ID, h5p::H5P_LST_FILE_MOUNT);
link_hid!(H5P_LST_GROUP_CREATE_ID, h5p::H5P_LST_GROUP_CREATE);
link_hid!(H5P_LST_GROUP_ACCESS_ID, h5p::H5P_LST_GROUP_ACCESS);
link_hid!(H5P_LST_DATATYPE_CREATE_ID, h5p::H5P_LST_DATATYPE_CREATE);
link_hid!(H5P_LST_DATATYPE_ACCESS_ID, h5p::H5P_LST_DATATYPE_ACCESS);
link_hid!(H5P_LST_ATTRIBUTE_CREATE_ID, h5p::H5P_LST_ATTRIBUTE_CREATE);
link_hid!(H5P_LST_OBJECT_COPY_ID, h5p::H5P_LST_OBJECT_COPY);
link_hid!(H5P_LST_LINK_CREATE_ID, h5p::H5P_LST_LINK_CREATE);
link_hid!(H5P_LST_LINK_ACCESS_ID, h5p::H5P_LST_LINK_ACCESS);

// Error class
link_hid!(H5E_ERR_CLS, h5e::H5E_ERR_CLS);

// Errors
link_hid!(H5E_DATASET, h5e::H5E_DATASET);
link_hid!(H5E_FUNC, h5e::H5E_FUNC);
link_hid!(H5E_STORAGE, h5e::H5E_STORAGE);
link_hid!(H5E_FILE, h5e::H5E_FILE);
link_hid!(H5E_SOHM, h5e::H5E_SOHM);
link_hid!(H5E_SYM, h5e::H5E_SYM);
link_hid!(H5E_PLUGIN, h5e::H5E_PLUGIN);
link_hid!(H5E_VFL, h5e::H5E_VFL);
link_hid!(H5E_INTERNAL, h5e::H5E_INTERNAL);
link_hid!(H5E_BTREE, h5e::H5E_BTREE);
link_hid!(H5E_REFERENCE, h5e::H5E_REFERENCE);
link_hid!(H5E_DATASPACE, h5e::H5E_DATASPACE);
link_hid!(H5E_RESOURCE, h5e::H5E_RESOURCE);
link_hid!(H5E_PLIST, h5e::H5E_PLIST);
link_hid!(H5E_LINK, h5e::H5E_LINK);
link_hid!(H5E_DATATYPE, h5e::H5E_DATATYPE);
link_hid!(H5E_RS, h5e::H5E_RS);
link_hid!(H5E_HEAP, h5e::H5E_HEAP);
link_hid!(H5E_OHDR, h5e::H5E_OHDR);
#[cfg(not(feature = "1.14.0"))]
link_hid!(H5E_ATOM, h5e::H5E_ATOM);
link_hid!(H5E_ATTR, h5e::H5E_ATTR);
link_hid!(H5E_NONE_MAJOR, h5e::H5E_NONE_MAJOR);
link_hid!(H5E_IO, h5e::H5E_IO);
link_hid!(H5E_SLIST, h5e::H5E_SLIST);
link_hid!(H5E_EFL, h5e::H5E_EFL);
link_hid!(H5E_TST, h5e::H5E_TST);
link_hid!(H5E_ARGS, h5e::H5E_ARGS);
link_hid!(H5E_ERROR, h5e::H5E_ERROR);
link_hid!(H5E_PLINE, h5e::H5E_PLINE);
link_hid!(H5E_FSPACE, h5e::H5E_FSPACE);
link_hid!(H5E_CACHE, h5e::H5E_CACHE);
link_hid!(H5E_SEEKERROR, h5e::H5E_SEEKERROR);
link_hid!(H5E_READERROR, h5e::H5E_READERROR);
link_hid!(H5E_WRITEERROR, h5e::H5E_WRITEERROR);
link_hid!(H5E_CLOSEERROR, h5e::H5E_CLOSEERROR);
link_hid!(H5E_OVERFLOW, h5e::H5E_OVERFLOW);
link_hid!(H5E_FCNTL, h5e::H5E_FCNTL);
link_hid!(H5E_NOSPACE, h5e::H5E_NOSPACE);
link_hid!(H5E_CANTALLOC, h5e::H5E_CANTALLOC);
link_hid!(H5E_CANTCOPY, h5e::H5E_CANTCOPY);
link_hid!(H5E_CANTFREE, h5e::H5E_CANTFREE);
link_hid!(H5E_ALREADYEXISTS, h5e::H5E_ALREADYEXISTS);
link_hid!(H5E_CANTLOCK, h5e::H5E_CANTLOCK);
link_hid!(H5E_CANTUNLOCK, h5e::H5E_CANTUNLOCK);
link_hid!(H5E_CANTGC, h5e::H5E_CANTGC);
link_hid!(H5E_CANTGETSIZE, h5e::H5E_CANTGETSIZE);
link_hid!(H5E_OBJOPEN, h5e::H5E_OBJOPEN);
link_hid!(H5E_CANTRESTORE, h5e::H5E_CANTRESTORE);
link_hid!(H5E_CANTCOMPUTE, h5e::H5E_CANTCOMPUTE);
link_hid!(H5E_CANTEXTEND, h5e::H5E_CANTEXTEND);
link_hid!(H5E_CANTATTACH, h5e::H5E_CANTATTACH);
link_hid!(H5E_CANTUPDATE, h5e::H5E_CANTUPDATE);
link_hid!(H5E_CANTOPERATE, h5e::H5E_CANTOPERATE);
link_hid!(H5E_CANTINIT, h5e::H5E_CANTINIT);
link_hid!(H5E_ALREADYINIT, h5e::H5E_ALREADYINIT);
link_hid!(H5E_CANTRELEASE, h5e::H5E_CANTRELEASE);
link_hid!(H5E_CANTGET, h5e::H5E_CANTGET);
link_hid!(H5E_CANTSET, h5e::H5E_CANTSET);
link_hid!(H5E_DUPCLASS, h5e::H5E_DUPCLASS);
link_hid!(H5E_SETDISALLOWED, h5e::H5E_SETDISALLOWED);
link_hid!(H5E_CANTMERGE, h5e::H5E_CANTMERGE);
link_hid!(H5E_CANTREVIVE, h5e::H5E_CANTREVIVE);
link_hid!(H5E_CANTSHRINK, h5e::H5E_CANTSHRINK);
link_hid!(H5E_LINKCOUNT, h5e::H5E_LINKCOUNT);
link_hid!(H5E_VERSION, h5e::H5E_VERSION);
link_hid!(H5E_ALIGNMENT, h5e::H5E_ALIGNMENT);
link_hid!(H5E_BADMESG, h5e::H5E_BADMESG);
link_hid!(H5E_CANTDELETE, h5e::H5E_CANTDELETE);
link_hid!(H5E_BADITER, h5e::H5E_BADITER);
link_hid!(H5E_CANTPACK, h5e::H5E_CANTPACK);
link_hid!(H5E_CANTRESET, h5e::H5E_CANTRESET);
link_hid!(H5E_CANTRENAME, h5e::H5E_CANTRENAME);
link_hid!(H5E_SYSERRSTR, h5e::H5E_SYSERRSTR);
link_hid!(H5E_NOFILTER, h5e::H5E_NOFILTER);
link_hid!(H5E_CALLBACK, h5e::H5E_CALLBACK);
link_hid!(H5E_CANAPPLY, h5e::H5E_CANAPPLY);
link_hid!(H5E_SETLOCAL, h5e::H5E_SETLOCAL);
link_hid!(H5E_NOENCODER, h5e::H5E_NOENCODER);
link_hid!(H5E_CANTFILTER, h5e::H5E_CANTFILTER);
link_hid!(H5E_CANTOPENOBJ, h5e::H5E_CANTOPENOBJ);
link_hid!(H5E_CANTCLOSEOBJ, h5e::H5E_CANTCLOSEOBJ);
link_hid!(H5E_COMPLEN, h5e::H5E_COMPLEN);
link_hid!(H5E_PATH, h5e::H5E_PATH);
link_hid!(H5E_NONE_MINOR, h5e::H5E_NONE_MINOR);
link_hid!(H5E_OPENERROR, h5e::H5E_OPENERROR);
link_hid!(H5E_FILEEXISTS, h5e::H5E_FILEEXISTS);
link_hid!(H5E_FILEOPEN, h5e::H5E_FILEOPEN);
link_hid!(H5E_CANTCREATE, h5e::H5E_CANTCREATE);
link_hid!(H5E_CANTOPENFILE, h5e::H5E_CANTOPENFILE);
link_hid!(H5E_CANTCLOSEFILE, h5e::H5E_CANTCLOSEFILE);
link_hid!(H5E_NOTHDF5, h5e::H5E_NOTHDF5);
link_hid!(H5E_BADFILE, h5e::H5E_BADFILE);
link_hid!(H5E_TRUNCATED, h5e::H5E_TRUNCATED);
link_hid!(H5E_MOUNT, h5e::H5E_MOUNT);
#[cfg(not(feature = "1.14.0"))]
link_hid!(H5E_BADATOM, h5e::H5E_BADATOM);
link_hid!(H5E_BADGROUP, h5e::H5E_BADGROUP);
link_hid!(H5E_CANTREGISTER, h5e::H5E_CANTREGISTER);
link_hid!(H5E_CANTINC, h5e::H5E_CANTINC);
link_hid!(H5E_CANTDEC, h5e::H5E_CANTDEC);
link_hid!(H5E_NOIDS, h5e::H5E_NOIDS);
link_hid!(H5E_CANTFLUSH, h5e::H5E_CANTFLUSH);
link_hid!(H5E_CANTSERIALIZE, h5e::H5E_CANTSERIALIZE);
link_hid!(H5E_CANTLOAD, h5e::H5E_CANTLOAD);
link_hid!(H5E_PROTECT, h5e::H5E_PROTECT);
link_hid!(H5E_NOTCACHED, h5e::H5E_NOTCACHED);
link_hid!(H5E_SYSTEM, h5e::H5E_SYSTEM);
link_hid!(H5E_CANTINS, h5e::H5E_CANTINS);
link_hid!(H5E_CANTPROTECT, h5e::H5E_CANTPROTECT);
link_hid!(H5E_CANTUNPROTECT, h5e::H5E_CANTUNPROTECT);
link_hid!(H5E_CANTPIN, h5e::H5E_CANTPIN);
link_hid!(H5E_CANTUNPIN, h5e::H5E_CANTUNPIN);
link_hid!(H5E_CANTMARKDIRTY, h5e::H5E_CANTMARKDIRTY);
link_hid!(H5E_CANTDIRTY, h5e::H5E_CANTDIRTY);
link_hid!(H5E_CANTEXPUNGE, h5e::H5E_CANTEXPUNGE);
link_hid!(H5E_CANTRESIZE, h5e::H5E_CANTRESIZE);
link_hid!(H5E_TRAVERSE, h5e::H5E_TRAVERSE);
link_hid!(H5E_NLINKS, h5e::H5E_NLINKS);
link_hid!(H5E_NOTREGISTERED, h5e::H5E_NOTREGISTERED);
link_hid!(H5E_CANTMOVE, h5e::H5E_CANTMOVE);
link_hid!(H5E_CANTSORT, h5e::H5E_CANTSORT);
link_hid!(H5E_MPI, h5e::H5E_MPI);
link_hid!(H5E_MPIERRSTR, h5e::H5E_MPIERRSTR);
link_hid!(H5E_CANTRECV, h5e::H5E_CANTRECV);
link_hid!(H5E_CANTCLIP, h5e::H5E_CANTCLIP);
link_hid!(H5E_CANTCOUNT, h5e::H5E_CANTCOUNT);
link_hid!(H5E_CANTSELECT, h5e::H5E_CANTSELECT);
link_hid!(H5E_CANTNEXT, h5e::H5E_CANTNEXT);
link_hid!(H5E_BADSELECT, h5e::H5E_BADSELECT);
link_hid!(H5E_CANTCOMPARE, h5e::H5E_CANTCOMPARE);
link_hid!(H5E_UNINITIALIZED, h5e::H5E_UNINITIALIZED);
link_hid!(H5E_UNSUPPORTED, h5e::H5E_UNSUPPORTED);
link_hid!(H5E_BADTYPE, h5e::H5E_BADTYPE);
link_hid!(H5E_BADRANGE, h5e::H5E_BADRANGE);
link_hid!(H5E_BADVALUE, h5e::H5E_BADVALUE);
link_hid!(H5E_NOTFOUND, h5e::H5E_NOTFOUND);
link_hid!(H5E_EXISTS, h5e::H5E_EXISTS);
link_hid!(H5E_CANTENCODE, h5e::H5E_CANTENCODE);
link_hid!(H5E_CANTDECODE, h5e::H5E_CANTDECODE);
link_hid!(H5E_CANTSPLIT, h5e::H5E_CANTSPLIT);
link_hid!(H5E_CANTREDISTRIBUTE, h5e::H5E_CANTREDISTRIBUTE);
link_hid!(H5E_CANTSWAP, h5e::H5E_CANTSWAP);
link_hid!(H5E_CANTINSERT, h5e::H5E_CANTINSERT);
link_hid!(H5E_CANTLIST, h5e::H5E_CANTLIST);
link_hid!(H5E_CANTMODIFY, h5e::H5E_CANTMODIFY);
link_hid!(H5E_CANTREMOVE, h5e::H5E_CANTREMOVE);
link_hid!(H5E_CANTCONVERT, h5e::H5E_CANTCONVERT);
link_hid!(H5E_BADSIZE, h5e::H5E_BADSIZE);

// H5R constants
lazy_static! {
    pub static ref H5R_OBJ_REF_BUF_SIZE: usize = mem::size_of::<haddr_t>();
    pub static ref H5R_DSET_REG_REF_BUF_SIZE: usize = mem::size_of::<haddr_t>() + 4;
}

// File drivers
lazy_static! {
    pub static ref H5FD_CORE: hid_t = h5lock!(H5FD_core_init());
    pub static ref H5FD_SEC2: hid_t = h5lock!(H5FD_sec2_init());
    pub static ref H5FD_STDIO: hid_t = h5lock!(H5FD_stdio_init());
    pub static ref H5FD_FAMILY: hid_t = h5lock!(H5FD_family_init());
    pub static ref H5FD_LOG: hid_t = h5lock!(H5FD_log_init());
    pub static ref H5FD_MULTI: hid_t = h5lock!(H5FD_multi_init());
}

// MPI-IO file driver
#[cfg(feature = "have-parallel")]
lazy_static! {
    pub static ref H5FD_MPIO: hid_t = h5lock!(H5FD_mpio_init());
}
#[cfg(not(feature = "have-parallel"))]
lazy_static! {
    pub static ref H5FD_MPIO: hid_t = H5I_INVALID_HID;
}

// Direct VFD
#[cfg(feature = "have-direct")]
lazy_static! {
    pub static ref H5FD_DIRECT: hid_t = h5lock!(H5FD_direct_init());
}
#[cfg(not(feature = "have-direct"))]
lazy_static! {
    pub static ref H5FD_DIRECT: hid_t = H5I_INVALID_HID;
}

#[cfg(target_os = "windows")]
lazy_static! {
    pub static ref H5FD_WINDOWS: hid_t = *H5FD_SEC2;
}

#[cfg(test)]
mod tests {
    use std::mem;

    use hdf5_sys::{h5::haddr_t, h5i::H5I_INVALID_HID};

    use super::{
        H5E_DATASET, H5E_ERR_CLS, H5P_LST_LINK_ACCESS_ID, H5P_ROOT, H5R_DSET_REG_REF_BUF_SIZE,
        H5R_OBJ_REF_BUF_SIZE, H5T_IEEE_F32BE, H5T_NATIVE_INT,
    };

    #[test]
    pub fn test_lazy_globals() {
        assert_ne!(*H5T_IEEE_F32BE, H5I_INVALID_HID);
        assert_ne!(*H5T_NATIVE_INT, H5I_INVALID_HID);

        assert_ne!(*H5P_ROOT, H5I_INVALID_HID);
        assert_ne!(*H5P_LST_LINK_ACCESS_ID, H5I_INVALID_HID);

        assert_ne!(*H5E_ERR_CLS, H5I_INVALID_HID);
        assert_ne!(*H5E_DATASET, H5I_INVALID_HID);

        assert_eq!(*H5R_OBJ_REF_BUF_SIZE, mem::size_of::<haddr_t>());
        assert_eq!(*H5R_DSET_REG_REF_BUF_SIZE, mem::size_of::<haddr_t>() + 4);
    }
}
