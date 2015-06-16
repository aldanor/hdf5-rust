// Copyright 2015 The Rust Project Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate winapi;
extern crate kernel32;

use std::mem;
use std::cell::UnsafeCell;

pub struct ReentrantMutex { inner: UnsafeCell<winapi::CRITICAL_SECTION> }

unsafe impl Send for ReentrantMutex {}
unsafe impl Sync for ReentrantMutex {}

impl ReentrantMutex {
    #[inline]
    pub unsafe fn uninitialized() -> ReentrantMutex {
        mem::uninitialized()
    }

    #[inline]
    pub unsafe fn init(&mut self) {
        kernel32::InitializeCriticalSection(self.inner.get());
    }

    #[inline]
    pub unsafe fn lock(&self) {
        kernel32::EnterCriticalSection(self.inner.get());
    }

    #[inline]
    pub unsafe fn try_lock(&self) -> bool {
        kernel32::TryEnterCriticalSection(self.inner.get()) != 0
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        kernel32::LeaveCriticalSection(self.inner.get());
    }

    #[inline]
    pub unsafe fn destroy(&self) {
        kernel32::DeleteCriticalSection(self.inner.get());
    }
}
