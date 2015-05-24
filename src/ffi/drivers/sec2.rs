use ffi::types::hid_t;

lazy_static! {
    pub static ref H5FD_SEC2: hid_t = unsafe { H5FD_sec2_init() };
}

#[link(name = "hdf5")]
extern {
    pub fn H5FD_sec2_init() -> hid_t;
    pub fn H5FD_sec2_term();
    pub fn H5Pset_fapl_sec2(fapl_id: hid_t);
}
