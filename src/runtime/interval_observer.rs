//! Read-only, one-time SDK interval observations for the opt-in interval experiment.
//! The signatures and object layout are specific to 13.0.5's SDK. Unknown
//! mappings fail closed. No pointers survive WindowInitialize, and no Surface
//! field, callback, pacing credit, or wait is changed here.
use super::memory;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

struct Observation {
    valid: AtomicBool,
    values: [AtomicI32; 5],
}
impl Observation {
    const fn new() -> Self {
        Self {
            valid: AtomicBool::new(false),
            values: [const { AtomicI32::new(-1) }; 5],
        }
    }
}
static BEFORE: Observation = Observation::new();
static AFTER: Observation = Observation::new();

unsafe fn pointer(address: usize) -> Option<usize> {
    if address % 8 != 0 || !memory::accessible(address, 8, false) {
        return None;
    }
    let value = *(address as *const usize);
    (value >= 0x10000 && value % 8 == 0).then_some(value)
}
unsafe fn integer(base: usize, offset: usize) -> Option<i32> {
    let address = base.checked_add(offset)?;
    if address % 4 != 0 || !memory::accessible(address, 4, false) {
        return None;
    }
    Some(*(address as *const i32))
}
unsafe fn signature(address: usize, words: &[u32]) -> bool {
    address % 4 == 0
        && memory::executable(address, words.len() * 4)
        && words
            .iter()
            .enumerate()
            .all(|(i, expected)| *((address + i * 4) as *const u32) == *expected)
}

const SIGNATURES: &[(usize, &[u32])] = &[
    (
        0x429d60,
        &[
            0xa9be4ff4, 0xa9017bfd, 0x910043fd, 0xb9404008, 0x6b01011f, 0x54000120, 0xaa0003f4,
            0xf9402400, 0xb40000c0, 0xf9404008, 0xb4000088, 0x2a0103f3, 0xd63f0100, 0xb9004293,
            0xa9417bfd, 0xa8c24ff4, 0xd65f03c0,
        ],
    ),
    (0x40e480, &[0xf940b800, 0xf9404002, 0xd61f0040]),
    (
        0x40fc00,
        &[
            0xb940f408, 0x6b01011f, 0x54000100, 0xa9bf7bfd, 0x910003fd, 0xb900f401, 0xf9406c00,
            0xf9403808, 0xd63f0100, 0xa8c17bfd, 0x2a1f03e0, 0xd65f03c0,
        ],
    ),
    (
        0x269b0,
        &[
            0xd1004008, 0xf100001f, 0x9a8803e0, 0xf9400008, 0xf9402d02, 0xd61f0040,
        ],
    ),
    (
        0x26e80,
        &[
            0x2949a408, 0x6b01011f, 0x1a81c108, 0x6b09011f, 0x1a88c128, 0xb90f2408, 0x2a1f03e0,
            0xd65f03c0,
        ],
    ),
    (0x429e50, &[0xb9404000, 0xd65f03c0]),
    (0x27360, &[0xb94f26ab]),
    (0x27380, &[0xb9002feb]),
    (0x26908, &[0xf804c268]),
];

unsafe fn snapshot(window: usize, setter: usize) -> Option<[i32; 5]> {
    let base = setter.checked_sub(0x429d60)?;
    // Derive the candidate base from the resolved setter, then require the
    // executable mapping itself to own this exact SDK text base.
    let (mapped_base, _) = memory::region(setter)?;
    if mapped_base != base
        || !SIGNATURES.iter().all(|(offset, words)| {
            base.checked_add(*offset)
                .is_some_and(|p| signature(p, words))
        })
    {
        return None;
    }
    let a = pointer(window.checked_add(0x48)?)?;
    let b = pointer(a.checked_add(0x170)?)?;
    let n = pointer(b.checked_add(0xd8)?)?;
    let surface = n.checked_sub(0x10)?;
    let vtable = pointer(surface)?;
    for (object, slot, target) in [
        (a, 0x80, 0x40e480),
        (b, 0x80, 0x40fc00),
        (n, 0x70, 0x269b0),
        (vtable, 0x58, 0x26e80),
    ] {
        if pointer(object.checked_add(slot)?)? != base.checked_add(target)? {
            return None;
        }
    }
    let values = [
        integer(window, 0x40)?,
        integer(b, 0xf4)?,
        integer(surface, 0x4c)?,
        integer(surface, 0x50)?,
        integer(surface, 0xf24)?,
    ];
    (values[2] <= values[3]).then_some(values)
}

/// Called only while the original initialization call owns a live window.
pub(super) unsafe fn observe(window: usize, setter: usize, after: bool) {
    let observation = if after { &AFTER } else { &BEFORE };
    observation.valid.store(false, Ordering::Release);
    if let Some(values) = snapshot(window, setter) {
        for (slot, value) in observation.values.iter().zip(values) {
            slot.store(value, Ordering::Relaxed);
        }
        observation.valid.store(true, Ordering::Release);
    }
}

/// Initialization snapshots, not a live scanout or queue-depth measurement.
pub(super) fn write_status(out: &mut impl Write) -> io::Result<()> {
    for (label, observation) in [("before", &BEFORE), ("after", &AFTER)] {
        let valid = observation.valid.load(Ordering::Acquire);
        writeln!(out, "research_interval_surface_{label}_valid={valid}")?;
        if valid {
            for (name, value) in [
                "window_cached",
                "provider_cached",
                "minimum",
                "maximum",
                "effective",
            ]
            .iter()
            .zip(&observation.values)
            {
                writeln!(
                    out,
                    "research_interval_surface_{label}_{name}={}",
                    value.load(Ordering::Relaxed)
                )?;
            }
        }
    }
    Ok(())
}
