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

Skyline is required. Replace the old Online Deluxe/SsbuSync/PIA-manager stack and
overlapping latency or character-FX plugins. Keep dependencies needed by other
installed mods. Training Modpack and the gamepad overlay remain separate mods.

Deluxe contains its own ImGui backend. When a compatible standalone
`libimgui_smash.nro` is installed, Deluxe reuses that host. Keep the standalone
plugin if **Elite Legal Stages** needs it; Deluxe's embedded fallback does not
replace another plugin's dependency. Compatibility is checked through host exports
and the ImGui ABI, not just the filename. See [ImGui dependencies](docs/IMGUI_DEPENDENCY.md).

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
**settings → performance → hidden**. The small Player Info window works in menus,
character select and matches.

Character select also keeps the fork-style native status at the top after the
window closes. It shows your selected profile and input buffer, plus the selected
opponent's available settings and colored RTT. Edit values in the window;
Streamer restores the game's original title and background. Arena text uses the
fork's room-ID path, but still needs an NSO test. See [native display](docs/NATIVE_DISPLAY.md).

| Control | Action |
| --- | --- |
| Up / Down | Select Peer, NetLatency, NetProfile, FX, Stealth or Streamer |
| Left / Right | Change the selected value |
| Shoulder chord + Left / Right outside settings | Cycle the selected remote peer |
| L+R + either ZL or ZR + Y | Toggle Streamer |
| Plus or Minus held with the window chord | Open the debug view |

The first controller operates the window. Input also reaches the game. If
Elite's menu is open too, D-pad edits can affect both menus; close one before
changing settings. With the
current Eden keyboard mapping, **Q+E+Down Arrow** is the window chord.

NetLatency is the game's online input buffer, **Auto or 0–25**. It is separate
from NetProfile's rendering policy. A menu change is saved immediately; the
game's next relevant buffer hook applies the selected value.

FX **On/Off** affects newly requested, named character effects. Existing effects
expire naturally. Default **On** preserves them. **Reduced is not implemented**.
See [effect coverage and exclusions](docs/FX.md).

Stealth defaults to **On** and suppresses this mod's recognition beacon and
custom settings payload. Its value is latched for the connection. A change
during a connection shows a reconnect notice and takes effect on the next one.
It does not promise anonymity or detection avoidance by Nintendo.

Streamer hides remote names, connection types, settings and ping; it does not
change network traffic. Peer RTT uses the game's existing measurement and the
fork's stability colors. Missing or stale data displays `---`. Interface shows
local Wi-Fi/Ethernet/Unknown and the selected peer's announced type, if available.
An emulator's reported interface can be virtual; it does not identify the host
computer's physical network connection.

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

Upstream's separate **FPS++** option is not implemented yet. A live debugger
trace identifies it as an emulator composition-timing extension through the
native-window swap interval, separate from hardware overclocking. Its additional
latency benefit has not been measured in this rewrite. See the
[FPS++ investigation](docs/FPS_BOOST_INVESTIGATION.md).

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

Current candidate `4a927dc20904722d` passes **192 host tests** and all required
host/Skyline checks. It restores the fork's native CSS display alongside Player
Info. Tests covered edits, closing the window, Streamer restoration, CSS
re-entry, a completed offline match on the preceding CSS-equivalent build,
and bidirectional peer names/profiles/colored RTT in linked CSS. Solo tests
showed both Deluxe and Elite windows with the standalone ImGui host, including
with the G&W fix loaded.

One concurrent two-instance run had invisible external ImGui windows on the
full-stack peer; its native header and the other peer's embedded window worked.
This did not recur in solo tests with or without G&W. The cause remains
unresolved. Arena hooks are implemented but await a Switch NSO test. See
[native display validation](docs/NATIVE_DISPLAY.md) for the exact scope.

The preceding candidate `d9acd03611e1d1d6` passed **176 host tests**, host and Skyline
checks with warnings denied, the no-default-features build and the optimized
Skyline build. It passed fresh Stealth On suppression, Off after reconnect,
exact menu-saved INI reload, Streamer privacy, remote buffer/profile/colored RTT,
Auto singles selection and a linked LessLag G&W match through results and CSS.
The G&W fix was on one peer only. Elite was enabled alongside the standalone
ImGui host, Training Modpack and corrected gamepad overlay; the second peer
used Deluxe's embedded host.

The preceding `b19b7d94ae136e55` candidate also completed a four-fighter
Auto/LLDoubles match against a clean peer, a G&W LLUltra match, cut-in/Gigaflare
resolution recovery and the title crop regression. The d9 candidate changed
only four display/test files: the remote buffer cell and compact Auto labels.
New rendering modes have not been validated on Switch.

In 24 controlled Eden crouch trials on **b19**, the first captured response
arrived after these frame counts:

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

Prepare the native dependency and build/package the Switch plugin:

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
