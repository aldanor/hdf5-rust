#![cfg_attr(feature = "unstable", feature(plugin))]
#![cfg_attr(feature = "unstable", plugin(clippy))]
#![cfg_attr(feature = "unstable", allow(block_in_if_condition_stmt, needless_return))]
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

/// Returns the version of the HDF5 library that the crate was compiled against.
pub fn hdf5_version() -> (u8, u8, u8) {
    use ffi::h5;
    use libc::c_uint;

    let mut version: (c_uint, c_uint, c_uint) = (0, 0, 0);
    h5lock!({
        h5::H5open();
        h5::H5get_libversion(&mut version.0, &mut version.1, &mut version.2);
    });
    (version.0 as u8, version.1 as u8, version.2 as u8)
}

#[cfg(test)]
pub mod tests {
    use super::hdf5_version;

    #[test]
    pub fn test_hdf5_version() {
        let version = hdf5_version();
        assert!(version.0 >= 1 && version.1 >= 8);
    }
}
