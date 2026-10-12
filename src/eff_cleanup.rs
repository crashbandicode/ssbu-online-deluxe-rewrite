//! Per-thread record of one sibling cleanup that was actually attempted.
//! The key is the accessor, the original hash, and kill versus detach.
//! A different key still runs. Dropping the guard restores the previous key.
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};

pub const CLEANUP_KILL: u8 = 1;
pub const CLEANUP_DETACH: u8 = 2;
pub const ROUTE_SLOTS: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CleanupKey {
    accessor: usize,
    hash: u64,
    kind: u8,
}

thread_local! {
    // Skyline uses the OS-key macro, whose generated initializer hides this
    // existing const block from Clippy. Keep the exception on this key only.
    #[cfg_attr(target_os = "switch", allow(clippy::missing_const_for_thread_local))]
    static DELIVERED: Cell<Option<CleanupKey>> = const { Cell::new(None) };
}

/// Restores the key this attempt replaced, including a nested attempt.
pub struct DeliveredCleanup {
    previous: Option<CleanupKey>,
}

impl DeliveredCleanup {
    fn arm(accessor: usize, hash: u64, kind: u8) -> Self {
        let previous = DELIVERED.with(|slot| {
            slot.replace(Some(CleanupKey {
                accessor,
                hash,
                kind,
            }))
        });
        Self { previous }
    }
}

impl Drop for DeliveredCleanup {
    fn drop(&mut self) {
        let previous = self.previous;
        DELIVERED.with(|slot| slot.set(previous));
    }
}

/// Arm only for a real clone call on a real accessor. A missing alias or a
/// null accessor leaves the current key untouched.
pub fn begin_alias(
    accessor: usize,
    hash: u64,
    kind: u8,
    alias: Option<u64>,
) -> Option<DeliveredCleanup> {
    if accessor == 0 || alias.is_none() {
        None
    } else {
        Some(DeliveredCleanup::arm(accessor, hash, kind))
    }
}

pub fn delivered_matches(accessor: usize, hash: u64, kind: u8) -> bool {
    DELIVERED.with(|slot| {
        slot.get()
            == Some(CleanupKey {
                accessor,
                hash,
                kind,
            })
    })
}

pub fn route_index(layer: u64, nested: bool) -> Option<usize> {
    let layer = usize::try_from(layer).ok()?;
    if !(1..=4).contains(&layer) {
        return None;
    }
    Some((layer - 1) * 2 + usize::from(nested))
}

/// First event and every 128th for this layer and nested bit only.
pub fn take_route_sample(counts: &[AtomicU64; ROUTE_SLOTS], layer: u64, nested: bool) -> bool {
    let Some(slot) = route_index(layer, nested) else {
        return false;
    };
    let count = counts[slot].fetch_add(1, Ordering::Relaxed) + 1;
    count == 1 || count.is_multiple_of(128)
}

#[cfg(test)]
mod tests {
    use super::{begin_alias, delivered_matches, take_route_sample, CLEANUP_DETACH, CLEANUP_KILL};
    use std::sync::atomic::AtomicU64;

    #[test]
    fn unrelated_key_is_not_suppressed_and_nested_key_restores() {
        assert!(begin_alias(1, 10, CLEANUP_KILL, None).is_none());
        assert!(begin_alias(0, 10, CLEANUP_KILL, Some(99)).is_none());
        assert!(!delivered_matches(1, 10, CLEANUP_KILL));
        let outer = begin_alias(1, 10, CLEANUP_KILL, Some(11)).unwrap();
        assert!(delivered_matches(1, 10, CLEANUP_KILL));
        assert!(!delivered_matches(1, 12, CLEANUP_KILL));
        assert!(!delivered_matches(2, 10, CLEANUP_KILL));
        assert!(!delivered_matches(1, 10, CLEANUP_DETACH));
        {
            let inner = begin_alias(2, 20, CLEANUP_DETACH, Some(21)).unwrap();
            assert!(delivered_matches(2, 20, CLEANUP_DETACH));
            assert!(!delivered_matches(1, 10, CLEANUP_KILL));
            drop(inner);
        }
        assert!(delivered_matches(1, 10, CLEANUP_KILL));
        drop(outer);
        assert!(!delivered_matches(1, 10, CLEANUP_KILL));
    }

    #[test]
    fn delivered_key_is_per_thread() {
        let outer = begin_alias(1, 10, CLEANUP_KILL, Some(11)).unwrap();
        let joined = std::thread::spawn(|| {
            assert!(!delivered_matches(1, 10, CLEANUP_KILL));
            let local = begin_alias(3, 30, CLEANUP_KILL, Some(31)).unwrap();
            assert!(delivered_matches(3, 30, CLEANUP_KILL));
            drop(local);
            assert!(!delivered_matches(3, 30, CLEANUP_KILL));
        })
        .join();
        assert!(joined.is_ok());
        assert!(delivered_matches(1, 10, CLEANUP_KILL));
        drop(outer);
    }

    #[test]
    fn each_route_category_keeps_its_own_first_sample() {
        let counts: [AtomicU64; 8] = std::array::from_fn(|_| AtomicU64::new(0));
        assert!(take_route_sample(&counts, 1, false));
        assert!(!take_route_sample(&counts, 1, false));
        assert!(take_route_sample(&counts, 1, true));
        assert!(!take_route_sample(&counts, 0, false));
        assert!(!take_route_sample(&counts, 5, true));
        assert!(take_route_sample(&counts, 2, false));
        for _ in 0..126 {
            assert!(!take_route_sample(&counts, 2, false));
        }
        assert!(take_route_sample(&counts, 2, false));
        assert!(take_route_sample(&counts, 3, false));
        assert!(take_route_sample(&counts, 4, true));
    }
}
