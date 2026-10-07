//! Display text for the CSS and arena titles. Pointer writes stay on Switch.

use crate::pia_rtt::PingRgba;

pub const TEXT_CAP: usize = 120;
pub const MARKER: char = '\u{200B}';
pub const ROOM_ID_UNITS: usize = 5;

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

/// Fork `update_online_arena_ui` Y for the lines this build can show.
/// `displayed_remote_count` is the selected peer line: 0 or 1.
pub fn arena_pos_y(displayed_remote_count: u8) -> f32 {
    let lines = f32::from(u8::from(displayed_remote_count > 0));
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
    /// Displayed selected-peer line. This build has 0 or 1.
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

pub fn is_our_text(text: &str) -> bool {
    text.starts_with(MARKER)
}

pub fn metrics_plausible(metrics: &FontMetrics) -> bool {
    [metrics.font_x, metrics.font_y, metrics.line_space, metrics.char_space]
        .iter()
        .all(|value| value.is_finite())
        && (1.0..=128.0).contains(&metrics.font_x)
        && (1.0..=160.0).contains(&metrics.font_y)
        && (-32.0..=32.0).contains(&metrics.line_space)
        && (-8.0..=16.0).contains(&metrics.char_space)
}

/// A remote line exists only for a known peer or a non-offline connection.
pub fn show_native_remote(peer_known: bool, connection_online: bool) -> bool {
    peer_known || connection_online
}

pub fn font_for(streamer: bool, captured: Option<FontMetrics>) -> Option<FontMetrics> {
    let captured = captured.filter(metrics_plausible)?;
    if streamer {
        Some(captured)
    } else {
        Some(TELEMETRY_FONT)
    }
}

pub fn material_for(
    streamer: bool,
    captured: Option<[[u8; 4]; 2]>,
) -> Option<[[u8; 4]; 2]> {
    let captured = captured?;
    if streamer {
        Some(captured)
    } else {
        Some(TELEMETRY_MATERIAL)
    }
}

pub fn colors_for(
    streamer: bool,
    captured: [[u8; 4]; 2],
    ping: Option<[u8; 4]>,
) -> [[u8; 4]; 2] {
    if streamer {
        return captured;
    }
    match ping {
        Some(color) => [color, color],
        None => captured,
    }
}

pub fn ping_bytes(color: PingRgba) -> [u8; 4] {
    [channel(color.r), channel(color.g), channel(color.b), channel(color.a)]
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
    if !input.captured || !input.same_identity || input.saved_text.is_empty() || is_our_text(input.saved_text)
    {
        return leave(input.telemetry_applied);
    }
    if input.streamer {
        let pending = input.telemetry_applied
            || is_our_text(input.current)
            || !style_matches(input);
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
    let end = units.iter().position(|unit| *unit == 0).unwrap_or(units.len());
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
pub fn main_image_vtable(
    vtable: u64,
    entry: u64,
    text: u64,
    rodata: u64,
    data_end: u64,
) -> bool {
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
    if text.is_empty() || cap_units == 0 || cap_units > 512 || text.len() > TEXT_CAP {
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
        assert!(!show_native_remote(false, false));
        assert!(show_native_remote(true, false));
        assert!(show_native_remote(false, true));
        let offline = telemetry_text(&parts("Auto:Van", "Auto", false, "Unavailable"));
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
        let mut stale = base_input(true, "Online", Some(game_font()), Some(TELEMETRY_FONT), &telemetry);
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
        let mut input = base_input(false, "Online", Some(game_font()), Some(game_font()), &telemetry);
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
            GraphNode { parent: Some(1), prev: None },
            GraphNode { parent: None, prev: Some(2) },
            GraphNode { parent: None, prev: Some(3) },
            GraphNode { parent: Some(4), prev: None },
            GraphNode { parent: Some(5), prev: None },
            GraphNode { parent: None, prev: Some(6) },
            GraphNode { parent: None, prev: None },
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
        assert!(!chained_text_setter(&[0xf81d_0ffc, 0xa901_4ff4], &SETTER_TAIL));
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
        assert!(main_image_vtable(0x0523_d9a8, entry, text, rodata, data_end));
        assert!(main_image_vtable(0x0524_1f38, entry, text, rodata, data_end));
        assert!(!main_image_vtable(0x0523_d9a8, data, text, rodata, data_end));
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
        assert_eq!(arena_pos_y(2), arena_pos_y(1));
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
}
