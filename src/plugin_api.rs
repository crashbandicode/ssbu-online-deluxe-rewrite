//! Registry behind the Plugins page. Separately built NROs register options
//! through the C ABI in `deluxe-plugin-abi`; this module validates and stores
//! them and owns the row, page and value/request rules the overlay uses.
//!
//! Strings are copied during registration. Only the opaque `user` pointer and
//! the two callbacks are retained. Callbacks are invoked outside the registry
//! lock, on the overlay draw thread, and only while the Plugins page is shown
//! or an edge was accepted. The host never assumes a request succeeded: the
//! displayed value is always the plugin's answer to `current`.
//!
//! Descriptor versions 2, 3 and 4 share one layout and are read under one
//! rule set (the version 4 rules); version 1 is refused.

use core::ffi::{c_char, c_void};
use deluxe_plugin_abi as abi;
use std::sync::{Arc, Mutex};

/// How a row behaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowKind {
    /// Left/Right request the adjacent choice.
    Value,
    /// `OPTION_FLAG_READ_ONLY` or a single choice: the value follows
    /// `current`, Left/Right do nothing and `request` is never called.
    Status,
    /// `OPTION_FLAG_HEADER`: label only, no choices, skipped by Up/Down,
    /// never persisted, callbacks never called.
    Header,
    /// `OPTION_FLAG_CELL` (descriptor revision 5): a passive status cell.
    /// `current` picks a value whose tone says ready (a check) or not (a
    /// cross). Never requested, never persisted, never selected. A run of
    /// them is drawn as one table.
    Cell,
}

/// One registered option, strings owned by the host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginOption {
    pub label: String,
    pub choices: Vec<String>,
    pub kind: RowKind,
    /// INI key for host persistence (the option id unless overridden).
    pub config_key: String,
    /// Persisted value per choice; `None` writes the decimal index.
    pub values: Option<Vec<String>>,
    /// Cleared by `OPTION_FLAG_NO_PERSIST` and `OPTION_FLAG_READ_ONLY`.
    pub persist: bool,
    /// `TONE_*` byte per choice; `None` is neutral.
    pub tones: Option<Vec<u8>>,
}

impl PluginOption {
    /// Whether Left/Right may request a change.
    pub fn editable(&self) -> bool {
        self.kind == RowKind::Value
    }

    /// Tone for `choice`; neutral when the plugin declared none.
    pub fn tone_of(&self, choice: usize) -> u8 {
        self.tones
            .as_ref()
            .and_then(|tones| tones.get(choice).copied())
            .unwrap_or(abi::TONE_NEUTRAL)
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
    /// Drawn on its own page of the window cycle (`PLUGIN_FLAG_OWN_PAGE`)
    /// instead of on the shared Plugins page.
    pub own_page: bool,
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
    /// `(plugin, option, choice)` of requests answered `Pending` and not yet
    /// seen in the plugin's live value.
    pending: Mutex<Vec<(usize, usize, usize)>>,
    last_step: Mutex<Option<StepResult>>,
    writes: Mutex<Vec<PersistWrite>>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

/// Bounded copy of a NUL-terminated string. Null is `REGISTER_ERR_NULL`; no
/// terminator within `max + 1` bytes (so a bad descriptor cannot make the
/// host scan unbounded memory) or a string `valid` refuses is
/// `REGISTER_ERR_INVALID`.
///
/// # Safety
/// `ptr` must be null, readable for `max + 1` bytes, or NUL-terminated before that.
unsafe fn copy_str(
    ptr: *const c_char,
    max: usize,
    valid: fn(&[u8]) -> bool,
) -> Result<String, i32> {
    if ptr.is_null() {
        return Err(abi::REGISTER_ERR_NULL);
    }
    let bytes = ptr.cast::<u8>();
    for len in 0..=max {
        if *bytes.add(len) == 0 {
            let text = std::slice::from_raw_parts(bytes, len);
            return if valid(text) {
                Ok(String::from_utf8_lossy(text).into_owned())
            } else {
                Err(abi::REGISTER_ERR_INVALID)
            };
        }
    }
    Err(abi::REGISTER_ERR_INVALID)
}

/// Copies and validates a raw descriptor without touching any registry.
///
/// # Safety
/// `desc` must be null or point to a `PluginDesc` whose pointer fields are
/// null or valid for the duration of the call.
pub unsafe fn parse_descriptor(desc: *const abi::PluginDesc) -> Result<Plugin, i32> {
    const ALLOWED_OPTION_FLAGS: u32 = abi::OPTION_FLAG_NO_PERSIST
        | abi::OPTION_FLAG_READ_ONLY
        | abi::OPTION_FLAG_HEADER
        | abi::OPTION_FLAG_INLINE;
    // Only a revision 5 descriptor may carry the cell bit; older revisions
    // refuse it exactly as they refused any unknown bit before it existed.
    let allowed_flags = |version: u32| {
        if version >= abi::CELL_ABI_VERSION {
            ALLOWED_OPTION_FLAGS | abi::OPTION_FLAG_CELL
        } else {
            ALLOWED_OPTION_FLAGS
        }
    };
    if desc.is_null() {
        return Err(abi::REGISTER_ERR_NULL);
    }
    let desc = &*desc;
    if desc.struct_size as usize != core::mem::size_of::<abi::PluginDesc>()
        || !abi::descriptor_version_supported(desc.abi_version)
    {
        return Err(abi::REGISTER_ERR_ABI);
    }
    if desc.flags & !abi::PLUGIN_FLAG_OWN_PAGE != 0 {
        return Err(abi::REGISTER_ERR_INVALID);
    }
    let (Some(current), Some(request)) = (desc.current, desc.request) else {
        return Err(abi::REGISTER_ERR_CALLBACKS);
    };
    let id = copy_str(desc.id, abi::MAX_ID_LEN, abi::valid_id)?;
    let label = copy_str(desc.label, abi::MAX_LABEL_LEN, abi::valid_label)?;
    let config_path = if desc.config_path.is_null() {
        None
    } else {
        Some(copy_str(
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
    let mut ids: Vec<String> = Vec::with_capacity(option_count);
    let mut options = Vec::with_capacity(option_count);
    for raw in std::slice::from_raw_parts(desc.options, option_count) {
        let header = raw.flags & abi::OPTION_FLAG_HEADER != 0;
        let cell = raw.flags & abi::OPTION_FLAG_CELL != 0;
        if raw.flags & !allowed_flags(desc.abi_version) != 0
            || (header && raw.flags & abi::OPTION_FLAG_INLINE != 0)
            || (cell && raw.flags & (abi::OPTION_FLAG_HEADER | abi::OPTION_FLAG_INLINE) != 0)
        {
            return Err(abi::REGISTER_ERR_INVALID);
        }
        let option_id = copy_str(raw.id, abi::MAX_ID_LEN, abi::valid_id)?;
        if ids.contains(&option_id) {
            return Err(abi::REGISTER_ERR_INVALID);
        }
        let label = copy_str(raw.label, abi::MAX_LABEL_LEN, abi::valid_label)?;
        let choice_count = raw.choice_count as usize;
        if header {
            // A header carries its label and nothing else.
            if choice_count != 0
                || !raw.choices.is_null()
                || !raw.choice_values.is_null()
                || !raw.choice_tones.is_null()
            {
                return Err(abi::REGISTER_ERR_INVALID);
            }
            options.push(PluginOption {
                label,
                choices: Vec::new(),
                kind: RowKind::Header,
                config_key: String::new(),
                values: None,
                persist: false,
                tones: None,
            });
            ids.push(option_id);
            continue;
        }
        if choice_count == 0 || choice_count > abi::MAX_CHOICES {
            return Err(abi::REGISTER_ERR_INVALID);
        }
        if raw.choices.is_null() {
            return Err(abi::REGISTER_ERR_NULL);
        }
        let mut choices = Vec::with_capacity(choice_count);
        for choice in std::slice::from_raw_parts(raw.choices, choice_count) {
            choices.push(copy_str(*choice, abi::MAX_LABEL_LEN, abi::valid_label)?);
        }
        let config_key = if raw.config_key.is_null() {
            option_id.clone()
        } else {
            copy_str(raw.config_key, abi::MAX_ID_LEN, abi::valid_id)?
        };
        let values = if raw.choice_values.is_null() {
            None
        } else {
            let mut values = Vec::with_capacity(choice_count);
            for value in std::slice::from_raw_parts(raw.choice_values, choice_count) {
                values.push(copy_str(*value, abi::MAX_VALUE_LEN, abi::valid_value)?);
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
        // The tone is a cell's ready/not-ready contract, so it is required.
        if cell && tones.is_none() {
            return Err(abi::REGISTER_ERR_INVALID);
        }
        let locked = cell || raw.flags & abi::OPTION_FLAG_READ_ONLY != 0;
        options.push(PluginOption {
            label,
            kind: if cell {
                RowKind::Cell
            } else if locked || choice_count == 1 {
                RowKind::Status
            } else {
                RowKind::Value
            },
            choices,
            config_key,
            values,
            // Read-only rows are never edited, so there is nothing to persist.
            persist: raw.flags & abi::OPTION_FLAG_NO_PERSIST == 0 && !locked,
            tones,
        });
        ids.push(option_id);
    }
    // One table holds at most `MAX_CELL_RUN` consecutive cells.
    let mut run = 0usize;
    for option in &options {
        run = if option.kind == RowKind::Cell {
            run + 1
        } else {
            0
        };
        if run > abi::MAX_CELL_RUN {
            return Err(abi::REGISTER_ERR_INVALID);
        }
    }
    // Two persisted options sharing one key would overwrite each other.
    let mut keys: Vec<&str> = Vec::new();
    for option in options.iter().filter(|option| option.persist) {
        if keys.contains(&option.config_key.as_str()) {
            return Err(abi::REGISTER_ERR_INVALID);
        }
        keys.push(&option.config_key);
    }
    Ok(Plugin {
        id,
        label,
        options,
        config_path,
        own_page: desc.flags & abi::PLUGIN_FLAG_OWN_PAGE != 0,
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

    /// Registration order is the display order.
    pub fn snapshot(&self) -> Vec<Arc<Plugin>> {
        self.plugins.lock().map(|p| p.clone()).unwrap_or_default()
    }

    /// Inserts a validated plugin. Returns its handle or a `REGISTER_ERR_*` code.
    pub fn register(&self, plugin: Plugin) -> i32 {
        let Ok(mut plugins) = self.plugins.lock() else {
            return abi::REGISTER_ERR_FULL;
        };
        if plugins.len() >= abi::MAX_PLUGINS {
            return abi::REGISTER_ERR_FULL;
        }
        if plugins.iter().any(|p| p.id == plugin.id) {
            return abi::REGISTER_ERR_DUPLICATE;
        }
        plugins.push(Arc::new(plugin));
        (plugins.len() - 1) as i32
    }

    /// The C entry: parse, then insert.
    ///
    /// # Safety
    /// See [`parse_descriptor`].
    pub unsafe fn register_raw(&self, desc: *const abi::PluginDesc) -> i32 {
        match parse_descriptor(desc) {
            Ok(plugin) => self.register(plugin),
            Err(code) => code,
        }
    }

    fn pending_for(&self, plugin: usize, option: usize) -> Option<usize> {
        self.pending
            .lock()
            .ok()?
            .iter()
            .find_map(|(p, o, choice)| (*p == plugin && *o == option).then_some(*choice))
    }

    fn set_pending(&self, plugin: usize, option: usize, choice: Option<usize>) {
        let Ok(mut pending) = self.pending.lock() else {
            return;
        };
        pending.retain(|(p, o, _)| !(*p == plugin && *o == option));
        if let Some(choice) = choice {
            pending.push((plugin, option, choice));
        }
    }

    /// Current display text. Pure: a pending request stays shown until
    /// [`Self::reconcile_pending`], which the overlay runs before every draw,
    /// sees the plugin's live value reach it and persists it.
    pub fn value(&self, plugin_index: usize, plugin: &Plugin, option: usize) -> ValueText {
        let Some(spec) = plugin.options.get(option) else {
            return ValueText {
                label: "Unavailable".to_owned(),
                pending: false,
                tone: abi::TONE_NEUTRAL,
            };
        };
        let current = plugin.current(option);
        ValueText {
            label: current
                .and_then(|index| spec.choices.get(index).cloned())
                .unwrap_or_else(|| "Unavailable".to_owned()),
            pending: self
                .pending_for(plugin_index, option)
                .is_some_and(|wanted| Some(wanted) != current),
            tone: current.map_or(abi::TONE_NEUTRAL, |index| spec.tone_of(index)),
        }
    }

    /// One Left (-1) or Right (+1) edge on a row. Status, header and
    /// unavailable options make no request. The plugin's answer is recorded
    /// for the runtime to log and returned to the caller.
    pub fn step(&self, plugin_index: usize, option: usize, direction: i8) -> Option<StepResult> {
        if direction == 0 {
            return None;
        }
        let plugin = self.snapshot().get(plugin_index)?.clone();
        let spec = plugin.options.get(option)?;
        if !spec.editable() {
            return None;
        }
        let current = plugin.current(option)?;
        // Step from an outstanding pending request rather than the live value,
        // so a second Left/Right can take a pending change back instead of
        // re-requesting the same choice.
        let base = self.pending_for(plugin_index, option).unwrap_or(current);
        let requested = wrap(base, direction, spec.choices.len());
        let status = plugin.request(option, requested);
        // A pending request that merely returns to the live value is a
        // cancellation: nothing stays pending and nothing is written. A
        // pending change is persisted only once the plugin applies it (see
        // `reconcile_pending`), so a plugin that defers in-match edits sees no
        // disk activity until then. Applied writes at once.
        let still_pending = status == RequestStatus::Pending && requested != current;
        self.set_pending(plugin_index, option, still_pending.then_some(requested));
        if status == RequestStatus::Applied {
            self.queue_persist(plugin_index, &plugin, option, requested);
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

    /// Queues the INI write for `choice` when the plugin persists this option.
    /// Same path and key coalesce to the newest value.
    fn queue_persist(&self, plugin_index: usize, plugin: &Plugin, option: usize, choice: usize) {
        let Some(spec) = plugin.options.get(option) else {
            return;
        };
        let Some(path) = plugin.config_path.as_ref().filter(|_| spec.persist) else {
            return;
        };
        let Ok(mut writes) = self.writes.lock() else {
            return;
        };
        writes.retain(|w| !(w.path == *path && w.key == spec.config_key));
        writes.push(PersistWrite {
            plugin: plugin_index,
            option,
            path: path.clone(),
            key: spec.config_key.clone(),
            value: spec
                .values
                .as_ref()
                .and_then(|values| values.get(choice).cloned())
                .unwrap_or_else(|| choice.to_string()),
        });
    }

    /// Checks every pending request against the plugin's live value and
    /// persists the ones that have been applied. The overlay calls this once
    /// per draw, so a change a plugin applies at match end is written even
    /// while the Plugins page is hidden. Callbacks run on the draw thread.
    pub fn reconcile_pending(&self) {
        let pending: Vec<(usize, usize, usize)> = match self.pending.lock() {
            Ok(list) if list.is_empty() => return,
            Ok(list) => list.clone(),
            Err(_) => return,
        };
        let plugins = self.snapshot();
        for (plugin_index, option, choice) in pending {
            let Some(plugin) = plugins.get(plugin_index) else {
                continue;
            };
            if plugin.current(option) == Some(choice) {
                self.set_pending(plugin_index, option, None);
                self.queue_persist(plugin_index, plugin, option, choice);
            }
        }
    }
}

/// Steps `index` by `direction` (-1, 0, +1) with wrap-around over `count`
/// entries. An empty list keeps zero.
pub fn wrap(index: usize, direction: i8, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    (index.min(count - 1) as i64 + i64::from(direction)).rem_euclid(count as i64) as usize
}

/// One row of the Plugins page or an own page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageRow {
    /// A plugin's label heading its options on the shared page. Never selected.
    Group(usize),
    Option {
        plugin: usize,
        option: usize,
    },
}

/// Indices of the plugins that asked for their own page, in registration
/// order: the window cycle visits them after the shared Plugins page.
pub fn page_plugins(plugins: &[Arc<Plugin>]) -> Vec<usize> {
    plugins
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.own_page)
        .map(|(index, _)| index)
        .collect()
}

/// Whether any plugin shows on the shared Plugins page.
pub fn has_shared_rows(plugins: &[Arc<Plugin>]) -> bool {
    plugins.iter().any(|entry| !entry.own_page)
}

/// Rows of a view. The shared page (`None`) has a `Group` row per plugin
/// that did not ask for its own page, followed by its options; own page
/// `Some(i)` (indexing [`page_plugins`]) has that plugin's options alone.
pub fn rows_for(plugins: &[Arc<Plugin>], page: Option<usize>) -> Vec<PageRow> {
    let options = |plugin: usize| {
        (0..plugins[plugin].options.len()).map(move |option| PageRow::Option { plugin, option })
    };
    match page {
        None => plugins
            .iter()
            .enumerate()
            .filter(|(_, entry)| !entry.own_page)
            .flat_map(|(plugin, _)| std::iter::once(PageRow::Group(plugin)).chain(options(plugin)))
            .collect(),
        Some(page) => page_plugins(plugins)
            .get(page)
            .map(|plugin| options(*plugin).collect())
            .unwrap_or_default(),
    }
}

/// Whether Up/Down may land on `row`: only an editable option. Group,
/// header, status and cell rows never hold the cursor, so a page with no
/// editable option highlights nothing.
pub fn selectable(plugins: &[Arc<Plugin>], row: PageRow) -> bool {
    match row {
        PageRow::Group(_) => false,
        PageRow::Option { plugin, option } => plugins
            .get(plugin)
            .and_then(|entry| entry.options.get(option))
            .is_none_or(PluginOption::editable),
    }
}

/// The half-open range of `rows` forming the cell table that contains
/// `index`: its consecutive cell options of one plugin. `None` when the row
/// is not a cell.
pub fn cell_group(
    plugins: &[Arc<Plugin>],
    rows: &[PageRow],
    index: usize,
) -> Option<(usize, usize)> {
    let is_cell = |row: &PageRow| match *row {
        PageRow::Option { plugin, option } => plugins
            .get(plugin)
            .and_then(|entry| entry.options.get(option))
            .is_some_and(|spec| spec.kind == RowKind::Cell),
        PageRow::Group(_) => false,
    };
    let owner = |row: &PageRow| match *row {
        PageRow::Option { plugin, .. } => Some(plugin),
        PageRow::Group(_) => None,
    };
    let row = rows.get(index)?;
    if !is_cell(row) {
        return None;
    }
    let plugin = owner(row);
    let mut start = index;
    while start > 0 && is_cell(&rows[start - 1]) && owner(&rows[start - 1]) == plugin {
        start -= 1;
    }
    let mut end = index + 1;
    while end < rows.len() && is_cell(&rows[end]) && owner(&rows[end]) == plugin {
        end += 1;
    }
    Some((start, end))
}

/// `selected` clamped into `rows` and moved forward onto a selectable row,
/// then stepped by `direction` (-1, 0, +1) with wrap, skipping group and
/// header rows. A list without a selectable row behaves like [`wrap`].
pub fn move_selection(
    plugins: &[Arc<Plugin>],
    rows: &[PageRow],
    selected: usize,
    direction: i8,
) -> usize {
    if rows.is_empty() {
        return 0;
    }
    let settle = |mut row: usize, direction: i8| {
        for _ in 0..rows.len() {
            if selectable(plugins, rows[row]) {
                break;
            }
            row = wrap(row, direction, rows.len());
        }
        row
    };
    let row = settle(selected.min(rows.len() - 1), 1);
    if direction == 0 {
        row
    } else {
        settle(wrap(row, direction, rows.len()), direction)
    }
}

/// Process-wide registry used by the runtime export and the overlay.
pub static REGISTRY: Registry = Registry::new();

/// Test helpers: a descriptor with owned C strings and a scripted fake plugin.
#[cfg(test)]
pub(crate) mod testing {
    use super::*;
    use std::ffi::CString;

    pub struct DescHolder {
        _strings: Vec<CString>,
        _choice_tables: Vec<Vec<*const c_char>>,
        _tone_tables: Vec<Vec<u8>>,
        pub _options: Vec<abi::OptionDesc>,
        pub desc: abi::PluginDesc,
    }

    /// Persistence and presentation settings for one option in [`descriptor_with`].
    #[derive(Clone, Copy, Default)]
    pub struct Persist {
        pub key: Option<&'static str>,
        pub values: Option<&'static [&'static str]>,
        pub no_persist: bool,
        pub tones: Option<&'static [u8]>,
        pub read_only: bool,
        /// Header row: the choice list is ignored and sent as 0 choices with
        /// null tables.
        pub header: bool,
        /// `OPTION_FLAG_INLINE`: still a valid wire bit, drawn as a value row.
        pub inline: bool,
        /// `OPTION_FLAG_CELL`: a passive status cell (revision 5 only).
        pub cell: bool,
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
        let mut choice_tables: Vec<Vec<*const c_char>> = Vec::new();
        let mut tone_tables: Vec<Vec<u8>> = Vec::new();
        let mut raw_options = Vec::new();
        for (index, (oid, olabel, choices)) in options.iter().enumerate() {
            let oid_ptr = keep(oid);
            let olabel_ptr = keep(olabel);
            let settings = persist.get(index).copied().unwrap_or_default();
            let key_ptr = settings.key.map_or(std::ptr::null(), &mut keep);
            let mut flags = 0;
            for (set, bit) in [
                (settings.no_persist, abi::OPTION_FLAG_NO_PERSIST),
                (settings.read_only, abi::OPTION_FLAG_READ_ONLY),
                (settings.header, abi::OPTION_FLAG_HEADER),
                (settings.inline, abi::OPTION_FLAG_INLINE),
                (settings.cell, abi::OPTION_FLAG_CELL),
            ] {
                if set {
                    flags |= bit;
                }
            }
            if settings.header {
                raw_options.push(abi::OptionDesc {
                    id: oid_ptr,
                    label: olabel_ptr,
                    choices: std::ptr::null(),
                    choice_count: 0,
                    flags,
                    config_key: key_ptr,
                    choice_values: std::ptr::null(),
                    choice_tones: std::ptr::null(),
                });
                continue;
            }
            // Tables are boxed into `choice_tables` so their addresses stay
            // put while the vector grows.
            let choices_ptr = {
                choice_tables.push(choices.iter().map(|c| keep(c)).collect());
                choice_tables.last().unwrap().as_ptr()
            };
            let choice_count = choices.len() as u32;
            let values_ptr = match settings.values {
                Some(values) => {
                    choice_tables.push(values.iter().map(|v| keep(v)).collect());
                    choice_tables.last().unwrap().as_ptr()
                }
                None => std::ptr::null(),
            };
            let tones_ptr = match settings.tones {
                Some(tones) => {
                    tone_tables.push(tones.to_vec());
                    tone_tables.last().unwrap().as_ptr()
                }
                None => std::ptr::null(),
            };
            raw_options.push(abi::OptionDesc {
                id: oid_ptr,
                label: olabel_ptr,
                choices: choices_ptr,
                choice_count,
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
            desc,
        }
    }

    /// Fake plugin state: one atomic per option (a fifth slot starts at 0 for
    /// five-row plugins) plus a scripted request answer.
    pub struct FakePlugin {
        pub values: [std::sync::atomic::AtomicU32; 5],
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
                    AtomicU32::new(0),
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

    fn option(plugin: usize, option: usize) -> PageRow {
        PageRow::Option { plugin, option }
    }

    #[test]
    fn registration_copies_strings_and_assigns_ordered_handles() {
        let registry = Registry::new();
        assert!(registry.snapshot().is_empty());
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
        assert_eq!(plugins[0].options[2].kind, RowKind::Status);
        assert!(!plugins[0].options[2].editable());
        assert!(plugins[0].options[0].editable());
        assert_eq!(plugins[1].label, "Replay");
        assert!(has_shared_rows(&plugins));
        assert!(page_plugins(&plugins).is_empty());
        // The shared page heads each plugin's options with a group row.
        assert_eq!(
            rows_for(&plugins, None),
            vec![
                PageRow::Group(0),
                option(0, 0),
                option(0, 1),
                option(0, 2),
                PageRow::Group(1),
                option(1, 0),
            ]
        );
        assert!(rows_for(&plugins, Some(0)).is_empty());
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
        // Versions 2 to 5 are read under one rule set (5 alone adds the cell bit);
        // 0, 1 and 6 are not ours.
        for version in [0, 1, abi::CELL_ABI_VERSION + 1] {
            holder.desc.abi_version = version;
            assert_eq!(
                unsafe { registry.register_raw(&holder.desc) },
                abi::REGISTER_ERR_ABI,
                "version {version}"
            );
        }
        holder.desc.abi_version = abi::ABI_VERSION;
        holder.desc.struct_size += 8;
        assert_eq!(
            unsafe { registry.register_raw(&holder.desc) },
            abi::REGISTER_ERR_ABI
        );
        holder.desc.struct_size = core::mem::size_of::<abi::PluginDesc>() as u32;
        // Unknown plugin flag bits are invalid; the own-page bit is accepted
        // from every version now.
        holder.desc.flags = 2;
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
        let saved_options = holder.desc.options;
        holder.desc.options = std::ptr::null();
        assert_eq!(
            unsafe { registry.register_raw(&holder.desc) },
            abi::REGISTER_ERR_NULL
        );
        holder.desc.options = saved_options;
        // The own-page bit is accepted from every version now.
        for version in [2, 3, abi::ABI_VERSION] {
            let fresh = Registry::new();
            holder.desc.abi_version = version;
            holder.desc.flags = abi::PLUGIN_FLAG_OWN_PAGE;
            assert_eq!(
                unsafe { fresh.register_raw(&holder.desc) },
                0,
                "version {version}"
            );
            assert!(fresh.snapshot()[0].own_page);
        }
        holder.desc.flags = 0;
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
        // A label that never terminates within its limit is refused, not scanned on.
        let unterminated = [b'x'; abi::MAX_LABEL_LEN + 8];
        let mut holder = descriptor(
            "ok",
            "Demo",
            DEMO,
            user,
            Some(fake_current),
            Some(fake_request),
        );
        holder.desc.label = unterminated.as_ptr().cast();
        assert_eq!(
            unsafe { registry.register_raw(&holder.desc) },
            abi::REGISTER_ERR_INVALID
        );
        assert_eq!(registry.snapshot().len(), 1);
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
        // Option limits: 64 rows fit, 65 do not, in every accepted version.
        let user = &TOGGLE as *const FakePlugin as *mut c_void;
        let ids: Vec<String> = (0..65).map(|index| format!("o{index}")).collect();
        let choices: &[&str] = &["A", "B"];
        let rows: Vec<(&str, &str, &[&str])> =
            ids.iter().map(|id| (id.as_str(), "x", choices)).collect();
        let wide = Registry::new();
        let mut holder = descriptor(
            "wide",
            "Wide",
            &rows[..64],
            user,
            Some(fake_current),
            Some(fake_request),
        );
        holder.desc.abi_version = 2;
        assert_eq!(unsafe { wide.register_raw(&holder.desc) }, 0);
        let holder = descriptor(
            "wider",
            "Wider",
            &rows,
            user,
            Some(fake_current),
            Some(fake_request),
        );
        assert_eq!(
            unsafe { wide.register_raw(&holder.desc) },
            abi::REGISTER_ERR_INVALID
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
        assert_eq!(registry.value(0, &plugin, 9).label, "Unavailable");
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
        // A second press steps from the pending choice, so Left takes it back
        // to the live value and the row stops showing pending.
        let back = registry.step(0, 0, -1).unwrap();
        assert_eq!(back.requested, 1);
        assert_eq!(back.status, RequestStatus::Pending);
        assert!(!registry.value(0, &plugins[0], 0).pending);
        assert_eq!(registry.step(0, 0, 1).unwrap().requested, 2);
        assert!(registry.value(0, &plugins[0], 0).pending);
        // The plugin applies on its own: `value` is pure, so the row reads
        // not-pending at once and the draw-time reconcile clears the entry.
        PENDING.values[0].store(2, Ordering::Relaxed);
        assert_eq!(
            registry.value(0, &plugins[0], 0),
            ValueText {
                label: "Auto".to_owned(),
                pending: false,
                tone: abi::TONE_NEUTRAL,
            }
        );
        registry.reconcile_pending();
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
        assert_eq!(
            record.values.as_deref(),
            Some(&["off".to_owned(), "on".to_owned()][..])
        );
        assert_eq!(plugins[0].options[1].config_key, "quality");
        assert_eq!(plugins[0].options[1].values, None);
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
            ]
        );
        // The pending plugin (2) wrote nothing yet: a PENDING answer defers
        // persistence until the plugin's live value shows the change.
        assert!(registry.take_writes().is_empty());
        registry.reconcile_pending();
        assert!(registry.take_writes().is_empty());
        PERSISTED_PENDING.values[0].store(1, Ordering::Relaxed);
        registry.reconcile_pending();
        assert_eq!(
            registry.take_writes(),
            vec![PersistWrite {
                plugin: 2,
                option: 0,
                path: "sd:/ultimate/later/config.ini".to_owned(),
                key: "record_enabled".to_owned(),
                value: "on".to_owned(),
            }]
        );
        // Satisfied once: no second write and nothing left pending.
        registry.reconcile_pending();
        assert!(registry.take_writes().is_empty());
        assert!(!registry.value(2, &plugins[2], 0).pending);
        for fake in [&PERSISTED, &PERSISTED_PENDING, &PERSISTED_REJECTED] {
            for slot in &fake.values {
                slot.store(0, Ordering::Relaxed);
            }
        }
    }

    static DEFER: FakePlugin = FakePlugin::new([0, 0, 0, 0], abi::REQUEST_PENDING);

    fn deferring_registry() -> Registry {
        let registry = Registry::new();
        DEFER.answer.store(abi::REQUEST_PENDING, Ordering::Relaxed);
        for slot in &DEFER.values {
            slot.store(0, Ordering::Relaxed);
        }
        DEFER.requests.lock().unwrap().clear();
        let holder = descriptor_with(
            "capture",
            "Capture",
            &[
                ("record", "Record", &["Off", "On"]),
                ("quality", "Quality", &["Low", "Mid", "High"]),
            ],
            Some("sd:/ultimate/capture/config.ini"),
            &[
                Persist {
                    values: Some(&["off", "on"]),
                    ..Persist::default()
                },
                Persist::default(),
            ],
            &DEFER as *const FakePlugin as *mut c_void,
            Some(fake_current),
            Some(fake_request),
        );
        assert_eq!(unsafe { registry.register_raw(&holder.desc) }, 0);
        registry
    }

    static DEFER_GATE: Mutex<()> = Mutex::new(());

    #[test]
    fn pending_then_applied_by_the_plugin_writes_the_applied_value_once() {
        let _gate = DEFER_GATE.lock().unwrap();
        let registry = deferring_registry();
        let plugin = registry.snapshot()[0].clone();
        let step = registry.step(0, 0, 1).unwrap();
        assert_eq!((step.requested, step.status), (1, RequestStatus::Pending));
        assert!(registry.value(0, &plugin, 0).pending);
        // Mid-match: the plugin has not applied, so the host must not write.
        registry.reconcile_pending();
        assert!(registry.take_writes().is_empty());
        // Match ends: the plugin flips its live value itself.
        DEFER.values[0].store(1, Ordering::Relaxed);
        registry.reconcile_pending();
        let writes = registry.take_writes();
        assert_eq!(writes.len(), 1);
        assert_eq!(
            (writes[0].key.as_str(), writes[0].value.as_str()),
            ("record", "on")
        );
        assert!(!registry.value(0, &plugin, 0).pending);
        registry.reconcile_pending();
        assert!(registry.take_writes().is_empty());
        assert_eq!(DEFER.requests.lock().unwrap().len(), 1);
    }

    #[test]
    fn pending_then_cancelled_by_stepping_back_writes_nothing() {
        let _gate = DEFER_GATE.lock().unwrap();
        let registry = deferring_registry();
        let plugin = registry.snapshot()[0].clone();
        assert_eq!(registry.step(0, 0, 1).unwrap().requested, 1);
        assert!(registry.value(0, &plugin, 0).pending);
        let back = registry.step(0, 0, -1).unwrap();
        assert_eq!((back.requested, back.status), (0, RequestStatus::Pending));
        assert!(!registry.value(0, &plugin, 0).pending);
        registry.reconcile_pending();
        assert!(registry.take_writes().is_empty());
        // The plugin was asked twice (1, then 0); the host wrote nothing.
        assert_eq!(DEFER.requests.lock().unwrap().as_slice(), &[(0, 1), (0, 0)]);
    }

    #[test]
    fn pending_then_applied_on_a_second_request_writes_once() {
        let _gate = DEFER_GATE.lock().unwrap();
        let registry = deferring_registry();
        let plugin = registry.snapshot()[0].clone();
        // Three-choice row: first press is deferred.
        assert_eq!(
            registry.step(0, 1, 1).unwrap().status,
            RequestStatus::Pending
        );
        assert!(registry.take_writes().is_empty());
        // The plugin now accepts immediately; the second press steps from the
        // pending value (1) to 2 and applies.
        DEFER.answer.store(abi::REQUEST_APPLIED, Ordering::Relaxed);
        let step = registry.step(0, 1, 1).unwrap();
        assert_eq!((step.requested, step.status), (2, RequestStatus::Applied));
        let writes = registry.take_writes();
        assert_eq!(writes.len(), 1);
        assert_eq!(
            (writes[0].key.as_str(), writes[0].value.as_str()),
            ("quality", "2")
        );
        assert!(!registry.value(0, &plugin, 1).pending);
        registry.reconcile_pending();
        assert!(registry.take_writes().is_empty());
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
            (
                None,
                Persist {
                    tones: Some(&[abi::TONE_OK, 9]),
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
        // A read-only row may reuse a persisted key: it is never written.
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
                    read_only: true,
                    ..Persist::default()
                },
            ],
            user,
            Some(fake_current),
            Some(fake_request),
        );
        assert_eq!(unsafe { registry.register_raw(&holder.desc) }, 0);
        // Bit 4 is not an option flag in any version.
        let mut holder = descriptor(
            "q",
            "Q",
            &[("o", "O", &["Off", "On"])],
            user,
            Some(fake_current),
            Some(fake_request),
        );
        holder._options[0].flags = 1 << 4;
        assert_eq!(
            unsafe { registry.register_raw(&holder.desc) },
            abi::REGISTER_ERR_INVALID
        );
        assert_eq!(registry.snapshot().len(), 1);
    }

    static TONED: FakePlugin = FakePlugin::new([1, 2, 0, 0], abi::REQUEST_APPLIED);

    #[test]
    fn tones_and_read_only_rows_render_and_lock_as_declared() {
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
        assert_eq!(holder.desc.abi_version, abi::ABI_VERSION);
        assert_eq!(unsafe { registry.register_raw(&holder.desc) }, 0);
        let plugin = registry.snapshot()[0].clone();
        assert!(!plugin.own_page);
        assert_eq!(plugin.options[0].tone_of(1), abi::TONE_OK);
        assert_eq!(plugin.options[0].tone_of(0), abi::TONE_NEUTRAL);
        assert_eq!(plugin.options[2].tone_of(1), abi::TONE_NEUTRAL);
        assert_eq!(plugin.options[1].kind, RowKind::Status);
        assert!(!plugin.options[1].persist);
        assert!(plugin.options[0].editable());
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
    }

    /// The replay plugin's five rows exactly as its version 2 SDK sends them:
    /// two persisted toggles, a six-choice toned status row, a three-choice
    /// tone-only row and an editable `no_persist` row whose `request` answers
    /// `Pending`.
    static REPLAY: FakePlugin = FakePlugin::new([0, 1, 2, 1], abi::REQUEST_PENDING);

    fn replay_descriptor() -> DescHolder {
        static STATUS_TONES: [u8; 6] = [
            abi::TONE_NEUTRAL,
            abi::TONE_OK,
            abi::TONE_WARN,
            abi::TONE_OK,
            abi::TONE_ERROR,
            abi::TONE_ERROR,
        ];
        static OBS_TONES: [u8; 3] = [abi::TONE_OK, abi::TONE_WARN, abi::TONE_ERROR];
        static STOP_TONES: [u8; 2] = [abi::TONE_NEUTRAL, abi::TONE_WARN];
        let mut holder = descriptor_with(
            "smash_replay_to_mp4",
            "OBS Recording",
            &[
                ("enabled", "Enabled", &["Off", "On"]),
                ("record_playback", "Playback", &["Off", "On"]),
                (
                    "status",
                    "Status",
                    &["Idle", "Rec", "Pending", "Kept", "Dropped", "No OBS"],
                ),
                ("obs", "OBS", &["OBS", "OBS", "OBS"]),
                ("stop_now", "Stop rec", &["Ready", "Stop"]),
            ],
            Some("sd:/ultimate/smash_replay_to_mp4/config.ini"),
            &[
                Persist {
                    key: Some("enabled"),
                    values: Some(&["off", "on"]),
                    ..Persist::default()
                },
                Persist {
                    key: Some("record_playback"),
                    values: Some(&["off", "on"]),
                    ..Persist::default()
                },
                Persist {
                    no_persist: true,
                    tones: Some(&STATUS_TONES),
                    read_only: true,
                    ..Persist::default()
                },
                Persist {
                    no_persist: true,
                    tones: Some(&OBS_TONES),
                    read_only: true,
                    ..Persist::default()
                },
                Persist {
                    no_persist: true,
                    tones: Some(&STOP_TONES),
                    ..Persist::default()
                },
            ],
            &REPLAY as *const FakePlugin as *mut c_void,
            Some(fake_current),
            Some(fake_request),
        );
        holder.desc.abi_version = 2;
        holder
    }

    #[test]
    fn replay_plugin_version_two_descriptor_registers_with_rows_tones_and_pending() {
        let registry = Registry::new();
        let holder = replay_descriptor();
        assert_eq!(holder.desc.abi_version, 2);
        assert_eq!(unsafe { registry.register_raw(&holder.desc) }, 0);
        let plugins = registry.snapshot();
        let plugin = plugins[0].clone();
        assert_eq!(plugin.label, "OBS Recording");
        assert!(!plugin.own_page);
        assert_eq!(
            plugin.options.iter().map(|o| o.kind).collect::<Vec<_>>(),
            [
                RowKind::Value,
                RowKind::Value,
                RowKind::Status,
                RowKind::Status,
                RowKind::Value
            ]
        );
        assert_eq!(
            plugin.options.iter().map(|o| o.persist).collect::<Vec<_>>(),
            [true, true, false, false, false]
        );
        assert_eq!(plugin.options[1].config_key, "record_playback");
        // The shared page: one group row, five options; the cursor skips the two status rows.
        let rows = rows_for(&plugins, None);
        assert_eq!(rows.len(), 6);
        assert_eq!(move_selection(&plugins, &rows, 0, 0), 1);
        assert_eq!(move_selection(&plugins, &rows, 5, 1), 1);
        assert_eq!(move_selection(&plugins, &rows, 1, -1), 5);
        // Tones follow the live choice; read-only rows never request.
        let status = registry.value(0, &plugin, 2);
        assert_eq!(
            (status.label.as_str(), status.tone),
            ("Pending", abi::TONE_WARN)
        );
        assert_eq!(registry.value(0, &plugin, 3).tone, abi::TONE_WARN);
        assert_eq!(registry.step(0, 2, 1), None);
        assert_eq!(registry.step(0, 3, 1), None);
        assert!(REPLAY.requests.lock().unwrap().is_empty());
        // "Stop rec" answers Pending: shown as pending, nothing persisted.
        let stop = registry.step(0, 4, 1).unwrap();
        assert_eq!((stop.requested, stop.status), (1, RequestStatus::Pending));
        let shown = registry.value(0, &plugin, 4);
        assert_eq!((shown.label.as_str(), shown.pending), ("Ready", true));
        registry.reconcile_pending();
        assert!(registry.take_writes().is_empty());
        REPLAY.values[4].store(1, Ordering::Relaxed);
        registry.reconcile_pending();
        assert!(registry.take_writes().is_empty(), "no_persist row");
        let shown = registry.value(0, &plugin, 4);
        assert_eq!(
            (shown.label.as_str(), shown.pending, shown.tone),
            ("Stop", false, abi::TONE_WARN)
        );
        // A persisted toggle answered Pending is written once the plugin applies it.
        assert_eq!(
            registry.step(0, 0, 1).unwrap().status,
            RequestStatus::Pending
        );
        registry.reconcile_pending();
        assert!(registry.take_writes().is_empty());
        REPLAY.values[0].store(1, Ordering::Relaxed);
        registry.reconcile_pending();
        assert_eq!(
            registry.take_writes(),
            vec![PersistWrite {
                plugin: 0,
                option: 0,
                path: "sd:/ultimate/smash_replay_to_mp4/config.ini".to_owned(),
                key: "enabled".to_owned(),
                value: "on".to_owned(),
            }]
        );
        assert_eq!(
            REPLAY.requests.lock().unwrap().as_slice(),
            &[(4, 1), (0, 1)]
        );
        REPLAY.values[0].store(0, Ordering::Relaxed);
        REPLAY.values[4].store(0, Ordering::Relaxed);
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
    fn wrap_and_selection_moves_are_bounded() {
        assert_eq!(wrap(0, 1, 2), 1);
        assert_eq!(wrap(1, 2, 2), 1);
        assert_eq!(wrap(1, 1, 2), 0);
        assert_eq!(wrap(0, -1, 3), 2);
        assert_eq!(wrap(0, 1, 0), 0);
        assert_eq!(wrap(0, -1, 4), 3);
        assert_eq!(wrap(3, 1, 4), 0);
        assert_eq!(wrap(9, 1, 4), 0);
        assert_eq!(wrap(2, 0, 4), 2);
        assert_eq!(move_selection(&[], &[], 3, 1), 0);
        // Headers and group rows are skipped in both directions and never
        // hold a clamped selection. The FX descriptor on the shared page.
        let registry = Registry::new();
        let mut holder = grouped_descriptor(abi::ABI_VERSION);
        holder.desc.flags = 0;
        assert_eq!(unsafe { registry.register_raw(&holder.desc) }, 0);
        let plugins = registry.snapshot();
        let rows = rows_for(&plugins, None);
        assert_eq!(
            rows.iter()
                .map(|row| selectable(&plugins, *row))
                .collect::<Vec<_>>(),
            [false, true, false, true, false]
        );
        assert_eq!(move_selection(&plugins, &rows, 0, 0), 1);
        assert_eq!(move_selection(&plugins, &rows, 2, 0), 3);
        assert_eq!(move_selection(&plugins, &rows, 9, 0), 1);
        assert_eq!(move_selection(&plugins, &rows, 1, 1), 3);
        assert_eq!(move_selection(&plugins, &rows, 3, -1), 1);
        assert_eq!(move_selection(&plugins, &rows, 4, 1), 3);
        assert_eq!(move_selection(&plugins, &rows, 1, -1), 3);
    }

    static GROUPED: FakePlugin = FakePlugin::new([1, 0, 2, 0], abi::REQUEST_APPLIED);

    fn grouped_descriptor(version: u32) -> DescHolder {
        let user = &GROUPED as *const FakePlugin as *mut c_void;
        let mut holder = descriptor_with(
            "fx",
            "FX",
            &[
                ("kirby", "Kirby", &["On", "Off", "Reduced"]),
                ("hdr_kirby", "Kirby", &[]),
                (
                    "kirby_hammer",
                    "Hammer",
                    &["Original", "Variant 1", "Variant 2", "Variant 3"],
                ),
                ("status", "EFF variants", &["Ready", "Pending"]),
            ],
            None,
            &[
                Persist::default(),
                Persist {
                    header: true,
                    no_persist: true,
                    read_only: true,
                    ..Persist::default()
                },
                Persist {
                    inline: true,
                    no_persist: true,
                    tones: Some(&[
                        abi::TONE_NEUTRAL,
                        abi::TONE_WARN,
                        abi::TONE_WARN,
                        abi::TONE_WARN,
                    ]),
                    ..Persist::default()
                },
                Persist {
                    read_only: true,
                    ..Persist::default()
                },
            ],
            user,
            Some(fake_current),
            Some(fake_request),
        );
        holder.desc.abi_version = version;
        holder.desc.flags = abi::PLUGIN_FLAG_OWN_PAGE;
        holder
    }

    #[test]
    fn header_rows_parse_and_inline_rows_are_plain_value_rows_in_every_version() {
        for version in [2, 3, abi::ABI_VERSION] {
            let registry = Registry::new();
            let holder = grouped_descriptor(version);
            assert_eq!(
                unsafe { registry.register_raw(&holder.desc) },
                0,
                "version {version}"
            );
            let plugins = registry.snapshot();
            let plugin = plugins[0].clone();
            assert!(plugin.own_page);
            let header = &plugin.options[1];
            assert_eq!(header.kind, RowKind::Header);
            assert!(header.choices.is_empty() && !header.persist && !header.editable());
            let hammer = &plugin.options[2];
            assert_eq!(hammer.kind, RowKind::Value);
            assert_eq!(hammer.choices.len(), 4);
            assert_eq!(plugin.options[3].kind, RowKind::Status);
            // The header answers nothing; nothing is asked of it.
            assert_eq!(registry.step(0, 1, 1), None);
            assert!(GROUPED.requests.lock().unwrap().is_empty());
            // The former inline row steps like any row and keeps its tone.
            assert_eq!(registry.step(0, 2, 1).unwrap().requested, 3);
            assert_eq!(GROUPED.requests.lock().unwrap().as_slice(), &[(2, 3)]);
            assert_eq!(registry.value(0, &plugin, 2).tone, abi::TONE_WARN);
            GROUPED.values[2].store(2, Ordering::Relaxed);
            GROUPED.requests.lock().unwrap().clear();
            // The own page has no group row; the shared page has none of its rows.
            assert_eq!(page_plugins(&plugins), vec![0]);
            assert!(!has_shared_rows(&plugins));
            assert!(rows_for(&plugins, None).is_empty());
            let rows = rows_for(&plugins, Some(0));
            assert_eq!(rows.len(), 4);
            assert_eq!(move_selection(&plugins, &rows, 1, 0), 2);
            assert!(rows_for(&plugins, Some(1)).is_empty());
        }
    }

    #[test]
    fn header_rows_are_validated() {
        let registry = Registry::new();
        // A header must come without choices.
        let mut holder = grouped_descriptor(abi::ABI_VERSION);
        let borrowed_choices = holder._options[0].choices;
        holder._options[1].choice_count = 1;
        holder._options[1].choices = borrowed_choices;
        assert_eq!(
            unsafe { registry.register_raw(&holder.desc) },
            abi::REGISTER_ERR_INVALID
        );
        // Header and inline together are refused.
        let mut holder = grouped_descriptor(abi::ABI_VERSION);
        holder._options[2].flags |= abi::OPTION_FLAG_HEADER;
        assert_eq!(
            unsafe { registry.register_raw(&holder.desc) },
            abi::REGISTER_ERR_INVALID
        );
        // A null choice table on a value row is a null error.
        let mut holder = grouped_descriptor(abi::ABI_VERSION);
        holder._options[0].choices = std::ptr::null();
        assert_eq!(
            unsafe { registry.register_raw(&holder.desc) },
            abi::REGISTER_ERR_NULL
        );
        assert!(registry.snapshot().is_empty());
    }

    static CELLS: FakePlugin = FakePlugin::new([0, 0, 0, 1], abi::REQUEST_APPLIED);

    const READY: &[&str] = &["Ready", "Unavailable"];
    const SAVE: &[&str] = &["Ready", "Saving", "Not saved"];
    const READY_TONES: &[u8] = &[abi::TONE_OK, abi::TONE_ERROR];
    const SAVE_TONES: &[u8] = &[abi::TONE_OK, abi::TONE_WARN, abi::TONE_ERROR];

    /// Value row, passive status row, then three cells: the status page shape.
    fn cell_descriptor(version: u32, cells: usize) -> DescHolder {
        let user = &CELLS as *const FakePlugin as *mut c_void;
        let mut rows: Vec<(&str, &str, &[&str])> = vec![
            ("toggle", "Toggle", &["Off", "On"]),
            ("needs", "Needs pool", &["No", "Yes"]),
        ];
        let mut persist = vec![
            Persist::default(),
            Persist {
                no_persist: true,
                read_only: true,
                ..Persist::default()
            },
        ];
        let names = ["c0", "c1", "c2", "c3", "c4"];
        for (index, name) in names.iter().enumerate().take(cells) {
            rows.push((
                *name,
                ["Pool", "Results", "Save", "Net", "Extra"][index],
                SAVE,
            ));
            persist.push(Persist {
                no_persist: true,
                read_only: true,
                cell: true,
                tones: Some(SAVE_TONES),
                ..Persist::default()
            });
        }
        let mut holder = descriptor_with(
            "status",
            "Status",
            &rows,
            None,
            &persist,
            user,
            Some(fake_current),
            Some(fake_request),
        );
        holder.desc.abi_version = version;
        holder.desc.flags = abi::PLUGIN_FLAG_OWN_PAGE;
        holder
    }

    #[test]
    fn cell_rows_parse_only_in_revision_five_and_are_passive() {
        let registry = Registry::new();
        // Older revisions refuse the new bit exactly as they refused any
        // unknown bit before revision 5 existed.
        for version in [2, 3, abi::ABI_VERSION] {
            let holder = cell_descriptor(version, 3);
            assert_eq!(
                unsafe { registry.register_raw(&holder.desc) },
                abi::REGISTER_ERR_INVALID,
                "version {version}"
            );
        }
        assert!(registry.snapshot().is_empty());
        let holder = cell_descriptor(abi::CELL_ABI_VERSION, 3);
        assert_eq!(unsafe { registry.register_raw(&holder.desc) }, 0);
        // Revision 6 is still unknown.
        let mut future = cell_descriptor(abi::CELL_ABI_VERSION + 1, 3);
        future.desc.flags = 0;
        assert_eq!(
            unsafe { Registry::new().register_raw(&future.desc) },
            abi::REGISTER_ERR_ABI
        );
        let plugins = registry.snapshot();
        let plugin = plugins[0].clone();
        assert_eq!(
            plugin.options.iter().map(|o| o.kind).collect::<Vec<_>>(),
            [
                RowKind::Value,
                RowKind::Status,
                RowKind::Cell,
                RowKind::Cell,
                RowKind::Cell
            ]
        );
        for cell in &plugin.options[2..] {
            assert!(!cell.editable() && !cell.persist);
            assert_eq!(cell.tones.as_deref(), Some(SAVE_TONES));
        }
        // A cell is never requested, even when the plugin could answer.
        for option in 1..5 {
            assert_eq!(registry.step(0, option, 1), None, "option {option}");
            assert_eq!(registry.step(0, option, -1), None, "option {option}");
        }
        assert!(CELLS.requests.lock().unwrap().is_empty());
        // The value follows the plugin and the tone says ready or not.
        let ready = registry.value(0, &plugin, 2);
        assert_eq!((ready.label.as_str(), ready.tone), ("Ready", abi::TONE_OK));
        let saving = registry.value(0, &plugin, 3);
        assert_eq!(
            (saving.label.as_str(), saving.tone),
            ("Saving", abi::TONE_WARN)
        );
        let last = registry.value(0, &plugin, 4);
        assert_eq!((last.label.as_str(), last.tone), ("Ready", abi::TONE_OK));
        assert!(!ready.pending && !saving.pending && !last.pending);
    }

    #[test]
    fn cell_rows_are_validated() {
        let bad = |holder: &DescHolder| unsafe { Registry::new().register_raw(&holder.desc) };
        // More than four consecutive cells are refused.
        assert_eq!(
            bad(&cell_descriptor(abi::CELL_ABI_VERSION, 5)),
            abi::REGISTER_ERR_INVALID
        );
        assert_eq!(bad(&cell_descriptor(abi::CELL_ABI_VERSION, 4)), 0);
        // A cell needs its tone table, which is the ready/not-ready contract.
        let mut holder = cell_descriptor(abi::CELL_ABI_VERSION, 3);
        holder._options[3].choice_tones = std::ptr::null();
        assert_eq!(bad(&holder), abi::REGISTER_ERR_INVALID);
        // A cell cannot also be a header or an inline row.
        for extra in [abi::OPTION_FLAG_HEADER, abi::OPTION_FLAG_INLINE] {
            let mut holder = cell_descriptor(abi::CELL_ABI_VERSION, 3);
            holder._options[3].flags |= extra;
            assert_eq!(bad(&holder), abi::REGISTER_ERR_INVALID, "extra {extra}");
        }
        // Tone bytes are still validated.
        let mut holder = cell_descriptor(abi::CELL_ABI_VERSION, 3);
        let tones = [abi::TONE_MAX + 1; 3];
        holder._options[3].choice_tones = tones.as_ptr();
        assert_eq!(bad(&holder), abi::REGISTER_ERR_INVALID);
        // Two runs separated by another row may each use the full run.
        let user = &CELLS as *const FakePlugin as *mut c_void;
        let mut rows: Vec<(&str, &str, &[&str])> = Vec::new();
        let mut persist = Vec::new();
        let names = ["a0", "a1", "a2", "a3", "sep", "b0", "b1", "b2", "b3"];
        for name in names {
            rows.push((name, "Cell", SAVE));
            persist.push(Persist {
                cell: name != "sep",
                tones: Some(SAVE_TONES),
                ..Persist::default()
            });
        }
        let holder = descriptor_with(
            "runs",
            "Runs",
            &rows,
            None,
            &persist,
            user,
            Some(fake_current),
            Some(fake_request),
        );
        let mut holder = holder;
        holder.desc.abi_version = abi::CELL_ABI_VERSION;
        assert_eq!(bad(&holder), 0);
    }

    #[test]
    fn passive_rows_never_hold_the_cursor_and_cells_group_by_run() {
        let registry = Registry::new();
        let holder = cell_descriptor(abi::CELL_ABI_VERSION, 3);
        assert_eq!(unsafe { registry.register_raw(&holder.desc) }, 0);
        let plugins = registry.snapshot();
        let rows = rows_for(&plugins, Some(0));
        assert_eq!(rows.len(), 5);
        assert_eq!(
            rows.iter()
                .map(|row| selectable(&plugins, *row))
                .collect::<Vec<_>>(),
            [true, false, false, false, false]
        );
        // Only the toggle can hold the cursor, from any index and direction.
        for selected in 0..8 {
            for direction in [-1, 0, 1] {
                assert_eq!(
                    move_selection(&plugins, &rows, selected, direction),
                    0,
                    "{selected}/{direction}"
                );
            }
        }
        assert_eq!(cell_group(&plugins, &rows, 0), None);
        assert_eq!(cell_group(&plugins, &rows, 1), None);
        for index in 2..5 {
            assert_eq!(cell_group(&plugins, &rows, index), Some((2, 5)), "{index}");
        }
        assert_eq!(cell_group(&plugins, &rows, 9), None);
    }

    #[test]
    fn a_page_without_an_editable_row_has_no_selectable_row() {
        let registry = Registry::new();
        let holder = descriptor_with(
            "diag",
            "Diag",
            &[("a", "A", READY), ("b", "B", &["Ready"]), ("c", "C", READY)],
            None,
            &[
                Persist {
                    read_only: true,
                    ..Persist::default()
                },
                Persist::default(),
                Persist {
                    cell: true,
                    tones: Some(READY_TONES),
                    ..Persist::default()
                },
            ],
            &CELLS as *const FakePlugin as *mut c_void,
            Some(fake_current),
            Some(fake_request),
        );
        let mut holder = holder;
        holder.desc.abi_version = abi::CELL_ABI_VERSION;
        holder.desc.flags = abi::PLUGIN_FLAG_OWN_PAGE;
        assert_eq!(unsafe { registry.register_raw(&holder.desc) }, 0);
        let plugins = registry.snapshot();
        let rows = rows_for(&plugins, Some(0));
        assert!(rows.iter().all(|row| !selectable(&plugins, *row)));
        // The cursor index still scrolls (the host highlights nothing), one
        // row per press, and Left/Right request nothing.
        assert_eq!(move_selection(&plugins, &rows, 0, 1), 1);
        assert_eq!(move_selection(&plugins, &rows, 2, 1), 0);
        assert_eq!(move_selection(&plugins, &rows, 0, -1), 2);
        for option in 0..3 {
            assert_eq!(registry.step(0, option, 1), None);
        }
    }
}
