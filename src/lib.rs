extern crate libc;
extern crate num;

#[macro_use]
extern crate lazy_static;

#[macro_use]
pub mod macros;

#[allow(non_camel_case_types, non_snake_case, raw_pointer_derive, dead_code)]
pub mod ffi;

pub mod error;
pub mod object;
pub mod sync;
pub mod plist;
