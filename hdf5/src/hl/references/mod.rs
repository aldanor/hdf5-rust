use crate::internal_prelude::*;
use crate::Location;

mod legacy;
mod standard;

pub use legacy::ObjectReference1;
pub use standard::ObjectReference2;

pub trait ObjectReference: Sized + H5Type {
    fn create(source: &Location, name: &str) -> Result<Self>;
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
