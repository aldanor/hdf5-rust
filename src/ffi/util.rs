use libc::{self, c_char, c_void, size_t};

use std::mem;
use std::ptr;

use std::num::{Int, NumCast};
use std::ffi::c_str_to_bytes;
use std::str::from_utf8_unchecked;

pub fn str_from_c(string: *const c_char) -> String {
    unsafe {
        from_utf8_unchecked(c_str_to_bytes(&string)).to_string()
    }
}

pub fn get_h5_str<T, F>(func: F) -> String
                 where F: Fn(*mut c_char, size_t) -> T, T: Int + NumCast {
    unsafe {
        let len: size_t = 1 + NumCast::from(func(ptr::null_mut::<c_char>(), 0)).unwrap();
        let buf = libc::malloc(len * mem::size_of::<c_char>() as size_t) as *mut c_char;
        func(buf, len);
        let msg = str_from_c(buf);
        libc::free(buf as *mut c_void);
        msg
    }
}

#[test]
pub fn test_get_h5_str() {
    use ffi::h5e::{H5E_type_t, H5Eget_msg, H5E_CANTOPENOBJ};
    let s = get_h5_str(unsafe { |msg, size| {
        H5Eget_msg(*H5E_CANTOPENOBJ, ptr::null_mut::<H5E_type_t>(), msg, size)
    }});
    assert_eq!(s.as_slice(), "Can't open object");
}
