//! Registry behind the Plugins page. Separately built NROs register options
//! through the C ABI in `deluxe-plugin-abi`; this module validates and stores
//! them and owns the row, page and value/request rules the overlay uses.
//!
//! Strings are copied during registration. Only the opaque `user` pointer and
//! the two callbacks are retained. Callbacks are invoked outside the registry
//! lock, on the overlay draw thread, and only while the Plugins page is shown
//! or an edge was accepted. The host never assumes a request succeeded: the
//! displayed value is always the plugin's answer to `current`.

use core::ffi::{c_char, c_void};
use deluxe_plugin_abi as abi;
use std::sync::{Arc, Mutex};

/// One registered option, strings owned by the host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginOption {
    pub id: String,
    pub label: String,
    pub choices: Vec<String>,
    /// INI key for host persistence (the option id unless overridden).
    pub config_key: String,
    /// Persisted value per choice; `None` writes the decimal index.
    pub values: Option<Vec<String>>,
    /// Cleared by `OPTION_FLAG_NO_PERSIST`.
    pub persist: bool,
    /// `TONE_*` byte per choice from a version 2 descriptor; `None` is neutral.
    pub tones: Option<Vec<u8>>,
    /// `OPTION_FLAG_READ_ONLY`: a status row whose value follows `current`.
    pub locked: bool,
}

impl PluginOption {
    /// A single choice, or a row flagged read-only, is a status row:
    /// Left/Right do nothing and `request` is never called.
    pub fn read_only(&self) -> bool {
        self.locked || self.choices.len() <= 1
    }

    /// Tone for `choice`; neutral when the plugin declared none.
    pub fn tone_of(&self, choice: usize) -> u8 {
        self.tones
            .as_ref()
            .and_then(|tones| tones.get(choice).copied())
            .unwrap_or(abi::TONE_NEUTRAL)
    }

    /// Token written for `choice` when the plugin persists.
    pub fn persisted_value(&self, choice: usize) -> String {
        match &self.values {
            Some(values) => values
                .get(choice)
                .cloned()
                .unwrap_or_else(|| choice.to_string()),
            None => choice.to_string(),
        }
    }
}

/// One registered plugin.
#[derive(Debug)]
pub struct Plugin {
    pub id: String,
    pub label: String,
    pub options: Vec<PluginOption>,
    /// The plugin's own INI the host updates after menu edits, when set.
    pub config_path: Option<String>,
    user: usize,
    current: abi::CurrentFn,
    request: abi::RequestFn,
}

/// One queued `key=value` write into a plugin's INI, drained by the runtime's
/// background thread. Same path and key coalesce to the newest value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersistWrite {
    pub plugin: usize,
    pub option: usize,
    pub path: String,
    pub key: String,
    pub value: String,
}

impl Plugin {
    /// Asks the plugin for the option's current choice index.
    pub fn current(&self, option: usize) -> Option<usize> {
        let Ok(index) = u32::try_from(option) else {
            return None;
        };
        let choice = unsafe { (self.current)(self.user as *mut c_void, index) };
        let count = self.options.get(option)?.choices.len();
        (choice != abi::CHOICE_UNAVAILABLE && (choice as usize) < count).then_some(choice as usize)
    }

    fn request(&self, option: usize, choice: usize) -> RequestStatus {
        let (Ok(option), Ok(choice)) = (u32::try_from(option), u32::try_from(choice)) else {
            return RequestStatus::Rejected;
        };
        RequestStatus::from_code(unsafe {
            (self.request)(self.user as *mut c_void, option, choice)
        })
    }
}

/// Plugin answer to a change request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestStatus {
    Applied,
    Pending,
    Rejected,
    Unavailable,
}

impl RequestStatus {
    pub fn from_code(code: u32) -> Self {
        match code {
            abi::REQUEST_APPLIED => Self::Applied,
            abi::REQUEST_PENDING => Self::Pending,
            abi::REQUEST_UNAVAILABLE => Self::Unavailable,
            _ => Self::Rejected,
        }
    }

    pub const fn code(self) -> u32 {
        match self {
            Self::Applied => abi::REQUEST_APPLIED,
            Self::Pending => abi::REQUEST_PENDING,
            Self::Rejected => abi::REQUEST_REJECTED,
            Self::Unavailable => abi::REQUEST_UNAVAILABLE,
        }
    }
}

/// Outcome of one Left/Right edge on a plugin row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepResult {
    pub plugin: usize,
    pub option: usize,
    pub requested: usize,
    pub status: RequestStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PendingChoice {
    plugin: usize,
    option: usize,
    choice: usize,
}

/// Displayed value of one option.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValueText {
    pub label: String,
    /// A `Pending` request has not yet become the plugin's current answer.
    pub pending: bool,
    /// `TONE_*` byte for the displayed choice; neutral for `Unavailable`.
    pub tone: u8,
}

/// Registered plugins plus the host-side pending bookkeeping. The runtime uses
/// one global instance; tests use their own so parallel tests do not race.
pub struct Registry {
    plugins: Mutex<Vec<Arc<Plugin>>>,
    pending: Mutex<Vec<PendingChoice>>,
    last_step: Mutex<Option<StepResult>>,
    writes: Mutex<Vec<PersistWrite>>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

/// Reads a NUL-terminated string of at most `max` bytes. `None` when the
/// pointer is null or no terminator appears within `max + 1` bytes, so a bad
/// descriptor cannot make the host scan unbounded memory.
///
/// # Safety
/// `ptr` must be readable for `max + 1` bytes or NUL-terminated before that.
unsafe fn bounded_cstr<'a>(ptr: *const c_char, max: usize) -> Option<&'a [u8]> {
    if ptr.is_null() {
        return None;
    }
    let bytes = ptr.cast::<u8>();
    for len in 0..=max {
        if *bytes.add(len) == 0 {
            return Some(std::slice::from_raw_parts(bytes, len));
        }
    }
    None
}

unsafe fn owned_id(ptr: *const c_char) -> Result<String, i32> {
    let bytes = bounded_cstr(ptr, abi::MAX_ID_LEN).ok_or(if ptr.is_null() {
        abi::REGISTER_ERR_NULL
    } else {
        abi::REGISTER_ERR_INVALID
    })?;
    if !abi::valid_id(bytes) {
        return Err(abi::REGISTER_ERR_INVALID);
    }
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

unsafe fn owned_label(ptr: *const c_char) -> Result<String, i32> {
    let bytes = bounded_cstr(ptr, abi::MAX_LABEL_LEN).ok_or(if ptr.is_null() {
        abi::REGISTER_ERR_NULL
    } else {
        abi::REGISTER_ERR_INVALID
    })?;
    if !abi::valid_label(bytes) {
        return Err(abi::REGISTER_ERR_INVALID);
    }
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

/// Non-null pointer required; `valid` decides acceptance.
unsafe fn owned_checked(
    ptr: *const c_char,
    max: usize,
    valid: fn(&[u8]) -> bool,
) -> Result<String, i32> {
    let bytes = bounded_cstr(ptr, max).ok_or(if ptr.is_null() {
        abi::REGISTER_ERR_NULL
    } else {
        abi::REGISTER_ERR_INVALID
    })?;
    if !valid(bytes) {
        return Err(abi::REGISTER_ERR_INVALID);
    }
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

/// Validated, host-owned copy of a descriptor, ready to insert.
#[derive(Debug)]
pub struct ParsedPlugin {
    pub id: String,
    pub label: String,
    pub options: Vec<PluginOption>,
    pub config_path: Option<String>,
    pub user: usize,
    pub current: abi::CurrentFn,
    pub request: abi::RequestFn,
}

/// Copies and validates a raw descriptor without touching any registry.
///
/// # Safety
/// `desc` must be null or point to a `PluginDesc` whose pointer fields are
/// null or valid for the duration of the call.
pub unsafe fn parse_descriptor(desc: *const abi::PluginDesc) -> Result<ParsedPlugin, i32> {
    if desc.is_null() {
        return Err(abi::REGISTER_ERR_NULL);
    }
    let desc = &*desc;
    if desc.struct_size as usize != core::mem::size_of::<abi::PluginDesc>()
        || !abi::descriptor_version_supported(desc.abi_version)
    {
        return Err(abi::REGISTER_ERR_ABI);
    }
    if desc.flags != 0 {
        return Err(abi::REGISTER_ERR_INVALID);
    }
    let (Some(current), Some(request)) = (desc.current, desc.request) else {
        return Err(abi::REGISTER_ERR_CALLBACKS);
    };
    let id = owned_id(desc.id)?;
    let label = owned_label(desc.label)?;
    let config_path = if desc.config_path.is_null() {
        None
    } else {
        Some(owned_checked(
            desc.config_path,
            abi::MAX_PATH_LEN,
            abi::valid_config_path,
        )?)
    };
    let option_count = desc.option_count as usize;
    if option_count == 0 || option_count > abi::MAX_OPTIONS {
        return Err(abi::REGISTER_ERR_INVALID);
    }
    if desc.options.is_null() {
        return Err(abi::REGISTER_ERR_NULL);
    }
    // Version 1 rows are 48 bytes without tones; widen them to the current layout.
    let raw_options: Vec<abi::OptionDesc> = if desc.abi_version == 1 {
        std::slice::from_raw_parts(desc.options.cast::<abi::OptionDescV1>(), option_count)
            .iter()
            .map(|raw| raw.widen())
            .collect()
    } else {
        std::slice::from_raw_parts(desc.options, option_count).to_vec()
    };
    let allowed_flags = if desc.abi_version >= 2 {
        abi::OPTION_FLAG_NO_PERSIST | abi::OPTION_FLAG_READ_ONLY
    } else {
        abi::OPTION_FLAG_NO_PERSIST
    };
    let mut options = Vec::with_capacity(option_count);
    for raw in &raw_options {
        if raw.flags & !allowed_flags != 0 {
            return Err(abi::REGISTER_ERR_INVALID);
        }
        let option_id = owned_id(raw.id)?;
        if options.iter().any(|o: &PluginOption| o.id == option_id) {
            return Err(abi::REGISTER_ERR_INVALID);
        }
        let option_label = owned_label(raw.label)?;
        let choice_count = raw.choice_count as usize;
        if choice_count == 0 || choice_count > abi::MAX_CHOICES {
            return Err(abi::REGISTER_ERR_INVALID);
        }
        if raw.choices.is_null() {
            return Err(abi::REGISTER_ERR_NULL);
        }
        let raw_choices = std::slice::from_raw_parts(raw.choices, choice_count);
        let mut choices = Vec::with_capacity(choice_count);
        for choice in raw_choices {
            choices.push(owned_label(*choice)?);
        }
        let config_key = if raw.config_key.is_null() {
            option_id.clone()
        } else {
            owned_id(raw.config_key)?
        };
        let values = if raw.choice_values.is_null() {
            None
        } else {
            let raw_values = std::slice::from_raw_parts(raw.choice_values, choice_count);
            let mut values = Vec::with_capacity(choice_count);
            for value in raw_values {
                values.push(owned_checked(*value, abi::MAX_VALUE_LEN, abi::valid_value)?);
            }
            Some(values)
        };
        let tones = if raw.choice_tones.is_null() {
            None
        } else {
            let bytes = std::slice::from_raw_parts(raw.choice_tones, choice_count);
            if !abi::valid_tones(bytes) {
                return Err(abi::REGISTER_ERR_INVALID);
            }
            Some(bytes.to_vec())
        };
        let locked = raw.flags & abi::OPTION_FLAG_READ_ONLY != 0;
        options.push(PluginOption {
            id: option_id,
            label: option_label,
            choices,
            config_key,
            values,
            // Read-only rows are never edited, so there is nothing to persist.
            persist: raw.flags & abi::OPTION_FLAG_NO_PERSIST == 0 && !locked,
            tones,
            locked,
        });
    }
    if options
        .iter()
        .filter(|o| o.persist)
        .enumerate()
        .any(|(i, o)| {
            options
                .iter()
                .filter(|p| p.persist)
                .take(i)
                .any(|p| p.config_key == o.config_key)
        })
    {
        return Err(abi::REGISTER_ERR_INVALID);
    }
    Ok(ParsedPlugin {
        id,
        label,
        options,
        config_path,
        user: desc.user as usize,
        current,
        request,
    })
}

/// Replaces the `key=value` line in an INI text, or appends it. Other lines,
/// comments and blank lines are kept. The result always ends with a newline.
pub fn apply_ini_assignment(text: &str, key: &str, value: &str) -> String {
    let assignment = format!("{key}={value}");
    let mut replaced = false;
    let mut out = String::with_capacity(text.len() + assignment.len() + 1);
    for line in text.lines() {
        let content = line.split('#').next().unwrap_or_default().trim();
        let matches = content
            .split_once('=')
            .is_some_and(|(existing, _)| existing.trim() == key);
        if matches && !replaced {
            out.push_str(&assignment);
            replaced = true;
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    if !replaced {
        out.push_str(&assignment);
        out.push('\n');
    }
    out
}

impl Registry {
    pub const fn new() -> Self {
        Self {
            plugins: Mutex::new(Vec::new()),
            pending: Mutex::new(Vec::new()),
            last_step: Mutex::new(None),
            writes: Mutex::new(Vec::new()),
        }
    }

    /// Drains queued INI writes for the background thread.
    pub fn take_writes(&self) -> Vec<PersistWrite> {
        self.writes
            .lock()
            .map(|mut writes| std::mem::take(&mut *writes))
            .unwrap_or_default()
    }

    fn queue_write(&self, write: PersistWrite) {
        let Ok(mut writes) = self.writes.lock() else {
            return;
        };
        writes.retain(|w| !(w.path == write.path && w.key == write.key));
        writes.push(write);
    }

    pub fn count(&self) -> usize {
        self.plugins.lock().map(|p| p.len()).unwrap_or(0)
    }

    /// Registration order is the display order.
    pub fn snapshot(&self) -> Vec<Arc<Plugin>> {
        self.plugins.lock().map(|p| p.clone()).unwrap_or_default()
    }

    /// Inserts a validated plugin. Returns its handle or a `REGISTER_ERR_*` code.
    pub fn register(&self, parsed: ParsedPlugin) -> i32 {
        let Ok(mut plugins) = self.plugins.lock() else {
            return abi::REGISTER_ERR_FULL;
        };
        if plugins.len() >= abi::MAX_PLUGINS {
            return abi::REGISTER_ERR_FULL;
        }
        if plugins.iter().any(|p| p.id == parsed.id) {
            return abi::REGISTER_ERR_DUPLICATE;
        }
        plugins.push(Arc::new(Plugin {
            id: parsed.id,
            label: parsed.label,
            options: parsed.options,
            config_path: parsed.config_path,
            user: parsed.user,
            current: parsed.current,
            request: parsed.request,
        }));
        (plugins.len() - 1) as i32
    }

    /// The C entry: parse, then insert.
    ///
    /// # Safety
    /// See [`parse_descriptor`].
    pub unsafe fn register_raw(&self, desc: *const abi::PluginDesc) -> i32 {
        match parse_descriptor(desc) {
            Ok(parsed) => self.register(parsed),
            Err(code) => code,
        }
    }

    fn pending_for(&self, plugin: usize, option: usize) -> Option<usize> {
        self.pending
            .lock()
            .ok()?
            .iter()
            .find_map(|p| (p.plugin == plugin && p.option == option).then_some(p.choice))
    }

    fn set_pending(&self, plugin: usize, option: usize, choice: Option<usize>) {
        let Ok(mut pending) = self.pending.lock() else {
            return;
        };
        pending.retain(|p| !(p.plugin == plugin && p.option == option));
        if let Some(choice) = choice {
            pending.push(PendingChoice {
                plugin,
                option,
                choice,
            });
        }
    }

    /// Current display text. A satisfied pending request is cleared here.
    pub fn value(&self, plugin_index: usize, plugin: &Plugin, option: usize) -> ValueText {
        let Some(spec) = plugin.options.get(option) else {
            return ValueText {
                label: "Unavailable".to_owned(),
                pending: false,
                tone: abi::TONE_NEUTRAL,
            };
        };
        let current = plugin.current(option);
        let pending = self.pending_for(plugin_index, option);
        let pending = match (pending, current) {
            (Some(wanted), Some(now)) if wanted == now => {
                self.set_pending(plugin_index, option, None);
                false
            }
            (Some(_), _) => true,
            (None, _) => false,
        };
        ValueText {
            label: current
                .and_then(|index| spec.choices.get(index).cloned())
                .unwrap_or_else(|| "Unavailable".to_owned()),
            pending,
            tone: current.map_or(abi::TONE_NEUTRAL, |index| spec.tone_of(index)),
        }
    }

    /// One Left (-1) or Right (+1) edge on a row. Read-only and unavailable
    /// options make no request. The plugin's answer is recorded for the
    /// runtime to log and returned to the caller.
    pub fn step(&self, plugin_index: usize, option: usize, direction: i8) -> Option<StepResult> {
        if direction == 0 {
            return None;
        }
        let plugin = self.snapshot().get(plugin_index)?.clone();
        let spec = plugin.options.get(option)?;
        if spec.read_only() {
            return None;
        }
        let current = plugin.current(option)?;
        let requested = cycle_choice(current, spec.choices.len(), direction);
        let status = plugin.request(option, requested);
        self.set_pending(
            plugin_index,
            option,
            (status == RequestStatus::Pending).then_some(requested),
        );
        if let Some(path) = plugin.config_path.as_ref().filter(|_| {
            spec.persist && matches!(status, RequestStatus::Applied | RequestStatus::Pending)
        }) {
            self.queue_write(PersistWrite {
                plugin: plugin_index,
                option,
                path: path.clone(),
                key: spec.config_key.clone(),
                value: spec.persisted_value(requested),
            });
        }
        let result = StepResult {
            plugin: plugin_index,
            option,
            requested,
            status,
        };
        if let Ok(mut last) = self.last_step.lock() {
            *last = Some(result);
        }
        Some(result)
    }

    /// The most recent step since the previous call, for diagnostics.
    pub fn take_last_step(&self) -> Option<StepResult> {
        self.last_step.lock().ok()?.take()
    }
}

/// Right walks forward and wraps; Left walks backward and wraps.
pub fn cycle_choice(current: usize, count: usize, direction: i8) -> usize {
    if count == 0 {
        return 0;
    }
    let count = count as i64;
    ((current as i64 + i64::from(direction)).rem_euclid(count)) as usize
}

/// Flattened navigable row: one option of one plugin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginRow {
    pub plugin: usize,
    pub option: usize,
}

pub fn plugin_rows(plugins: &[Arc<Plugin>]) -> Vec<PluginRow> {
    plugins
        .iter()
        .enumerate()
        .flat_map(|(plugin, entry)| {
            (0..entry.options.len()).map(move |option| PluginRow { plugin, option })
        })
        .collect()
}

/// Steps to the adjacent row, wrapping. An empty list keeps zero.
pub fn move_plugin_row(selected: usize, direction: i8, total: usize) -> usize {
    if total == 0 {
        return 0;
    }
    let total = total as i64;
    ((selected.min(total as usize - 1) as i64 + i64::from(direction)).rem_euclid(total)) as usize
}

/// Fields shown per compact-strip page. The strip never scales text below
/// readability for a long plugin list; it pages instead.
pub const COMPACT_PAGE: usize = 4;

/// Rows per vertical window before the list is clipped around the selection.
pub const VERTICAL_WINDOW: usize = 10;

/// Page containing `selected`: `(start, end, page_index, page_count)`,
/// one-based page numbers for display.
pub fn compact_page(selected: usize, total: usize, page: usize) -> (usize, usize, usize, usize) {
    if total == 0 || page == 0 {
        return (0, 0, 1, 1);
    }
    let pages = total.div_ceil(page);
    let index = selected.min(total - 1) / page;
    let start = index * page;
    (start, (start + page).min(total), index + 1, pages)
}

/// Contiguous window of at most `window` rows containing `selected`.
pub fn vertical_window(selected: usize, total: usize, window: usize) -> (usize, usize) {
    if total == 0 || window == 0 {
        return (0, 0);
    }
    if total <= window {
        return (0, total);
    }
    let selected = selected.min(total - 1);
    let half = window / 2;
    let start = selected.saturating_sub(half).min(total - window);
    (start, start + window)
}

/// Single-option plugins show their plugin label; others `Plugin.Option`.
pub fn compact_label(plugin_label: &str, option_label: &str, option_count: usize) -> String {
    if option_count <= 1 {
        plugin_label.to_owned()
    } else {
        format!("{plugin_label}.{option_label}")
    }
}

/// One unwrapped compact row for a page of plugin fields.
pub fn compact_line(fields: &[(String, ValueText, bool)]) -> String {
    compact_segments(fields)
        .into_iter()
        .map(|(text, _)| text)
        .collect()
}

/// The same row split into `(text, tone)` segments so the renderer can colour
/// each value: `>Label:` and the separators stay neutral, the value and its
/// pending marker take the choice's tone. Concatenating the texts gives
/// [`compact_line`] exactly, so font fitting can use the joined string.
pub fn compact_segments(fields: &[(String, ValueText, bool)]) -> Vec<(String, u8)> {
    let mut segments = Vec::with_capacity(fields.len() * 3);
    for (index, (label, value, selected)) in fields.iter().enumerate() {
        if index > 0 {
            segments.push(("   ".to_owned(), abi::TONE_NEUTRAL));
        }
        segments.push((
            format!("{}{label}:", if *selected { ">" } else { "" }),
            abi::TONE_NEUTRAL,
        ));
        segments.push((
            format!("{}{}", value.label, if value.pending { "~" } else { "" }),
            value.tone,
        ));
    }
    segments
}

/// Process-wide registry used by the runtime export and the overlay.
pub static REGISTRY: Registry = Registry::new();

/// Test and host-side helper: a descriptor with owned C strings.
#[cfg(test)]
pub(crate) mod testing {
    use super::*;
    use std::ffi::CString;

    pub struct DescHolder {
        _strings: Vec<CString>,
        _choice_tables: Vec<Vec<*const c_char>>,
        _tone_tables: Vec<Vec<u8>>,
        pub _options: Vec<abi::OptionDesc>,
        pub _options_v1: Vec<abi::OptionDescV1>,
        pub desc: abi::PluginDesc,
    }

    impl DescHolder {
        /// Re-points the descriptor at a version 1 (48-byte, tone-less) option
        /// array, as an older consumer would send it.
        pub fn as_version_1(&mut self) {
            self._options_v1 = self
                ._options
                .iter()
                .map(|o| abi::OptionDescV1 {
                    id: o.id,
                    label: o.label,
                    choices: o.choices,
                    choice_count: o.choice_count,
                    flags: o.flags,
                    config_key: o.config_key,
                    choice_values: o.choice_values.cast::<c_char>(),
                })
                .collect();
            self.desc.abi_version = 1;
            self.desc.options = self._options_v1.as_ptr().cast::<abi::OptionDesc>();
        }
    }

    /// Persistence and presentation settings for one option in [`descriptor_with`].
    #[derive(Clone, Copy, Default)]
    pub struct Persist {
        pub key: Option<&'static str>,
        pub values: Option<&'static [&'static str]>,
        pub no_persist: bool,
        pub tones: Option<&'static [u8]>,
        pub read_only: bool,
    }

    pub fn descriptor(
        id: &str,
        label: &str,
        options: &[(&str, &str, &[&str])],
        user: *mut c_void,
        current: Option<abi::CurrentFn>,
        request: Option<abi::RequestFn>,
    ) -> DescHolder {
        descriptor_with(id, label, options, None, &[], user, current, request)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn descriptor_with(
        id: &str,
        label: &str,
        options: &[(&str, &str, &[&str])],
        config_path: Option<&str>,
        persist: &[Persist],
        user: *mut c_void,
        current: Option<abi::CurrentFn>,
        request: Option<abi::RequestFn>,
    ) -> DescHolder {
        let mut strings = Vec::new();
        let mut keep = |text: &str| -> *const c_char {
            let c = CString::new(text).unwrap();
            let ptr = c.as_ptr();
            strings.push(c);
            ptr
        };
        let id_ptr = keep(id);
        let label_ptr = keep(label);
        let path_ptr = config_path.map_or(std::ptr::null(), &mut keep);
        let mut choice_tables = Vec::new();
        let mut tone_tables: Vec<Vec<u8>> = Vec::new();
        let mut raw_options = Vec::new();
        for (index, (oid, olabel, choices)) in options.iter().enumerate() {
            let oid_ptr = keep(oid);
            let olabel_ptr = keep(olabel);
            let settings = persist.get(index).copied().unwrap_or_default();
            let key_ptr = settings.key.map_or(std::ptr::null(), &mut keep);
            let table: Vec<*const c_char> = choices.iter().map(|c| keep(c)).collect();
            choice_tables.push(table);
            let values_ptr = match settings.values {
                Some(values) => {
                    let table: Vec<*const c_char> = values.iter().map(|v| keep(v)).collect();
                    choice_tables.push(table);
                    choice_tables.last().unwrap().as_ptr()
                }
                None => std::ptr::null(),
            };
            let table = if settings.values.is_some() {
                &choice_tables[choice_tables.len() - 2]
            } else {
                choice_tables.last().unwrap()
            };
            let tones_ptr = match settings.tones {
                Some(tones) => {
                    tone_tables.push(tones.to_vec());
                    tone_tables.last().unwrap().as_ptr()
                }
                None => std::ptr::null(),
            };
            let mut flags = 0;
            if settings.no_persist {
                flags |= abi::OPTION_FLAG_NO_PERSIST;
            }
            if settings.read_only {
                flags |= abi::OPTION_FLAG_READ_ONLY;
            }
            raw_options.push(abi::OptionDesc {
                id: oid_ptr,
                label: olabel_ptr,
                choices: table.as_ptr(),
                choice_count: table.len() as u32,
                flags,
                config_key: key_ptr,
                choice_values: values_ptr,
                choice_tones: tones_ptr,
            });
        }
        let desc = abi::PluginDesc {
            struct_size: core::mem::size_of::<abi::PluginDesc>() as u32,
            abi_version: abi::ABI_VERSION,
            id: id_ptr,
            label: label_ptr,
            options: raw_options.as_ptr(),
            option_count: raw_options.len() as u32,
            flags: 0,
            user,
            current,
            request,
            config_path: path_ptr,
        };
        DescHolder {
            _strings: strings,
            _choice_tables: choice_tables,
            _tone_tables: tone_tables,
            _options: raw_options,
            _options_v1: Vec::new(),
            desc,
        }
    }

    /// Fake plugin state: one atomic per option plus a scripted request answer.
    pub struct FakePlugin {
        pub values: [std::sync::atomic::AtomicU32; 4],
        pub answer: std::sync::atomic::AtomicU32,
        pub requests: Mutex<Vec<(u32, u32)>>,
    }

    impl FakePlugin {
        pub const fn new(initial: [u32; 4], answer: u32) -> Self {
            use std::sync::atomic::AtomicU32;
            Self {
                values: [
                    AtomicU32::new(initial[0]),
                    AtomicU32::new(initial[1]),
                    AtomicU32::new(initial[2]),
                    AtomicU32::new(initial[3]),
                ],
                answer: AtomicU32::new(answer),
                requests: Mutex::new(Vec::new()),
            }
        }
    }

    pub unsafe extern "C" fn fake_current(user: *mut c_void, option: u32) -> u32 {
        let fake = &*(user as *const FakePlugin);
        fake.values
            .get(option as usize)
            .map(|v| v.load(std::sync::atomic::Ordering::Relaxed))
            .unwrap_or(abi::CHOICE_UNAVAILABLE)
    }

    pub unsafe extern "C" fn fake_request(user: *mut c_void, option: u32, choice: u32) -> u32 {
        let fake = &*(user as *const FakePlugin);
        fake.requests.lock().unwrap().push((option, choice));
        let answer = fake.answer.load(std::sync::atomic::Ordering::Relaxed);
        if answer == abi::REQUEST_APPLIED {
            if let Some(slot) = fake.values.get(option as usize) {
                slot.store(choice, std::sync::atomic::Ordering::Relaxed);
            }
        }
        answer
    }

    pub fn register_fake(
        registry: &Registry,
        id: &str,
        label: &str,
        options: &[(&str, &str, &[&str])],
        fake: &'static FakePlugin,
    ) -> i32 {
        let holder = descriptor(
            id,
            label,
            options,
            fake as *const FakePlugin as *mut c_void,
            Some(fake_current),
            Some(fake_request),
        );
        unsafe { registry.register_raw(&holder.desc) }
    }
}

#[cfg(test)]
mod tests {
    use super::testing::*;
    use super::*;
    use std::sync::atomic::Ordering;

    static TOGGLE: FakePlugin = FakePlugin::new([0, 0, 0, 0], abi::REQUEST_APPLIED);
    static PENDING: FakePlugin = FakePlugin::new([1, 0, 0, 0], abi::REQUEST_PENDING);
    static REJECTING: FakePlugin = FakePlugin::new([2, 0, 0, 0], abi::REQUEST_REJECTED);
    static GONE: FakePlugin =
        FakePlugin::new([abi::CHOICE_UNAVAILABLE, 0, 0, 0], abi::REQUEST_UNAVAILABLE);

    const DEMO: &[(&str, &str, &[&str])] = &[
        ("enabled", "Enabled", &["Off", "On"]),
        ("mode", "Mode", &["A", "B", "C"]),
        ("status", "Status", &["Ready"]),
    ];

    #[test]
    fn registration_copies_strings_and_assigns_ordered_handles() {
        let registry = Registry::new();
        assert_eq!(registry.count(), 0);
        assert_eq!(register_fake(&registry, "demo", "Demo", DEMO, &TOGGLE), 0);
        assert_eq!(
            register_fake(
                &registry,
                "replay",
                "Replay",
                &[("record", "Record", &["Off", "On"])],
                &PENDING
            ),
            1
        );
        assert_eq!(
            register_fake(&registry, "demo", "Other", DEMO, &TOGGLE),
            abi::REGISTER_ERR_DUPLICATE
        );
        let plugins = registry.snapshot();
        assert_eq!(plugins.len(), 2);
        assert_eq!(plugins[0].id, "demo");
        assert_eq!(plugins[0].label, "Demo");
        assert_eq!(plugins[0].options.len(), 3);
        assert_eq!(plugins[0].options[1].choices, vec!["A", "B", "C"]);
        assert!(plugins[0].options[2].read_only());
        assert!(!plugins[0].options[0].read_only());
        assert_eq!(plugins[1].label, "Replay");
        assert_eq!(
            plugin_rows(&plugins),
            vec![
                PluginRow {
                    plugin: 0,
                    option: 0
                },
                PluginRow {
                    plugin: 0,
                    option: 1
                },
                PluginRow {
                    plugin: 0,
                    option: 2
                },
                PluginRow {
                    plugin: 1,
                    option: 0
                },
            ]
        );
    }

    #[test]
    fn invalid_descriptors_are_refused_with_specific_codes() {
        let registry = Registry::new();
        assert_eq!(
            unsafe { registry.register_raw(std::ptr::null()) },
            abi::REGISTER_ERR_NULL
        );
        let user = &TOGGLE as *const FakePlugin as *mut c_void;
        let mut holder = descriptor(
            "demo",
            "Demo",
            DEMO,
            user,
            Some(fake_current),
            Some(fake_request),
        );
        holder.desc.abi_version = abi::ABI_VERSION + 1;
        assert_eq!(
            unsafe { registry.register_raw(&holder.desc) },
            abi::REGISTER_ERR_ABI
        );
        holder.desc.abi_version = 0;
        assert_eq!(
            unsafe { registry.register_raw(&holder.desc) },
            abi::REGISTER_ERR_ABI
        );
        holder.desc.abi_version = abi::ABI_VERSION;
        holder.desc.struct_size += 8;
        assert_eq!(
            unsafe { registry.register_raw(&holder.desc) },
            abi::REGISTER_ERR_ABI
        );
        holder.desc.struct_size = core::mem::size_of::<abi::PluginDesc>() as u32;
        holder.desc.flags = 1;
        assert_eq!(
            unsafe { registry.register_raw(&holder.desc) },
            abi::REGISTER_ERR_INVALID
        );
        holder.desc.flags = 0;
        holder.desc.request = None;
        assert_eq!(
            unsafe { registry.register_raw(&holder.desc) },
            abi::REGISTER_ERR_CALLBACKS
        );
        holder.desc.request = Some(fake_request);
        let saved_id = holder.desc.id;
        holder.desc.id = std::ptr::null();
        assert_eq!(
            unsafe { registry.register_raw(&holder.desc) },
            abi::REGISTER_ERR_NULL
        );
        holder.desc.id = saved_id;
        holder.desc.option_count = 0;
        assert_eq!(
            unsafe { registry.register_raw(&holder.desc) },
            abi::REGISTER_ERR_INVALID
        );
        holder.desc.option_count = 3;
        assert_eq!(unsafe { registry.register_raw(&holder.desc) }, 0);

        for (id, label, options) in [
            ("Bad Id", "Demo", DEMO),
            ("ok", "", DEMO),
            (
                "ok",
                "Demo",
                &[("dup", "A", &["x"] as &[&str]), ("dup", "B", &["y"])][..],
            ),
            ("ok", "Demo", &[("none", "None", &[][..])][..]),
            (
                "ok",
                "Demo",
                &[("bad", "Line\nbreak", &["x"] as &[&str])][..],
            ),
        ] {
            let holder = descriptor(
                id,
                label,
                options,
                user,
                Some(fake_current),
                Some(fake_request),
            );
            assert_eq!(
                unsafe { registry.register_raw(&holder.desc) },
                abi::REGISTER_ERR_INVALID,
                "{id} {label}"
            );
        }
        let long = "x".repeat(abi::MAX_LABEL_LEN + 1);
        let holder = descriptor(
            "ok",
            &long,
            DEMO,
            user,
            Some(fake_current),
            Some(fake_request),
        );
        assert_eq!(
            unsafe { registry.register_raw(&holder.desc) },
            abi::REGISTER_ERR_INVALID
        );
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn capacity_is_bounded() {
        let registry = Registry::new();
        for n in 0..abi::MAX_PLUGINS {
            assert_eq!(
                register_fake(
                    &registry,
                    &format!("p{n}"),
                    "P",
                    &[("o", "O", &["a", "b"])],
                    &TOGGLE
                ),
                n as i32
            );
        }
        assert_eq!(
            register_fake(
                &registry,
                "overflow",
                "P",
                &[("o", "O", &["a", "b"])],
                &TOGGLE
            ),
            abi::REGISTER_ERR_FULL
        );
    }

    #[test]
    fn steps_request_adjacent_choices_and_display_follows_the_plugin() {
        let registry = Registry::new();
        register_fake(&registry, "demo", "Demo", DEMO, &TOGGLE);
        let plugin = registry.snapshot()[0].clone();
        assert_eq!(
            registry.value(0, &plugin, 0),
            ValueText {
                label: "Off".to_owned(),
                pending: false,
                tone: abi::TONE_NEUTRAL,
            }
        );
        let step = registry.step(0, 0, 1).unwrap();
        assert_eq!(step.requested, 1);
        assert_eq!(step.status, RequestStatus::Applied);
        assert_eq!(registry.value(0, &plugin, 0).label, "On");
        assert_eq!(registry.step(0, 0, 1).unwrap().requested, 0);
        assert_eq!(registry.step(0, 0, -1).unwrap().requested, 1);
        assert_eq!(registry.step(0, 1, -1).unwrap().requested, 2);
        assert_eq!(registry.value(0, &plugin, 1).label, "C");
        assert_eq!(registry.step(0, 2, 1), None);
        assert_eq!(registry.step(0, 0, 0), None);
        assert_eq!(registry.step(0, 9, 1), None);
        assert_eq!(registry.step(5, 0, 1), None);
        assert_eq!(registry.take_last_step().unwrap().option, 1);
        assert_eq!(registry.take_last_step(), None);
        TOGGLE.values[0].store(0, Ordering::Relaxed);
        TOGGLE.values[1].store(0, Ordering::Relaxed);
    }

    #[test]
    fn pending_rejected_and_unavailable_answers_are_not_treated_as_applied() {
        let registry = Registry::new();
        register_fake(
            &registry,
            "pending",
            "Pending",
            &[("r", "Record", &["Off", "On", "Auto"])],
            &PENDING,
        );
        register_fake(
            &registry,
            "rejecting",
            "Rejecting",
            &[("m", "Mode", &["A", "B", "C"])],
            &REJECTING,
        );
        register_fake(
            &registry,
            "gone",
            "Gone",
            &[("g", "Gone", &["A", "B"])],
            &GONE,
        );
        let plugins = registry.snapshot();
        let step = registry.step(0, 0, 1).unwrap();
        assert_eq!(step.status, RequestStatus::Pending);
        assert_eq!(step.requested, 2);
        assert_eq!(
            registry.value(0, &plugins[0], 0),
            ValueText {
                label: "On".to_owned(),
                pending: true,
                tone: abi::TONE_NEUTRAL,
            }
        );
        PENDING.values[0].store(2, Ordering::Relaxed);
        assert_eq!(
            registry.value(0, &plugins[0], 0),
            ValueText {
                label: "Auto".to_owned(),
                pending: false,
                tone: abi::TONE_NEUTRAL,
            }
        );
        PENDING.values[0].store(1, Ordering::Relaxed);
        assert!(!registry.value(0, &plugins[0], 0).pending);

        let step = registry.step(1, 0, 1).unwrap();
        assert_eq!(step.status, RequestStatus::Rejected);
        assert_eq!(registry.value(1, &plugins[1], 0).label, "C");
        assert!(!registry.value(1, &plugins[1], 0).pending);
        assert_eq!(REJECTING.requests.lock().unwrap().as_slice(), &[(0, 0)]);

        assert_eq!(registry.step(2, 0, 1), None);
        assert!(GONE.requests.lock().unwrap().is_empty());
        assert_eq!(registry.value(2, &plugins[2], 0).label, "Unavailable");
        assert_eq!(RequestStatus::from_code(7), RequestStatus::Rejected);
        for status in [
            RequestStatus::Applied,
            RequestStatus::Pending,
            RequestStatus::Rejected,
            RequestStatus::Unavailable,
        ] {
            assert_eq!(RequestStatus::from_code(status.code()), status);
        }
    }

    static PERSISTED: FakePlugin = FakePlugin::new([0, 0, 0, 0], abi::REQUEST_APPLIED);
    static PERSISTED_PENDING: FakePlugin = FakePlugin::new([0, 0, 0, 0], abi::REQUEST_PENDING);
    static PERSISTED_REJECTED: FakePlugin = FakePlugin::new([0, 0, 0, 0], abi::REQUEST_REJECTED);

    fn persisting(
        registry: &Registry,
        id: &str,
        path: Option<&str>,
        fake: &'static FakePlugin,
    ) -> i32 {
        let holder = descriptor_with(
            id,
            "Replay",
            &[
                ("record", "Record", &["Off", "On"]),
                ("quality", "Quality", &["Low", "High", "Max"]),
                ("status", "Status", &["Ready"]),
                ("volatile", "Volatile", &["A", "B"]),
            ],
            path,
            &[
                Persist {
                    key: Some("record_enabled"),
                    values: Some(&["off", "on"]),
                    ..Persist::default()
                },
                Persist::default(),
                Persist::default(),
                Persist {
                    no_persist: true,
                    ..Persist::default()
                },
            ],
            fake as *const FakePlugin as *mut c_void,
            Some(fake_current),
            Some(fake_request),
        );
        unsafe { registry.register_raw(&holder.desc) }
    }

    #[test]
    fn persisting_plugins_queue_coalesced_ini_writes_after_accepted_requests() {
        let registry = Registry::new();
        assert_eq!(
            persisting(
                &registry,
                "replay",
                Some("sd:/ultimate/replay/config.ini"),
                &PERSISTED
            ),
            0
        );
        assert_eq!(persisting(&registry, "quiet", None, &PERSISTED), 1);
        assert_eq!(
            persisting(
                &registry,
                "later",
                Some("sd:/ultimate/later/config.ini"),
                &PERSISTED_PENDING
            ),
            2
        );
        assert_eq!(
            persisting(
                &registry,
                "refusing",
                Some("sd:/ultimate/refusing/config.ini"),
                &PERSISTED_REJECTED
            ),
            3
        );
        let plugins = registry.snapshot();
        assert_eq!(
            plugins[0].config_path.as_deref(),
            Some("sd:/ultimate/replay/config.ini")
        );
        assert_eq!(plugins[1].config_path, None);
        let record = &plugins[0].options[0];
        assert_eq!(record.config_key, "record_enabled");
        assert_eq!(record.persisted_value(1), "on");
        assert_eq!(plugins[0].options[1].config_key, "quality");
        assert_eq!(plugins[0].options[1].persisted_value(2), "2");
        assert!(!plugins[0].options[3].persist);
        assert!(registry.take_writes().is_empty());

        registry.step(0, 0, 1);
        registry.step(0, 1, 1);
        registry.step(0, 1, 1);
        registry.step(0, 2, 1);
        registry.step(0, 3, 1);
        registry.step(1, 0, 1);
        registry.step(2, 0, 1);
        registry.step(3, 0, 1);
        let writes = registry.take_writes();
        assert_eq!(
            writes,
            vec![
                PersistWrite {
                    plugin: 0,
                    option: 0,
                    path: "sd:/ultimate/replay/config.ini".to_owned(),
                    key: "record_enabled".to_owned(),
                    value: "on".to_owned(),
                },
                PersistWrite {
                    plugin: 0,
                    option: 1,
                    path: "sd:/ultimate/replay/config.ini".to_owned(),
                    key: "quality".to_owned(),
                    value: "2".to_owned(),
                },
                PersistWrite {
                    plugin: 2,
                    option: 0,
                    path: "sd:/ultimate/later/config.ini".to_owned(),
                    key: "record_enabled".to_owned(),
                    value: "on".to_owned(),
                },
            ]
        );
        assert!(registry.take_writes().is_empty());
        for fake in [&PERSISTED, &PERSISTED_PENDING, &PERSISTED_REJECTED] {
            for slot in &fake.values {
                slot.store(0, Ordering::Relaxed);
            }
        }
    }

    #[test]
    fn persistence_descriptor_fields_are_validated() {
        let registry = Registry::new();
        let user = &PERSISTED as *const FakePlugin as *mut c_void;
        for (path, persist) in [
            (Some("/no/prefix.ini"), Persist::default()),
            (Some("sd:/"), Persist::default()),
            (
                Some("sd:/ok.ini"),
                Persist {
                    key: Some("Bad Key"),
                    ..Persist::default()
                },
            ),
            (
                Some("sd:/ok.ini"),
                Persist {
                    values: Some(&["o n", "off"]),
                    ..Persist::default()
                },
            ),
        ] {
            let holder = descriptor_with(
                "p",
                "P",
                &[("o", "O", &["Off", "On"])],
                path,
                &[persist],
                user,
                Some(fake_current),
                Some(fake_request),
            );
            assert_eq!(
                unsafe { registry.register_raw(&holder.desc) },
                abi::REGISTER_ERR_INVALID,
                "{path:?}"
            );
        }
        // Two persisted options sharing one key would overwrite each other.
        let holder = descriptor_with(
            "p",
            "P",
            &[("a", "A", &["x", "y"]), ("b", "B", &["x", "y"])],
            Some("sd:/ok.ini"),
            &[
                Persist {
                    key: Some("same"),
                    ..Persist::default()
                },
                Persist {
                    key: Some("same"),
                    ..Persist::default()
                },
            ],
            user,
            Some(fake_current),
            Some(fake_request),
        );
        assert_eq!(
            unsafe { registry.register_raw(&holder.desc) },
            abi::REGISTER_ERR_INVALID
        );
        let mut holder = descriptor(
            "p",
            "P",
            &[("o", "O", &["Off", "On"])],
            user,
            Some(fake_current),
            Some(fake_request),
        );
        // Bit 2 is READ_ONLY in version 2; bit 3 is still unknown.
        holder._options[0].flags = 1 << 2;
        assert_eq!(
            unsafe { registry.register_raw(&holder.desc) },
            abi::REGISTER_ERR_INVALID
        );
        assert_eq!(registry.count(), 0);
    }

    static TONED: FakePlugin = FakePlugin::new([1, 2, 0, 0], abi::REQUEST_APPLIED);

    #[test]
    fn version_two_tones_and_read_only_rows_render_and_lock_as_declared() {
        let registry = Registry::new();
        let user = &TONED as *const FakePlugin as *mut c_void;
        let holder = descriptor_with(
            "link",
            "Link",
            &[
                ("enabled", "Enabled", &["Off", "On"]),
                ("health", "Health", &["Ok", "Warn", "Fail"]),
                ("plain", "Plain", &["A", "B"]),
            ],
            None,
            &[
                Persist {
                    tones: Some(&[abi::TONE_NEUTRAL, abi::TONE_OK]),
                    ..Persist::default()
                },
                Persist {
                    tones: Some(&[abi::TONE_OK, abi::TONE_WARN, abi::TONE_ERROR]),
                    read_only: true,
                    ..Persist::default()
                },
                Persist::default(),
            ],
            user,
            Some(fake_current),
            Some(fake_request),
        );
        assert_eq!(holder.desc.abi_version, 2);
        assert_eq!(unsafe { registry.register_raw(&holder.desc) }, 0);
        let plugin = registry.snapshot()[0].clone();
        assert_eq!(plugin.options[0].tone_of(1), abi::TONE_OK);
        assert_eq!(plugin.options[0].tone_of(0), abi::TONE_NEUTRAL);
        assert_eq!(plugin.options[2].tone_of(1), abi::TONE_NEUTRAL);
        assert!(plugin.options[1].read_only());
        assert!(plugin.options[1].locked);
        assert!(!plugin.options[1].persist);
        assert!(!plugin.options[0].read_only());
        let enabled = registry.value(0, &plugin, 0);
        assert_eq!((enabled.label.as_str(), enabled.tone), ("On", abi::TONE_OK));
        let health = registry.value(0, &plugin, 1);
        assert_eq!(
            (health.label.as_str(), health.tone),
            ("Fail", abi::TONE_ERROR)
        );
        // Read-only: no request reaches the plugin.
        assert_eq!(registry.step(0, 1, 1), None);
        assert!(TONED.requests.lock().unwrap().is_empty());
        // Plain rows still edit.
        assert_eq!(registry.step(0, 2, 1).unwrap().requested, 1);
        TONED.values[2].store(0, Ordering::Relaxed);

        let fields = vec![
            ("Link.Enabled".to_owned(), enabled, true),
            ("Link.Health".to_owned(), health, false),
        ];
        let segments = compact_segments(&fields);
        assert_eq!(
            segments,
            vec![
                (">Link.Enabled:".to_owned(), abi::TONE_NEUTRAL),
                ("On".to_owned(), abi::TONE_OK),
                ("   ".to_owned(), abi::TONE_NEUTRAL),
                ("Link.Health:".to_owned(), abi::TONE_NEUTRAL),
                ("Fail".to_owned(), abi::TONE_ERROR),
            ]
        );
        assert_eq!(compact_line(&fields), ">Link.Enabled:On   Link.Health:Fail");
    }

    #[test]
    fn version_one_descriptors_still_register_without_tones() {
        let registry = Registry::new();
        let user = &TONED as *const FakePlugin as *mut c_void;
        let mut holder = descriptor_with(
            "old",
            "Old",
            &[
                ("enabled", "Enabled", &["Off", "On"]),
                ("mode", "Mode", &["A", "B"]),
            ],
            Some("sd:/ultimate/old/config.ini"),
            &[
                Persist {
                    key: Some("enabled"),
                    values: Some(&["off", "on"]),
                    ..Persist::default()
                },
                Persist {
                    no_persist: true,
                    ..Persist::default()
                },
            ],
            user,
            Some(fake_current),
            Some(fake_request),
        );
        holder.as_version_1();
        assert_eq!(holder.desc.abi_version, 1);
        assert_eq!(unsafe { registry.register_raw(&holder.desc) }, 0);
        let plugin = registry.snapshot()[0].clone();
        assert_eq!(plugin.options.len(), 2);
        assert_eq!(plugin.options[0].config_key, "enabled");
        assert_eq!(plugin.options[0].persisted_value(1), "on");
        assert_eq!(plugin.options[0].tones, None);
        assert!(!plugin.options[1].persist);
        assert_eq!(registry.value(0, &plugin, 0).tone, abi::TONE_NEUTRAL);
        assert_eq!(
            plugin.config_path.as_deref(),
            Some("sd:/ultimate/old/config.ini")
        );

        // A version 1 descriptor may not use version 2 flag bits.
        let mut flagged = descriptor_with(
            "old2",
            "Old2",
            &[("health", "Health", &["Ok", "Fail"])],
            None,
            &[Persist {
                read_only: true,
                ..Persist::default()
            }],
            user,
            Some(fake_current),
            Some(fake_request),
        );
        flagged.as_version_1();
        assert_eq!(
            unsafe { registry.register_raw(&flagged.desc) },
            abi::REGISTER_ERR_INVALID
        );
        // Unknown descriptor versions are refused as ABI errors.
        flagged.desc.abi_version = 3;
        assert_eq!(
            unsafe { registry.register_raw(&flagged.desc) },
            abi::REGISTER_ERR_ABI
        );
        // Bad tone bytes are invalid.
        let bad = descriptor_with(
            "bad",
            "Bad",
            &[("health", "Health", &["Ok", "Fail"])],
            None,
            &[Persist {
                tones: Some(&[abi::TONE_OK, 9]),
                ..Persist::default()
            }],
            user,
            Some(fake_current),
            Some(fake_request),
        );
        assert_eq!(
            unsafe { registry.register_raw(&bad.desc) },
            abi::REGISTER_ERR_INVALID
        );
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn ini_assignment_replaces_in_place_or_appends_and_keeps_everything_else() {
        assert_eq!(apply_ini_assignment("", "record", "on"), "record=on\n");
        assert_eq!(
            apply_ini_assignment("version=1\nrecord=off", "record", "on"),
            "version=1\nrecord=on\n"
        );
        assert_eq!(
            apply_ini_assignment(
                "# replay\nversion=1\n\n record = off # trailing\nother=1\n",
                "record",
                "on"
            ),
            "# replay\nversion=1\n\nrecord=on\nother=1\n"
        );
        assert_eq!(
            apply_ini_assignment("version=1\n# record=off\n", "record", "on"),
            "version=1\n# record=off\nrecord=on\n"
        );
        assert_eq!(
            apply_ini_assignment("recorder=1\nrecord_x=2\n", "record", "on"),
            "recorder=1\nrecord_x=2\nrecord=on\n"
        );
        assert_eq!(
            apply_ini_assignment("record=a\nrecord=b\n", "record", "c"),
            "record=c\nrecord=b\n"
        );
    }

    #[test]
    fn row_navigation_pages_and_windows_are_bounded() {
        assert_eq!(cycle_choice(0, 2, 1), 1);
        assert_eq!(cycle_choice(1, 2, 1), 0);
        assert_eq!(cycle_choice(0, 3, -1), 2);
        assert_eq!(cycle_choice(0, 0, 1), 0);
        assert_eq!(move_plugin_row(0, 1, 0), 0);
        assert_eq!(move_plugin_row(0, -1, 4), 3);
        assert_eq!(move_plugin_row(3, 1, 4), 0);
        assert_eq!(move_plugin_row(9, 1, 4), 0);
        assert_eq!(compact_page(0, 0, COMPACT_PAGE), (0, 0, 1, 1));
        assert_eq!(compact_page(0, 3, COMPACT_PAGE), (0, 3, 1, 1));
        assert_eq!(compact_page(3, 9, COMPACT_PAGE), (0, 4, 1, 3));
        assert_eq!(compact_page(4, 9, COMPACT_PAGE), (4, 8, 2, 3));
        assert_eq!(compact_page(8, 9, COMPACT_PAGE), (8, 9, 3, 3));
        assert_eq!(compact_page(50, 9, COMPACT_PAGE), (8, 9, 3, 3));
        assert_eq!(vertical_window(0, 0, VERTICAL_WINDOW), (0, 0));
        assert_eq!(vertical_window(2, 6, VERTICAL_WINDOW), (0, 6));
        assert_eq!(vertical_window(0, 25, VERTICAL_WINDOW), (0, 10));
        assert_eq!(vertical_window(12, 25, VERTICAL_WINDOW), (7, 17));
        assert_eq!(vertical_window(24, 25, VERTICAL_WINDOW), (15, 25));
        assert_eq!(compact_label("Replay", "Record", 1), "Replay");
        assert_eq!(compact_label("Demo", "Mode", 3), "Demo.Mode");
        let line = compact_line(&[
            (
                "Replay".to_owned(),
                ValueText {
                    label: "On".to_owned(),
                    pending: true,
                    tone: abi::TONE_NEUTRAL,
                },
                true,
            ),
            (
                "Demo.Mode".to_owned(),
                ValueText {
                    label: "B".to_owned(),
                    pending: false,
                    tone: abi::TONE_NEUTRAL,
                },
                false,
            ),
        ]);
        assert_eq!(line, ">Replay:On~   Demo.Mode:B");
        assert!(!line.contains('\n'));
    }
}
