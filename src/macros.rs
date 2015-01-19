macro_rules! ensure {
    ($expr:expr, $err:expr) => {
        if !($expr) { return Err(::std::error::FromError::from_error($err)); }
    }
}

macro_rules! h5lock {
    ($expr:expr) => {
        {
            use sync::h5sync;
            h5sync(|| { $expr })
        }
    }
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
