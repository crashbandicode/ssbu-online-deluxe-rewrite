//! Device side of the capture pair (0.5.3): the callback ring, the four
//! triggers (Debug chord, `CAPTURE_TIMING` label, `CAPTURE_CSS` entry,
//! `CAPTURE_MATCH` stage), the character-select visit, the Debug rows and the
//! diagnostics-thread service that streams both CSVs. Only that thread
//! touches files or allocates; no hook does I/O. These are callback and API
//! observations, not display or present timestamps.
use super::{record, Code, DATA_DIR};
use crate::auto_policy::SceneStatus;
use crate::capture::{
    self, status_label, Entry, Event, Finish, Ring, Sample, Stream, Summary, ABORT_MARGIN_SECONDS,
    API_ROWS, CALLBACK_ROWS, DURATION_TICKS, EXPORT_BUFFER_BYTES, MATCH_DURATION_TICKS,
    PHASE_CAPTURING, PHASE_EXPORTING, PHASE_EXPORT_ERROR, PHASE_IDLE, PHASE_SAVED, TICK_HZ,
};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Read, Write},
    path::Path,
    sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicU8, Ordering},
    time::Instant,
};

/// `FLAGS` bits: the two boot flags in the data directory.
pub(super) const FLAG_CSS: u8 = 1;
pub(super) const FLAG_MATCH: u8 = 2;
/// `PENDING`: 0 none, otherwise one request, a match (`PENDING_MATCH + status`)
/// outranking the controller chord, which outranks character-select entry.
const PENDING_CONTROLLER: u8 = 1;
const PENDING_CSS: u8 = 2;
const PENDING_MATCH: u8 = 16;
/// `b` of code 106 `a=2` when the character-select update ended the capture.
const END_BY_CSS: u64 = 0xC55;
/// A visit closes one second after the last character-select update.
const CSS_IDLE_TICKS: u64 = TICK_HZ;
const TIMING: usize = 0;
const API: usize = 1;
const STREAMS: [Stream; 2] = [Stream::Timing, Stream::Api];
const FILE_STEMS: [&str; 2] = ["timing", "api"];

static CALLBACKS: Ring<Sample> = Ring::new(CALLBACK_ROWS);
static CALLBACK: AtomicU64 = AtomicU64::new(0);
static VIEW: AtomicU8 = AtomicU8::new(0);
static STATUS: AtomicU8 = AtomicU8::new(PHASE_IDLE);
static PENDING: AtomicU8 = AtomicU8::new(0);
/// The scene left the stage: a running match capture ends at the next poll.
static MATCH_ENDED: AtomicBool = AtomicBool::new(false);
/// A match capture is recording or exporting (read per CSS frame).
static MATCH_ACTIVE: AtomicBool = AtomicBool::new(false);
/// The current match capture's end was recorded; cleared by the next stage.
static END_RECORDED: AtomicBool = AtomicBool::new(false);
static FLAGS: AtomicU8 = AtomicU8::new(0);
static IN_CSS: AtomicBool = AtomicBool::new(false);
static LAST_CSS_TICK: AtomicU64 = AtomicU64::new(0);
static CSS_ARMED: AtomicU32 = AtomicU32::new(0);
static MATCH_ARMED: AtomicU32 = AtomicU32::new(0);
static MATCH_ENDS: AtomicU32 = AtomicU32::new(0);

fn tick() -> u64 {
    unsafe { skyline::nn::os::GetSystemTick() as u64 }
}

/// Recording or exporting: requests are ignored meanwhile.
fn busy() -> bool {
    matches!(
        STATUS.load(Ordering::Acquire),
        PHASE_CAPTURING | PHASE_EXPORTING
    )
}

/// Startup: reads both flags (code 106 `a=0`). `css_capture::install` reads
/// [`flags`] after this.
pub(super) fn install() {
    let root = Path::new(DATA_DIR);
    let flags = u8::from(root.join("CAPTURE_CSS").is_file())
        | u8::from(root.join("CAPTURE_MATCH").is_file()) << 1;
    FLAGS.store(flags, Ordering::Release);
    record(Code::MatchCapture, 0, u64::from(flags & FLAG_MATCH != 0));
    if flags & FLAG_MATCH != 0 {
        skyline::println!("[Deluxe] match capture flag present; arming on stage entry");
    }
}

pub(super) fn flags() -> u8 {
    FLAGS.load(Ordering::Acquire)
}

/// Overlay view for the `overlay_view` column (render callback).
pub(super) fn set_view(view: u8) {
    VIEW.store(view, Ordering::Relaxed);
}

/// Render callback: the counter always, the ring only while capturing.
pub(super) fn observe(buttons: u64) {
    let callback = CALLBACK.fetch_add(1, Ordering::Relaxed) + 1;
    if STATUS.load(Ordering::Acquire) != PHASE_CAPTURING {
        return;
    }
    CALLBACKS.observe(Sample {
        callback,
        tick: tick(),
        buttons,
        overlay_view: VIEW.load(Ordering::Relaxed),
    });
}

/// L+R+Up in the Debug view. Outranks a pending CSS request, never a match.
pub(super) fn request() {
    if !busy() {
        let _ = PENDING.fetch_update(Ordering::AcqRel, Ordering::Acquire, |pending| {
            (pending < PENDING_MATCH).then_some(PENDING_CONTROLLER)
        });
    }
}

/// The game's character-select update ran (native banner hook, every frame).
/// With `CAPTURE_MATCH` it ends a running match capture (the Training CSS keeps
/// the Training status, so the status alone would not). The first call of a
/// visit records code 12 and, with both hooks live and `CAPTURE_CSS`, requests
/// a capture.
pub(super) fn note_css_frame() {
    let now = tick();
    LAST_CSS_TICK.store(now, Ordering::Relaxed);
    let flags = flags();
    if flags & FLAG_MATCH != 0 && MATCH_ACTIVE.load(Ordering::Acquire) {
        end_match(END_BY_CSS);
    }
    if IN_CSS.swap(true, Ordering::AcqRel) {
        return;
    }
    let ready = flags & FLAG_CSS != 0 && super::css_capture::hooks_ready();
    record(Code::CssEntered, now, u64::from(ready));
    if ready
        && !busy()
        && PENDING
            .compare_exchange(0, PENDING_CSS, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    {
        CSS_ARMED.fetch_add(1, Ordering::Relaxed);
    }
}

/// A character-select visit is open (an update ran within the last second).
pub(super) fn in_css() -> bool {
    IN_CSS.load(Ordering::Acquire)
}

fn on_stage(status: SceneStatus) -> bool {
    matches!(
        status,
        SceneStatus::Singles | SceneStatus::Doubles | SceneStatus::Training
    )
}

fn stage_label(status: SceneStatus) -> &'static str {
    match status {
        SceneStatus::Singles => "match_singles",
        SceneStatus::Doubles => "match_doubles",
        SceneStatus::Training => "match_training",
        SceneStatus::Inactive => "match",
    }
}

/// The stage-setup hook classified a loading stage (every stage load, also
/// Training re-entered from its own character select). Code 106 `a=1`.
pub(super) fn note_stage(status: SceneStatus) {
    if flags() & FLAG_MATCH == 0 || !on_stage(status) {
        return;
    }
    END_RECORDED.store(false, Ordering::Release);
    if PENDING.swap(PENDING_MATCH + status as u8, Ordering::AcqRel) < PENDING_MATCH {
        MATCH_ARMED.fetch_add(1, Ordering::Relaxed);
    }
    record(Code::MatchCapture, 1, status as u64);
}

/// Scene status changed (any scene hook): leaving a stage for Inactive ends
/// the running match capture.
pub(super) fn note_scene(previous: SceneStatus, status: SceneStatus) {
    if flags() & FLAG_MATCH != 0 && on_stage(previous) && status == SceneStatus::Inactive {
        end_match(previous as u64);
    }
}

/// Ends the running match capture at the next poll, withdraws a match request
/// that was never armed and records code 106 `a=2` once per capture.
fn end_match(reason: u64) {
    let _ = PENDING.fetch_update(Ordering::AcqRel, Ordering::Acquire, |pending| {
        (pending >= PENDING_MATCH).then_some(0)
    });
    MATCH_ENDED.store(true, Ordering::Release);
    if !END_RECORDED.swap(true, Ordering::AcqRel) {
        MATCH_ENDS.fetch_add(1, Ordering::Relaxed);
        record(Code::MatchCapture, 2, reason);
    }
}

/// Debug row `Timing`.
pub(super) fn status() -> &'static str {
    status_label(STATUS.load(Ordering::Acquire))
}

/// Debug row `CSS capture`.
pub(super) fn css_label() -> String {
    let hooks = super::css_capture::hooks();
    if flags() & FLAG_CSS == 0 {
        return match hooks {
            0 => "Off (no CAPTURE_CSS)".to_string(),
            hooks => format!("Hooks {hooks:#x} for CAPTURE_MATCH; not arming"),
        };
    }
    match hooks {
        super::css_capture::HOOKS_ALL => format!(
            "Armed; {} capture(s) on CSS entry",
            CSS_ARMED.load(Ordering::Relaxed)
        ),
        0 => "Unavailable (words differ)".to_string(),
        hooks => format!("Partial {hooks:#x}; not arming"),
    }
}

/// Debug row `Match capture`.
pub(super) fn match_label() -> String {
    if flags() & FLAG_MATCH == 0 {
        return "Off (no CAPTURE_MATCH)".to_string();
    }
    format!(
        "Armed; {} capture(s) on stage entry, {} ended",
        MATCH_ARMED.load(Ordering::Relaxed),
        MATCH_ENDS.load(Ordering::Relaxed)
    )
}

fn finish_both(reason: Finish) {
    CALLBACKS.finish(reason);
    super::api_trace::TRACE.finish(reason);
}

fn write_status(message: &str) -> io::Result<()> {
    let mut file = File::create(Path::new(DATA_DIR).join("timing-status.txt"))?;
    writeln!(file, "{message}")?;
    file.sync_all()
}

/// `CAPTURE_TIMING`: the label, or nothing. Even an invalid request is
/// consumed, so it is not re-read every poll while playing.
fn read_request() -> io::Result<Option<String>> {
    let path = Path::new(DATA_DIR).join("CAPTURE_TIMING");
    if !path.exists() {
        return Ok(None);
    }
    let mut value = Vec::new();
    // Close the handle before removing the file (Switch filesystem semantics).
    let read_result = (|| File::open(&path)?.take(65).read_to_end(&mut value))();
    fs::remove_file(path)?;
    read_result?;
    let label = capture::request_label(&value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    Ok(Some(label.into()))
}

/// Missing paths are created; an existing path is an error, never an
/// overwrite (skyline-v3 honours `create_new` only together with `create`).
fn create_new(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .truncate(false)
        .create(true)
        .create_new(true)
        .open(path)
}

/// Footer `# engine=`; `Mixed` when it changed while recording.
fn effective_engine() -> &'static str {
    let decision = crate::render_policy::load_decision();
    if !decision.live {
        return super::renderer::mode().label();
    }
    if decision.swapping {
        return "Transition";
    }
    match decision.effective {
        crate::render_policy::Profile::Vanilla => "Vanilla",
        crate::render_policy::Profile::LessLag => "LessLag",
        crate::render_policy::Profile::Ultra => "LLUltra",
        crate::render_policy::Profile::Doubles => "LLDoubles",
    }
}

/// One capture in progress. Both CSVs are open from arm to finish and receive
/// rows every poll; only the footers are written at the end.
struct Running {
    id: u64,
    label: String,
    started: Instant,
    engine: &'static str,
    /// The effective engine differed from `engine` at some poll while recording.
    engine_changed: bool,
    is_match: bool,
    abort_seconds: u64,
    /// Timing then API. A stream whose write failed is closed; the first
    /// error is reported by the export.
    files: [Option<BufWriter<File>>; 2],
    next: [usize; 2],
    error: Option<io::Error>,
    summaries: [Option<Summary>; 2],
}

impl Running {
    fn append<T: Entry>(&mut self, stream: usize, rows: &[T]) {
        let Some(out) = self.files[stream].as_mut() else {
            return;
        };
        if let Err(error) = capture::rows(out, &mut self.next[stream], rows) {
            self.error.get_or_insert(error);
            self.files[stream] = None;
        }
    }

    /// Footers, flush and sync in stream order; a deferred write error wins.
    fn close(mut self, engine: &str) -> io::Result<()> {
        if let Some(error) = self.error.take() {
            return Err(error);
        }
        for stream in [TIMING, API] {
            let (Some(mut out), Some(summary)) =
                (self.files[stream].take(), self.summaries[stream])
            else {
                return Err(io::Error::other("capture file already closed"));
            };
            capture::footer(&mut out, STREAMS[stream], engine, &summary)?;
            out.flush()?;
            out.into_inner()
                .map_err(|error| error.into_error())?
                .sync_all()?;
        }
        Ok(())
    }
}

/// Export: Exporting before any CSV work, Saved only after both footers and
/// the saved status succeed; otherwise Export error with the reason.
fn export(run: Running) {
    let id = run.id;
    let engine = if run.engine_changed {
        "Mixed"
    } else {
        run.engine
    };
    let path = Path::new(DATA_DIR).join(format!("timing-{id}.csv"));
    let exporting = format!("exporting {id} {}; metric=render_callback", run.label);
    STATUS.store(PHASE_EXPORTING, Ordering::Release);
    let result = write_status(&exporting)
        .and_then(|()| run.close(engine))
        .and_then(|()| write_status(&format!("saved {}", path.display())));
    match &result {
        Ok(()) => STATUS.store(PHASE_SAVED, Ordering::Release),
        Err(error) => {
            STATUS.store(PHASE_EXPORT_ERROR, Ordering::Release);
            let _ = write_status(&format!("timing export failed: {error}"));
        }
    }
    record(Code::TimingFinished, id, u64::from(result.is_ok()));
    MATCH_ACTIVE.store(false, Ordering::Release);
}

#[derive(Default)]
pub(super) struct Service {
    current: Option<Running>,
    /// Swap partners of the two rings: reserved at arm, freed with the export.
    samples: Vec<Sample>,
    events: Vec<Event>,
    request_error_reported: bool,
}

impl Service {
    /// A capture is recording or waiting for its export; the diagnostics loop
    /// polls twice as often while this holds.
    pub(super) fn capturing(&self) -> bool {
        self.current.is_some()
    }

    /// Called only by the diagnostics thread, every 500 ms (250 ms while a
    /// capture runs).
    pub(super) fn poll(&mut self) {
        let now = tick();
        if IN_CSS.load(Ordering::Acquire)
            && now.wrapping_sub(LAST_CSS_TICK.load(Ordering::Relaxed)) > CSS_IDLE_TICKS
        {
            IN_CSS.store(false, Ordering::Release);
            record(Code::CssExited, now, 0);
        }
        let Some(run) = self.current.as_mut() else {
            self.intake();
            return;
        };
        if run.started.elapsed().as_secs() >= run.abort_seconds {
            finish_both(Finish::Aborted);
        }
        let next_match_pending = PENDING.load(Ordering::Acquire) >= PENDING_MATCH;
        if run.is_match {
            // The game left the stage, or another stage loaded without a
            // visible exit: this capture ends and the next one arms once the
            // export is done.
            if MATCH_ENDED.swap(false, Ordering::AcqRel) || next_match_pending {
                finish_both(Finish::SceneEnd);
            }
        } else if next_match_pending {
            // A match started under a 30 s capture: end that one now so the
            // match capture can arm once its export is done.
            finish_both(Finish::Aborted);
            record(Code::MatchCapture, 3, run.id);
        }
        // Rows produced since the last poll, then, once a ring has finished,
        // its last rows together with the summary that counted them.
        CALLBACKS.drain(&mut self.samples);
        run.append(TIMING, &self.samples);
        super::api_trace::TRACE.drain(&mut self.events);
        run.append(API, &self.events);
        if let Some(summary) = CALLBACKS.take_ready(&mut self.samples) {
            run.append(TIMING, &self.samples);
            run.summaries[TIMING] = Some(summary);
        }
        if let Some(summary) = super::api_trace::TRACE.take_ready(&mut self.events) {
            run.append(API, &self.events);
            run.summaries[API] = Some(summary);
        }
        // The engine is sampled every poll while recording, so a profile
        // change after the capture finished (the cleanup hold at a match's
        // results screen) does not relabel a single-engine capture Mixed.
        if CALLBACKS.active() && run.engine != effective_engine() {
            run.engine_changed = true;
        }
        if run.summaries.iter().any(Option::is_none) {
            return;
        }
        if let Some(run) = self.current.take() {
            export(run);
        }
        self.samples = Vec::new();
        self.events = Vec::new();
    }

    /// Nothing running: the pending request, else the `CAPTURE_TIMING` file.
    fn intake(&mut self) {
        let (label, duration, is_match) = match PENDING.swap(0, Ordering::AcqRel) {
            0 => match read_request() {
                Ok(label) => {
                    self.request_error_reported = false;
                    let Some(label) = label else {
                        return;
                    };
                    (label, DURATION_TICKS, false)
                }
                Err(error) => {
                    STATUS.store(PHASE_EXPORT_ERROR, Ordering::Release);
                    if !self.request_error_reported {
                        let _ = write_status(&format!("timing request failed: {error}"));
                        self.request_error_reported = true;
                    }
                    return;
                }
            },
            PENDING_CONTROLLER => ("controller".to_owned(), DURATION_TICKS, false),
            PENDING_CSS => ("css".to_owned(), DURATION_TICKS, false),
            pending => (
                stage_label(SceneStatus::from_byte(pending - PENDING_MATCH)).to_owned(),
                MATCH_DURATION_TICKS,
                true,
            ),
        };
        self.arm(label, duration, is_match);
    }

    fn arm(&mut self, label: String, duration: u64, is_match: bool) {
        let id = tick();
        // Complete control-file I/O before starting the measured interval.
        let _ = write_status(&format!("capturing {id} {label}; metric=render_callback"));
        match self.start(id, label, duration, is_match) {
            Ok(run) => {
                if is_match {
                    // An end that arrived before this arm belongs to the
                    // previous stage; the new stage is live now.
                    MATCH_ENDED.store(false, Ordering::Release);
                    MATCH_ACTIVE.store(true, Ordering::Release);
                }
                record(Code::TimingStarted, id, u64::from(is_match));
                STATUS.store(PHASE_CAPTURING, Ordering::Release);
                self.current = Some(run);
            }
            Err(failure) => {
                finish_both(Finish::Aborted);
                let _ = CALLBACKS.take_ready(&mut self.samples);
                let _ = super::api_trace::TRACE.take_ready(&mut self.events);
                self.samples = Vec::new();
                self.events = Vec::new();
                STATUS.store(PHASE_EXPORT_ERROR, Ordering::Release);
                let _ = write_status(&failure);
            }
        }
    }

    /// Both CSV files are opened and headed (API first) before the hooks start
    /// producing rows, so a full card or a name clash refuses the capture here
    /// instead of losing it at the end. Then the swap buffers and both rings.
    fn start(
        &mut self,
        id: u64,
        label: String,
        duration: u64,
        is_match: bool,
    ) -> Result<Running, String> {
        let build = crate::build_info::BUILD_ID;
        // Clock state at arm time (0.5.3): the ledger refuses an `oc` capture.
        let clock =
            crate::clock_probe::header_lines(super::clock_probe::mhz(), super::clock_probe::hoc());
        let mut files: [Option<BufWriter<File>>; 2] = [None, None];
        for stream in [API, TIMING] {
            let path = Path::new(DATA_DIR).join(format!("{}-{id}.csv", FILE_STEMS[stream]));
            let opened = create_new(&path).and_then(|file| {
                let mut out = BufWriter::with_capacity(EXPORT_BUFFER_BYTES, file);
                capture::header(
                    &mut out,
                    STREAMS[stream],
                    id,
                    duration,
                    build,
                    &label,
                    &clock,
                )?;
                Ok(out)
            });
            match opened {
                Ok(out) => files[stream] = Some(out),
                Err(error) => {
                    return Err(format!(
                        "timing capture could not open {}: {error}",
                        path.display()
                    ))
                }
            }
        }
        if self.samples.try_reserve_exact(CALLBACK_ROWS).is_err()
            || self.events.try_reserve_exact(API_ROWS).is_err()
            || !CALLBACKS.begin(id, duration)
            || !super::api_trace::TRACE.begin(id, duration)
        {
            return Err("timing capture could not be armed".to_owned());
        }
        Ok(Running {
            id,
            label,
            started: Instant::now(),
            engine: effective_engine(),
            engine_changed: false,
            is_match,
            abort_seconds: duration / TICK_HZ + ABORT_MARGIN_SECONDS,
            files,
            next: [0; 2],
            error: None,
            summaries: [None; 2],
        })
    }
}
