use std::ops::Deref;

use hdf5_sys::h5o::H5O_type_t;
use hdf5_sys::h5r::H5R_type_t::H5R_OBJECT2;
use hdf5_sys::h5r::{
    H5R_ref_t, H5Rcopy, H5Rcreate_object, H5Rdestroy, H5Rget_obj_type3, H5Ropen_object,
};

use super::ObjectReference;
use crate::internal_prelude::*;
use crate::Location;

/// A reference to a HDF5 item that can be stored in attributes or datasets.
#[repr(transparent)]
pub struct StdReference {
    inner: H5R_ref_t,
}

impl StdReference {
    fn ptr(&self) -> *const H5R_ref_t {
        std::ptr::addr_of!(self.inner)
    }

    pub fn try_clone(&self) -> Result<Self> {
        unsafe {
            let mut dst = std::mem::MaybeUninit::uninit();
            h5call!(H5Rcopy(self.ptr(), dst.as_mut_ptr()))?;
            Ok(Self { inner: dst.assume_init() })
        }
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct ObjectReference2(StdReference);

impl ObjectReference2 {
    pub fn try_clone(&self) -> Result<Self> {
        self.0.try_clone().map(|std_ref| Self(std_ref))
    }
}

impl Deref for ObjectReference2 {
    type Target = StdReference;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ObjectReference for ObjectReference2 {
    const REF_TYPE: hdf5_sys::h5r::H5R_type_t = H5R_OBJECT2;

    fn ptr(&self) -> *const c_void {
        self.0.ptr().cast()
    }

    fn create(location: &Location, name: &str) -> Result<Self> {
        let reference: H5R_ref_t = create_object_reference(location, name)?;
        Ok(Self(StdReference { inner: reference }))
    }

    fn get_object_type(&self, _location: &Location) -> Result<hdf5_sys::h5o::H5O_type_t> {
        let mut objtype = std::mem::MaybeUninit::<H5O_type_t>::uninit();
        h5call!(H5Rget_obj_type3(self.0.ptr(), H5P_DEFAULT, objtype.as_mut_ptr()))?;
        let objtype = unsafe { objtype.assume_init() };
        Ok(objtype)
    }

    fn dereference(&self, location: &Location) -> Result<ReferencedObject> {
        let object_type = self.get_object_type(location)?;
        let object_id = h5call!(H5Ropen_object(self.0.ptr(), H5P_DEFAULT, H5P_DEFAULT))?;
        ReferencedObject::from_type_and_id(object_type, object_id)
    }
}

unsafe impl H5Type for ObjectReference2 {
    fn type_descriptor() -> hdf5_types::TypeDescriptor {
        hdf5_types::TypeDescriptor::Reference(hdf5_types::Reference::Std)
    }
}

//todo: could we query some actual object parameters to make this more useful?
impl std::fmt::Debug for StdReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StdReference")
    }
}

unsafe impl H5Type for StdReference {
    fn type_descriptor() -> hdf5_types::TypeDescriptor {
        hdf5_types::TypeDescriptor::Reference(hdf5_types::Reference::Std)
    }
}

impl Drop for StdReference {
    fn drop(&mut self) {
        let _e = h5call!(H5Rdestroy(&mut self.inner));
    }
}

fn create_object_reference(dataset: &Location, name: &str) -> Result<H5R_ref_t> {
    let mut out: std::mem::MaybeUninit<H5R_ref_t> = std::mem::MaybeUninit::uninit();
    let name = to_cstring(name)?;
    h5call!(H5Rcreate_object(dataset.id(), name.as_ptr(), H5P_DEFAULT, out.as_mut_ptr().cast(),))?;
    unsafe { Ok(out.assume_init()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_references() {
        use super::ReferencedObject;
        with_tmp_file(|file| {
            file.create_group("g").unwrap();
            let gref = file.reference::<ObjectReference2>("g").unwrap();
            let group = file.dereference(&gref).unwrap();
            assert!(matches!(group, ReferencedObject::Group(_)));

            file.new_dataset::<i32>().create("ds").unwrap();
            let dsref = file.reference::<ObjectReference2>("ds").unwrap();
            let ds = file.dereference(&dsref).unwrap();
            assert!(matches!(ds, ReferencedObject::Dataset(_)));
        })
    }

    #[test]
    fn test_standard_reference_clone() {
        with_tmp_file(|file| {
            let orig_group = file.create_group("g").unwrap();
            let ref1 = file.reference::<ObjectReference2>("g").unwrap();
            let ref2 = ref1.try_clone().unwrap();

            let ref2_group = ref2.dereference(&file).unwrap();
            match ref2_group {
                crate::ReferencedObject::Group(g) => {
                    assert_eq!(g.name(), orig_group.name())
                }
                _ => panic!("Wrong reference type."),
            }
        })
    }
}
