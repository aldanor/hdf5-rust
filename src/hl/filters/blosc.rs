use std::ptr;
use std::slice;

use lazy_static::lazy_static;

use hdf5_sys::h5p::{H5Pget_chunk, H5Pget_filter_by_id2, H5Pmodify_filter};
use hdf5_sys::h5t::{H5Tclose, H5Tget_class, H5Tget_size, H5Tget_super, H5T_ARRAY};
use hdf5_sys::h5z::{H5Z_class2_t, H5Z_filter_t, H5Zregister, H5Z_CLASS_T_VERS, H5Z_FLAG_REVERSE};

use crate::globals::{H5E_CALLBACK, H5E_CANTREGISTER, H5E_PLIST};
use crate::internal_prelude::*;

pub const BLOSC_VERSION_FORMAT: c_uint = 2;

pub const BLOSC_MAX_TYPESIZE: c_uint = 255;

pub const BLOSC_NOSHUFFLE: c_uint = 0;
pub const BLOSC_SHUFFLE: c_uint = 1;
pub const BLOSC_BITSHUFFLE: c_uint = 2;

pub const BLOSC_BLOSCLZ: c_uint = 0;
pub const BLOSC_LZ4: c_uint = 1;
pub const BLOSC_LZ4HC: c_uint = 2;
pub const BLOSC_SNAPPY: c_uint = 3;
pub const BLOSC_ZLIB: c_uint = 4;
pub const BLOSC_ZSTD: c_uint = 5;

extern "C" {
    pub fn blosc_init();
    pub fn blosc_get_version_string() -> *const c_char;
    pub fn blosc_get_nthreads() -> c_int;
    pub fn blosc_set_nthreads(nthreads: c_int) -> c_int;
    pub fn blosc_compress(
        clevel: c_int, doshuffle: c_int, typesize: size_t, nbytes: size_t, src: *const c_void,
        dest: *mut c_void, destsize: size_t,
    ) -> c_int;
    pub fn blosc_decompress(src: *const c_void, dest: *mut c_void, destsize: size_t) -> c_int;
    pub fn blosc_set_compressor(compname: *const c_char) -> c_int;
    pub fn blosc_cbuffer_sizes(
        cbuffer: *const c_void, nbytes: *mut size_t, cbytes: *mut size_t, blocksize: *mut size_t,
    );
    pub fn blosc_list_compressors() -> *const c_char;
    pub fn blosc_compcode_to_compname(compcode: c_int, compname: *mut *const c_char) -> c_int;
}

const BLOSC_FILTER_NAME: &[u8] = b"blosc\0";
pub(crate) const BLOSC_FILTER_ID: H5Z_filter_t = 32001;
const BLOSC_FILTER_VERSION: c_uint = 2;

const BLOSC_FILTER_INFO: H5Z_class2_t = H5Z_class2_t {
    version: H5Z_CLASS_T_VERS as _,
    id: BLOSC_FILTER_ID,
    encoder_present: 1,
    decoder_present: 1,
    name: BLOSC_FILTER_NAME.as_ptr() as *const _,
    can_apply: None,
    set_local: Some(set_local_blosc),
    filter: Some(filter_blosc),
};

lazy_static! {
    static ref BLOSC_INIT: Result<()> = {
        h5try!({
            blosc_init();
            let ret = H5Zregister(&BLOSC_FILTER_INFO as *const _ as *const _);
            h5maybe_err!(ret, "Can't register Blosc filter", H5E_PLIST, H5E_CANTREGISTER)
        });
        Ok(())
    };
}

pub(crate) fn register_blosc() -> Result<()> {
    (*BLOSC_INIT).clone()
}

extern "C" fn set_local_blosc(dcpl_id: hid_t, type_id: hid_t, _space_id: hid_t) -> herr_t {
    let mut flags: c_uint = 0;
    let mut nelmts: size_t = 0;
    let mut values = vec![0 as c_uint; 8];
    let ret = unsafe {
        H5Pget_filter_by_id2(
            dcpl_id,
            BLOSC_FILTER_ID,
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
    nelmts = nelmts.max(4);
    values[0] = BLOSC_FILTER_VERSION;
    values[1] = BLOSC_VERSION_FORMAT;
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
    let typesize: size_t = unsafe { H5Tget_size(type_id) };
    if typesize == 0 {
        return -1;
    }
    let mut basetypesize = typesize;
    unsafe {
        if H5Tget_class(type_id) == H5T_ARRAY {
            let super_type = H5Tget_super(type_id);
            basetypesize = H5Tget_size(super_type);
            H5Tclose(super_type);
        }
    }
    if basetypesize > BLOSC_MAX_TYPESIZE as _ {
        basetypesize = 1;
    }
    values[2] = basetypesize as _;
    let mut bufsize = typesize;
    for i in 0..(ndims as usize) {
        bufsize *= chunkdims[i] as size_t;
    }
    values[3] = bufsize as _;
    let r = unsafe { H5Pmodify_filter(dcpl_id, BLOSC_FILTER_ID, flags, nelmts, values.as_ptr()) };
    if r < 0 {
        -1
    } else {
        1
    }
}

struct BloscConfig {
    pub typesize: size_t,
    pub outbuf_size: size_t,
    pub clevel: c_int,
    pub doshuffle: c_int,
    pub compname: *const c_char,
}

impl Default for BloscConfig {
    fn default() -> Self {
        const DEFAULT_COMPNAME: &[u8] = b"blosclz\0";
        Self {
            typesize: 0,
            outbuf_size: 0,
            clevel: 5,
            doshuffle: 1,
            compname: DEFAULT_COMPNAME.as_ptr() as *const _,
        }
    }
}

fn parse_blosc_cdata(cd_nelmts: size_t, cd_values: *const c_uint) -> Option<BloscConfig> {
    let cdata = unsafe { slice::from_raw_parts(cd_values, cd_nelmts as _) };
    let mut cfg = BloscConfig::default();
    cfg.typesize = cdata[2] as _;
    cfg.outbuf_size = cdata[3] as _;
    if cdata.len() >= 5 {
        cfg.clevel = cdata[4] as _;
    };
    if cdata.len() >= 6 {
        let v = unsafe { slice::from_raw_parts(blosc_get_version_string() as *mut u8, 4) };
        if v[0] <= b'1' && v[1] == b'.' && v[2] < b'8' && v[3] == b'.' {
            h5err!(
                "This Blosc library version is not supported. Please update to >= 1.8",
                H5E_PLIST,
                H5E_CALLBACK
            );
            return None;
        }
        cfg.doshuffle = cdata[5] as _;
    }
    if cdata.len() >= 7 {
        let r = unsafe { blosc_compcode_to_compname(cdata[6] as _, &mut cfg.compname as *mut _) };
        if r == -1 {
            let complist = string_from_cstr(unsafe { blosc_list_compressors() });
            let errmsg = format!(
                concat!(
                    "This Blosc library does not have support for the '{}' compressor, ",
                    "but only for: {}"
                ),
                string_from_cstr(cfg.compname),
                complist
            );
            h5err!(errmsg, H5E_PLIST, H5E_CALLBACK);
            return None;
        }
    }
    Some(cfg)
}

extern "C" fn filter_blosc(
    flags: c_uint, cd_nelmts: size_t, cd_values: *const c_uint, nbytes: size_t,
    buf_size: *mut size_t, buf: *mut *mut c_void,
) -> size_t {
    let cfg = if let Some(cfg) = parse_blosc_cdata(cd_nelmts, cd_values) {
        cfg
    } else {
        return 0;
    };
    if flags & H5Z_FLAG_REVERSE == 0 {
        unsafe { filter_blosc_compress(&cfg, nbytes, buf_size, buf) }
    } else {
        unsafe { filter_blosc_decompress(&cfg, buf_size, buf) }
    }
}

unsafe fn filter_blosc_compress(
    cfg: &BloscConfig, nbytes: size_t, buf_size: *mut size_t, buf: *mut *mut c_void,
) -> size_t {
    let outbuf_size = *buf_size;
    let outbuf = libc::malloc(outbuf_size);
    if outbuf.is_null() {
        h5err!("Can't allocate compression buffer", H5E_PLIST, H5E_CALLBACK);
        return 0;
    }
    blosc_set_compressor(cfg.compname);
    let status =
        blosc_compress(cfg.clevel, cfg.doshuffle, cfg.typesize, nbytes, *buf, outbuf, nbytes);
    if status >= 0 {
        libc::free(*buf);
        *buf = outbuf;
        status as _
    } else {
        libc::free(outbuf);
        0
    }
}

unsafe fn filter_blosc_decompress(
    cfg: &BloscConfig, buf_size: *mut size_t, buf: *mut *mut c_void,
) -> size_t {
    let mut outbuf_size: size_t = cfg.outbuf_size;
    let (mut cbytes, mut blocksize): (size_t, size_t) = (0, 0);
    blosc_cbuffer_sizes(
        *buf,
        &mut outbuf_size as *mut _,
        &mut cbytes as *mut _,
        &mut blocksize as *mut _,
    );
    let outbuf = libc::malloc(outbuf_size);
    if outbuf.is_null() {
        h5err!("Can't allocate decompression buffer", H5E_PLIST, H5E_CALLBACK);
        return 0;
    }
    let status = blosc_decompress(*buf, outbuf, outbuf_size);
    if status > 0 {
        libc::free(*buf);
        *buf = outbuf;
        *buf_size = outbuf_size as _;
        status as _
    } else {
        libc::free(outbuf);
        h5err!("Blosc decompression error", H5E_PLIST, H5E_CALLBACK);
        0
    }
}
