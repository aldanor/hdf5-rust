mod macros;

pub mod h5;
pub mod h5a;
pub mod h5ac;
pub mod h5c;
pub mod h5d;
pub mod h5e;
pub mod h5f;
pub mod h5g;
pub mod h5i;
pub mod h5l;
pub mod h5o;
pub mod h5t;

pub mod types {
    pub use libc::{c_int, c_uint, c_void, c_char, c_ulonglong, c_longlong, size_t, ssize_t};

    pub use ffi::h5i::hid_t;
    pub use ffi::h5::{herr_t, hbool_t, htri_t, hsize_t, hssize_t, haddr_t};
}

#[allow(unstable, unused_unsafe)]
fn h5sync<T, F>(func: F) -> T where F: FnOnce() -> T,
{
    use std::sync::{StaticMutex, MUTEX_INIT};
    static LOCK: StaticMutex = MUTEX_INIT;
    let _guard = LOCK.lock();
    unsafe { func() }
}
