//! Opt-in, finite callback traces. These are not display/present timestamps.
//! Only the existing diagnostic thread touches files or allocates an export.
use super::{effects_off, record, settings, Code, DATA_DIR};
use crate::capture_export::{
    controller_request_busy, create_export_file, export_ready_csvs, status_label, write_buffered,
    PHASE_CAPTURING, PHASE_EXPORT_ERROR, PHASE_IDLE,
};
use crate::timing::{Capture, Sample};
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::Path,
    sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
    time::Instant,
};

static CAPTURE: Capture = Capture::new();
static REQUESTED: AtomicBool = AtomicBool::new(false);
static CSS_REQUESTED: AtomicBool = AtomicBool::new(false);
/// Wall-clock abort margin after the guest-clock duration.
const ABORT_SECONDS: u64 = crate::timing::DURATION_SECONDS + 10;
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

#[derive(Default)]
pub(super) struct Service {
    current: Option<(String, Instant, &'static str)>,
    callback_ready: Option<crate::timing::Snapshot>,
    api_ready: Option<crate::render_trace::Snapshot>,
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

impl Service {
    /// Called only by the existing background sink, once every 500 ms.
    pub(super) fn poll(&mut self) {
        super::css_capture::poll();
        if let Some((label, started, engine)) = &self.current {
            if started.elapsed().as_secs() >= ABORT_SECONDS {
                CAPTURE.finish();
                super::api_trace::TRACE.finish();
            }
            if self.callback_ready.is_none() {
                self.callback_ready = CAPTURE.take_ready();
            }
            if self.api_ready.is_none() {
                self.api_ready = super::api_trace::TRACE.take_ready();
            }
            if let (Some(snapshot), Some(api)) = (&self.callback_ready, &self.api_ready) {
                let engine = if *engine == effective_engine() {
                    *engine
                } else {
                    "Mixed"
                };
                let path = Path::new(DATA_DIR).join(format!("timing-{}.csv", snapshot.id));
                let api_path = Path::new(DATA_DIR).join(format!("api-{}.csv", api.id));
                let exporting = format!(
                    "exporting {} {}; metric=render_callback",
                    snapshot.id, label
                );
                let saved = format!("saved {}", path.display());
                let build = crate::build_info::BUILD_ID;
                let result = export_ready_csvs(
                    &STATUS,
                    &exporting,
                    &saved,
                    write_status,
                    || -> io::Result<()> {
                        let file = create_export_file(&path)?;
                        let file = write_buffered(file, |out| {
                            crate::timing::write_csv(out, snapshot, build, label, engine)
                        })?;
                        file.sync_all()
                    },
                    || -> io::Result<()> {
                        let file = create_export_file(&api_path)?;
                        let file = write_buffered(file, |out| {
                            crate::render_trace::write_csv(out, api, build, label, engine)
                        })?;
                        file.sync_all()
                    },
                );
                record(Code::TimingFinished, snapshot.id, u64::from(result.is_ok()));
                self.current = None;
                self.callback_ready = None;
                self.api_ready = None;
            }
            return;
        }
        let label = if REQUESTED.swap(false, Ordering::AcqRel) {
            CSS_REQUESTED.store(false, Ordering::Release);
            Some("controller".to_owned())
        } else if CSS_REQUESTED.swap(false, Ordering::AcqRel) {
            Some("css".to_owned())
        } else {
            match read_request() {
                Ok(value) => {
                    self.request_error_reported = false;
                    value
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
        if let Some(label) = label {
            let id = unsafe { skyline::nn::os::GetSystemTick() as u64 };
            // Complete control-file I/O before starting the measured interval.
            let _ = write_status(&format!("capturing {id} {label}; metric=render_callback"));
            if CAPTURE.begin(id) && super::api_trace::TRACE.begin(id) {
                self.current = Some((label, Instant::now(), effective_engine()));
                record(Code::TimingStarted, id, 0);
                record(
                    Code::MenuOrder,
                    id,
                    u64::from(super::overlay_host::embedded()),
                );
                STATUS.store(PHASE_CAPTURING, Ordering::Release);
            } else {
                CAPTURE.finish();
                super::api_trace::TRACE.finish();
                let _ = CAPTURE.take_ready();
                let _ = super::api_trace::TRACE.take_ready();
                STATUS.store(PHASE_EXPORT_ERROR, Ordering::Release);
                let _ = write_status("timing capture could not be armed");
            }
        }
    }
}
