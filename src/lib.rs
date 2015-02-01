#![feature(collections, std_misc, core)]

#![feature(hash)] // temporarily, because of warnings in bitflags

#![feature(libc)]
extern crate libc;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate bitflags;

#[macro_use]
pub mod macros;

#[allow(non_camel_case_types, non_snake_case, raw_pointer_derive, dead_code)]
pub mod ffi;

pub mod error;

pub mod sync {
    pub fn h5sync<T, F>(func: F) -> T where F: FnOnce() -> T,
    {
        use std::sync::{StaticMutex, MUTEX_INIT};
        static LOCK: StaticMutex = MUTEX_INIT;
        let _guard = LOCK.lock();
        func()
    }
}
