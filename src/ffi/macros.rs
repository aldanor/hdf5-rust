#![macro_use]

macro_rules! register_hid {
    ($rust_name:ident, $c_name:ident, $func:ident) => {
        extern "C" {
            static mut $c_name: hid_t;
        }
        lazy_static! {
            pub static ref $rust_name: hid_t = { $func(); $c_name };
        }
    }
}
