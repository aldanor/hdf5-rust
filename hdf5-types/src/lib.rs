#![recursion_limit = "1024"]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::missing_safety_doc))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::missing_const_for_fn))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::redundant_pub_crate))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::must_use_candidate))]

//! Types that can be stored and retrieved from a `HDF5` dataset
//!
//! Crate features:
//! * `h5-alloc`: Use the `hdf5` allocator for varlen types and dynamic values.
//!                     This is necessary on platforms which uses different allocators
//!                     in different libraries (e.g. dynamic libraries on windows),
//!                     or if `hdf5-c` is compiled with the MEMCHECKER option.
//!                     This option is forced on in the case of using a `windows` DLL.

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod array;
pub mod dyn_value;
mod h5type;
mod string;

#[cfg(feature = "complex")]
mod complex;

pub use self::array::VarLenArray;
pub use self::dyn_value::{DynValue, OwnedDynValue};
pub use self::h5type::{
    CompoundField, CompoundType, EnumMember, EnumType, FloatSize, H5Type, IntSize, TypeDescriptor,
};
pub use self::string::{FixedAscii, FixedUnicode, StringError, VarLenAscii, VarLenUnicode};

pub(crate) unsafe fn malloc(n: usize) -> *mut core::ffi::c_void {
    cfg_if::cfg_if! {
        if #[cfg(any(feature = "h5-alloc", windows_dll))] {
            hdf5_sys::h5::H5allocate_memory(n, 0)
        } else {
            libc::malloc(n)
        }
    }
}

pub(crate) unsafe fn free(ptr: *mut core::ffi::c_void) {
    cfg_if::cfg_if! {
        if #[cfg(any(feature = "h5-alloc", windows_dll))] {
            hdf5_sys::h5::H5free_memory(ptr);
        } else {
            libc::free(ptr);
        }
    }
}

pub const USING_H5_ALLOCATOR: bool = {
    cfg_if::cfg_if! {
        if #[cfg(any(feature = "h5-alloc", windows_dll))] {
            true
        } else {
            false
        }
    }
};
