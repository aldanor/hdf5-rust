use std::ptr;
use std::slice;

use lazy_static::lazy_static;

use hdf5_sys::h5p::{H5Pget_chunk, H5Pget_filter_by_id2, H5Pmodify_filter};
use hdf5_sys::h5t::H5Tget_size;
use hdf5_sys::h5z::{H5Z_class2_t, H5Z_filter_t, H5Zregister, H5Z_CLASS_T_VERS, H5Z_FLAG_REVERSE};

use crate::globals::{H5E_CALLBACK, H5E_CANTREGISTER, H5E_PLIST};
use crate::internal_prelude::*;

extern "C" {
    fn lzf_compress(
        in_data: *const c_void, in_len: c_uint, out_data: *mut c_void, out_len: c_uint,
    ) -> c_uint;
    fn lzf_decompress(
        in_data: *const c_void, in_len: c_uint, out_data: *mut c_void, out_len: c_uint,
    ) -> c_uint;
}

const LZF_VERSION: c_uint = 0x0105;
const LZF_FILTER_NAME: &[u8] = b"lzf\0";
pub(crate) const LZF_FILTER_ID: H5Z_filter_t = 32000;
const LZF_FILTER_VERSION: c_uint = 4;

const LZF_FILTER_INFO: H5Z_class2_t = H5Z_class2_t {
    version: H5Z_CLASS_T_VERS as _,
    id: LZF_FILTER_ID,
    encoder_present: 1,
    decoder_present: 1,
    name: LZF_FILTER_NAME.as_ptr() as *const _,
    can_apply: None,
    set_local: Some(set_local_lzf),
    filter: Some(filter_lzf),
};

lazy_static! {
    static ref LZF_INIT: Result<()> = {
        h5try!({
            let ret = H5Zregister(&LZF_FILTER_INFO as *const _ as *const _);
            h5maybe_err!(ret, "Can't register LZF filter", H5E_PLIST, H5E_CANTREGISTER)
        });
        Ok(())
    };
}

pub(crate) fn register_lzf() -> Result<()> {
    (*LZF_INIT).clone()
}

extern "C" fn set_local_lzf(dcpl_id: hid_t, type_id: hid_t, _space_id: hid_t) -> herr_t {
    let mut flags: c_uint = 0;
    let mut nelmts: size_t = 0;
    let mut values = vec![0 as c_uint; 8];
    let ret = unsafe {
        H5Pget_filter_by_id2(
            dcpl_id,
            LZF_FILTER_ID,
            &mut flags as *mut _,
            &mut nelmts as *mut _,
            values.as_mut_ptr(),
            0,
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    if ret < 0 {
        return -1;
    }
    nelmts = nelmts.max(3);
    if values[0] == 0 {
        values[0] = LZF_FILTER_VERSION;
    }
    if values[1] == 0 {
        values[1] = LZF_VERSION;
    }
    const MAX_NDIMS: usize = 32;
    let mut chunkdims = vec![0 as hsize_t; MAX_NDIMS];
    let ndims: c_int = unsafe { H5Pget_chunk(dcpl_id, MAX_NDIMS as _, chunkdims.as_mut_ptr()) };
    if ndims < 0 {
        return -1;
    }
    if ndims > MAX_NDIMS as _ {
        h5err!("Chunk rank exceeds limit", H5E_PLIST, H5E_CALLBACK);
        return -1;
    }
    let mut bufsize: size_t = unsafe { H5Tget_size(type_id) };
    if bufsize == 0 {
        return -1;
    }
    for i in 0..(ndims as usize) {
        bufsize *= chunkdims[i] as size_t;
    }
    values[2] = bufsize as _;
    let r = unsafe { H5Pmodify_filter(dcpl_id, LZF_FILTER_ID, flags, nelmts, values.as_ptr()) };
    if r < 0 {
        -1
    } else {
        1
    }
}

extern "C" fn filter_lzf(
    flags: c_uint, cd_nelmts: size_t, cd_values: *const c_uint, nbytes: size_t,
    buf_size: *mut size_t, buf: *mut *mut c_void,
) -> size_t {
    if flags & H5Z_FLAG_REVERSE == 0 {
        unsafe { filter_lzf_compress(nbytes, buf_size, buf) }
    } else {
        unsafe { filter_lzf_decompress(cd_nelmts, cd_values, nbytes, buf_size, buf) }
    }
}

unsafe fn filter_lzf_compress(
    nbytes: size_t, buf_size: *mut size_t, buf: *mut *mut c_void,
) -> size_t {
    let outbuf_size = *buf_size;
    let outbuf = libc::malloc(outbuf_size);
    if outbuf.is_null() {
        h5err!("Can't allocate compression buffer", H5E_PLIST, H5E_CALLBACK);
        return 0;
    }
    let status = lzf_compress(*buf, nbytes as _, outbuf, outbuf_size as _);
    if status != 0 {
        libc::free(*buf);
        *buf = outbuf;
    } else {
        libc::free(outbuf);
    }
    status as _
}

unsafe fn filter_lzf_decompress(
    cd_nelmts: size_t, cd_values: *const c_uint, nbytes: size_t, buf_size: *mut size_t,
    buf: *mut *mut c_void,
) -> size_t {
    let cdata = slice::from_raw_parts(cd_values, cd_nelmts as _);
    let mut outbuf_size = if cd_nelmts >= 3 && cdata[2] != 0 { cdata[2] as _ } else { *buf_size };
    let mut outbuf: *mut c_void;
    let mut status: c_uint;
    loop {
        outbuf = libc::malloc(outbuf_size);
        if outbuf.is_null() {
            h5err!("Can't allocate decompression buffer", H5E_PLIST, H5E_CALLBACK);
            return 0;
        }
        status = lzf_decompress(*buf, nbytes as _, outbuf, outbuf_size as _);
        if status != 0 {
            break;
        }
        libc::free(outbuf);
        let e = errno::errno().0;
        if e == 7 {
            outbuf_size += *buf_size;
            continue;
        } else if e == 22 {
            h5err!("Invalid data for LZF decompression", H5E_PLIST, H5E_CALLBACK);
        } else {
            h5err!("Unknown LZF decompression error", H5E_PLIST, H5E_CALLBACK);
        }
        return 0;
    }
    libc::free(*buf);
    *buf = outbuf;
    *buf_size = outbuf_size as _;
    status as _
}
