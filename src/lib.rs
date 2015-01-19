#[macro_use]
extern crate lazy_static;

#[allow(unstable)]
extern crate libc;

#[macro_use]
pub mod macros;

#[allow(non_camel_case_types, non_snake_case, raw_pointer_derive, dead_code)]
pub mod ffi;

#[allow(unstable)]
pub mod error;

pub mod sync {
    #[allow(unstable, unused_unsafe)]
    pub fn h5sync<T, F>(func: F) -> T where F: FnOnce() -> T,
    {
        use std::sync::{StaticMutex, MUTEX_INIT};
        static LOCK: StaticMutex = MUTEX_INIT;
        let _guard = LOCK.lock();
        unsafe { func() }
    }
}
