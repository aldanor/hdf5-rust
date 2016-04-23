use std::mem;

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
    ($(#[repr($ty:ident)] $(#[$attr:meta])* enum $s:ident { $($i:ident = $v:expr),+$(,)* })*) => (
        $(
            #[allow(dead_code)] #[derive(Clone, Copy)] #[repr($ty)] $(#[$attr])*
            enum $s { $($i = $v),+ }
            h5def!(@impl_enum $s($ty) { $($i = $v),+ });
        )*
    );

    ($(#[repr($ty:ident)] $(#[$attr:meta])* pub enum $s:ident { $($i:ident = $v:expr),+$(,)* })*) => (
        $(
            #[allow(dead_code)] #[derive(Clone, Copy)] #[repr($ty)] $(#[$attr])*
            pub enum $s { $($i = $v),+ }
            h5def!(@impl_enum $s($ty) { $($i = $v),+ });
        )*
    );

    ($($(#[$attr:meta])* #[repr($ty:ty)] struct $s:ident { $($i:ident: $t:ty),+$(,)* })*) => (
        $(
            #[allow(dead_code)] #[derive(Clone)] #[repr(C)] $(#[$attr])*
            struct $s { $($i: $t),+ }
            h5def!(@impl_struct $s($ty) { $($i: $t),+ });
        )*
    );

    ($($(#[$attr:meta])* pub struct $s:ident { $($i:ident: $t:ty),+$(,)* })*) => (
        $(
            #[allow(dead_code)] #[derive(Clone)] #[repr(C)] $(#[$attr])*
            pub struct $s { $($i: $t),+ }
            h5def!(@impl_struct $s { $($i: $t),+ });
        )*
    );

    ($($(#[$attr:meta])* pub struct $s:ident { $(pub $i:ident: $t:ty),+$(,)* })*) => (
        $(
            #[allow(dead_code)] #[derive(Clone)] #[repr(C)] $(#[$attr])*
            pub struct $s { $(pub $i: $t),+ }
            h5def!(@impl_struct $s { $($i: $t),+ });
        )*
    );

    (@impl_enum $s:ident($ty:ident) { $($i:ident = $v:expr),+ }) => (
        unsafe impl $crate::types::ToValueType for $s {
            fn value_type() -> $crate::types::ValueType {
                $crate::types::ValueType::Enum(
                    $crate::types::EnumType {
                        size: match ::std::mem::size_of::<$ty>() {
                            1 => IntSize::U1, 2 => IntSize::U2, 4 => IntSize::U4, 8 => IntSize::U8,
                            _ => panic!("invalid int size"),
                        },
                        signed: ::std::$ty::MIN != 0,
                        members: vec![$(
                            $crate::types::EnumMember {
                                name: stringify!($i).into(),
                                value: $v as $ty as u64,
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

#[cfg(test)]
pub mod tests {
    use std::mem;

    use super::*;
    use super::ValueType::*;

    #[test]
    pub fn test_value_type_size() {
        assert_eq!(bool::value_type().size(), 1);
        assert_eq!(i16::value_type().size(), 2);
        assert_eq!(u32::value_type().size(), 4);
        assert_eq!(f64::value_type().size(), 8);
        assert_eq!(usize::value_type().size(), mem::size_of::<usize>());
    }

    #[test]
    pub fn test_scalar_value_types() {
        assert_eq!(bool::value_type(), Boolean);
        assert_eq!(i8::value_type(), Integer(IntSize::U1));
        assert_eq!(i16::value_type(), Integer(IntSize::U2));
        assert_eq!(i32::value_type(), Integer(IntSize::U4));
        assert_eq!(i64::value_type(), Integer(IntSize::U8));
        assert_eq!(u8::value_type(), Unsigned(IntSize::U1));
        assert_eq!(u16::value_type(), Unsigned(IntSize::U2));
        assert_eq!(u32::value_type(), Unsigned(IntSize::U4));
        assert_eq!(u64::value_type(), Unsigned(IntSize::U8));
        assert_eq!(f32::value_type(), Float(FloatSize::U4));
        assert_eq!(f64::value_type(), Float(FloatSize::U8));
    }

    #[test]
    #[cfg(target_pointer_width = "32")]
    pub fn test_ptr_sized_ints() {
        assert_eq!(isize::value_type(), Integer(IntSize::U4));
        assert_eq!(usize::value_type(), Unsigned(IntSize::U4));
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    pub fn test_ptr_sized_ints() {
        assert_eq!(isize::value_type(), Integer(IntSize::U8));
        assert_eq!(usize::value_type(), Unsigned(IntSize::U8));
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
        assert_eq!(T::value_type(), FixedArray(Box::new(Unsigned(IntSize::U4)), 256));
        type S = [T; 4];
        assert_eq!(S::value_type(), FixedArray(Box::new(T::value_type()), 4));
    }

    #[test]
    pub fn test_enum() {
        h5def!(#[repr(i64)] enum Foo { A = 1, B = -2 });
        assert_eq!(Foo::value_type(), Enum(EnumType {
            size: IntSize::U8,
            signed: true,
            members: vec![
                EnumMember { name: "A".into(), value: 1 },
                EnumMember { name: "B".into(), value: -2i64 as u64 },
            ]
        }));
        assert_eq!(Foo::value_type().size(), 8);

        h5def!(#[repr(u8)] #[derive(Debug)] pub enum Bar { A = 1, B = 2, });
        assert_eq!(Bar::value_type(), Enum(EnumType {
            size: IntSize::U1,
            signed: false,
            members: vec![
                EnumMember { name: "A".into(), value: 1 },
                EnumMember { name: "B".into(), value: 2 },
            ]
        }));
        assert_eq!(format!("{:?}", Bar::A), "A");
        assert_eq!(Bar::value_type().size(), 1);
    }
}
