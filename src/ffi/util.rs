use libc::{self, c_char, c_void, size_t};

use std::mem::size_of;
use std::ptr::null_mut;
use num::{Integer, NumCast};
use num::traits::cast;
use std::ffi::{CStr, CString};

use error::Result;

pub fn string_from_cstr(string: *const c_char) -> String {
    unsafe {
        String::from_utf8_unchecked(CStr::from_ptr(string).to_bytes().to_vec())
    }
}

pub fn string_to_cstr<S>(string: S) -> *const c_char where S: Into<String> {
    unsafe {
        CString::from_vec_unchecked(string.into().into_bytes()).as_ptr()
    }
}

#[test]
pub fn test_string_cstr() {
    let s1: String = "foo".to_string();
    assert_eq!(s1, string_from_cstr(string_to_cstr(s1.clone())));
    let s2: &str = "bar";
    assert_eq!(s2, string_from_cstr(string_to_cstr(s2)));
}

pub fn get_h5_str<T, F>(func: F) -> Result<String>
                 where F: Fn(*mut c_char, size_t) -> T, T: Integer + NumCast {
    unsafe {
        let len: isize = 1 + cast::<T, isize>(func(null_mut(), 0)).unwrap();
        ensure!(len > 0, "negative string length in get_h5_str()");
        let buf = libc::malloc((len as size_t) * size_of::<c_char>() as size_t) as *mut c_char;
        func(buf, len as size_t);
        let msg = string_from_cstr(buf);
        libc::free(buf as *mut c_void);
        Ok(msg)
    }
}

#[test]
pub fn test_get_h5_str() {
    use ffi::h5e::{H5Eget_msg, H5E_CANTOPENOBJ};
    let s = unsafe {
        get_h5_str(|msg, size| {
            H5Eget_msg(*H5E_CANTOPENOBJ, null_mut(), msg, size)
        }).ok().unwrap()
    };
    assert_eq!(s, "Can't open object");
}
