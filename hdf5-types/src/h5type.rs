use std::fmt::{self, Display};
use std::mem;
use std::os::raw::c_void;
use std::ptr;

use crate::array::{Array, VarLenArray};
use crate::string::{FixedAscii, FixedUnicode, VarLenAscii, VarLenUnicode};

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct hvl_t {
    pub len: usize,
    pub ptr: *mut c_void,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntSize {
    U1 = 1,
    U2 = 2,
    U4 = 4,
    U8 = 8,
}

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
pub enum FloatSize {
    U4 = 4,
    U8 = 8,
}

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
    pub index: usize,
}

impl CompoundField {
    pub fn new(name: &str, ty: TypeDescriptor, offset: usize, index: usize) -> Self {
        Self { name: name.to_owned(), ty, offset, index }
    }

    pub fn typed<T: H5Type>(name: &str, offset: usize, index: usize) -> Self {
        Self::new(name, T::type_descriptor(), offset, index)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundType {
    pub fields: Vec<CompoundField>,
    pub size: usize,
}

impl CompoundType {
    pub fn to_c_repr(&self) -> CompoundType {
        let mut layout = self.clone();
        layout.fields.sort_by_key(|f| f.index);
        let mut offset = 0;
        let mut max_align = 1;
        for f in layout.fields.iter_mut() {
            f.ty = f.ty.to_c_repr();
            let align = f.ty.c_alignment();
            while offset % align != 0 {
                offset += 1;
            }
            f.offset = offset;
            max_align = max_align.max(align);
            offset += f.ty.size();
            layout.size = offset;
            while layout.size % max_align != 0 {
                layout.size += 1;
            }
        }
        layout
    }

    pub fn to_packed_repr(&self) -> CompoundType {
        let mut layout = self.clone();
        layout.fields.sort_by_key(|f| f.index);
        layout.size = 0;
        for f in layout.fields.iter_mut() {
            f.ty = f.ty.to_packed_repr();
            f.offset = layout.size;
            layout.size += f.ty.size();
        }
        layout
    }
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

impl Display for TypeDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TypeDescriptor::Integer(IntSize::U1) => write!(f, "int8"),
            TypeDescriptor::Integer(IntSize::U2) => write!(f, "int16"),
            TypeDescriptor::Integer(IntSize::U4) => write!(f, "int32"),
            TypeDescriptor::Integer(IntSize::U8) => write!(f, "int64"),
            TypeDescriptor::Unsigned(IntSize::U1) => write!(f, "uint8"),
            TypeDescriptor::Unsigned(IntSize::U2) => write!(f, "uint16"),
            TypeDescriptor::Unsigned(IntSize::U4) => write!(f, "uint32"),
            TypeDescriptor::Unsigned(IntSize::U8) => write!(f, "uint64"),
            TypeDescriptor::Float(FloatSize::U4) => write!(f, "float32"),
            TypeDescriptor::Float(FloatSize::U8) => write!(f, "float64"),
            TypeDescriptor::Boolean => write!(f, "bool"),
            TypeDescriptor::Enum(ref tp) => write!(f, "enum ({})", tp.base_type()),
            TypeDescriptor::Compound(ref tp) => write!(f, "compound ({} fields)", tp.fields.len()),
            TypeDescriptor::FixedArray(ref tp, n) => write!(f, "[{}; {}]", tp, n),
            TypeDescriptor::FixedAscii(n) => write!(f, "string (len {})", n),
            TypeDescriptor::FixedUnicode(n) => write!(f, "unicode (len {})", n),
            TypeDescriptor::VarLenArray(ref tp) => write!(f, "[{}] (var len)", tp),
            TypeDescriptor::VarLenAscii => write!(f, "string (var len)"),
            TypeDescriptor::VarLenUnicode => write!(f, "unicode (var len)"),
        }
    }
}

impl TypeDescriptor {
    pub fn size(&self) -> usize {
        use self::TypeDescriptor::*;

        match *self {
            Integer(size) | Unsigned(size) => size as _,
            Float(size) => size as _,
            Boolean => 1,
            Enum(ref enum_type) => enum_type.size as _,
            Compound(ref compound) => compound.size,
            FixedArray(ref ty, len) => ty.size() * len,
            FixedAscii(len) | FixedUnicode(len) => len,
            VarLenArray(_) => mem::size_of::<hvl_t>(),
            VarLenAscii | VarLenUnicode => mem::size_of::<*const u8>(),
        }
    }

    fn c_alignment(&self) -> usize {
        use self::TypeDescriptor::*;

        match *self {
            Compound(ref compound) => {
                compound.fields.iter().map(|f| f.ty.c_alignment()).max().unwrap_or(1)
            }
            FixedArray(ref ty, _) => ty.c_alignment(),
            FixedAscii(_) | FixedUnicode(_) => 1,
            VarLenArray(_) => mem::size_of::<usize>(),
            _ => self.size(),
        }
    }

    pub fn to_c_repr(&self) -> Self {
        use self::TypeDescriptor::*;

        match *self {
            Compound(ref compound) => Compound(compound.to_c_repr()),
            FixedArray(ref ty, size) => FixedArray(Box::new(ty.to_c_repr()), size),
            VarLenArray(ref ty) => VarLenArray(Box::new(ty.to_c_repr())),
            _ => self.clone(),
        }
    }

    pub fn to_packed_repr(&self) -> Self {
        use self::TypeDescriptor::*;

        match *self {
            Compound(ref compound) => Compound(compound.to_packed_repr()),
            FixedArray(ref ty, size) => FixedArray(Box::new(ty.to_packed_repr()), size),
            VarLenArray(ref ty) => VarLenArray(Box::new(ty.to_packed_repr())),
            _ => self.clone(),
        }
    }
}

pub unsafe trait H5Type: 'static {
    fn type_descriptor() -> TypeDescriptor;
}

macro_rules! impl_h5type {
    ($ty:ty, $variant:ident, $size:expr) => {
        unsafe impl H5Type for $ty {
            #[inline]
            fn type_descriptor() -> TypeDescriptor {
                $crate::h5type::TypeDescriptor::$variant($size)
            }
        }
    };
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
            offset: f as *const _ as _,
            index,
        });
        impl_tuple!(@parse_fields [$($s)*] $origin $fields | $($tt),*);
    );

    (@parse_fields [$($s:ident)*] $origin:ident $fields:ident |) => ();

    ($t:ident) => (
        unsafe impl<$t> H5Type for ($t,) where $t: H5Type {
            #[inline]
            fn type_descriptor() -> TypeDescriptor {
                let size = mem::size_of::<($t,)>();
                assert_eq!(size, mem::size_of::<$t>());
                TypeDescriptor::Compound(CompoundType {
                    fields: vec![CompoundField::typed::<$t>("0", 0, 0)],
                    size,
                })
            }
        }
    );

    ($t:ident, $($tt:ident),*) => (
        #[allow(dead_code, unused_variables)]
        unsafe impl<$t, $($tt),*> H5Type for ($t, $($tt),*)
            where $t: H5Type, $($tt: H5Type),*
        {
            fn type_descriptor() -> TypeDescriptor {
                let origin: *const Self = ptr::null();
                let mut fields = Vec::new();
                impl_tuple!(@parse_fields [] origin fields | $t, $($tt),*);
                let size = mem::size_of::<Self>();
                fields.sort_by_key(|f| f.offset);
                TypeDescriptor::Compound(CompoundType { fields, size })
            }
        }

        impl_tuple!($($tt),*);
    );
}

impl_tuple! { A, B, C, D, E, F, G, H, I, J, K, L }

unsafe impl<T: Array<Item = I>, I: H5Type> H5Type for T {
    #[inline]
    fn type_descriptor() -> TypeDescriptor {
        TypeDescriptor::FixedArray(
            Box::new(<I as H5Type>::type_descriptor()),
            <T as Array>::capacity(),
        )
    }
}

unsafe impl<T: Copy + H5Type> H5Type for VarLenArray<T> {
    #[inline]
    fn type_descriptor() -> TypeDescriptor {
        TypeDescriptor::VarLenArray(Box::new(<T as H5Type>::type_descriptor()))
    }
}

unsafe impl<A: Array<Item = u8>> H5Type for FixedAscii<A> {
    #[inline]
    fn type_descriptor() -> TypeDescriptor {
        TypeDescriptor::FixedAscii(A::capacity())
    }
}

unsafe impl<A: Array<Item = u8>> H5Type for FixedUnicode<A> {
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
    use super::TypeDescriptor as TD;
    use super::{hvl_t, CompoundField, CompoundType, FloatSize, H5Type, IntSize};
    use crate::array::VarLenArray;
    use crate::string::{FixedAscii, FixedUnicode, VarLenAscii, VarLenUnicode};
    use std::mem;

    #[test]
    pub fn test_scalar_types() {
        assert_eq!(bool::type_descriptor(), TD::Boolean);
        assert_eq!(i8::type_descriptor(), TD::Integer(IntSize::U1));
        assert_eq!(i16::type_descriptor(), TD::Integer(IntSize::U2));
        assert_eq!(i32::type_descriptor(), TD::Integer(IntSize::U4));
        assert_eq!(i64::type_descriptor(), TD::Integer(IntSize::U8));
        assert_eq!(u8::type_descriptor(), TD::Unsigned(IntSize::U1));
        assert_eq!(u16::type_descriptor(), TD::Unsigned(IntSize::U2));
        assert_eq!(u32::type_descriptor(), TD::Unsigned(IntSize::U4));
        assert_eq!(u64::type_descriptor(), TD::Unsigned(IntSize::U8));
        assert_eq!(f32::type_descriptor(), TD::Float(FloatSize::U4));
        assert_eq!(f64::type_descriptor(), TD::Float(FloatSize::U8));

        assert_eq!(bool::type_descriptor().size(), 1);
        assert_eq!(i16::type_descriptor().size(), 2);
        assert_eq!(u32::type_descriptor().size(), 4);
        assert_eq!(f64::type_descriptor().size(), 8);
    }

    #[test]
    #[cfg(target_pointer_width = "32")]
    pub fn test_ptr_sized_ints() {
        assert_eq!(isize::type_descriptor(), TD::Integer(IntSize::U4));
        assert_eq!(usize::type_descriptor(), TD::Unsigned(IntSize::U4));

        assert_eq!(usize::type_descriptor().size(), 4);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    pub fn test_ptr_sized_ints() {
        assert_eq!(isize::type_descriptor(), TD::Integer(IntSize::U8));
        assert_eq!(usize::type_descriptor(), TD::Unsigned(IntSize::U8));

        assert_eq!(usize::type_descriptor().size(), 8);
    }

    #[test]
    pub fn test_fixed_array() {
        type S = [T; 4];
        type T = [u32; 256];
        assert_eq!(T::type_descriptor(), TD::FixedArray(Box::new(TD::Unsigned(IntSize::U4)), 256));
        assert_eq!(S::type_descriptor(), TD::FixedArray(Box::new(T::type_descriptor()), 4));
    }

    #[test]
    pub fn test_varlen_array() {
        type S = VarLenArray<u16>;
        assert_eq!(S::type_descriptor(), TD::VarLenArray(Box::new(u16::type_descriptor())));
        assert_eq!(mem::size_of::<VarLenArray<u8>>(), mem::size_of::<hvl_t>());
    }

    #[test]
    pub fn test_string_types() {
        type FA = FixedAscii<[u8; 16]>;
        type FU = FixedUnicode<[u8; 32]>;
        assert_eq!(FA::type_descriptor(), TD::FixedAscii(16));
        assert_eq!(FU::type_descriptor(), TD::FixedUnicode(32));
        assert_eq!(VarLenAscii::type_descriptor(), TD::VarLenAscii);
        assert_eq!(VarLenUnicode::type_descriptor(), TD::VarLenUnicode);
    }

    #[test]
    pub fn test_tuples() {
        type T1 = (u16,);
        let td = T1::type_descriptor();
        assert_eq!(
            td,
            TD::Compound(CompoundType {
                fields: vec![CompoundField::typed::<u16>("0", 0, 0),],
                size: 2,
            })
        );
        assert_eq!(td.size(), 2);
        assert_eq!(mem::size_of::<T1>(), 2);

        type T2 = (i32, f32, (u64,));
        let td = T2::type_descriptor();
        assert_eq!(
            td,
            TD::Compound(CompoundType {
                fields: vec![
                    CompoundField::typed::<i32>("0", 0, 0),
                    CompoundField::typed::<f32>("1", 4, 1),
                    CompoundField::new(
                        "2",
                        TD::Compound(CompoundType {
                            fields: vec![CompoundField::typed::<u64>("0", 0, 0),],
                            size: 8,
                        }),
                        8,
                        2
                    ),
                ],
                size: 16,
            })
        );
        assert_eq!(td.size(), 16);
        assert_eq!(mem::size_of::<T2>(), 16);
    }

    #[test]
    pub fn test_tuple_various_reprs() {
        type T = (i8, u64, f32, bool);
        assert_eq!(mem::size_of::<T>(), 16);

        let td = T::type_descriptor();
        assert_eq!(
            td,
            TD::Compound(CompoundType {
                fields: vec![
                    CompoundField::typed::<u64>("1", 0, 1),
                    CompoundField::typed::<f32>("2", 8, 2),
                    CompoundField::typed::<i8>("0", 12, 0),
                    CompoundField::typed::<bool>("3", 13, 3),
                ],
                size: 16,
            })
        );
        assert_eq!(td.size(), 16);

        let td = T::type_descriptor().to_c_repr();
        assert_eq!(
            td,
            TD::Compound(CompoundType {
                fields: vec![
                    CompoundField::typed::<i8>("0", 0, 0),
                    CompoundField::typed::<u64>("1", 8, 1),
                    CompoundField::typed::<f32>("2", 16, 2),
                    CompoundField::typed::<bool>("3", 20, 3),
                ],
                size: 24,
            })
        );
        assert_eq!(td.size(), 24);

        let td = T::type_descriptor().to_packed_repr();
        assert_eq!(
            td,
            TD::Compound(CompoundType {
                fields: vec![
                    CompoundField::typed::<i8>("0", 0, 0),
                    CompoundField::typed::<u64>("1", 1, 1),
                    CompoundField::typed::<f32>("2", 9, 2),
                    CompoundField::typed::<bool>("3", 13, 3),
                ],
                size: 14,
            })
        );
        assert_eq!(td.size(), 14);
    }
}
