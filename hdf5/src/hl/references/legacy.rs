//! The pre v1.12.0 reference types.

use hdf5_sys::h5r::hobj_ref_t;
use hdf5_types::H5Type;

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct ObjectReference1 {
    inner: hobj_ref_t,
}

unsafe impl H5Type for ObjectReference1 {
    fn type_descriptor() -> hdf5_types::TypeDescriptor {
        hdf5_types::TypeDescriptor::Reference(hdf5_types::Reference::Object)
    }
}

// impl Drop for ObjectReference {
//     fn drop(&mut self) {
//         // let _e = h5call!(H5Rdestroy(&mut self.inner));
//     }
// }
