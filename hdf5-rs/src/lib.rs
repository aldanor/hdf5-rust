#![cfg_attr(feature = "cargo-clippy", allow(clippy::block_in_if_condition_stmt))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::needless_return))]
#![cfg_attr(all(feature = "cargo-clippy", test), allow(clippy::cyclomatic_complexity))]
#![cfg_attr(not(test), allow(dead_code))]

mod export {
    pub use crate::{
        container::Container,
        dataset::Dataset,
        datatype::Datatype,
        error::{Error, Result},
        file::File,
        filters::Filters,
        group::Group,
        location::Location,
        object::Object,
        space::{Dataspace, Dimension, Ix},
    };
}

pub use crate::export::*;

pub use hdf5_derive::H5Type;
pub use hdf5_types::{self as types, H5Type};

#[macro_use]
mod macros;
#[macro_use]
mod class;

mod container;
mod dataset;
mod datatype;
mod error;
mod file;
mod filters;
mod globals;
mod group;
mod handle;
mod location;
mod object;
mod plist;
mod space;
mod sync;
mod util;

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

    pub use super::{Container, Dimension, H5Type, Location, Object};
}

mod internal_prelude {
    pub use libc::{c_char, c_int, c_uint, c_void, size_t};

    pub use libhdf5_sys::{
        h5::{haddr_t, hbool_t, herr_t, hsize_t},
        h5i::H5I_type_t::{self, *},
        h5i::{hid_t, H5I_INVALID_HID},
        h5p::H5P_DEFAULT,
    };

    pub use crate::{
        class::ObjectClass,
        dataset::DatasetBuilder,
        error::silence_errors,
        export::*,
        file::FileBuilder,
        handle::{get_id_type, is_valid_user_id, Handle},
        plist::PropertyList,
        types::H5Type,
        util::{get_h5_str, string_from_cstr, to_cstring},
    };

    #[cfg(test)]
    pub use crate::test::{with_tmp_dir, with_tmp_file, with_tmp_path};
}

#[cfg(test)]
pub mod test;

/// Returns the version of the HDF5 library that the crate was compiled against.
pub fn hdf5_version() -> (u8, u8, u8) {
    h5lock!(libhdf5_lib::hdf5_version()).unwrap_or((0, 0, 0))
}

#[cfg(test)]
pub mod tests {
    use super::hdf5_version;

    #[test]
    pub fn test_hdf5_version() {
        assert!(hdf5_version() >= (1, 8, 0));
    }
}
