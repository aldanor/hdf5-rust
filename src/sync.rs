use lazy_static::lazy_static;
use parking_lot::ReentrantMutex;

/// Guards the execution of the provided closure with a recursive static mutex.
pub fn sync<T, F>(func: F) -> T
where
    F: FnOnce() -> T,
{
    lazy_static! {
        static ref LOCK: ReentrantMutex<()> = ReentrantMutex::new(());
    }
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
}
