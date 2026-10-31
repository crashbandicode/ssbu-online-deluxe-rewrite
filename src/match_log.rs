//! One log per stage (0.9.0): names, the index that bounds how many are kept,
//! and the present-spacing summary written when the stage ends. The runtime
//! (`runtime/match_log.rs`) does the file work; this part is host-tested.
use crate::auto_policy::SceneStatus;

/// Folder under the data directory.
pub const DIRECTORY: &str = "matches";
/// Names of the kept logs, oldest first, one per line. The index replaces a
/// directory listing, which the runtime does not otherwise use.
pub const INDEX: &str = "index.txt";
/// Logs kept; the oldest beyond this are deleted when a new one opens.
pub const KEEP: usize = 60;
/// Upper edges (µs) of the present-spacing bins; the last bin is open. One
/// frame at 60 FPS is 16,667 µs; one 120 Hz period is 8,333 µs.
pub const SPACING_EDGES_US: [u64; 6] = [6_000, 10_000, 14_000, 19_000, 25_000, 40_000];
pub const SPACING_BINS: usize = SPACING_EDGES_US.len() + 1;
const SPACING_LABELS: [&str; SPACING_BINS] = [
    "<6ms", "6-10ms", "10-14ms", "14-19ms", "19-25ms", "25-40ms", ">=40ms",
];

/// `000042-singles-6b.log`: sequence, scene, stage id in hex.
pub fn file_name(sequence: u64, scene: SceneStatus, stage: u64) -> String {
    format!("{sequence:06}-{}-{stage:x}.log", scene.key())
}

fn sequence_of(name: &str) -> Option<u64> {
    name.split('-').next()?.parse().ok()
}

/// The kept names, oldest first. Anything that is not a plain log name
/// (a path, a stray line) is ignored, so a damaged index cannot point the
/// cleanup outside the folder.
pub fn parse_index(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|name| {
            name.ends_with(".log")
                && !name.contains(['/', '\\'])
                && !name.starts_with('.')
                && sequence_of(name).is_some()
        })
        .map(str::to_owned)
        .collect()
}

pub fn render_index(names: &[String]) -> String {
    names.iter().map(|name| format!("{name}\n")).collect()
}

/// One past the highest kept sequence, so numbering continues across boots.
pub fn next_sequence(names: &[String]) -> u64 {
    names
        .iter()
        .filter_map(|name| sequence_of(name))
        .max()
        .map_or(1, |sequence| sequence + 1)
}

/// Adds `name`; returns the names to keep (oldest first) and the ones to
/// delete.
pub fn admit(mut names: Vec<String>, name: String) -> (Vec<String>, Vec<String>) {
    names.push(name);
    let excess = names.len().saturating_sub(KEEP);
    let expired = names.drain(..excess).collect();
    (names, expired)
}

pub fn spacing_bin(spacing_us: u64) -> usize {
    SPACING_EDGES_US
        .iter()
        .position(|edge| spacing_us < *edge)
        .unwrap_or(SPACING_EDGES_US.len())
}

/// `present_spacing <6ms=0 6-10ms=0 10-14ms=1 14-19ms=14520 …`
pub fn spacing_line(bins: &[u64; SPACING_BINS]) -> String {
    let fields: Vec<String> = SPACING_LABELS
        .iter()
        .zip(bins)
        .map(|(label, count)| format!("{label}={count}"))
        .collect();
    format!("present_spacing {}", fields.join(" "))
}

/// The scene an `AutoContext` (code 73) row carries in bits 8–15 of `a`.
pub const fn context_scene(a: u64) -> SceneStatus {
    SceneStatus::from_byte((a >> 8) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_carry_sequence_scene_and_stage() {
        assert_eq!(
            file_name(42, SceneStatus::Singles, 0x6b),
            "000042-singles-6b.log"
        );
        assert_eq!(sequence_of("000042-singles-6b.log"), Some(42));
    }

    #[test]
    fn the_index_ignores_paths_and_junk_and_numbering_continues() {
        let names = parse_index(
            "000003-training-0.log\n../config.ini\nsub/000009-x.log\n.hidden.log\njunk\n000007-doubles-1a.log\n",
        );
        assert_eq!(names, ["000003-training-0.log", "000007-doubles-1a.log"]);
        assert_eq!(next_sequence(&names), 8);
        assert_eq!(next_sequence(&[]), 1);
        assert_eq!(parse_index(&render_index(&names)), names);
    }

    #[test]
    fn admitting_past_the_limit_expires_the_oldest() {
        let names: Vec<String> = (1..=KEEP as u64)
            .map(|n| file_name(n, SceneStatus::Training, 0))
            .collect();
        let (kept, expired) = admit(names.clone(), file_name(99, SceneStatus::Singles, 1));
        assert_eq!(kept.len(), KEEP);
        assert_eq!(expired, [names[0].clone()]);
        assert_eq!(kept.last().unwrap(), "000099-singles-1.log");
        let (kept, expired) = admit(Vec::new(), "000001-singles-1.log".into());
        assert_eq!((kept.len(), expired.len()), (1, 0));
    }

    #[test]
    fn spacing_bins_split_60fps_from_120hz_halves_and_drops() {
        assert_eq!(spacing_bin(16_667), 3);
        assert_eq!(spacing_bin(8_333), 1);
        assert_eq!(spacing_bin(33_333), 5);
        assert_eq!(spacing_bin(5_999), 0);
        assert_eq!(spacing_bin(40_000), SPACING_BINS - 1);
        let mut bins = [0; SPACING_BINS];
        bins[3] = 14_520;
        assert_eq!(
            spacing_line(&bins),
            "present_spacing <6ms=0 6-10ms=0 10-14ms=0 14-19ms=14520 19-25ms=0 25-40ms=0 >=40ms=0"
        );
    }

    #[test]
    fn context_rows_carry_the_scene_in_the_second_byte() {
        assert_eq!(context_scene(0x1_0203), SceneStatus::Singles);
        assert_eq!(context_scene(0x1_0003), SceneStatus::Inactive);
    }
}
