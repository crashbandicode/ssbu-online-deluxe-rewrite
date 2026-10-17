//! Switch side of [`crate::clock_probe`]: the Horizon-OC file check at boot
//! and the dependent-add calibration loop, sampled on the diagnostics thread
//! every 5 s. Hooks never call this.
use super::{record, Code, DATA_DIR};
use crate::clock_probe::{classify, Class, HocState, ADDS_PER_ITERATION, ITERATIONS_PER_SAMPLE};
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};

static MHZ: AtomicU32 = AtomicU32::new(0);
static HOC_BITS: AtomicU64 = AtomicU64::new(0);
static LAST_CLASS: AtomicU8 = AtomicU8::new(0);
const SAMPLE_EVERY: u64 = 10; // diagnostics iterations (500 ms each)
const LOG_EVERY: u64 = 120; // one unconditional event per minute

const HOC_KIP: &str = "sd:/atmosphere/kips/hoc.kip";
const HOC_CLK_FLAG: &str = "sd:/atmosphere/contents/00FF0000636C6BFF/flags/boot2.flag";
const HOC_CONFIG: &str = "sd:/config/horizon-oc/config.ini";

pub(super) fn mhz() -> u32 {
    MHZ.load(Ordering::Relaxed)
}

pub(super) fn hoc() -> HocState {
    HocState::from_bits(HOC_BITS.load(Ordering::Relaxed))
}

/// Boot: read the Horizon-OC files, take the first sample, log code 111.
pub(super) fn install() {
    let (title_profile, uncapped) = std::fs::read_to_string(HOC_CONFIG)
        .map(|text| crate::clock_probe::parse_hoc_config(&text))
        .unwrap_or((false, false));
    let state = HocState {
        kip: std::path::Path::new(HOC_KIP).exists(),
        autostart: std::path::Path::new(HOC_CLK_FLAG).exists(),
        title_profile,
        uncapped,
    };
    HOC_BITS.store(state.bits(), Ordering::Relaxed);
    let mhz = sample();
    LAST_CLASS.store(classify(mhz) as u8, Ordering::Relaxed);
    record(Code::ClockSample, u64::from(mhz), state.bits());
    skyline::println!(
        "[Deluxe] clock probe: {} (HOC {})",
        crate::clock_probe::status_label(mhz, state),
        DATA_DIR
    );
}

/// Diagnostics thread, every 500 ms iteration.
pub(super) fn poll(iteration: u64) {
    if !iteration.is_multiple_of(SAMPLE_EVERY) {
        return;
    }
    let mhz = sample();
    let class = classify(mhz) as u8;
    let changed = LAST_CLASS.swap(class, Ordering::Relaxed) != class;
    if changed || iteration.is_multiple_of(LOG_EVERY) {
        record(
            Code::ClockSample,
            u64::from(mhz),
            HOC_BITS.load(Ordering::Relaxed),
        );
    }
}

/// Best of four runs (preemption only lengthens a run, never shortens it).
fn sample() -> u32 {
    let cycles = ADDS_PER_ITERATION * ITERATIONS_PER_SAMPLE;
    let mut best = u64::MAX;
    for _ in 0..4 {
        let start = unsafe { skyline::nn::os::GetSystemTick() as u64 };
        let acc = dependent_adds(ITERATIONS_PER_SAMPLE);
        let end = unsafe { skyline::nn::os::GetSystemTick() as u64 };
        std::hint::black_box(acc);
        best = best.min(end.saturating_sub(start));
    }
    let mhz = crate::clock_probe::mhz_from_sample(cycles, best, crate::capture::TICK_HZ);
    MHZ.store(mhz, Ordering::Relaxed);
    mhz
}

/// `iterations` loops of `ADDS_PER_ITERATION` dependent adds: one cycle per
/// add on the Cortex-A57, so the loop runs `32 * iterations` cycles plus one
/// predicted branch per iteration.
#[cfg(target_arch = "aarch64")]
fn dependent_adds(iterations: u64) -> u64 {
    let mut acc: u64 = 0;
    let mut n = iterations;
    unsafe {
        core::arch::asm!(
            "2:",
            "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}",
            "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}",
            "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}",
            "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}",
            "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}",
            "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}",
            "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}",
            "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}", "add {acc}, {acc}, {one}",
            "subs {n}, {n}, #1",
            "b.ne 2b",
            acc = inout(reg) acc,
            n = inout(reg) n,
            one = in(reg) 1u64,
            options(nomem, nostack),
        );
    }
    let _ = n;
    acc
}

#[cfg(not(target_arch = "aarch64"))]
fn dependent_adds(iterations: u64) -> u64 {
    let mut acc = 0u64;
    for _ in 0..iterations {
        for _ in 0..ADDS_PER_ITERATION {
            acc = std::hint::black_box(acc + 1);
        }
    }
    acc
}

const _: () = assert!(Class::Unknown as u8 == 0);
