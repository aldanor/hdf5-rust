use std::mem;

use ffi::h5t::hvl_t;

use types::{Array, VarLenArray, FixedAscii, FixedUnicode, VarLenAscii, VarLenUnicode};

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
    FixedAscii(usize),
    FixedUnicode(usize),
    VarLenArray(Box<ValueType>),
    VarLenAscii,
    VarLenUnicode,
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
            FixedAscii(len) | FixedUnicode(len) => len,
            VarLenArray(_) => mem::size_of::<hvl_t>(),
            VarLenAscii | VarLenUnicode => mem::size_of::<*const u8>(),
        }
    }
}

pub unsafe trait H5Type {
    fn value_type() -> ValueType;
}

macro_rules! impl_value_type {
    ($ty:ty, $variant:ident, $size:expr) => (
        unsafe impl H5Type for $ty {
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

unsafe impl H5Type for bool {
    fn value_type() -> ValueType {
        ValueType::Boolean
    }
}

unsafe impl<T: Array<Item=I>, I: H5Type> H5Type for T {
    fn value_type() -> ValueType {
        ValueType::FixedArray(
            Box::new(<I as H5Type>::value_type()),
            <T as Array>::capacity()
        )
    }
}

unsafe impl<T: Copy + H5Type> H5Type for VarLenArray<T> {
    fn value_type() -> ValueType {
        ValueType::VarLenArray(Box::new(<T as H5Type>::value_type()))
    }
}

unsafe impl<A: Array<Item=u8>> H5Type for FixedAscii<A> {
    fn value_type() -> ValueType {
        ValueType::FixedAscii(A::capacity())
    }
}

unsafe impl<A: Array<Item=u8>> H5Type for FixedUnicode<A> {
    fn value_type() -> ValueType {
        ValueType::FixedUnicode(A::capacity())
    }
}

unsafe impl H5Type for VarLenAscii {
    fn value_type() -> ValueType {
        ValueType::VarLenAscii
    }
}

unsafe impl H5Type for VarLenUnicode {
    fn value_type() -> ValueType {
        ValueType::VarLenUnicode
    }
}

#[cfg(test)]
pub mod tests {
    use super::ValueType as VT;
    use super::{IntSize, FloatSize, H5Type};
    use types::{VarLenArray, FixedAscii, FixedUnicode, VarLenAscii, VarLenUnicode};
    use std::mem;
    use ffi::h5t::hvl_t;

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
    pub fn test_fixed_array() {
        type S = [T; 4];
        type T = [u32; 256];
        assert_eq!(T::value_type(),
                   VT::FixedArray(Box::new(VT::Unsigned(IntSize::U4)), 256));
        assert_eq!(S::value_type(),
                   VT::FixedArray(Box::new(T::value_type()), 4));
    }

    #[test]
    pub fn test_varlen_array() {
        type S = VarLenArray<u16>;
        assert_eq!(S::value_type(),
                   VT::VarLenArray(Box::new(u16::value_type())));
        assert_eq!(mem::size_of::<VarLenArray<u8>>(),
                   mem::size_of::<hvl_t>());
    }

    #[test]
    pub fn test_string_types() {
        type FA = FixedAscii<[u8; 16]>;
        type FU = FixedUnicode<[u8; 32]>;
        assert_eq!(FA::value_type(), VT::FixedAscii(16));
        assert_eq!(FU::value_type(), VT::FixedUnicode(32));
        assert_eq!(VarLenAscii::value_type(), VT::VarLenAscii);
        assert_eq!(VarLenUnicode::value_type(), VT::VarLenUnicode);
    }
}
