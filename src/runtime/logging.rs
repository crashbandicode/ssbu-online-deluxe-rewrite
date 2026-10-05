use super::{publish, record, settings, Code, CONFIG_GENERATION, DATA_DIR, RECORDER};
use crate::{
    build_info,
    config::{self, Config},
};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

const MAX_LOG_BYTES: u64 = 4 * 1024 * 1024;
static PANICKING: AtomicBool = AtomicBool::new(false);

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
    let temp = path.with_extension("tmp");
    let mut file = File::create(&temp)?;
    file.write_all(config.serialize().as_bytes())?;
    file.sync_all()?;
    fs::rename(temp, path)
}

pub(super) fn start(version: &str) {
    let root = Path::new(DATA_DIR);
    let dir_result = fs::create_dir_all(root);
    let config_path = root.join("config.ini");
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
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let config = Config::default();
            let note = match save_config(&config) {
                Ok(()) => "created default config".to_owned(),
                Err(error) => format!("default config not persisted: {error}"),
            };
            (config, note)
        }
        Err(error) => (Config::default(), format!("config unreadable: {error}")),
    };
    publish(&config);
    record(
        Code::ConfigLoaded,
        config.effects as u64,
        config.input_buffer as i64 as u64,
    );
    let text_base =
        unsafe { skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as u64 };
    let (plugin_start, plugin_end) = unsafe {
        skyline::info::containing_plugin(super::main as *const () as *const skyline::libc::c_void)
    };
    let module_id = if plugin_end.saturating_sub(plugin_start) >= 0x60 && plugin_start != 0 {
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
    skyline::println!("{}", header);
    if let Err(error) = dir_result {
        skyline::println!("[Deluxe] diagnostic directory unavailable: {}", error);
    }
    if let Ok(mut file) = rotate(&root.join("install.log")) {
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
    record(Code::Boot, plugin_start, text_base);

    let mut generation = CONFIG_GENERATION.load(Ordering::Acquire);
    if let Err(error) = std::thread::Builder::new()
        .name("deluxe-diagnostics".into())
        .spawn(move || {
            let mut persisted = 0;
            let mut dropped = 0;
            let mut iteration = 0u64;
            let mut error_reported = false;
            loop {
                std::thread::sleep(Duration::from_millis(500));
                iteration += 1;
                if iteration % 10 == 0 {
                    record(Code::Heartbeat, settings().effects as u64, iteration);
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
                        Ok(()) => generation = next_generation,
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
