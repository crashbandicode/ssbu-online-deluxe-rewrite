//! Display text for the CSS and arena titles. Pointer writes stay on Switch.

use crate::pia_rtt::PingRgba;

pub const MARKER: char = '\u{200B}';
pub const MARKER_BYTES: usize = 3;
pub const ROOM_ID_UNITS: usize = 5;
/// Wire name window (`ENTRY_NAME_LEN`). Longer display names are cut on a char boundary.
pub const ARENA_NAME_MAX: usize = 40;
pub const ARENA_PROFILE_MAX: usize = 9;
pub const ARENA_LATENCY_MAX: usize = 4;
/// `9999999ms` plus ` [Wired]`.
pub const ARENA_PING_MAX: usize = 17;
pub const ROOM_LINE_MAX: usize = 25;
/// Longest row label plus the applied FPS++ `++` suffix.
pub const YOU_PROFILE_MAX: usize = 20;
pub const YOU_LINE_MAX: usize = 32;

pub const fn remote_line_max() -> usize {
    1 + ARENA_NAME_MAX + 2 + ARENA_PROFILE_MAX + 3 + ARENA_LATENCY_MAX + 2 + ARENA_PING_MAX + 1
}

/// Marker, room line, You line, and eight remote lines at the bounds above.
pub const TEXT_CAP: usize = MARKER_BYTES + ROOM_LINE_MAX + YOU_LINE_MAX + 8 * remote_line_max();
/// One past the text the setter is given. Longer declared buffers are corrupt.
pub const PANE_SCAN_MAX: usize = TEXT_CAP + 1;

const _: () = assert!(TEXT_CAP == 692);
const _: () = assert!(PANE_SCAN_MAX <= 1024);

/// Local-wireless update site. Words are the clean 13.0.5 entry from the pane probe.
pub const LOCAL_UPDATE: usize = 0x1bd7a80;
pub const LOCAL_UPDATE_WORDS: [u32; 4] = [0x3600_00a0, 0xf940_c268, 0xb942_5517, 0x3700_08f6];
/// Existing scene entry. Its hook is already installed; this word is checked before that patch.
pub const LOCAL_SCENE: usize = 0x1bd45e0;
pub const LOCAL_SCENE_WORD: u32 = 0x946e_9728;
pub const LOCAL_PANE_NAME: &str = "set_text_dev_count";
/// `0x85666320 - 0x80425000` from the same pane at the scene entry and the update site.
pub const LOCAL_PANE_VTABLE: u64 = 0x0052_41320;
/// `nn::ldn::NetworkState::Initialized`, observed from `GetState`.
pub const LDN_INITIALIZED: u32 = 1;

/// `LookupSymbol` names. The 13.0.5 image stores these without LLVM's leading `0x01`.
pub const LDN_INITIALIZE: &[u8] = b"_ZN2nn3ldn10InitializeEv\0";
pub const LDN_GET_STATE: &[u8] = b"_ZN2nn3ldn8GetStateEv\0";
pub const LDN_SCAN: &[u8] = b"_ZN2nn3ldn4ScanEPNS0_11NetworkInfoEPiiRKNS0_10ScanFilterEi\0";

pub fn local_update_accepted(words: [u32; 4]) -> bool {
    words == LOCAL_UPDATE_WORDS
}

/// Name and vtable from the probed pane. The vtable is an offset from main text.
pub fn local_pane_identity(name: &str, vtable: u64, text_base: u64) -> bool {
    text_base != 0 && name == LOCAL_PANE_NAME && vtable == text_base.wrapping_add(LOCAL_PANE_VTABLE)
}

/// All three `LookupSymbol` results must be distinct and instruction-aligned.
pub fn ldn_symbols_usable(initialize: usize, get_state: usize, scan: usize) -> bool {
    let ok = |address: usize| address != 0 && address.is_multiple_of(4);
    ok(initialize)
        && ok(get_state)
        && ok(scan)
        && initialize != get_state
        && get_state != scan
        && scan != initialize
}

/// Missing symbols, a stale scene generation, or the wrong pane leave the game text alone.
pub fn local_count_active(
    symbols_ready: bool,
    scene_generation: u64,
    captured_generation: u64,
    identity_ok: bool,
) -> bool {
    symbols_ready
        && captured_generation != 0
        && scene_generation == captured_generation
        && identity_ok
}

/// Fork arena ping suffix. Unknown and emulated types add nothing.
pub fn interface_suffix(interface: &str) -> &'static str {
    match interface {
        "Wi-Fi" => " [Wifi]",
        "Ethernet" => " [Wired]",
        _ => "",
    }
}

/// Absolute telemetry size. Never a multiple of the live size.
pub const TELEMETRY_FONT: FontMetrics = FontMetrics {
    font_x: 25.48,
    font_y: 51.0,
    line_space: -8.0,
    char_space: 1.0,
    // The fork calls Center/Center; both pinned nnsdk enum values are zero.
    text_position: 0,
};

/// Byte material colors that let the text color show. Index 0 is black, 1 is white.
pub const TELEMETRY_MATERIAL: [[u8; 4]; 2] = [[0, 0, 0, 255], [255, 255, 255, 255]];

/// Fork CSS walk: parent, back two (the background), up two, then prev (second title).
pub const CSS_TITLE_WALK: [PaneLink; 6] = [
    PaneLink::Parent,
    PaneLink::Prev,
    PaneLink::Prev,
    PaneLink::Parent,
    PaneLink::Parent,
    PaneLink::Prev,
];

pub const CSS_BACKGROUND_STEPS: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneLink {
    Parent,
    Prev,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontMetrics {
    pub font_x: f32,
    pub font_y: f32,
    pub line_space: f32,
    pub char_space: f32,
    pub text_position: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphNode {
    pub parent: Option<usize>,
    pub prev: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BannerPaint {
    Leave,
    Write {
        text: String,
        font: Option<FontMetrics>,
        colors: [[u8; 4]; 2],
        material: Option<[[u8; 4]; 2]>,
        /// `None` on CSS. Arena telemetry uses the fork Y; Streamer restores the original.
        pos_y: Option<f32>,
    },
}

/// Complete remote lines an arena pane can show. Matches the handshake table.
pub const ARENA_REMOTE_CAP: u8 = 8;

/// Fork `update_online_arena_ui` Y. The count is complete lines actually written.
pub fn arena_pos_y(displayed_remote_count: u8) -> f32 {
    let lines = f32::from(displayed_remote_count.min(ARENA_REMOTE_CAP));
    -115.0 - (22.5 * lines)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observe {
    /// Empty text, or text this mod already wrote. Do not touch the saved style.
    Keep,
    /// Same pane showed game text again. Refresh the saved words only.
    RefreshText,
    /// New address, vtable, name, or scene generation. Capture style once.
    Replace,
}

pub struct LineParts<'a> {
    pub you_profile: &'a str,
    pub you_latency: &'a str,
    pub show_remote: bool,
    pub remote_name: &'a str,
    pub remote_profile: &'a str,
    pub remote_latency: &'a str,
    pub ping: &'a str,
}

pub struct PaintInput<'a> {
    pub same_identity: bool,
    pub captured: bool,
    pub telemetry_applied: bool,
    pub streamer: bool,
    /// Arena keeps its captured font. CSS keeps `TELEMETRY_FONT`.
    pub arena: bool,
    pub current: &'a str,
    pub saved_text: &'a str,
    pub saved_font: Option<FontMetrics>,
    pub live_font: Option<FontMetrics>,
    pub saved_colors: [[u8; 4]; 2],
    pub saved_material: Option<[[u8; 4]; 2]>,
    pub material_matches: bool,
    pub telemetry: &'a str,
    pub ping: Option<[u8; 4]>,
    /// Original `Pane.pos_y`, captured once with the arena style.
    pub saved_pos_y: Option<f32>,
    /// Complete remote lines written on this arena pane. CSS passes 0.
    pub remote_lines: u8,
}

pub struct PaintOutcome {
    pub paint: BannerPaint,
    pub telemetry_applied: bool,
}

pub fn local_latency_label(buffer: i8) -> String {
    if buffer == -1 {
        "Auto".to_owned()
    } else {
        buffer.to_string()
    }
}

/// Last `nn::ldn::Scan` count. Negative results are zero. The value stays a `u8`.
pub fn store_scan_count(found: i32) -> u8 {
    if found <= 0 {
        0
    } else {
        u8::try_from(found).unwrap_or(u8::MAX)
    }
}

/// Fork `num_available_networks`: only Initialized (`GetState == 1`) keeps the scan.
pub fn displayed_network_count(state: u32, stored: u8) -> u8 {
    visible_network_count(state == LDN_INITIALIZED, stored)
}

fn visible_network_count(initialized: bool, stored: u8) -> u8 {
    if initialized {
        stored
    } else {
        0
    }
}

/// Local-wireless pane. Streamer replaces the private buffer with the scan count.
///
/// Provenance: `reference/quickplay/src/ui/native/mod.rs` `update_local_online_ui`
/// and `reference/quickplay/src/net/ldn/mod.rs`. `Initialize` stores zero then
/// calls the original once. `Scan` calls the original once, then keeps the count.
/// `GetState == 1` is the only state that shows that count.
pub fn local_wireless_text(streamer: bool, latency: &str, initialized: bool, stored: u8) -> String {
    if streamer {
        visible_network_count(initialized, stored).to_string()
    } else {
        field(latency, "---")
    }
}

/// One arena remote line. Fields already belong to the same station.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemoteLine<'a> {
    pub name: &'a str,
    pub profile: &'a str,
    pub latency: &'a str,
    pub ping: &'a str,
}

/// Room id, the local line, then each complete remote line that fits.
/// A line that would exceed [`TEXT_CAP`] is omitted whole.
pub fn arena_status(
    room_id: &str,
    you_profile: &str,
    you_latency: &str,
    remotes: &[RemoteLine<'_>],
) -> (String, u8) {
    let you_profile = bound_bytes(&field(you_profile, "---"), YOU_PROFILE_MAX);
    let you_latency = bound_bytes(&field(you_latency, "---"), ARENA_LATENCY_MAX);
    let room = bound_bytes(&field(room_id, ""), ROOM_LINE_MAX.saturating_sub(10));
    let mut body = if room.is_empty() {
        format!("You: {you_profile} | {you_latency}")
    } else {
        format!("ROOM ID: {room}\nYou: {you_profile} | {you_latency}")
    };
    let marker = MARKER.len_utf8();
    if marker + body.len() > TEXT_CAP {
        body = cap_str(&body, TEXT_CAP.saturating_sub(marker)).to_owned();
        let mut out = String::new();
        out.push(MARKER);
        out.push_str(&body);
        return (out, 0);
    }
    let mut shown = 0u8;
    for remote in remotes.iter().take(usize::from(ARENA_REMOTE_CAP)) {
        let name = bound_bytes(&field(remote.name, "Unavailable"), ARENA_NAME_MAX);
        let profile = bound_bytes(&field(remote.profile, "---"), ARENA_PROFILE_MAX);
        let latency = bound_bytes(&field(remote.latency, "---"), ARENA_LATENCY_MAX);
        let ping = bound_bytes(&field(remote.ping, "---"), ARENA_PING_MAX);
        let line = format!("\n{name}: {profile} | {latency} ({ping})");
        if marker + body.len() + line.len() > TEXT_CAP {
            break;
        }
        body.push_str(&line);
        shown = shown.saturating_add(1);
    }
    let mut out = String::new();
    out.push(MARKER);
    out.push_str(&body);
    (out, shown)
}

pub fn is_our_text(text: &str) -> bool {
    text.starts_with(MARKER)
}

pub fn metrics_plausible(metrics: &FontMetrics) -> bool {
    [
        metrics.font_x,
        metrics.font_y,
        metrics.line_space,
        metrics.char_space,
    ]
    .iter()
    .all(|value| value.is_finite())
        && (1.0..=128.0).contains(&metrics.font_x)
        && (1.0..=160.0).contains(&metrics.font_y)
        && (-32.0..=32.0).contains(&metrics.line_space)
        && (-8.0..=16.0).contains(&metrics.char_space)
}

/// A remote banner line exists only for a mapped connected station.
pub fn show_native_remote(connected: bool) -> bool {
    connected
}

pub fn font_for(streamer: bool, captured: Option<FontMetrics>) -> Option<FontMetrics> {
    let captured = captured.filter(metrics_plausible)?;
    if streamer {
        Some(captured)
    } else {
        Some(TELEMETRY_FONT)
    }
}

pub fn material_for(streamer: bool, captured: Option<[[u8; 4]; 2]>) -> Option<[[u8; 4]; 2]> {
    let captured = captured?;
    if streamer {
        Some(captured)
    } else {
        Some(TELEMETRY_MATERIAL)
    }
}

pub fn colors_for(streamer: bool, captured: [[u8; 4]; 2], ping: Option<[u8; 4]>) -> [[u8; 4]; 2] {
    if streamer {
        return captured;
    }
    match ping {
        Some(color) => [color, color],
        None => captured,
    }
}

pub fn ping_bytes(color: PingRgba) -> [u8; 4] {
    [
        channel(color.r),
        channel(color.g),
        channel(color.b),
        channel(color.a),
    ]
}

fn channel(value: f32) -> u8 {
    if !value.is_finite() {
        return 0;
    }
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Game text on this identity. A later text reset must not replace the first style.
pub fn observe_style(captured: bool, same_identity: bool, text: &str) -> Observe {
    if text.is_empty() || is_our_text(text) {
        Observe::Keep
    } else if captured && same_identity {
        Observe::RefreshText
    } else {
        Observe::Replace
    }
}

pub fn telemetry_text(parts: &LineParts<'_>) -> String {
    let profile = field(parts.you_profile, "---");
    let latency = field(parts.you_latency, "---");
    let you = format!("You: {profile} | {latency}");
    let body = if parts.show_remote {
        let name = field(parts.remote_name, "Unavailable");
        let remote_profile = field(parts.remote_profile, "---");
        let remote_latency = field(parts.remote_latency, "---");
        let ping = field(parts.ping, "---");
        format!("{you}\n{name}: {remote_profile} | {remote_latency} ({ping})")
    } else {
        you
    };
    let mut out = String::new();
    out.push(MARKER);
    let room = TEXT_CAP.saturating_sub(out.len());
    out.push_str(cap_str(&body, room));
    out
}

pub fn with_room_id(room_id: &str, telemetry: &str) -> String {
    let id = field(room_id, "");
    if id.is_empty() || !is_our_text(telemetry) {
        return telemetry.to_owned();
    }
    let rest = telemetry.trim_start_matches(MARKER);
    let mut out = String::new();
    out.push(MARKER);
    let body = format!("ROOM ID: {id}\n{rest}");
    let room = TEXT_CAP.saturating_sub(out.len());
    out.push_str(cap_str(&body, room));
    out
}

pub fn banner_paint(input: &PaintInput<'_>) -> PaintOutcome {
    let leave = |applied| PaintOutcome {
        paint: BannerPaint::Leave,
        telemetry_applied: applied,
    };
    if !input.captured
        || !input.same_identity
        || input.saved_text.is_empty()
        || is_our_text(input.saved_text)
    {
        return leave(input.telemetry_applied);
    }
    if input.streamer {
        let pending =
            input.telemetry_applied || is_our_text(input.current) || !style_matches(input);
        if !pending {
            return leave(false);
        }
        return PaintOutcome {
            paint: BannerPaint::Write {
                text: input.saved_text.to_owned(),
                font: font_choice(input),
                colors: colors_for(true, input.saved_colors, None),
                material: material_for(true, input.saved_material),
                pos_y: arena_y(input),
            },
            telemetry_applied: false,
        };
    }
    if !is_our_text(input.telemetry) || input.telemetry.len() > TEXT_CAP {
        return leave(input.telemetry_applied);
    }
    PaintOutcome {
        paint: BannerPaint::Write {
            text: input.telemetry.to_owned(),
            font: font_choice(input),
            colors: colors_for(false, input.saved_colors, input.ping),
            material: material_for(false, input.saved_material),
            pos_y: arena_y(input),
        },
        telemetry_applied: true,
    }
}

fn font_choice(input: &PaintInput<'_>) -> Option<FontMetrics> {
    if input.arena {
        input.saved_font.filter(metrics_plausible)
    } else {
        font_for(input.streamer, input.saved_font)
    }
}

fn arena_y(input: &PaintInput<'_>) -> Option<f32> {
    if !input.arena {
        return None;
    }
    if input.streamer {
        return input.saved_pos_y.filter(|value| value.is_finite());
    }
    Some(arena_pos_y(input.remote_lines))
}

fn style_matches(input: &PaintInput<'_>) -> bool {
    let font_ok = match (input.saved_font.filter(metrics_plausible), input.live_font) {
        (Some(saved), Some(live)) => saved == live,
        (None, _) => true,
        (Some(_), None) => false,
    };
    font_ok && input.material_matches
}

pub fn decode_name(bytes: &[u8]) -> Option<&str> {
    let end = bytes.iter().position(|byte| *byte == 0)?;
    if end == 0 || end > 24 {
        return None;
    }
    let name = &bytes[..end];
    if name
        .iter()
        .any(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
    {
        return None;
    }
    std::str::from_utf8(name).ok()
}

pub fn decode_utf8_capped(bytes: &[u8]) -> Option<String> {
    if bytes.len() > TEXT_CAP + 1 {
        return None;
    }
    let end = bytes.iter().position(|byte| *byte == 0)?;
    if end > TEXT_CAP {
        return None;
    }
    std::str::from_utf8(&bytes[..end]).ok().map(str::to_owned)
}

pub fn decode_utf16_capped(units: &[u16]) -> Option<String> {
    if units.len() > TEXT_CAP + 1 {
        return None;
    }
    let end = units.iter().position(|unit| *unit == 0)?;
    if end > TEXT_CAP {
        return None;
    }
    String::from_utf16(&units[..end]).ok()
}

/// Arena room id is at most five UTF-16 units. Invalid sequences are absent.
pub fn decode_room_id(units: &[u16]) -> Option<String> {
    if units.is_empty() || units.len() > ROOM_ID_UNITS {
        return None;
    }
    let end = units
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(units.len());
    if end == 0 {
        return None;
    }
    let text = String::from_utf16(&units[..end]).ok()?;
    if text.is_empty() || text.chars().any(|ch| ch.is_control()) {
        return None;
    }
    Some(text)
}

pub fn walk_pane(nodes: &[GraphNode], start: usize, steps: &[PaneLink]) -> Option<usize> {
    let mut current = start;
    for step in steps {
        let node = nodes.get(current)?;
        let next = match step {
            PaneLink::Parent => node.parent?,
            PaneLink::Prev => node.prev?,
        };
        if next == start || next == current || nodes.get(next).is_none() {
            return None;
        }
        current = next;
    }
    Some(current)
}

/// The arena room-id pane named by the 13.0.5 init site. No prefix match.
pub fn is_arena_room_name(name: &str) -> bool {
    name == "set_txt_roomid"
}

/// Skyline far hook. The 16-byte trampoline ends at `+0x10`.
pub const SETTER_HOOK: [u32; 2] = [0x5800_0051, 0xd61f_0220];
/// Untouched 13.0.5 setter words at `+0x10`, `+0x14`, and `+0x18`.
pub const SETTER_TAIL: [u32; 3] = [0xd140_07ff, 0xaa01_03f4, 0xaa00_03f3];
/// 13.0.5 main `DATA` size. UI vtables sit inside this extent.
pub const MAIN_DATA_BYTES: u64 = 0x0039_2570;

pub fn main_data_end(data: u64) -> Option<u64> {
    data.checked_add(MAIN_DATA_BYTES)
}

/// `ldr x17; br x17`, and the original setter still present after that trampoline.
pub fn chained_text_setter(entry: &[u32; 2], tail: &[u32; 3]) -> bool {
    entry == &SETTER_HOOK && tail == &SETTER_TAIL
}

/// Readable vtable in main Rodata through `Data + 0x392570`, first slot in text.
pub fn main_image_vtable(vtable: u64, entry: u64, text: u64, rodata: u64, data_end: u64) -> bool {
    rodata > text
        && data_end > rodata
        && vtable.is_multiple_of(8)
        && vtable >= rodata
        && vtable < data_end
        && entry.is_multiple_of(4)
        && entry >= text
        && entry < rodata
}

/// The 13.0.5 UTF-8 setter only decodes 1–3 byte sequences. A 4-byte scalar
/// makes it read past the terminating NUL, so those scalars are removed first.
pub fn setter_utf8(text: &str) -> Option<String> {
    if text.is_empty() || text.len() > TEXT_CAP || text.bytes().any(|byte| byte == 0) {
        return None;
    }
    let cleaned: String = text.chars().filter(|ch| ch.len_utf8() < 4).collect();
    if cleaned.is_empty() || cleaned.len() > TEXT_CAP {
        None
    } else {
        Some(cleaned)
    }
}

pub fn text_fits(is_utf8: bool, cap_units: usize, text: &str) -> bool {
    if text.is_empty() || cap_units == 0 || cap_units > PANE_SCAN_MAX || text.len() > TEXT_CAP {
        return false;
    }
    if is_utf8 {
        text.len() < cap_units
    } else {
        let units = text.encode_utf16().count();
        units > 0 && units <= TEXT_CAP && units < cap_units
    }
}

pub fn name_words(name: &str) -> (u64, u64) {
    let mut words = [0u64; 2];
    for (index, byte) in name.bytes().take(16).enumerate() {
        words[index / 8] |= u64::from(byte) << ((index % 8) * 8);
    }
    (words[0], words[1])
}

fn field(value: &str, fallback: &str) -> String {
    let cleaned: String = value
        .chars()
        .filter(|ch| !ch.is_control() && *ch != MARKER)
        .collect();
    if cleaned.is_empty() {
        fallback.to_owned()
    } else {
        cleaned
    }
}

fn bound_bytes(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_owned();
    }
    let mut end = max_bytes;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_owned()
}

fn cap_str(text: &str, max_bytes: usize) -> &str {
    if text.len() <= max_bytes {
        return text;
    }
    let mut end = max_bytes;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pia_rtt::{PingRgba, PING_STABLE};

    fn parts<'a>(profile: &'a str, latency: &'a str, remote: bool, name: &'a str) -> LineParts<'a> {
        LineParts {
            you_profile: profile,
            you_latency: latency,
            show_remote: remote,
            remote_name: name,
            remote_profile: "LessLag",
            remote_latency: "4",
            ping: "12ms",
        }
    }

    fn game_font() -> FontMetrics {
        FontMetrics {
            font_x: 41.0,
            font_y: 71.0,
            line_space: 0.0,
            char_space: 0.0,
            text_position: 8,
        }
    }

    fn paint(input: PaintInput<'_>) -> PaintOutcome {
        banner_paint(&input)
    }

    fn base_input<'a>(
        streamer: bool,
        current: &'a str,
        saved_font: Option<FontMetrics>,
        live_font: Option<FontMetrics>,
        telemetry: &'a str,
    ) -> PaintInput<'a> {
        PaintInput {
            same_identity: true,
            captured: true,
            telemetry_applied: true,
            streamer,
            arena: false,
            current,
            saved_text: "Online",
            saved_font,
            live_font,
            saved_colors: [[9, 9, 9, 9], [1, 2, 3, 4]],
            saved_material: Some([[8, 8, 8, 8], [7, 7, 7, 7]]),
            material_matches: false,
            telemetry,
            ping: Some([10, 20, 30, 40]),
            saved_pos_y: None,
            remote_lines: 0,
        }
    }

    #[test]
    fn local_latency_matches_the_window_auto_sentinel() {
        assert_eq!(local_latency_label(-1), "Auto");
        assert_eq!(local_latency_label(0), "0");
        assert_eq!(local_latency_label(4), "4");
    }

    #[test]
    fn offline_omits_the_remote_line_and_a_connection_keeps_masked_details() {
        assert!(!show_native_remote(false));
        assert!(show_native_remote(true));
        let offline = telemetry_text(&parts("Auto:Van", "Auto", false, "Unavailable"));
        assert!(offline.contains("You: Auto:Van | Auto"));
        assert!(!offline.contains("Unavailable"));
        assert_eq!(offline.matches('\n').count(), 0);
        let masked = telemetry_text(&LineParts {
            you_profile: "Auto:Van",
            you_latency: "Auto",
            show_remote: true,
            remote_name: "Unavailable",
            remote_profile: "---",
            remote_latency: "---",
            ping: "---",
        });
        assert!(masked.contains("Unavailable: --- | --- (---)"));
    }

    #[test]
    fn telemetry_keeps_marker_profile_peer_and_one_separator() {
        let text = telemetry_text(&parts("Auto:Van", "Auto", true, "Ann"));
        assert!(text.starts_with(MARKER));
        assert!(text.contains("You: Auto:Van | Auto"));
        assert!(text.contains("Ann: LessLag | 4 (12ms)"));
        assert_eq!(text.matches('\n').count(), 1);
        assert_eq!(text.chars().filter(|ch| *ch == MARKER).count(), 1);
        assert!(text.len() <= TEXT_CAP);
    }

    #[test]
    fn telemetry_strips_controls_and_caps_on_a_char_boundary() {
        let text = telemetry_text(&parts("Auto:Van", "Auto", true, "Ann\nBob\u{200B}"));
        assert_eq!(text.matches('\n').count(), 1);
        assert!(text.contains("AnnBob"));
        let long = "A".repeat(400);
        let capped = telemetry_text(&parts("Auto:Van", "Auto", true, &long));
        assert!(capped.len() <= TEXT_CAP);
        assert!(capped.is_char_boundary(capped.len()));
    }

    #[test]
    fn repeated_game_text_does_not_replace_the_first_style() {
        assert_eq!(observe_style(false, false, "Online"), Observe::Replace);
        assert_eq!(observe_style(true, true, "Online"), Observe::RefreshText);
        assert_eq!(
            observe_style(true, true, "\u{200B}You: Auto:Van | Auto"),
            Observe::Keep
        );
        let telemetry = telemetry_text(&parts("Auto:Van", "Auto", false, ""));
        let original = game_font();
        let shown = paint(base_input(
            false,
            "Online",
            Some(original),
            Some(TELEMETRY_FONT),
            &telemetry,
        ));
        match shown.paint {
            BannerPaint::Write { font, material, .. } => {
                assert_eq!(font, Some(TELEMETRY_FONT));
                assert_ne!(font, Some(original));
                assert_eq!(material, Some(TELEMETRY_MATERIAL));
            }
            BannerPaint::Leave => panic!("telemetry should write"),
        }
        assert!(shown.telemetry_applied);
        let again = paint(base_input(
            false,
            "Online",
            Some(original),
            Some(TELEMETRY_FONT),
            &telemetry,
        ));
        match again.paint {
            BannerPaint::Write { font, .. } => assert_eq!(font, Some(TELEMETRY_FONT)),
            BannerPaint::Leave => panic!("second pass should stay absolute"),
        }
    }

    #[test]
    fn same_name_new_pane_captures_again_and_stale_identity_does_not_restore() {
        assert_eq!(observe_style(true, false, "Online"), Observe::Replace);
        let telemetry = telemetry_text(&parts("Auto:Van", "Auto", false, ""));
        let mut stale = base_input(
            true,
            "Online",
            Some(game_font()),
            Some(TELEMETRY_FONT),
            &telemetry,
        );
        stale.same_identity = false;
        assert!(matches!(paint(stale).paint, BannerPaint::Leave));
    }

    #[test]
    fn streamer_restores_style_when_the_game_already_restored_the_text() {
        let telemetry = telemetry_text(&parts("Auto:Van", "Auto", false, ""));
        let original = game_font();
        let outcome = paint(base_input(
            true,
            "Online",
            Some(original),
            Some(TELEMETRY_FONT),
            &telemetry,
        ));
        match outcome.paint {
            BannerPaint::Write {
                text,
                font,
                colors,
                material,
                pos_y,
            } => {
                assert_eq!(text, "Online");
                assert!(!is_our_text(&text));
                assert_eq!(font, Some(original));
                assert_eq!(colors[0], [9, 9, 9, 9]);
                assert_eq!(material, Some([[8, 8, 8, 8], [7, 7, 7, 7]]));
                assert_eq!(pos_y, None);
            }
            BannerPaint::Leave => panic!("style should be restored"),
        }
        assert!(!outcome.telemetry_applied);
        let mut settled = base_input(true, "Online", Some(original), Some(original), &telemetry);
        settled.telemetry_applied = false;
        settled.material_matches = true;
        assert!(matches!(paint(settled).paint, BannerPaint::Leave));
    }

    #[test]
    fn missing_original_is_left_alone() {
        let telemetry = telemetry_text(&parts("Auto:Van", "Auto", false, ""));
        let mut input = base_input(
            false,
            "Online",
            Some(game_font()),
            Some(game_font()),
            &telemetry,
        );
        input.captured = false;
        assert!(matches!(paint(input).paint, BannerPaint::Leave));
    }

    #[test]
    fn decode_rejects_bad_utf8_utf16_and_unbounded_names() {
        assert_eq!(decode_name(b"txt_all_title_00\0"), Some("txt_all_title_00"));
        assert_eq!(decode_name(b"txt_all_title_00"), None);
        assert_eq!(decode_utf8_capped(&[0xff, 0]), None);
        assert_eq!(decode_utf16_capped(&[0xD800, 0]), None);
        assert_eq!(
            decode_room_id(&[b'A' as u16, b'1' as u16, 0, 0, 0]),
            Some("A1".to_owned())
        );
        assert_eq!(decode_room_id(&[0xD800, 0, 0, 0, 0]), None);
        assert_eq!(decode_room_id(&[b'A' as u16; 6]), None);
    }

    #[test]
    fn second_title_follows_the_fork_background_then_upward_prev() {
        let nodes = [
            GraphNode {
                parent: Some(1),
                prev: None,
            },
            GraphNode {
                parent: None,
                prev: Some(2),
            },
            GraphNode {
                parent: None,
                prev: Some(3),
            },
            GraphNode {
                parent: Some(4),
                prev: None,
            },
            GraphNode {
                parent: Some(5),
                prev: None,
            },
            GraphNode {
                parent: None,
                prev: Some(6),
            },
            GraphNode {
                parent: None,
                prev: None,
            },
        ];
        assert_eq!(
            walk_pane(&nodes, 0, &CSS_TITLE_WALK[..CSS_BACKGROUND_STEPS]),
            Some(3)
        );
        assert_eq!(walk_pane(&nodes, 0, &CSS_TITLE_WALK), Some(6));
        assert_eq!(walk_pane(&nodes[..1], 0, &CSS_TITLE_WALK), None);
        let mut looped = nodes;
        looped[0].parent = Some(0);
        assert_eq!(walk_pane(&looped, 0, &CSS_TITLE_WALK), None);
    }

    #[test]
    fn chained_setter_accepts_only_the_skyline_far_hook() {
        assert!(chained_text_setter(&SETTER_HOOK, &SETTER_TAIL));
        assert!(!chained_text_setter(
            &[0xf81d_0ffc, 0xa901_4ff4],
            &SETTER_TAIL
        ));
        assert!(!chained_text_setter(&SETTER_HOOK, &[0; 3]));
    }

    #[test]
    fn data_vtables_stay_valid_when_their_first_slot_is_text() {
        let text = 0u64;
        let rodata = 0x039c_9000;
        let data = 0x04f1_7000;
        let data_end = main_data_end(data).unwrap();
        let entry = 0x1000u64;
        assert_eq!(data_end, 0x052a_9570);
        assert!(0x0523_d9a8 >= data && 0x0524_1f38 < data_end);
        assert!(main_image_vtable(
            0x0523_d9a8,
            entry,
            text,
            rodata,
            data_end
        ));
        assert!(main_image_vtable(
            0x0524_1f38,
            entry,
            text,
            rodata,
            data_end
        ));
        assert!(!main_image_vtable(
            0x0523_d9a8,
            data,
            text,
            rodata,
            data_end
        ));
        assert!(!main_image_vtable(data_end, entry, text, rodata, data_end));
        assert!(main_data_end(u64::MAX).is_none());
    }

    #[test]
    fn arena_keeps_its_font_and_css_keeps_the_telemetry_font() {
        let arena_font = FontMetrics {
            font_x: 18.0,
            font_y: 36.0,
            line_space: 2.0,
            char_space: 0.5,
            text_position: 0,
        };
        let telemetry = telemetry_text(&parts("LessLag", "0", true, "Ann"));
        let css = paint(base_input(
            false,
            "Online",
            Some(arena_font),
            Some(arena_font),
            &telemetry,
        ));
        match css.paint {
            BannerPaint::Write { font, pos_y, .. } => {
                assert_eq!(font, Some(TELEMETRY_FONT));
                assert_eq!(pos_y, None);
            }
            BannerPaint::Leave => panic!("css should write"),
        }
        let mut arena = base_input(
            false,
            "Online",
            Some(arena_font),
            Some(TELEMETRY_FONT),
            &telemetry,
        );
        arena.arena = true;
        arena.saved_pos_y = Some(-40.0);
        arena.remote_lines = 0;
        match banner_paint(&arena).paint {
            BannerPaint::Write { font, pos_y, .. } => {
                assert_eq!(font, Some(arena_font));
                assert_eq!(pos_y, Some(arena_pos_y(0)));
                assert_eq!(pos_y, Some(-115.0));
            }
            BannerPaint::Leave => panic!("arena should write"),
        }
        arena.remote_lines = 1;
        match banner_paint(&arena).paint {
            BannerPaint::Write { font, pos_y, .. } => {
                assert_eq!(font, Some(arena_font));
                assert_eq!(pos_y, Some(-137.5));
            }
            BannerPaint::Leave => panic!("arena remote line should write"),
        }
        assert_eq!(arena_pos_y(2), -160.0);
        assert_eq!(arena_pos_y(9), arena_pos_y(ARENA_REMOTE_CAP));
        let mut streamer = base_input(
            true,
            "Online",
            Some(arena_font),
            Some(TELEMETRY_FONT),
            &telemetry,
        );
        streamer.arena = true;
        streamer.saved_pos_y = Some(-40.0);
        streamer.remote_lines = 1;
        match paint(streamer).paint {
            BannerPaint::Write {
                text, font, pos_y, ..
            } => {
                assert_eq!(text, "Online");
                assert_eq!(font, Some(arena_font));
                assert_eq!(pos_y, Some(-40.0));
            }
            BannerPaint::Leave => panic!("streamer should restore the arena pane"),
        }
    }

    #[test]
    fn arena_line_keeps_the_room_id_and_only_exact_names() {
        let body = telemetry_text(&parts("Auto:Van", "Auto", false, ""));
        let lined = with_room_id("AB12", &body);
        assert!(lined.starts_with(MARKER));
        assert!(lined.contains("ROOM ID: AB12\nYou: Auto:Van | Auto"));
        assert!(is_arena_room_name("set_txt_roomid"));
        assert!(!is_arena_room_name("set_txt_room_info"));
        assert!(!is_arena_room_name("set_txt_room_info_ng"));
        assert!(!is_arena_room_name("txt_room_id"));
        assert!(!is_arena_room_name("set_txt_title_00"));
        assert_eq!(setter_utf8("AB12"), Some("AB12".to_owned()));
        assert_eq!(setter_utf8("A\u{1F600}B"), Some("AB".to_owned()));
        assert_eq!(setter_utf8("\u{1F600}"), None);
        let (left, right) = name_words("txt_all_title_00");
        let (other_left, other_right) = name_words("txt_all_title_01");
        assert_eq!(left, other_left);
        assert_ne!(right, other_right);
    }

    #[test]
    fn text_fit_never_claims_past_the_buffer() {
        assert!(text_fits(true, 3, "hi"));
        assert!(!text_fits(true, 2, "hi"));
        assert!(!text_fits(true, 800, "A"));
    }

    #[test]
    fn stable_ping_color_follows_the_snapshot_channels() {
        assert_eq!(
            ping_bytes(PingRgba {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            }),
            [0, 255, 0, 255]
        );
        let stable = ping_bytes(PING_STABLE);
        assert!(stable[1] > stable[0]);
        assert_eq!(stable[0], stable[2]);
        assert_eq!(stable[3], 255);
    }

    #[test]
    fn arena_lists_each_peer_and_bounds_an_overlong_name() {
        let remotes = [
            RemoteLine {
                name: "Ann",
                profile: "LessLag",
                latency: "4",
                ping: "12ms",
            },
            RemoteLine {
                name: "Bob",
                profile: "LLUltra",
                latency: "---",
                ping: "---",
            },
        ];
        let (text, shown) = arena_status("AB12", "Auto:Van", "Auto", &remotes);
        assert_eq!(shown, 2);
        assert!(text.starts_with(MARKER));
        assert!(text.contains("ROOM ID: AB12\nYou: Auto:Van | Auto\nAnn: LessLag | 4 (12ms)\nBob: LLUltra | --- (---)"));
        assert!(!text.contains("Ann: LLUltra"));
        assert!(!text.contains("Bob: LessLag"));
        assert_eq!(arena_pos_y(shown), -160.0);
        let huge = "Z".repeat(200);
        let overflow = [
            remotes[0],
            RemoteLine {
                name: &huge,
                profile: "Vanilla",
                latency: "1",
                ping: "9ms",
            },
        ];
        let (capped, shown) = arena_status("AB12", "Auto:Van", "Auto", &overflow);
        assert_eq!(shown, 2);
        assert!(capped.contains("Ann: LessLag | 4 (12ms)"));
        assert!(capped.contains(&huge[..ARENA_NAME_MAX]));
        assert!(!capped.contains(&huge));
        assert!(capped.len() <= TEXT_CAP);
        let mut eight = Vec::new();
        for index in 0..8 {
            eight.push(RemoteLine {
                name: "Name",
                profile: "LLDoubles",
                latency: "Auto",
                ping: "12ms [Wifi]",
            });
            let _ = index;
        }
        let (full, shown) = arena_status("AB12", "Auto (unavailable)", "Auto", &eight);
        assert_eq!(shown, 8);
        assert_eq!(full.matches("LLDoubles").count(), 8);
        assert!(full.len() <= TEXT_CAP);
        assert_eq!(interface_suffix("Wi-Fi"), " [Wifi]");
        assert_eq!(interface_suffix("Ethernet"), " [Wired]");
        assert_eq!(interface_suffix("Unknown"), "");
        let (boosted, _) = arena_status("AB12", "Auto (unavailable)++", "0", &[]);
        assert!(boosted.contains("Auto (unavailable)++"));
        let (quiet, shown) = arena_status("AB12", "LessLag", "0", &[]);
        assert_eq!(shown, 0);
        assert!(!quiet.contains("Unavailable"));
        assert_eq!(arena_pos_y(shown), -115.0);
    }

    #[test]
    fn streamer_lobby_count_hides_the_buffer_and_stale_scans() {
        assert_eq!(store_scan_count(-3), 0);
        assert_eq!(store_scan_count(4), 4);
        assert_eq!(store_scan_count(400), 255);
        assert_eq!(visible_network_count(false, 7), 0);
        assert_eq!(visible_network_count(true, 7), 7);
        assert_eq!(local_wireless_text(false, "Auto", true, 7), "Auto");
        assert_eq!(local_wireless_text(true, "Auto", true, 7), "7");
        assert_eq!(local_wireless_text(true, "Auto", false, 7), "0");
        assert!(!local_wireless_text(false, "Auto", true, 7).contains('7'));
        assert!(!local_wireless_text(true, "Auto", true, 7).contains("Auto"));
        assert_eq!(displayed_network_count(LDN_INITIALIZED, 7), 7);
        assert_eq!(displayed_network_count(0, 7), 0);
        assert_eq!(displayed_network_count(3, 7), 0);
    }

    #[test]
    fn local_count_requires_the_probed_pane_and_live_symbols() {
        assert!(local_update_accepted(LOCAL_UPDATE_WORDS));
        assert!(!local_update_accepted([
            0x3600_00a0,
            0xf940_c268,
            0xb942_5517,
            0
        ]));
        assert_eq!(LOCAL_SCENE_WORD, 0x946e_9728);
        assert!(local_pane_identity(
            "set_text_dev_count",
            0x8566_6320,
            0x8042_5000
        ));
        assert!(!local_pane_identity(
            "set_txt_roomid",
            0x8566_6320,
            0x8042_5000
        ));
        assert!(!local_pane_identity(
            "set_text_dev_count",
            0x8566_6328,
            0x8042_5000
        ));
        assert!(!local_pane_identity("set_text_dev_count", 0x8566_6320, 0));
        assert_eq!(LDN_INITIALIZE[0], b'_');
        assert_eq!(LDN_GET_STATE[0], b'_');
        assert_eq!(LDN_SCAN[0], b'_');
        assert!(!LDN_INITIALIZE.contains(&1));
        assert!(LDN_INITIALIZE.starts_with(b"_ZN2nn3ldn10InitializeEv"));
        assert!(LDN_SCAN.starts_with(b"_ZN2nn3ldn4ScanE"));
        assert!(ldn_symbols_usable(0x1000, 0x2004, 0x3008));
        assert!(!ldn_symbols_usable(0, 0x2004, 0x3008));
        assert!(!ldn_symbols_usable(0x1001, 0x2004, 0x3008));
        assert!(!ldn_symbols_usable(0x1000, 0x1000, 0x3008));
        assert!(local_count_active(true, 4, 4, true));
        assert!(!local_count_active(false, 4, 4, true));
        assert!(!local_count_active(true, 5, 4, true));
        assert!(!local_count_active(true, 4, 0, true));
        assert!(!local_count_active(true, 4, 4, false));
    }
}
