#[macro_use]
extern crate lazy_static;

#[allow(unstable)]
extern crate libc;

pub mod error;

#[allow(non_camel_case_types, non_snake_case, raw_pointer_derive, dead_code)]
pub mod ffi;
