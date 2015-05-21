/*
  (C) Joshua Yanovski (@pythonesque)

  https://gist.github.com/pythonesque/5bdf071d3617b61b3fed
*/

use std::cell::UnsafeCell;
use std::mem;
use std::sync::{self, MutexGuard, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::marker::PhantomData;

// This may seem useless but provided that each thread has a unique
// thread local address, and this is created once per thread, it will
// always be unique.
thread_local!(static THREAD_ID: () = ());

#[derive(Debug)]
pub enum LockError {
    /// Mutex was poisoned,
    Poisoned,
    /// Mutex would block due to exceeded recursion limits.
    WouldBlockRecursive,
}

#[derive(Debug)]
pub enum TryLockError {
    /// Mutex was poisoned
    Poisoned,
    /// Mutex would block because it is taken by another thread.
    WouldBlockExclusive,
    /// Mutex would block due to exceeded recursion limits.
    WouldBlockRecursive,
}

pub struct RecursiveMutex {
    owner: AtomicUsize,
    recursion: UnsafeCell<u64>,
    mutex: Mutex<()>,
    guard: UnsafeCell<*mut MutexGuard<'static, ()>>,
}

unsafe impl Sync for RecursiveMutex {}
unsafe impl Send for RecursiveMutex {}

#[must_use]
pub struct RecursiveMutexGuard<'a> {
    mutex: &'a RecursiveMutex,
    marker: PhantomData<*mut ()>,  // for !Send
}

// Cannot implement Send because we rely on the guard being dropped in the
// same thread (otherwise we can't use Relaxed).  We might be able to allow
// it with Acquire / Release?

impl RecursiveMutex {
    pub fn new() -> RecursiveMutex {
        RecursiveMutex {
            owner: AtomicUsize::new(0),
            recursion: UnsafeCell::new(0),
            mutex: Mutex::new(()),
            guard: UnsafeCell::new(0 as *mut _),
        }
    }

    pub fn lock(&'static self) -> Result<RecursiveMutexGuard, LockError> {
        let tid = THREAD_ID.with(|x| x as *const _ as usize);

        // Relaxed is sufficient.  If tid == self.owner, it must have been set in the
        // same thread, and nothing else could have taken the lock in another thread;
        // hence, it is synchronized.  Similarly, if tid != self.owner, either the
        // lock was never taken by this thread, or the lock was taken by this thread
        // and then dropped in the same thread (known because the guard is not Send),
        // so that is synchronized as well.  The only reason it needs to be atomic at
        // all is to ensure it doesn't see partial data, and to make sure the load and
        // store aren't reordered around the acquire incorrectly (I believe this is why
        // Unordered is not suitable here, but I may be wrong since acquire() provides
        // a memory fence).
        if tid != self.owner.load(Ordering::Relaxed) {
            match self.mutex.lock() {
                Ok(guard) => unsafe {
                    self.owner.store(tid, Ordering::Relaxed);
                    *self.guard.get() = mem::transmute(Box::new(guard));
                },
                Err(_) => return Err(LockError::Poisoned),
            }
        }
        unsafe {
            let r = self.recursion.get();
            match (*r).checked_add(1) {
                Some(n) => {
                    *r = n;
                },
                None => return Err(LockError::WouldBlockRecursive)
            }
        }
        Ok(RecursiveMutexGuard {
            mutex: self,
            marker: PhantomData,
        })
    }

    pub fn try_lock(&'static self) -> Result<RecursiveMutexGuard, TryLockError> {
        let tid = &THREAD_ID as *const _ as usize;
        // Relaxed is sufficient.  If tid == self.owner, it must have been set in the
        // same thread, and nothing else could have taken the lock in another thread;
        // hence, it is synchronized.  Similarly, if tid != self.owner, either the
        // lock was never taken by this thread, or the lock was taken by this thread
        // and then dropped in the same thread (known because the guard is not Send),
        // so that is synchronized as well.  The only reason it needs to be atomic at
        // all is to ensure it doesn't see partial data, and to make sure the load and
        // store aren't reordered around the acquire incorrectly (I believe this is why
        // Unordered is not suitable here, but I may be wrong since acquire() provides
        // a memory fence).
        if tid != self.owner.load(Ordering::Relaxed) {
            match self.mutex.try_lock() {
                Ok(guard) => unsafe {
                    self.owner.store(tid, Ordering::Relaxed);
                    *self.guard.get() = mem::transmute(Box::new(guard));
                },
                Err(sync::TryLockError::Poisoned(_)) => return Err(TryLockError::Poisoned),
                Err(sync::TryLockError::WouldBlock)  => return Err(TryLockError::WouldBlockExclusive),
            }
        }
        unsafe {
            let r = self.recursion.get();
            match (*r).checked_add(1) {
                Some(n) => {
                    *r = n;
                },
                None => return Err(TryLockError::WouldBlockRecursive)
            }
        }
        Ok(RecursiveMutexGuard {
            mutex: self,
            marker: PhantomData,
        })
    }
}

impl<'a> Drop for RecursiveMutexGuard<'a> {
    fn drop(&mut self) {
        // We can avoid the assertion here because Rust can statically guarantee
        // we are not running the destructor in the wrong thread.
        unsafe {
            let recur = self.mutex.recursion.get();
            *recur -= 1;
            if *recur == 0 {
                self.mutex.owner.store(0, Ordering::Relaxed);
                mem::transmute::<_,Box<MutexGuard<()>>>(*self.mutex.guard.get());
            }
        }
    }
}
