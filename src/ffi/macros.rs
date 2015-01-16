#![macro_use]

macro_rules! register_hid {
    ($rust_name:ident, $c_name:ident, $func:ident) => {
        #[link(name = "hdf5")] extern {
            static mut $c_name: hid_t;
        }
        lazy_static! {
            pub static ref $rust_name: hid_t = { $func(); $c_name };
        }
    }
}

macro_rules! register_h5open {
    ($rust_name:ident, $c_name:ident) => {
        register_hid!($rust_name, $c_name, H5open);
    }
}
