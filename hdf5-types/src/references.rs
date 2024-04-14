//! Types for references.

use std::mem;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Reference {
    Object,
    Region,
    #[cfg(feature = "1.12.0")]
    Std,
}

impl Reference {
    pub fn size(self) -> usize {
        match self {
            Self::Object => mem::size_of::<hdf5_sys::h5r::hobj_ref_t>(),
            Self::Region => mem::size_of::<hdf5_sys::h5r::hdset_reg_ref_t>(),
            #[cfg(feature = "1.12.0")]
            Self::Std => mem::size_of::<hdf5_sys::h5r::H5R_ref_t>(),
        }
    }
}
