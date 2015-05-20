#![feature(std_misc)]
#![feature(core)]

extern crate libc;
extern crate num;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate bitflags;

#[macro_use]
pub mod macros;

#[allow(non_camel_case_types, non_snake_case, raw_pointer_derive, dead_code)]
pub mod ffi;

pub mod error;
pub mod mutex;

pub mod sync {
    pub fn h5sync<T, F>(func: F) -> T where F: FnOnce() -> T,
    {
        use mutex::{RecursiveMutex, RECURSIVE_MUTEX_INIT};
        static LOCK: RecursiveMutex = RECURSIVE_MUTEX_INIT;
        let _guard = LOCK.lock();
        func()
    }
}
