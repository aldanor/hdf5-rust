use std::mem;

use libc::c_char;

use ffi::h5t::hvl_t;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntSize { U1 = 1, U2 = 2, U4 = 4, U8 = 8 }

impl IntSize {
    pub fn from_int(size: usize) -> Option<IntSize> {
        if size == 1 {
            Some(IntSize::U1)
        } else if size == 2 {
            Some(IntSize::U2)
        } else if size == 4 {
            Some(IntSize::U4)
        } else if size == 8 {
            Some(IntSize::U8)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FloatSize { U4 = 4, U8 = 8 }

impl FloatSize {
    pub fn from_int(size: usize) -> Option<FloatSize> {
        if size == 4 {
            Some(FloatSize::U4)
        } else if size == 8 {
            Some(FloatSize::U8)
        } else {
            None
        }
    }
}

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

impl EnumType {
    pub fn base_type(&self) -> ValueType {
        if self.signed {
            ValueType::Integer(self.size)
        } else {
            ValueType::Unsigned(self.size)
        }
    }
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
            Integer(size) | Unsigned(size) => size as usize,
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
        type S = [T; 4];
        type T = [u32; 256];
        assert_eq!(T::value_type(), VT::FixedArray(Box::new(VT::Unsigned(IntSize::U4)), 256));
        assert_eq!(S::value_type(), VT::FixedArray(Box::new(T::value_type()), 4));
    }
}
