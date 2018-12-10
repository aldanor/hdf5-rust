#![recursion_limit = "1024"]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::transmute_bytes_to_str))]

#[macro_use]
extern crate error_chain;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod array;
mod h5type;
mod string;

mod errors {
    error_chain! {
        foreign_links {
            Ascii(::ascii::AsAsciiStrError);
        }
    }
}

pub use self::array::{Array, VarLenArray};
pub use self::errors::Error;
pub use self::h5type::{
    CompoundField, CompoundType, EnumMember, EnumType, FloatSize, H5Type, IntSize, TypeDescriptor,
};
pub use self::string::{FixedAscii, FixedUnicode, VarLenAscii, VarLenUnicode};
