#![recursion_limit = "1024"]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::missing_safety_doc))]

//! Types that can be stored and retrieved from a `HDF5` dataset
//!
//! Crate features:
//! * `const_generics`: Uses const generics to enable arrays [T; N] for all N.
//!                     Compiling without this limits arrays to certain prespecified
//!                     sizes

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod array;
pub mod dyn_value;
mod h5type;
mod string;

pub use self::array::{Array, VarLenArray};
pub use self::dyn_value::{DynValue, OwnedDynValue};
pub use self::h5type::{
    CompoundField, CompoundType, EnumMember, EnumType, FloatSize, H5Type, IntSize, TypeDescriptor,
};
pub use self::string::{FixedAscii, FixedUnicode, StringError, VarLenAscii, VarLenUnicode};
