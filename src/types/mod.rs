#[macro_use]
mod string_traits;

mod fixed_string;
mod value_type;

#[macro_use]
mod h5def;

pub use self::fixed_string::FixedString;
pub use self::value_type::{ValueType, ToValueType, Array};

#[cfg(feature = "varlen")]
mod varlen_string;
#[cfg(feature = "varlen")]
pub use self::varlen_string::VarLenString;
