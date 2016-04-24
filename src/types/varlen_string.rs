use std::borrow::{Borrow, Cow};
use std::ffi::CStr;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, Index, RangeFull};
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

impl<'a> From<&'a str> for VarLenString {
    fn from(s: &'a str) -> VarLenString {
        VarLenString::from_str(s)
    }
}

impl From<String> for VarLenString {
    fn from(s: String) -> VarLenString {
        VarLenString::from_str(&s)
    }
}

impl Into<Vec<u8>> for VarLenString {
    fn into(self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl Deref for VarLenString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.as_bytes()) }
    }
}

impl Borrow<str> for VarLenString {
    #[inline]
    fn borrow(&self) -> &str {
        self
    }
}

impl AsRef<str> for VarLenString {
    #[inline]
    fn as_ref(&self) -> &str {
        self
    }
}

impl Index<RangeFull> for VarLenString {
    type Output = str;

    #[inline]
    fn index(&self, _: RangeFull) -> &str {
        self
    }
}

impl PartialEq for VarLenString {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self[..], &other[..])
    }
    #[inline]
    fn ne(&self, other: &Self) -> bool {
        PartialEq::ne(&self[..], &other[..])
    }
}

impl Eq for VarLenString { }

macro_rules! impl_eq {
    ($lhs:ty, $rhs: ty) => {
        impl<'a> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool { PartialEq::eq(&self[..], &other[..]) }
            #[inline]
            fn ne(&self, other: &$rhs) -> bool { PartialEq::ne(&self[..], &other[..]) }
        }

        impl<'a> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool { PartialEq::eq(&self[..], &other[..]) }
            #[inline]
            fn ne(&self, other: &$lhs) -> bool { PartialEq::ne(&self[..], &other[..]) }
        }
    }
}

impl_eq!(VarLenString, str);
impl_eq!(VarLenString, &'a str);
impl_eq!(VarLenString, String);
impl_eq!(VarLenString, Cow<'a, str>);

impl fmt::Debug for VarLenString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl fmt::Display for VarLenString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl Hash for VarLenString {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        (**self).hash(hasher)
    }
}

impl Default for VarLenString {
    #[inline]
    fn default() -> VarLenString {
        VarLenString::new()
    }
}
