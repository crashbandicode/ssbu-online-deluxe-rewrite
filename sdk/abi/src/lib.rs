//! Online Deluxe plugin registration ABI, version 1.
//!
//! A separately built Skyline NRO registers menu options with the Deluxe host.
//! The host draws them on its Plugins page, in the vertical window outside a
//! match and in the compact two-row strip during a ready match. The plugin
//! supplies no ImGui code and owns its own behavior and persistence.
//!
//! Everything here is `#[repr(C)]` and has no dependencies so the same crate
//! compiles into the host, into the Rust SDK and into host-side tests. Breaking
//! changes bump [`ABI_VERSION`] and the exported symbol name together.
//!
//! # Discovery
//!
//! The host exports [`API_SYMBOL`]. A plugin resolves it at runtime with
//! `nn::ro::LookupSymbol` and calls it to receive a [`PluginApiV1`] table.
//! Skyline loads every plugin NRO before running any `main`, so lookup works
//! from the plugin's `main` regardless of load order. A null result means no
//! compatible Deluxe host is installed; the plugin must keep working without
//! a menu row.
//!
//! # Lifetime and thread rules
//!
//! * [`PluginApiV1::register`] copies every string out of the descriptor
//!   during the call. The descriptor and its strings only need to live for
//!   the duration of that call.
//! * `user`, `current` and `request` are retained for the life of the process.
//!   There is no unregister in version 1.
//! * Callbacks run on the host's overlay draw thread while the menu is open.
//!   They must be non-blocking, must not call back into the API and must not
//!   assume any game thread. `current` runs once per visible option per frame;
//!   `request` runs once per accepted controller edge.
//!
//! # Optional host persistence
//!
//! A plugin that keeps a `key=value` INI file may set [`PluginDesc::config_path`].
//! After a menu edit that the plugin answers with `REQUEST_APPLIED` or
//! `REQUEST_PENDING`, the host rewrites that file on its background thread:
//! the line for the option's key is replaced (or appended) with
//! `key=value`, every other line is kept, and the write is atomic with a
//! `.previous.ini` backup. The key is [`OptionDesc::config_key`] or the option
//! id; the value is the entry of [`OptionDesc::choice_values`] for the chosen
//! index, or the decimal index when that table is null. The host never reads
//! the file: the plugin loads it at startup and reports through `current`.
//! Read-only options and options flagged [`OPTION_FLAG_NO_PERSIST`] are not
//! written.

#![no_std]

use core::ffi::{c_char, c_void};

/// Descriptor revision the SDK sends, carried in [`PluginDesc::abi_version`]. Versions 2, 3
/// and 4 share one layout ([`OptionDesc`], 56 bytes); each added flag bits
/// and widened the option limit. Hosts since Deluxe 0.5.3 read every
/// accepted version under the version 4 rules, so a version 2 or 3
/// descriptor is validated exactly like a version 4 one.
pub const ABI_VERSION: u32 = 4;
/// Descriptor revision 5: the version 4 layout plus [`OPTION_FLAG_CELL`].
/// Only a plugin that sends a cell option should send it. A host that
/// predates cells answers [`REGISTER_ERR_ABI`], so the plugin can register a
/// flat version 4 layout instead. Revisions 2 to 4 refuse the cell bit exactly
/// as they refused any unknown bit before revision 5 existed.
pub const CELL_ABI_VERSION: u32 = 5;
/// Oldest descriptor revision the host accepts. Versions 2 and 3 share the
/// version 4 layout and are read under the version 4 rules. Version 1 (a
/// 48-byte row without tones) is refused with [`REGISTER_ERR_ABI`]; no
/// released plugin ever sent it.
pub const MIN_ABI_VERSION: u32 = 2;
/// Revision carried in [`PluginApiV1::abi_version`]. The exported table shape
/// has not changed, so consumers built against version 1 keep matching it.
pub const TABLE_VERSION: u32 = 1;

/// Exported host function name: `extern "C" fn() -> *const PluginApiV1`.
pub const API_SYMBOL: &str = "ssbu_online_deluxe_plugin_api_v1";

/// NUL-terminated form of [`API_SYMBOL`] for `LookupSymbol`.
pub const API_SYMBOL_C: &[u8] = b"ssbu_online_deluxe_plugin_api_v1\0";

/// Most plugins one host accepts.
pub const MAX_PLUGINS: usize = 16;
/// Most options one plugin may declare (a row per fighter plus a header and
/// a variant row per move fits). Hosts before Deluxe 0.5.3 accepted 32 for a
/// version 3 descriptor and 8 for version 2.
pub const MAX_OPTIONS: usize = 64;
/// Most choices one option may declare. One choice makes a read-only status row.
pub const MAX_CHOICES: usize = 16;
/// Longest `id` in bytes, excluding the terminator. Lowercase ASCII, digits, `_`.
pub const MAX_ID_LEN: usize = 32;
/// Longest display label or choice label in bytes, excluding the terminator.
pub const MAX_LABEL_LEN: usize = 24;
/// Longest persisted value token in bytes, excluding the terminator.
pub const MAX_VALUE_LEN: usize = 24;
/// Longest `config_path` in bytes, excluding the terminator.
pub const MAX_PATH_LEN: usize = 255;
/// Required prefix of `config_path`.
pub const CONFIG_PATH_PREFIX: &str = "sd:/";

/// `PluginDesc::flags`: draw this plugin on its own page in the window cycle
/// (settings → plugins → each own page → performance), titled with the
/// plugin label, instead of as a group on the shared Plugins page.
pub const PLUGIN_FLAG_OWN_PAGE: u32 = 1 << 0;

/// Exported host function name (0.5.0): `extern "C" fn() -> u32` returning
/// `ONLINE_CONTEXT_*` bits from the host's scene tracker. Resolve it with
/// `nn::ro::LookupSymbol`; a missing symbol means an older host.
pub const ONLINE_SYMBOL: &str = "ssbu_online_deluxe_online_context_v1";
/// NUL-terminated form of [`ONLINE_SYMBOL`].
pub const ONLINE_SYMBOL_C: &[u8] = b"ssbu_online_deluxe_online_context_v1\0";
/// Signature of the exported [`ONLINE_SYMBOL`].
pub type OnlineContextFn = unsafe extern "C" fn() -> u32;
/// Online context bit: the host's scene hooks are installed and ready.
pub const ONLINE_CONTEXT_READY: u32 = 1 << 0;
/// Online context bit: the scene is online (Quickplay, Arena or local
/// wireless, or the game's online-latency hook reached the current match).
pub const ONLINE_CONTEXT_ONLINE: u32 = 1 << 1;
/// Online context bit: a Singles or Doubles stage is loaded.
pub const ONLINE_CONTEXT_MATCH: u32 = 1 << 2;
/// Online context bit: the scene is Training (also the Quickplay waiting room).
pub const ONLINE_CONTEXT_TRAINING: u32 = 1 << 3;

/// `OptionDesc::flags`: never write this option even when the plugin persists.
pub const OPTION_FLAG_NO_PERSIST: u32 = 1 << 0;
/// `OptionDesc::flags`: a pure status row. Left/Right do nothing, `request`
/// is never called, and the value shown is whatever `current` reports, so a
/// multi-choice row can display state such as Ok/Warn/Fail.
pub const OPTION_FLAG_READ_ONLY: u32 = 1 << 1;
/// `OptionDesc::flags`: a header. The row shows only its label, flush with
/// the page: Up/Down skip it, Left/Right never reach it, `current` and
/// `request` are never called and nothing is persisted. `choice_count` must
/// be 0 and `choices`, `choice_values` and `choice_tones` null. Cannot be
/// combined with [`OPTION_FLAG_INLINE`].
pub const OPTION_FLAG_HEADER: u32 = 1 << 2;
/// `OptionDesc::flags`: accepted and drawn as an ordinary value row
/// (`Label: current choice`). Hosts 0.5.3 before the consolidation drew every
/// choice on the line; no released plugin sends this bit.
pub const OPTION_FLAG_INLINE: u32 = 1 << 3;
/// `OptionDesc::flags` (descriptor revision 5 only): a passive status cell.
/// Like [`OPTION_FLAG_READ_ONLY`] the row is never selected, never requested
/// and never persisted, and `current` picks the value. The host draws a run
/// of 1 to [`MAX_CELL_RUN`] consecutive cell options of one plugin as one
/// table: each cell's `label` is a column heading (separated by vertical
/// bars) and, centered beneath it, a check mark when the chosen value's tone
/// is [`TONE_OK`] and a cross otherwise, in that tone's color. The host
/// draws both marks as lines, so no font glyph is needed. `choice_tones` must
/// be non-null (the tone is the ready/not-ready contract) and the choice
/// labels are still validated, but the table shows no value text. A plugin
/// that has only status rows needs no highlight, and nothing here is
/// selectable. Cannot be combined with [`OPTION_FLAG_HEADER`] or
/// [`OPTION_FLAG_INLINE`]; `OPTION_FLAG_NO_PERSIST` and
/// [`OPTION_FLAG_READ_ONLY`] are accepted and implied.
pub const OPTION_FLAG_CELL: u32 = 1 << 4;
/// Most consecutive cell options one plugin may declare (one table).
pub const MAX_CELL_RUN: usize = 4;

/// `choice_tones` entry: default text colour.
pub const TONE_NEUTRAL: u8 = 0;
/// `choice_tones` entry: green, something is working.
pub const TONE_OK: u8 = 1;
/// `choice_tones` entry: yellow, degraded or pending attention.
pub const TONE_WARN: u8 = 2;
/// `choice_tones` entry: red, failed or unavailable.
pub const TONE_ERROR: u8 = 3;
/// Highest valid tone byte.
pub const TONE_MAX: u8 = TONE_ERROR;

/// `current` result meaning the option has no displayable value right now.
pub const CHOICE_UNAVAILABLE: u32 = u32::MAX;

/// `request` result: the change took effect.
pub const REQUEST_APPLIED: u32 = 0;
/// `request` result: accepted, applies later (for example after the match).
pub const REQUEST_PENDING: u32 = 1;
/// `request` result: refused in the current state. Nothing changed.
pub const REQUEST_REJECTED: u32 = 2;
/// `request` result: the feature behind this option is unavailable.
pub const REQUEST_UNAVAILABLE: u32 = 3;

/// `register` failed: null descriptor or null required pointer.
pub const REGISTER_ERR_NULL: i32 = -1;
/// `register` failed: `struct_size` or `abi_version` is not this ABI.
pub const REGISTER_ERR_ABI: i32 = -2;
/// `register` failed: a string, count or id charset is outside the limits.
pub const REGISTER_ERR_INVALID: i32 = -3;
/// `register` failed: the host already holds [`MAX_PLUGINS`] plugins.
pub const REGISTER_ERR_FULL: i32 = -4;
/// `register` failed: another plugin already registered this `id`.
pub const REGISTER_ERR_DUPLICATE: i32 = -5;
/// `register` failed: `current` or `request` is null.
pub const REGISTER_ERR_CALLBACKS: i32 = -6;

/// Host status bit: the Deluxe menu hooks are installed, so registered rows
/// can actually be shown. Clear when the host skipped its hooks (unsupported
/// game version, legacy SsbuSync host present, or a disabled MENU flag).
pub const HOST_STATUS_MENU_ACTIVE: u32 = 1 << 0;

/// Returns the index of the option's current choice, or [`CHOICE_UNAVAILABLE`].
pub type CurrentFn = unsafe extern "C" fn(user: *mut c_void, option: u32) -> u32;

/// Asks the plugin to move `option` to `choice`. Returns one of the
/// `REQUEST_*` codes. The host re-reads `current` on the next frame; it never
/// assumes the change succeeded.
pub type RequestFn = unsafe extern "C" fn(user: *mut c_void, option: u32, choice: u32) -> u32;

/// One menu row (descriptor versions 2 to 4, 56 bytes).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct OptionDesc {
    /// Stable identifier, unique within the plugin. Not displayed.
    pub id: *const c_char,
    /// Display label. Keep it short; the compact strip shows `Label:Value`.
    pub label: *const c_char,
    /// `choice_count` NUL-terminated display labels, in Left/Right order.
    pub choices: *const *const c_char,
    pub choice_count: u32,
    /// `OPTION_FLAG_*` bits; zero when none apply.
    pub flags: u32,
    /// INI key used by host persistence, or null for the option id. Same
    /// charset as an id.
    pub config_key: *const c_char,
    /// `choice_count` NUL-terminated value tokens written by host persistence
    /// (for example `off`/`on`), or null to write the decimal choice index.
    /// Tokens are 1 to [`MAX_VALUE_LEN`] printable ASCII bytes with no
    /// whitespace, `=` or `#`.
    pub choice_values: *const *const c_char,
    /// Null for neutral text, or `choice_count` `TONE_*` bytes: the host draws
    /// the current choice's value text in that tone in the vertical table and
    /// in the in-match strip.
    pub choice_tones: *const u8,
}

/// One plugin's registration.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PluginDesc {
    /// `size_of::<PluginDesc>()` (72).
    pub struct_size: u32,
    /// Descriptor revision, [`MIN_ABI_VERSION`] to [`CELL_ABI_VERSION`]; the
    /// SDK always sends [`ABI_VERSION`]. Any other value answers
    /// [`REGISTER_ERR_ABI`].
    pub abi_version: u32,
    /// Stable identifier, unique across plugins. Not displayed.
    pub id: *const c_char,
    /// Display label. Heads the plugin's rows on the shared Plugins page and
    /// titles its own page.
    pub label: *const c_char,
    pub options: *const OptionDesc,
    pub option_count: u32,
    /// `PLUGIN_FLAG_*` bits ([`PLUGIN_FLAG_OWN_PAGE`]); zero otherwise.
    pub flags: u32,
    /// Opaque pointer handed back to every callback.
    pub user: *mut c_void,
    pub current: Option<CurrentFn>,
    pub request: Option<RequestFn>,
    /// Null, or the NUL-terminated `sd:/...` path of the plugin's own
    /// `key=value` INI that the host should update after menu edits. See the
    /// crate docs. At most [`MAX_PATH_LEN`] bytes.
    pub config_path: *const c_char,
}

/// Function table returned by the exported [`API_SYMBOL`].
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PluginApiV1 {
    /// `size_of::<PluginApiV1>()`.
    pub struct_size: u32,
    /// [`TABLE_VERSION`] (always 1; the table shape is unchanged).
    pub abi_version: u32,
    /// Registers one plugin. Returns a non-negative handle, or a `REGISTER_ERR_*` code.
    pub register: unsafe extern "C" fn(desc: *const PluginDesc) -> i32,
    /// `HOST_STATUS_*` bits.
    pub host_status: unsafe extern "C" fn() -> u32,
    /// NUL-terminated host build identifier. Valid for the life of the process.
    pub host_build: unsafe extern "C" fn() -> *const c_char,
}

/// Signature of the exported [`API_SYMBOL`].
pub type ApiEntryFn = unsafe extern "C" fn() -> *const PluginApiV1;

/// Byte-level identity checks a consumer can use before trusting a table.
pub const fn table_matches(struct_size: u32, abi_version: u32) -> bool {
    struct_size as usize == core::mem::size_of::<PluginApiV1>() && abi_version == TABLE_VERSION
}

/// True when a host should parse a descriptor of this revision.
pub const fn descriptor_version_supported(abi_version: u32) -> bool {
    abi_version >= MIN_ABI_VERSION && abi_version <= CELL_ABI_VERSION
}

/// True when every tone byte is a known `TONE_*` value.
pub fn valid_tones(tones: &[u8]) -> bool {
    tones.iter().all(|tone| *tone <= TONE_MAX)
}

/// True when `id` is non-empty, within [`MAX_ID_LEN`] and uses only
/// lowercase ASCII letters, digits and underscores.
pub fn valid_id(id: &[u8]) -> bool {
    !id.is_empty()
        && id.len() <= MAX_ID_LEN
        && id
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
}

/// True when `label` is non-empty UTF-8 within [`MAX_LABEL_LEN`] bytes and
/// contains no control characters.
pub fn valid_label(label: &[u8]) -> bool {
    !label.is_empty()
        && label.len() <= MAX_LABEL_LEN
        && core::str::from_utf8(label).is_ok_and(|text| !text.chars().any(char::is_control))
}

/// True when `value` is a persistable token: 1 to [`MAX_VALUE_LEN`] printable
/// ASCII bytes with no whitespace, `=` or `#`.
pub fn valid_value(value: &[u8]) -> bool {
    !value.is_empty()
        && value.len() <= MAX_VALUE_LEN
        && value
            .iter()
            .all(|byte| byte.is_ascii_graphic() && *byte != b'=' && *byte != b'#')
}

/// True when `path` starts with [`CONFIG_PATH_PREFIX`], is within
/// [`MAX_PATH_LEN`] bytes, is UTF-8 and has no control characters.
pub fn valid_config_path(path: &[u8]) -> bool {
    path.len() <= MAX_PATH_LEN
        && path.starts_with(CONFIG_PATH_PREFIX.as_bytes())
        && path.len() > CONFIG_PATH_PREFIX.len()
        && core::str::from_utf8(path).is_ok_and(|text| !text.chars().any(char::is_control))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_is_fixed_and_versions_two_to_five_are_accepted() {
        assert_eq!(core::mem::size_of::<OptionDesc>(), 56);
        assert_eq!(core::mem::size_of::<PluginDesc>(), 72);
        assert_eq!(core::mem::size_of::<PluginApiV1>(), 32);
        assert_eq!(TABLE_VERSION, 1);
        assert_eq!((MIN_ABI_VERSION, ABI_VERSION, CELL_ABI_VERSION), (2, 4, 5));
        assert_eq!(OPTION_FLAG_CELL, 1 << 4);
        assert_eq!(
            OPTION_FLAG_CELL
                & (OPTION_FLAG_NO_PERSIST
                    | OPTION_FLAG_READ_ONLY
                    | OPTION_FLAG_HEADER
                    | OPTION_FLAG_INLINE),
            0
        );
        assert_eq!(MAX_CELL_RUN, 4);
        assert_eq!(MAX_OPTIONS, 64);
        assert_eq!(OPTION_FLAG_HEADER & OPTION_FLAG_INLINE, 0);
        assert_eq!(
            (OPTION_FLAG_HEADER | OPTION_FLAG_INLINE)
                & (OPTION_FLAG_NO_PERSIST | OPTION_FLAG_READ_ONLY),
            0
        );
        assert_eq!(PLUGIN_FLAG_OWN_PAGE, 1);
        assert!(table_matches(32, 1));
        assert!(!table_matches(32, 2));
        assert!(!table_matches(24, 1));
        assert!(!descriptor_version_supported(0));
        assert!(!descriptor_version_supported(1));
        assert!(descriptor_version_supported(2));
        assert!(descriptor_version_supported(3));
        assert!(descriptor_version_supported(4));
        assert!(descriptor_version_supported(5));
        assert!(!descriptor_version_supported(6));
        assert_eq!(ONLINE_SYMBOL_C.len(), ONLINE_SYMBOL.len() + 1);
        assert_eq!(ONLINE_SYMBOL_C[ONLINE_SYMBOL.len()], 0);
        assert_eq!(
            ONLINE_CONTEXT_READY
                | ONLINE_CONTEXT_ONLINE
                | ONLINE_CONTEXT_MATCH
                | ONLINE_CONTEXT_TRAINING,
            0b1111
        );
        assert!(valid_tones(&[0, 1, 2, 3]));
        assert!(!valid_tones(&[0, 4]));
        assert_eq!(API_SYMBOL_C.len(), API_SYMBOL.len() + 1);
        assert_eq!(&API_SYMBOL_C[..API_SYMBOL.len()], API_SYMBOL.as_bytes());
        assert_eq!(API_SYMBOL_C[API_SYMBOL.len()], 0);
    }

    #[test]
    fn identifier_and_label_limits() {
        assert!(valid_id(b"replay_recorder"));
        assert!(valid_id(b"a1"));
        assert!(!valid_id(b""));
        assert!(!valid_id(b"Replay"));
        assert!(!valid_id(b"re play"));
        assert!(!valid_id(&[b'a'; MAX_ID_LEN + 1]));
        assert!(valid_id(&[b'a'; MAX_ID_LEN]));
        assert!(valid_label("Replay".as_bytes()));
        assert!(valid_label("On (next match)".as_bytes()));
        assert!(!valid_label(b""));
        assert!(!valid_label(b"bad\nlabel"));
        assert!(!valid_label(&[0xff, 0xfe]));
        assert!(!valid_label(&[b'x'; MAX_LABEL_LEN + 1]));
        assert!(valid_value(b"on"));
        assert!(valid_value(b"1080p_strict"));
        assert!(!valid_value(b""));
        assert!(!valid_value(b"o n"));
        assert!(!valid_value(b"a=b"));
        assert!(!valid_value(b"#x"));
        assert!(!valid_value(&[b'v'; MAX_VALUE_LEN + 1]));
        assert!(valid_config_path(b"sd:/ultimate/replay/config.ini"));
        assert!(!valid_config_path(b"sd:/"));
        assert!(!valid_config_path(b"/ultimate/replay/config.ini"));
        assert!(!valid_config_path(b"sd:/bad\nname.ini"));
        let mut long = CONFIG_PATH_PREFIX.as_bytes().to_vec();
        long.extend(core::iter::repeat_n(b'a', MAX_PATH_LEN));
        assert!(!valid_config_path(&long));
    }
}
