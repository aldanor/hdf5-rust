#![recursion_limit = "1024"]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::missing_safety_doc))]

//! Types that can be stored and retrieved from a `HDF5` dataset
//!
//! Crate features:
//! * `const-generics`: Uses const generics to enable arrays [T; N] for all N.
//!                     Compiling without this limits arrays to certain prespecified
//!                     sizes
//! * `force-h5-allocator`: Force the `hdf5` allocator for varlen types and dynamic values.
//!                         This is necessary on platforms which uses different allocators
//!                         in different libraries (e.g. dynamic libraries on windows),
//!                         or if `hdf5-c` is compiled with the MEMCHECKER option.
//!                         This option is forced on in the case of using a `windows` DLL.

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod array;
pub mod dyn_value;
mod h5type;
mod string;

pub use self::array::{Array, VarLenArray};
pub use self::dyn_value::{DynValue, OwnedDynValue};
pub use self::h5type::{
    CompoundField, CompoundType, EnumMember, EnumType, FloatSize, H5Type, IntSize, TypeDescriptor,
};
pub use self::string::{FixedAscii, FixedUnicode, StringError, VarLenAscii, VarLenUnicode};

pub(crate) unsafe fn malloc(n: usize) -> *mut core::ffi::c_void {
    #[cfg(any(feature = "force-h5-allocator", windows_dll))]
    {
        hdf5_sys::h5::H5allocate_memory(n, 0)
    }
    #[cfg(not(any(feature = "force-h5-allocator", windows_dll)))]
    {
        libc::malloc(n)
    }
}

pub(crate) unsafe fn free(ptr: *mut core::ffi::c_void) {
    #[cfg(any(feature = "force-h5_allocator", windows_dll))]
    {
        hdf5_sys::h5::H5free_memory(ptr);
    }
    #[cfg(not(any(feature = "force-h5-allocator", windows_dll)))]
    {
        libc::free(ptr)
    }
}

#[cfg(any(feature = "force-h5-allocator", windows_dll))]
pub const USING_H5_ALLOCATOR: bool = true;
#[cfg(not(any(feature = "force-h5-allocator", windows_dll)))]
pub const USING_H5_ALLOCATOR: bool = false;
