use crate::internal_prelude::*;

use libc;

use std::borrow::Borrow;
use std::mem;
use std::ptr;
use num::{Integer, NumCast};
use num::traits::cast;
use std::ffi::{CStr, CString};

/// Convert a zero-terminated string (`const char *`) into a `String`.
pub fn string_from_cstr(string: *const c_char) -> String {
    unsafe {
        String::from_utf8_unchecked(CStr::from_ptr(string).to_bytes().to_vec())
    }
}

/// Convert a `String` or a `&str` into a zero-terminated string (`const char *`).
pub fn to_cstring<S: Borrow<str>>(string: S) -> Result<CString> {
    let string = string.borrow();
    CString::new(string).map_err(|_| format!("null byte in string: {:?}", string).into())
}

#[doc(hidden)]
pub fn get_h5_str<T, F>(func: F) -> Result<String>
where F: Fn(*mut c_char, size_t) -> T, T: Integer + NumCast {
    unsafe {
        let len: isize = 1 + cast::<T, isize>(func(ptr::null_mut(), 0)).unwrap();
        ensure!(len > 0, "negative string length in get_h5_str()");
        if len == 1 {
            return Ok("".to_owned());
        }
        let buf: *mut c_char = libc::malloc(((len as usize) * mem::size_of::<c_char>()) as _) as _;
        func(buf, len as _);
        let msg = string_from_cstr(buf);
        libc::free(buf as *mut _);
        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use crate::ffi::h5e::H5Eget_msg;
    use crate::globals::H5E_CANTOPENOBJ;
    use super::{string_from_cstr, to_cstring, get_h5_str};

    use std::ptr;

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
            get_h5_str(|msg, size| {
                H5Eget_msg(*H5E_CANTOPENOBJ, ptr::null_mut(), msg, size)
            }).ok().unwrap()
        });
        assert_eq!(s, "Can't open object");
    }
}
