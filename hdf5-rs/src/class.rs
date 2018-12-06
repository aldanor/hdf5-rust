use crate::internal_prelude::*;

pub trait ObjectClass: Sized {
    fn object_class_name() -> &'static str;

    fn valid_types() -> ValidTypes;

    fn from_handle(handle: Handle) -> Self;

    fn from_id(id: hid_t) -> Result<Self> {
        h5lock!({
            if Self::is_valid_id_type(get_id_type(id)) {
                Ok(Self::from_handle(Handle::new(id)?))
            } else {
                Err(From::from(format!("Invalid {} id: {}", Self::object_class_name(), id)))
            }
        })
    }

    fn invalid() -> Self;

    fn is_valid_id_type(tp: H5I_type_t) -> bool {
        Self::valid_types().check(tp)
    }

    unsafe fn transmute<T: ObjectClass>(&self) -> &T {
        &*(self as *const Self as *const T)
    }

    unsafe fn transmute_mut<T: ObjectClass>(&mut self) -> &mut T {
        &mut *(self as *mut Self as *mut T)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ValidTypes {
    All,
    Just(H5I_type_t),
    OneOf(&'static [H5I_type_t]),
}

impl ValidTypes {
    pub fn check(&self, tp: H5I_type_t) -> bool {
        match self {
            ValidTypes::All => true,
            ValidTypes::Just(t) => *t == tp,
            ValidTypes::OneOf(t) => t.contains(&tp),
        }
    }
}

impl From<H5I_type_t> for ValidTypes {
    fn from(tp: H5I_type_t) -> ValidTypes {
        ValidTypes::Just(tp)
    }
}

impl From<Option<H5I_type_t>> for ValidTypes {
    fn from(tp: Option<H5I_type_t>) -> ValidTypes {
        tp.map_or(ValidTypes::All, ValidTypes::Just)
    }
}

impl From<&'static [H5I_type_t]> for ValidTypes {
    fn from(tp: &'static [H5I_type_t]) -> ValidTypes {
        ValidTypes::OneOf(tp)
    }
}

pub trait MaybeString {
    fn maybe_string(self) -> Option<String>;
}

impl MaybeString for String {
    fn maybe_string(self) -> Option<String> {
        Some(self)
    }
}

impl MaybeString for Option<String> {
    fn maybe_string(self) -> Option<String> {
        self
    }
}

macro_rules! impl_class {
    ($ty:ident: name=$name:expr, types=$valid_types:expr, repr=$fmt:expr) => {
        impl crate::class::ObjectClass for $ty {
            #[inline]
            fn object_class_name() -> &'static str {
                $name
            }

            #[inline]
            fn valid_types() -> crate::class::ValidTypes {
                ($valid_types).into()
            }

            #[inline]
            fn from_handle(handle: crate::handle::Handle) -> Self {
                $ty(handle)
            }

            #[inline]
            fn invalid() -> Self {
                $ty(crate::handle::Handle::invalid())
            }
        }

        #[cfg_attr(feature = "cargo-clippy", allow(clippy::redundant_closure_call))]
        impl ::std::fmt::Debug for $ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                let name = Self::object_class_name();
                h5lock!({
                    if !crate::handle::is_valid_user_id(self.id()) {
                        write!(f, "<HDF5 {}: invalid id>", name)
                    } else {
                        use crate::class::MaybeString;
                        let d: Option<String> = ($fmt)(self).maybe_string();
                        if let Some(d) = d {
                            write!(f, "<HDF5 {}: {}>", name, d)
                        } else {
                            write!(f, "<HDF5 {}>", name)
                        }
                    }
                })
            }
        }
    };

    ($parent:ty => $ty:ident: name=$name:expr, types=$valid_types:expr, repr=$fmt:expr) => {
        impl_class!($ty: name = $name, types = $valid_types, repr = $fmt);

        impl ::std::ops::Deref for $ty {
            type Target = $parent;

            #[inline]
            fn deref(&self) -> &$parent {
                unsafe { self.transmute::<$parent>() }
            }
        }

        impl ::std::ops::DerefMut for $ty {
            #[inline]
            fn deref_mut(&mut self) -> &mut $parent {
                unsafe { self.transmute_mut::<$parent>() }
            }
        }
    };
}
