extern crate libc;
extern crate num;

#[allow(non_camel_case_types, non_snake_case, raw_pointer_derive, dead_code)]
extern crate libhdf5_sys as ffi;

extern crate remutex;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
extern crate tempdir;

#[cfg(test)]
extern crate regex;

#[macro_use]
pub mod macros;

mod handle;

pub mod container;
pub mod datatype;
pub mod error;
pub mod file;
pub mod filters;
pub mod globals;
pub mod group;
pub mod location;
pub mod object;
pub mod plist;
pub mod space;
pub mod sync;
pub mod util;

pub mod prelude;

#[cfg(test)]
pub mod test;
