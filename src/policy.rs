//! Fork-parity profile table. Auto recommendations never give Training or the
//! menu a lag profile. `auto_policy` applies this table when Auto is on.
//! Manual runtime requests are separate: Training can still use an explicit
//! saved profile. Connected local wireless is online, matching OnlineLocal.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchKind {
    Menu,
    Training,
    OfflineSingles,
    OfflineDoubles,
    QuickplaySingles,
    QuickplayDoubles,
    ArenaSingles,
    ArenaDoubles,
    LocalSingles,
    LocalDoubles,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Profile {
    Vanilla,
    LessLag,
    LessLagUltra,
    LessLagDoubles,
}

pub fn select_profile(kind: MatchKind, emulator: bool, manual: Option<Profile>) -> Profile {
    match kind {
        MatchKind::Menu | MatchKind::Training => Profile::Vanilla,
        _ if manual.is_some() => manual.unwrap_or(Profile::Vanilla),
        MatchKind::QuickplayDoubles | MatchKind::ArenaDoubles | MatchKind::LocalDoubles => {
            Profile::LessLagDoubles
        }
        MatchKind::QuickplaySingles | MatchKind::ArenaSingles | MatchKind::LocalSingles
            if emulator =>
        {
            Profile::LessLagUltra
        }
        MatchKind::QuickplaySingles | MatchKind::ArenaSingles | MatchKind::LocalSingles => {
            Profile::LessLag
        }
        MatchKind::OfflineSingles | MatchKind::OfflineDoubles => Profile::Vanilla,
    }
}

/// Auto and invalid selections preserve the game's current buffer. Runtime
/// chooses where to invoke this; this function makes no match-kind assumptions.
pub fn apply_input_buffer(current: u8, selected: i8) -> u8 {
    if (0..=25).contains(&selected) {
        selected as u8
    } else {
        current
    }
}

/// The frame count the game chose, observable only while the selection is
/// Auto: a manual selection overwrites the byte, so later calls would echo ours.
pub fn observed_auto_buffer(automatic: u8, selected: i8) -> Option<u8> {
    if (0..=25).contains(&selected) {
        None
    } else {
        Some(automatic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_buffer_is_observed_only_while_auto_is_selected() {
        assert_eq!(observed_auto_buffer(4, -1), Some(4));
        assert_eq!(observed_auto_buffer(0, -1), Some(0));
        assert_eq!(observed_auto_buffer(4, i8::MIN), Some(4));
        assert_eq!(observed_auto_buffer(4, 3), None);
        assert_eq!(observed_auto_buffer(4, 25), None);
    }

    #[test]
    fn manual_match_choice_survives_online_defaults_but_not_training_or_menu() {
        for kind in [
            MatchKind::QuickplaySingles,
            MatchKind::QuickplayDoubles,
            MatchKind::ArenaSingles,
            MatchKind::ArenaDoubles,
            MatchKind::OfflineSingles,
            MatchKind::OfflineDoubles,
            MatchKind::LocalSingles,
            MatchKind::LocalDoubles,
        ] {
            assert_eq!(
                select_profile(kind, true, Some(Profile::Vanilla)),
                Profile::Vanilla
            );
            assert_eq!(
                select_profile(kind, false, Some(Profile::LessLagUltra)),
                Profile::LessLagUltra
            );
        }
        for kind in [MatchKind::Training, MatchKind::Menu] {
            assert_eq!(
                select_profile(kind, true, Some(Profile::LessLagDoubles)),
                Profile::Vanilla
            );
        }
    }

    #[test]
    fn network_defaults_preserve_singles_doubles_and_offline_separation() {
        for kind in [MatchKind::QuickplaySingles, MatchKind::ArenaSingles] {
            assert_eq!(select_profile(kind, false, None), Profile::LessLag);
            assert_eq!(select_profile(kind, true, None), Profile::LessLagUltra);
        }
        for emulator in [false, true] {
            for kind in [MatchKind::QuickplayDoubles, MatchKind::ArenaDoubles] {
                assert_eq!(
                    select_profile(kind, emulator, None),
                    Profile::LessLagDoubles
                );
            }
            for kind in [
                MatchKind::Menu,
                MatchKind::Training,
                MatchKind::OfflineSingles,
                MatchKind::OfflineDoubles,
            ] {
                assert_eq!(select_profile(kind, emulator, None), Profile::Vanilla);
            }
            assert_eq!(
                select_profile(MatchKind::LocalSingles, emulator, None),
                if emulator {
                    Profile::LessLagUltra
                } else {
                    Profile::LessLag
                }
            );
            assert_eq!(
                select_profile(MatchKind::LocalDoubles, emulator, None),
                Profile::LessLagDoubles
            );
        }
    }

    #[test]
    fn auto_and_invalid_values_preserve_even_game_buffers_outside_menu_range() {
        for selected in [-128, -2, -1, 26, 127] {
            assert_eq!(apply_input_buffer(40, selected), 40);
        }
        assert_eq!(apply_input_buffer(40, 0), 0);
        assert_eq!(apply_input_buffer(40, 25), 25);
    }
}
