#![cfg_attr(feature = "cargo-clippy", allow(clippy::block_in_if_condition_stmt))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::needless_return))]
#![cfg_attr(all(feature = "cargo-clippy", test), allow(clippy::cyclomatic_complexity))]
#![cfg_attr(not(test), allow(dead_code))]

pub use crate::container::Container;
pub use crate::dataset::Dataset;
pub use crate::datatype::Datatype;
pub use crate::error::{Result, Error};
pub use crate::file::File;
pub use crate::filters::Filters;
pub use crate::group::Group;
pub use crate::location::Location;
pub use crate::object::Object;
pub use crate::space::{Dimension, Ix, Dataspace};

pub use crate::types::H5Type;

extern crate libc;
extern crate num;

extern crate libhdf5_lib as lib;
extern crate libhdf5_sys as ffi;
extern crate hdf5_types;
extern crate remutex;

#[cfg(test)]
#[macro_use]
extern crate hdf5_derive;

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

pub mod types {
    pub use hdf5_types::*;
}

pub mod prelude {
    //! The HDF5 prelude module.
    //!
    //! The purpose of this module is to provide reexports of many core `hdf5` traits so that
    //! they can be then glob-imported all at once:
    //!
    //! ```ignore
    //! use h5::prelude::*;
    //! ```
    //! This module provides reexports of such traits as `Object`, `Location` and `Container`
    //! and does not expose any structures or functions.

    pub use super::Object;
    pub use super::Location;
    pub use super::Container;
    pub use super::Dimension;
    pub use super::H5Type;
}

mod internal_prelude {
    pub use crate::container::Container;
    pub use crate::dataset::{Dataset, DatasetBuilder};
    pub use crate::datatype::Datatype;
    pub use crate::error::{Error, Result, silence_errors};
    pub use crate::file::{File, FileBuilder};
    pub use crate::filters::Filters;
    pub use crate::group::Group;
    pub use crate::handle::{Handle, ID, FromID, get_id_type};
    pub use crate::location::Location;
    pub use crate::object::Object;
    pub use crate::plist::PropertyList;
    pub use crate::space::{Dataspace, Dimension, Ix};
    pub use crate::types::H5Type;
    pub use crate::util::{to_cstring, string_from_cstr, get_h5_str};

    pub use libc::{c_int, c_uint, c_void, c_char, size_t};

    pub use crate::ffi::h5::{hsize_t, hbool_t, haddr_t, herr_t};
    pub use crate::ffi::h5i::{H5I_INVALID_HID, hid_t};
    pub use crate::ffi::h5p::H5P_DEFAULT;
    pub use crate::ffi::h5i::H5I_type_t::*;

    #[cfg(test)]
    pub use crate::test::{with_tmp_file, with_tmp_dir, with_tmp_path};
}

#[cfg(test)]
pub mod test;

/// Returns the version of the HDF5 library that the crate was compiled against.
pub fn hdf5_version() -> (u8, u8, u8) {
    h5lock!(lib::hdf5_version()).unwrap_or((0, 0, 0))
}

#[cfg(test)]
pub mod tests {
    use super::hdf5_version;

    #[test]
    pub fn test_hdf5_version() {
        assert!(hdf5_version() >= (1, 8, 0));
    }
}
