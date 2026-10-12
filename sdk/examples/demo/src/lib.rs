//! Demo plugin: registers one plugin with three options on the Online Deluxe
//! Plugins page. It has no hooks, no game offsets and no behavior of its own.
//!
//! The host is resolved at runtime (`nn::ro::LookupSymbol`), so this NRO also
//! loads when Online Deluxe is not installed; it then logs `HostMissing`.
//!
//! Persistence: the plugin names its own INI with `config_path`. After a menu
//! edit the host rewrites that file (`enabled=on`, `mode=b`); the host never
//! reads it, so this plugin loads it at startup and seeds its options from it.
//!
//! The Skyline entry point exists only on the switch target. A host build of
//! this crate is empty so `cargo check` and `cargo clippy` work off-console.

/// Parses the demo INI into the choice index for each persisted option.
///
/// Lines are `key=value`; blank lines and `#` comments are skipped, as are
/// unknown keys and values that are not one of the option's tokens. The last
/// valid line for a key wins. Returns `(enabled, mode)` as `Option` indices.
#[cfg_attr(not(any(test, target_os = "switch")), allow(dead_code))]
fn parse_config(text: &str) -> (Option<u32>, Option<u32>) {
    const ENABLED: &[&str] = &["off", "on"];
    const MODE: &[&str] = &["a", "b", "c"];
    let mut enabled = None;
    let mut mode = None;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        let index = |tokens: &[&str]| tokens.iter().position(|token| *token == value);
        match key {
            "enabled" => enabled = index(ENABLED).map(|i| i as u32).or(enabled),
            "mode" => mode = index(MODE).map(|i| i as u32).or(mode),
            _ => {}
        }
    }
    (enabled, mode)
}

#[cfg(test)]
mod tests {
    use super::parse_config;

    #[test]
    fn parses_persisted_choices() {
        assert_eq!(parse_config("enabled=on\nmode=c\n"), (Some(1), Some(2)));
        assert_eq!(parse_config("enabled=off\nmode=a\n"), (Some(0), Some(0)));
    }

    #[test]
    fn ignores_comments_unknown_keys_and_bad_values() {
        let text = "# demo\nother=1\nenabled=maybe\nmode = b\n\nnonsense\nstatus=0\n";
        assert_eq!(parse_config(text), (None, Some(1)));
        assert_eq!(parse_config(""), (None, None));
    }

    #[test]
    fn last_valid_line_wins() {
        assert_eq!(
            parse_config("enabled=on\nenabled=bogus\nenabled=off\n").0,
            Some(0)
        );
        assert_eq!(parse_config("enabled=on\nenabled=bogus\n").0, Some(1));
    }
}

#[cfg(target_os = "switch")]
mod switch {
    use super::parse_config;
    use deluxe_plugin_sdk::{abi, host, register, AtomicOptions, OptionSpec, PluginSpec};
    use std::ffi::CStr;

    const CONFIG_PATH: &str = "sd:/ultimate/deluxe_plugin_demo/config.ini";

    // enabled: Off/On, mode: A/B/C, status: one choice, so a read-only row.
    static OPTIONS: AtomicOptions<3> = AtomicOptions::new([0, 0, 0]);

    // The host persists `enabled` and `mode` into CONFIG_PATH after a menu edit.
    // `status` is read-only and flagged `no_persist`.
    static SPEC: PluginSpec = PluginSpec {
        id: "demo",
        label: "Demo",
        options: &[
            OptionSpec {
                id: "enabled",
                label: "Enabled",
                choices: &["Off", "On"],
                config_key: Some("enabled"),
                values: Some(&["off", "on"]),
                no_persist: false,
            },
            OptionSpec {
                id: "mode",
                label: "Mode",
                choices: &["A", "B", "C"],
                config_key: None,
                values: Some(&["a", "b", "c"]),
                no_persist: false,
            },
            OptionSpec {
                id: "status",
                label: "Status",
                choices: &["Ready"],
                config_key: None,
                values: None,
                no_persist: true,
            },
        ],
        config_path: Some(CONFIG_PATH),
    };

    /// Seeds `OPTIONS` from the persisted INI when it exists. The host never
    /// reads this file, so a choice survives a restart only through this.
    fn load_config() {
        let text = match std::fs::read_to_string(CONFIG_PATH) {
            Ok(text) => text,
            Err(error) => {
                skyline::println!("[deluxe-plugin-demo] no config loaded ({CONFIG_PATH}): {error}");
                return;
            }
        };
        let (enabled, mode) = parse_config(&text);
        if let Some(choice) = enabled {
            OPTIONS.set(0, choice);
        }
        if let Some(choice) = mode {
            OPTIONS.set(1, choice);
        }
        skyline::println!(
            "[deluxe-plugin-demo] loaded {CONFIG_PATH}: enabled={} mode={}",
            OPTIONS.get(0),
            OPTIONS.get(1)
        );
    }

    #[skyline::main(name = "deluxe-plugin-demo")]
    pub fn main() {
        match host() {
            Some(api) => {
                // SAFETY: `host()` only returns tables that passed `table_matches`;
                // `host_build` returns a NUL-terminated string valid for the process.
                let (build, status) = unsafe {
                    let build = (api.host_build)();
                    let build = if build.is_null() {
                        "<null>".to_owned()
                    } else {
                        CStr::from_ptr(build).to_string_lossy().into_owned()
                    };
                    (build, (api.host_status)())
                };
                skyline::println!(
                    "[deluxe-plugin-demo] host build: {build} status: {status:#x} menu_active: {}",
                    status & abi::HOST_STATUS_MENU_ACTIVE != 0
                );
            }
            None => skyline::println!("[deluxe-plugin-demo] host build: <no Deluxe host found>"),
        }
        // Load before registering so the first `current` call already reports
        // the persisted choice.
        load_config();
        let result = register(&SPEC, &OPTIONS);
        skyline::println!("[deluxe-plugin-demo] register: {result:?}");
    }
}
