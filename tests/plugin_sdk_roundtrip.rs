//! The public SDK registering through a table with the exact shape the NRO
//! exports, landing in the host registry the Plugins page reads. This is the
//! host-side proof of the contract; it is not an emulator or hardware run.

use core::ffi::c_char;
use deluxe_plugin_sdk::{abi, register_with, AtomicOptions, OptionSpec, PluginSpec, RegisterError};
use ssbu_online_deluxe_rewrite::plugin_api::{plugin_rows, Registry, RequestStatus};

static HOST: Registry = Registry::new();

unsafe extern "C" fn register(desc: *const abi::PluginDesc) -> i32 {
    HOST.register_raw(desc)
}

unsafe extern "C" fn host_status() -> u32 {
    abi::HOST_STATUS_MENU_ACTIVE
}

unsafe extern "C" fn host_build() -> *const c_char {
    c"host-test".as_ptr()
}

static TABLE: abi::PluginApiV1 = abi::PluginApiV1 {
    struct_size: core::mem::size_of::<abi::PluginApiV1>() as u32,
    abi_version: abi::ABI_VERSION,
    register,
    host_status,
    host_build,
};

static OPTIONS: AtomicOptions<3> = AtomicOptions::new([0, 0, 0]);
static SPEC: PluginSpec = PluginSpec::new(
    "demo",
    "Demo",
    &[
        OptionSpec::new("enabled", "Enabled", &["Off", "On"]),
        OptionSpec::new("mode", "Mode", &["A", "B", "C"]),
        OptionSpec::new("status", "Status", &["Ready"]),
    ],
);

static REPLAY: AtomicOptions<2> = AtomicOptions::new([0, 0]);
static REPLAY_SPEC: PluginSpec = PluginSpec {
    id: "replay",
    label: "Replay",
    options: &[
        OptionSpec {
            id: "record",
            label: "Record",
            choices: &["Off", "On"],
            config_key: Some("record_enabled"),
            values: Some(&["off", "on"]),
            no_persist: false,
        },
        OptionSpec {
            id: "quality",
            label: "Quality",
            choices: &["Low", "High"],
            config_key: None,
            values: None,
            no_persist: false,
        },
    ],
    config_path: Some("sd:/ultimate/replay/config.ini"),
};

#[test]
fn sdk_registration_reaches_the_host_registry_and_edits_round_trip() {
    let handle = unsafe { register_with(&TABLE, &SPEC, &OPTIONS) }.unwrap();
    assert_eq!(handle.0, 0);
    let replay = unsafe { register_with(&TABLE, &REPLAY_SPEC, &REPLAY) }.unwrap();
    assert_eq!(replay.0, 1);
    assert_eq!(
        unsafe { register_with(&TABLE, &SPEC, &OPTIONS) },
        Err(RegisterError::Duplicate)
    );

    let plugins = HOST.snapshot();
    assert_eq!(plugins.len(), 2);
    assert_eq!(plugins[0].id, "demo");
    assert_eq!(plugins[0].label, "Demo");
    assert_eq!(plugins[0].options[1].label, "Mode");
    assert_eq!(plugins[0].options[1].choices, vec!["A", "B", "C"]);
    assert!(plugins[0].options[2].read_only());
    assert_eq!(plugins[1].label, "Replay");
    assert_eq!(plugin_rows(&plugins).len(), 5);
    assert_eq!(plugins[0].config_path, None);
    assert_eq!(
        plugins[1].config_path.as_deref(),
        Some("sd:/ultimate/replay/config.ini")
    );
    assert_eq!(plugins[1].options[0].config_key, "record_enabled");
    assert_eq!(plugins[1].options[1].config_key, "quality");

    assert_eq!(HOST.value(0, &plugins[0], 0).label, "Off");
    let step = HOST.step(0, 0, 1).unwrap();
    assert_eq!(step.status, RequestStatus::Applied);
    assert_eq!(step.requested, 1);
    assert_eq!(OPTIONS.get(0), 1);
    assert_eq!(HOST.value(0, &plugins[0], 0).label, "On");
    assert!(!HOST.value(0, &plugins[0], 0).pending);

    let step = HOST.step(0, 1, -1).unwrap();
    assert_eq!(step.requested, 2);
    assert_eq!(OPTIONS.get(1), 2);
    assert_eq!(HOST.value(0, &plugins[0], 1).label, "C");
    assert_eq!(HOST.step(0, 2, 1), None);
    assert_eq!(HOST.value(0, &plugins[0], 2).label, "Ready");

    // Only the plugin that declared a config path queues host persistence.
    assert!(HOST.take_writes().is_empty());
    assert_eq!(HOST.step(1, 0, 1).unwrap().requested, 1);
    assert_eq!(REPLAY.get(0), 1);
    assert_eq!(HOST.value(1, &plugins[1], 0).label, "On");
    assert_eq!(HOST.step(1, 1, 1).unwrap().requested, 1);
    let writes = HOST.take_writes();
    assert_eq!(writes.len(), 2);
    assert_eq!(writes[0].path, "sd:/ultimate/replay/config.ini");
    assert_eq!(
        (writes[0].key.as_str(), writes[0].value.as_str()),
        ("record_enabled", "on")
    );
    assert_eq!(
        (writes[1].key.as_str(), writes[1].value.as_str()),
        ("quality", "1")
    );
    assert_eq!(
        ssbu_online_deluxe_rewrite::plugin_api::apply_ini_assignment(
            "version=1\nrecord_enabled=off\n",
            &writes[0].key,
            &writes[0].value
        ),
        "version=1\nrecord_enabled=on\n"
    );

    // The SDK handles an out-of-range host query without panicking.
    assert_eq!(plugins[0].current(7), None);

    // Local validation refuses before the host sees anything.
    static BAD: PluginSpec = PluginSpec::new("Bad Id", "Bad", &[OptionSpec::new("x", "X", &["a"])]);
    static BAD_STATE: AtomicOptions<1> = AtomicOptions::new([0]);
    assert_eq!(
        unsafe { register_with(&TABLE, &BAD, &BAD_STATE) },
        Err(RegisterError::Invalid)
    );
    assert_eq!(HOST.count(), 2);
    assert_eq!(
        unsafe { (TABLE.host_status)() },
        abi::HOST_STATUS_MENU_ACTIVE
    );
}
