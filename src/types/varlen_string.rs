use std::ffi::CStr;
use std::ptr;
use std::str;

use libc::{c_char, c_void};

use types::{ValueType, ToValueType, Array};

#[repr(C)]
#[unsafe_no_drop_flag]
pub struct VarLenString {
    ptr: *mut u8,
}

impl Drop for VarLenString {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                ::libc::free(self.ptr as *mut c_void);
            }
        }
    }
}

unsafe impl ToValueType for VarLenString {
    fn value_type() -> ValueType {
        ValueType::VarLenString
    }
}

impl VarLenString {
    pub fn new() -> VarLenString {
        unsafe {
            let p = ::libc::malloc(1) as *mut u8;
            *p = 0;
            VarLenString { ptr: p }
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { CStr::from_ptr(self.ptr as *const c_char).to_bytes() }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.as_bytes().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn from_str(s: &str) -> VarLenString {
        unsafe {
            let p = ::libc::malloc(1 + s.len()) as *mut u8;
            ptr::copy_nonoverlapping(s.as_ptr(), p, s.len());
            *(p.offset(s.len() as isize)) = 0;
            VarLenString { ptr: p }
        }
    }
}

impl Clone for VarLenString {
    fn clone(&self) -> VarLenString {
        VarLenString::from_str(&*self)
    }
}

impl_string_traits!(VarLenString, VarLenString);

#[cfg(test)]
pub mod tests {
    use std::borrow::Borrow;
    use std::hash::{Hash, Hasher, SipHasher};
    use std::mem;

    use super::VarLenString;
    use types::ToValueType;
    use types::ValueType as VT;

    #[test]
    pub fn test_fixed_string() {
        type S = VarLenString;
        assert_eq!(S::value_type(), VT::VarLenString);
        assert_eq!(S::value_type().size(), mem::size_of::<*mut u8>());
        assert_eq!(mem::size_of::<S>(), S::value_type().size());

        assert!(S::from_str("").is_empty());
        assert!(S::from_str("\0").is_empty());
        assert!(S::new().is_empty());
        assert!(S::default().is_empty());

        assert_eq!(S::from("abc").as_str(), "abc");
        assert_eq!(S::from("abc".to_owned()).as_str(), "abc");
        let v: Vec<u8> = S::from("abc").into();
        assert_eq!(v, "abc".as_bytes().to_vec());

        let s = VarLenString::from_str("abc");
        assert_eq!(s.len(), 3);
        assert!(!s.is_empty());
        assert_eq!(s.as_bytes(), "abc".as_bytes());
        assert_eq!(s.as_str(), "abc");
        assert_eq!(&*s, "abc");
        assert_eq!(s.borrow() as &str, "abc");
        assert_eq!(s.as_ref() as &str, "abc");
        assert_eq!(&s[..], "abc");
        assert_eq!(s, "abc");
        assert_eq!("abc", s);
        assert_eq!(&s, "abc");
        assert_eq!("abc", &s);
        assert_eq!(s, "abc".to_owned());
        assert_eq!("abc".to_owned(), s);
        assert_eq!(s, s);
        assert_eq!(format!("{}", s), "abc");
        assert_eq!(format!("{:?}", s), "\"abc\"");

        let (mut h1, mut h2) = (SipHasher::new(), SipHasher::new());
        s.hash(&mut h1);
        "abc".hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }
}
