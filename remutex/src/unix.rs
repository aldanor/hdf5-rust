// Copyright 2015 The Rust Project Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::mem;
use std::cell::UnsafeCell;

pub struct ReentrantMutex { inner: UnsafeCell<ffi::pthread_mutex_t> }

unsafe impl Send for ReentrantMutex {}
unsafe impl Sync for ReentrantMutex {}

impl ReentrantMutex {
    pub unsafe fn uninitialized() -> ReentrantMutex {
        ReentrantMutex { inner: mem::uninitialized() }
    }

    pub unsafe fn init(&mut self) {
        let mut attr: ffi::pthread_mutexattr_t = mem::uninitialized();
        let result = ffi::pthread_mutexattr_init(&mut attr as *mut _);
        debug_assert_eq!(result, 0);
        let result = ffi::pthread_mutexattr_settype(&mut attr as *mut _,
                                                    ffi::PTHREAD_MUTEX_RECURSIVE);
        debug_assert_eq!(result, 0);
        let result = ffi::pthread_mutex_init(self.inner.get(), &attr as *const _);
        debug_assert_eq!(result, 0);
        let result = ffi::pthread_mutexattr_destroy(&mut attr as *mut _);
        debug_assert_eq!(result, 0);
    }

    pub unsafe fn lock(&self) {
        let result = ffi::pthread_mutex_lock(self.inner.get());
        debug_assert_eq!(result, 0);
    }

    #[inline]
    pub unsafe fn try_lock(&self) -> bool {
        ffi::pthread_mutex_trylock(self.inner.get()) == 0
    }

    pub unsafe fn unlock(&self) {
        let result = ffi::pthread_mutex_unlock(self.inner.get());
        debug_assert_eq!(result, 0);
    }

    pub unsafe fn destroy(&self) {
        let result = ffi::pthread_mutex_destroy(self.inner.get());
        debug_assert_eq!(result, 0);
    }
}

mod ffi {
    use libc;
    pub use self::os::{pthread_mutex_t, pthread_mutexattr_t, PTHREAD_MUTEX_RECURSIVE};

    extern {
        pub fn pthread_mutex_init(lock: *mut pthread_mutex_t, attr: *const pthread_mutexattr_t)
                                 -> libc::c_int;
        pub fn pthread_mutex_destroy(lock: *mut pthread_mutex_t) -> libc::c_int;
        pub fn pthread_mutex_lock(lock: *mut pthread_mutex_t) -> libc::c_int;
        pub fn pthread_mutex_trylock(lock: *mut pthread_mutex_t) -> libc::c_int;
        pub fn pthread_mutex_unlock(lock: *mut pthread_mutex_t) -> libc::c_int;

        pub fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> libc::c_int;
        pub fn pthread_mutexattr_destroy(attr: *mut pthread_mutexattr_t) -> libc::c_int;
        pub fn pthread_mutexattr_settype(attr: *mut pthread_mutexattr_t, _type: libc::c_int)
                                        -> libc::c_int;
    }

    #[cfg(any(target_os = "freebsd", target_os = "dragonfly",
              target_os = "bitrig",  target_os = "openbsd"))]
    mod os {
        use libc;

        pub type pthread_mutex_t = *mut libc::c_void;
        pub type pthread_mutexattr_t = *mut libc::c_void;
        pub const PTHREAD_MUTEX_RECURSIVE: libc::c_int = 2;
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    mod os {
        use libc;

        #[cfg(any(target_arch = "x86_64",
                  target_arch = "aarch64"))]
        const __PTHREAD_MUTEX_SIZE__: usize = 56;
        #[cfg(any(target_arch = "x86",
                  target_arch = "arm"))]
        const __PTHREAD_MUTEX_SIZE__: usize = 40;

        #[repr(C)]
        pub struct pthread_mutex_t {
            __sig: libc::c_long,
            __opaque: [u8; __PTHREAD_MUTEX_SIZE__],
        }
        #[repr(C)]
        pub struct pthread_mutexattr_t {
            __sig: libc::c_long,
            __opaque: [u8; 16],
        }
        pub const PTHREAD_MUTEX_RECURSIVE: libc::c_int = 2;
    }

    #[cfg(target_os = "linux")]
    mod os {
        use libc;

        #[cfg(target_arch = "x86_64")]
        const __SIZEOF_PTHREAD_MUTEX_T: usize = 40 - 8;
        #[cfg(any(target_arch = "x86",
                  target_arch = "arm",
                  target_arch = "mips",
                  target_arch = "mipsel",
                  target_arch = "powerpc"))]
        const __SIZEOF_PTHREAD_MUTEX_T: usize = 24 - 8;
        #[cfg(target_arch = "aarch64")]
        const __SIZEOF_PTHREAD_MUTEX_T: usize = 48 - 8;

        #[repr(C)]
        pub struct pthread_mutex_t {
            __align: libc::c_longlong,
            size: [u8; __SIZEOF_PTHREAD_MUTEX_T],
        }
        #[repr(C)]
        pub struct pthread_mutexattr_t {
            __align: libc::c_longlong,
            size: [u8; 16],
        }
        pub const PTHREAD_MUTEX_RECURSIVE: libc::c_int = 1;
    }

    #[cfg(target_os = "android")]
    mod os {
        use libc;

        #[repr(C)]
        pub struct pthread_mutex_t { value: libc::c_int }
        pub type pthread_mutexattr_t = libc::c_long;
        pub const PTHREAD_MUTEX_RECURSIVE: libc::c_int = 1;
    }
}
