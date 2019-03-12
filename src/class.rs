use std::fmt;
use std::mem;
use std::ptr;

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

    fn validate(&self) -> Result<()> {
        // any extra post-validation goes here if needed
        Ok(())
    }

    fn from_id(id: hid_t) -> Result<Self> {
        h5lock!({
            if Self::is_valid_id_type(get_id_type(id)) {
                let handle = Handle::try_new(id)?;
                let obj = Self::from_handle(handle);
                obj.validate().map(|_| obj)
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

    unsafe fn cast<T: ObjectClass>(self) -> T {
        // This method requires you to be 18 years or older to use it
        let obj = ptr::read(&self as *const _ as *const _);
        mem::forget(self);
        obj
    }

    fn debug_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: this can moved out if/when specialization lands in stable
        h5lock!({
            if !is_valid_user_id(self.handle().id()) {
                write!(f, "<HDF5 {}: invalid id>", Self::NAME)
            } else if let Some(d) = self.short_repr() {
                write!(f, "<HDF5 {}: {}>", Self::NAME, d)
            } else {
                write!(f, "<HDF5 {}>", Self::NAME)
            }
        })
    }
}

pub unsafe fn from_id<T: ObjectClass>(id: hid_t) -> Result<T> {
    T::from_id(id)
}
