#![cfg_attr(feature = "cargo-clippy", allow(clippy::block_in_if_condition_stmt))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::needless_return))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::len_without_is_empty))]
#![cfg_attr(all(feature = "cargo-clippy", test), allow(clippy::cyclomatic_complexity))]
#![cfg_attr(not(test), allow(dead_code))]

mod export {
    pub use crate::{
        dim::{Dimension, Ix},
        error::{Error, Result},
        filters::Filters,
        hl::{Dataset, Dataspace, Datatype, File, Group, Location, Object, PropertyList},
    };

    pub use hdf5_derive::H5Type;
    pub use hdf5_types::{self as types, H5Type};
}

pub use crate::export::*;

#[macro_use]
mod macros;
#[macro_use]
mod class;

mod dim;
mod error;
mod filters;
mod globals;
mod handle;
mod sync;
mod util;

mod hl;

pub mod plist;

mod internal_prelude {
    pub use libc::size_t;
    pub use std::os::raw::{c_char, c_int, c_uint, c_void};

    pub use libhdf5_sys::{
        h5::{haddr_t, hbool_t, herr_t, hsize_t},
        h5i::H5I_type_t::{self, *},
        h5i::{hid_t, H5I_INVALID_HID},
        h5p::H5P_DEFAULT,
        h5s::{H5S_ALL, H5S_UNLIMITED},
    };

    pub use crate::{
        class::ObjectClass,
        dim::Dimension,
        error::{h5check, silence_errors, ResultExt},
        export::*,
        handle::{get_id_type, is_valid_user_id, Handle},
        hl::datatype::Conversion,
        hl::plist::PropertyListClass,
        hl::*,
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
