use std::fmt;

use crate::internal_prelude::*;

pub trait ObjectClass: Sized {
    const NAME: &'static str;
    const VALID_TYPES: &'static [H5I_type_t];

    fn from_handle(handle: Handle) -> Self;

    fn handle(&self) -> &Handle;

    fn short_repr(&self) -> Option<String> {
        // TODO: remove Option<> if it's implemented for all types, and make this required?
        None
    }

    fn from_id(id: hid_t) -> Result<Self> {
        h5lock!({
            if Self::is_valid_id_type(get_id_type(id)) {
                Ok(Self::from_handle(Handle::try_new(id)?))
            } else {
                Err(From::from(format!("Invalid {} id: {}", Self::NAME, id)))
            }
        })
    }

    fn invalid() -> Self {
        Self::from_handle(Handle::invalid())
    }

    fn is_valid_id_type(tp: H5I_type_t) -> bool {
        Self::VALID_TYPES.is_empty() || Self::VALID_TYPES.contains(&tp)
    }

    unsafe fn transmute<T: ObjectClass>(&self) -> &T {
        &*(self as *const Self as *const T)
    }

    unsafe fn transmute_mut<T: ObjectClass>(&mut self) -> &mut T {
        &mut *(self as *mut Self as *mut T)
    }

    fn debug_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: this can moved out if/when specialization lands in stable
        h5lock!({
            if !is_valid_user_id(self.handle().id()) {
                write!(f, "<HDF5 {}: invalid id>", Self::NAME)
            } else {
                if let Some(d) = self.short_repr() {
                    write!(f, "<HDF5 {}: {}>", Self::NAME, d)
                } else {
                    write!(f, "<HDF5 {}>", Self::NAME)
                }
            }
        })
    }
}
