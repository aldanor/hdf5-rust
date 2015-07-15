macro_rules! fail {
    ($err:expr) => (
        return Err(From::from($err));
    );
    ($fmt:expr, $($arg:tt)*) => (
        return Err(From::from(format!($fmt, $($arg)*)));
    );
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
                    panic!("assertion failed: error message \"{}\" doesn't match \"{}\" in `{}`",
                           desc, re, stringify!($expr));
                }
            }
        }
    }
}

/// Run a safe expression in a closure synchronized by a global reentrant mutex.
macro_rules! h5lock_s {
    ($expr:expr) => ({
        use ::sync::sync;
        sync(|| { $expr })
    })
}

/// Run an unsafe expression in a closure synchronized by a global reentrant mutex.
macro_rules! h5lock {
    ($expr:expr) => (h5lock_s!(unsafe { $expr }))
}


macro_rules! h5call_s {
    ($expr:expr) => ({
        use error::h5check;
        h5lock_s!(h5check($expr))
    })
}

macro_rules! h5call {
    ($expr:expr) => (h5call_s!(unsafe { $expr }))
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
    ($expr:expr) => (h5try_s!(unsafe { $expr }))
}

thread_local!(static THREAD_ID: () = ());

pub fn thread_id() -> usize {
    THREAD_ID.with(|x| x as *const _ as usize)
}

macro_rules! debug {
    ($arg:expr) => {
        {
            use macros::thread_id;
            println!("{:x}: {}:{}: {}", thread_id(), file!(), line!(), $arg);
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        {
            use macros::thread_id;
            println!("{:x}: {}:{}: {}", thread_id(), file!(), line!(), format!($fmt, $($arg)*));
        }
    };
}

macro_rules! debug_id {
    ($id:expr) => {
        {
            use object::Object;
            debug!("{} / {} / {}", ($id).id(), ($id).refcount(), ($id).is_valid());
        }
    };
    ($id:expr, $msg:expr) => {
        {
            use object::Object;
            debug!("{}: {} / {} / {}", $msg, ($id).id(), ($id).refcount(), ($id).is_valid());
        }
    };
}
