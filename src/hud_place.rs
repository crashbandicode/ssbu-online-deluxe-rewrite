//! Where the global Player Info window sits. Native CSS and arena panes are
//! not moved. A validated scene comes from the existing Auto snapshot.

use crate::auto_policy::{Connection, SceneStatus};

/// Gap between the window and the right edge of the viewport, in pixels.
pub const HUD_MARGIN: f32 = 16.0;

/// Training, and Arena/Local/Quickplay outside a singles or doubles match.
/// An unvalidated snapshot and the main menu stay on the left.
pub fn on_right(ready: bool, connection: Connection, status: SceneStatus) -> bool {
    if !ready {
        return false;
    }
    match status {
        SceneStatus::Training => true,
        SceneStatus::Singles | SceneStatus::Doubles => false,
        SceneStatus::Inactive => connection.is_online(),
    }
}

/// Margin for a usable viewport. A bad viewport contributes no gap.
pub fn margin_for(viewport_width: f32) -> f32 {
    if viewport_width.is_finite() && viewport_width > 0.0 {
        HUD_MARGIN
    } else {
        0.0
    }
}

/// Measured outer width when it is usable, otherwise the size configured for
/// this frame. Neither value is a screen resolution.
pub fn window_extent(measured: Option<f32>, configured: f32) -> Option<f32> {
    if let Some(measured) = measured {
        if measured.is_finite() && measured > 0.0 {
            return Some(measured);
        }
    }
    (configured.is_finite() && configured > 0.0).then_some(configured)
}

/// Left is the origin. Right is viewport width minus the window and the margin.
pub fn origin_x(
    on_right: bool,
    viewport_width: f32,
    measured: Option<f32>,
    configured: f32,
    margin: f32,
) -> f32 {
    if !on_right || !viewport_width.is_finite() || viewport_width <= 0.0 {
        return 0.0;
    }
    let Some(extent) = window_extent(measured, configured) else {
        return 0.0;
    };
    let margin = if margin.is_finite() && margin > 0.0 {
        margin
    } else {
        0.0
    };
    (viewport_width - extent - margin).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.02, "{actual} != {expected}");
    }

    #[test]
    fn training_and_online_lobbies_are_right_matches_and_the_menu_stay_left() {
        let right = [
            (Connection::Offline, SceneStatus::Training),
            (Connection::Local, SceneStatus::Training),
            (Connection::Arena, SceneStatus::Training),
            (Connection::Quickplay, SceneStatus::Training),
            (Connection::Local, SceneStatus::Inactive),
            (Connection::Arena, SceneStatus::Inactive),
            (Connection::Quickplay, SceneStatus::Inactive),
        ];
        let left = [
            (Connection::Offline, SceneStatus::Inactive),
            (Connection::Offline, SceneStatus::Singles),
            (Connection::Offline, SceneStatus::Doubles),
            (Connection::Local, SceneStatus::Singles),
            (Connection::Local, SceneStatus::Doubles),
            (Connection::Arena, SceneStatus::Singles),
            (Connection::Arena, SceneStatus::Doubles),
            (Connection::Quickplay, SceneStatus::Singles),
            (Connection::Quickplay, SceneStatus::Doubles),
        ];
        for (connection, status) in right {
            assert!(on_right(true, connection, status));
            assert!(!on_right(false, connection, status));
        }
        for (connection, status) in left {
            assert!(!on_right(true, connection, status));
            assert!(!on_right(false, connection, status));
        }
    }

    #[test]
    fn origin_tracks_the_viewport_and_each_visible_window_width() {
        for viewport in [720.0_f32, 1280.0, 1366.0, 1600.0, 1920.0, 2560.0] {
            let margin = margin_for(viewport);
            assert_eq!(margin, HUD_MARGIN);
            for design in [300.0_f32, 380.0] {
                let configured = design * (viewport / 1920.0);
                let x = origin_x(true, viewport, None, configured, margin);
                near(x, (viewport - configured - margin).max(0.0));
                assert!(x + configured + margin <= viewport + 0.02);
            }
        }
        assert_eq!(origin_x(false, 1280.0, Some(200.0), 200.0, HUD_MARGIN), 0.0);
    }

    #[test]
    fn a_measured_width_replaces_the_configured_width() {
        near(origin_x(true, 1600.0, Some(500.0), 300.0, 10.0), 1090.0);
        near(origin_x(true, 1600.0, Some(f32::NAN), 300.0, 10.0), 1290.0);
        near(origin_x(true, 1600.0, Some(0.0), 280.0, 10.0), 1310.0);
        near(origin_x(true, 1600.0, Some(-4.0), 280.0, 10.0), 1310.0);
    }

    #[test]
    fn unusable_viewport_or_width_stays_at_the_left_edge() {
        assert_eq!(margin_for(f32::NAN), 0.0);
        assert_eq!(margin_for(0.0), 0.0);
        assert_eq!(origin_x(true, f32::NAN, None, 300.0, HUD_MARGIN), 0.0);
        assert_eq!(origin_x(true, 0.0, None, 300.0, HUD_MARGIN), 0.0);
        assert_eq!(origin_x(true, 1280.0, None, f32::NAN, HUD_MARGIN), 0.0);
        assert_eq!(origin_x(true, 100.0, None, 400.0, HUD_MARGIN), 0.0);
        assert_eq!(origin_x(true, 200.0, Some(190.0), 40.0, -8.0), 10.0);
    }
}
