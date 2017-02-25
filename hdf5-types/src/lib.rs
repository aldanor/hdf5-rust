#![recursion_limit = "1024"]

#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

extern crate ascii;
extern crate libc;

#[macro_use]
extern crate error_chain;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod array;
mod string;
mod h5type;

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
    TypeDescriptor, H5Type, IntSize, FloatSize,
    CompoundField, CompoundType, EnumMember, EnumType
};
pub use self::string::{FixedAscii, FixedUnicode, VarLenAscii, VarLenUnicode};
