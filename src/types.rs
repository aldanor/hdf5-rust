use std::borrow::Borrow;
use std::ffi::CStr;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::{Deref, Index, RangeFull};
use std::ptr;
use std::str;

use libc::c_char;

use ffi::h5t::hvl_t;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntSize { U1 = 1, U2 = 2, U4 = 4, U8 = 8 }

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FloatSize { U4 = 4, U8 = 8 }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumMember {
    pub name: String,
    pub value: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumType {
    pub size: IntSize,
    pub signed: bool,
    pub members: Vec<EnumMember>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundField {
    pub name: String,
    pub ty: ValueType,
    pub offset: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundType {
    pub fields: Vec<CompoundField>,
    pub size: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValueType {
    Integer(IntSize),
    Unsigned(IntSize),
    Float(FloatSize),
    Boolean,
    Enum(EnumType),
    Compound(CompoundType),
    FixedArray(Box<ValueType>, usize),
    FixedString(usize),
    VarLenArray(Box<ValueType>),
    VarLenString,
}

impl ValueType {
    pub fn size(&self) -> usize {
        use self::ValueType::*;

        match *self {
            Integer(size) => size as usize,
            Unsigned(size) => size as usize,
            Float(size) => size as usize,
            Boolean => 1,
            Enum(ref enum_type) => enum_type.size as usize,
            Compound(ref compound) => compound.size,
            FixedArray(ref ty, len) => ty.size() * len,
            FixedString(len) => mem::size_of::<c_char>() * (len + 1),
            VarLenArray(_) => mem::size_of::<hvl_t>(),
            VarLenString => mem::size_of::<*const c_char>(),
        }
    }
}

pub unsafe trait ToValueType {
    fn value_type() -> ValueType;
}

macro_rules! impl_value_type {
    ($ty:ty, $variant:ident, $size:expr) => (
        unsafe impl ToValueType for $ty {
            fn value_type() -> ValueType {
                $crate::types::ValueType::$variant($size)
            }
        }
    )
}

impl_value_type!(i8, Integer, IntSize::U1);
impl_value_type!(i16, Integer, IntSize::U2);
impl_value_type!(i32, Integer, IntSize::U4);
impl_value_type!(i64, Integer, IntSize::U8);
impl_value_type!(u8, Unsigned, IntSize::U1);
impl_value_type!(u16, Unsigned, IntSize::U2);
impl_value_type!(u32, Unsigned, IntSize::U4);
impl_value_type!(u64, Unsigned, IntSize::U8);
impl_value_type!(f32, Float, FloatSize::U4);
impl_value_type!(f64, Float, FloatSize::U8);

#[cfg(target_pointer_width = "32")]
impl_value_type!(isize, Integer, IntSize::U4);
#[cfg(target_pointer_width = "32")]
impl_value_type!(usize, Unsigned, IntSize::U4);

#[cfg(target_pointer_width = "64")]
impl_value_type!(isize, Integer, IntSize::U8);
#[cfg(target_pointer_width = "64")]
impl_value_type!(usize, Unsigned, IntSize::U8);

unsafe impl ToValueType for bool {
    fn value_type() -> ValueType {
        ValueType::Boolean
    }
}

pub unsafe trait Array {
    type Item;
    fn as_ptr(&self) -> *const Self::Item;
    fn as_mut_ptr(&mut self) -> *mut Self::Item;
    fn capacity() -> usize;
}

macro_rules! impl_array {
    () => ();

    ($n:expr, $($ns:expr,)*) => (
        unsafe impl<T> Array for [T; $n] {
            type Item = T;
            #[inline(always)]
            fn as_ptr(&self) -> *const T { self as *const _ as *const _ }
            #[inline(always)]
            fn as_mut_ptr(&mut self) -> *mut T { self as *mut _ as *mut _}
            #[inline(always)]
            fn capacity() -> usize { $n }
        }

        impl_array!($($ns,)*);
    );
}

impl_array!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
            16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,);
impl_array!(32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
            48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,);
impl_array!(64, 70, 72, 80, 90, 96, 100, 110, 120, 128, 130, 140, 150,
            160, 170, 180, 190, 192, 200, 210, 220, 224, 230, 240, 250,);
impl_array!(256, 300, 384, 400, 500, 512, 600, 700, 768, 800, 900, 1000, 1024,
            2048, 4096, 8192, 16384, 32768,);

unsafe impl<T: Array<Item=I>, I: ToValueType> ToValueType for T {
    fn value_type() -> ValueType {
        ValueType::FixedArray(
            Box::new(<I as ToValueType>::value_type()),
            <T as Array>::capacity()
        )
    }
}

#[macro_export]
macro_rules! h5def {
    ($(#[repr($t:ident)] $(#[$a:meta])* enum $s:ident { $($i:ident = $v:expr),+$(,)* })*) => (
        $(
            #[allow(dead_code)] #[derive(Clone, Copy)] #[repr($t)] $(#[$a])*
            enum $s { $($i = $v),+ }
            h5def!(@impl_enum $s($t) { $($i = $v),+ });
        )*
    );

    ($(#[repr($t:ident)] $(#[$a:meta])* pub enum $s:ident { $($i:ident = $v:expr),+$(,)* })*) => (
        $(
            #[allow(dead_code)] #[derive(Clone, Copy)] #[repr($t)] $(#[$a])*
            pub enum $s { $($i = $v),+ }
            h5def!(@impl_enum $s($t) { $($i = $v),+ });
        )*
    );

    ($($(#[$a:meta])* struct $s:ident { $($i:ident: $t:ty),+$(,)* })*) => (
        $(
            #[allow(dead_code)] #[derive(Clone)] #[repr(C)] $(#[$a])*
            struct $s { $($i: $t),+ }
            h5def!(@impl_struct $s { $($i: $t),+ });
        )*
    );

    ($($(#[$a:meta])* pub struct $s:ident { $($i:ident: $t:ty),+$(,)* })*) => (
        $(
            #[allow(dead_code)] #[derive(Clone)] #[repr(C)] $(#[$a])*
            pub struct $s { $($i: $t),+ }
            h5def!(@impl_struct $s { $($i: $t),+ });
        )*
    );

    ($($(#[$a:meta])* pub struct $s:ident { $(pub $i:ident: $t:ty),+$(,)* })*) => (
        $(
            #[allow(dead_code)] #[derive(Clone)] #[repr(C)] $(#[$a])*
            pub struct $s { $(pub $i: $t),+ }
            h5def!(@impl_struct $s { $($i: $t),+ });
        )*
    );

    (@impl_enum $s:ident($t:ident) { $($i:ident = $v:expr),+ }) => (
        unsafe impl $crate::types::ToValueType for $s {
            fn value_type() -> $crate::types::ValueType {
                $crate::types::ValueType::Enum(
                    $crate::types::EnumType {
                        size: match ::std::mem::size_of::<$t>() {
                            1 => IntSize::U1, 2 => IntSize::U2, 4 => IntSize::U4, 8 => IntSize::U8,
                            _ => panic!("invalid int size"),
                        },
                        signed: ::std::$t::MIN != 0,
                        members: vec![$(
                            $crate::types::EnumMember {
                                name: stringify!($i).into(),
                                value: $v as $t as u64,
                            }),+],
                    }
                )
            }
        }
    );

    (@impl_struct $s:ident { $($i:ident: $t:ty),+ }) => (
        unsafe impl $crate::types::ToValueType for $s {
            fn value_type() -> $crate::types::ValueType {
                let base = 0usize as *const $s;
                $crate::types::ValueType::Compound(
                    $crate::types::CompoundType {
                        fields: vec![$(
                            $crate::types::CompoundField {
                                name: stringify!($i).into(),
                                ty: <$t as $crate::types::ToValueType>::value_type(),
                                offset: unsafe { &((*base).$i) as *const $t as usize }
                            }),+],
                        size: ::std::mem::size_of::<$s>(),
                    }
                )
            }
        }
    );
}

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
    use super::*;
    use super::ValueType as VT;

    #[test]
    pub fn test_scalar_types() {
        assert_eq!(bool::value_type(), VT::Boolean);
        assert_eq!(i8::value_type(), VT::Integer(IntSize::U1));
        assert_eq!(i16::value_type(), VT::Integer(IntSize::U2));
        assert_eq!(i32::value_type(), VT::Integer(IntSize::U4));
        assert_eq!(i64::value_type(), VT::Integer(IntSize::U8));
        assert_eq!(u8::value_type(), VT::Unsigned(IntSize::U1));
        assert_eq!(u16::value_type(), VT::Unsigned(IntSize::U2));
        assert_eq!(u32::value_type(), VT::Unsigned(IntSize::U4));
        assert_eq!(u64::value_type(), VT::Unsigned(IntSize::U8));
        assert_eq!(f32::value_type(), VT::Float(FloatSize::U4));
        assert_eq!(f64::value_type(), VT::Float(FloatSize::U8));

        assert_eq!(bool::value_type().size(), 1);
        assert_eq!(i16::value_type().size(), 2);
        assert_eq!(u32::value_type().size(), 4);
        assert_eq!(f64::value_type().size(), 8);
    }

    #[test]
    #[cfg(target_pointer_width = "32")]
    pub fn test_ptr_sized_ints() {
        assert_eq!(isize::value_type(), VT::Integer(IntSize::U4));
        assert_eq!(usize::value_type(), VT::Unsigned(IntSize::U4));

        assert_eq!(usize::value_type().size(), 4);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    pub fn test_ptr_sized_ints() {
        assert_eq!(isize::value_type(), VT::Integer(IntSize::U8));
        assert_eq!(usize::value_type(), VT::Unsigned(IntSize::U8));

        assert_eq!(usize::value_type().size(), 8);
    }

    #[test]
    pub fn test_array_trait() {
        type T = [u32; 256];
        assert_eq!(<T as Array>::capacity(), 256);
        let mut arr = [1, 2, 3];
        assert_eq!(arr.as_ptr(), &arr[0] as *const _);
        assert_eq!(arr.as_mut_ptr(), &mut arr[0] as *mut _);
    }

    #[test]
    pub fn test_fixed_size_array() {
        type T = [u32; 256];
        assert_eq!(T::value_type(), VT::FixedArray(Box::new(VT::Unsigned(IntSize::U4)), 256));
        type S = [T; 4];
        assert_eq!(S::value_type(), VT::FixedArray(Box::new(T::value_type()), 4));
    }

    #[test]
    pub fn test_enum_type() {
        h5def!(#[repr(i64)] enum Foo { A = 1, B = -2 });
        assert_eq!(Foo::value_type(), VT::Enum(EnumType {
            size: IntSize::U8,
            signed: true,
            members: vec![
                EnumMember { name: "A".into(), value: 1 },
                EnumMember { name: "B".into(), value: -2i64 as u64 },
            ]
        }));
        assert_eq!(Foo::value_type().size(), 8);

        h5def!(#[repr(u8)] #[derive(Debug)] pub enum Bar { A = 1, B = 2, });
        assert_eq!(Bar::value_type(), VT::Enum(EnumType {
            size: IntSize::U1,
            signed: false,
            members: vec![
                EnumMember { name: "A".into(), value: 1 },
                EnumMember { name: "B".into(), value: 2 },
            ]
        }));
        assert_eq!(format!("{:?}", Bar::A), "A");
        assert_eq!(Bar::value_type().size(), 1);

        h5def!(#[repr(u8)] enum E1 { A = 1, B = 2 }
               #[repr(u8)] enum E2 { A = 1, B = 2});
        assert_eq!(E1::value_type(), Bar::value_type());
        assert_eq!(E2::value_type(), Bar::value_type());

        h5def!(#[repr(u8)] pub enum E3 { A = 1, B = 2 }
               #[repr(u8)] pub enum E4 { A = 1, B = 2});
        assert_eq!(E3::value_type(), Bar::value_type());
        assert_eq!(E4::value_type(), Bar::value_type());
    }

    #[test]
    pub fn test_compound_type() {
        h5def!(struct Foo { a: i64, b: u64 });
        assert_eq!(Foo::value_type(), VT::Compound(CompoundType {
            fields: vec![
                CompoundField { name: "a".into(), ty: i64::value_type(), offset: 0 },
                CompoundField { name: "b".into(), ty: u64::value_type(), offset: 8 },
            ],
            size: 16,
        }));
        assert_eq!(Foo::value_type().size(), 16);

        h5def!(pub struct Bar { a: i64, b: u64 });
        assert_eq!(Bar::value_type(), Foo::value_type());

        h5def!(#[derive(Debug)] pub struct Baz { pub a: i64, pub b: u64 });
        assert_eq!(Baz::value_type(), Foo::value_type());
        assert!(format!("{:?}", Baz { a: 1, b: 2 }).len() > 0);

        h5def!(struct S1 { a: i64, b: u64 }
               struct S2 { a: i64, b: u64 } );
        assert_eq!(S1::value_type(), Foo::value_type());
        assert_eq!(S2::value_type(), Foo::value_type());

        h5def!(pub struct S3 { a: i64, b: u64 }
               pub struct S4 { a: i64, b: u64 } );
        assert_eq!(S3::value_type(), Foo::value_type());
        assert_eq!(S4::value_type(), Foo::value_type());

        h5def!(pub struct S5 { pub a: i64, pub b: u64 }
               pub struct S6 { pub a: i64, pub b: u64 });
        assert_eq!(S5::value_type(), Foo::value_type());
        assert_eq!(S6::value_type(), Foo::value_type());
    }

    #[test]
    pub fn test_fixed_string() {
        use libc::c_char;
        use std::mem;
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

        use std::borrow::Borrow;
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

        use std::hash::{Hash, Hasher, SipHasher};
        let (mut h1, mut h2) = (SipHasher::new(), SipHasher::new());
        s.hash(&mut h1);
        "abc".hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }
}
