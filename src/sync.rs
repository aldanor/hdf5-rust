/*
  (C) Joshua Yanovski (@pythonesque)

  https://gist.github.com/pythonesque/5bdf071d3617b61b3fed
*/

#![allow(dead_code)]

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
enum LockError {
    /// Mutex was poisoned,
    Poisoned,
    /// Mutex would block due to exceeded recursion limits.
    WouldBlockRecursive,
}

#[derive(Debug)]
enum TryLockError {
    /// Mutex was poisoned
    Poisoned,
    /// Mutex would block because it is taken by another thread.
    WouldBlockExclusive,
    /// Mutex would block due to exceeded recursion limits.
    WouldBlockRecursive,
}

struct RecursiveMutex {
    owner: AtomicUsize,
    recursion: UnsafeCell<u64>,
    mutex: Mutex<()>,
    guard: UnsafeCell<*mut MutexGuard<'static, ()>>,
}

unsafe impl Sync for RecursiveMutex {}
unsafe impl Send for RecursiveMutex {}

#[must_use]
struct RecursiveMutexGuard<'a> {
    mutex: &'a RecursiveMutex,
    marker: PhantomData<*mut ()>,  // !Send
}

// Cannot implement Send because we rely on the guard being dropped in the
// same thread (otherwise we can't use Relaxed).  We might be able to allow
// it with Acquire / Release?

impl RecursiveMutex {
    fn new() -> RecursiveMutex {
        RecursiveMutex {
            owner: AtomicUsize::new(0),
            recursion: UnsafeCell::new(0),
            mutex: Mutex::new(()),
            guard: UnsafeCell::new(0 as *mut _),
        }
    }

    fn get_thread_id(&self) -> usize {
        THREAD_ID.with(|x| x as *const _ as usize)
    }

    fn is_same_thread(&self) -> bool {
        self.get_thread_id() == self.owner.load(Ordering::Relaxed)
    }

    fn store_thread_id(&self, guard: MutexGuard<()>) {
        unsafe {
            let tid = self.get_thread_id();
            self.owner.store(tid, Ordering::Relaxed);
            *self.guard.get() = mem::transmute(Box::new(guard));
        }
    }

    fn check_recursion(&self) -> Option<RecursiveMutexGuard> {
        unsafe {
            let recursion = self.recursion.get();
            if let Some(n) = (*recursion).checked_add(1) {
                *recursion = n;
                Some(RecursiveMutexGuard { mutex: self, marker: PhantomData })
            } else {
                None
            }
        }
    }

    pub fn lock(&'static self) -> Result<RecursiveMutexGuard, LockError> {
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
        if !self.is_same_thread() {
            match self.mutex.lock() {
                Ok(guard) => self.store_thread_id(guard),
                Err(_)    => return Err(LockError::Poisoned),
            }
        }
        match self.check_recursion() {
            Some(guard) => Ok(guard),
            None        => Err(LockError::WouldBlockRecursive),
        }
    }

    #[allow(dead_code)]
    fn try_lock(&'static self) -> Result<RecursiveMutexGuard, TryLockError> {
        // Same reasoning as in lock().
        if !self.is_same_thread() {
            match self.mutex.try_lock() {
                Ok(guard)                            => self.store_thread_id(guard),
                Err(sync::TryLockError::Poisoned(_)) => return Err(TryLockError::Poisoned),
                Err(sync::TryLockError::WouldBlock)  => return Err(TryLockError::WouldBlockExclusive),
            }
        }
        match self.check_recursion() {
            Some(guard) => Ok(guard),
            None        => Err(TryLockError::WouldBlockRecursive),
        }
    }
}

impl<'a> Drop for RecursiveMutexGuard<'a> {
    fn drop(&mut self) {
        // We can avoid the assertion here because Rust can statically guarantee
        // we are not running the destructor in the wrong thread.
        unsafe {
            let recursion = self.mutex.recursion.get();
            *recursion -= 1;
            if *recursion == 0 {
                self.mutex.owner.store(0, Ordering::Relaxed);
                mem::transmute::<_,Box<MutexGuard<()>>>(*self.mutex.guard.get());
            }
        }
    }
}

/// Guards the execution of the provided closure with a recursive static mutex.
pub fn sync<T, F>(func: F) -> T where F: FnOnce() -> T,
{
    use remutex::ReentrantMutex;
    lazy_static! {
        static ref LOCK: ReentrantMutex<()> = ReentrantMutex::new(());
    }
    let _guard = LOCK.lock();
    func()
}

#[cfg(test)]
mod tests {
    use super::RecursiveMutex;

    #[test]
    pub fn test_recursive_mutex() {
        lazy_static! {
            static ref LOCK: RecursiveMutex = RecursiveMutex::new();
        }
        let _g1 = LOCK.try_lock();
        let _g2 = LOCK.lock();
        let _g3 = LOCK.try_lock();
        let _g4 = LOCK.lock();
    }
}
