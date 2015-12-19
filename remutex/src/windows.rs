// Copyright 2015 The Rust Project Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::mem;
use std::cell::UnsafeCell;

pub struct ReentrantMutex { inner: UnsafeCell<ffi::CRITICAL_SECTION> }

unsafe impl Send for ReentrantMutex {}
unsafe impl Sync for ReentrantMutex {}

impl ReentrantMutex {
    #[inline]
    pub unsafe fn uninitialized() -> ReentrantMutex {
        mem::uninitialized()
    }

    #[inline]
    pub unsafe fn init(&mut self) {
        ffi::InitializeCriticalSection(self.inner.get());
    }

    #[inline]
    pub unsafe fn lock(&self) {
        ffi::EnterCriticalSection(self.inner.get());
    }

    #[inline]
    pub unsafe fn try_lock(&self) -> bool {
        ffi::TryEnterCriticalSection(self.inner.get()) != 0
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        ffi::LeaveCriticalSection(self.inner.get());
    }

    #[inline]
    pub unsafe fn destroy(&self) {
        ffi::DeleteCriticalSection(self.inner.get());
    }
}

#[allow(non_camel_case_types)]
mod ffi {
    use libc::{c_long, c_ulong, c_void};

    pub type BOOLEAN = u8;
    pub type LONG = c_long;
    pub type LPVOID = *mut c_void;
    pub type HANDLE = LPVOID;
    pub type ULONG_PTR = c_ulong;

    #[allow(non_snake_case)]
    #[repr(C)]
    pub struct CRITICAL_SECTION {
        CriticalSectionDebug: LPVOID,
        LockCount: LONG,
        RecursionCount: LONG,
        OwningThread: HANDLE,
        LockSemaphore: HANDLE,
        SpinCount: ULONG_PTR
    }

    extern "system" {
        pub fn InitializeCriticalSection(CriticalSection: *mut CRITICAL_SECTION);
        pub fn EnterCriticalSection(CriticalSection: *mut CRITICAL_SECTION);
        pub fn TryEnterCriticalSection(CriticalSection: *mut CRITICAL_SECTION) -> BOOLEAN;
        pub fn LeaveCriticalSection(CriticalSection: *mut CRITICAL_SECTION);
        pub fn DeleteCriticalSection(CriticalSection: *mut CRITICAL_SECTION);
    }
}
