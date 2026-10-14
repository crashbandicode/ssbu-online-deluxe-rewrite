//! Bounded CSV export for a finished capture pair.
//! Recording stays in the capture modules. This path only formats already-ready
//! snapshots. Native writes are not timed out or cancelled.
#[cfg(test)]
use std::io::{BufWriter, Write};
use std::{
    fs::{File, OpenOptions},
    io,
    path::Path,
    sync::atomic::{AtomicU8, Ordering},
};

pub(crate) const PHASE_IDLE: u8 = 0;
pub(crate) const PHASE_CAPTURING: u8 = 1;
pub(crate) const PHASE_SAVED: u8 = 2;
pub(crate) const PHASE_EXPORT_ERROR: u8 = 3;
pub(crate) const PHASE_EXPORTING: u8 = 4;
pub(crate) const EXPORT_BUFFER_BYTES: usize = 64 * 1024;

pub(crate) fn status_label(phase: u8) -> &'static str {
    match phase {
        PHASE_CAPTURING => "Capturing",
        PHASE_SAVED => "CSV saved",
        PHASE_EXPORT_ERROR => "Export error",
        PHASE_EXPORTING => "Exporting CSV",
        _ => "Idle",
    }
}

/// Repeated controller requests are ignored for the whole recording and export.
pub(crate) fn controller_request_busy(phase: u8) -> bool {
    phase == PHASE_CAPTURING || phase == PHASE_EXPORTING
}

/// Missing paths are created. An existing path is an error, not an overwrite.
///
/// skyline-v3 checks `create_new` for existing paths, but creates missing paths
/// only when `create` is set. Truncate stays off. A newly created file is empty.
pub(crate) fn create_export_file(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .truncate(false)
        .create(true)
        .create_new(true)
        .open(path)
}

/// Run `write_body` through a 64 KiB buffer and flush before returning the sink.
/// `Drop` is not the flush boundary: its flush error would be discarded.
/// Callers that hold a `File` still apply `sync_all` after this returns.
/// Since the match capture both CSVs stream through their own `BufWriter`
/// from arm to finish, so only the tests exercise this single-shot form.
#[cfg(test)]
pub(crate) fn write_buffered<W: Write>(
    inner: W,
    write_body: impl FnOnce(&mut BufWriter<W>) -> io::Result<()>,
) -> io::Result<W> {
    let mut writer = BufWriter::with_capacity(EXPORT_BUFFER_BYTES, inner);
    write_body(&mut writer)?;
    writer.flush()?;
    writer.into_inner().map_err(|error| error.into_error())
}

/// Set Exporting before any CSV work. Persist saved only after both writers and
/// the saved status succeed. A failed error-status write leaves export error set.
pub(crate) fn export_ready_csvs(
    phase: &AtomicU8,
    exporting_status: &str,
    saved_status: &str,
    mut persist_status: impl FnMut(&str) -> io::Result<()>,
    mut write_timing: impl FnMut() -> io::Result<()>,
    mut write_api: impl FnMut() -> io::Result<()>,
) -> io::Result<()> {
    phase.store(PHASE_EXPORTING, Ordering::Release);
    let result = (|| {
        persist_status(exporting_status)?;
        write_timing()?;
        write_api()?;
        persist_status(saved_status)?;
        Ok(())
    })();
    match &result {
        Ok(()) => phase.store(PHASE_SAVED, Ordering::Release),
        Err(error) => {
            phase.store(PHASE_EXPORT_ERROR, Ordering::Release);
            let _ = persist_status(&format!("timing export failed: {error}"));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        render_trace::{self, Event},
        timing::{self, Finish, Sample, Snapshot},
    };
    use std::{cell::RefCell, fs, sync::atomic::Ordering};

    #[derive(Default)]
    struct ProbeWriter {
        bytes: Vec<u8>,
        lengths: Vec<usize>,
        fail_write: bool,
        fail_flush: bool,
    }

    impl Write for ProbeWriter {
        fn write(&mut self, data: &[u8]) -> io::Result<usize> {
            if self.fail_write {
                return Err(io::Error::other("injected write failure"));
            }
            self.lengths.push(data.len());
            self.bytes.extend_from_slice(data);
            Ok(data.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            if self.fail_flush {
                return Err(io::Error::other("injected flush failure"));
            }
            Ok(())
        }
    }

    fn sample(n: u64) -> Sample {
        Sample {
            callback: 10 + n,
            tick: 1_000 + n * 16,
            buttons: if n.is_multiple_of(2) { 0 } else { u64::MAX },
            fx_mode: (n % 3) as u8,
            fx_scope: (n % 2) as u8,
            fx_effective_off: n.is_multiple_of(5),
            overlay_view: (n % 4) as u8,
        }
    }

    fn timing_snapshot(count: usize, reason: Finish) -> Snapshot {
        Snapshot {
            id: 129_729_755_560,
            reason,
            skipped: 2,
            samples: (0..count as u64).map(sample).collect(),
        }
    }

    fn api_snapshot(count: usize) -> render_trace::Snapshot {
        render_trace::Snapshot {
            id: 129_729_755_560,
            reason: Finish::Elapsed,
            skipped: 4,
            events: (0..count)
                .map(|n| Event {
                    begin: 50_000 + n as u64,
                    end: 50_016 + n as u64,
                    kind: (n % 35) as u8,
                    index: if n.is_multiple_of(2) { -1 } else { i32::MIN },
                    result: if n.is_multiple_of(3) { i32::MAX } else { -7 },
                })
                .collect(),
        }
    }

    fn direct_timing(snapshot: &Snapshot) -> Vec<u8> {
        let mut bytes = Vec::new();
        timing::write_csv(
            &mut bytes,
            snapshot,
            timing::DURATION_TICKS,
            "build-1",
            "controller",
            "Vanilla",
        )
        .unwrap();
        bytes
    }

    #[test]
    fn buffered_csvs_match_serializers_and_bound_underlying_writes() {
        let timing = timing_snapshot(600, Finish::Elapsed);
        let timing_direct = direct_timing(&timing);
        let timing_buffered = write_buffered(Vec::new(), |out| {
            timing::write_csv(
                out,
                &timing,
                timing::DURATION_TICKS,
                "build-1",
                "controller",
                "Vanilla",
            )
        })
        .unwrap();
        assert_eq!(timing_buffered, timing_direct);
        let timing_csv = String::from_utf8(timing_buffered).unwrap();
        assert!(timing_csv.contains("# reason=elapsed\n"));
        assert!(timing_csv.contains("# sample_count=600\n"));
        assert!(timing_csv.ends_with("# complete=1\n"));
        assert_eq!(
            timing_csv
                .lines()
                .filter(|line| !line.starts_with('#') && !line.starts_with("sample,"))
                .count(),
            600
        );

        let aborted = timing_snapshot(3, Finish::Aborted);
        let aborted_direct = {
            let mut bytes = Vec::new();
            timing::write_csv(
                &mut bytes,
                &aborted,
                timing::DURATION_TICKS,
                "build-1",
                "controller",
                "Mixed",
            )
            .unwrap();
            bytes
        };
        let aborted_buffered = write_buffered(Vec::new(), |out| {
            timing::write_csv(
                out,
                &aborted,
                timing::DURATION_TICKS,
                "build-1",
                "controller",
                "Mixed",
            )
        })
        .unwrap();
        assert_eq!(aborted_buffered, aborted_direct);
        assert!(String::from_utf8(aborted_direct)
            .unwrap()
            .contains("# reason=aborted\n"));

        let api = api_snapshot(4_000);
        let mut raw = ProbeWriter::default();
        render_trace::write_csv(&mut raw, &api, "build-1", "controller", "LessLag").unwrap();
        let buffered = write_buffered(ProbeWriter::default(), |out| {
            render_trace::write_csv(out, &api, "build-1", "controller", "LessLag")
        })
        .unwrap();
        assert_eq!(buffered.bytes, raw.bytes);
        let csv = String::from_utf8(buffered.bytes.clone()).unwrap();
        assert!(csv.contains("# event_count=4000\n"));
        assert!(csv.contains(",-1,"));
        assert!(csv.contains(&i32::MIN.to_string()));
        assert!(csv.contains("-7"));
        assert!(csv.ends_with("# complete=1\n"));
        assert!(csv.len() > EXPORT_BUFFER_BYTES);
        assert!(raw.lengths.len() > api.events.len() * 8);
        assert!(buffered.lengths.len() < api.events.len());
        let chunks = csv.len().div_ceil(EXPORT_BUFFER_BYTES);
        assert!(buffered.lengths.len() >= 2);
        assert!(buffered.lengths.len() <= chunks + 1);
    }

    #[test]
    fn write_and_flush_failures_do_not_report_saved() {
        let snapshot = timing_snapshot(8, Finish::Elapsed);
        let phase = AtomicU8::new(PHASE_CAPTURING);
        let mut lines = Vec::new();
        let mut api_called = false;
        let flush_error = export_ready_csvs(
            &phase,
            "exporting 129729755560 controller; metric=render_callback",
            "saved timing-129729755560.csv",
            |message| {
                if message.starts_with("timing export failed") {
                    return Err(io::Error::other("status persist failed"));
                }
                lines.push(message.to_owned());
                Ok(())
            },
            || {
                let writer = ProbeWriter {
                    fail_flush: true,
                    ..ProbeWriter::default()
                };
                let _file = write_buffered(writer, |out| {
                    timing::write_csv(
                        out,
                        &snapshot,
                        timing::DURATION_TICKS,
                        "build-1",
                        "controller",
                        "Vanilla",
                    )
                })?;
                Ok(())
            },
            || {
                api_called = true;
                Ok(())
            },
        )
        .unwrap_err();
        assert!(!api_called);
        assert!(flush_error.to_string().contains("injected flush failure"));
        assert_eq!(phase.load(Ordering::Acquire), PHASE_EXPORT_ERROR);
        assert_eq!(status_label(PHASE_EXPORT_ERROR), "Export error");
        assert_eq!(
            lines,
            vec!["exporting 129729755560 controller; metric=render_callback".to_owned()]
        );

        let phase = AtomicU8::new(PHASE_CAPTURING);
        let mut lines = Vec::new();
        let write_error = export_ready_csvs(
            &phase,
            "exporting 42 controller; metric=render_callback",
            "saved timing-42.csv",
            |message| {
                lines.push(message.to_owned());
                if message.starts_with("saved ") {
                    return Err(io::Error::other("saved status failed"));
                }
                Ok(())
            },
            || {
                let writer = ProbeWriter {
                    fail_write: true,
                    ..ProbeWriter::default()
                };
                let _file = write_buffered(writer, |out| {
                    timing::write_csv(
                        out,
                        &snapshot,
                        timing::DURATION_TICKS,
                        "build-1",
                        "controller",
                        "Vanilla",
                    )
                })?;
                Ok(())
            },
            || Ok(()),
        )
        .unwrap_err();
        assert!(write_error.to_string().contains("injected write failure"));
        assert_eq!(phase.load(Ordering::Acquire), PHASE_EXPORT_ERROR);
        assert!(lines.iter().any(|line| line.starts_with("exporting 42 ")));
        assert!(lines.iter().any(|line| {
            line.starts_with("timing export failed:") && line.contains("injected write failure")
        }));
        assert!(!lines.iter().any(|line| line.starts_with("saved ")));

        let phase = AtomicU8::new(PHASE_CAPTURING);
        let mut lines = Vec::new();
        let saved_error = export_ready_csvs(
            &phase,
            "exporting 77 controller; metric=render_callback",
            "saved timing-77.csv",
            |message| {
                if message.starts_with("saved ") {
                    return Err(io::Error::other("saved status failed"));
                }
                lines.push(message.to_owned());
                Ok(())
            },
            || Ok(()),
            || Ok(()),
        )
        .unwrap_err();
        assert!(saved_error.to_string().contains("saved status failed"));
        assert_eq!(phase.load(Ordering::Acquire), PHASE_EXPORT_ERROR);
        assert_eq!(
            lines,
            vec![
                "exporting 77 controller; metric=render_callback".to_owned(),
                "timing export failed: saved status failed".to_owned(),
            ]
        );
    }

    #[test]
    fn exporting_is_published_before_csv_work_and_busy_rejects_requests() {
        let phase = AtomicU8::new(PHASE_CAPTURING);
        let steps = RefCell::new(Vec::new());
        export_ready_csvs(
            &phase,
            "exporting 42 controller; metric=render_callback",
            "saved timing-42.csv",
            |message| {
                steps.borrow_mut().push(format!(
                    "status:{message}:phase={}",
                    phase.load(Ordering::Acquire)
                ));
                Ok(())
            },
            || {
                assert!(controller_request_busy(phase.load(Ordering::Acquire)));
                steps
                    .borrow_mut()
                    .push(format!("timing:phase={}", phase.load(Ordering::Acquire)));
                Ok(())
            },
            || {
                steps
                    .borrow_mut()
                    .push(format!("api:phase={}", phase.load(Ordering::Acquire)));
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(phase.load(Ordering::Acquire), PHASE_SAVED);
        assert_eq!(
            steps.borrow().as_slice(),
            vec![
                "status:exporting 42 controller; metric=render_callback:phase=4".to_owned(),
                "timing:phase=4".to_owned(),
                "api:phase=4".to_owned(),
                "status:saved timing-42.csv:phase=4".to_owned(),
            ]
        );
        assert!(controller_request_busy(PHASE_CAPTURING));
        assert!(controller_request_busy(PHASE_EXPORTING));
        assert!(!controller_request_busy(PHASE_IDLE));
        assert!(!controller_request_busy(PHASE_SAVED));
        assert!(!controller_request_busy(PHASE_EXPORT_ERROR));
        assert_eq!(status_label(PHASE_EXPORTING), "Exporting CSV");
        assert_eq!(status_label(PHASE_CAPTURING), "Capturing");
        assert_eq!(status_label(PHASE_SAVED), "CSV saved");
        assert_eq!(status_label(PHASE_IDLE), "Idle");
    }

    #[test]
    fn buffered_file_export_syncs_and_refuses_overwrite() {
        let dir =
            std::env::temp_dir().join(format!("deluxe-capture-export-{}-file", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("timing-1.csv");
        let snapshot = timing_snapshot(4, Finish::Elapsed);
        let direct = direct_timing(&snapshot);
        let file = create_export_file(&path).unwrap();
        let file = write_buffered(file, |out| {
            timing::write_csv(
                out,
                &snapshot,
                timing::DURATION_TICKS,
                "build-1",
                "controller",
                "Vanilla",
            )
        })
        .unwrap();
        file.sync_all().unwrap();
        drop(file);
        assert_eq!(fs::read(&path).unwrap(), direct);
        let before = fs::read(&path).unwrap();
        assert!(create_export_file(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        let _ = fs::remove_dir_all(&dir);
    }
}
