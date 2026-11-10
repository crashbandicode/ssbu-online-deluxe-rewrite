//! The label feed of the diagnostic build (`--features diag`), for Eden: the
//! opponent label shows, in turn, names read from the diag file instead of
//! the opponent's own, so a test can watch the overlay's dynamic glyph set
//! follow the text on screen. Never part of the shipped NRO.
//!
//! Diag file lines: `label_feed=日本語のなまえ|강철무사|Ølstue Ærø` and
//! `label_feed_seconds=12` (1-600, default 10).

pub struct Feed {
    pub names: Vec<String>,
    pub step_ms: u64,
}

pub fn parse(text: &str) -> Option<Feed> {
    let mut names = Vec::new();
    let mut seconds = 10u64;
    for line in text.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("label_feed=") {
            names = value
                .split('|')
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_owned)
                .collect();
        } else if let Some(value) = line.strip_prefix("label_feed_seconds=") {
            if let Ok(value) = value.trim().parse::<u64>() {
                seconds = value.clamp(1, 600);
            }
        }
    }
    (!names.is_empty()).then(|| Feed {
        names,
        step_ms: seconds * 1000,
    })
}

/// The name shown `elapsed_ms` after the opponent's label first showed: index
/// 0 (the opponent's own name) for one step, then each fed name (1..) for one
/// step each, then the opponent's own name for good.
pub fn index_at(feed: &Feed, elapsed_ms: u64) -> usize {
    let step = (elapsed_ms / feed.step_ms) as usize;
    if step > feed.names.len() {
        0
    } else {
        step
    }
}

pub fn name<'a>(feed: &'a Feed, index: usize, own: &'a str) -> &'a str {
    index
        .checked_sub(1)
        .and_then(|i| feed.names.get(i))
        .map_or(own, String::as_str)
}

/// Distinct characters of `name` outside the overlay's static glyph set
/// (ASCII, Latin-1 Supplement, Latin Extended-A): the dynamic glyphs it needs.
pub fn dynamic_chars(name: &str) -> usize {
    let mut chars: Vec<char> = name
        .chars()
        .filter(|c| {
            let c = u32::from(*c);
            !(c <= 0x7E || (0xA0..=0x17F).contains(&c))
        })
        .collect();
    chars.sort_unstable();
    chars.dedup();
    chars.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_names_and_step() {
        let feed = parse("# test\nlabel_feed= 日本語のなまえ | 강철무사 ||Ølstue Ærø\nlabel_feed_seconds=12\n")
            .unwrap();
        assert_eq!(feed.names, ["日本語のなまえ", "강철무사", "Ølstue Ærø"]);
        assert_eq!(feed.step_ms, 12_000);
        assert_eq!(parse("label_feed=a\nlabel_feed_seconds=0").unwrap().step_ms, 1000);
        assert_eq!(parse("label_feed=a\nlabel_feed_seconds=x").unwrap().step_ms, 10_000);
        assert!(parse("label_feed=\nlabel_feed_seconds=5").is_none());
        assert!(parse("").is_none());
    }

    #[test]
    fn own_name_then_each_fed_name_then_own_name_for_good() {
        let feed = parse("label_feed=日本語のなまえ|강철무사\nlabel_feed_seconds=10").unwrap();
        let shown: Vec<&str> = [0, 9_999, 10_000, 19_999, 20_000, 29_999, 30_000, 900_000]
            .iter()
            .map(|ms| name(&feed, index_at(&feed, *ms), "Æsir テスト"))
            .collect();
        assert_eq!(
            shown,
            [
                "Æsir テスト",
                "Æsir テスト",
                "日本語のなまえ",
                "日本語のなまえ",
                "강철무사",
                "강철무사",
                "Æsir テスト",
                "Æsir テスト"
            ]
        );
    }

    #[test]
    fn dynamic_chars_skip_the_static_set() {
        assert_eq!(dynamic_chars("Æsir テスト"), 3);
        assert_eq!(dynamic_chars("Ølstue Ærø Ōno"), 0);
        assert_eq!(dynamic_chars("日本語のなまえ"), 7);
        assert_eq!(dynamic_chars("강철무사"), 4);
        assert_eq!(dynamic_chars("ててて"), 1);
    }
}
