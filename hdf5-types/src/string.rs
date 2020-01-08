use std::borrow::{Borrow, Cow};
use std::error::Error as StdError;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::{Deref, Index, RangeFull};
use std::ptr;
use std::slice::{self, SliceIndex};
use std::str::{self, FromStr};

use ascii::{AsAsciiStr, AsAsciiStrError, AsciiStr};

use crate::array::Array;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StringError {
    InternalNull,
    InsufficientCapacity,
    AsciiError(AsAsciiStrError),
    #[doc(hidden)]
    __Incomplete,
}

impl From<AsAsciiStrError> for StringError {
    fn from(err: AsAsciiStrError) -> Self {
        StringError::AsciiError(err)
    }
}

impl StdError for StringError {}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StringError::InternalNull => {
                write!(f, "string error: variable length string with internal null")
            }
            StringError::InsufficientCapacity => {
                write!(f, "string error: insufficient capacity for fixed sized string")
            }
            StringError::AsciiError(err) => write!(f, "string error: {}", err),
            _ => write!(f, ""),
        }
    }
}

// ================================================================================

macro_rules! impl_string_eq {
    ($lhs:ty, $rhs:ty $(,$t:ident: $b:ident<$a:ident=$v:ty>)*) => {
        impl<'a $(,$t: $b<$a=$v>)*> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
        }
    }
}

macro_rules! impl_string_traits {
    ($nm:ident, $ty:ty $(,$t:ident: $b:ident<$a:ident=$v:ty>)*) => (
        impl<'a $(,$t: $b<$a=$v>)*> fmt::Debug for $ty {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.as_str().fmt(f)
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> fmt::Display for $ty {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.as_str().fmt(f)
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> Hash for $ty {
            #[inline]
            fn hash<H: Hasher>(&self, hasher: &mut H) {
                Hash::hash(&self.as_bytes(), hasher)
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> Default for $ty {
            #[inline]
            fn default() -> $ty {
                $nm::new()
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> Deref for $ty {
            type Target = str;

            #[inline]
            fn deref(&self) -> &str {
                self.as_str()
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> Borrow<str> for $ty {
            #[inline]
            fn borrow(&self) -> &str {
                self
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> AsRef<str> for $ty {
            #[inline]
            fn as_ref(&self) -> &str {
                self
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> AsRef<[u8]> for $ty {
            #[inline]
            fn as_ref(&self) -> &[u8] {
                self.as_bytes()
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> Index<RangeFull> for $ty {
            type Output = str;

            #[inline]
            fn index(&self, _: RangeFull) -> &str {
                self
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> PartialEq for $ty {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> Eq for $ty { }

        impl_string_eq!($ty, str $(,$t: $b<$a=$v>)*);
        impl_string_eq!($ty, &'a str $(,$t: $b<$a=$v>)*);
        impl_string_eq!($ty, String $(,$t: $b<$a=$v>)*);
        impl_string_eq!($ty, Cow<'a, str> $(,$t: $b<$a=$v>)*);

        impl<'a $(,$t: $b<$a=$v>)*> From<$ty> for String {
            #[inline]
            fn from(s: $ty) -> String {
                s.as_str().to_owned()
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> From<&'a $ty> for &'a [u8] {
            #[inline]
            fn from(s: &$ty) -> &[u8] {
                s.as_bytes()
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> From<&'a $ty> for &'a str {
            #[inline]
            fn from(s: &$ty) -> &str {
                s.as_str()
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> From<$ty> for Vec<u8> {
            #[inline]
            fn from(s: $ty) -> Vec<u8> {
                s.as_bytes().to_vec()
            }
        }
    )
}

impl_string_traits!(FixedAscii, FixedAscii<A>, A: Array<Item = u8>);
impl_string_traits!(FixedUnicode, FixedUnicode<A>, A: Array<Item = u8>);
impl_string_traits!(VarLenAscii, VarLenAscii);
impl_string_traits!(VarLenUnicode, VarLenUnicode);

// ================================================================================

#[repr(C)]
pub struct VarLenAscii {
    ptr: *mut u8,
}

impl Drop for VarLenAscii {
    #[inline]
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { libc::free(self.ptr as *mut _) };
        }
    }
}

impl Clone for VarLenAscii {
    #[inline]
    fn clone(&self) -> Self {
        unsafe { Self::from_bytes(self.as_bytes()) }
    }
}

impl VarLenAscii {
    #[inline]
    pub fn new() -> Self {
        unsafe {
            let ptr = libc::malloc(1) as *mut _;
            *ptr = 0;
            VarLenAscii { ptr }
        }
    }

    #[inline]
    unsafe fn from_bytes(bytes: &[u8]) -> Self {
        let ptr = libc::malloc(bytes.len() + 1) as *mut _;
        ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
        *ptr.add(bytes.len()) = 0;
        VarLenAscii { ptr }
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe { libc::strlen(self.ptr as *const _) }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr as *const _, self.len()) }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { mem::transmute(self.as_bytes()) }
    }

    #[inline]
    pub unsafe fn from_ascii_unchecked<B: ?Sized + AsRef<[u8]>>(bytes: &B) -> Self {
        Self::from_bytes(bytes.as_ref())
    }

    pub fn from_ascii<B: ?Sized + AsRef<[u8]>>(bytes: &B) -> Result<Self, StringError> {
        let bytes = bytes.as_ref();
        if !bytes.iter().all(|&c| c != 0) {
            return Err(StringError::InternalNull);
        }
        let s = AsciiStr::from_ascii(bytes)?;
        unsafe { Ok(Self::from_bytes(s.as_bytes())) }
    }
}

impl AsAsciiStr for VarLenAscii {
    type Inner = u8;

    #[inline]
    fn slice_ascii<R>(&self, range: R) -> Result<&AsciiStr, AsAsciiStrError>
    where
        R: SliceIndex<[u8], Output = [u8]>,
    {
        self.as_bytes().slice_ascii(range)
    }

    #[inline]
    fn as_ascii_str(&self) -> Result<&AsciiStr, AsAsciiStrError> {
        AsciiStr::from_ascii(self.as_bytes())
    }

    #[inline]
    unsafe fn as_ascii_str_unchecked(&self) -> &AsciiStr {
        AsciiStr::from_ascii_unchecked(self.as_bytes())
    }
}

// ================================================================================

#[repr(C)]
pub struct VarLenUnicode {
    ptr: *mut u8,
}

impl Drop for VarLenUnicode {
    #[inline]
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { libc::free(self.ptr as *mut _) };
        }
    }
}

impl Clone for VarLenUnicode {
    #[inline]
    fn clone(&self) -> Self {
        unsafe { Self::from_bytes(self.as_bytes()) }
    }
}

impl VarLenUnicode {
    #[inline]
    pub fn new() -> Self {
        unsafe {
            let ptr = libc::malloc(1) as *mut _;
            *ptr = 0;
            VarLenUnicode { ptr }
        }
    }

    #[inline]
    unsafe fn from_bytes(bytes: &[u8]) -> Self {
        let ptr = libc::malloc(bytes.len() + 1) as *mut _;
        ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
        *ptr.add(bytes.len()) = 0;
        VarLenUnicode { ptr }
    }

    #[inline]
    unsafe fn raw_len(&self) -> usize {
        libc::strlen(self.ptr as *const _)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.as_str().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        unsafe { self.raw_len() == 0 }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr as *const _, self.raw_len()) }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.as_bytes()) }
    }

    #[inline]
    pub unsafe fn from_str_unchecked<S: Borrow<str>>(s: S) -> Self {
        Self::from_bytes(s.borrow().as_bytes())
    }
}

impl FromStr for VarLenUnicode {
    type Err = StringError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        if s.chars().all(|c| c != '\0') {
            unsafe { Ok(Self::from_bytes(s.as_bytes())) }
        } else {
            Err(StringError::InternalNull)
        }
    }
}

// ================================================================================

#[repr(C)]
#[derive(Copy)]
pub struct FixedAscii<A: Array<Item = u8>> {
    buf: A,
}

impl<A: Array<Item = u8>> Clone for FixedAscii<A> {
    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            let mut buf = mem::MaybeUninit::<A>::uninit();
            ptr::copy_nonoverlapping(self.buf.as_ptr(), buf.as_mut_ptr() as *mut _, A::capacity());
            FixedAscii { buf: buf.assume_init() }
        }
    }
}

impl<A: Array<Item = u8>> FixedAscii<A> {
    #[inline]
    pub fn new() -> Self {
        unsafe { FixedAscii { buf: mem::zeroed() } }
    }

    #[inline]
    unsafe fn from_bytes(bytes: &[u8]) -> Self {
        let len = if bytes.len() < A::capacity() { bytes.len() } else { A::capacity() };
        let mut buf: A = mem::zeroed();
        ptr::copy_nonoverlapping(bytes.as_ptr(), buf.as_mut_ptr() as *mut _, len);
        FixedAscii { buf }
    }

    #[inline]
    fn as_raw_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.buf.as_ptr(), A::capacity()) }
    }

    #[inline]
    pub fn capacity() -> usize {
        A::capacity()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.as_raw_slice().iter().rev().skip_while(|&c| *c == 0).count()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.as_raw_slice().iter().all(|&c| c == 0)
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.buf.as_ptr()
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.as_raw_slice()[..self.len()]
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { mem::transmute(self.as_bytes()) }
    }

    #[inline]
    pub unsafe fn from_ascii_unchecked<B: ?Sized + AsRef<[u8]>>(bytes: &B) -> Self {
        Self::from_bytes(bytes.as_ref())
    }

    pub fn from_ascii<B: ?Sized + AsRef<[u8]>>(bytes: &B) -> Result<Self, StringError> {
        let bytes = bytes.as_ref();
        if bytes.len() > A::capacity() {
            return Err(StringError::InsufficientCapacity);
        }
        let s = AsciiStr::from_ascii(bytes)?;
        unsafe { Ok(Self::from_bytes(s.as_bytes())) }
    }
}

impl<A: Array<Item = u8>> AsAsciiStr for FixedAscii<A> {
    type Inner = u8;

    #[inline]
    fn slice_ascii<R>(&self, range: R) -> Result<&AsciiStr, AsAsciiStrError>
    where
        R: SliceIndex<[u8], Output = [u8]>,
    {
        self.as_bytes().slice_ascii(range)
    }

    #[inline]
    fn as_ascii_str(&self) -> Result<&AsciiStr, AsAsciiStrError> {
        AsciiStr::from_ascii(self.as_bytes())
    }

    #[inline]
    unsafe fn as_ascii_str_unchecked(&self) -> &AsciiStr {
        AsciiStr::from_ascii_unchecked(self.as_bytes())
    }
}

// ================================================================================

#[repr(C)]
#[derive(Copy)]
pub struct FixedUnicode<A: Array<Item = u8>> {
    buf: A,
}

impl<A: Array<Item = u8>> Clone for FixedUnicode<A> {
    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            let mut buf = mem::MaybeUninit::<A>::uninit();
            ptr::copy_nonoverlapping(self.buf.as_ptr(), buf.as_mut_ptr() as *mut _, A::capacity());
            FixedUnicode { buf: buf.assume_init() }
        }
    }
}

impl<A: Array<Item = u8>> FixedUnicode<A> {
    #[inline]
    pub fn new() -> Self {
        unsafe { FixedUnicode { buf: mem::zeroed() } }
    }

    #[inline]
    unsafe fn from_bytes(bytes: &[u8]) -> Self {
        let len = if bytes.len() < A::capacity() { bytes.len() } else { A::capacity() };
        let mut buf: A = mem::zeroed();
        ptr::copy_nonoverlapping(bytes.as_ptr(), buf.as_mut_ptr() as *mut _, len);
        FixedUnicode { buf }
    }

    #[inline]
    fn as_raw_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.buf.as_ptr(), A::capacity()) }
    }

    #[inline]
    fn raw_len(&self) -> usize {
        self.as_raw_slice().iter().rev().skip_while(|&c| *c == 0).count()
    }

    #[inline]
    pub fn capacity() -> usize {
        A::capacity()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.as_str().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.raw_len() == 0
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.buf.as_ptr()
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.as_raw_slice()[..self.raw_len()]
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { mem::transmute(self.as_bytes()) }
    }

    #[inline]
    pub unsafe fn from_str_unchecked<S: Borrow<str>>(s: S) -> Self {
        Self::from_bytes(s.borrow().as_bytes())
    }
}

impl<A> FromStr for FixedUnicode<A>
where
    A: Array<Item = u8>,
{
    type Err = StringError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        if s.as_bytes().len() <= A::capacity() {
            unsafe { Ok(Self::from_bytes(s.as_bytes())) }
        } else {
            Err(StringError::InsufficientCapacity)
        }
    }
}

// ================================================================================

#[cfg(test)]
pub mod tests {
    use super::*;

    use std::borrow::Borrow;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::slice;

    use ascii::{AsAsciiStr, AsciiString};
    use quickcheck::{Arbitrary, Gen};

    type VA = VarLenAscii;
    type VU = VarLenUnicode;
    type FA = FixedAscii<[u8; 1024]>;
    type FU = FixedUnicode<[u8; 1024]>;

    #[derive(Clone, Debug)]
    pub struct AsciiGen(pub Vec<u8>);

    #[derive(Clone, Debug)]
    pub struct UnicodeGen(pub String);

    impl Arbitrary for AsciiGen {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut bytes: Vec<u8> = Arbitrary::arbitrary(g);
            for c in &mut bytes {
                *c = *c % 0x7e + 1;
            }
            if bytes.len() > 1024 {
                bytes = bytes[..1024].to_vec();
            }
            AsciiGen(bytes)
        }
    }

    impl AsciiGen {
        pub fn expected(&self) -> AsciiString {
            AsciiString::from_ascii(self.0.clone()).unwrap()
        }

        pub fn as_bytes(&self) -> &[u8] {
            &self.0
        }
    }

    impl Arbitrary for UnicodeGen {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let s: String = Arbitrary::arbitrary(g);
            let mut s: String = s.chars().filter(|&c| c != '\0').collect();
            while s.as_bytes().len() > 1024 {
                let n = s.len() - 1;
                s.truncate(n);
            }
            UnicodeGen(s)
        }
    }

    impl UnicodeGen {
        pub fn expected(&self) -> String {
            self.0.clone()
        }

        pub fn as_bytes(&self) -> &[u8] {
            self.0.as_bytes()
        }
    }

    #[test]
    pub fn test_internal_null() {
        assert!(VA::from_ascii("foo\0bar").is_err());
        assert!(VU::from_str("foo\0bar").is_err());
    }

    #[test]
    pub fn test_capacity() {
        type A = FixedAscii<[u8; 2]>;
        type U = FixedUnicode<[u8; 2]>;
        assert_eq!(A::from_ascii("ab").unwrap().as_str(), "ab");
        assert!(A::from_ascii("abc").is_err());
        assert_eq!(U::from_str("ab").unwrap().as_str(), "ab");
        assert!(U::from_str("abc").is_err());
        assert_eq!(U::from_str("®").unwrap().as_str(), "®");
        assert!(U::from_str("€").is_err());
    }

    #[test]
    pub fn test_non_ascii() {
        assert!(VA::from_ascii("®").is_err());
        assert!(VA::from_ascii("€").is_err());
        assert!(FA::from_ascii("®").is_err());
        assert!(FA::from_ascii("€").is_err());
    }

    #[test]
    pub fn test_null_padding() {
        type A = FixedAscii<[u8; 3]>;
        type U = FixedUnicode<[u8; 3]>;
        assert_eq!(A::from_ascii("a\0b").unwrap().as_str(), "a\0b");
        assert_eq!(A::from_ascii("a\0\0").unwrap().as_str(), "a");
        assert!(A::from_ascii("\0\0\0").unwrap().is_empty());
        assert_eq!(U::from_str("a\0b").unwrap().as_str(), "a\0b");
        assert_eq!(U::from_str("a\0\0").unwrap().as_str(), "a");
        assert!(U::from_str("\0\0\0").unwrap().is_empty());
    }

    macro_rules! test_default {
        ($test_name:ident, $ty:ident) => {
            #[test]
            pub fn $test_name() {
                for s in &vec![$ty::new(), Default::default()] {
                    assert_eq!(s.len(), 0);
                    assert!(s.is_empty());
                    assert_eq!(s.as_bytes(), &[] as &[u8]);
                    assert_eq!(s.as_str(), "");
                }
            }
        };
    }

    test_default!(test_default_va, VA);
    test_default!(test_default_fa, FA);
    test_default!(test_default_vu, VU);
    test_default!(test_default_fu, FU);

    macro_rules! check_invariants {
        ($s:ident, $exp:ident, $bytes:ident) => {{
            assert_eq!($s.len(), $exp.len());
            assert_eq!($s.is_empty(), $exp.is_empty());
            assert_eq!($s.is_empty(), $bytes.is_empty());
            assert_eq!($s.as_str(), $exp.as_str());
            assert_eq!($s.as_bytes(), $bytes);
            assert_eq!($s.clone().as_bytes(), $s.as_bytes());
            let (mut h1, mut h2) = (DefaultHasher::new(), DefaultHasher::new());
            $s.hash(&mut h1);
            $bytes.hash(&mut h2);
            assert_eq!(h1.finish(), h2.finish());
            assert_eq!(format!("{}", $s), $s.as_str());
            assert_eq!(format!("{:?}", $s), format!("{:?}", $s.as_str()));
            assert_eq!($s.borrow() as &str, $s.as_str());
            assert_eq!($s.as_ref() as &str, $s.as_str());
            assert_eq!($s.as_ref() as &[u8], $bytes);
            assert_eq!(&$s[..], $s.as_str());
            assert_eq!($s, $s);
            assert_eq!($s, $s.as_str());
            assert_eq!($s.as_str(), $s);
            assert_eq!(&$s, $s.as_str());
            assert_eq!($s.as_str(), &$s);
            assert_eq!($s, $s.as_str().to_owned());
            assert_eq!($s.as_str().to_owned(), $s);
            assert_eq!(&*$s, $s.as_str());
            let v: Vec<u8> = $s.clone().into();
            assert_eq!(v, $bytes.to_vec());
            let v: &[u8] = (&$s).into();
            assert_eq!(v, $bytes);
            let v: &str = (&$s).into();
            assert_eq!(v, $s.as_str());
            let v: String = $s.clone().into();
            assert_eq!(v, $s.as_str().to_owned());
            unsafe {
                assert_eq!(slice::from_raw_parts($s.as_ptr(), $s.len()), $bytes);
            }
        }};
    }

    macro_rules! test_quickcheck_ascii {
        ($test_name:ident, $ty:ident) => {
            quickcheck! {
                fn $test_name(b: AsciiGen) -> () {
                    let (exp, bytes) = (b.expected(), b.as_bytes());
                    let s = $ty::from_ascii(bytes).unwrap();
                    check_invariants!(s, exp, bytes);
                    assert_eq!(s.len(), bytes.len());
                    assert_eq!(s.as_ascii_str().unwrap(), exp);
                    unsafe {
                        assert_eq!($ty::from_ascii_unchecked(bytes).as_bytes(), bytes);
                        assert_eq!(s.as_ascii_str_unchecked(), exp);
                    }
                }
            }
        };
    }

    test_quickcheck_ascii!(test_quickcheck_va, VA);
    test_quickcheck_ascii!(test_quickcheck_fa, FA);

    macro_rules! test_quickcheck_unicode {
        ($test_name:ident, $ty:ident) => {
            quickcheck! {
                fn $test_name(b: UnicodeGen) -> () {
                    let (exp, bytes) = (b.expected(), b.as_bytes());
                    let s = $ty::from_str(exp.as_str()).unwrap();
                    check_invariants!(s, exp, bytes);
                    unsafe {
                        assert_eq!($ty::from_str_unchecked(exp.as_str()).as_bytes(), bytes);
                    }
                }
            }
        };
    }

    test_quickcheck_unicode!(test_quickcheck_vu, VU);
    test_quickcheck_unicode!(test_quickcheck_fu, FU);
}
