// Copyright 2014 The Rust Project Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::cell::UnsafeCell;
use std::error::Error;
use std::fmt;
use std::thread;

use std::any::Any;

pub struct Flag { failed: UnsafeCell<bool> }

// This flag is only ever accessed with a lock previously held. Note that this
// a totally private structure.
unsafe impl Send for Flag {}
unsafe impl Sync for Flag {}

impl Flag {
    #[inline]
    pub fn borrow(&self) -> LockResult<Guard> {
        let ret = Guard { panicking: thread::panicking() };
        if unsafe { *self.failed.get() } {
            Err(PoisonError::new(ret))
        } else {
            Ok(ret)
        }
    }

    #[inline]
    pub fn done(&self, guard: &Guard) {
        if !guard.panicking && thread::panicking() {
            unsafe { *self.failed.get() = true; }
        }
    }

    #[inline]
    pub fn get(&self) -> bool {
        unsafe { *self.failed.get() }
    }

    pub fn new() -> Flag {
        Flag { failed: UnsafeCell::new(false) }
    }
}

pub struct Guard {
    panicking: bool,
}

/// A type of error which can be returned whenever a lock is acquired.
///
/// Both Mutexes and RwLocks are poisoned whenever a task fails while the lock
/// is held. The precise semantics for when a lock is poisoned is documented on
/// each lock, but once a lock is poisoned then all future acquisitions will
/// return this error.
pub struct PoisonError<T> {
    guard: T,
}

/// An enumeration of possible errors which can occur while calling the
/// `try_lock` method.
pub enum TryLockError<T> {
    /// The lock could not be acquired because another task failed while holding
    /// the lock.
    Poisoned(PoisonError<T>),
    /// The lock could not be acquired at this time because the operation would
    /// otherwise block.
    WouldBlock,
}

/// A type alias for the result of a lock method which can be poisoned.
///
/// The `Ok` variant of this result indicates that the primitive was not
/// poisoned, and the `Guard` is contained within. The `Err` variant indicates
/// that the primitive was poisoned. Note that the `Err` variant *also* carries
/// the associated guard, and it can be acquired through the `into_inner`
/// method.
pub type LockResult<Guard> = Result<Guard, PoisonError<Guard>>;

/// A type alias for the result of a nonblocking locking method.
///
/// For more information, see `LockResult`. A `TryLockResult` doesn't
/// necessarily hold the associated guard in the `Err` type as the lock may not
/// have been acquired for other reasons.
pub type TryLockResult<Guard> = Result<Guard, TryLockError<Guard>>;

impl<T> fmt::Debug for PoisonError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        "PoisonError { inner: .. }".fmt(f)
    }
}

impl<T> fmt::Display for PoisonError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        "poisoned lock: another task failed inside".fmt(f)
    }
}

impl<T: Send + Any> Error for PoisonError<T> {
    fn description(&self) -> &str {
        "poisoned lock: another task failed inside"
    }
}

impl<T> PoisonError<T> {
    /// Create a `PoisonError`.
    pub fn new(guard: T) -> PoisonError<T> {
        PoisonError { guard: guard }
    }

    /// Consumes this error indicating that a lock is poisoned, returning the
    /// underlying guard to allow access regardless.
    pub fn into_inner(self) -> T { self.guard }

    /// Reaches into this error indicating that a lock is poisoned, returning a
    /// reference to the underlying guard to allow access regardless.
    pub fn get_ref(&self) -> &T { &self.guard }

    /// Reaches into this error indicating that a lock is poisoned, returning a
    /// mutable reference to the underlying guard to allow access regardless.
    pub fn get_mut(&mut self) -> &mut T { &mut self.guard }
}

impl<T> From<PoisonError<T>> for TryLockError<T> {
    fn from(err: PoisonError<T>) -> TryLockError<T> {
        TryLockError::Poisoned(err)
    }
}

impl<T> fmt::Debug for TryLockError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TryLockError::Poisoned(..) => "Poisoned(..)".fmt(f),
            TryLockError::WouldBlock => "WouldBlock".fmt(f)
        }
    }
}

impl<T: Send + Any> fmt::Display for TryLockError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl<T: Send + Any> Error for TryLockError<T> {
    fn description(&self) -> &str {
        match *self {
            TryLockError::Poisoned(ref p) => p.description(),
            TryLockError::WouldBlock => "try_lock failed because the operation would block"
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            TryLockError::Poisoned(ref p) => Some(p),
            _ => None
        }
    }
}

pub fn map_result<T, U, F>(result: LockResult<T>, f: F)
                           -> LockResult<U>
                           where F: FnOnce(T) -> U {
    match result {
        Ok(t) => Ok(f(t)),
        Err(PoisonError { guard }) => Err(PoisonError::new(f(guard)))
    }
}
