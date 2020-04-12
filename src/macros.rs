#![allow(unused_macros)]

use crate::internal_prelude::*;

macro_rules! fail {
    ($err:expr) => (
        return Err(From::from($err));
    );

    ($fmt:expr, $($arg:tt)*) => (
        fail!(format!($fmt, $($arg)*))
    );
}

macro_rules! try_ref_clone {
    ($expr:expr) => {
        match $expr {
            Ok(ref val) => val,
            Err(ref err) => return Err(From::from(err.clone())),
        }
    };
}

macro_rules! ensure {
    ($expr:expr, $err:expr) => (
        if !($expr) {
            fail!($err);
        }
    );
    ($expr: expr, $fmt:expr, $($arg:tt)*) => (
        if !($expr) {
            fail!(format!($fmt, $($arg)*));
        }
    );
}

/// Panics if `$expr` is not an Err(err) with err.description() containing `$err`.
#[cfg(test)]
#[allow(unused_macros)]
macro_rules! assert_err {
    ($expr:expr, $err:expr) => {
        match $expr {
            Ok(_) => {
                panic!("assertion failed: not an error in `{}`", stringify!($expr));
            }
            Err(ref value) => {
                let desc = value.description().to_string();
                if !desc.contains($err) {
                    panic!(
                        "assertion failed: error message `{}` doesn't contain `{}` in `{}`",
                        desc,
                        $err,
                        stringify!($expr)
                    );
                }
            }
        }
    };
}

/// Panics if `$expr` is not an Err(err) with err.description() matching regexp `$err`.
#[cfg(test)]
#[allow(unused_macros)]
macro_rules! assert_err_re {
    ($expr:expr, $err:expr) => {
        match $expr {
            Ok(_) => {
                panic!("assertion failed: not an error in `{}`", stringify!($expr));
            }
            Err(ref value) => {
                use regex::Regex;
                let re = Regex::new($err).unwrap();
                let desc = value.description().to_string();
                if !re.is_match(desc.as_ref()) {
                    panic!(
                        "assertion failed: error message `{}` doesn't match `{}` in `{}`",
                        desc,
                        re,
                        stringify!($expr)
                    );
                }
            }
        }
    };
}

/// Run code containing HDF5 calls in a closure synchronized by a global reentrant mutex.
#[macro_export]
#[doc(hidden)]
macro_rules! h5lock {
    ($expr:expr) => {{
        #[cfg_attr(feature = "cargo-clippy", allow(clippy::redundant_closure))]
        #[allow(unused_unsafe)]
        unsafe {
            $crate::sync::sync(|| $expr)
        }
    }};
}

/// Convert result of an HDF5 call to `hdf5::Result` (guarded by a global reentrant mutex).
#[macro_export]
#[doc(hidden)]
macro_rules! h5call {
    ($expr:expr) => {
        $crate::h5lock!($crate::h5check($expr))
    };
}

/// `h5try!(..)` is a convenience shortcut for `try!(h5call!(..))`.
#[macro_export]
#[doc(hidden)]
macro_rules! h5try {
    ($expr:expr) => {
        match $crate::h5call!($expr) {
            Ok(value) => value,
            Err(err) => return Err(From::from(err)),
        }
    };
}

pub trait H5Get: Copy + Default {
    type Func;

    fn h5get(func: Self::Func, id: hid_t) -> Result<Self>;

    #[inline]
    fn h5get_d(func: Self::Func, id: hid_t) -> Self {
        Self::h5get(func, id).unwrap_or_else(|_| Self::default())
    }
}

macro_rules! h5get {
    ($func:ident($id:expr): $ty:ty) => {
        <($ty,) as $crate::macros::H5Get>::h5get($func as _, $id).map(|x| x.0)
    };
    ($func:ident($id:expr): $($ty:ty),+) => {
        <($($ty),+) as $crate::macros::H5Get>::h5get($func as _, $id)
    };
}

macro_rules! h5get_d {
    ($func:ident($id:expr): $ty:ty) => {
        <($ty,) as $crate::macros::H5Get>::h5get_d($func as _, $id).0
    };
    ($func:ident($id:expr): $($ty:ty),+) => {
        <($($ty),+) as $crate::macros::H5Get>::h5get_d($func as _, $id)
    };
}

macro_rules! impl_h5get {
    ($($name:ident: $ty:ident),+) => {
        impl<$($ty),+> H5Get for ($($ty,)+)
        where
            $($ty: Copy + Default),+,
        {
            type Func = unsafe extern "C" fn(hid_t, $(*mut $ty),+) -> herr_t;

            #[inline]
            fn h5get(func: Self::Func, id: hid_t) -> Result<Self> {
                $(let mut $name: $ty = Default::default();)+
                h5call!(func(id, $(&mut $name),+)).map(|_| ($($name,)+))
            }
        }
    };
}

impl_h5get!(a: A);
impl_h5get!(a: A, b: B);
impl_h5get!(a: A, b: B, c: C);
impl_h5get!(a: A, b: B, c: C, d: D);

macro_rules! h5err {
    ($msg:expr, $major:expr, $minor:expr) => {
        let line = line!();
        let file = $crate::util::to_cstring(file!()).unwrap_or_default();
        let modpath = $crate::util::to_cstring(module_path!()).unwrap_or_default();
        let msg = to_cstring($msg).unwrap_or_default();
        #[allow(unused_unsafe)]
        unsafe {
            ::hdf5_sys::h5e::H5Epush2(
                ::hdf5_sys::h5e::H5E_DEFAULT,
                file.as_ptr(),
                modpath.as_ptr(),
                line as _,
                *$crate::globals::H5E_ERR_CLS,
                *$major,
                *$minor,
                msg.as_ptr(),
            );
        }
    };
}

macro_rules! h5maybe_err {
    ($retcode:expr, $msg:expr, $major:expr, $minor:expr) => {{
        if $crate::error::is_err_code($retcode) {
            h5err!($msg, $major, $minor);
        }
        $retcode
    }};
}
