use ffi::types::hid_t;

lazy_static! {
    pub static ref H5FD_STDIO: hid_t = unsafe { H5FD_stdio_init() };
}

#[link(name = "hdf5")]
extern {
    pub fn H5FD_stdio_init() -> hid_t;
    pub fn H5FD_stdio_term();
    pub fn H5Pset_fapl_stdio(fapl_id: hid_t);
}
