//! The one way a game hook takes a mutex the diagnostics thread or another
//! plugin's thread can hold: try once, never wait (ssbu-mod-playbook WORKFLOW,
//! "Hooks never wait on a plugin thread"). A busy lock returns `None`, the
//! caller leaves its work for its next call, and the miss is counted for the
//! status file. A poisoned lock is still taken: its data stays usable, as with
//! the `into_inner` the blocking callers use.
//!
//! `tests/hook_locks.rs` checks that no hook reaches a blocking `.lock()`.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, TryLockError};

static SKIPPED: AtomicU64 = AtomicU64::new(0);

/// The guard, or `None` while another thread holds the lock.
pub fn try_take<T>(lock: &Mutex<T>) -> Option<MutexGuard<'_, T>> {
    match lock.try_lock() {
        Ok(guard) => Some(guard),
        Err(TryLockError::Poisoned(poisoned)) => Some(poisoned.into_inner()),
        Err(TryLockError::WouldBlock) => {
            SKIPPED.fetch_add(1, Ordering::Relaxed);
            None
        }
    }
}

/// Hook-side takes that found their lock busy since boot.
pub fn skipped() -> u64 {
    SKIPPED.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_busy_lock_is_skipped_and_counted_and_a_free_one_is_taken() {
        static LOCK: Mutex<u32> = Mutex::new(7);
        let before = skipped();
        let held = LOCK.lock().unwrap();
        assert!(std::thread::scope(|s| s.spawn(|| try_take(&LOCK).is_none()).join().unwrap()));
        assert!(skipped() > before);
        drop(held);
        assert_eq!(try_take(&LOCK).map(|g| *g), Some(7));
    }

    #[test]
    fn a_poisoned_lock_is_still_taken() {
        static LOCK: Mutex<u32> = Mutex::new(3);
        let _ = std::thread::spawn(|| {
            let _guard = LOCK.lock().unwrap();
            panic!("poison");
        })
        .join();
        assert!(LOCK.is_poisoned());
        assert_eq!(try_take(&LOCK).map(|g| *g), Some(3));
    }
}
