//! The pre v1.12.0 reference types.

use crate::internal_prelude::*;

use hdf5_sys::{
    h5o::H5O_type_t,
    h5p::H5P_DEFAULT,
    h5r::{hobj_ref_t, H5Rcreate, H5Rdereference, H5Rget_obj_type2},
};
use hdf5_types::H5Type;

#[cfg(not(feature = "1.12.0"))]
use hdf5_sys::h5r::H5R_OBJECT as H5R_OBJECT1;
#[cfg(feature = "1.12.0")]
use hdf5_sys::h5r::H5R_OBJECT1;

use crate::{Location, ObjectReference};

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

impl ObjectReference for ObjectReference1 {
    const REF_TYPE: hdf5_sys::h5r::H5R_type_t = H5R_OBJECT1;

    fn ptr(&self) -> *const c_void {
        let pointer = std::ptr::addr_of!(self.inner);
        pointer.cast()
    }

    fn create(dataspace: &Location, name: &str) -> Result<Self> {
        let mut ref_out: std::mem::MaybeUninit<hobj_ref_t> = std::mem::MaybeUninit::uninit();
        let name = to_cstring(name)?;
        h5call!(H5Rcreate(
            ref_out.as_mut_ptr().cast(),
            dataspace.id(),
            name.as_ptr(),
            Self::REF_TYPE,
            -1
        ))?;
        let reference = unsafe { ref_out.assume_init() };
        Ok(Self { inner: reference })
    }

    fn get_object_type(&self, dataset: &Location) -> Result<hdf5_sys::h5o::H5O_type_t> {
        let mut objtype = std::mem::MaybeUninit::<H5O_type_t>::uninit();
        h5call!(H5Rget_obj_type2(dataset.id(), H5R_OBJECT1, self.ptr(), objtype.as_mut_ptr()))?;
        let objtype = unsafe { objtype.assume_init() };
        Ok(objtype)
    }

    fn dereference(&self, dataset: &Location) -> Result<ReferencedObject> {
        let object_type = self.get_object_type(dataset)?;
        let object_id =
            h5call!(H5Rdereference(dataset.id(), H5P_DEFAULT, H5R_OBJECT1, self.ptr()))?;
        ReferencedObject::from_type_and_id(object_type, object_id)
    }
}
