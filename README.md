# Online Deluxe rewrite

A private Rust/Skyline reconstruction for **Super Smash Bros. Ultimate 13.0.5**,
delivered as **one NRO**. It combines the rendering policies, global Player Info
window, native character-select status, network display and scoped character-FX controls. No overclocking behavior
is included.

The preserved fork and predecessor references are described in
[reference/README.md](reference/README.md). See the [parity ledger](docs/FORK_PARITY.md)
for deliberate differences and unfinished features.

**Current release: 0.4.2** (tags `v0.4.0`–`v0.4.2`; every release carries the
NRO and its manifest). 0.4.0 made two-image LessLag present at the flush,
which removed the character-select stall on the console, added the `Vanilla*`
banner label and the opt-in match capture; 0.4.1 made native character-select
edits require ZL, dropped the opponent from the banner when they decline a
rematch, and let Auto recognise a match the scene hooks missed; 0.4.2 keeps
Quickplay across the main menu. Details and evidence are in
[validation](docs/VALIDATION.md).

## Screenshots

Captured on 2026-10-14 in Eden (GIF-recording build, isolated test profile)
with the 0.4.2 NRO; the game renders at 1280×720 in these runs. Details for
each view are in [Controls](#controls) and [Debugging](docs/DEBUGGING.md).

| | |
| --- | --- |
| ![Settings window on the main menu](docs/media/settings-view.png) <br> The Player Info window (Settings view) on the main menu. | ![Debug view](docs/media/debug-view.png) <br> The Debug view: build, renderer, FX tracker, capture rows. |
| ![Settings window close-up](docs/media/settings-window.png) <br> Settings rows: Name, Interface, NetLatency, NetProfile, Status, FX, Stealth, Streamer, FPS++, then the live Resolution, DRS, Backoff, Memory and FPS rows. | ![Debug window close-up](docs/media/debug-window.png) <br> Debug rows, including `Timing`, `Capture L+R+Up (30 s)`, `CSS capture` and `Match capture`. |
| ![Training character select with the native status line](docs/media/training-css.png) <br> The native status line at the top of the character select: `You: LessLag \| Auto` (profile and input buffer). | ![Debug view during Training](docs/media/debug-training.png) <br> Training with the Debug view open: `Match capture: Armed; 1 capture(s)` and `Timing: Capturing` while the match capture streams. |

![Cycling the overlay views](docs/media/overlay-cycle.gif)

Cycling the overlay with L+R+Down (Settings, Performance, hidden) and opening
Debug with L+R+Plus.

![Editing settings in the window](docs/media/settings-edit.gif)

Editing in the Settings window: Up/Down selects the row, Left/Right changes
NetProfile and NetLatency.

![Training with the Player Info window](docs/media/training-strip.gif)

A few seconds of Training with the Player Info window on the right.

| | |
| --- | --- |
| ![Regular character select after Training](docs/media/smash-css-after-training.png) <br> The regular character select entered after a Training session: the status line still reads `LessLag \| Auto`, because the game's training flag is still set there and the scene stays classified Training. The `Vanilla*` hold applies after a Singles or Doubles match. | ![Streamer on](docs/media/streamer-on.png) <br> Streamer on in the Settings window at the main menu; the NetProfile row reads `Vanilla*` because the cleanup hold is running Vanilla for the manual LessLag selection. |

## Install

Close Smash before replacing files. Copy the rewrite NRO to:

```text
sd:/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libssbu_online_deluxe_rewrite.nro
```

Skyline is required. This NRO implements the sync/render transitions and the
peer-manager handshake itself (`src/render_policy.rs`, `src/runtime/resolution.rs`,
`src/peer_transport.rs`, `src/runtime/pia_rtt.rs`). Leave out the old Online
Deluxe, SsbuSync, and PIA-manager plugins: if SsbuSync exports are already
present, the rewrite skips its hooks (`src/runtime/mod.rs` `legacy_host_present`).
Replace overlapping latency or character-FX plugins. Keep dependencies needed by
other installed mods. Training Modpack and the gamepad overlay remain separate mods.

The menu statically links `libimgui_smash.a`, built from pinned public source
with our renderer patch. The source, build recipe and digest are recorded in
[ImGui dependencies](docs/IMGUI_DEPENDENCY.md). When a compatible standalone
`libimgui_smash.nro` is installed, Deluxe reuses that host.
Keep the standalone plugin if **Elite Legal Stages** needs it; Deluxe's embedded
copy does not replace another plugin's dependency. Compatibility is checked
through host exports and the ImGui ABI, not just the filename.
When the standalone host is active, its renderer owns the GPU buffers; updating
Deluxe alone does not update that backend. Use the matching patched standalone
host to receive the renderer fix in an Elite installation.

Elite's stage filtering applies to **NSO**, not offline matches. Offline stages
remaining unrestricted is expected. Emulator coexistence tests do not validate
NSO matchmaking or its stage filter.

For the normal Eden profile, the plugin directory is:

```text
/home/intpa/.local/share/eden/sdmc/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/
```

The automated test profiles under `eden-cli-handoff` are separate from that path.

## Controls

Hold **L+R** or **ZL+ZR**, then tap **D-pad Down** to cycle
**settings → performance → hidden**. When a separately installed plugin NRO has
registered options, a **plugins** page sits between settings and performance
(see [Plugins](#plugins)). The small Player Info window works in menus,
character select and matches. It moves to the right in Training and connected
lobbies, keeping the pause controls visible, and stays left in regular matches.

Character select also keeps the fork-style native status at the top after the
window closes. It shows your selected profile and input buffer, plus the selected
opponent's available settings and colored RTT. Edit values in the window;
Streamer restores the game's original title and background. Arena text uses the
fork's room-ID path, but still needs an NSO test. See [native display](docs/NATIVE_DISPLAY.md).

| Control | Action |
| --- | --- |
| Up / Down | Select Peer, NetLatency, NetProfile, FX, FPS++, Stealth or Streamer (or a plugin row on the plugins page) |
| Left / Right | Change the selected value |
| Shoulder chord + Left / Right outside settings | Cycle the selected remote peer |
| ZL + Left / Right or Up / Down on an online character select or arena pane, window closed | Edit the input buffer or the profile on the native status (since 0.4.1 the D-pad alone only moves the hand) |
| L+R + either ZL or ZR + Y | Toggle Streamer |
| L+R (or ZL+ZR) + Plus or Minus | Open the debug view; again to hide it |

The window commands fire when the whole combination is held, in any press
order, and once per press: L+R+Down works whether the bumpers or the D-pad
land first. Holding Plus or Minus while tapping Down with the chord opens Debug
from Settings or Plugins and keeps Debug open; from hidden the chord still
opens Settings and from Performance it still hides the window.

The first controller operates the window. Input also reaches the game. If
Elite's menu is open too, D-pad edits can affect both menus; close one before
changing settings. With the
current Eden keyboard mapping, **Q+E+Down Arrow** is the window chord.

NetLatency is the game's online input buffer, **Auto or 0–25**. It is separate
from NetProfile's rendering policy. A menu change takes effect at once and is
written to `config.ini` by the background thread within about half a second;
the game's next relevant buffer hook applies the selected value.

FX **On / Off / Reduced** affects newly requested, named character effects.
Existing effects expire naturally. **On** is the default. **Reduced** keeps
recognizable cues while removing selected extra layers; supported asset variants
keep fewer particle emitters. Unsupported effects retain their original visuals.
This is visual coverage, not a claim that every move now runs faster.
See [effect coverage, dependencies and exclusions](docs/FX.md).

Stealth defaults to **On** and suppresses this mod's recognition beacon and
custom settings payload. Its value is latched for the connection. A change
during a connection shows a reconnect notice and takes effect on the next one.
It does not promise anonymity or detection avoidance by Nintendo.

Streamer hides remote names, connection types, settings and ping; it does not
change network traffic. Peer RTT uses the game's existing measurement and the
fork's stability colors. The opponent column appears only for a connected,
identified station; a connected peer with unknown RTT still appears with `---`.
Interface shows
local Wi-Fi/Ethernet/Unknown and the selected peer's announced type, if available.
An emulator's reported interface can be virtual; it does not identify the host
computer's physical network connection.

## Plugins

Other mods can add their own rows without writing menu code. A separately
built Skyline NRO resolves the exported `ssbu_online_deluxe_plugin_api_v1`
symbol at runtime and registers a plugin label plus one or more options, each
with a finite list of choices (for example `Record: Off / On`). Deluxe draws
them on the plugins page: a vertical table outside matches and the compact
two-row strip during a ready match, with paging for long lists. Deluxe asks
the plugin for the current value every frame and sends Left/Right as a
request; the plugin decides whether it applies now, later, or not at all, and
keeps its own saved settings. A plugin can colour its values (green, yellow,
red) per choice and mark a row read-only, so a status such as `Health: Failed`
shows in red on both layouts without being editable. A plugin that declares its own `key=value` INI
path gets that file updated by Deluxe after each accepted menu edit, so the
plugin only has to read it at startup. Deluxe's `config.ini` is unchanged. The ABI,
Rust SDK and a demo NRO live under [`sdk/`](sdk/README.md). The page is
covered by host tests and was verified in Eden on 2026-10-12 (Plugins page,
pending requests, tones; see [validation](docs/VALIDATION.md)); hardware
behaviour is not yet observed.

## Rendering profiles

**Auto is the default.** It uses Vanilla in menus, offline matches and Training;
online singles use LessLag on console or LLUltra on emulator; online matches with
more than two fighters use LLDoubles. Local wireless counts as online here.
The online context comes from the game's scene entries: main-menu entry keeps
a stored Quickplay (background matchmaking passes through the main menu, 0.4.2),
a Quickplay that reaches a match without the online-latency hook is demoted
to Offline five seconds in, and a match the online-latency hook reaches while
the scene hooks still say Offline counts as online (0.4.1).

| Profile | Active textures | Presentation policy | Console resolution / DRS | Emulator resolution / DRS |
| --- | --- | --- | --- | --- |
| Vanilla | 3 | TwoBehind | 1080p / off | 1080p / off |
| LessLag | 2 | OneBehind, presents at the flush (0.4.0) | 1080p / on | 1080p / off |
| LLUltra | 2 | Immediate, earlier flush | 576p / on | 1080p / off |
| LLDoubles | 3 | OneBehind | 1080p / on | 1080p / on |

Since 0.4.0 two-image LessLag presents the acquired image at the end of the
flush that submits it and skips the game's later present of that index. The
console trace behind it: the game presented a frame only after running the
next frame's whole CPU update, 13.7 ms after submitting it, and in a two-image
ring a present after the vsync costs the next acquire a full vsync. On the
console the character-select hand sweep went from 49.9 fps with 20 % of frames
over 20 ms to 59.85 fps with 1.3 %; online matches run at 59.7–59.9 fps.
`renderer-status.txt` counts `early_presents`, `early_present_skips` and
`early_present_mismatches`; a mismatch is event log code 105.

DRS is dynamic resolution scaling during supported expensive visual events.
The resolution row reports the applied internal level, not just a requested
preset. In the console-policy Eden test, full Gigaflare lowered LessLag and
LLDoubles from 1080p to 720p, and LLUltra from 576p to 480p, then restored them.
Vanilla kept DRS off. This verifies the trigger and policy, not Switch performance.
Luigi's close-range Up-B cut-in also lowered LLDoubles from 1080p to 720p and
restored it in Eden.

Manual selection works during play and remains saved when Auto is selected.
The renderer reports **Active** only after its guarded transition has completed.
Match cleanup returns to Vanilla; the next match or an explicit selection can
apply the saved manual mode again. Auto does not rewrite that saved mode.

The INI-only `lesslag_core` and `doubles_core` modes are component experiments,
require restart and are not substitutes for the full presets.

Upstream's separate **FPS++** option was implemented for investigation, but its
native path sped up the match clock in Eden. The normal build reports it as
**Unsupported** and does not apply it, even if its saved setting is On. It remains
an explicit diagnostic experiment, separate from the working LessLag profiles
and from hardware overclocking. See the
[FPS++ measurements](docs/FPS_BOOST_INVESTIGATION.md).

## Dispatcher backoff

The game's task dispatcher polls three local and three shared job queues in a
loop while it waits for a dependency. When every queue is empty and the awaited
job is still running, it does not wait on its event; it rescans immediately.
On the character select the main thread spends about 5 ms of every frame in
that loop while one UI layout job runs on a worker, and under CPU pressure the
two-image LessLag ring cannot hide the resulting variance (Switch: 48–50 fps on
the CSS, 60 in matches).

This build adds one inline hook at the point where all six probes have failed.
When the dispatcher is the main thread, every pending record is still blocked,
the queues are empty and no fiber is ready to resume, the thread sleeps briefly
before the game's own rescan. No job, counter, lock or ordering changes; the
hook only decides whether to wait before repeating. It is installed for every
rendering profile and scene because the waste is in game code and exists in
Vanilla too.

Eden A/B/A with a 175% CPU quota on the CSS: LessLag 52 → 59 fps and 18% less
CPU per frame; Vanilla and LLDoubles kept 60 with the same CPU saving; a live
match lost no frames. Those are emulator results; the hardware benefit is
measured separately (see [validation](docs/VALIDATION.md)). The Performance
view shows a **Backoff** row: sleeps per second and the last sleep length,
`Idle` when nothing is waiting, `External` when another plugin already owns
the hook site, `Unavailable` when the game image does not match 13.0.5, and
`Off` when disabled.

Tuning is optional. Create `sd:/ultimate/ssbu_online_deluxe_rewrite/dispatcher_backoff.ini`:

```ini
# idle passes to spin before the first sleep (default 16)
spin_passes=16
# sleep on every Nth idle pass after that (default 1)
every_passes=1
# first sleep and the cap of the doubling ramp, nanoseconds (default 5000 / 50000)
base_ns=5000
max_ns=50000
# minimum gap between sleeps in microseconds (default 0 = none)
cooldown_us=0
# 1 lets worker dispatchers sleep too (default 0: MainThread only)
all_threads=0
# preset=measured reproduces the Eden diagnostic cadence (1 us once per 171 passes)
```

`DISABLE_BACKOFF` in the data directory turns the hook off for an A/B. The same
fix ships alone as the
[ssbu-dispatcher-backoff](https://git.syntaxbyexample.com/intpa/ssbu-dispatcher-backoff)
plugin; exactly one of the two installs the hook, whichever loads first, and
the other reports `External`.

Since 0.3.1 the hook also has an eligibility gate: it never sleeps before one
of the scene hooks has fired (main menu, Quickplay, Arena, local wireless,
stage setup or the character select), and never while the tracked window has
not presented for 250 ms. Boot, the logo/movie/title sequence and a stalled
renderer therefore run the game's original loop. The Backoff row reads
`Armed` until the gate first passes; event log code 102 with `a=5` marks
that moment. If the scene hooks fail their own checks the gate stays closed.

### Character-select capture (opt-in)

To locate the remaining hardware character-select cost, create an empty file
`sd:/ultimate/ssbu_online_deluxe_rewrite/CAPTURE_CSS` before boot. The build
then installs three extra read-only hooks (dispatcher job registration,
dependency wait, the UI layout job) and arms a 30 s timing capture
automatically each time the character select is entered: leave the hand still
for the first ten seconds, then move it continuously. The `api-<tick>.csv`
gains per-frame backoff sleep totals (kinds 60/61), one read-only priority and
core-mask row per thread (62/63), the UI job's dispatch (64), every dependency
wait with its counter and thread (66) and the UI job span (77). Nothing is
changed: no priority, affinity, clock or task order. Without the flag none of
these hooks exist. The Debug view's `CSS capture` row shows the state, and
`tools/analyze_render_trace.py` prints a `css_capture` section. The manual
L+R+Up capture also lasts 30 s now.

### Match capture (opt-in)

To see what a whole match does, create an empty file
`sd:/ultimate/ssbu_online_deluxe_rewrite/CAPTURE_MATCH` before boot. The same
capture pair (`timing-<tick>.csv`, `api-<tick>.csv`) is then armed when a
stage loads (Singles, Doubles or Training) and ends when the match leaves the
stage, or after ten minutes. Both files stream to the card while the match
runs, about 4 MB per minute, from the background thread; nothing is written
from a game hook. The flag also installs the three read-only `CAPTURE_CSS`
hooks so dependency waits and the UI job appear in the trace; with both flags
on the card the character-select capture runs as well. The Debug view's
`Match capture` row counts the captures armed and ended.

## Configuration

Settings are created at:

```text
sd:/ultimate/ssbu_online_deluxe_rewrite/config.ini
```

Default configuration:

```ini
version=1
effects=on
effects_scope=online
input_buffer=auto
render_mode=vanilla
auto_profile=true
platform=console
stealth=true
streamer=false
fps_boost=false
peer_observer=false
```

Use the explicit `platform=emulator` setting in Eden. For offline FX testing,
also set `effects_scope=all`. The [Eden test configuration](config/eden-test.ini)
contains both overrides. The default `online` scope applies the FX policy only
in recognized online contexts.

Menu edits apply live as described above and persist automatically. External
INI edits require restarting Smash. `config.previous.ini` retains the previous
valid configuration for recovery. An explicit `stealth=false` stays false;
missing Stealth settings default to true.

## Validation and frame measurements

Build-specific acceptance is recorded in [STATUS](docs/STATUS.md) and
[VALIDATION](docs/VALIDATION.md). Tests have covered the global window plus native
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

The [frame-measurement methodology](docs/LATENCY_METHODOLOGY.md) defines input
submission, plugin input observation, captured response, phase controls,
consecutive-frame checks and exclusions. [Performance](docs/PERFORMANCE.md)
retains the results; [validation](docs/VALIDATION.md) records build-specific
compatibility and linked-match evidence.

Client-side rendering and effect changes avoid gameplay/RNG modifications.
Matching linked-match results are evidence for those trials, not proof that
every online scenario is desync-free. NSO, console timing and four-console
coverage must be reported separately.

## Diagnostics

Retain the exact NRO and matching debug bundle when reporting a crash. Logs live
under:

```text
sd:/ultimate/ssbu_online_deluxe_rewrite/
  install.log
  events.log
  panic.log
  renderer-status.txt
  hang-trace.log
  config.ini
  config.previous.ini
```

Startup checkpoints show which hooks initialized. Events are bounded; status
records requested and committed rendering state. Panic logs exist only when a
panic was captured. `hang-trace.log` (0.3.2) is a 5 s heartbeat of this
plugin's hook call counts, marking any hook that entered and never returned,
with scene changes and first firings stamped by system tick; use it when the
game freezes on a loading screen with no crash report. Since 0.3.6 each
heartbeat also carries the kernel's memory accounting for the process
(`mem_used`, `mem_total`, `mem_free`, `heap_region`, bytes), mirrored by a
`Memory` row in the Debug view, so plugin memory budgets can be measured on
the console and on Eden alike. An abrupt native crash may require Atmosphère's crash report
and the matching optimized ELF. See [debugging](docs/DEBUGGING.md).

## Build

Use the pinned Rust toolchain in `rust-toolchain.toml`, installed cargo-skyline
and the Skyline target. Native ImGui preparation and provenance are documented
in [IMGUI_DEPENDENCY.md](docs/IMGUI_DEPENDENCY.md).

Run each host check separately from the repository root:

```powershell
cargo clippy --all-targets -- -D warnings
cargo build --no-default-features
cargo test
```

The installed cargo-skyline 3.5.0 check wrapper needs two separators to forward
warning flags:

```powershell
cargo skyline clippy -- -- -D warnings
```

With Docker and Git available, build the native dependency from source, then
build and package the Switch plugin:

```powershell
/usr/bin/python3 tools/prepare_imgui.py
/usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -Command '& ./tools/Build.ps1 -Package'
```

Outputs:

```text
target/aarch64-skyline-switch/release/libssbu_online_deluxe_rewrite.nro
dist/<NRO-hash-prefix>/
```

The ignored bundle contains the NRO, optimized ELF, source archive, dependency
provenance and hashes. Preserve each tested bundle. No Nintendo assets or
third-party compiled plugins are committed. Build checks alone do not establish
runtime compatibility or reduced latency.
