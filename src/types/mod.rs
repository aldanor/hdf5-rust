#[macro_use]
mod string_traits;

mod value_type;
mod fixed_string;
mod varlen_string;
mod varlen_array;

#[macro_use]
mod h5def;

pub use self::value_type::{
    ValueType, ToValueType, Array, IntSize, FloatSize,
    CompoundField, CompoundType, EnumMember, EnumType
};
pub use self::fixed_string::FixedString;
pub use self::varlen_string::VarLenString;
pub use self::varlen_array::VarLenArray;
