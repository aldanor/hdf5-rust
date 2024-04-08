use std::ops::Deref;

use hdf5_sys::h5r::{
    H5R_ref_t, H5Rcopy, H5Rcreate_object, H5Rdereference2, H5Rdestroy, H5Requal, H5Rget_obj_type3,
    H5R_OBJECT1,
};

use super::ObjectReference;
use crate::internal_prelude::*;
use crate::{Dataset, Datatype, Group, Location};

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
#[derive(Debug, PartialEq, Eq)]
pub struct ObjectReference2(StdReference);

impl Deref for ObjectReference2 {
    type Target = StdReference;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ObjectReference for ObjectReference2 {
    fn create(source: &Location, name: &str) -> Result<Self> {
        let mut out: std::mem::MaybeUninit<_> = std::mem::MaybeUninit::uninit();
        let name = to_cstring(name)?;
        h5call!(H5Rcreate_object(source.id(), name.as_ptr(), H5P_DEFAULT, out.as_mut_ptr()))?;
        Ok(Self(StdReference { inner: unsafe { out.assume_init() } }))
    }

    fn dereference(&self, location: &Location) -> Result<ReferencedObject> {
        let mut objtype = std::mem::MaybeUninit::uninit();
        h5call!(H5Rget_obj_type3(self.ptr(), H5P_DEFAULT, objtype.as_mut_ptr()))?;
        let objtype = unsafe { objtype.assume_init() };

        let objid =
            h5call!(H5Rdereference2(location.id(), H5P_DEFAULT, H5R_OBJECT1, self.ptr().cast(),))?;
        use hdf5_sys::h5o::H5O_type_t::*;
        Ok(match objtype {
            H5O_TYPE_GROUP => ReferencedObject::Group(Group::from_id(objid)?),
            H5O_TYPE_DATASET => ReferencedObject::Dataset(Dataset::from_id(objid)?),
            H5O_TYPE_NAMED_DATATYPE => ReferencedObject::Datatype(Datatype::from_id(objid)?),
            #[cfg(feature = "1.12.0")]
            H5O_TYPE_MAP => fail!("Can not create object from a map"),
            H5O_TYPE_UNKNOWN => fail!("Unknown datatype"),
            H5O_TYPE_NTYPES => fail!("hdf5 should not produce this type"),
        })
    }
}

unsafe impl H5Type for ObjectReference2 {
    fn type_descriptor() -> hdf5_types::TypeDescriptor {
        hdf5_types::TypeDescriptor::Reference(hdf5_types::Reference::Std)
    }
}

impl PartialEq for StdReference {
    fn eq(&self, other: &Self) -> bool {
        let result = unsafe { H5Requal(self.ptr(), other.ptr()) };
        println!("Result of H5Requal: {}", result);
        match result {
            0 => false,
            1.. => true,
            // Less than 0 indicates an error but not clear on what the error conditions could be. Fail the equality right now.
            _ => false,
        }
    }
}

impl Eq for StdReference {}

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
    fn test_reference_equality() {
        with_tmp_file(|file| {
            file.create_group("g").unwrap();
            let gref1 = file.reference::<ObjectReference2>("g").unwrap();
            let gref2 = file.reference("g").unwrap();
            assert_eq!(gref1, gref2);

            file.new_dataset::<i32>().create("ds").unwrap();
            file.new_dataset::<i32>().create("ds2").unwrap();
            let dsref1 = file.reference("ds").unwrap();
            let dsref2 = file.reference("ds").unwrap();
            assert_eq!(dsref1, dsref2);

            println!("{}", gref1 == dsref1);
            assert_ne!(gref1, dsref1);
            assert_ne!(dsref1, file.reference("ds2").unwrap());
        })
    }
}
