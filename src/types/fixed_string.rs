use std::borrow::Borrow;
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
pub struct FixedString<A: Array<Item=c_char>> {
    buf: A,
    eof: c_char,
}

unsafe impl<A: Array<Item=c_char>> ToValueType for FixedString<A> {
    fn value_type() -> ValueType {
        ValueType::FixedString(A::capacity())
    }
}

impl<A: Array<Item=c_char>> FixedString<A> {
    pub fn new() -> FixedString<A> {
        unsafe {
            FixedString {
                buf: mem::zeroed(),
                eof: 0,
            }
        }
    }

    pub fn as_ptr(&self) -> *const c_char {
        self.buf.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut c_char {
        self.buf.as_mut_ptr()
    }

    #[inline(always)]
    pub fn capacity() -> usize {
        A::capacity()
    }

    pub fn as_str(&self) -> &str {
        self
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            CStr::from_ptr(self.buf.as_ptr()).to_bytes()
        }
    }

    pub fn len(&self) -> usize {
        self.as_bytes().len()
    }

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

impl<'a, A: Array<Item=c_char>> From<&'a str> for FixedString<A> {
    fn from(s: &'a str) -> FixedString<A> {
        FixedString::from_str(s)
    }
}

impl<A: Array<Item=c_char>> From<String> for FixedString<A> {
    fn from(s: String) -> FixedString<A> {
        FixedString::from_str(&s)
    }
}

impl<A: Array<Item=c_char>> Deref for FixedString<A> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(self.as_bytes())
        }
    }
}

impl<A: Array<Item=c_char>> Borrow<str> for FixedString<A> {
    #[inline]
    fn borrow(&self) -> &str {
        self
    }
}

impl<A: Array<Item=c_char>> AsRef<str> for FixedString<A> {
    #[inline]
    fn as_ref(&self) -> &str {
        self
    }
}

impl<A: Array<Item=c_char>> Index<RangeFull> for FixedString<A> {
    type Output = str;

    #[inline]
    fn index(&self, _: RangeFull) -> &str {
        self
    }
}

impl<A: Array<Item=c_char>> PartialEq for FixedString<A> {
    fn eq(&self, rhs: &Self) -> bool {
        **self == **rhs
    }
}

impl<'a, A: Array<Item=c_char>> PartialEq<&'a str> for FixedString<A> {
    fn eq(&self, rhs: & &'a str) -> bool {
        &&**self == rhs
    }
}

impl<'a, A: Array<Item=c_char>> PartialEq<FixedString<A>> for &'a str {
    fn eq(&self, rhs: &FixedString<A>) -> bool {
        self == &&**rhs
    }
}

impl<A: Array<Item=c_char>> PartialEq<str> for FixedString<A> {
    fn eq(&self, rhs: &str) -> bool {
        &**self == rhs
    }
}

impl<A: Array<Item=c_char>> PartialEq<FixedString<A>> for str {
    fn eq(&self, rhs: &FixedString<A>) -> bool {
        self == &**rhs
    }
}

impl<A: Array<Item=c_char>> PartialEq<String> for FixedString<A> {
    fn eq(&self, rhs: &String) -> bool {
        &**self == rhs.as_str()
    }
}

impl<A: Array<Item=c_char>> PartialEq<FixedString<A>> for String {
    fn eq(&self, rhs: &FixedString<A>) -> bool {
        self.as_str() == &**rhs
    }
}

impl<A: Array<Item=c_char>> Eq for FixedString<A> { }

impl<A: Array<Item=c_char>> fmt::Debug for FixedString<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<A: Array<Item=c_char>> fmt::Display for FixedString<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<A: Array<Item=c_char>> Hash for FixedString<A> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        (**self).hash(h)
    }
}

#[cfg(test)]
pub mod tests {
    use std::borrow::Borrow;
    use std::hash::{Hash, Hasher, SipHasher};
    use std::mem;

    use libc::c_char;

    use super::FixedString;
    use types::ToValueType;
    use types::ValueType as VT;

    #[test]
    pub fn test_fixed_string() {
        type S = FixedString<[c_char; 5]>;
        assert_eq!(S::value_type(), VT::FixedString(5));
        assert_eq!(S::value_type().size(), 6);
        assert_eq!(mem::size_of::<S>(), 6);
        assert_eq!(S::capacity(), 5);

        assert_eq!(S::from_str("abcdefg"), "abcde");

        assert!(S::from_str("").is_empty());
        assert!(S::from_str("\0").is_empty());
        assert!(S::new().is_empty());

        assert_eq!(S::from("abc").as_str(), "abc");
        assert_eq!(S::from("abc".to_owned()).as_str(), "abc");

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
