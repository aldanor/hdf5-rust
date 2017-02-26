use std::mem;

use libc::{size_t, c_void};

use array::{Array, VarLenArray};
use string::{FixedAscii, FixedUnicode, VarLenAscii, VarLenUnicode};

#[repr(C)]
struct hvl_t {
    len: size_t,
    p: *mut c_void,
}

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
    #[inline]
    pub fn base_type(&self) -> TypeDescriptor {
        if self.signed {
            TypeDescriptor::Integer(self.size)
        } else {
            TypeDescriptor::Unsigned(self.size)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundField {
    pub name: String,
    pub ty: TypeDescriptor,
    pub offset: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundType {
    pub fields: Vec<CompoundField>,
    pub size: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeDescriptor {
    Integer(IntSize),
    Unsigned(IntSize),
    Float(FloatSize),
    Boolean,
    Enum(EnumType),
    Compound(CompoundType),
    FixedArray(Box<TypeDescriptor>, usize),
    FixedAscii(usize),
    FixedUnicode(usize),
    VarLenArray(Box<TypeDescriptor>),
    VarLenAscii,
    VarLenUnicode,
}

impl TypeDescriptor {
    pub fn size(&self) -> usize {
        use self::TypeDescriptor::*;

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
    fn type_descriptor() -> TypeDescriptor;
}

macro_rules! impl_h5type {
    ($ty:ty, $variant:ident, $size:expr) => (
        unsafe impl H5Type for $ty {
            #[inline]
            fn type_descriptor() -> TypeDescriptor {
                $crate::h5type::TypeDescriptor::$variant($size)
            }
        }
    )
}

impl_h5type!(i8, Integer, IntSize::U1);
impl_h5type!(i16, Integer, IntSize::U2);
impl_h5type!(i32, Integer, IntSize::U4);
impl_h5type!(i64, Integer, IntSize::U8);
impl_h5type!(u8, Unsigned, IntSize::U1);
impl_h5type!(u16, Unsigned, IntSize::U2);
impl_h5type!(u32, Unsigned, IntSize::U4);
impl_h5type!(u64, Unsigned, IntSize::U8);
impl_h5type!(f32, Float, FloatSize::U4);
impl_h5type!(f64, Float, FloatSize::U8);

#[cfg(target_pointer_width = "32")]
impl_h5type!(isize, Integer, IntSize::U4);
#[cfg(target_pointer_width = "32")]
impl_h5type!(usize, Unsigned, IntSize::U4);

#[cfg(target_pointer_width = "64")]
impl_h5type!(isize, Integer, IntSize::U8);
#[cfg(target_pointer_width = "64")]
impl_h5type!(usize, Unsigned, IntSize::U8);

unsafe impl H5Type for bool {
    #[inline]
    fn type_descriptor() -> TypeDescriptor {
        TypeDescriptor::Boolean
    }
}

macro_rules! impl_tuple {
    (@second $a:tt $b:tt) => ($b);

    (@parse_fields [$($s:ident)*] $origin:ident $fields:ident | $t:ty $(,$tt:ty)*) => (
        let &$($s)*(.., ref f, $(impl_tuple!(@second $tt _),)*) = unsafe { &*$origin };
        let index = $fields.len();
        $fields.push(CompoundField {
            name: format!("{}", index),
            ty: <$t as H5Type>::type_descriptor(),
            offset: f as *const _ as usize
        });
        impl_tuple!(@parse_fields [$($s)*] $origin $fields | $($tt),*);
    );

    (@parse_fields [$($s:ident)*] $origin:ident $fields:ident |) => ();

    ($t:ident) => (
        unsafe impl<$t> H5Type for ($t,) where $t: H5Type {
            #[inline]
            fn type_descriptor() -> TypeDescriptor {
                assert!(mem::size_of::<$t>() == mem::size_of::<($t,)>());
                <$t as H5Type>::type_descriptor()
            }
        }
    );

    ($t:ident, $($tt:ident),*) => (
        unsafe impl<$t, $($tt),*> H5Type for ($t, $($tt),*)
            where $t: H5Type, $($tt: H5Type),*
        {
            #[allow(dead_code, unused_variables)]
            fn type_descriptor() -> TypeDescriptor {
                let origin = 0usize as *const ($t, $($tt),*);
                let mut fields = Vec::<CompoundField>::new();
                impl_tuple!(@parse_fields [] origin fields | $t, $($tt),*);
                TypeDescriptor::Compound(CompoundType {
                    fields: fields,
                    size: mem::size_of::<($t, $($tt),*)>(),
                })
            }
        }

        impl_tuple!($($tt),*);
    );
}

impl_tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15 }

unsafe impl<T: Array<Item=I>, I: H5Type> H5Type for T {
    #[inline]
    fn type_descriptor() -> TypeDescriptor {
        TypeDescriptor::FixedArray(
            Box::new(<I as H5Type>::type_descriptor()),
            <T as Array>::capacity()
        )
    }
}

unsafe impl<T: Copy + H5Type> H5Type for VarLenArray<T> {
    #[inline]
    fn type_descriptor() -> TypeDescriptor {
        TypeDescriptor::VarLenArray(Box::new(<T as H5Type>::type_descriptor()))
    }
}

unsafe impl<A: Array<Item=u8>> H5Type for FixedAscii<A> {
    #[inline]
    fn type_descriptor() -> TypeDescriptor {
        TypeDescriptor::FixedAscii(A::capacity())
    }
}

unsafe impl<A: Array<Item=u8>> H5Type for FixedUnicode<A> {
    #[inline]
    fn type_descriptor() -> TypeDescriptor {
        TypeDescriptor::FixedUnicode(A::capacity())
    }
}

unsafe impl H5Type for VarLenAscii {
    #[inline]
    fn type_descriptor() -> TypeDescriptor {
        TypeDescriptor::VarLenAscii
    }
}

unsafe impl H5Type for VarLenUnicode {
    #[inline]
    fn type_descriptor() -> TypeDescriptor {
        TypeDescriptor::VarLenUnicode
    }
}

#[cfg(test)]
pub mod tests {
    use super::TypeDescriptor as VT;
    use super::{IntSize, FloatSize, H5Type, hvl_t};
    use array::VarLenArray;
    use string::{FixedAscii, FixedUnicode, VarLenAscii, VarLenUnicode};
    use std::mem;

    #[test]
    pub fn test_scalar_types() {
        assert_eq!(bool::type_descriptor(), VT::Boolean);
        assert_eq!(i8::type_descriptor(), VT::Integer(IntSize::U1));
        assert_eq!(i16::type_descriptor(), VT::Integer(IntSize::U2));
        assert_eq!(i32::type_descriptor(), VT::Integer(IntSize::U4));
        assert_eq!(i64::type_descriptor(), VT::Integer(IntSize::U8));
        assert_eq!(u8::type_descriptor(), VT::Unsigned(IntSize::U1));
        assert_eq!(u16::type_descriptor(), VT::Unsigned(IntSize::U2));
        assert_eq!(u32::type_descriptor(), VT::Unsigned(IntSize::U4));
        assert_eq!(u64::type_descriptor(), VT::Unsigned(IntSize::U8));
        assert_eq!(f32::type_descriptor(), VT::Float(FloatSize::U4));
        assert_eq!(f64::type_descriptor(), VT::Float(FloatSize::U8));

        assert_eq!(bool::type_descriptor().size(), 1);
        assert_eq!(i16::type_descriptor().size(), 2);
        assert_eq!(u32::type_descriptor().size(), 4);
        assert_eq!(f64::type_descriptor().size(), 8);
    }

    #[test]
    #[cfg(target_pointer_width = "32")]
    pub fn test_ptr_sized_ints() {
        assert_eq!(isize::type_descriptor(), VT::Integer(IntSize::U4));
        assert_eq!(usize::type_descriptor(), VT::Unsigned(IntSize::U4));

        assert_eq!(usize::type_descriptor().size(), 4);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    pub fn test_ptr_sized_ints() {
        assert_eq!(isize::type_descriptor(), VT::Integer(IntSize::U8));
        assert_eq!(usize::type_descriptor(), VT::Unsigned(IntSize::U8));

        assert_eq!(usize::type_descriptor().size(), 8);
    }

    #[test]
    pub fn test_fixed_array() {
        type S = [T; 4];
        type T = [u32; 256];
        assert_eq!(T::type_descriptor(),
                   VT::FixedArray(Box::new(VT::Unsigned(IntSize::U4)), 256));
        assert_eq!(S::type_descriptor(),
                   VT::FixedArray(Box::new(T::type_descriptor()), 4));
    }

    #[test]
    pub fn test_varlen_array() {
        type S = VarLenArray<u16>;
        assert_eq!(S::type_descriptor(),
                   VT::VarLenArray(Box::new(u16::type_descriptor())));
        assert_eq!(mem::size_of::<VarLenArray<u8>>(),
                   mem::size_of::<hvl_t>());
    }

    #[test]
    pub fn test_string_types() {
        type FA = FixedAscii<[u8; 16]>;
        type FU = FixedUnicode<[u8; 32]>;
        assert_eq!(FA::type_descriptor(), VT::FixedAscii(16));
        assert_eq!(FU::type_descriptor(), VT::FixedUnicode(32));
        assert_eq!(VarLenAscii::type_descriptor(), VT::VarLenAscii);
        assert_eq!(VarLenUnicode::type_descriptor(), VT::VarLenUnicode);
    }
}
