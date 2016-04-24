use std::ffi::CStr;
use std::mem;
use std::ptr;
use std::str;

use libc::c_char;

use types::{ValueType, ToValueType, Array};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FixedString<A: Array<Item=u8>> {
    buf: A,
    eof: u8,
}

unsafe impl<A: Array<Item=u8>> ToValueType for FixedString<A> {
    fn value_type() -> ValueType {
        ValueType::FixedString(A::capacity())
    }
}

impl<A: Array<Item=u8>> FixedString<A> {
    pub fn new(s: &str) -> Self {
        let len = if s.len() < Self::capacity() {
            s.len()
        } else {
            Self::capacity()
        };
        unsafe {
            let mut buf: A =  mem::zeroed();
            ptr::copy_nonoverlapping(s.as_ptr(), buf.as_mut_ptr() as *mut u8, len);
            FixedString { buf: buf, eof: 0 }
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.buf.as_ptr()
    }

    #[inline(always)]
    pub fn capacity() -> usize {
        A::capacity()
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { CStr::from_ptr(self.buf.as_ptr() as *const c_char).to_bytes() }
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

impl_string_traits!(FixedString, FixedString<A>, A: Array<Item=u8>);

#[cfg(test)]
pub mod tests {
    use super::FixedString;
    use types::ToValueType;
    use types::ValueType as VT;

    type S = FixedString<[u8; 5]>;

    #[test]
    pub fn test_value_type() {
        use std::mem;

        assert_eq!(S::value_type(), VT::FixedString(5));
        assert_eq!(S::value_type().size(), 6);
        assert_eq!(mem::size_of::<S>(), 6);
        assert_eq!(S::capacity(), 5);
    }

    #[test]
    pub fn test_empty_default() {
        assert!(S::new("").is_empty());
        assert!(S::new("\0").is_empty());
        assert!(S::default().is_empty());
    }

    #[test]
    pub fn test_overflow() {
        assert_eq!(S::new("abcdefg"), "abcde");
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

        let s = FixedString::<[_; 5]>::new("abc");
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
