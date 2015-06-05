extern crate libc;
extern crate num;

#[allow(non_camel_case_types, non_snake_case, raw_pointer_derive, dead_code)]
extern crate libhdf5_sys as ffi;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
extern crate tempdir;

#[cfg(test)]
extern crate regex;

#[macro_use]
pub mod macros;

pub mod container;
pub mod error;
pub mod file;
pub mod globals;
pub mod group;
pub mod location;
pub mod object;
pub mod plist;
pub mod sync;
pub mod util;

#[cfg(test)]
pub mod test;
