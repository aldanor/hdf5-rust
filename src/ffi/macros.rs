#![macro_use]

macro_rules! h5lock {
    ($expr:expr) => {
        {
            use ffi::h5sync;
            h5sync(|| { $expr })
        }
    }
}

macro_rules! register_hid {
    ($rust_name:ident, $c_name:ident, $init:expr) => {
        #[link(name = "hdf5")]
        extern {
            static $c_name: hid_t;
        }
        lazy_static! {
            pub static ref $rust_name: hid_t = {
                h5lock!($init);
                $c_name
            };
        }
    }
}

macro_rules! register_hid_h5open {
    ($rust_name:ident, $c_name:ident) => {
        register_hid!($rust_name, $c_name, {
            use ffi::h5::H5open;
            H5open()
        });
    }
}
