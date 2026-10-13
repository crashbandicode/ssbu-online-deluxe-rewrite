# Online Deluxe rewrite

A private Rust/Skyline reconstruction for **Super Smash Bros. Ultimate 13.0.5**,
delivered as **one NRO**. It combines the rendering policies, global Player Info
window, native character-select status, network display and scoped character-FX controls. No overclocking behavior
is included.

The preserved fork and predecessor references are described in
[reference/README.md](reference/README.md). See the [parity ledger](docs/FORK_PARITY.md)
for deliberate differences and unfinished features.

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
| L+R + either ZL or ZR + Y | Toggle Streamer |
| L+R (or ZL+ZR) + Plus or Minus | Open the debug view; again to hide it |

The window commands fire when the whole combination is held, in any press
order, and once per press: L+R+Down works whether the bumpers or the D-pad
land first. Holding Plus or Minus while tapping Down with the chord also opens
Debug, as before.

The first controller operates the window. Input also reaches the game. If
Elite's menu is open too, D-pad edits can affect both menus; close one before
changing settings. With the
current Eden keyboard mapping, **Q+E+Down Arrow** is the window chord.

NetLatency is the game's online input buffer, **Auto or 0–25**. It is separate
from NetProfile's rendering policy. A menu change is saved immediately; the
game's next relevant buffer hook applies the selected value.

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
covered by host tests; emulator and hardware behaviour are not yet verified.

## Rendering profiles

**Auto is the default.** It uses Vanilla in menus, offline matches and Training;
online singles use LessLag on console or LLUltra on emulator; online matches with
more than two fighters use LLDoubles. Local wireless counts as online here.

| Profile | Active textures | Presentation policy | Console resolution / DRS | Emulator resolution / DRS |
| --- | --- | --- | --- | --- |
| Vanilla | 3 | TwoBehind | 1080p / off | 1080p / off |
| LessLag | 2 | OneBehind | 1080p / on | 1080p / off |
| LLUltra | 2 | Immediate, earlier flush | 576p / on | 1080p / off |
| LLDoubles | 3 | OneBehind | 1080p / on | 1080p / on |

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
`Armed` until the gate first passes; install log code 102 with `a=5` marks
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
display/effect builds do not inherit a new timing measurement. Each cell contains
three trials at the specified injection phase.

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
  config.ini
  config.previous.ini
```

Startup checkpoints show which hooks initialized. Events are bounded; status
records requested and committed rendering state. Panic logs exist only when a
panic was captured. An abrupt native crash may require Atmosphère's crash report
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
