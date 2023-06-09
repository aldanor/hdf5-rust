//! HDF5 for Rust.
//!
//! This crate provides thread-safe Rust bindings and high-level wrappers for the `HDF5`
//! library API. Some of the features include:
//!
//! - Thread-safety with non-threadsafe libhdf5 builds guaranteed via reentrant mutexes.
//! - Native representation of most HDF5 types, including variable-length strings and arrays.
//! - Derive-macro for automatic mapping of user structs and enums to `HDF5` types.
//! - Multi-dimensional array reading/writing interface via `ndarray`.
//!
//! Direct low-level bindings are also available and provided in the `hdf5-sys` crate.
//!
//! Requires `HDF5` library of version 1.8.4 or later. Newer versions will enable additional
//! features of the library. Such items are marked in the documentation with a version number
//! indicating the required version of `HDF5`. The `have-direct` and `have-parallel` features
//! also indicates `HDF5` functionality.

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
#![cfg_attr(feature = "cargo-clippy", allow(clippy::missing_safety_doc))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::missing_errors_doc))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::cognitive_complexity))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::must_use_candidate))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::wildcard_imports))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::struct_excessive_bools))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::redundant_pub_crate))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::unnecessary_unwrap))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::unnecessary_wraps))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::upper_case_acronyms))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::missing_panics_doc))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::missing_const_for_fn))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::option_if_let_else))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::return_self_not_must_use))]
#![cfg_attr(all(feature = "cargo-clippy", test), allow(clippy::cyclomatic_complexity))]
#![cfg_attr(not(test), allow(dead_code))]
// To build docs locally:
// RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --features blosc,lzf
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(all(feature = "mpio", not(feature = "have-parallel")))]
compile_error!("Enabling \"mpio\" feature requires HDF5 library built with MPI support");

mod export {
    pub use crate::{
        class::from_id,
        dim::{Dimension, Ix},
        error::{silence_errors, Error, ErrorFrame, ErrorStack, ExpandedErrorStack, Result},
        hl::extents::{Extent, Extents, SimpleExtents},
        hl::selection::{Hyperslab, Selection, SliceOrIndex},
        hl::{
            Attribute, AttributeBuilder, AttributeBuilderData, AttributeBuilderEmpty,
            AttributeBuilderEmptyShape, ByteReader, Container, Conversion, Dataset, DatasetBuilder,
            DatasetBuilderData, DatasetBuilderEmpty, DatasetBuilderEmptyShape, Dataspace, Datatype,
            File, FileBuilder, Group, LinkInfo, LinkType, Location, LocationInfo, LocationToken,
            LocationType, Object, PropertyList, Reader, Writer,
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
        #[cfg(feature = "1.10.5")]
        pub use crate::hl::chunks::ChunkInfo;
        #[cfg(feature = "1.14.0")]
        pub use crate::hl::chunks::ChunkInfoRef;
        pub use crate::hl::dataset::{Chunk, Dataset, DatasetBuilder};
        pub use crate::hl::plist::dataset_access::*;
        pub use crate::hl::plist::dataset_create::*;
    }

    pub mod datatype {
        pub use crate::hl::datatype::{ByteOrder, Conversion, Datatype};
    }

    pub mod file {
        pub use crate::hl::file::{File, FileBuilder, OpenMode};
        pub use crate::hl::plist::file_access::*;
        pub use crate::hl::plist::file_create::*;
    }

    pub mod plist {
        pub use crate::hl::plist::dataset_access::{DatasetAccess, DatasetAccessBuilder};
        pub use crate::hl::plist::dataset_create::{DatasetCreate, DatasetCreateBuilder};
        pub use crate::hl::plist::file_access::{FileAccess, FileAccessBuilder};
        pub use crate::hl::plist::file_create::{FileCreate, FileCreateBuilder};
        pub use crate::hl::plist::link_create::{LinkCreate, LinkCreateBuilder};
        pub use crate::hl::plist::{PropertyList, PropertyListClass};

        pub mod dataset_access {
            pub use crate::hl::plist::dataset_access::*;
        }
        pub mod dataset_create {
            pub use crate::hl::plist::dataset_create::*;
        }
        pub mod file_access {
            pub use crate::hl::plist::file_access::*;
        }
        pub mod file_create {
            pub use crate::hl::plist::file_create::*;
        }
        pub mod link_create {
            pub use crate::hl::plist::link_create::*;
        }
    }
    pub mod filters {
        pub use crate::hl::filters::*;
    }
}

pub use crate::export::*;

#[macro_use]
mod macros;
#[macro_use]
mod class;

mod dim;
mod error;
#[doc(hidden)]
pub mod globals;
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
        error::{h5check, H5ErrorCode},
        export::*,
        handle::Handle,
        hl::plist::PropertyListClass,
        sync::sync,
        util::{
            get_h5_str, h5_free_memory, string_from_cstr, string_from_fixed_bytes,
            string_to_fixed_bytes, to_cstring,
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
    #[cfg(feature = "1.8.16")]
    {
        use self::internal_prelude::hbool_t;
        use hdf5_sys::h5::H5is_library_threadsafe;
        let mut ts: hbool_t = 0;
        h5call!(H5is_library_threadsafe(&mut ts)).map(|_| ts > 0).unwrap_or(false)
    }
    #[cfg(not(feature = "1.8.16"))]
    {
        cfg!(feature = "threadsafe")
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
