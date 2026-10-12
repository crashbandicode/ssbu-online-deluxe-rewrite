//! Rust consumer SDK for the Online Deluxe plugin registration ABI (version 1).
//!
//! A separately built Skyline NRO describes its menu options with a
//! [`PluginSpec`], supplies a [`PluginHandler`] and calls [`register`]. The
//! Deluxe host draws the rows; the plugin owns its behavior and persistence.
//!
//! The host is found at runtime with `nn::ro::LookupSymbol`, so a plugin built
//! with this crate loads and runs even when Deluxe is not installed:
//! [`register`] then returns [`RegisterError::HostMissing`] and the plugin
//! simply has no menu row. On non-switch builds the lookup is stubbed to "no
//! host" so the crate compiles and tests on a development machine; use
//! [`register_with`] to exercise it against a fake table.
//!
//! Rules inherited from the ABI (see `deluxe_plugin_abi`):
//!
//! * The host copies every string during `register`.
//! * Handler callbacks run on the host's overlay draw thread. They must not
//!   block, must not call back into the host API and must not assume a game
//!   thread. [`PluginHandler`] therefore requires `Send + Sync`.
//! * There is no unregister. The handler is leaked for the life of the process
//!   after a successful registration.
//!
//! # Optional host persistence
//!
//! Setting [`PluginSpec::config_path`] to an `sd:/...` path asks the host to
//! rewrite that `key=value` INI after each menu edit the handler answers with
//! `Applied` or `Pending`. The key is [`OptionSpec::config_key`] or the option
//! id; the value is the matching entry of [`OptionSpec::values`] or the decimal
//! choice index. Read-only (single-choice) options and options with
//! [`OptionSpec::no_persist`] are never written. The host never reads the file:
//! the plugin loads it at startup and reports the result through `current`.

use std::ffi::{c_char, c_void, CString};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;

pub use deluxe_plugin_abi as abi;

/// One menu option: a stable id, a display label and its ordered choices.
///
/// One choice makes a read-only status row.
#[derive(Clone, Copy, Debug)]
pub struct OptionSpec {
    /// Stable identifier, unique within the plugin. Lowercase ASCII, digits, `_`.
    pub id: &'static str,
    /// Display label (at most [`abi::MAX_LABEL_LEN`] bytes).
    pub label: &'static str,
    /// Display labels in Left/Right order, 1 to [`abi::MAX_CHOICES`] entries.
    pub choices: &'static [&'static str],
    /// INI key for host persistence (same charset as an id); `None` = the option id.
    pub config_key: Option<&'static str>,
    /// Value tokens written by host persistence, one per choice (each 1 to
    /// [`abi::MAX_VALUE_LEN`] printable ASCII bytes without whitespace, `=` or
    /// `#`); `None` = the decimal choice index.
    pub values: Option<&'static [&'static str]>,
    /// Never persisted by the host even when the plugin sets a config path.
    pub no_persist: bool,
}

impl OptionSpec {
    /// An option with host-persistence fields defaulted (`None`/`false`).
    pub const fn new(
        id: &'static str,
        label: &'static str,
        choices: &'static [&'static str],
    ) -> Self {
        Self {
            id,
            label,
            choices,
            config_key: None,
            values: None,
            no_persist: false,
        }
    }
}

/// One plugin: a stable id, a display label and 1 to [`abi::MAX_OPTIONS`] options.
#[derive(Clone, Copy, Debug)]
pub struct PluginSpec {
    /// Stable identifier, unique across plugins. Lowercase ASCII, digits, `_`.
    pub id: &'static str,
    /// Display label. A single-option plugin shows this label in the strip.
    pub label: &'static str,
    pub options: &'static [OptionSpec],
    /// Own `key=value` INI the host should update after menu edits (`sd:/...`,
    /// at most [`abi::MAX_PATH_LEN`] bytes). `None` = no host persistence.
    pub config_path: Option<&'static str>,
}

impl PluginSpec {
    /// A plugin with no host persistence (`config_path` is `None`).
    pub const fn new(
        id: &'static str,
        label: &'static str,
        options: &'static [OptionSpec],
    ) -> Self {
        Self {
            id,
            label,
            options,
            config_path: None,
        }
    }
}

/// Result of a [`PluginHandler::request`], mirroring the ABI `REQUEST_*` codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestStatus {
    /// The change took effect.
    Applied,
    /// Accepted, applies later (for example after the match).
    Pending,
    /// Refused in the current state. Nothing changed.
    Rejected,
    /// The feature behind this option is unavailable.
    Unavailable,
}

impl RequestStatus {
    const fn code(self) -> u32 {
        match self {
            RequestStatus::Applied => abi::REQUEST_APPLIED,
            RequestStatus::Pending => abi::REQUEST_PENDING,
            RequestStatus::Rejected => abi::REQUEST_REJECTED,
            RequestStatus::Unavailable => abi::REQUEST_UNAVAILABLE,
        }
    }
}

/// Plugin-side behavior behind the registered options.
///
/// Both methods run on the host's overlay draw thread: keep them quick and do
/// not block or call back into the host API. Indices are positions in the
/// [`PluginSpec`] the handler was registered with. The SDK range-checks the
/// indices the host passes, so the handler only ever sees valid ones.
pub trait PluginHandler: Send + Sync + 'static {
    /// Index of the option's current choice, or `None` for `CHOICE_UNAVAILABLE`.
    fn current(&self, option: usize) -> Option<usize>;
    /// Asked to move `option` to `choice`. The host re-reads [`Self::current`]
    /// on the next frame and never assumes the change succeeded.
    fn request(&self, option: usize, choice: usize) -> RequestStatus;
}

/// Why a registration did not produce a [`Handle`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegisterError {
    /// No Deluxe host exports the plugin API (not installed, or too old).
    HostMissing,
    /// A host table was found but its `struct_size` or `abi_version` differs.
    HostIncompatible,
    /// `REGISTER_ERR_NULL`.
    Null,
    /// `REGISTER_ERR_ABI`.
    Abi,
    /// The spec is outside the ABI limits, or `REGISTER_ERR_INVALID`.
    Invalid,
    /// `REGISTER_ERR_FULL`: the host already holds the maximum plugin count.
    Full,
    /// `REGISTER_ERR_DUPLICATE`: another plugin registered this id.
    Duplicate,
    /// `REGISTER_ERR_CALLBACKS`.
    Callbacks,
    /// A negative host result this SDK does not know.
    Unknown(i32),
}

impl RegisterError {
    fn from_code(code: i32) -> Self {
        match code {
            abi::REGISTER_ERR_NULL => RegisterError::Null,
            abi::REGISTER_ERR_ABI => RegisterError::Abi,
            abi::REGISTER_ERR_INVALID => RegisterError::Invalid,
            abi::REGISTER_ERR_FULL => RegisterError::Full,
            abi::REGISTER_ERR_DUPLICATE => RegisterError::Duplicate,
            abi::REGISTER_ERR_CALLBACKS => RegisterError::Callbacks,
            other => RegisterError::Unknown(other),
        }
    }
}

/// Non-negative registration handle returned by the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handle(pub i32);

/// Runtime lookup of the host table; `None` when absent or incompatible.
///
/// Always `None` on non-switch builds. The result is cached after the first call.
pub fn host() -> Option<&'static abi::PluginApiV1> {
    static HOST: OnceLock<Option<&'static abi::PluginApiV1>> = OnceLock::new();
    *HOST.get_or_init(lookup_host)
}

#[cfg(target_os = "switch")]
fn lookup_host() -> Option<&'static abi::PluginApiV1> {
    let mut address = 0usize;
    // SAFETY: `API_SYMBOL_C` is NUL-terminated and `address` is a valid out slot.
    unsafe {
        skyline::nn::ro::LookupSymbol(&mut address, abi::API_SYMBOL_C.as_ptr());
    }
    if address == 0 {
        return None;
    }
    // SAFETY: the symbol is the host's exported `ApiEntryFn` by contract.
    let entry = unsafe { std::mem::transmute::<usize, abi::ApiEntryFn>(address) };
    // SAFETY: calling the entry point takes no arguments and has no preconditions.
    let table = unsafe { entry() };
    if table.is_null() {
        return None;
    }
    // SAFETY: the host table lives for the life of the process; only its two
    // leading identity fields are read before `table_matches` accepts it.
    let table = unsafe { &*table };
    abi::table_matches(table.struct_size, table.abi_version).then_some(table)
}

#[cfg(not(target_os = "switch"))]
fn lookup_host() -> Option<&'static abi::PluginApiV1> {
    None
}

/// Looks up the host and registers `spec` with `handler`.
///
/// The spec is checked locally first, so misuse is reported as
/// [`RegisterError::Invalid`] even when no host is present. Then
/// [`RegisterError::HostMissing`] when [`host`] is `None`.
pub fn register<H: PluginHandler>(spec: &PluginSpec, handler: H) -> Result<Handle, RegisterError> {
    if !spec_is_valid(spec) {
        return Err(RegisterError::Invalid);
    }
    let api = host().ok_or(RegisterError::HostMissing)?;
    // SAFETY: `host()` only returns tables accepted by `table_matches`.
    unsafe { register_with(api, spec, handler) }
}

/// Registers through an explicit table (tests, embedders).
///
/// The spec is validated before the table identity is checked, and neither
/// failure calls the host. On any failure the handler is dropped; on success it
/// is leaked for the life of the process because the host keeps its pointer.
///
/// # Safety
///
/// `api` must be a valid version 1 table: its `register` function pointer must
/// be callable with the ABI contract of `deluxe_plugin_abi::PluginApiV1`.
pub unsafe fn register_with<H: PluginHandler>(
    api: &abi::PluginApiV1,
    spec: &PluginSpec,
    handler: H,
) -> Result<Handle, RegisterError> {
    if !spec_is_valid(spec) {
        return Err(RegisterError::Invalid);
    }
    if !abi::table_matches(api.struct_size, api.abi_version) {
        return Err(RegisterError::HostIncompatible);
    }
    let strings = DescStrings::new(spec).ok_or(RegisterError::Invalid)?;
    let option_descs: Vec<abi::OptionDesc> = strings
        .options
        .iter()
        .zip(spec.options)
        .map(|(option, option_spec)| abi::OptionDesc {
            id: option.id.as_ptr(),
            label: option.label.as_ptr(),
            choices: option.choice_ptrs.as_ptr(),
            choice_count: option.choice_ptrs.len() as u32,
            flags: if option_spec.no_persist {
                abi::OPTION_FLAG_NO_PERSIST
            } else {
                0
            },
            config_key: option
                .config_key
                .as_ref()
                .map_or(std::ptr::null(), |key| key.as_ptr()),
            choice_values: if option.value_ptrs.is_empty() {
                std::ptr::null()
            } else {
                option.value_ptrs.as_ptr()
            },
        })
        .collect();
    let choice_counts = spec
        .options
        .iter()
        .map(|option| option.choices.len() as u32)
        .collect();
    let user = Box::into_raw(Box::new(Registration {
        handler,
        choice_counts,
    }));
    let desc = abi::PluginDesc {
        struct_size: std::mem::size_of::<abi::PluginDesc>() as u32,
        abi_version: abi::ABI_VERSION,
        id: strings.id.as_ptr(),
        label: strings.label.as_ptr(),
        options: option_descs.as_ptr(),
        option_count: option_descs.len() as u32,
        flags: 0,
        user: user.cast::<c_void>(),
        current: Some(current_tramp::<H>),
        request: Some(request_tramp::<H>),
        config_path: strings
            .config_path
            .as_ref()
            .map_or(std::ptr::null(), |path| path.as_ptr()),
    };
    // SAFETY: `desc` and every string it points at outlive this call; the
    // caller guarantees `api.register` follows the ABI.
    let result = unsafe { (api.register)(&desc) };
    if result >= 0 {
        Ok(Handle(result))
    } else {
        // SAFETY: the host refused the registration, so it holds no copy of
        // `user`; reclaim the box allocated above exactly once.
        drop(unsafe { Box::from_raw(user) });
        Err(RegisterError::from_code(result))
    }
}

/// Handler plus the per-option choice counts used to range-check host indices.
struct Registration<H> {
    handler: H,
    choice_counts: Vec<u32>,
}

/// Owns the NUL-terminated copies handed to the host for one `register` call.
struct DescStrings {
    id: CString,
    label: CString,
    options: Vec<OptionStrings>,
    config_path: Option<CString>,
}

struct OptionStrings {
    id: CString,
    label: CString,
    // Keeps the choice strings alive; `choice_ptrs` points into them.
    _choices: Vec<CString>,
    choice_ptrs: Vec<*const c_char>,
    config_key: Option<CString>,
    // Keeps the value tokens alive; `value_ptrs` points into them. Empty when
    // the option has no value table.
    _values: Vec<CString>,
    value_ptrs: Vec<*const c_char>,
}

impl DescStrings {
    fn new(spec: &PluginSpec) -> Option<Self> {
        let mut options = Vec::with_capacity(spec.options.len());
        for option in spec.options {
            let choices = option
                .choices
                .iter()
                .map(|choice| CString::new(*choice).ok())
                .collect::<Option<Vec<_>>>()?;
            let choice_ptrs = choices.iter().map(|choice| choice.as_ptr()).collect();
            let values = option
                .values
                .unwrap_or_default()
                .iter()
                .map(|value| CString::new(*value).ok())
                .collect::<Option<Vec<_>>>()?;
            let value_ptrs = values.iter().map(|value| value.as_ptr()).collect();
            options.push(OptionStrings {
                id: CString::new(option.id).ok()?,
                label: CString::new(option.label).ok()?,
                _choices: choices,
                choice_ptrs,
                config_key: optional_cstring(option.config_key)?,
                _values: values,
                value_ptrs,
            });
        }
        Some(Self {
            id: CString::new(spec.id).ok()?,
            label: CString::new(spec.label).ok()?,
            options,
            config_path: optional_cstring(spec.config_path)?,
        })
    }
}

/// `None` stays `None`; `Some` converts, failing on an interior NUL.
fn optional_cstring(text: Option<&str>) -> Option<Option<CString>> {
    match text {
        None => Some(None),
        Some(text) => CString::new(text).ok().map(Some),
    }
}

fn spec_is_valid(spec: &PluginSpec) -> bool {
    if !abi::valid_id(spec.id.as_bytes())
        || !abi::valid_label(spec.label.as_bytes())
        || spec.options.is_empty()
        || spec.options.len() > abi::MAX_OPTIONS
    {
        return false;
    }
    if spec
        .config_path
        .is_some_and(|path| !abi::valid_config_path(path.as_bytes()))
    {
        return false;
    }
    spec.options.iter().enumerate().all(|(index, option)| {
        abi::valid_id(option.id.as_bytes())
            && abi::valid_label(option.label.as_bytes())
            && !option.choices.is_empty()
            && option.choices.len() <= abi::MAX_CHOICES
            && option
                .choices
                .iter()
                .all(|choice| abi::valid_label(choice.as_bytes()))
            && option
                .config_key
                .is_none_or(|key| abi::valid_id(key.as_bytes()))
            && option.values.is_none_or(|values| {
                values.len() == option.choices.len()
                    && values
                        .iter()
                        .all(|value| abi::valid_value(value.as_bytes()))
            })
            && !spec.options[..index]
                .iter()
                .any(|earlier| earlier.id == option.id)
    })
}

unsafe extern "C" fn current_tramp<H: PluginHandler>(user: *mut c_void, option: u32) -> u32 {
    if user.is_null() {
        return abi::CHOICE_UNAVAILABLE;
    }
    // SAFETY: `user` is the `Registration<H>` leaked by `register_with`.
    let registration = unsafe { &*user.cast::<Registration<H>>() };
    let Some(&choice_count) = registration.choice_counts.get(option as usize) else {
        return abi::CHOICE_UNAVAILABLE;
    };
    match registration.handler.current(option as usize) {
        Some(choice) if choice < choice_count as usize => choice as u32,
        _ => abi::CHOICE_UNAVAILABLE,
    }
}

unsafe extern "C" fn request_tramp<H: PluginHandler>(
    user: *mut c_void,
    option: u32,
    choice: u32,
) -> u32 {
    if user.is_null() {
        return abi::REQUEST_REJECTED;
    }
    // SAFETY: `user` is the `Registration<H>` leaked by `register_with`.
    let registration = unsafe { &*user.cast::<Registration<H>>() };
    match registration.choice_counts.get(option as usize) {
        Some(&choice_count) if choice < choice_count => registration
            .handler
            .request(option as usize, choice as usize)
            .code(),
        _ => abi::REQUEST_REJECTED,
    }
}

/// Ready-made handler: one atomic per option holding the current choice index.
///
/// `request` stores the choice and returns [`RequestStatus::Applied`]. Declare
/// it as a `static` and register it by reference:
///
/// ```
/// use deluxe_plugin_sdk::{register, AtomicOptions, OptionSpec, PluginSpec, RegisterError};
///
/// static OPTIONS: AtomicOptions<1> = AtomicOptions::new([0]);
/// static SPEC: PluginSpec = PluginSpec::new(
///     "example",
///     "Example",
///     &[OptionSpec::new("enabled", "Enabled", &["Off", "On"])],
/// );
///
/// // No Deluxe host exists off-console, so the plugin simply gets no row.
/// assert_eq!(register(&SPEC, &OPTIONS), Err(RegisterError::HostMissing));
/// assert_eq!(OPTIONS.get(0), 0);
/// ```
#[derive(Debug)]
pub struct AtomicOptions<const N: usize> {
    values: [AtomicU32; N],
}

impl<const N: usize> AtomicOptions<N> {
    /// Starts each option at the given choice index.
    pub const fn new(initial: [u32; N]) -> Self {
        let mut values = [const { AtomicU32::new(0) }; N];
        let mut index = 0;
        while index < N {
            values[index] = AtomicU32::new(initial[index]);
            index += 1;
        }
        Self { values }
    }

    /// Current choice index of `option`. Panics if `option >= N`.
    pub fn get(&self, option: usize) -> u32 {
        self.values[option].load(Ordering::Relaxed)
    }

    /// Sets the choice index of `option`. Panics if `option >= N`.
    pub fn set(&self, option: usize, choice: u32) {
        self.values[option].store(choice, Ordering::Relaxed);
    }
}

impl<const N: usize> PluginHandler for &'static AtomicOptions<N> {
    fn current(&self, option: usize) -> Option<usize> {
        self.values
            .get(option)
            .map(|value| value.load(Ordering::Relaxed) as usize)
    }

    fn request(&self, option: usize, choice: usize) -> RequestStatus {
        match (self.values.get(option), u32::try_from(choice)) {
            (Some(value), Ok(choice)) => {
                value.store(choice, Ordering::Relaxed);
                RequestStatus::Applied
            }
            _ => RequestStatus::Rejected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::ffi::CStr;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct SeenOption {
        id: String,
        label: String,
        choices: Vec<String>,
        flags: u32,
        config_key: Option<String>,
        values: Option<Vec<String>>,
    }

    #[derive(Clone)]
    struct Seen {
        struct_size: u32,
        abi_version: u32,
        flags: u32,
        id: String,
        label: String,
        options: Vec<SeenOption>,
        user: *mut c_void,
        current: Option<abi::CurrentFn>,
        request: Option<abi::RequestFn>,
        config_path: Option<String>,
    }

    thread_local! {
        static SEEN: RefCell<Option<Seen>> = const { RefCell::new(None) };
        static RESULT: Cell<i32> = const { Cell::new(0) };
        static CALLS: Cell<u32> = const { Cell::new(0) };
    }

    // Copies a NUL-terminated string. `CStr` scans for the terminator, so the
    // host-side copy only works if the SDK really supplied one.
    unsafe fn copy(ptr: *const c_char) -> String {
        assert!(!ptr.is_null());
        unsafe { CStr::from_ptr(ptr) }.to_str().unwrap().to_owned()
    }

    // Like `copy`, but null stays `None`.
    unsafe fn copy_opt(ptr: *const c_char) -> Option<String> {
        (!ptr.is_null()).then(|| unsafe { copy(ptr) })
    }

    unsafe extern "C" fn fake_register(desc: *const abi::PluginDesc) -> i32 {
        CALLS.with(|calls| calls.set(calls.get() + 1));
        let desc = unsafe { &*desc };
        let mut options = Vec::new();
        for index in 0..desc.option_count as usize {
            let option = unsafe { &*desc.options.add(index) };
            let mut choices = Vec::new();
            for choice in 0..option.choice_count as usize {
                choices.push(unsafe { copy(*option.choices.add(choice)) });
            }
            let values = (!option.choice_values.is_null()).then(|| {
                (0..option.choice_count as usize)
                    .map(|value| unsafe { copy(*option.choice_values.add(value)) })
                    .collect::<Vec<_>>()
            });
            options.push(SeenOption {
                id: unsafe { copy(option.id) },
                label: unsafe { copy(option.label) },
                choices,
                flags: option.flags,
                config_key: unsafe { copy_opt(option.config_key) },
                values,
            });
        }
        let seen = Seen {
            struct_size: desc.struct_size,
            abi_version: desc.abi_version,
            flags: desc.flags,
            config_path: unsafe { copy_opt(desc.config_path) },
            id: unsafe { copy(desc.id) },
            label: unsafe { copy(desc.label) },
            options,
            user: desc.user,
            current: desc.current,
            request: desc.request,
        };
        SEEN.with(|slot| *slot.borrow_mut() = Some(seen));
        RESULT.with(Cell::get)
    }

    unsafe extern "C" fn fake_status() -> u32 {
        abi::HOST_STATUS_MENU_ACTIVE
    }

    unsafe extern "C" fn fake_build() -> *const c_char {
        c"test-host".as_ptr()
    }

    fn fake_table(abi_version: u32) -> abi::PluginApiV1 {
        abi::PluginApiV1 {
            struct_size: std::mem::size_of::<abi::PluginApiV1>() as u32,
            abi_version,
            register: fake_register,
            host_status: fake_status,
            host_build: fake_build,
        }
    }

    fn reset(result: i32) {
        RESULT.with(|cell| cell.set(result));
        CALLS.with(|cell| cell.set(0));
        SEEN.with(|slot| *slot.borrow_mut() = None);
    }

    fn calls() -> u32 {
        CALLS.with(Cell::get)
    }

    fn seen() -> Seen {
        SEEN.with(|slot| slot.borrow().clone())
            .expect("register was called")
    }

    static SPEC_OPTIONS: [OptionSpec; 2] = [
        OptionSpec::new("enabled", "Enabled", &["Off", "On"]),
        OptionSpec::new("mode", "Mode", &["A", "B", "C"]),
    ];

    static SPEC: PluginSpec = PluginSpec::new("demo", "Demo", &SPEC_OPTIONS);

    static PERSIST_OPTIONS: [OptionSpec; 3] = [
        OptionSpec {
            id: "enabled",
            label: "Enabled",
            choices: &["Off", "On"],
            config_key: Some("enabled_key"),
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
    ];

    static PERSIST_SPEC: PluginSpec = PluginSpec {
        id: "demo",
        label: "Demo",
        options: &PERSIST_OPTIONS,
        config_path: Some("sd:/ultimate/demo/config.ini"),
    };

    /// Scriptable handler. Shared state lets the test observe calls after the
    /// handler has been leaked into the registration.
    #[derive(Default)]
    struct State {
        current: Mutex<Option<usize>>,
        status: Mutex<Option<RequestStatus>>,
        requests: Mutex<Vec<(usize, usize)>>,
    }

    struct Probe(Arc<State>);

    impl PluginHandler for Probe {
        fn current(&self, _option: usize) -> Option<usize> {
            *self.0.current.lock().unwrap()
        }

        fn request(&self, option: usize, choice: usize) -> RequestStatus {
            self.0.requests.lock().unwrap().push((option, choice));
            self.0
                .status
                .lock()
                .unwrap()
                .unwrap_or(RequestStatus::Applied)
        }
    }

    fn register_probe(state: &Arc<State>) -> Seen {
        reset(7);
        let table = fake_table(abi::ABI_VERSION);
        let handle = unsafe { register_with(&table, &SPEC, Probe(state.clone())) };
        assert_eq!(handle, Ok(Handle(7)));
        seen()
    }

    #[test]
    fn strings_and_layout_reach_the_host_intact() {
        let state = Arc::new(State::default());
        let seen = register_probe(&state);
        assert_eq!(
            seen.struct_size as usize,
            std::mem::size_of::<abi::PluginDesc>()
        );
        assert_eq!(seen.abi_version, abi::ABI_VERSION);
        assert_eq!(seen.flags, 0);
        assert_eq!(seen.id, "demo");
        assert_eq!(seen.label, "Demo");
        assert!(!seen.user.is_null());
        assert!(seen.current.is_some() && seen.request.is_some());
        assert_eq!(seen.options.len(), 2);
        assert_eq!(seen.options[0].id, "enabled");
        assert_eq!(seen.options[0].label, "Enabled");
        assert_eq!(seen.options[0].choices, ["Off", "On"]);
        assert_eq!(seen.options[1].id, "mode");
        assert_eq!(seen.options[1].label, "Mode");
        assert_eq!(seen.options[1].choices, ["A", "B", "C"]);
        assert!(seen.options.iter().all(|o| o.flags == 0));
        assert_eq!(calls(), 1);
    }

    #[test]
    fn descriptor_sizes_match_the_abi() {
        assert_eq!(std::mem::size_of::<abi::OptionDesc>(), 48);
        assert_eq!(std::mem::size_of::<abi::PluginDesc>(), 72);
        let state = Arc::new(State::default());
        assert_eq!(register_probe(&state).struct_size, 72);
    }

    #[test]
    fn config_fields_are_null_when_unset() {
        let state = Arc::new(State::default());
        let seen = register_probe(&state);
        assert_eq!(seen.config_path, None);
        assert!(seen
            .options
            .iter()
            .all(|o| o.config_key.is_none() && o.values.is_none()));
    }

    #[test]
    fn config_fields_arrive_as_nul_terminated_strings() {
        reset(4);
        let table = fake_table(abi::ABI_VERSION);
        let state = Arc::new(State::default());
        let result = unsafe { register_with(&table, &PERSIST_SPEC, Probe(state)) };
        assert_eq!(result, Ok(Handle(4)));
        let seen = seen();
        assert_eq!(seen.flags, 0);
        assert_eq!(
            seen.config_path.as_deref(),
            Some("sd:/ultimate/demo/config.ini")
        );
        assert_eq!(seen.options.len(), 3);
        assert_eq!(seen.options[0].config_key.as_deref(), Some("enabled_key"));
        assert_eq!(
            seen.options[0].values.as_deref(),
            Some(&["off".to_owned(), "on".to_owned()][..])
        );
        assert_eq!(seen.options[0].flags, 0);
        assert_eq!(seen.options[1].config_key, None);
        assert_eq!(
            seen.options[1].values.as_deref(),
            Some(&["a".to_owned(), "b".to_owned(), "c".to_owned()][..])
        );
        assert_eq!(seen.options[1].flags, 0);
        assert_eq!(seen.options[2].config_key, None);
        assert_eq!(seen.options[2].values, None);
        assert_eq!(seen.options[2].flags, abi::OPTION_FLAG_NO_PERSIST);
    }

    #[test]
    fn spec_constructors_default_the_persistence_fields() {
        let option = OptionSpec::new("id", "Label", &["x"]);
        assert_eq!(option.id, "id");
        assert_eq!(option.label, "Label");
        assert_eq!(option.choices, ["x"]);
        assert_eq!(option.config_key, None);
        assert_eq!(option.values, None);
        assert!(!option.no_persist);
        static OPTIONS: [OptionSpec; 1] = [OptionSpec::new("id", "Label", &["x"])];
        let spec = PluginSpec::new("plug", "Plug", &OPTIONS);
        assert_eq!(spec.id, "plug");
        assert_eq!(spec.label, "Plug");
        assert_eq!(spec.options.len(), 1);
        assert_eq!(spec.config_path, None);
    }

    #[test]
    fn invalid_persistence_fields_are_rejected_without_calling_the_host() {
        static GOOD: [&str; 2] = ["Off", "On"];
        static BAD_KEY: [OptionSpec; 1] = [OptionSpec {
            config_key: Some("Bad Key"),
            ..OptionSpec::new("enabled", "Enabled", &GOOD)
        }];
        static EMPTY_KEY: [OptionSpec; 1] = [OptionSpec {
            config_key: Some(""),
            ..OptionSpec::new("enabled", "Enabled", &GOOD)
        }];
        static SHORT_VALUES: [OptionSpec; 1] = [OptionSpec {
            values: Some(&["off"]),
            ..OptionSpec::new("enabled", "Enabled", &GOOD)
        }];
        static LONG_VALUES: [OptionSpec; 1] = [OptionSpec {
            values: Some(&["off", "on", "extra"]),
            ..OptionSpec::new("enabled", "Enabled", &GOOD)
        }];
        static BAD_VALUE_SPACE: [OptionSpec; 1] = [OptionSpec {
            values: Some(&["off", "o n"]),
            ..OptionSpec::new("enabled", "Enabled", &GOOD)
        }];
        static BAD_VALUE_EQUALS: [OptionSpec; 1] = [OptionSpec {
            values: Some(&["off", "a=b"]),
            ..OptionSpec::new("enabled", "Enabled", &GOOD)
        }];
        static BAD_VALUE_HASH: [OptionSpec; 1] = [OptionSpec {
            values: Some(&["#off", "on"]),
            ..OptionSpec::new("enabled", "Enabled", &GOOD)
        }];
        static EMPTY_VALUE: [OptionSpec; 1] = [OptionSpec {
            values: Some(&["", "on"]),
            ..OptionSpec::new("enabled", "Enabled", &GOOD)
        }];
        static LONG_VALUE: [OptionSpec; 1] = [OptionSpec {
            values: Some(&["off", "0123456789012345678901234"]),
            ..OptionSpec::new("enabled", "Enabled", &GOOD)
        }];
        static GOOD_OPTIONS: [OptionSpec; 1] = [OptionSpec::new("enabled", "Enabled", &GOOD)];
        // 4-byte prefix plus MAX_PATH_LEN - 3 bytes: one over MAX_PATH_LEN.
        let long_path: &'static str = Box::leak(
            format!(
                "{}{}",
                abi::CONFIG_PATH_PREFIX,
                "a".repeat(abi::MAX_PATH_LEN - 3)
            )
            .into_boxed_str(),
        );
        let with_path = |config_path| PluginSpec {
            config_path: Some(config_path),
            ..PluginSpec::new("demo", "Demo", &GOOD_OPTIONS)
        };
        let specs = [
            PluginSpec::new("demo", "Demo", &BAD_KEY),
            PluginSpec::new("demo", "Demo", &EMPTY_KEY),
            PluginSpec::new("demo", "Demo", &SHORT_VALUES),
            PluginSpec::new("demo", "Demo", &LONG_VALUES),
            PluginSpec::new("demo", "Demo", &BAD_VALUE_SPACE),
            PluginSpec::new("demo", "Demo", &BAD_VALUE_EQUALS),
            PluginSpec::new("demo", "Demo", &BAD_VALUE_HASH),
            PluginSpec::new("demo", "Demo", &EMPTY_VALUE),
            PluginSpec::new("demo", "Demo", &LONG_VALUE),
            with_path(""),
            with_path("sd:/"),
            with_path("/ultimate/demo/config.ini"),
            with_path("rom:/config.ini"),
            with_path("sd:/bad\nname.ini"),
            with_path("sd:/nul\0byte.ini"),
            with_path(long_path),
        ];
        assert_eq!(long_path.len(), abi::MAX_PATH_LEN + 1);
        let table = fake_table(abi::ABI_VERSION);
        for (index, spec) in specs.iter().enumerate() {
            reset(0);
            let state = Arc::new(State::default());
            let result = unsafe { register_with(&table, spec, Probe(state.clone())) };
            assert_eq!(result, Err(RegisterError::Invalid), "spec {index}");
            assert_eq!(calls(), 0, "spec {index} reached the host");
            assert_eq!(
                Arc::strong_count(&state),
                1,
                "spec {index} leaked the handler"
            );
        }
        // The exact maximum path length is accepted.
        reset(1);
        let state = Arc::new(State::default());
        let max_path = &long_path[..abi::MAX_PATH_LEN];
        let result = unsafe { register_with(&table, &with_path(max_path), Probe(state)) };
        assert_eq!(result, Ok(Handle(1)));
        assert_eq!(seen().config_path.as_deref(), Some(max_path));
    }

    #[test]
    fn current_maps_none_and_some() {
        let state = Arc::new(State::default());
        let seen = register_probe(&state);
        let current = seen.current.unwrap();
        *state.current.lock().unwrap() = None;
        assert_eq!(unsafe { current(seen.user, 0) }, abi::CHOICE_UNAVAILABLE);
        *state.current.lock().unwrap() = Some(1);
        assert_eq!(unsafe { current(seen.user, 0) }, 1);
        *state.current.lock().unwrap() = Some(2);
        assert_eq!(unsafe { current(seen.user, 1) }, 2);
    }

    #[test]
    fn request_maps_every_status() {
        let state = Arc::new(State::default());
        let seen = register_probe(&state);
        let request = seen.request.unwrap();
        for (status, code) in [
            (RequestStatus::Applied, abi::REQUEST_APPLIED),
            (RequestStatus::Pending, abi::REQUEST_PENDING),
            (RequestStatus::Rejected, abi::REQUEST_REJECTED),
            (RequestStatus::Unavailable, abi::REQUEST_UNAVAILABLE),
        ] {
            *state.status.lock().unwrap() = Some(status);
            assert_eq!(unsafe { request(seen.user, 1, 2) }, code);
        }
        assert_eq!(state.requests.lock().unwrap().len(), 4);
        assert!(state.requests.lock().unwrap().iter().all(|r| *r == (1, 2)));
    }

    #[test]
    fn out_of_range_indices_do_not_reach_the_handler_or_panic() {
        let state = Arc::new(State::default());
        let seen = register_probe(&state);
        let current = seen.current.unwrap();
        let request = seen.request.unwrap();
        *state.current.lock().unwrap() = Some(0);
        // Option index beyond the spec.
        assert_eq!(unsafe { current(seen.user, 2) }, abi::CHOICE_UNAVAILABLE);
        assert_eq!(
            unsafe { current(seen.user, u32::MAX) },
            abi::CHOICE_UNAVAILABLE
        );
        assert_eq!(unsafe { request(seen.user, 2, 0) }, abi::REQUEST_REJECTED);
        // Choice index beyond the option's choices (option 0 has 2).
        assert_eq!(unsafe { request(seen.user, 0, 2) }, abi::REQUEST_REJECTED);
        assert_eq!(
            unsafe { request(seen.user, 0, u32::MAX) },
            abi::REQUEST_REJECTED
        );
        // A handler answering past the choice count is unavailable, not trusted.
        *state.current.lock().unwrap() = Some(2);
        assert_eq!(unsafe { current(seen.user, 0) }, abi::CHOICE_UNAVAILABLE);
        *state.current.lock().unwrap() = Some(usize::MAX);
        assert_eq!(unsafe { current(seen.user, 1) }, abi::CHOICE_UNAVAILABLE);
        assert!(state.requests.lock().unwrap().is_empty());
        // A null user is tolerated.
        assert_eq!(
            unsafe { current(std::ptr::null_mut(), 0) },
            abi::CHOICE_UNAVAILABLE
        );
        assert_eq!(
            unsafe { request(std::ptr::null_mut(), 0, 0) },
            abi::REQUEST_REJECTED
        );
    }

    #[test]
    fn atomic_options_round_trip() {
        static OPTIONS: AtomicOptions<3> = AtomicOptions::new([0, 2, 1]);
        assert_eq!(OPTIONS.get(0), 0);
        assert_eq!(OPTIONS.get(1), 2);
        assert_eq!(OPTIONS.get(2), 1);
        OPTIONS.set(0, 1);
        assert_eq!(OPTIONS.get(0), 1);
        let handler = &OPTIONS;
        assert_eq!(handler.current(0), Some(1));
        assert_eq!(handler.current(3), None);
        assert_eq!(handler.request(2, 0), RequestStatus::Applied);
        assert_eq!(OPTIONS.get(2), 0);
        assert_eq!(handler.request(3, 0), RequestStatus::Rejected);
    }

    #[test]
    fn atomic_options_register_by_reference() {
        static OPTIONS: AtomicOptions<2> = AtomicOptions::new([0, 1]);
        reset(0);
        let table = fake_table(abi::ABI_VERSION);
        let handle = unsafe { register_with(&table, &SPEC, &OPTIONS) };
        assert_eq!(handle, Ok(Handle(0)));
        let seen = seen();
        let current = seen.current.unwrap();
        let request = seen.request.unwrap();
        assert_eq!(unsafe { current(seen.user, 0) }, 0);
        assert_eq!(unsafe { current(seen.user, 1) }, 1);
        assert_eq!(unsafe { request(seen.user, 1, 2) }, abi::REQUEST_APPLIED);
        assert_eq!(OPTIONS.get(1), 2);
        assert_eq!(unsafe { current(seen.user, 1) }, 2);
        // Choice 3 does not exist for "mode" (3 choices), so it is rejected.
        assert_eq!(unsafe { request(seen.user, 1, 3) }, abi::REQUEST_REJECTED);
        assert_eq!(OPTIONS.get(1), 2);
    }

    fn bad_specs() -> Vec<PluginSpec> {
        static LONG_LABEL: &str = "0123456789012345678901234"; // 25 bytes
        static LONG_ID: &str = "abcdefghijklmnopqrstuvwxyz0123456"; // 33 bytes
        static GOOD_CHOICES: [&str; 2] = ["Off", "On"];
        static SEVENTEEN: [&str; 17] = ["x"; 17];
        static EIGHT_OPTIONS: [OptionSpec; 9] = [OptionSpec::new("a", "A", &GOOD_CHOICES); 9];

        static DUPLICATE_IDS: [OptionSpec; 2] = [
            OptionSpec::new("same", "One", &GOOD_CHOICES),
            OptionSpec::new("same", "Two", &GOOD_CHOICES),
        ];

        macro_rules! single {
            ($name:ident, $id:expr, $label:expr, $choices:expr) => {{
                static $name: [OptionSpec; 1] = [OptionSpec::new($id, $label, $choices)];
                &$name as &'static [OptionSpec]
            }};
        }

        let plugin = PluginSpec::new;
        let good = single!(GOOD, "enabled", "Enabled", &GOOD_CHOICES);
        vec![
            plugin("Demo", "Demo", good),
            plugin("", "Demo", good),
            plugin("de mo", "Demo", good),
            plugin(LONG_ID, "Demo", good),
            plugin("demo", "", good),
            plugin("demo", LONG_LABEL, good),
            plugin("demo", "bad\nlabel", good),
            plugin("demo", "Demo", &[]),
            // 9 options is one above MAX_OPTIONS (8).
            plugin("demo", "Demo", &EIGHT_OPTIONS),
            plugin("demo", "Demo", single!(O1, "Up", "L", &GOOD_CHOICES)),
            plugin("demo", "Demo", single!(O2, "", "L", &GOOD_CHOICES)),
            plugin("demo", "Demo", single!(O3, "ok", "", &GOOD_CHOICES)),
            plugin("demo", "Demo", single!(O4, "ok", LONG_LABEL, &GOOD_CHOICES)),
            plugin("demo", "Demo", single!(O5, "ok", "L", &[])),
            plugin("demo", "Demo", single!(O6, "ok", "L", &SEVENTEEN)),
            plugin("demo", "Demo", single!(O7, "ok", "L", &["fine", ""])),
            plugin(
                "demo",
                "Demo",
                single!(O8, "ok", "L", &["fine", LONG_LABEL]),
            ),
            plugin("demo", "Demo", single!(O9, "ok", "L", &["nul\0byte"])),
            plugin("demo", "Demo", &DUPLICATE_IDS),
        ]
    }

    #[test]
    fn local_validation_rejects_without_calling_the_host() {
        let table = fake_table(abi::ABI_VERSION);
        let specs = bad_specs();
        assert_eq!(specs.len(), 19);
        for (index, spec) in specs.iter().enumerate() {
            reset(0);
            let state = Arc::new(State::default());
            let result = unsafe { register_with(&table, spec, Probe(state.clone())) };
            assert_eq!(result, Err(RegisterError::Invalid), "spec {index}");
            assert_eq!(calls(), 0, "spec {index} reached the host");
            assert_eq!(
                Arc::strong_count(&state),
                1,
                "spec {index} leaked the handler"
            );
            // register() reports the same misuse even with no host.
            let result = register(spec, Probe(state.clone()));
            assert_eq!(result, Err(RegisterError::Invalid), "spec {index}");
        }
    }

    #[test]
    fn limits_accept_exact_maxima() {
        static CHOICES: [&str; abi::MAX_CHOICES] = ["c"; abi::MAX_CHOICES];
        static OPTIONS: [OptionSpec; abi::MAX_OPTIONS] = [
            OptionSpec::new("o0", "012345678901234567890123", &CHOICES),
            OptionSpec::new("o1", "x", &CHOICES),
            OptionSpec::new("o2", "x", &CHOICES),
            OptionSpec::new("o3", "x", &CHOICES),
            OptionSpec::new("o4", "x", &CHOICES),
            OptionSpec::new("o5", "x", &CHOICES),
            OptionSpec::new("o6", "x", &CHOICES),
            OptionSpec::new("o7", "x", &CHOICES),
        ];
        let spec = PluginSpec::new(
            "abcdefghijklmnopqrstuvwxyz012345", // 32 bytes
            "Demo",
            &OPTIONS,
        );
        reset(3);
        let table = fake_table(abi::ABI_VERSION);
        let state = Arc::new(State::default());
        let result = unsafe { register_with(&table, &spec, Probe(state)) };
        assert_eq!(result, Ok(Handle(3)));
        let seen = seen();
        assert_eq!(seen.options.len(), abi::MAX_OPTIONS);
        assert_eq!(seen.options[0].choices.len(), abi::MAX_CHOICES);
    }

    #[test]
    fn register_reports_host_missing_on_the_host() {
        assert!(host().is_none());
        let state = Arc::new(State::default());
        assert_eq!(
            register(&SPEC, Probe(state.clone())),
            Err(RegisterError::HostMissing)
        );
        assert_eq!(Arc::strong_count(&state), 1);
    }

    #[test]
    fn wrong_table_identity_is_incompatible() {
        let state = Arc::new(State::default());
        reset(0);
        let wrong_version = fake_table(abi::ABI_VERSION + 1);
        assert_eq!(
            unsafe { register_with(&wrong_version, &SPEC, Probe(state.clone())) },
            Err(RegisterError::HostIncompatible)
        );
        let mut wrong_size = fake_table(abi::ABI_VERSION);
        wrong_size.struct_size -= 8;
        assert_eq!(
            unsafe { register_with(&wrong_size, &SPEC, Probe(state.clone())) },
            Err(RegisterError::HostIncompatible)
        );
        assert_eq!(calls(), 0);
        assert_eq!(Arc::strong_count(&state), 1);
    }

    #[test]
    fn host_error_codes_map_and_reclaim_the_handler() {
        let table = fake_table(abi::ABI_VERSION);
        for (code, expected) in [
            (abi::REGISTER_ERR_NULL, RegisterError::Null),
            (abi::REGISTER_ERR_ABI, RegisterError::Abi),
            (abi::REGISTER_ERR_INVALID, RegisterError::Invalid),
            (abi::REGISTER_ERR_FULL, RegisterError::Full),
            (abi::REGISTER_ERR_DUPLICATE, RegisterError::Duplicate),
            (abi::REGISTER_ERR_CALLBACKS, RegisterError::Callbacks),
            (-99, RegisterError::Unknown(-99)),
        ] {
            reset(code);
            let state = Arc::new(State::default());
            let result = unsafe { register_with(&table, &SPEC, Probe(state.clone())) };
            assert_eq!(result, Err(expected), "code {code}");
            assert_eq!(calls(), 1);
            assert_eq!(
                Arc::strong_count(&state),
                1,
                "code {code} leaked the handler"
            );
        }
    }

    #[test]
    fn success_keeps_the_handler_alive() {
        let state = Arc::new(State::default());
        let _ = register_probe(&state);
        // The registration (leaked on purpose) still owns a clone.
        assert_eq!(Arc::strong_count(&state), 2);
    }

    #[test]
    fn host_status_and_build_are_reachable_through_a_table() {
        let table = fake_table(abi::ABI_VERSION);
        assert_eq!(
            unsafe { (table.host_status)() },
            abi::HOST_STATUS_MENU_ACTIVE
        );
        let build = unsafe { CStr::from_ptr((table.host_build)()) };
        assert_eq!(build.to_str().unwrap(), "test-host");
    }
}
