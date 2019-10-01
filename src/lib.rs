#![cfg_attr(feature = "cargo-clippy", warn(clippy::pedantic))]
#![cfg_attr(feature = "cargo-clippy", warn(clippy::nursery))]
#![cfg_attr(feature = "cargo-clippy", warn(clippy::all))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::identity_op))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::erasing_op))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::cast_sign_loss))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::module_name_repetitions))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::cast_possible_truncation))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::cast_possible_wrap))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::cast_precision_loss))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::similar_names))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::missing_const_for_fn))]
#![cfg_attr(all(feature = "cargo-clippy", test), allow(clippy::cyclomatic_complexity))]
#![cfg_attr(not(test), allow(dead_code))]

#[cfg(all(feature = "mpio", not(h5_have_parallel)))]
compile_error!("Enabling \"mpio\" feature requires HDF5 library built with MPI support");

mod export {
    pub use crate::{
        class::from_id,
        dim::{Dimension, Ix},
        error::{silence_errors, Error, Result},
        filters::Filters,
        hl::{
            Container, Conversion, Dataset, DatasetBuilder, Dataspace, Datatype, File, FileBuilder,
            Group, Location, Object, PropertyList, Reader, Writer,
        },
    };

    #[doc(hidden)]
    pub use crate::error::h5check;

    pub use hdf5_derive::H5Type;
    pub use hdf5_types::H5Type;

    pub mod types {
        pub use hdf5_types::*;
    }

    pub mod dataset {
        pub use crate::hl::dataset::{Chunk, Dataset, DatasetBuilder};
        pub use crate::hl::plist::dataset_access::*;
    }

    pub mod file {
        pub use crate::hl::file::{File, FileBuilder, OpenMode};
        pub use crate::hl::plist::file_access::*;
        pub use crate::hl::plist::file_create::*;
    }

    pub mod plist {
        pub use crate::hl::plist::dataset_access::DatasetAccess;
        pub use crate::hl::plist::file_access::FileAccess;
        pub use crate::hl::plist::file_create::FileCreate;
        pub use crate::hl::plist::{PropertyList, PropertyListClass};

        pub mod dataset_access {
            pub use crate::hl::plist::dataset_access::*;
        }
        pub mod file_access {
            pub use crate::hl::plist::file_access::*;
        }
        pub mod file_create {
            pub use crate::hl::plist::file_create::*;
        }
    }
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
#[doc(hidden)]
pub mod sync;
mod util;

mod hl;

mod internal_prelude {
    pub use libc::size_t;
    pub use std::os::raw::{c_char, c_double, c_int, c_long, c_uint, c_void};

    pub use hdf5_sys::{
        h5::{haddr_t, hbool_t, herr_t, hsize_t},
        h5i::H5I_type_t::{self, *},
        h5i::{hid_t, H5I_INVALID_HID},
        h5p::H5P_DEFAULT,
        h5s::{H5S_ALL, H5S_UNLIMITED},
    };

    pub use crate::{
        class::ObjectClass,
        dim::Dimension,
        error::{h5check, silence_errors},
        export::*,
        handle::{get_id_type, is_valid_user_id, Handle},
        hl::plist::PropertyListClass,
        sync::sync,
        util::{
            get_h5_str, string_from_cstr, string_from_fixed_bytes, string_to_fixed_bytes,
            to_cstring,
        },
    };

    #[cfg(test)]
    pub use crate::test::{with_tmp_dir, with_tmp_file, with_tmp_path};
}

#[cfg(test)]
pub mod test;

/// Returns the runtime version of the HDF5 library.
pub fn library_version() -> (u8, u8, u8) {
    use self::internal_prelude::c_uint;
    use hdf5_sys::h5::H5get_libversion;
    let mut v: (c_uint, c_uint, c_uint) = (0, 0, 0);
    h5call!(H5get_libversion(&mut v.0, &mut v.1, &mut v.2))
        .map(|_| (v.0 as _, v.1 as _, v.2 as _))
        .unwrap_or((0, 0, 0))
}

/// Returns true if the HDF5 library is threadsafe.
pub fn is_library_threadsafe() -> bool {
    #[cfg(hdf5_1_8_16)]
    {
        use self::internal_prelude::hbool_t;
        use hdf5_sys::h5::H5is_library_threadsafe;
        let mut ts: hbool_t = 0;
        h5call!(H5is_library_threadsafe(&mut ts)).map(|_| ts > 0).unwrap_or(false)
    }
    #[cfg(not(hdf5_1_8_16))]
    {
        cfg!(h5_have_threadsafe)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::library_version;

    #[test]
    pub fn test_library_version() {
        assert!(library_version() >= (1, 8, 4));
    }
}
