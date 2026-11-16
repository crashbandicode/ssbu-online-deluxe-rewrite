//! Read-only 13.0.5 current-match service lookup, with an injected memory reader.
//! Mirrors the native Hash40 bucket walk without taking its shared reference.
//! A single traversal has cycle detection; it never retries a changing registry.
const SERVICE_ROOT: usize = 0x52c3910;
const MATCH_KEY: usize = 0x10af089a93;
const HASH_MASK: usize = 0xffffffffff;
const MATCH_VTABLE: usize = 0x5070bf0;
const MATCH_TYPE: usize = 0x15789d0;
const REMAINING_GETTER: usize = 0x157c950;
const PRIMARY_GETTER: usize = 0x157c970;

/// Fresh registry identity, compared again after reading the native accumulator.
/// No pointer in this snapshot is retained between UI samples.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MatchObject {
    pub object: usize,
    root: usize,
    map: usize,
    buckets: usize,
    bucket_count: usize,
    predecessor: usize,
    node: usize,
}

fn bucket(hash: usize, count: usize) -> usize {
    if count.is_power_of_two() {
        hash & (count - 1)
    } else {
        hash % count
    }
}

pub(crate) fn resolve_match_object(
    base: usize,
    mut read: impl FnMut(usize) -> Option<usize>,
) -> Option<MatchObject> {
    let root = read(base.checked_add(SERVICE_ROOT)?)?;
    let map = read(root)?;
    let buckets = read(map)?;
    let bucket_count = read(map.checked_add(8)?)?;
    if bucket_count == 0 {
        return None;
    }
    let selected = bucket(MATCH_KEY, bucket_count);
    let predecessor = read(buckets.checked_add(selected.checked_mul(8)?)?)?;
    let mut node = read(predecessor)?;
    // Brent cycle detection bounds malformed cyclic chains without an arbitrary
    // collision limit. The native bucket-exit condition also ends this one pass.
    let mut anchor = node;
    let mut power = 1usize;
    let mut distance = 0usize;
    while node != 0 {
        let hash = read(node.checked_add(8)?)?;
        if hash != MATCH_KEY && bucket(hash, bucket_count) != selected {
            return None;
        }
        if hash == MATCH_KEY && read(node.checked_add(16)?)? & HASH_MASK == MATCH_KEY {
            let object = read(node.checked_add(24)?)?;
            let table = read(object)?;
            if table != base.checked_add(MATCH_VTABLE)?
                || read(table.checked_add(0x18)?)? != base.checked_add(MATCH_TYPE)?
                || read(table.checked_add(0x128)?)? != base.checked_add(REMAINING_GETTER)?
                || read(table.checked_add(0x138)?)? != base.checked_add(PRIMARY_GETTER)?
            {
                return None;
            }
            return Some(MatchObject {
                object,
                root,
                map,
                buckets,
                bucket_count,
                predecessor,
                node,
            });
        }
        node = read(node)?;
        distance = distance.checked_add(1)?;
        if node == anchor {
            return None;
        }
        if distance == power {
            anchor = node;
            power = power.checked_mul(2)?;
            distance = 0;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    const BASE: usize = 0x80000000;
    const ROOT: usize = 0x100000;
    const MAP: usize = 0x110000;
    const BUCKETS: usize = 0x120000;
    const PREDECESSOR: usize = 0x130000;
    const NODE: usize = 0x140000;
    const OBJECT: usize = 0x150000;

    fn fixture(count: usize) -> BTreeMap<usize, usize> {
        BTreeMap::from([
            (BASE + SERVICE_ROOT, ROOT),
            (ROOT, MAP),
            (MAP, BUCKETS),
            (MAP + 8, count),
            (BUCKETS + bucket(MATCH_KEY, count) * 8, PREDECESSOR),
            (PREDECESSOR, NODE),
            (NODE, 0),
            (NODE + 8, MATCH_KEY),
            (NODE + 16, MATCH_KEY),
            (NODE + 24, OBJECT),
            (OBJECT, BASE + MATCH_VTABLE),
            (BASE + MATCH_VTABLE + 0x18, BASE + MATCH_TYPE),
            (BASE + MATCH_VTABLE + 0x128, BASE + REMAINING_GETTER),
            (BASE + MATCH_VTABLE + 0x138, BASE + PRIMARY_GETTER),
        ])
    }

    #[test]
    fn current_service_resolves_power_of_two_and_prime_buckets_with_collision() {
        for count in [16, 17] {
            let mut memory = fixture(count);
            let collision = NODE + 0x100;
            memory.insert(PREDECESSOR, collision);
            memory.insert(collision, NODE);
            memory.insert(collision + 8, MATCH_KEY + count);
            memory.insert(collision + 16, MATCH_KEY + count);
            assert_eq!(
                resolve_match_object(BASE, |a| memory.get(&a).copied())
                    .unwrap()
                    .object,
                OBJECT
            );
        }
    }

    #[test]
    fn current_service_rejects_missing_memory_empty_table_and_wrong_header() {
        let memory = fixture(16);
        for missing in [ROOT, MAP + 8, PREDECESSOR, NODE + 24, OBJECT] {
            let mut broken = memory.clone();
            broken.remove(&missing);
            assert_eq!(
                resolve_match_object(BASE, |a| broken.get(&a).copied()),
                None
            );
        }
        for changed in [MAP + 8, OBJECT, BASE + MATCH_VTABLE + 0x18] {
            let mut broken = memory.clone();
            broken.insert(changed, 0);
            assert_eq!(
                resolve_match_object(BASE, |a| broken.get(&a).copied()),
                None
            );
        }
    }

    #[test]
    fn current_service_stops_at_next_bucket_and_rejects_a_cyclic_chain() {
        let mut memory = fixture(16);
        memory.insert(NODE + 8, MATCH_KEY + 1);
        assert_eq!(
            resolve_match_object(BASE, |a| memory.get(&a).copied()),
            None
        );
        for length in [1usize, 2, 3, 8] {
            let mut memory = fixture(16);
            for i in 0..length {
                let node = NODE + i * 0x100;
                memory.insert(node, NODE + ((i + 1) % length) * 0x100);
                memory.insert(node + 8, MATCH_KEY + 16);
            }
            assert_eq!(
                resolve_match_object(BASE, |a| memory.get(&a).copied()),
                None
            );
        }
    }

    #[test]
    fn current_service_snapshot_changes_when_current_object_or_registry_changes() {
        let mut memory = fixture(16);
        let before = resolve_match_object(BASE, |a| memory.get(&a).copied()).unwrap();
        memory.insert(NODE + 24, OBJECT + 0x1000);
        memory.insert(OBJECT + 0x1000, BASE + MATCH_VTABLE);
        let after = resolve_match_object(BASE, |a| memory.get(&a).copied()).unwrap();
        assert_ne!(before, after);
        memory.insert(NODE + 24, OBJECT);
        memory.insert(BASE + SERVICE_ROOT, ROOT + 8);
        memory.insert(ROOT + 8, MAP);
        let rebound = resolve_match_object(BASE, |a| memory.get(&a).copied()).unwrap();
        assert_ne!(before, rebound);
        assert_eq!(before.object, rebound.object);
    }
}
