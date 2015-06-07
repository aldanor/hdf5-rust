#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unreachable_code)]

extern crate libc;

#[cfg(unix)]
#[path = "unix.rs"] mod sys;
#[cfg(windows)]
#[path = "windows.rs"] mod sys;

mod poison;
mod remutex;

pub use remutex::ReentrantMutex;
