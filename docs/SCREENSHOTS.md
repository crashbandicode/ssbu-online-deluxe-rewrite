# Screenshots

Captured on 2026-10-14 in Eden (GIF-recording build, isolated test profile)
with the 0.5.0 NRO and the [FX Control](https://git.syntaxbyexample.com/intpa/ssbu-fx-control)
0.1.0 plugin installed next to it; the game renders at 1280×720 in these
runs. Views are described in [Controls](CONTROLS.md) and
[Debugging](DEBUGGING.md). Raw captures and the preparation script live in
the operator's evidence root (`docs-media-050/`; the gamepad-overlay captures
are from `docs-media-1014/`).

## The window

| | |
| --- | --- |
| ![Settings window on the main menu](media/settings-view.png) <br> The Player Info window (Settings view) on the main menu. | ![Performance view](media/performance-view.png) <br> The Performance view. |
| ![Settings window close-up](media/settings-window.png) <br> Settings rows: Name, Interface, NetLatency, NetProfile, Status, Stealth, Streamer, FPS++, then the live Resolution, DRS, Backoff, Memory and FPS rows. Character FX is no longer a Deluxe row. | ![Debug window close-up](media/debug-window.png) <br> Debug rows, including `Timing`, `Capture L+R+Up (30 s)`, `CSS capture` and `Match capture`. |
| ![Debug view](media/debug-view.png) <br> The Debug view on the main menu: build, renderer and capture rows. | ![Streamer on](media/streamer-on.png) <br> Streamer on in the Settings window at the main menu (NetProfile LessLag, Status Active). |
| ![FX Control page](media/fx-control-view.png) <br> A plugin's own page in the window cycle: the FX Control plugin's page (Scope, All, one row per covered fighter, EFF variants), one D-pad Down after Settings. | ![FX Control page close-up](media/fx-control-window.png) <br> The same page up close. |

![Cycling the overlay views](media/overlay-cycle.gif)

Cycling the overlay with L+R+Down (Settings, the FX Control plugin page,
Performance, hidden) and opening Debug with L+R+Plus.

![Editing settings in the window](media/settings-edit.gif)

Editing in the Settings window: Up/Down selects the row, Left/Right changes
NetProfile and NetLatency.

![Editing a plugin page](media/fx-control-edit.gif)

Editing on a plugin page: a fighter row goes On → Off → Reduced and back; the
All row reads `Unavailable` while the fighter rows differ.

## Character select and Training

| | |
| --- | --- |
| ![Training character select with the native status line](media/training-css.png) <br> The native status line at the top of the Training character select (profile and input buffer). In this capture, taken after a Training session, it reads `You: Vanilla* \| Auto`: the cleanup hold was running Vanilla for the saved LessLag selection at that moment. The same screen read `LessLag \| Auto` after another session, so the hold is not a fixed Training behaviour. | ![Regular character select after Training](media/smash-css-after-training.png) <br> The regular character select entered after that Training session: the status line reads `LessLag \| Auto`. |
| ![Debug view during Training](media/debug-training.png) <br> Training with the Debug view open: `Match capture: Armed; 1 capture(s)` and `Timing: Capturing` while the match capture streams. | ![Main menu](media/main-menu.png) <br> The main menu with the window hidden. |

![Training with the Player Info window](media/training-strip.gif)

A few seconds of Training with the Player Info window on the right.

## In a match

In a running Singles or Doubles match the window becomes the compact two-row
strip at the top left: one row of `Label:Value` fields, one row with FPS,
frame time and the frame-time bar.

![Compact strip close-up](media/match-strip-closeup.png)

| | |
| --- | --- |
| ![Match with the Settings strip](media/match-settings-strip.png) <br> A Singles match vs a level 9 CPU on Final Destination with the Settings strip open: `>Net:Auto Profile:lesslag Stealth:Off Stream:Off FPS++:Off` and the frame-time line. | ![Match with the Performance strip](media/match-performance.png) <br> The Performance view in the same match. |
| ![Debug table over the pause menu](media/match-debug.png) <br> The Debug view in a match; the Plus in the chord also opens the game's pause menu. | ![Results after quitting](media/results-quit.png) <br> The results screen after quitting through the pause menu. |

![Match with the compact strip](media/match-strip.gif)

Six seconds of the match with the strip open.

![Character select after the match](media/smash-css-after-match.png)

The regular character select right after the match: the status line reads
`You: Vanilla* | Auto`. The manual LessLag selection is held at Vanilla until
the next match or a new selection, and the banner says so.
