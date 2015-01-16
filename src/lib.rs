#[macro_use]
extern crate lazy_static;

#[allow(unstable)]
extern crate libc;

pub mod error;

#[allow(non_camel_case_types, raw_pointer_derive)]
pub mod ffi;
