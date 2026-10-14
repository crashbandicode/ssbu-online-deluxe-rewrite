# Rendering profiles

**Auto is the default.** It uses Vanilla in menus, offline matches and Training;
online singles use LessLag on console or LLUltra on emulator; online matches with
more than two fighters use LLDoubles. Local wireless counts as online here.

| Profile | Active textures | Presentation policy | Console resolution / DRS | Emulator resolution / DRS |
| --- | --- | --- | --- | --- |
| Vanilla | 3 | TwoBehind | 1080p / off | 1080p / off |
| LessLag | 2 | OneBehind, presents at the flush (0.4.0) | 1080p / on | 1080p / off |
| LLUltra | 2 | Immediate, earlier flush | 576p / on | 1080p / off |
| LLDoubles | 3 | OneBehind | 1080p / on | 1080p / on |

## How Auto knows it is online

The online context comes from the game's scene entries (main menu, Online →
Smash, background matchmaking, arena, local wireless, stage setup, character
select). Main-menu entry keeps a stored Quickplay, because background
matchmaking passes through the main menu and starts the match from there
(0.4.2). A Quickplay that reaches a Singles/Doubles stage without the game's
online-latency hook firing within five seconds is demoted to Offline. A match
the online-latency hook reaches while the scene hooks still say Offline counts
as online for Auto (0.4.1). Event log code 107 records both decisions; see
[Debugging](DEBUGGING.md).

## Present at the flush (0.4.0)

Two-image LessLag presents the acquired image at the end of the flush that
submits it and skips the game's later present of that index. The console trace
behind it: the game presented a frame only after running the next frame's whole
CPU update, 13.7 ms after submitting it, and in a two-image ring a present after
the vsync costs the next acquire a full vsync. On the console the
character-select hand sweep went from 49.9 fps with 20 % of frames over 20 ms
to 59.85 fps with 1.3 %; online matches run at 59.7–59.9 fps.
`renderer-status.txt` counts `early_presents`, `early_present_skips` and
`early_present_mismatches`; a mismatch is event log code 105. Evidence is in
[validation](VALIDATION.md) ("Present-at-flush candidate" and "Console
verdict and 0.4.0").

## Dynamic resolution

DRS is dynamic resolution scaling during supported expensive visual events.
The resolution row reports the applied internal level, not just a requested
preset. In the console-policy Eden test, full Gigaflare lowered LessLag and
LLDoubles from 1080p to 720p, and LLUltra from 576p to 480p, then restored them.
Vanilla kept DRS off. This verifies the trigger and policy, not Switch performance.
Luigi's close-range Up-B cut-in also lowered LLDoubles from 1080p to 720p and
restored it in Eden.

## Manual selection and the cleanup hold

Manual selection works during play and remains saved when Auto is selected.
The renderer reports **Active** only after its guarded transition has completed.
Match cleanup returns to Vanilla on the inactive scene (the banner and the
settings row read `Vanilla*`); the next match or an explicit selection applies
the saved manual mode again. Auto does not rewrite that saved mode.

The INI-only `lesslag_core` and `doubles_core` modes are component experiments,
require restart and are not substitutes for the full presets.

## FPS++

Upstream's separate **FPS++** option was implemented for investigation, but its
native path sped up the match clock in Eden. The normal build reports it as
**Unsupported** and does not apply it, even if its saved setting is On. It remains
an explicit diagnostic experiment, separate from the working LessLag profiles
and from hardware overclocking. See the
[FPS++ measurements](FPS_BOOST_INVESTIGATION.md).

## Measured results

Build-specific acceptance is recorded in [STATUS](STATUS.md) and
[VALIDATION](VALIDATION.md). Tests have covered the global window plus native
status, standalone and embedded ImGui, Training Modpack, the corrected gamepad
overlay, Elite Legal Stages, persistent settings and linked play.

Earlier builds completed asymmetric LessLag/LLUltra matches and a real
four-fighter Auto/LLDoubles team match with the G&W fix on only one peer.
The newer peer-display build also joined three independent network stations
and exercised peer selection in the lobby. That run ended before a match;
it is not a three-station desync result.

A known emulator issue remains: concurrent instances under host load can hide
ImGui windows with manual LessLag/LLDoubles. Native text remains visible.
Removing the load can restore the windows. The cause is under investigation.

The following **24 response trials belong to `b19b7d94ae136e55`**; newer
display/effect builds do not inherit a new timing measurement, and the LessLag
render path changed in 0.4.0 (present at the flush) after these were taken, so
the LessLag row describes the earlier path. Each cell contains three trials at
the specified injection phase.

| Profile | Normal injection phase, three trials | Later injection phase, three trials |
| --- | --- | --- |
| Vanilla | 6, 6, 6 | 6, 6, 6 |
| LessLag | 3, 3, 3 | 3, 3, 3 |
| LLUltra | 2, 2, 2 | 3, 2, 3 |
| LLDoubles | 4, 4, 4 | 4, 4, 4 |

This supports reductions of 3 frames for LessLag, 3–4 for LLUltra and 2 for
LLDoubles in this captured-response experiment. LLDoubles was measured as a
manual profile in two-fighter Training; it is not a four-fighter timing result.
These are synthetic-input emulator screenshots, not Switch input-to-photon
measurements.

The [frame-measurement methodology](LATENCY_METHODOLOGY.md) defines input
submission, plugin input observation, captured response, phase controls,
consecutive-frame checks and exclusions. [Performance](PERFORMANCE.md)
retains the results; [validation](VALIDATION.md) records build-specific
compatibility and linked-match evidence.

Client-side rendering changes avoid gameplay/RNG modifications.
Matching linked-match results are evidence for those trials, not proof that
every online scenario is desync-free. NSO, console timing and four-console
coverage must be reported separately.
