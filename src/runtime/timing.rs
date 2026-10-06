//! Opt-in, finite callback traces. These are not display/present timestamps.
//! Only the existing diagnostic thread touches files or allocates an export.
use super::{effects_off, record, settings, Code, DATA_DIR};
use crate::timing::{Capture, Sample};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
    sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
    time::Instant,
};

static CAPTURE: Capture = Capture::new();
static REQUESTED: AtomicBool = AtomicBool::new(false);
static CALLBACK: AtomicU64 = AtomicU64::new(0);
static STATUS: AtomicU8 = AtomicU8::new(0);
static VIEW: AtomicU8 = AtomicU8::new(0);

pub(super) fn set_view(view: u8) {
    VIEW.store(view, Ordering::Relaxed);
}

pub(super) fn request() {
    // A running/exporting capture is never interrupted by a repeated shortcut.
    if STATUS.load(Ordering::Acquire) != 1 {
        REQUESTED.store(true, Ordering::Release);
    }
}

pub(super) fn status() -> &'static str {
    match STATUS.load(Ordering::Relaxed) {
        1 => "Capturing (10 s)",
        2 => "CSV saved",
        3 => "Export error",
        _ => "Idle",
    }
}

pub(super) fn observe(buttons: u64) {
    let callback = CALLBACK.fetch_add(1, Ordering::Relaxed) + 1;
    if STATUS.load(Ordering::Acquire) != 1 {
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

impl Service {
    /// Called only by the existing background sink, once every 500 ms.
    pub(super) fn poll(&mut self) {
        if let Some((label, started, engine)) = &self.current {
            if started.elapsed().as_secs() >= 30 {
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
                let engine = if *engine == super::renderer::mode().label() {
                    *engine
                } else {
                    "Mixed"
                };
                let path = Path::new(DATA_DIR).join(format!("timing-{}.csv", snapshot.id));
                let result = (|| -> io::Result<()> {
                    let mut file = OpenOptions::new()
                        .write(true)
                        // skyline-v3 checks `create_new` for existing paths,
                        // but creates missing paths only when `create` is set.
                        // Truncate stays off: an existing CSV is an error, not
                        // an overwrite. A newly created file is already empty.
                        .truncate(false)
                        .create(true)
                        .create_new(true)
                        .open(&path)?;
                    crate::timing::write_csv(
                        &mut file,
                        snapshot,
                        crate::build_info::BUILD_ID,
                        label,
                        engine,
                    )?;
                    file.sync_all()?;
                    let api_path = Path::new(DATA_DIR).join(format!("api-{}.csv", api.id));
                    let mut api_file = OpenOptions::new()
                        .write(true)
                        .truncate(false)
                        .create(true)
                        .create_new(true)
                        .open(api_path)?;
                    crate::render_trace::write_csv(
                        &mut api_file,
                        api,
                        crate::build_info::BUILD_ID,
                        label,
                        engine,
                    )?;
                    api_file.sync_all()?;
                    write_status(&format!("saved {}", path.display()))
                })();
                STATUS.store(if result.is_ok() { 2 } else { 3 }, Ordering::Release);
                record(Code::TimingFinished, snapshot.id, u64::from(result.is_ok()));
                if let Err(error) = result {
                    let _ = write_status(&format!("timing export failed: {error}"));
                }
                self.current = None;
                self.callback_ready = None;
                self.api_ready = None;
            }
            return;
        }
        let label = if REQUESTED.swap(false, Ordering::AcqRel) {
            Some("controller".to_owned())
        } else {
            match read_request() {
                Ok(value) => {
                    self.request_error_reported = false;
                    value
                }
                Err(error) => {
                    STATUS.store(3, Ordering::Release);
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
                self.current = Some((label, Instant::now(), super::renderer::mode().label()));
                record(Code::TimingStarted, id, 0);
                STATUS.store(1, Ordering::Release);
            } else {
                CAPTURE.finish();
                super::api_trace::TRACE.finish();
                let _ = CAPTURE.take_ready();
                let _ = super::api_trace::TRACE.take_ready();
                STATUS.store(3, Ordering::Release);
                let _ = write_status("timing capture could not be armed");
            }
        }
    }
}
