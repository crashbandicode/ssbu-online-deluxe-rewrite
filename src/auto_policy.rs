//! Scene request for the E2 machine. This does not write the INI and does not
//! guess an untracked Quickplay match. LLDoubles is the live 3-texture
//! OneBehind profile. Dynamic resolution is not applied here.

use crate::config::RenderMode;
use crate::policy::{self, MatchKind, Profile};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Connection {
    Offline = 0,
    Local = 1,
    Arena = 2,
    Quickplay = 3,
}

impl Connection {
    pub const fn from_byte(value: u8) -> Self {
        match value {
            1 => Self::Local,
            2 => Self::Arena,
            3 => Self::Quickplay,
            _ => Self::Offline,
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Self::Offline => "offline",
            Self::Local => "local",
            Self::Arena => "arena",
            Self::Quickplay => "quickplay",
        }
    }

    pub const fn is_online(self) -> bool {
        !matches!(self, Self::Offline)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SceneStatus {
    Inactive = 0,
    Training = 1,
    Singles = 2,
    Doubles = 3,
}

impl SceneStatus {
    pub const fn from_byte(value: u8) -> Self {
        match value {
            1 => Self::Training,
            2 => Self::Singles,
            3 => Self::Doubles,
            _ => Self::Inactive,
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Self::Inactive => "inactive",
            Self::Training => "training",
            Self::Singles => "singles",
            Self::Doubles => "doubles",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Platform {
    Console = 0,
    Emulator = 1,
}

impl Platform {
    pub const fn from_byte(value: u8) -> Self {
        match value {
            1 => Self::Emulator,
            _ => Self::Console,
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Self::Console => "console",
            Self::Emulator => "emulator",
        }
    }

    pub const fn emulator(self) -> bool {
        matches!(self, Self::Emulator)
    }
}

/// `ready` is the scene-hook install gate. A failed guard leaves Auto untouched.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Scene {
    pub ready: bool,
    pub connection: Connection,
    pub status: SceneStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Selection {
    pub auto_profile: bool,
    pub manual: RenderMode,
    pub platform: Platform,
    pub selection_generation: u64,
    /// `active` remembers a post-match Vanilla hold for manual mode.
    /// The generation must still match `selection_generation`.
    pub cleanup_active: bool,
    pub cleanup_generation: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Applied {
    pub request: RenderMode,
    pub available: bool,
    pub auto_applied: bool,
    pub cleanup_hold: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProfileChoice {
    Auto,
    Manual(RenderMode),
}

const MANUAL_CYCLE: [RenderMode; 4] = [
    RenderMode::Vanilla,
    RenderMode::LessLag,
    RenderMode::LessLagUltra,
    RenderMode::LessLagDoubles,
];

/// Stage-setup classification. Result stages win over the training symbol.
/// Fighter count is authoritative for Singles versus Doubles; this does not
/// look at station count. Inactive, including a connected CSS or results
/// screen, is not classified here.
pub fn classify_stage(stage_id: u32, training: bool, fighter_count: i32) -> SceneStatus {
    if stage_id == 310 || stage_id == 354 {
        SceneStatus::Inactive
    } else if training || stage_id == 311 {
        SceneStatus::Training
    } else if fighter_count > 2 {
        SceneStatus::Doubles
    } else {
        SceneStatus::Singles
    }
}

/// CSS reflects the live Training-mode bit, not the stage that led to it.
pub fn classify_css(training_mode: bool) -> SceneStatus {
    if training_mode {
        SceneStatus::Training
    } else {
        SceneStatus::Inactive
    }
}

pub fn scene_status_key(ready: bool, status: SceneStatus) -> &'static str {
    if ready {
        status.key()
    } else {
        "unavailable"
    }
}

/// Inactive is the menu profile for every connection. The fork applies a lag
/// profile at match init and returns to the menu profile at match cleanup, so
/// a connected CSS or results screen stays Vanilla until stage setup.
pub fn match_kind(connection: Connection, status: SceneStatus) -> MatchKind {
    match status {
        SceneStatus::Training => MatchKind::Training,
        SceneStatus::Inactive => MatchKind::Menu,
        SceneStatus::Singles => match connection {
            Connection::Offline => MatchKind::OfflineSingles,
            Connection::Local => MatchKind::LocalSingles,
            Connection::Arena => MatchKind::ArenaSingles,
            Connection::Quickplay => MatchKind::QuickplaySingles,
        },
        SceneStatus::Doubles => match connection {
            Connection::Offline => MatchKind::OfflineDoubles,
            Connection::Local => MatchKind::LocalDoubles,
            Connection::Arena => MatchKind::ArenaDoubles,
            Connection::Quickplay => MatchKind::QuickplayDoubles,
        },
    }
}

pub fn mode_for(profile: Profile) -> RenderMode {
    match profile {
        Profile::Vanilla => RenderMode::Vanilla,
        Profile::LessLag => RenderMode::LessLag,
        Profile::LessLagUltra => RenderMode::LessLagUltra,
        Profile::LessLagDoubles => RenderMode::LessLagDoubles,
    }
}

/// Auto uses the scene table. Inactive always requests the menu profile.
/// Manual keeps the saved mode in menus, matches, and Training. That Training
/// path is an accepted diagnostic choice; Auto still requests Vanilla there.
/// After a real match or Training ends, manual requests Vanilla until the next
/// match or a newer explicit selection. A guard failure returns the saved mode
/// and does not apply Auto.
pub fn decide(scene: Scene, selection: Selection) -> Applied {
    if !scene.ready {
        return Applied {
            request: selection.manual,
            available: false,
            auto_applied: false,
            cleanup_hold: false,
        };
    }
    if selection.auto_profile {
        let profile = policy::select_profile(
            match_kind(scene.connection, scene.status),
            selection.platform.emulator(),
            None,
        );
        return Applied {
            request: mode_for(profile),
            available: true,
            auto_applied: true,
            cleanup_hold: false,
        };
    }
    let cleanup_hold = selection.cleanup_active
        && selection.cleanup_generation == selection.selection_generation
        && scene.status == SceneStatus::Inactive;
    Applied {
        request: if cleanup_hold {
            RenderMode::Vanilla
        } else {
            selection.manual
        },
        available: true,
        auto_applied: false,
        cleanup_hold,
    }
}

/// Leaving Singles, Doubles, or Training for Inactive holds Vanilla for the
/// current selection generation. Entering any of those clears the hold.
pub fn transition_cleanup(
    previous: SceneStatus,
    next: SceneStatus,
    selection_generation: u64,
    cleanup_active: bool,
    cleanup_generation: u64,
) -> (bool, u64) {
    match (previous, next) {
        (
            SceneStatus::Singles | SceneStatus::Doubles | SceneStatus::Training,
            SceneStatus::Inactive,
        ) => (true, selection_generation),
        (
            SceneStatus::Inactive,
            SceneStatus::Singles | SceneStatus::Doubles | SceneStatus::Training,
        ) => (false, 0),
        _ => (cleanup_active, cleanup_generation),
    }
}

/// Later menu contract. Either direction from Auto lands on manual Vanilla.
/// Left from Vanilla, or right from LLDoubles, returns to Auto. Core modes are
/// not in the cycle; either direction lands on manual Vanilla.
pub fn cycle_profile_choice(current: ProfileChoice, direction: i8) -> ProfileChoice {
    let ProfileChoice::Manual(mode) = current else {
        return ProfileChoice::Manual(RenderMode::Vanilla);
    };
    let Some(index) = MANUAL_CYCLE.iter().position(|item| *item == mode) else {
        return ProfileChoice::Manual(RenderMode::Vanilla);
    };
    if index == 0 && direction < 0 || index + 1 == MANUAL_CYCLE.len() && direction > 0 {
        return ProfileChoice::Auto;
    }
    let next = index as i8 + direction;
    ProfileChoice::Manual(MANUAL_CYCLE[next as usize])
}

pub fn profile_row_label(
    auto_profile: bool,
    ready: bool,
    selected: RenderMode,
    requested: RenderMode,
) -> String {
    if auto_profile && !ready {
        return "Auto (unavailable)".to_owned();
    }
    if auto_profile {
        auto_choice_label(requested).to_owned()
    } else if requested == RenderMode::Vanilla && selected != RenderMode::Vanilla {
        // Cleanup hold: after a match, Training or the Smash menu the renderer
        // stays on Vanilla until a newer explicit selection or the next match.
        // Show what is live, marked, instead of the saved choice (2026-10-14
        // hardware report: banner said LessLag while the renderer ran Vanilla).
        "Vanilla*".to_owned()
    } else {
        selected.label().to_owned()
    }
}

fn auto_choice_label(requested: RenderMode) -> &'static str {
    match requested {
        RenderMode::Vanilla => "Auto:Van",
        RenderMode::LessLag => "Auto:Less",
        RenderMode::LessLagUltra => "Auto:Ultra",
        RenderMode::LessLagDoubles => "Auto:LLDbl",
        RenderMode::LessLagCore | RenderMode::DoublesCore => requested.label(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selection(auto_profile: bool, manual: RenderMode, platform: Platform) -> Selection {
        Selection {
            auto_profile,
            manual,
            platform,
            selection_generation: 4,
            cleanup_active: false,
            cleanup_generation: 0,
        }
    }

    fn scene(connection: Connection, status: SceneStatus) -> Scene {
        Scene {
            ready: true,
            connection,
            status,
        }
    }

    #[test]
    fn stage_words_classify_training_results_and_fighter_count() {
        assert_eq!(classify_stage(310, false, 4), SceneStatus::Inactive);
        assert_eq!(classify_stage(354, true, 4), SceneStatus::Inactive);
        assert_eq!(classify_stage(311, false, 2), SceneStatus::Training);
        assert_eq!(classify_stage(1, true, 4), SceneStatus::Training);
        assert_eq!(classify_stage(1, false, 2), SceneStatus::Singles);
        assert_eq!(classify_stage(1, false, 3), SceneStatus::Doubles);
        assert_eq!(classify_stage(1, false, 4), SceneStatus::Doubles);
        assert_eq!(classify_stage(1, false, 0), SceneStatus::Singles);
    }

    #[test]
    fn regular_css_after_stage_311_is_inactive_left_and_holds_manual_cleanup() {
        let waiting_room = Scene {
            ready: true,
            connection: Connection::Offline,
            status: classify_stage(311, false, 2),
        };
        assert_eq!(waiting_room.status, SceneStatus::Training);

        let css = Scene {
            status: classify_css(false),
            ..waiting_room
        };
        assert_eq!(css.connection, waiting_room.connection);
        assert_eq!(css.status, SceneStatus::Inactive);
        assert_eq!(match_kind(css.connection, css.status), MatchKind::Menu);
        assert!(!crate::hud_place::on_right(
            css.ready,
            css.connection,
            css.status
        ));

        let mut manual = selection(false, RenderMode::LessLag, Platform::Console);
        let (active, generation) = transition_cleanup(
            waiting_room.status,
            css.status,
            manual.selection_generation,
            manual.cleanup_active,
            manual.cleanup_generation,
        );
        manual.cleanup_active = active;
        manual.cleanup_generation = generation;
        let after_css = decide(css, manual);
        assert!(after_css.cleanup_hold);
        assert_eq!(after_css.request, RenderMode::Vanilla);

        // CSS does not infer or overwrite the transport. Inactive remains the
        // menu/Vanilla Auto choice regardless of the preserved connection.
        for connection in [
            Connection::Offline,
            Connection::Local,
            Connection::Arena,
            Connection::Quickplay,
        ] {
            let context = Scene { connection, ..css };
            assert_eq!(context.connection, connection);
            assert_eq!(
                match_kind(context.connection, context.status),
                MatchKind::Menu
            );
            let auto = decide(
                context,
                selection(true, RenderMode::LessLagUltra, Platform::Console),
            );
            assert_eq!(auto.request, RenderMode::Vanilla, "{connection:?}");
            assert!(auto.auto_applied);
            assert!(!auto.cleanup_hold);
        }
    }

    #[test]
    fn training_mode_css_stays_training_and_right_without_cleanup() {
        let status = classify_css(true);
        assert_eq!(status, SceneStatus::Training);
        let training_css = scene(Connection::Offline, status);
        assert!(crate::hud_place::on_right(
            training_css.ready,
            training_css.connection,
            training_css.status
        ));

        let (active, generation) = transition_cleanup(SceneStatus::Training, status, 4, false, 0);
        assert!(!active);
        assert_eq!(generation, 0);

        let manual = decide(
            training_css,
            selection(false, RenderMode::LessLagUltra, Platform::Console),
        );
        assert_eq!(manual.request, RenderMode::LessLagUltra);
        assert!(!manual.cleanup_hold);
        let auto = decide(
            training_css,
            selection(true, RenderMode::LessLagUltra, Platform::Console),
        );
        assert_eq!(auto.request, RenderMode::Vanilla);
        assert!(!auto.cleanup_hold);
    }

    #[test]
    fn auto_matrix_uses_connection_and_match_count_without_a_quickplay_guess() {
        let cases = [
            (
                Connection::Offline,
                SceneStatus::Inactive,
                false,
                RenderMode::Vanilla,
            ),
            (
                Connection::Offline,
                SceneStatus::Training,
                true,
                RenderMode::Vanilla,
            ),
            (
                Connection::Offline,
                SceneStatus::Singles,
                false,
                RenderMode::Vanilla,
            ),
            (
                Connection::Offline,
                SceneStatus::Doubles,
                true,
                RenderMode::Vanilla,
            ),
            (
                Connection::Local,
                SceneStatus::Inactive,
                false,
                RenderMode::Vanilla,
            ),
            (
                Connection::Local,
                SceneStatus::Inactive,
                true,
                RenderMode::Vanilla,
            ),
            (
                Connection::Arena,
                SceneStatus::Inactive,
                false,
                RenderMode::Vanilla,
            ),
            (
                Connection::Quickplay,
                SceneStatus::Inactive,
                true,
                RenderMode::Vanilla,
            ),
            (
                Connection::Arena,
                SceneStatus::Singles,
                false,
                RenderMode::LessLag,
            ),
            (
                Connection::Quickplay,
                SceneStatus::Singles,
                true,
                RenderMode::LessLagUltra,
            ),
            (
                Connection::Local,
                SceneStatus::Doubles,
                false,
                RenderMode::LessLagDoubles,
            ),
            (
                Connection::Arena,
                SceneStatus::Doubles,
                true,
                RenderMode::LessLagDoubles,
            ),
            (
                Connection::Quickplay,
                SceneStatus::Doubles,
                false,
                RenderMode::LessLagDoubles,
            ),
            (
                Connection::Local,
                SceneStatus::Training,
                true,
                RenderMode::Vanilla,
            ),
            (
                Connection::Offline,
                SceneStatus::Singles,
                true,
                RenderMode::Vanilla,
            ),
        ];
        for (connection, status, emulator, expected) in cases {
            let platform = if emulator {
                Platform::Emulator
            } else {
                Platform::Console
            };
            let applied = decide(
                scene(connection, status),
                selection(true, RenderMode::LessLagUltra, platform),
            );
            assert_eq!(
                applied.request, expected,
                "{connection:?} {status:?} {emulator}"
            );
            assert!(applied.auto_applied);
            assert!(applied.available);
            assert!(!applied.cleanup_hold);
        }
    }

    #[test]
    fn guard_failure_keeps_the_saved_mode_and_reports_unavailable() {
        let applied = decide(
            Scene {
                ready: false,
                connection: Connection::Local,
                status: SceneStatus::Doubles,
            },
            selection(true, RenderMode::LessLag, Platform::Emulator),
        );
        assert_eq!(applied.request, RenderMode::LessLag);
        assert!(!applied.available);
        assert!(!applied.auto_applied);
        assert_eq!(scene_status_key(false, SceneStatus::Doubles), "unavailable");
        assert_eq!(
            profile_row_label(true, false, RenderMode::LessLag, applied.request),
            "Auto (unavailable)"
        );
    }

    #[test]
    fn manual_survives_matches_and_training_but_cleanup_requests_vanilla() {
        let mut saved = selection(false, RenderMode::LessLagDoubles, Platform::Console);
        let training = decide(scene(Connection::Offline, SceneStatus::Training), saved);
        assert_eq!(training.request, RenderMode::LessLagDoubles);
        assert!(!training.cleanup_hold);
        let playing = decide(scene(Connection::Arena, SceneStatus::Doubles), saved);
        assert_eq!(playing.request, RenderMode::LessLagDoubles);
        let (active, generation) = transition_cleanup(
            SceneStatus::Doubles,
            SceneStatus::Inactive,
            saved.selection_generation,
            false,
            0,
        );
        saved.cleanup_active = active;
        saved.cleanup_generation = generation;
        let results = decide(scene(Connection::Arena, SceneStatus::Inactive), saved);
        assert!(results.cleanup_hold);
        assert_eq!(results.request, RenderMode::Vanilla);
        assert_eq!(saved.manual, RenderMode::LessLagDoubles);
        saved.selection_generation = saved.selection_generation.wrapping_add(1);
        let edited = decide(scene(Connection::Arena, SceneStatus::Inactive), saved);
        assert!(!edited.cleanup_hold);
        assert_eq!(edited.request, RenderMode::LessLagDoubles);
        let (active, generation) = transition_cleanup(
            SceneStatus::Inactive,
            SceneStatus::Singles,
            saved.selection_generation,
            saved.cleanup_active,
            saved.cleanup_generation,
        );
        saved.cleanup_active = active;
        saved.cleanup_generation = generation;
        let next = decide(scene(Connection::Quickplay, SceneStatus::Singles), saved);
        assert_eq!(next.request, RenderMode::LessLagDoubles);
        assert!(!next.cleanup_hold);
    }

    #[test]
    fn auto_ignores_the_manual_hold_and_inactive_requests_menu() {
        let mut saved = selection(true, RenderMode::Vanilla, Platform::Console);
        saved.cleanup_active = true;
        saved.cleanup_generation = saved.selection_generation;
        let css = decide(scene(Connection::Local, SceneStatus::Inactive), saved);
        assert_eq!(css.request, RenderMode::Vanilla);
        assert!(!css.cleanup_hold);
        assert_eq!(
            match_kind(Connection::Local, SceneStatus::Inactive),
            MatchKind::Menu
        );
        assert_eq!(
            match_kind(Connection::Quickplay, SceneStatus::Inactive),
            MatchKind::Menu
        );
        assert_eq!(
            profile_row_label(true, true, RenderMode::Vanilla, css.request),
            "Auto:Van"
        );
        assert_eq!(
            profile_row_label(true, true, RenderMode::Vanilla, RenderMode::LessLag),
            "Auto:Less"
        );
        assert_eq!(
            profile_row_label(true, true, RenderMode::Vanilla, RenderMode::LessLagUltra),
            "Auto:Ultra"
        );
        assert_eq!(
            profile_row_label(true, true, RenderMode::Vanilla, RenderMode::LessLagDoubles),
            "Auto:LLDbl"
        );
        // Manual mode with a Vanilla request is the cleanup hold: the label
        // follows the live renderer, marked, not the saved choice.
        assert_eq!(
            profile_row_label(false, true, RenderMode::LessLagDoubles, RenderMode::Vanilla),
            "Vanilla*"
        );
        assert_eq!(
            profile_row_label(false, true, RenderMode::LessLag, RenderMode::Vanilla),
            "Vanilla*"
        );
        assert_eq!(
            profile_row_label(false, true, RenderMode::LessLagUltra, RenderMode::Vanilla),
            "Vanilla*"
        );
        assert_eq!(
            profile_row_label(false, true, RenderMode::LessLag, RenderMode::LessLag),
            "LessLag"
        );
        assert_eq!(
            profile_row_label(false, true, RenderMode::Vanilla, RenderMode::Vanilla),
            "Vanilla"
        );
        assert_eq!(
            profile_row_label(false, true, RenderMode::Vanilla, RenderMode::LessLagDoubles),
            "Vanilla"
        );
    }

    #[test]
    fn same_ini_values_do_not_define_a_new_selection_generation() {
        let previous = selection(false, RenderMode::LessLag, Platform::Console);
        let reloaded = previous;
        assert_eq!(previous.selection_generation, reloaded.selection_generation);
        let (active, generation) = transition_cleanup(
            SceneStatus::Singles,
            SceneStatus::Inactive,
            previous.selection_generation,
            false,
            0,
        );
        let mut held = previous;
        held.cleanup_active = active;
        held.cleanup_generation = generation;
        assert_eq!(
            decide(scene(Connection::Offline, SceneStatus::Inactive), held).request,
            RenderMode::Vanilla
        );
        assert_eq!(
            decide(scene(Connection::Offline, SceneStatus::Inactive), reloaded).request,
            RenderMode::LessLag
        );
    }

    #[test]
    fn menu_cycle_inserts_lldoubles_and_returns_to_auto_at_the_ends() {
        let mut choice = ProfileChoice::Auto;
        choice = cycle_profile_choice(choice, 1);
        assert_eq!(choice, ProfileChoice::Manual(RenderMode::Vanilla));
        choice = cycle_profile_choice(choice, -1);
        assert_eq!(choice, ProfileChoice::Auto);
        choice = cycle_profile_choice(choice, -1);
        assert_eq!(choice, ProfileChoice::Manual(RenderMode::Vanilla));
        for expected in [
            RenderMode::LessLag,
            RenderMode::LessLagUltra,
            RenderMode::LessLagDoubles,
        ] {
            choice = cycle_profile_choice(choice, 1);
            assert_eq!(choice, ProfileChoice::Manual(expected));
        }
        choice = cycle_profile_choice(choice, 1);
        assert_eq!(choice, ProfileChoice::Auto);
        choice = ProfileChoice::Manual(RenderMode::DoublesCore);
        assert_eq!(
            cycle_profile_choice(choice, 1),
            ProfileChoice::Manual(RenderMode::Vanilla)
        );
        assert_eq!(
            cycle_profile_choice(ProfileChoice::Manual(RenderMode::LessLagCore), -1),
            ProfileChoice::Manual(RenderMode::Vanilla)
        );
    }
}
