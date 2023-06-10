use std::sync::atomic::{AtomicBool, Ordering};

use lazy_static::lazy_static;
use parking_lot::ReentrantMutex;

thread_local! {
    pub static SILENCED: AtomicBool = AtomicBool::new(false);
}

lazy_static! {
    pub(crate) static ref LIBRARY_INIT: () = {
        // No functions called here must try to create the LOCK,
        // as this could cause a deadlock in initialisation
        unsafe {
            // Ensure hdf5 does not invalidate handles which might
            // still be live on other threads on program exit
            ::hdf5_sys::h5::H5dont_atexit();
            ::hdf5_sys::h5::H5open();
            // Ignore errors on stdout
            crate::error::silence_errors_no_sync(true);
            // Register filters lzf/blosc if available
            crate::hl::filters::register_filters();
        }
    };
}

/// Guards the execution of the provided closure with a recursive static mutex.
pub fn sync<T, F>(func: F) -> T
where
    F: FnOnce() -> T,
{
    lazy_static! {
        static ref LOCK: ReentrantMutex<()> = {
            lazy_static::initialize(&LIBRARY_INIT);
            ReentrantMutex::new(())
        };
    }
    SILENCED.with(|silence| {
        let is_silenced = silence.load(Ordering::Acquire);
        if !is_silenced {
            let _guard = LOCK.lock();
            unsafe {
                crate::error::silence_errors_no_sync(true);
            }
            silence.store(true, Ordering::Release);
        }
    });
    let _guard = LOCK.lock();
    func()
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use parking_lot::ReentrantMutex;

    #[test]
    pub fn test_reentrant_mutex() {
        lazy_static! {
            static ref LOCK: ReentrantMutex<()> = ReentrantMutex::new(());
        }
        let g1 = LOCK.try_lock();
        assert!(g1.is_some());
        let g2 = LOCK.lock();
        assert_eq!(*g2, ());
        let g3 = LOCK.try_lock();
        assert!(g3.is_some());
        let g4 = LOCK.lock();
        assert_eq!(*g4, ());
    }

    #[test]
    // Test for locking behaviour on initialisation
    pub fn lock_part1() {
        let _ = *crate::globals::H5P_ROOT;
    }

    #[test]
    // Test for locking behaviour on initialisation
    pub fn lock_part2() {
        let _ = h5call!(*crate::globals::H5P_ROOT);
    }
}
