use std::borrow::Borrow;
use std::ffi::{CStr, CString};
use std::ptr;
use std::str;

use num_integer::Integer;
use num_traits::{cast, NumCast};

use crate::internal_prelude::*;

/// Convert a zero-terminated string (`const char *`) into a `String`.
pub fn string_from_cstr(string: *const c_char) -> String {
    unsafe { String::from_utf8_unchecked(CStr::from_ptr(string).to_bytes().to_vec()) }
}

/// Convert a `String` or a `&str` into a zero-terminated string (`const char *`).
pub fn to_cstring<S: Borrow<str>>(string: S) -> Result<CString> {
    let string = string.borrow();
    CString::new(string).map_err(|_| format!("null byte in string: {:?}", string).into())
}

/// Convert a fixed-length (possibly zero-terminated) char buffer to a string.
pub fn string_from_fixed_bytes(bytes: &[c_char], len: usize) -> String {
    let len = bytes.iter().position(|&c| c == 0).unwrap_or(len);
    let s = unsafe { str::from_utf8_unchecked(&*(&bytes[..len] as *const _ as *const _)) };
    s.to_owned()
}

/// Write a string into a fixed-length char buffer (possibly truncating it).
pub fn string_to_fixed_bytes(s: &str, buf: &mut [c_char]) {
    let mut s = s;
    while s.as_bytes().len() > buf.len() {
        s = &s[..(s.len() - 1)];
    }
    let bytes = s.as_bytes();
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), buf.as_mut_ptr() as *mut _, bytes.len());
    }
    for c in &mut buf[bytes.len()..] {
        *c = 0;
    }
}

#[cfg(hdf5_1_8_13)]
pub fn h5_free_memory(mem: *mut c_void) {
    use hdf5_sys::h5::H5free_memory;
    unsafe { H5free_memory(mem) };
}

#[cfg(not(hdf5_1_8_13))]
pub fn h5_free_memory(mem: *mut c_void) {
    // this may fail in debug builds of HDF5
    use libc::free;
    unsafe { free(mem) };
}

#[doc(hidden)]
pub fn get_h5_str<T, F>(func: F) -> Result<String>
where
    F: Fn(*mut c_char, size_t) -> T,
    T: Integer + NumCast,
{
    let len = 1 + cast::<T, isize>(func(ptr::null_mut(), 0)).unwrap();
    ensure!(len > 0, "negative string length in get_h5_str()");
    if len == 1 {
        Ok("".to_owned())
    } else {
        let mut buf = vec![0; len as usize];
        func(buf.as_mut_ptr(), len as _);
        Ok(string_from_cstr(buf.as_ptr()))
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use hdf5_sys::h5e::H5Eget_msg;

    use crate::globals::H5E_CANTOPENOBJ;

    use super::{get_h5_str, string_from_cstr, to_cstring};

    #[test]
    pub fn test_string_cstr() {
        let s1 = "foo".to_owned();
        let c_s1 = to_cstring(s1.clone()).unwrap();
        assert_eq!(s1, string_from_cstr(c_s1.as_ptr()));
        let s2 = "bar";
        let c_s2 = to_cstring(s2).unwrap();
        assert_eq!(s2, string_from_cstr(c_s2.as_ptr()));
    }

    #[test]
    pub fn test_get_h5_str() {
        let s = h5lock!({
            get_h5_str(|msg, size| H5Eget_msg(*H5E_CANTOPENOBJ, ptr::null_mut(), msg, size))
                .ok()
                .unwrap()
        });
        assert_eq!(s, "Can't open object");
    }
}
