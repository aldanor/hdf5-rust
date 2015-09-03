macro_rules! fail {
    ($err:expr) => (
        return Err(From::from($err));
    );

    ($fmt:expr, $($arg:tt)*) => (
        fail!(format!($fmt, $($arg)*))
    );
}

macro_rules! try_ref_clone {
    ($expr:expr) => (
        match $expr {
            Ok(ref val) => val,
            Err(ref err) => {
                return Err(From::from(err.clone()))
            }
        }
    )
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

/// Panics if `$expr` is not an Err(err) with err.description() matching regexp `$err`.
macro_rules! assert_err {
    ($expr:expr, $err:expr) => {
        match &($expr) {
            &Ok(_) => {
                panic!("assertion failed: not an error in `{}`", stringify!($expr));
            }
            &Err(ref value) => {
                use regex::Regex;
                use std::error::Error as BaseError;
                let re = Regex::new($err).unwrap();
                let desc = value.description().to_string();
                if !re.is_match(desc.as_ref()) {
                    panic!(
                        "assertion failed: error message \"{}\" doesn't match \"{}\" in `{}`",
                        desc, re, stringify!($expr)
                    );
                }
            }
        }
    }
}

/// Run a safe expression in a closure synchronized by a global reentrant mutex.
macro_rules! h5lock_s {
    ($expr:expr) => (
        $crate::sync::sync(|| { $expr })
    )
}

/// Run an unsafe expression in a closure synchronized by a global reentrant mutex.
macro_rules! h5lock {
    ($expr:expr) => (
        h5lock_s!(unsafe { $expr })
    )
}

macro_rules! h5call_s {
    ($expr:expr) => (
        h5lock_s!($crate::error::h5check($expr))
    )
}

macro_rules! h5call {
    ($expr:expr) => (
        h5call_s!(unsafe { $expr })
    )
}

macro_rules! h5try_s {
    ($expr:expr) => (match h5call_s!($expr) {
        Ok(value) => value,
        Err(err)  => {
            return Err(From::from(err))
        },
    })
}

macro_rules! h5try {
    ($expr:expr) => (
        h5try_s!(unsafe { $expr })
    )
}
