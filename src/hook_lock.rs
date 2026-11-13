//! Hook-safe shared state (ssbu-mod-playbook WORKFLOW, "Hooks never wait on a
//! plugin thread"). Nothing here blocks a game hook, and nothing allocates:
//!
//! - [`TryLock`]: a mutex made of one atomic flag. Hooks only try it
//!   ([`try_take`] counts a busy try for the status file); plugin threads may
//!   [`TryLock::lock`], which retries and sleeps between tries. Its API is
//!   `std::sync::Mutex`'s (`lock` gives a `LockResult`, `try_lock` a
//!   `TryLockResult`), so a static changes type without its callers changing.
//!   Unlike Horizon's std mutex it is not boxed on first use, so taking it
//!   never reaches the allocator.
//! - [`SnapCell`]: a small `Copy` value that game threads update with a
//!   compare-and-swap of whole copies. A thread preempted mid-update holds
//!   nothing the others need; an update retries only because another one
//!   succeeded.
//!
//! `tests/hook_locks.rs` checks the source (no hook reaches a blocking
//! `.lock()`), `tools/hook_wait_check.py` the built ELF (no hook reaches a
//! wait; no per-frame hook reaches the allocator).

use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem::{size_of, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{fence, AtomicBool, AtomicU64, Ordering};
use std::sync::{LockResult, TryLockError, TryLockResult};

static SKIPPED: AtomicU64 = AtomicU64::new(0);

/// A mutex no hook waits on: one flag, taken by compare-and-swap.
pub struct TryLock<T> {
    taken: AtomicBool,
    value: UnsafeCell<T>,
}

// SAFETY: the flag gives one guard at a time, as a mutex does.
unsafe impl<T: Send> Sync for TryLock<T> {}
unsafe impl<T: Send> Send for TryLock<T> {}

pub struct TryGuard<'a, T> {
    lock: &'a TryLock<T>,
}

impl<T> TryLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            taken: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    fn take(&self) -> Option<TryGuard<'_, T>> {
        self.taken
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
            // Built only on success: a guard dropped on a failed take would
            // release another holder's flag.
            .then(|| TryGuard { lock: self })
    }

    /// Any thread, hooks included: the guard, or `WouldBlock` while another
    /// thread holds it. Never `Poisoned`.
    pub fn try_lock(&self) -> TryLockResult<TryGuard<'_, T>> {
        self.take().ok_or(TryLockError::WouldBlock)
    }

    /// Plugin threads only: tries until free, yielding and then sleeping a
    /// millisecond between tries. Never `Err`.
    pub fn lock(&self) -> LockResult<TryGuard<'_, T>> {
        let mut tries = 0u32;
        loop {
            if let Some(guard) = self.take() {
                return Ok(guard);
            }
            tries = tries.saturating_add(1);
            if tries < 64 {
                std::thread::yield_now();
            } else {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    }
}

impl<T> Deref for TryGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: the guard is the only holder of the flag.
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for TryGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: the guard is the only holder of the flag.
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for TryGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.taken.store(false, Ordering::Release);
    }
}

/// The guard, or `None` while another thread holds the lock: the hook leaves
/// its work for its next call and the miss is counted for the status file.
pub fn try_take<T>(lock: &TryLock<T>) -> Option<TryGuard<'_, T>> {
    let guard = lock.take();
    if guard.is_none() {
        note_skipped();
    }
    guard
}

/// A hook that left work undone because shared state was busy.
pub fn note_skipped() {
    SKIPPED.fetch_add(1, Ordering::Relaxed);
}

/// Hook-side takes and updates that found their state busy since boot.
pub fn skipped() -> u64 {
    SKIPPED.load(Ordering::Relaxed)
}

/// Copies a [`SnapCell`] keeps: more than the threads that can update it at
/// once (the game's main, render and task worker threads, and plugin threads),
/// so a free one is always there.
const SLOTS: usize = 16;
const READ_TRIES: usize = 64;
const UPDATE_TRIES: usize = 256;

/// A `Copy` value of at most `W` words that any thread reads and updates
/// without a lock. Each update copies the current value, changes the copy in
/// a free slot and publishes it with one compare-and-swap of `current`; reads
/// check the slot's sequence count around their copy.
///
/// An update closure may run more than once (when another update wins the
/// race), so it must keep its effects in the value; the caller acts on the
/// result after `update` returns. `read` and `update` give `None` only after
/// [`READ_TRIES`] / [`UPDATE_TRIES`] lost races in a row, or with every slot
/// taken; both are counted in [`skipped`].
pub struct SnapCell<T: Copy, const W: usize> {
    initial: T,
    /// 0 until the first update; then `version << 8 | slot`.
    current: AtomicU64,
    claimed: [AtomicBool; SLOTS],
    seq: [AtomicU64; SLOTS],
    words: [[AtomicU64; W]; SLOTS],
    _value: PhantomData<T>,
}

// SAFETY: values move between threads only as copies of their words.
unsafe impl<T: Copy + Send, const W: usize> Sync for SnapCell<T, W> {}
unsafe impl<T: Copy + Send, const W: usize> Send for SnapCell<T, W> {}

impl<T: Copy, const W: usize> SnapCell<T, W> {
    pub const fn new(initial: T) -> Self {
        assert!(size_of::<T>() <= W * 8, "SnapCell: W words are too few for T");
        Self {
            initial,
            current: AtomicU64::new(0),
            claimed: [const { AtomicBool::new(false) }; SLOTS],
            seq: [const { AtomicU64::new(0) }; SLOTS],
            words: [const { [const { AtomicU64::new(0) }; W] }; SLOTS],
            _value: PhantomData,
        }
    }

    /// The value published as `current`, if one read of it was consistent.
    fn read_at(&self, current: u64) -> Option<T> {
        if current == 0 {
            return Some(self.initial);
        }
        let slot = (current & 0xff) as usize;
        let before = self.seq[slot].load(Ordering::Acquire);
        if before & 1 == 1 {
            return None;
        }
        let mut buf = [0u64; W];
        for (word, cell) in buf.iter_mut().zip(&self.words[slot]) {
            *word = cell.load(Ordering::Relaxed);
        }
        fence(Ordering::Acquire);
        if self.seq[slot].load(Ordering::Relaxed) != before
            || self.current.load(Ordering::Relaxed) != current
        {
            return None;
        }
        let mut value = MaybeUninit::<T>::uninit();
        // SAFETY: `buf` holds the bytes of a `T` that `write_slot` stored and
        // nothing changed while they were copied (the checks above).
        unsafe {
            std::ptr::copy_nonoverlapping(
                buf.as_ptr().cast::<u8>(),
                value.as_mut_ptr().cast::<u8>(),
                size_of::<T>(),
            );
            Some(value.assume_init())
        }
    }

    fn write_slot(&self, slot: usize, value: &T) {
        let mut buf = [0u64; W];
        // SAFETY: `buf` is at least `size_of::<T>()` bytes (asserted in `new`).
        unsafe {
            std::ptr::copy_nonoverlapping(
                (value as *const T).cast::<u8>(),
                buf.as_mut_ptr().cast::<u8>(),
                size_of::<T>(),
            );
        }
        let seq = self.seq[slot].load(Ordering::Relaxed);
        self.seq[slot].store(seq.wrapping_add(1), Ordering::Relaxed);
        fence(Ordering::Release);
        for (cell, word) in self.words[slot].iter().zip(buf) {
            cell.store(word, Ordering::Relaxed);
        }
        self.seq[slot].store(seq.wrapping_add(2), Ordering::Release);
    }

    pub fn read(&self) -> Option<T> {
        for _ in 0..READ_TRIES {
            if let Some(value) = self.read_at(self.current.load(Ordering::Acquire)) {
                return Some(value);
            }
            std::hint::spin_loop();
        }
        note_skipped();
        None
    }

    pub fn update<R>(&self, mut change: impl FnMut(&mut T) -> R) -> Option<R> {
        // One free slot for every attempt of this update.
        let Some(slot) = (0..SLOTS).find(|&slot| {
            self.claimed[slot]
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
        }) else {
            note_skipped();
            return None;
        };
        for attempt in 0..UPDATE_TRIES {
            let current = self.current.load(Ordering::Acquire);
            if let Some(mut next) = self.read_at(current) {
                let out = change(&mut next);
                self.write_slot(slot, &next);
                let version = (current >> 8).wrapping_add(1).max(1);
                if self
                    .current
                    .compare_exchange(current, version << 8 | slot as u64, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
                {
                    if current != 0 {
                        self.claimed[(current & 0xff) as usize].store(false, Ordering::Release);
                    }
                    return Some(out);
                }
            }
            // Another update won; a short pause before the next attempt keeps
            // racing updaters from failing each other in lockstep.
            for _ in 0..(1u32 << attempt.min(8)) {
                std::hint::spin_loop();
            }
        }
        self.claimed[slot].store(false, Ordering::Release);
        note_skipped();
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_busy_lock_is_skipped_and_counted_and_a_free_one_is_taken() {
        static LOCK: TryLock<u32> = TryLock::new(7);
        let before = skipped();
        let held = LOCK.lock().unwrap();
        assert!(std::thread::scope(|s| s.spawn(|| try_take(&LOCK).is_none()).join().unwrap()));
        assert!(matches!(LOCK.try_lock(), Err(TryLockError::WouldBlock)));
        assert!(skipped() > before);
        drop(held);
        assert_eq!(try_take(&LOCK).map(|g| *g), Some(7));
    }

    #[test]
    fn a_plugin_thread_lock_waits_for_the_holder_and_excludes() {
        static LOCK: TryLock<u64> = TryLock::new(0);
        std::thread::scope(|s| {
            for _ in 0..4 {
                s.spawn(|| {
                    for _ in 0..2_000 {
                        let mut guard = LOCK.lock().unwrap();
                        let seen = *guard;
                        std::hint::spin_loop();
                        *guard = seen + 1;
                    }
                });
            }
        });
        assert_eq!(*LOCK.lock().unwrap(), 8_000);
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Pair {
        a: u64,
        b: u64,
        tag: u8,
    }

    #[test]
    fn a_cell_starts_at_its_initial_value_and_keeps_each_update() {
        let cell: SnapCell<Pair, 3> = SnapCell::new(Pair { a: 1, b: 2, tag: 3 });
        assert_eq!(cell.read(), Some(Pair { a: 1, b: 2, tag: 3 }));
        assert_eq!(cell.update(|p| {
            p.a += 10;
            p.a
        }), Some(11));
        assert_eq!(cell.read(), Some(Pair { a: 11, b: 2, tag: 3 }));
    }

    #[test]
    fn concurrent_updates_lose_nothing_and_reads_never_tear() {
        static CELL: SnapCell<Pair, 3> = SnapCell::new(Pair { a: 0, b: 0, tag: 0 });
        std::thread::scope(|s| {
            for _ in 0..8 {
                s.spawn(|| {
                    for _ in 0..5_000 {
                        CELL.update(|p| {
                            p.a += 1;
                            p.b += 2;
                        })
                        .expect("an update gave up");
                    }
                });
            }
            s.spawn(|| {
                for _ in 0..20_000 {
                    if let Some(p) = CELL.read() {
                        assert_eq!(p.b, p.a * 2, "torn read");
                    }
                }
            });
        });
        assert_eq!(CELL.read(), Some(Pair { a: 40_000, b: 80_000, tag: 0 }));
    }

    #[test]
    fn a_stalled_updater_holds_nothing_the_others_need() {
        // One thread claims a slot and stops before publishing (as a thread
        // preempted mid-update would); every other update still completes.
        static CELL: SnapCell<u64, 1> = SnapCell::new(0);
        let stalled = (0..SLOTS)
            .find(|&slot| CELL.claimed[slot].compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok())
            .unwrap();
        for _ in 0..100 {
            assert!(CELL.update(|v| *v += 1).is_some());
        }
        assert_eq!(CELL.read(), Some(100));
        CELL.claimed[stalled].store(false, Ordering::Release);
    }
}
