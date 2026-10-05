use super::{publish, record, settings, Code, CONFIG_GENERATION, DATA_DIR, RECORDER};
use crate::{
    build_info,
    config::{self, Config},
};
use std::{
    ffi::CString,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

extern "C" {
    // Hidden, module-local symbols from the Skyline Rust linker script. Using
    // them avoids get_plugin_addresses(), whose C++ singleton is distinct from
    // the manager used by the loader and enters __cxa_guard_acquire at startup.
    static __text_start: u8;
    static __bss_end: u8;
}

const MAX_LOG_BYTES: u64 = 4 * 1024 * 1024;
static PANICKING: AtomicBool = AtomicBool::new(false);

/// Persist startup progress before configuration, recorder locks or Skyline's
/// asynchronous logger are used. This also catches failures in diagnostics itself.
pub(super) fn prepare() {
    let root = Path::new(DATA_DIR);
    if fs::create_dir_all(root).is_ok() {
        if let Ok(mut file) = rotate(&root.join("install.log")) {
            let _ = writeln!(file, "build={}\nphase=entered_main", build_info::BUILD_ID);
            let _ = file.sync_all();
        }
    }
}

pub(super) fn startup_phase(phase: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(Path::new(DATA_DIR).join("install.log"))
    {
        let _ = writeln!(file, "phase={phase}");
        let _ = file.sync_all();
    }
}

fn rotate(path: &Path) -> io::Result<File> {
    if path.exists() {
        let previous = path.with_extension("previous.log");
        if previous.exists() {
            fs::remove_file(&previous)?;
        }
        fs::rename(path, previous)?;
    }
    OpenOptions::new().create(true).append(true).open(path)
}

fn save_config(config: &Config) -> io::Result<()> {
    let path = Path::new(DATA_DIR).join("config.ini");
    crate::config_store::save(&path, &config.serialize())
}

fn config_is_missing(path: &Path) -> bool {
    // skyline-v3's std maps GetEntryType's 0x202 to ErrorKind::Other.
    // Ask the native API so only a confirmed missing file is created; an
    // unreadable or invalid existing config must remain untouched.
    let Ok(path) = CString::new(path.as_os_str().as_encoded_bytes()) else {
        return false;
    };
    let mut entry_type = 0;
    unsafe { skyline::nn::fs::GetEntryType(&mut entry_type, path.as_ptr().cast()) == 0x202 }
}

pub(super) fn start(version: &str) {
    let root = Path::new(DATA_DIR);
    let dir_result = fs::create_dir_all(root);
    let config_path = root.join("config.ini");
    startup_phase("config_read_begin");
    if let Err(error) = crate::config_store::recover(&config_path) {
        skyline::println!("[Deluxe] config recovery failed: {}", error);
    }
    let (config, config_note) = match fs::read_to_string(&config_path) {
        Ok(text) => match config::parse(&text) {
            Ok(config) => (config, "loaded config".to_owned()),
            Err(error) => {
                record(Code::ConfigRejected, 0, 0);
                (
                    Config::default(),
                    format!("rejected config (file preserved): {error}"),
                )
            }
        },
        Err(error)
            if error.kind() == io::ErrorKind::NotFound || config_is_missing(&config_path) =>
        {
            let config = Config::default();
            let note = match save_config(&config) {
                Ok(()) => "created default config".to_owned(),
                Err(error) => format!("default config not persisted: {error}"),
            };
            (config, note)
        }
        Err(error) => (Config::default(), format!("config unreadable: {error}")),
    };
    startup_phase("config_read_complete");
    publish(&config);
    startup_phase("recorder_begin");
    record(
        Code::ConfigLoaded,
        config.effects as u64,
        config.input_buffer as i64 as u64,
    );
    startup_phase("recorder_ready");
    let text_base =
        unsafe { skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as u64 };
    startup_phase("game_text_ready");
    let plugin_start = core::ptr::addr_of!(__text_start) as u64;
    let plugin_end = core::ptr::addr_of!(__bss_end) as u64;
    startup_phase("plugin_bounds_ready");
    let module_id = if plugin_end.saturating_sub(plugin_start) >= 0x60
        && unsafe { super::memory::accessible(plugin_start as usize, 0x60, false) }
    {
        let bytes = unsafe { std::slice::from_raw_parts(plugin_start as *const u8, 0x60) };
        if &bytes[0x10..0x14] == b"NRO0" {
            bytes[0x40..0x60]
                .iter()
                .map(|v| format!("{v:02x}"))
                .collect::<String>()
        } else {
            "header-unavailable".to_owned()
        }
    } else {
        "range-unavailable".to_owned()
    };
    let header = format!(
        "Deluxe rewrite {}\ncommit={}\nSmash={version}\ngame_text=0x{text_base:x}\nplugin_start=0x{plugin_start:x} plugin_end=0x{plugin_end:x}\nmodule_id={module_id}\n{config_note}\nsettings={config:?}\nrender_backend=not-installed\nsequence tick code arg0 arg1\n",
        build_info::BUILD_ID, build_info::GIT_COMMIT,
    );
    startup_phase("skyline_header_begin");
    skyline::println!("{}", header);
    startup_phase("skyline_header_queued");
    if let Err(error) = dir_result {
        skyline::println!("[Deluxe] diagnostic directory unavailable: {}", error);
    }
    if let Ok(mut file) = OpenOptions::new()
        .append(true)
        .open(root.join("install.log"))
    {
        let _ = file
            .write_all(header.as_bytes())
            .and_then(|_| file.sync_all());
    }
    let log_path = root.join("events.log");
    let mut log = match rotate(&log_path) {
        Ok(mut file) => {
            if let Err(error) = file
                .write_all(header.as_bytes())
                .and_then(|_| file.sync_data())
            {
                skyline::println!("[Deluxe] diagnostic header write failed: {}", error);
            }
            Some(file)
        }
        Err(error) => {
            skyline::println!("[Deluxe] file logging unavailable: {}", error);
            None
        }
    };

    startup_phase("panic_hook_begin");

    // Preserve other mods' panic hook. Hardware exceptions never reach this hook.
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if !PANICKING.swap(true, Ordering::AcqRel) {
            record(Code::Panic, 0, 0);
            if let Ok(mut file) = File::create(Path::new(DATA_DIR).join("panic.log")) {
                let _ = writeln!(file, "build={}\n{info}", build_info::BUILD_ID);
                if let Some(snapshot) = RECORDER.snapshot() {
                    for event in snapshot.after(0) {
                        let _ = writeln!(file, "{event:?}");
                    }
                    let _ = writeln!(file, "dropped={}", snapshot.dropped);
                }
                let _ = file.sync_all();
            }
        }
        previous(info);
    }));
    startup_phase("panic_hook_ready");
    record(Code::Boot, plugin_start, text_base);

    let mut generation = CONFIG_GENERATION.load(Ordering::Acquire);
    startup_phase("diagnostic_thread_begin");
    if let Err(error) = std::thread::Builder::new()
        .name("deluxe-diagnostics".into())
        .spawn(move || {
            let mut persisted = 0;
            let mut dropped = 0;
            let mut iteration = 0u64;
            let mut error_reported = false;
            let mut timing = super::timing::Service::default();
            loop {
                std::thread::sleep(Duration::from_millis(500));
                timing.poll();
                iteration += 1;
                if iteration % 10 == 0 {
                    record(Code::Heartbeat, settings().effects as u64, iteration);
                    record(
                        Code::OverlayHealth,
                        super::overlay_host::ready() as u64,
                        super::overlay_host::bootstrap_calls(),
                    );
                }
                if let Some(file) = log.as_mut() {
                    if let Some(snapshot) = RECORDER.snapshot() {
                        let result: io::Result<()> = (|| {
                            if snapshot.newest.saturating_sub(persisted)
                                > crate::diagnostics::CAPACITY as u64
                            {
                                writeln!(file, "gap: ring overwritten after sequence {persisted}")?;
                            }
                            for event in snapshot.after(persisted) {
                                writeln!(
                                    file,
                                    "{} {} {} 0x{:x} 0x{:x}",
                                    event.sequence, event.tick, event.code, event.a, event.b
                                )?;
                            }
                            if snapshot.dropped != dropped {
                                writeln!(file, "dropped_total={}", snapshot.dropped)?;
                            }
                            file.sync_data()
                        })();
                        match result {
                            Ok(()) => {
                                persisted = snapshot.newest;
                                dropped = snapshot.dropped;
                            }
                            Err(error) if !error_reported => {
                                skyline::println!("[Deluxe] log sink error: {}", error);
                                error_reported = true;
                            }
                            Err(_) => {}
                        }
                    }
                    if file
                        .metadata()
                        .map(|m| m.len() >= MAX_LOG_BYTES)
                        .unwrap_or(false)
                    {
                        log = rotate(&log_path).ok();
                        if let Some(file) = log.as_mut() {
                            let _ = file.write_all(header.as_bytes());
                        }
                    }
                }
                let next_generation = CONFIG_GENERATION.load(Ordering::Acquire);
                if next_generation != generation {
                    match save_config(&settings()) {
                        Ok(()) => {
                            generation = next_generation;
                            record(Code::ConfigSaved, generation, settings().effects as u64);
                        }
                        Err(error) if !error_reported => {
                            skyline::println!("[Deluxe] config save failed: {}", error);
                            error_reported = true;
                        }
                        Err(_) => {}
                    }
                }
            }
        })
    {
        skyline::println!("[Deluxe] diagnostic thread failed: {}", error);
    }
    startup_phase("diagnostic_thread_returned");
}

/// Synchronous only during installation, never in a frame/effect callback.
pub(super) fn install_checkpoint(code: Code, address: u64, detail: u64) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(Path::new(DATA_DIR).join("install.log"))
    {
        let _ = writeln!(file, "{} 0x{address:x} 0x{detail:x}", code as u16);
        let _ = file.sync_all();
    }
}
