macro_rules! fail {
    ($err:expr) => (
        return Err(From::from($err));
    )
}

macro_rules! ensure {
    ($expr:expr, $err:expr) => (
        if !($expr) {
            fail!($err);
        }
    )
}

macro_rules! assert_err {
    ($expr:expr, $err:expr) => {
        match &($expr) {
            &Ok(ref value)  => {
                panic!("assertion failed: `{}` is not an error", stringify!($expr));
            }
            &Err(ref value) => {
                use regex::Regex;
                use std::error::Error as Error_;
                let re = Regex::new($err).unwrap();
                let desc = value.description().to_string();
                if !re.is_match(desc.as_ref()) {
                    panic!("assertion failed: \"{}\" doesn't match \"{}\" in `{}`",
                           desc, re, stringify!($expr));
                }
            }
        }
    }
}

macro_rules! h5lock_s {
    ($expr:expr) => ({
        use ::sync::sync;
        sync(|| { $expr })
    })
}

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

macro_rules! register_hid {
    ($rust_name:ident, $c_name:ident) => {
        #[link(name = "hdf5")]
        extern {
            static $c_name: hid_t;
        }
        lazy_static! {
            pub static ref $rust_name: hid_t = {
                use ffi::h5;
                h5lock!(h5::H5open());
                $c_name
            };
        }
    }
}
