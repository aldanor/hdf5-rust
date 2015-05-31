extern crate libc;
extern crate num;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
extern crate tempdir;

#[cfg(test)]
extern crate regex;

#[macro_use]
pub mod macros;

#[allow(non_camel_case_types, non_snake_case, raw_pointer_derive, dead_code)]
pub mod ffi;

pub mod error;
pub mod file;
pub mod object;
pub mod plist;
pub mod sync;

#[cfg(test)]
pub mod test;
