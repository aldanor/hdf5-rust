use crate::internal_prelude::*;
use crate::Location;

mod legacy;

#[cfg(feature = "1.12.1")]
mod standard;

use hdf5_sys::h5o::H5O_type_t;
use hdf5_sys::h5r::H5R_type_t;

pub use legacy::ObjectReference1;
#[cfg(feature = "1.12.1")]
pub use standard::ObjectReference2;

pub trait ObjectReference: Sized + H5Type {
    const REF_TYPE: H5R_type_t;
    fn ptr(&self) -> *const c_void;

    /// Get the type of the object that the reference points in the same space as the provided location.
    fn get_object_type(&self, location: &Location) -> Result<H5O_type_t>;

    /// Create a new reference in the same structure as the location provided.
    fn create(location: &Location, name: &str) -> Result<Self>;

    /// Dereference the object reference in the space provided.
    fn dereference(&self, location: &Location) -> Result<ReferencedObject>;
}
/// The result of dereferencing an [object reference](ObjectReference).
///
/// Each variant represents a different type of object that can be referenced by a [ObjectReference].
#[derive(Clone, Debug)]
pub enum ReferencedObject {
    Group(Group),
    Dataset(Dataset),
    Datatype(Datatype),
}

impl ReferencedObject {
    pub fn from_type_and_id(object_type: H5O_type_t, object_id: hid_t) -> Result<Self> {
        use hdf5_sys::h5o::H5O_type_t::*;
        let referenced_object = match object_type {
            H5O_TYPE_GROUP => ReferencedObject::Group(Group::from_id(object_id)?),
            H5O_TYPE_DATASET => ReferencedObject::Dataset(Dataset::from_id(object_id)?),
            H5O_TYPE_NAMED_DATATYPE => ReferencedObject::Datatype(Datatype::from_id(object_id)?),
            #[cfg(feature = "1.12.0")]
            H5O_TYPE_MAP => fail!("Can not create object from a map"),
            H5O_TYPE_UNKNOWN => fail!("Unknown datatype"),
            H5O_TYPE_NTYPES => fail!("hdf5 should not produce this type"),
        };
        Ok(referenced_object)
    }
}
