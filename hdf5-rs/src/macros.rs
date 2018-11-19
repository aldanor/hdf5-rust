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

/// Run a potentially unsafe expression in a closure synchronized by a global reentrant mutex.
#[macro_export]
macro_rules! h5lock {
    ($expr:expr) => {{
        #[cfg_attr(feature = "cargo-clippy", allow(clippy::redundant_closure))]
        #[allow(unused_unsafe)]
        unsafe {
            $crate::sync::sync(|| $expr)
        }
    }};
}

/// Convert result of HDF5 call to Result; execution is guarded by a global reentrant mutex.
#[macro_export]
macro_rules! h5call {
    ($expr:expr) => {
        h5lock!($crate::error::h5check($expr))
    };
}

/// `h5try!(..)` is equivalent to try!(h5call!(..)).
macro_rules! h5try {
    ($expr:expr) => {
        match h5call!($expr) {
            Ok(value) => value,
            Err(err) => return Err(From::from(err)),
        }
    };
}
