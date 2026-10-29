//! One log per stage under `matches/` (0.8.2), so a session of many matches
//! needs no manual rotation. Each holds the settings, render and 120 Hz state
//! at stage entry, every `events.log` row while the stage runs, and a
//! present-pacing summary at its end. `crate::match_log` names the files and
//! keeps the newest `KEEP`. Runs on the diagnostics thread only.
use super::{auto_driver, nvn, record, settings, Code, DATA_DIR};
use crate::auto_policy::SceneStatus;
use crate::diagnostics::Event;
use crate::match_log as plan;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

/// The refresh-clock plugin's status file (rewritten each second while its
/// `telemetry.flag` exists), copied at both ends of each stage so its 120 Hz
/// counters sit next to the present pacing.
const REFRESH_CLOCK_STATUS: &str = "sd:/ultimate/ssbu-refresh-clock/status.txt";

struct Stage {
    file: File,
    opened_tick: u64,
    start: nvn::Pacing,
    /// The stage's `net_watch` rows, for the closing `net` line.
    net: crate::net_watch::Summary,
}

#[derive(Default)]
pub(super) struct MatchLog {
    stage: Option<Stage>,
    /// Newest sequence handled: a batch that `events.log` failed to write is
    /// replayed, and its rows must not reach a stage log twice.
    last: u64,
}

impl MatchLog {
    /// One `events.log` row, in order. A stage load (code 110) closes the
    /// running log and opens one for any stage but Inactive; a scene context
    /// (code 73) that turns Inactive closes it after that row.
    pub(super) fn event(&mut self, event: &Event) {
        if event.sequence <= self.last {
            return;
        }
        self.last = event.sequence;
        if event.code == Code::StageLoaded as u16 {
            self.close(event.tick, "next_stage");
            let scene = SceneStatus::from_byte(event.b as u8);
            if scene != SceneStatus::Inactive {
                self.open(event, scene);
            }
        }
        if let Some(stage) = self.stage.as_mut() {
            let _ = writeln!(
                stage.file,
                "{} {} {} 0x{:x} 0x{:x}",
                event.sequence, event.tick, event.code, event.a, event.b
            );
            stage.net.row(event.code, event.a, event.b);
        }
        if event.code == Code::AutoContext as u16
            && plan::context_scene(event.a) == SceneStatus::Inactive
        {
            self.close(event.tick, "scene_left");
        }
    }

    /// Called after each batch of rows, like the `events.log` sync.
    pub(super) fn sync(&mut self) {
        if let Some(stage) = self.stage.as_mut() {
            let _ = stage.file.sync_data();
        }
    }

    fn folder() -> PathBuf {
        Path::new(DATA_DIR).join(plan::DIRECTORY)
    }

    fn open(&mut self, event: &Event, scene: SceneStatus) {
        let folder = Self::folder();
        if fs::create_dir_all(&folder).is_err() {
            return;
        }
        let index = folder.join(plan::INDEX);
        let names = plan::parse_index(&fs::read_to_string(&index).unwrap_or_default());
        let name = plan::file_name(plan::next_sequence(&names), scene, event.a);
        let Ok(mut file) = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(folder.join(&name))
        else {
            return;
        };
        let (kept, expired) = plan::admit(names, name);
        for old in expired {
            let _ = fs::remove_file(folder.join(old));
        }
        let _ = fs::write(&index, plan::render_index(&kept));
        let _ = file.write_all(header(event, scene).as_bytes());
        self.stage = Some(Stage {
            file,
            opened_tick: event.tick,
            start: nvn::pacing(),
            net: crate::net_watch::Summary::default(),
        });
    }

    fn close(&mut self, tick: u64, reason: &str) {
        let Some(mut stage) = self.stage.take() else {
            return;
        };
        let _ = stage
            .file
            .write_all(footer(&stage, tick, reason).as_bytes());
        let _ = stage.file.sync_all();
    }
}

/// Code 120 (0.8.2): the refresh-clock plugin's 120 Hz counters, called every
/// 5 s from boot. The row reaches `events.log` and, during a stage, its log, so
/// the 120 Hz data needs no relaunch or flag change. Nothing is recorded while
/// the file is absent or mid-rewrite.
pub(super) fn record_refresh_clock() {
    let text = fs::read_to_string(REFRESH_CLOCK_STATUS).unwrap_or_default();
    if let Some(sample) = crate::refresh_clock_status::parse(&text) {
        let (a, b) = crate::refresh_clock_status::pack(sample);
        record(Code::RefreshClock, a, b);
    }
}

fn render_line(prefix: &str) -> String {
    let auto = auto_driver::snapshot();
    let policy = crate::render_policy::labels(crate::render_policy::load_machine());
    let (active, allocated, interval) = nvn::window_counts();
    format!(
        "{prefix} selected={} auto_profile={} requested={} committed={} effective={} textures={active}/{allocated} present_interval={interval}\n",
        auto.selected.key(),
        auto.auto_profile,
        auto.request.key(),
        policy.committed,
        policy.effective,
    )
}

/// `prefix.<line>` for every line of the refresh-clock status, or
/// `prefix=absent`.
fn refresh_clock(prefix: &str) -> String {
    match fs::read_to_string(REFRESH_CLOCK_STATUS) {
        Ok(text) => text
            .lines()
            .map(|line| format!("{prefix}.{line}\n"))
            .collect(),
        Err(_) => format!("{prefix}=absent\n"),
    }
}

fn clock() -> String {
    crate::clock_probe::status_lines(super::clock_probe::mhz(), super::clock_probe::hoc())
}

fn header(event: &Event, scene: SceneStatus) -> String {
    format!(
        "Deluxe stage log\nbuild={}\nstage=0x{:x} scene={} connection={} opened_tick={}\nsettings={:?}\n{}{}\n{}sequence tick code arg0 arg1\n",
        crate::build_info::BUILD_ID,
        event.a,
        scene.key(),
        auto_driver::snapshot().connection.key(),
        event.tick,
        settings(),
        render_line("render_start"),
        clock(),
        refresh_clock("refresh_clock_start"),
    )
}

fn footer(stage: &Stage, tick: u64, reason: &str) -> String {
    let pacing = nvn::pacing().since(stage.start);
    let seconds = tick.saturating_sub(stage.opened_tick) as f64 / 19_200_000.0;
    format!(
        "closed_tick={tick} reason={reason} seconds={seconds:.1}\npresents={} early_presents={} early_present_skips={} early_present_mismatches={} present_pace_outliers={} present_reacquired_last={}\n{}\n{}{}{}\n{}",
        pacing.actual,
        pacing.early,
        pacing.skips,
        pacing.mismatches,
        pacing.outliers,
        pacing.reacquired,
        plan::spacing_line(&pacing.spacing),
        stage.net.line(),
        render_line("render_end"),
        clock(),
        refresh_clock("refresh_clock_end"),
    )
}
