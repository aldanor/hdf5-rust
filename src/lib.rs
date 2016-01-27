#![cfg_attr(not(test), allow(dead_code))]

pub use container::Container;
pub use dataset::Dataset;
pub use datatype::{Datatype, ToDatatype};
pub use error::{Result, Error};
pub use file::File;
pub use filters::Filters;
pub use group::Group;
pub use location::Location;
pub use object::Object;
pub use space::{Dimension, Ix, Dataspace};

extern crate libc;
extern crate num;

#[allow(non_camel_case_types, non_snake_case, dead_code)]
extern crate libhdf5_sys as ffi;

extern crate remutex;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
extern crate tempdir;

#[cfg(test)]
extern crate regex;

#[macro_use]
mod macros;

mod container;
mod dataset;
mod datatype;
mod error;
mod file;
mod filters;
mod group;
mod handle;
mod location;
mod object;
mod plist;
mod space;
mod sync;
mod util;

#[allow(dead_code)]
mod globals;

pub mod prelude;

#[cfg(test)]
pub mod test;
