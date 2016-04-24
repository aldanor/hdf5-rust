use std::borrow::{Borrow, Cow};
use std::ffi::CStr;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::{Deref, Index, RangeFull};
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
    pub fn new() -> FixedString<A> {
        unsafe {
            FixedString {
                buf: mem::zeroed(),
                eof: 0,
            }
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
        unsafe {
            CStr::from_ptr(self.buf.as_ptr() as *const c_char).to_bytes()
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.as_bytes().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn from_str(s: &str) -> Self {
        let len = if s.len() < Self::capacity() {
            s.len()
        } else {
            Self::capacity()
        };
        let mut fs = Self::new();
        unsafe {
            ptr::copy_nonoverlapping(s.as_ptr(), fs.buf.as_mut_ptr() as *mut u8, len);
        }
        fs
    }
}

impl<'a, A: Array<Item=u8>> From<&'a str> for FixedString<A> {
    fn from(s: &'a str) -> FixedString<A> {
        FixedString::from_str(s)
    }
}

impl<A: Array<Item=u8>> From<String> for FixedString<A> {
    fn from(s: String) -> FixedString<A> {
        FixedString::from_str(&s)
    }
}

impl<A: Array<Item=u8>> Into<Vec<u8>> for FixedString<A> {
    fn into(self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl<A: Array<Item=u8>> Deref for FixedString<A> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.as_bytes()) }
    }
}

impl<A: Array<Item=u8>> Borrow<str> for FixedString<A> {
    #[inline]
    fn borrow(&self) -> &str {
        self
    }
}

impl<A: Array<Item=u8>> AsRef<str> for FixedString<A> {
    #[inline]
    fn as_ref(&self) -> &str {
        self
    }
}

impl<A: Array<Item=u8>> Index<RangeFull> for FixedString<A> {
    type Output = str;

    #[inline]
    fn index(&self, _: RangeFull) -> &str {
        self
    }
}

impl<A: Array<Item=u8>> PartialEq for FixedString<A> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self[..], &other[..])
    }
    #[inline]
    fn ne(&self, other: &Self) -> bool {
        PartialEq::ne(&self[..], &other[..])
    }
}

impl<A: Array<Item=u8>> Eq for FixedString<A> { }

macro_rules! impl_eq {
    ($lhs:ty, $rhs: ty) => {
        impl<'a, A: Array<Item=u8>> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool { PartialEq::eq(&self[..], &other[..]) }
            #[inline]
            fn ne(&self, other: &$rhs) -> bool { PartialEq::ne(&self[..], &other[..]) }
        }

        impl<'a, A: Array<Item=u8>> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool { PartialEq::eq(&self[..], &other[..]) }
            #[inline]
            fn ne(&self, other: &$lhs) -> bool { PartialEq::ne(&self[..], &other[..]) }
        }
    }
}

impl_eq!(FixedString<A>, str);
impl_eq!(FixedString<A>, &'a str);
impl_eq!(FixedString<A>, String);
impl_eq!(FixedString<A>, Cow<'a, str>);

impl<A: Array<Item=u8>> fmt::Debug for FixedString<A> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<A: Array<Item=u8>> fmt::Display for FixedString<A> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<A: Array<Item=u8>> Hash for FixedString<A> {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        (**self).hash(hasher)
    }
}

impl<A: Array<Item=u8>> Default for FixedString<A> {
    #[inline]
    fn default() -> FixedString<A> {
        FixedString::new()
    }
}

#[cfg(test)]
pub mod tests {
    use std::borrow::Borrow;
    use std::hash::{Hash, Hasher, SipHasher};
    use std::mem;

    use super::FixedString;
    use types::ToValueType;
    use types::ValueType as VT;

    #[test]
    pub fn test_fixed_string() {
        type S = FixedString<[u8; 5]>;
        assert_eq!(S::value_type(), VT::FixedString(5));
        assert_eq!(S::value_type().size(), 6);
        assert_eq!(mem::size_of::<S>(), 6);
        assert_eq!(S::capacity(), 5);

        assert_eq!(S::from_str("abcdefg"), "abcde");

        assert!(S::from_str("").is_empty());
        assert!(S::from_str("\0").is_empty());
        assert!(S::new().is_empty());
        assert!(S::default().is_empty());

        assert_eq!(S::from("abc").as_str(), "abc");
        assert_eq!(S::from("abc".to_owned()).as_str(), "abc");
        let v: Vec<u8> = S::from("abc").into();
        assert_eq!(v, "abc".as_bytes().to_vec());

        let s = FixedString::<[_; 5]>::from_str("abc");
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
