//! Opt-in, finite callback traces. These are not display/present timestamps.
//! Only the existing diagnostic thread touches files or allocates an export.
use super::{effects_off, record, settings, Code, DATA_DIR};
use crate::auto_policy::SceneStatus;
use crate::capture_export::{
    controller_request_busy, create_export_file, export_ready_csvs, status_label,
    EXPORT_BUFFER_BYTES, PHASE_CAPTURING, PHASE_EXPORT_ERROR, PHASE_IDLE,
};
use crate::timing::{Capture, Finish, Sample};
use std::{
    fs::{self, File},
    io::{self, BufWriter, Read, Write},
    path::Path,
    sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
    time::Instant,
};

static CAPTURE: Capture = Capture::new();
static REQUESTED: AtomicBool = AtomicBool::new(false);
static CSS_REQUESTED: AtomicBool = AtomicBool::new(false);
/// `CAPTURE_MATCH` request: 0 none, otherwise the stage status byte plus one.
static MATCH_REQUESTED: AtomicU8 = AtomicU8::new(0);
/// The scene left the stage: a running match capture ends at the next poll.
static MATCH_ENDED: AtomicBool = AtomicBool::new(false);
/// A match capture is recording or exporting (read per CSS frame).
static MATCH_ACTIVE: AtomicBool = AtomicBool::new(false);
/// Wall-clock abort margin after the guest-clock duration.
const ABORT_MARGIN_SECONDS: u64 = 10;
static CALLBACK: AtomicU64 = AtomicU64::new(0);
static STATUS: AtomicU8 = AtomicU8::new(PHASE_IDLE);
static VIEW: AtomicU8 = AtomicU8::new(0);

pub(super) fn set_view(view: u8) {
    VIEW.store(view, Ordering::Relaxed);
}

pub(super) fn request() {
    // Recording and CSV export both ignore a repeated controller shortcut.
    if !controller_request_busy(STATUS.load(Ordering::Acquire)) {
        REQUESTED.store(true, Ordering::Release);
    }
}

/// Character-select entry with the CSS capture hooks live. Returns whether the
/// request was taken; a running recording or export ignores it.
pub(super) fn request_css() -> bool {
    if controller_request_busy(STATUS.load(Ordering::Acquire)) || REQUESTED.load(Ordering::Acquire)
    {
        return false;
    }
    !CSS_REQUESTED.swap(true, Ordering::AcqRel)
}

/// Stage load with `CAPTURE_MATCH` present (game thread). The request is kept
/// until the poll consumes it; a running capture of any kind is ended first
/// so the new match starts within a poll or two. Returns whether it was new.
pub(super) fn request_match(status: SceneStatus) -> bool {
    MATCH_REQUESTED.swap(status as u8 + 1, Ordering::AcqRel) == 0
}

/// The game left the stage. Ends a running match capture at the next poll
/// and withdraws a match request that was never armed.
pub(super) fn end_match() {
    MATCH_REQUESTED.store(0, Ordering::Release);
    MATCH_ENDED.store(true, Ordering::Release);
}

pub(super) fn match_active() -> bool {
    MATCH_ACTIVE.load(Ordering::Acquire)
}

pub(super) fn status() -> &'static str {
    status_label(STATUS.load(Ordering::Acquire))
}

pub(super) fn observe(buttons: u64) {
    let callback = CALLBACK.fetch_add(1, Ordering::Relaxed) + 1;
    if STATUS.load(Ordering::Acquire) != PHASE_CAPTURING {
        return;
    }
    let config = settings();
    CAPTURE.observe(Sample {
        callback,
        tick: unsafe { skyline::nn::os::GetSystemTick() as u64 },
        buttons,
        fx_mode: config.effects as u8,
        fx_scope: config.effects_scope as u8,
        fx_effective_off: effects_off(),
        overlay_view: VIEW.load(Ordering::Relaxed),
    });
}

/// One streamed CSV: open from arm to finish, rows appended every poll. A
/// write failure closes the file; the export reports it at the end.
struct Stream {
    out: Option<BufWriter<File>>,
    error: Option<io::Error>,
    next_row: usize,
}

impl Stream {
    fn new(out: BufWriter<File>) -> Self {
        Self {
            out: Some(out),
            error: None,
            next_row: 0,
        }
    }

    /// Placeholder left in `Running` once the real stream moved to the export.
    fn closed() -> Self {
        Self {
            out: None,
            error: None,
            next_row: 0,
        }
    }

    fn append(&mut self, write: impl FnOnce(&mut BufWriter<File>, &mut usize) -> io::Result<()>) {
        if let Some(out) = self.out.as_mut() {
            if let Err(error) = write(out, &mut self.next_row) {
                self.error = Some(error);
                self.out = None;
            }
        }
    }

    /// Footer, flush and sync; consumes the stream.
    fn close(
        mut self,
        footer: impl FnOnce(&mut BufWriter<File>) -> io::Result<()>,
    ) -> io::Result<()> {
        if let Some(error) = self.error.take() {
            return Err(error);
        }
        let Some(mut out) = self.out.take() else {
            return Err(io::Error::other("capture file already closed"));
        };
        footer(&mut out)?;
        out.flush()?;
        let file = out.into_inner().map_err(|error| error.into_error())?;
        file.sync_all()
    }
}

/// One capture in progress. Both CSVs are open from arm to finish and receive
/// rows every poll; only the footers are written at the end.
struct Running {
    id: u64,
    label: String,
    started: Instant,
    engine: &'static str,
    is_match: bool,
    abort_seconds: u64,
    timing: Stream,
    api: Stream,
}

#[derive(Default)]
pub(super) struct Service {
    current: Option<Running>,
    callback_ready: Option<crate::timing::Summary>,
    api_ready: Option<crate::render_trace::Summary>,
    /// Drain scratch, one chunk each; allocated once, on this thread.
    scratch: Vec<crate::render_trace::Event>,
    samples: Vec<Sample>,
    request_error_reported: bool,
}

fn write_status(message: &str) -> io::Result<()> {
    let mut file = File::create(Path::new(DATA_DIR).join("timing-status.txt"))?;
    writeln!(file, "{message}")?;
    file.sync_all()
}

fn read_request() -> io::Result<Option<String>> {
    let path = Path::new(DATA_DIR).join("CAPTURE_TIMING");
    if !path.exists() {
        return Ok(None);
    }
    let mut value = Vec::new();
    // Close the handle before removing the file (Switch filesystem semantics).
    let read_result = (|| File::open(&path)?.take(65).read_to_end(&mut value))();
    // Consume even an invalid request: do not repeatedly read it while playing.
    fs::remove_file(path)?;
    read_result?;
    let label = crate::timing::request_label(&value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    Ok(Some(label.into()))
}

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

/// What to arm: label, guest duration and whether the scene end finishes it.
struct Request {
    label: String,
    duration_ticks: u64,
    is_match: bool,
}

fn match_request() -> Option<Request> {
    let packed = MATCH_REQUESTED.swap(0, Ordering::AcqRel);
    if packed == 0 {
        return None;
    }
    let status = SceneStatus::from_byte(packed - 1);
    Some(Request {
        label: super::match_capture::label(status).to_owned(),
        duration_ticks: crate::timing::MATCH_DURATION_TICKS,
        is_match: true,
    })
}

fn finite(label: String) -> Request {
    Request {
        label,
        duration_ticks: crate::timing::DURATION_TICKS,
        is_match: false,
    }
}

impl Service {
    /// A capture is recording or waiting for its export; the diagnostics loop
    /// polls twice as often while this holds.
    pub(super) fn capturing(&self) -> bool {
        self.current.is_some()
    }

    /// Called only by the existing background sink, every 500 ms (250 ms while
    /// a capture runs).
    pub(super) fn poll(&mut self) {
        super::css_capture::poll();
        if let Some(run) = self.current.as_mut() {
            if run.started.elapsed().as_secs() >= run.abort_seconds {
                CAPTURE.finish();
                super::api_trace::TRACE.finish();
            }
            let next_match_pending = MATCH_REQUESTED.load(Ordering::Acquire) != 0;
            if run.is_match {
                // The game left the stage, or another stage loaded without a
                // visible exit: this capture ends and the next one arms once
                // the export is done.
                if MATCH_ENDED.swap(false, Ordering::AcqRel) || next_match_pending {
                    CAPTURE.finish_with(Finish::SceneEnd);
                    super::api_trace::TRACE.finish_with(Finish::SceneEnd);
                }
            } else if next_match_pending {
                // A match started under a 30 s capture: end that one now so
                // the match capture can arm once its export is done.
                CAPTURE.finish();
                super::api_trace::TRACE.finish();
                record(Code::MatchCapture, 3, run.id);
            }
            // Stream the rows and samples produced since the last poll.
            self.scratch.clear();
            if super::api_trace::TRACE.drain(&mut self.scratch) > 0 {
                let rows = &self.scratch;
                run.api
                    .append(|out, next| crate::render_trace::write_csv_rows(out, next, rows));
            }
            self.samples.clear();
            if CAPTURE.drain(&mut self.samples) > 0 {
                let samples = &self.samples;
                run.timing
                    .append(|out, next| crate::timing::write_csv_rows(out, next, samples));
            }
            if self.callback_ready.is_none() {
                self.callback_ready = CAPTURE.take_ready();
            }
            if self.api_ready.is_none() {
                self.api_ready = super::api_trace::TRACE.take_ready();
            }
            if let (Some(summary), Some(api)) = (self.callback_ready, self.api_ready) {
                let engine = if run.engine == effective_engine() {
                    run.engine
                } else {
                    "Mixed"
                };
                let label = run.label.as_str();
                let path = Path::new(DATA_DIR).join(format!("timing-{}.csv", summary.id));
                let exporting =
                    format!("exporting {} {}; metric=render_callback", summary.id, label);
                let saved = format!("saved {}", path.display());
                let mut timing_stream = Some(std::mem::replace(&mut run.timing, Stream::closed()));
                let mut api_stream = Some(std::mem::replace(&mut run.api, Stream::closed()));
                let result = export_ready_csvs(
                    &STATUS,
                    &exporting,
                    &saved,
                    write_status,
                    || -> io::Result<()> {
                        let Some(stream) = timing_stream.take() else {
                            return Err(io::Error::other("timing file already closed"));
                        };
                        stream.close(|out| crate::timing::write_csv_footer(out, engine, &summary))
                    },
                    || -> io::Result<()> {
                        let Some(stream) = api_stream.take() else {
                            return Err(io::Error::other("api trace file already closed"));
                        };
                        stream.close(|out| crate::render_trace::write_csv_footer(out, engine, &api))
                    },
                );
                record(Code::TimingFinished, summary.id, u64::from(result.is_ok()));
                self.current = None;
                MATCH_ACTIVE.store(false, Ordering::Release);
                self.callback_ready = None;
                self.api_ready = None;
            }
            return;
        }
        let request = if let Some(request) = match_request() {
            Some(request)
        } else if REQUESTED.swap(false, Ordering::AcqRel) {
            CSS_REQUESTED.store(false, Ordering::Release);
            Some(finite("controller".to_owned()))
        } else if CSS_REQUESTED.swap(false, Ordering::AcqRel) {
            Some(finite("css".to_owned()))
        } else {
            match read_request() {
                Ok(value) => {
                    self.request_error_reported = false;
                    value.map(finite)
                }
                Err(error) => {
                    STATUS.store(PHASE_EXPORT_ERROR, Ordering::Release);
                    if !self.request_error_reported {
                        let _ = write_status(&format!("timing request failed: {error}"));
                        self.request_error_reported = true;
                    }
                    return;
                }
            }
        };
        if let Some(request) = request {
            self.arm(request);
        }
    }

    fn arm(&mut self, request: Request) {
        let Request {
            label,
            duration_ticks,
            is_match,
        } = request;
        let id = unsafe { skyline::nn::os::GetSystemTick() as u64 };
        // Complete control-file I/O before starting the measured interval.
        let _ = write_status(&format!("capturing {id} {label}; metric=render_callback"));
        // Both CSV files are opened and headed before the hooks start producing
        // rows, so a full SD card or a name clash refuses the capture here
        // instead of losing it at the end.
        let build = crate::build_info::BUILD_ID;
        let api_path = Path::new(DATA_DIR).join(format!("api-{id}.csv"));
        let timing_path = Path::new(DATA_DIR).join(format!("timing-{id}.csv"));
        let api = create_export_file(&api_path).and_then(|file| {
            let mut out = BufWriter::with_capacity(EXPORT_BUFFER_BYTES, file);
            crate::render_trace::write_csv_header(&mut out, id, build, &label)?;
            Ok(out)
        });
        let timing = create_export_file(&timing_path).and_then(|file| {
            let mut out = BufWriter::with_capacity(EXPORT_BUFFER_BYTES, file);
            crate::timing::write_csv_header(&mut out, id, duration_ticks, build, &label)?;
            Ok(out)
        });
        let failure = match (api, timing) {
            (Ok(api), Ok(timing)) => {
                if CAPTURE.begin(id, duration_ticks)
                    && super::api_trace::TRACE.begin(id, duration_ticks)
                {
                    if self.scratch.capacity() == 0 {
                        self.scratch.reserve(crate::render_trace::CHUNK);
                    }
                    if self.samples.capacity() == 0 {
                        self.samples.reserve(crate::timing::CAPACITY);
                    }
                    self.current = Some(Running {
                        id,
                        label,
                        started: Instant::now(),
                        engine: effective_engine(),
                        is_match,
                        abort_seconds: duration_ticks / crate::timing::TICK_HZ
                            + ABORT_MARGIN_SECONDS,
                        timing: Stream::new(timing),
                        api: Stream::new(api),
                    });
                    if is_match {
                        // An end that arrived before this arm belongs to the
                        // previous stage; the new stage is live now.
                        MATCH_ENDED.store(false, Ordering::Release);
                        MATCH_ACTIVE.store(true, Ordering::Release);
                    }
                    record(Code::TimingStarted, id, u64::from(is_match));
                    record(
                        Code::MenuOrder,
                        id,
                        u64::from(super::overlay_host::embedded()),
                    );
                    STATUS.store(PHASE_CAPTURING, Ordering::Release);
                    return;
                }
                "timing capture could not be armed".to_owned()
            }
            (Err(error), _) => format!(
                "timing capture could not open {}: {error}",
                api_path.display()
            ),
            (_, Err(error)) => format!(
                "timing capture could not open {}: {error}",
                timing_path.display()
            ),
        };
        CAPTURE.finish();
        super::api_trace::TRACE.finish();
        let _ = CAPTURE.take_ready();
        let _ = super::api_trace::TRACE.take_ready();
        STATUS.store(PHASE_EXPORT_ERROR, Ordering::Release);
        let _ = write_status(&failure);
    }
}
