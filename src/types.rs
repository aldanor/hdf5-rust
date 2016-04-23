use std::mem;

use libc::c_char;

use ffi::h5t::hvl_t;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntSize { U1 = 1, U2 = 2, U4 = 4, U8 = 8 }

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FloatSize { U4 = 4, U8 = 8 }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumMember {
    pub name: String,
    pub value: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Enum {
    pub size: IntSize,
    pub signed: bool,
    pub members: Vec<EnumMember>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundField {
    pub name: String,
    pub ty: ValueType,
    pub offset: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Compound {
    pub fields: Vec<CompoundField>,
    pub size: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValueType {
    Integer(IntSize),
    Unsigned(IntSize),
    Float(FloatSize),
    Boolean,
    Enum(Enum),
    Compound(Compound),
    FixedArray(Box<ValueType>, u32),
    FixedString(u32),
    VarLenArray(Box<ValueType>),
    VarLenString,
}

impl ValueType {
    pub fn size(&self) -> u32 {
        use self::ValueType::*;

        match *self {
            Integer(size) => size as u32,
            Unsigned(size) => size as u32,
            Float(size) => size as u32,
            Boolean => 1,
            Enum(ref enum_type) => enum_type.size as u32,
            Compound(ref compound) => compound.size,
            FixedArray(ref ty, len) => ty.size() * len,
            FixedString(len) => (mem::size_of::<c_char>() as u32) * (len + 1),
            VarLenArray(_) => mem::size_of::<hvl_t>() as u32,
            VarLenString => mem::size_of::<*const c_char> as u32,
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

#[cfg(test)]
pub mod tests {
    use std::mem;

    use super::{ToValueType, IntSize, FloatSize};
    use super::ValueType::*;

    #[test]
    fn test_value_type_size() {
        assert_eq!(bool::value_type().size(), 1);
        assert_eq!(i16::value_type().size(), 2);
        assert_eq!(u32::value_type().size(), 4);
        assert_eq!(f64::value_type().size(), 8);
        assert_eq!(usize::value_type().size(), mem::size_of::<usize>() as u32);
    }

    #[test]
    fn test_scalar_value_types() {
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
    fn test_ptr_sized_ints() {
        assert_eq!(isize::value_type(), Integer(IntSize::U4));
        assert_eq!(usize::value_type(), Unsigned(IntSize::U4));
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn test_ptr_sized_ints() {
        assert_eq!(isize::value_type(), Integer(IntSize::U8));
        assert_eq!(usize::value_type(), Unsigned(IntSize::U8));
    }
}
