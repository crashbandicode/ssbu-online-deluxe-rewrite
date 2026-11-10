//! Diagnostic build only: the label feed (`crate::label_feed`). The file is
//! read once at boot; the frame code only reads the parsed feed.
use super::{record, Code, DATA_DIR};
use crate::label_feed::{self, Feed};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        OnceLock,
    },
};

static FEED: OnceLock<Option<Feed>> = OnceLock::new();
/// System tick when the opponent's label first showed; 0 before that.
static STARTED: AtomicU64 = AtomicU64::new(0);
static SHOWN: AtomicUsize = AtomicUsize::new(usize::MAX);
/// `nn::os::GetSystemTick` ticks per millisecond (19.2 MHz).
const TICKS_PER_MS: u64 = 19_200;

pub(super) fn load() {
    let text = std::fs::read_to_string(Path::new(DATA_DIR).join("diag.txt")).unwrap_or_default();
    let _ = FEED.set(label_feed::parse(&text));
}

/// The name the opponent label shows now: `own` until the feed is loaded and
/// the label has shown a live opponent, then the feed's schedule.
pub(super) fn label(own: &str, live: bool) -> String {
    let Some(Some(feed)) = FEED.get() else {
        return own.to_owned();
    };
    if !live {
        return own.to_owned();
    }
    let now = unsafe { skyline::nn::os::GetSystemTick() as u64 }.max(1);
    let started = match STARTED.compare_exchange(0, now, Ordering::AcqRel, Ordering::Acquire) {
        Ok(_) => now,
        Err(started) => started,
    };
    let index = label_feed::index_at(feed, now.saturating_sub(started) / TICKS_PER_MS);
    let name = label_feed::name(feed, index, own);
    if SHOWN.swap(index, Ordering::AcqRel) != index {
        record(
            Code::LabelFeed,
            index as u64,
            label_feed::dynamic_chars(name) as u64,
        );
    }
    name.to_owned()
}
