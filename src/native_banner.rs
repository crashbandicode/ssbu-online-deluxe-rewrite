//! Display text and style decisions for the CSS, arena and local-wireless
//! panes. Pointer reads and writes stay in `runtime/native_banner.rs`.

use crate::pia_rtt::PingRgba;
use PaneLink::{Parent, Prev};

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
/// `Auto (25f)`: Auto with the frame count the game chose (0.6.2).
pub const YOU_LATENCY_MAX: usize = 10;
/// `You: ` + profile + ` | ` + latency.
pub const YOU_LINE_MAX: usize = 5 + YOU_PROFILE_MAX + 3 + YOU_LATENCY_MAX;
pub const REMOTE_LINE_MAX: usize =
    1 + ARENA_NAME_MAX + 2 + ARENA_PROFILE_MAX + 3 + ARENA_LATENCY_MAX + 2 + ARENA_PING_MAX + 1;

/// Marker, room line, You line, and eight remote lines at the bounds above.
pub const TEXT_CAP: usize = MARKER_BYTES + ROOM_LINE_MAX + YOU_LINE_MAX + 8 * REMOTE_LINE_MAX;
/// One past the text the setter is given. Longer declared buffers are corrupt.
pub const PANE_SCAN_MAX: usize = TEXT_CAP + 1;

const _: () = assert!(TEXT_CAP == 698);
const _: () = assert!(PANE_SCAN_MAX <= 1024);

/// Pane text as UTF-8: [`TEXT_CAP`] UTF-16 units decode to at most three bytes each.
pub type PaneText = crate::fixed::FixedStr<{ 3 * TEXT_CAP }>;
/// Our status text, marker included.
pub type StatusText = crate::fixed::FixedStr<TEXT_CAP>;
/// The text handed to the setter, NUL included.
pub type SetterText = crate::fixed::FixedStr<{ TEXT_CAP + 1 }>;
/// A pane name ([`decode_name`]: at most 24 bytes).
pub type PaneName = crate::fixed::FixedStr<24>;
/// An arena room id: at most [`ROOM_ID_UNITS`] units.
pub type RoomId = crate::fixed::FixedStr<{ 3 * ROOM_ID_UNITS }>;
/// One field of a remote line, before [`status_text`] cleans and cuts it.
pub type Field = crate::fixed::FixedStr<64>;
/// A latency label (`Auto (4f)`, `-1`..`9`).
pub type Latency = crate::fixed::FixedStr<16>;
/// A remote line's profile or ping before [`status_text`] cuts it (both are
/// cut to fewer bytes than this).
pub type ShortField = crate::fixed::FixedStr<32>;
/// One remote line of [`status_text`]: newline, name, profile, latency, ping.
type RemoteText = crate::fixed::FixedStr<{ REMOTE_LINE_MAX + 8 }>;

/// Local-wireless update site. Words are the clean 13.0.5 entry from the pane probe.
pub const LOCAL_UPDATE: usize = 0x1bd7a80;
pub const LOCAL_UPDATE_WORDS: [u32; 4] = [0x3600_00a0, 0xf940_c268, 0xb942_5517, 0x3700_08f6];
/// Existing scene entry. Its hook is already installed; this word is checked before that patch.
pub const LOCAL_SCENE: usize = 0x1bd45e0;
pub const LOCAL_SCENE_WORD: u32 = 0x946e_9728;
pub const LOCAL_PANE_NAME: &str = "set_text_dev_count";
/// `0x85666320 - 0x80425000` from the same pane at the scene entry and the update site.
pub const LOCAL_PANE_VTABLE: u64 = 0x0052_41320;
/// The arena room-id pane named by the 13.0.5 init site. No prefix match.
pub const ARENA_ROOM_NAME: &str = "set_txt_roomid";

/// Name and vtable from the probed pane. The vtable is an offset from main text.
pub fn local_pane_identity(name: &str, vtable: u64, text_base: u64) -> bool {
    text_base != 0 && name == LOCAL_PANE_NAME && vtable == text_base.wrapping_add(LOCAL_PANE_VTABLE)
}

/// A stale scene generation leaves the game text alone.
pub fn local_count_active(scene_generation: u64, captured_generation: u64) -> bool {
    captured_generation != 0 && scene_generation == captured_generation
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
pub const CSS_TITLE_WALK: [PaneLink; 6] = [Parent, Prev, Prev, Parent, Parent, Prev];

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

/// Which pane a write is for. CSS titles take the fork telemetry font, the
/// arena keeps its own font and moves with the line count, the local-wireless
/// lobby pane keeps its own font and material and only changes text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneKind {
    Css,
    Arena,
    Local,
}

/// Font, colours, material and Y of one pane. `None` leaves that part alone.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style {
    pub font: Option<FontMetrics>,
    pub colors: [[u8; 4]; 2],
    pub material: Option<[[u8; 4]; 2]>,
    /// Arena only: the fork Y for telemetry, the original Y for Streamer.
    pub pos_y: Option<f32>,
}

/// Pane text with its style: the game's own, captured once per pane identity
/// and scene, or what [`paint`] decides to write.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Styled {
    pub text: PaneText,
    pub style: Style,
}

/// Complete remote lines an arena pane can show. Matches the handshake table.
pub const ARENA_REMOTE_CAP: u8 = 8;

/// Fork `update_online_arena_ui` Y. The count is complete lines actually written.
pub fn arena_pos_y(displayed_remote_count: u8) -> f32 {
    -115.0 - 22.5 * f32::from(displayed_remote_count.min(ARENA_REMOTE_CAP))
}

/// `Auto`, or `Auto (4f)` once the online-latency hook has seen the frame
/// count the game chose for this session (0.6.2); a manual buffer is its number.
pub fn local_latency_label(buffer: i8, auto_frames: Option<u8>) -> Latency {
    match (buffer, auto_frames) {
        (-1, Some(frames)) => crate::fixed!("Auto ({frames}f)"),
        (-1, None) => Latency::from("Auto"),
        _ => crate::fixed!("{buffer}"),
    }
}

/// Local-wireless lobby text: the input buffer behind the marker, so the
/// game's own count (which can be the same digit) is told apart from ours.
/// That pane is a digit wide, so Auto stays `Auto` here.
pub fn local_lobby_text(buffer: i8) -> crate::fixed::FixedStr<24> {
    crate::fixed!("{MARKER}{}", local_latency_label(buffer, None))
}

/// One remote line. Fields already belong to the same station.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemoteLine {
    pub name: Field,
    pub profile: ShortField,
    pub latency: Latency,
    pub ping: ShortField,
}

/// Marker, the room line when there is a room, the local line, then each
/// complete remote line that fits. A line that would exceed [`TEXT_CAP`] is
/// omitted whole. Returns the text and the number of remote lines written.
pub fn status_text(
    room: Option<&str>,
    you_profile: &str,
    you_latency: &str,
    remotes: &[RemoteLine],
) -> (StatusText, u8) {
    let profile = field(you_profile, "---", YOU_PROFILE_MAX);
    let latency = field(you_latency, "---", YOU_LATENCY_MAX);
    let room = field(room.unwrap_or(""), "", ROOM_LINE_MAX - 10);
    let mut body: StatusText = crate::fixed!("{MARKER}");
    if !room.is_empty() {
        let _ = std::fmt::Write::write_fmt(&mut body, format_args!("ROOM ID: {room}\n"));
    }
    let _ = std::fmt::Write::write_fmt(&mut body, format_args!("You: {profile} | {latency}"));
    let mut shown = 0u8;
    for remote in remotes.iter().take(usize::from(ARENA_REMOTE_CAP)) {
        let name = field(&remote.name, "Unavailable", ARENA_NAME_MAX);
        let profile = field(&remote.profile, "---", ARENA_PROFILE_MAX);
        let latency = field(&remote.latency, "---", ARENA_LATENCY_MAX);
        let ping = field(&remote.ping, "---", ARENA_PING_MAX);
        let line: RemoteText = crate::fixed!("\n{name}: {profile} | {latency} ({ping})");
        if body.len() + line.len() > TEXT_CAP {
            break;
        }
        body.push_str(&line);
        shown += 1;
    }
    (body, shown)
}

pub fn is_our_text(text: &str) -> bool {
    text.starts_with(MARKER)
}

pub fn metrics_plausible(metrics: &FontMetrics) -> bool {
    (1.0..=128.0).contains(&metrics.font_x)
        && (1.0..=160.0).contains(&metrics.font_y)
        && (-32.0..=32.0).contains(&metrics.line_space)
        && (-8.0..=16.0).contains(&metrics.char_space)
}

/// Channels as bytes. A NaN channel saturates to zero.
pub fn ping_bytes(color: PingRgba) -> [u8; 4] {
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    [color.r, color.g, color.b, color.a].map(channel)
}

/// What to write on a pane whose game text and style were captured.
///
/// Streamer (`telemetry` None) restores the captured text and style, once,
/// while the pane still shows ours (`ours`). Otherwise the telemetry is
/// written with the fork font and material on the CSS titles, the saved font
/// and the fork material on the arena, and no style change on the lobby pane;
/// the text colour is the ping colour when there is one.
pub fn paint<'a>(
    saved: &'a Styled,
    ours: bool,
    telemetry: Option<(&'a str, Option<[u8; 4]>)>,
    kind: PaneKind,
    remote_lines: u8,
) -> Option<Write<'a>> {
    paint_parts(&saved.text, &saved.style, ours, telemetry, kind, remote_lines)
}

/// [`paint`] with the captured text and style held apart.
pub fn paint_parts<'a>(
    saved_text: &'a str,
    saved_style: &Style,
    ours: bool,
    telemetry: Option<(&'a str, Option<[u8; 4]>)>,
    kind: PaneKind,
    remote_lines: u8,
) -> Option<Write<'a>> {
    if saved_text.is_empty() || is_our_text(saved_text) {
        return None;
    }
    let (text, ping) = match telemetry {
        None if ours => (saved_text, None),
        Some((text, ping)) if is_our_text(text) && text.len() <= TEXT_CAP => (text, ping),
        _ => return None,
    };
    let restore = telemetry.is_none();
    let arena = kind == PaneKind::Arena;
    let styled = kind != PaneKind::Local;
    let font = saved_style.font.filter(|_| styled);
    let material = saved_style.material.filter(|_| styled);
    let telemetry_y = (arena && !restore).then(|| arena_pos_y(remote_lines));
    let style = Style {
        font: font.map(|font| {
            if restore || arena {
                font
            } else {
                TELEMETRY_FONT
            }
        }),
        colors: ping.map_or(saved_style.colors, |color| [color, color]),
        material: material.map(|material| {
            if restore {
                material
            } else {
                TELEMETRY_MATERIAL
            }
        }),
        pos_y: saved_style
            .pos_y
            .filter(|_| arena && restore)
            .or(telemetry_y),
    };
    Some(Write { text, style })
}

/// What [`paint`] writes: the captured or the telemetry text, borrowed (a
/// pane text is too large to copy on a hook's stack), and the style.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Write<'a> {
    pub text: &'a str,
    pub style: Style,
}

/// NUL-terminated pane name of 1–24 ASCII word characters.
pub fn decode_name(bytes: &[u8]) -> Option<&str> {
    let end = bytes.iter().position(|byte| *byte == 0)?;
    let name = std::str::from_utf8(&bytes[..end]).ok()?;
    let word = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
    ((1..=24).contains(&end) && name.bytes().all(word)).then_some(name)
}

/// Terminator position of a buffer the setter could have written.
fn capped_end(len: usize, terminator: Option<usize>) -> Option<usize> {
    terminator.filter(|end| len <= PANE_SCAN_MAX && *end <= TEXT_CAP)
}

pub fn decode_utf8_capped(bytes: &[u8]) -> Option<PaneText> {
    let mut out = PaneText::new();
    decode_utf8_into(bytes, &mut out).then_some(out)
}

pub fn decode_utf16_capped(units: &[u16]) -> Option<PaneText> {
    let mut out = PaneText::new();
    decode_utf16_into(units, &mut out).then_some(out)
}

/// [`decode_utf8_capped`] into `out` (a hook's buffer, not its stack);
/// `false`, with `out` unspecified, where that returns `None`.
pub fn decode_utf8_into(bytes: &[u8], out: &mut PaneText) -> bool {
    let Some(end) = capped_end(bytes.len(), bytes.iter().position(|byte| *byte == 0)) else {
        return false;
    };
    let Ok(text) = std::str::from_utf8(&bytes[..end]) else {
        return false;
    };
    out.clear();
    out.push_str(text)
}

/// [`decode_utf16_capped`] into `out`, as [`decode_utf8_into`].
pub fn decode_utf16_into(units: &[u16], out: &mut PaneText) -> bool {
    let Some(end) = capped_end(units.len(), units.iter().position(|unit| *unit == 0)) else {
        return false;
    };
    utf16_into(&units[..end], out)
}

/// Valid UTF-16 as UTF-8 into `out`; `false` when invalid or too long.
fn utf16_into<const N: usize>(units: &[u16], out: &mut crate::fixed::FixedStr<N>) -> bool {
    out.clear();
    for ch in char::decode_utf16(units.iter().copied()) {
        match ch {
            Ok(ch) if out.push(ch) => {}
            _ => return false,
        }
    }
    true
}

/// Arena room id is at most five UTF-16 units. Invalid sequences are absent.
pub fn decode_room_id(units: &[u16]) -> Option<RoomId> {
    let units = Some(units).filter(|units| (1..=ROOM_ID_UNITS).contains(&units.len()))?;
    let end = units
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(units.len());
    let mut text = RoomId::new();
    if !utf16_into(&units[..end], &mut text) {
        return None;
    }
    (!text.is_empty() && !text.chars().any(char::is_control)).then_some(text)
}

/// Skyline far hook. The 16-byte trampoline ends at `+0x10`.
pub const SETTER_HOOK: [u32; 2] = [0x5800_0051, 0xd61f_0220];
/// Untouched 13.0.5 setter words at `+0x10`, `+0x14`, and `+0x18`.
pub const SETTER_TAIL: [u32; 3] = [0xd140_07ff, 0xaa01_03f4, 0xaa00_03f3];
/// 13.0.5 main `DATA` size. UI vtables sit inside this extent.
pub const MAIN_DATA_BYTES: u64 = 0x0039_2570;

/// `ldr x17; br x17`, and the original setter still present after that trampoline.
pub fn chained_text_setter(entry: &[u32; 2], tail: &[u32; 3]) -> bool {
    entry == &SETTER_HOOK && tail == &SETTER_TAIL
}

/// Readable vtable in main Rodata through `Data + 0x392570`, first slot in text.
pub fn main_image_vtable(vtable: u64, entry: u64, text: u64, rodata: u64, data_end: u64) -> bool {
    rodata > text
        && data_end > rodata
        && vtable.is_multiple_of(8)
        && (rodata..data_end).contains(&vtable)
        && entry.is_multiple_of(4)
        && (text..rodata).contains(&entry)
}

/// The 13.0.5 UTF-8 setter only decodes 1–3 byte sequences. A 4-byte scalar
/// makes it read past the terminating NUL, so those scalars are removed first.
pub fn setter_utf8(text: &str) -> Option<SetterText> {
    if text.is_empty() || text.len() > TEXT_CAP || text.bytes().any(|byte| byte == 0) {
        return None;
    }
    let mut cleaned = SetterText::new();
    for ch in text.chars().filter(|ch| ch.len_utf8() < 4) {
        cleaned.push(ch);
    }
    (!cleaned.is_empty()).then_some(cleaned)
}

pub fn name_words(name: &str) -> (u64, u64) {
    let mut words = [0u64; 2];
    for (index, byte) in name.bytes().take(16).enumerate() {
        words[index / 8] |= u64::from(byte) << ((index % 8) * 8);
    }
    (words[0], words[1])
}

/// Control characters and the marker removed, `fallback` when nothing is
/// left, then cut to `max_bytes` on a char boundary.
fn field(value: &str, fallback: &str, max_bytes: usize) -> Field {
    let clean = |ch: &char| !ch.is_control() && *ch != MARKER;
    let mut cleaned = Field::new();
    for ch in value.chars().filter(clean) {
        if cleaned.len() + ch.len_utf8() > max_bytes {
            break;
        }
        cleaned.push(ch);
    }
    if cleaned.is_empty() && !value.chars().any(|ch| clean(&ch)) {
        return Field::from(cap_str(fallback, max_bytes));
    }
    cleaned
}

fn cap_str(text: &str, max_bytes: usize) -> &str {
    let mut end = max_bytes.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pia_rtt::{ping_rgba, Stability};

    fn remote(name: &str, profile: &str, latency: &str, ping: &str) -> RemoteLine {
        RemoteLine {
            name: name.into(),
            profile: profile.into(),
            latency: latency.into(),
            ping: ping.into(),
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

    fn saved(text: &str, pos_y: Option<f32>) -> Styled {
        Styled {
            text: text.into(),
            style: Style {
                font: Some(game_font()),
                colors: [[9, 9, 9, 9], [1, 2, 3, 4]],
                material: Some([[8, 8, 8, 8], [7, 7, 7, 7]]),
                pos_y,
            },
        }
    }

    /// A telemetry write of `text` on `kind`, colour optional.
    fn write<'a>(
        saved: &'a Styled,
        text: &'a str,
        ping: Option<[u8; 4]>,
        kind: PaneKind,
        lines: u8,
    ) -> Write<'a> {
        paint(saved, false, Some((text, ping)), kind, lines).unwrap()
    }

    fn vtable_ok(vtable: u64, entry: u64, rodata: u64, data_end: u64) -> bool {
        main_image_vtable(vtable, entry, 0, rodata, data_end)
    }

    #[test]
    fn status_text_keeps_marker_room_you_and_remote_lines() {
        let (css, shown) = status_text(None, "Auto:Van", "Auto", &[]);
        assert_eq!(css, "\u{200B}You: Auto:Van | Auto");
        assert_eq!(shown, 0);
        let one = [remote("Ann", "LessLag", "4", "12ms")];
        let (css, shown) = status_text(None, "Auto:Van", "Auto", &one);
        assert_eq!(css, "\u{200B}You: Auto:Van | Auto\nAnn: LessLag | 4 (12ms)");
        assert_eq!(shown, 1);
        let masked = [remote("", "", "", "")];
        let (text, _) = status_text(None, "", "", &masked);
        assert_eq!(text, "\u{200B}You: --- | ---\nUnavailable: --- | --- (---)");
        let two = [one[0], remote("Bob", "LLUltra", "---", "---")];
        let (arena, shown) = status_text(Some("AB12"), "Auto:Van", "Auto", &two);
        assert_eq!(shown, 2);
        assert_eq!(
            arena,
            "\u{200B}ROOM ID: AB12\nYou: Auto:Van | Auto\nAnn: LessLag | 4 (12ms)\nBob: LLUltra | --- (---)"
        );
        assert_eq!(arena_pos_y(shown), -160.0);
        let (boosted, _) = status_text(Some("AB12"), "Auto (unavailable)++", "0", &[]);
        assert!(boosted.contains("You: Auto (unavailable)++ | 0"));
        assert_eq!(local_latency_label(-1, None), "Auto");
        assert_eq!(local_latency_label(-1, Some(4)), "Auto (4f)");
        assert_eq!(local_latency_label(-1, Some(25)), "Auto (25f)");
        assert_eq!(local_latency_label(4, Some(7)), "4");
        assert_eq!(local_lobby_text(-1), "\u{200B}Auto");
        let (auto_css, _) = status_text(None, "Auto:Van", "Auto (25f)", &[]);
        assert_eq!(auto_css, "\u{200B}You: Auto:Van | Auto (25f)");
        assert_eq!(YOU_LINE_MAX, 38);
        assert!(is_our_text(&local_lobby_text(0)));
    }

    #[test]
    fn status_text_bounds_fields_strips_controls_and_caps_at_eight_lines() {
        let huge = "Z".repeat(200);
        let lines = [
            remote("Ann\nBob\u{200B}", "LessLag", "4", "12ms"),
            remote(&huge, "Vanilla", "1", "9ms"),
        ];
        let (text, shown) = status_text(Some("AB12"), "Auto:Van", "Auto", &lines);
        assert_eq!(shown, 2);
        assert!(text.contains("\nAnnBob: LessLag"));
        assert!(text.contains(&huge[..ARENA_NAME_MAX]));
        assert!(!text.contains(&huge[..ARENA_NAME_MAX + 1]));
        assert_eq!(text.chars().filter(|ch| *ch == MARKER).count(), 1);
        assert!(text.len() <= TEXT_CAP);
        let eight = vec![remote("Name", "LLDoubles", "Auto", "12ms [Wifi]"); 9];
        let (full, shown) = status_text(Some("AB12"), "Auto (unavailable)", "Auto", &eight);
        assert_eq!(shown, 8);
        assert_eq!(full.matches("LLDoubles").count(), 8);
        assert!(full.len() <= TEXT_CAP);
        let long_room = "R".repeat(40);
        let (capped, _) = status_text(Some(&long_room), "P", "L", &[]);
        assert!(capped.contains(&long_room[..ROOM_LINE_MAX - 10]));
        assert!(!capped.contains(&long_room[..ROOM_LINE_MAX - 9]));
        assert_eq!(interface_suffix("Wi-Fi"), " [Wifi]");
        assert_eq!(interface_suffix("Ethernet"), " [Wired]");
        assert_eq!(interface_suffix("Unknown"), "");
    }

    #[test]
    fn unicode_names_reach_the_banner_text_and_the_setter_unchanged() {
        let lines = [remote("Æsir テスト日本", "LessLag", "4", "12ms")];
        let (text, shown) = status_text(None, "Auto:Van", "Auto", &lines);
        assert_eq!(shown, 1);
        assert!(text.ends_with("\nÆsir テスト日本: LessLag | 4 (12ms)"));
        assert_eq!(setter_utf8(&text).as_deref(), Some(text.as_str()));
        // A name over the 40-byte field is cut on a character boundary.
        let long = "テ".repeat(14);
        let (cut, _) = status_text(None, "P", "L", &[remote(&long, "", "", "")]);
        assert!(cut.contains(&format!("\n{}: ", "テ".repeat(13))));
    }

    #[test]
    fn telemetry_writes_css_font_and_arena_keeps_its_font_and_moves() {
        let original = saved("Online", Some(-40.0));
        let (telemetry, _) = status_text(None, "LessLag", "0", &[]);
        let ping = Some([10, 20, 30, 40]);
        let css = write(&original, &telemetry, ping, PaneKind::Css, 0);
        assert_eq!(css.text, telemetry);
        assert_eq!(css.style.font, Some(TELEMETRY_FONT));
        assert_eq!(css.style.colors, [[10, 20, 30, 40]; 2]);
        assert_eq!(css.style.material, Some(TELEMETRY_MATERIAL));
        assert_eq!(css.style.pos_y, None);
        let uncolored = write(&original, &telemetry, None, PaneKind::Css, 0);
        assert_eq!(uncolored.style.colors, original.style.colors);
        let arena = write(&original, &telemetry, ping, PaneKind::Arena, 0);
        assert_eq!(arena.style.font, Some(game_font()));
        assert_eq!(arena.style.material, Some(TELEMETRY_MATERIAL));
        assert_eq!(arena.style.pos_y, Some(-115.0));
        let lined = write(&original, &telemetry, ping, PaneKind::Arena, 1);
        assert_eq!(lined.style.pos_y, Some(-137.5));
        assert_eq!(arena_pos_y(9), arena_pos_y(ARENA_REMOTE_CAP));
        let local = write(&original, &telemetry, None, PaneKind::Local, 0);
        assert_eq!((local.style.font, local.style.material), (None, None));
        assert_eq!(local.style.colors, original.style.colors);
        assert_eq!(local.style.pos_y, None);
        let mut no_font = original;
        no_font.style.font = None;
        assert_eq!(
            write(&no_font, &telemetry, None, PaneKind::Css, 0)
                .style
                .font,
            None
        );
    }

    #[test]
    fn streamer_restores_once_while_ours_shows_and_bad_text_leaves_the_pane() {
        let original = saved("Online", Some(-40.0));
        let restored = paint(&original, true, None, PaneKind::Arena, 3).unwrap();
        let expected = saved("Online", Some(-40.0));
        assert_eq!((restored.text, restored.style), (expected.text.as_str(), expected.style));
        let css = paint(&original, true, None, PaneKind::Css, 0).unwrap();
        assert_eq!(css.style.pos_y, None);
        assert_eq!(css.style.font, Some(game_font()));
        let local = paint(&original, true, None, PaneKind::Local, 0).unwrap();
        assert_eq!((local.style.font, local.style.material), (None, None));
        assert_eq!(paint(&original, false, None, PaneKind::Css, 0), None);
        let (telemetry, _) = status_text(None, "LessLag", "0", &[]);
        assert_eq!(
            paint(&saved(&telemetry, None), true, None, PaneKind::Css, 0),
            None
        );
        assert_eq!(paint(&saved("", None), true, None, PaneKind::Css, 0), None);
        let game = Some(("Online", None));
        assert_eq!(paint(&original, true, game, PaneKind::Css, 0), None);
        let over = format!("{MARKER}{}", "A".repeat(TEXT_CAP));
        let long = Some((over.as_str(), None));
        assert_eq!(paint(&original, true, long, PaneKind::Css, 0), None);
    }

    #[test]
    fn decoders_reject_bad_utf8_utf16_overlong_and_unbounded_input() {
        assert_eq!(decode_name(b"txt_all_title_00\0"), Some("txt_all_title_00"));
        assert_eq!(decode_name(b"txt_all_title_00"), None);
        assert_eq!(decode_name(b"\0"), None);
        assert_eq!(decode_name(b"bad name\0"), None);
        let mut long_name = [b'a'; 26];
        long_name[25] = 0;
        assert_eq!(decode_name(&long_name), None);
        assert_eq!(decode_utf8_capped(&[0xff, 0]), None);
        assert_eq!(decode_utf8_capped(b"Online\0xx").as_deref(), Some("Online"));
        assert_eq!(decode_utf8_capped(&[b'A'; PANE_SCAN_MAX + 1]), None);
        let mut long = vec![b'A'; TEXT_CAP + 1];
        long.push(0);
        assert_eq!(decode_utf8_capped(&long), None);
        assert_eq!(decode_utf16_capped(&[0xD800, 0]), None);
        let ok = [b'O' as u16, b'k' as u16, 0];
        assert_eq!(decode_utf16_capped(&ok).as_deref(), Some("Ok"));
        let room = [b'A' as u16, b'1' as u16, 0, 0, 0];
        assert_eq!(decode_room_id(&room).as_deref(), Some("A1"));
        assert_eq!(decode_room_id(&[0xD800, 0, 0, 0, 0]), None);
        assert_eq!(decode_room_id(&[b'A' as u16; 6]), None);
        assert_eq!(decode_room_id(&[0; 5]), None);
        assert_eq!(decode_room_id(&[]), None);
    }

    #[test]
    fn setter_text_drops_four_byte_scalars_and_rejects_nul() {
        assert_eq!(setter_utf8("AB12").as_deref(), Some("AB12"));
        assert_eq!(setter_utf8("A\u{1F600}B").as_deref(), Some("AB"));
        assert_eq!(setter_utf8("\u{1F600}"), None);
        assert_eq!(setter_utf8(""), None);
        assert_eq!(setter_utf8("A\0B"), None);
        assert_eq!(setter_utf8(&"A".repeat(TEXT_CAP + 1)), None);
        let (left, right) = name_words("txt_all_title_00");
        let (other_left, other_right) = name_words("txt_all_title_01");
        assert_eq!(left, other_left);
        assert_ne!(right, other_right);
    }

    #[test]
    fn metrics_and_ping_colours_stay_in_range() {
        assert!(metrics_plausible(&game_font()));
        assert!(metrics_plausible(&TELEMETRY_FONT));
        let bad = |font_x, font_y, line_space| FontMetrics {
            font_x,
            font_y,
            line_space,
            ..game_font()
        };
        assert!(!metrics_plausible(&bad(f32::NAN, 71.0, 0.0)));
        assert!(!metrics_plausible(&bad(41.0, 0.5, 0.0)));
        assert!(!metrics_plausible(&bad(41.0, 71.0, 40.0)));
        let odd = PingRgba {
            r: 0.0,
            g: 1.0,
            b: f32::NAN,
            a: 2.0,
        };
        assert_eq!(ping_bytes(odd), [0, 255, 0, 255]);
        let stable = ping_bytes(ping_rgba(Stability::Stable));
        assert!(stable[1] > stable[0]);
        assert_eq!(stable[0], stable[2]);
    }

    #[test]
    fn chained_setter_and_main_image_vtables_follow_the_image_layout() {
        assert!(chained_text_setter(&SETTER_HOOK, &SETTER_TAIL));
        let other_entry = [0xf81d_0ffc, 0xa901_4ff4];
        assert!(!chained_text_setter(&other_entry, &SETTER_TAIL));
        assert!(!chained_text_setter(&SETTER_HOOK, &[0; 3]));
        let (rodata, data) = (0x039c_9000, 0x04f1_7000);
        let data_end = data + MAIN_DATA_BYTES;
        assert_eq!(data_end, 0x052a_9570);
        assert!(vtable_ok(0x0523_d9a8, 0x1000, rodata, data_end));
        assert!(vtable_ok(0x0524_1f38, 0x1000, rodata, data_end));
        assert!(!vtable_ok(0x0523_d9a8, data, rodata, data_end));
        assert!(!vtable_ok(0x0523_d9ac, 0x1000, rodata, data_end));
        assert!(!vtable_ok(data_end, 0x1000, rodata, data_end));
        assert!(!vtable_ok(0x0523_d9a8, 0x1000, 0, data_end));
    }

    #[test]
    fn local_pane_needs_the_probed_identity_and_a_live_scene() {
        assert_eq!(LOCAL_SCENE_WORD, 0x946e_9728);
        assert_eq!(LOCAL_UPDATE_WORDS[0], 0x3600_00a0);
        let (vtable, text) = (0x8566_6320, 0x8042_5000);
        assert!(local_pane_identity("set_text_dev_count", vtable, text));
        assert!(!local_pane_identity("set_txt_roomid", vtable, text));
        assert!(!local_pane_identity("set_text_dev_count", vtable + 8, text));
        assert!(!local_pane_identity("set_text_dev_count", vtable, 0));
        assert!(local_count_active(4, 4));
        assert!(!local_count_active(5, 4));
        assert!(!local_count_active(4, 0));
        assert_eq!(ARENA_ROOM_NAME, "set_txt_roomid");
        assert_eq!(CSS_TITLE_WALK.len(), 6);
        assert_eq!(CSS_TITLE_WALK[CSS_BACKGROUND_STEPS - 1], Prev);
        assert_eq!(CSS_TITLE_WALK[CSS_BACKGROUND_STEPS], Parent);
    }
}
