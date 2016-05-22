use std::ffi::CStr;
use std::ptr;
use std::str;

use libc::{c_char, c_void};

use types::{ValueType, ToValueType};

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
    pub fn new(s: &str) -> VarLenString {
        unsafe {
            let p = ::libc::malloc(1 + s.len()) as *mut u8;
            ptr::copy_nonoverlapping(s.as_ptr(), p, s.len());
            *(p.offset(s.len() as isize)) = 0;
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
}

impl Clone for VarLenString {
    fn clone(&self) -> VarLenString {
        VarLenString::new(&*self)
    }
}

impl_string_traits!(VarLenString, VarLenString);

#[cfg(test)]
pub mod tests {
    use super::VarLenString;
    use types::{ValueType, ToValueType};

    type S = VarLenString;

    #[test]
    pub fn test_value_type() {
        use std::mem;

        assert_eq!(S::value_type(), ValueType::VarLenString);
        assert_eq!(S::value_type().size(), mem::size_of::<*mut u8>());
        assert_eq!(mem::size_of::<S>(), S::value_type().size());
    }

    #[test]
    pub fn test_empty_default() {
        assert!(S::new("").is_empty());
        assert!(S::new("\0").is_empty());
        assert!(S::default().is_empty());
    }

    #[test]
    pub fn test_into_from() {
        assert_eq!(S::from("abc").as_str(), "abc");
        assert_eq!(S::from("abc".to_owned()).as_str(), "abc");
        let v: Vec<u8> = S::from("abc").into();
        assert_eq!(v, "abc".as_bytes().to_vec());
    }

    #[test]
    pub fn test_string_traits() {
        use std::borrow::Borrow;
        use std::hash::{Hash, Hasher, SipHasher};

        let s = VarLenString::new("abc");
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
