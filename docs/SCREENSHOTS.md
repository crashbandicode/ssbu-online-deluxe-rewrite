# Screenshots

Captured on 2026-10-14 in Eden (GIF-recording build, isolated test profile)
with the 0.4.2 NRO; the game renders at 1280×720 in these runs. Views are
described in [Controls](CONTROLS.md) and [Debugging](DEBUGGING.md). Raw
captures and the preparation script live in the operator's evidence root
(`docs-media-1014/`).

## The window

| | |
| --- | --- |
| ![Settings window on the main menu](media/settings-view.png) <br> The Player Info window (Settings view) on the main menu. | ![Performance view](media/performance-view.png) <br> The Performance view. |
| ![Settings window close-up](media/settings-window.png) <br> Settings rows: Name, Interface, NetLatency, NetProfile, Status, FX, Stealth, Streamer, FPS++, then the live Resolution, DRS, Backoff, Memory and FPS rows. | ![Debug window close-up](media/debug-window.png) <br> Debug rows, including `Timing`, `Capture L+R+Up (30 s)`, `CSS capture` and `Match capture`. |
| ![Debug view](media/debug-view.png) <br> The Debug view on the main menu: build, renderer, FX tracker, capture rows. | ![Streamer on](media/streamer-on.png) <br> Streamer on in the Settings window at the main menu; the NetProfile row reads `Vanilla*` because the cleanup hold is running Vanilla for the manual LessLag selection. |

![Cycling the overlay views](media/overlay-cycle.gif)

Cycling the overlay with L+R+Down (Settings, Performance, hidden) and opening
Debug with L+R+Plus.

![Editing settings in the window](media/settings-edit.gif)

Editing in the Settings window: Up/Down selects the row, Left/Right changes
NetProfile and NetLatency.

## Character select and Training

| | |
| --- | --- |
| ![Training character select with the native status line](media/training-css.png) <br> The native status line at the top of the character select: `You: LessLag \| Auto` (profile and input buffer). | ![Regular character select after Training](media/smash-css-after-training.png) <br> The regular character select entered after a Training session: the status line still reads `LessLag \| Auto`, because the game's training flag is still set there and the scene stays classified Training. The `Vanilla*` hold applies after a Singles or Doubles match. |
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
| ![Match with the Settings strip](media/match-settings-strip.png) <br> Dark Samus vs a level 9 CPU on Final Destination with the Settings strip open: `>Net:Auto Profile:lesslag FX:On Stealth:Off Stream:Off FPS++:Off` and the frame-time line. | ![Match with the Performance strip](media/match-performance.png) <br> The Performance view in the same match. |
| ![Debug table over the pause menu](media/match-debug.png) <br> The Debug view in a match; the Plus in the chord also opens the game's pause menu. | ![Results after quitting](media/results-quit.png) <br> The results screen after quitting through the pause menu. |

![Match with the compact strip](media/match-strip.gif)

Six seconds of the match with the strip open.

![Character select after the match](media/smash-css-after-match.png)

The regular character select right after the match: the status line reads
`You: Vanilla* | Auto`. The manual LessLag selection is held at Vanilla until
the next match or a new selection, and the banner says so.
