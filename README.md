# Online Deluxe rewrite

> AI-assisted, human-directed and reviewed.

Tested in **Eden** and on an **Erista Switch** without issues observed.
Works with [Switch firmware 23.0.1][firmware], [Atmosphère 1.12.1][atmosphere]
and [Skyline beta `2eb226f`][skyline] (latest checked 2026-10-25).
[Validation](docs/VALIDATION.md).

A Rust/Skyline rebuild of Online Deluxe for **Super Smash Bros. Ultimate
13.0.5**, shipped as **one NRO**: lower-latency rendering, a small in-game
Player Info window and the fork's character-select and network display. No
overclocking.

![The Player Info Settings window: the You row with eye and TV icons, NetLatency and NetProfile](docs/media/settings-window.png)

**[0.8.0 — download NRO and manifest](https://git.syntaxbyexample.com/intpa/ssbu-online-deluxe-rewrite/releases/tag/v0.8.0)**
fixes stale `Auto (Nf)` counts with current-match buffer sampling and includes
passive plugin status cells. The new Auto sampling has host/static validation;
live-console validation remains pending. [Status and version history](docs/STATUS.md).

> **Install only this NRO.** It replaces the original Online Deluxe fork,
> SsbuSync and the PIA manager, so remove all three. If SsbuSync is still
> loaded, Deluxe switches its hooks off and you are playing the old fork.
> [Install guide](docs/INSTALL.md)

## What it does

### A window you can open anywhere

Hold **L+R** and tap **D-pad Down** to step through the pages; after the last
page the window hides.

![L+R+Down cycling Settings, the FX Control companion's two pages, Performance and Window Options, then hiding the window](docs/media/overlay-cycle.gif)

### Settings that need ZR

Select a row with **Up/Down**, then hold **ZR** and press **Left/Right**. A
plain D-pad press, a taunt or menu navigation never changes a setting.

![Selecting NetLatency, then ZR+Right changing Auto to 0 and ZR+Left restoring Auto](docs/media/settings-edit.gif)

### Stealth and Streamer on the You row

Select **You**, hold **ZR** (not ZL) and tap **X** for Stealth or **Y** for
Streamer. The eye closes and the TV turns green when they are on.

![ZR+X closing the Stealth eye and ZR+Y turning the Streamer TV green, then both switched off again](docs/media/privacy-shortcuts.gif)

### Horizon-OC only when online (ZR+L3)

With the game-aware Horizon-OC fork, **ZR+L3** on Settings switches when Smash's
saved Horizon-OC profile may apply: **always**, as Horizon-OC itself does, or
only while you are **online**. In online mode a clock follows the privacy icons:
green while the profile is applied, grey while it is held back.

![ZR+L3 switching to always, which removes the clock, and back to online, where the grey clock returns](docs/media/oc-indicator.gif)

These three clips are from Eden with the test-only
[hoc-standin](tools/hoc-standin/README.md) answering for Horizon-OC, offline,
so the clock stays grey; no clock was changed.

### A compact strip in matches

In ordinary matches the window becomes a two-row strip at the top left.

![The in-match strip: You with privacy icons, Net and Profile, then the timing values](docs/media/match-strip-closeup.png)

On character select, the fork's status line keeps your profile and input
buffer, plus the opponent's settings and coloured ping while their station is
live. [Native display](docs/NATIVE_DISPLAY.md)

### Lower-latency rendering

Vanilla, LessLag, LLUltra and LLDoubles trade buffered frames for latency, and
Auto picks one per scene. The default is **Vanilla+**. See the
[profile table](#rendering-profiles) and the
[measured frames](#measured-input-delay).

### Plugin pages

Other mods can add rows or a whole page through the exported API and
[Rust SDK](sdk/README.md). The separate
[FX Control](https://git.syntaxbyexample.com/intpa/ssbu-fx-control) plugin
adds its character-FX pages this way.

![Stepping a Captain Falcon row in FX Control's page from On to Off to Reduced and back to On with ZR+Right](docs/media/fx-control-edit.gif)

[Plugins](docs/PLUGINS.md) explains the API, including a plugin's own ImGui
windows. Also in the NRO: [dispatcher backoff](docs/DISPATCHER_BACKOFF.md) and
opt-in [timing captures](docs/CAPTURES.md).

*These examples are offline Eden captures of 0.7.0 with manual LessLag,
`vanilla_plus=false` and the optional FX Control companion. They show the menu,
not online play, Switch performance or FX's gameplay effects. More in
[Screenshots](docs/SCREENSHOTS.md); sources in
[Media provenance](docs/MEDIA_PROVENANCE.md).*

## Optimizations added by this rewrite

Two things the original Online Deluxe does not do; both have an INI switch
(`dispatcher_backoff`, `early_present`, default on). Basics first:
[How Deluxe LessLag and LLUltra work](docs/LESSLAG_LLULTRA.md).

**Dispatcher backoff.** When the main CPU task keeps finding no work it can do,
the rewrite lets it sleep briefly, with growing pauses up to a small limit, so
other tasks get the core. Locks and checks stay as they are.
The operator also observed its benefit on Switch, alongside the Eden results.
[How it works](docs/DISPATCHER_BACKOFF.md).

![Dispatcher backoff, before and after](docs/media/dispatcher-backoff-before-after.svg)

**Present at the flush.** The game asked for a frame to be shown only after the
next frame's CPU update, which could miss a refresh. The rewrite queues the
presentation as soon as the frame's commands are submitted. It does not make
frames faster to compute; it gives the display more time to show them on
schedule. [How it works](docs/RENDERING.md#present-at-the-flush-040).

![Presenting frame 99 before or after CPU work on frame 100](docs/media/present-at-flush-timeline.svg)

Conceptual timing diagram, not a measured trace.

Before and after in Eden under a CPU quota calibrated to the console's
character-select dip (both switches off on the left, on on the right; LessLag,
Performance view open):

| Scene | Both off (upstream behaviour) | Both on |
| --- | --- | --- |
| Character select, hand sweep | ![Character select, switches off](docs/media/toggles-css-off.gif) | ![Character select, switches on](docs/media/toggles-css-on.gif) |
| Training, LessLag, Pyra Blazing End | ![Training, switches off](docs/media/toggles-effects-off.gif) | ![Training, switches on](docs/media/toggles-effects-on.gif) |

The console went from 49.9 to 59.85 fps on the hand sweep with the 0.4.0 fix.
Emulator clips show the mechanism, not console numbers; the quota, the clip
procedure and the measured values are in
[Mimicking the Switch in Eden](docs/EMULATOR_METHOD.md) and the console results
in [Rendering profiles](docs/RENDERING.md#measured-results).

## Controls

| Press | Effect |
| --- | --- |
| L+R + Down | Next page: Settings → plugin pages → Performance → Window Options → hidden |
| Up / Down | Select a row |
| ZR + Left / Right | Change the selected value (shown as `< value >`) |
| ZR + X / Y, You selected | Stealth / Streamer |
| L+R + X | Toggle Vanilla+; with manual console LLUltra, cycle LLUltra → LLUltra+ → LLUltra++ |
| L+R + Plus or Minus | Show or hide Debug |

Everything else, including 0.6.2's older chords, is in
[Controls](docs/CONTROLS.md); the You-row icons are drawn in
[privacy-indicators.svg](docs/privacy-indicators.svg), and the Horizon-OC
clock that follows them (0.9.0, ZR+L3) in
[oc-indicators.svg](docs/oc-indicators.svg).

## Rendering profiles

| Profile | Textures | Presentation | Console | Emulator |
| --- | --- | --- | --- | --- |
| Vanilla | 3 | TwoBehind; Vanilla+ (the default) presents at the flush | 1080p, DRS off | 1080p, DRS off |
| LessLag | 2 | OneBehind, presents at the flush | 1080p, DRS on | 1080p, DRS off |
| LLUltra | 2 | Immediate, earlier flush, presents at the flush | 576p, DRS on; `ultra_resolution` raises it to 720p or 1080p (0.7.0 menu labels: LLUltra+ / LLUltra++) | 1080p, DRS off |
| LLDoubles | 3 | OneBehind, presents at the flush | 1080p, DRS on | 1080p, DRS on |

Auto uses Vanilla in menus, offline matches and Training; online singles get
LessLag on console or LLUltra on emulator; online matches with more than two
fighters get LLDoubles. How Auto recognises an online match, the cleanup hold
(`Vanilla*`) and DRS are in [Rendering profiles](docs/RENDERING.md).

### Measured input delay

Captured frames from a synthetic input to the first visible response in Eden,
three trials per cell, on the 0.5.2 build (the earlier builds gave the same
normal-phase counts):

| Profile | Normal phase | Later phase | Frames saved vs Vanilla |
| --- | --- | --- | --- |
| Vanilla | 6, 6, 6 | 6, 6, 6 | — |
| LessLag | 3, 3, 3 | 3, 3, 3 | **3** |
| LLUltra | 2, 2, 2 | 2, 3, 3 | **4** normal, **3–4** later |
| LLDoubles | 4, 4, 4 | 4, 4, 5 | **2** normal, **1–2** later |

| Vanilla: 6 frames | LessLag: 3 frames | LLUltra: 2 frames | LLUltra, late press: 3 frames |
| --- | --- | --- | --- |
| ![Vanilla input to crouch](docs/media/latency-vanilla.gif) | ![LessLag input to crouch](docs/media/latency-lesslag.gif) | ![LLUltra input to crouch](docs/media/latency-llultra.gif) | ![LLUltra input to crouch, input late in the frame](docs/media/latency-llultra-later.gif) |

These are emulator captures with synthetic input, not Switch input-to-photon
measurements. Method, every trial and the caveats:
[LATENCY_METHODOLOGY.md](docs/LATENCY_METHODOLOGY.md); full results:
[PERFORMANCE.md](docs/PERFORMANCE.md).

## Configuration

`sd:/ultimate/ssbu_online_deluxe_rewrite/config.ini`, created with defaults on
first boot; menu edits are saved automatically. Keys, boot flags and the Eden
overrides are in [Configuration](docs/CONFIGURATION.md).

## Diagnostics

Logs live under `sd:/ultimate/ssbu_online_deluxe_rewrite/` (`install.log`,
`events.log`, `hang-trace.log`, `renderer-status.txt`, `panic.log`). Keep the
exact NRO and its debug bundle when reporting a crash. Event codes, log
formats, bisection flags and the Eden test lifecycle are in
[Debugging](docs/DEBUGGING.md).

## Build

```powershell
cargo clippy --all-targets -- -D warnings
cargo build --no-default-features
cargo test
cargo skyline clippy -- -- -D warnings
/usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -Command '& ./tools/Build.ps1 -Package'
```

Toolchain, native dependency and release steps: [Build](docs/BUILD.md).

## Documentation

| Topic | Document |
| --- | --- |
| Install, coexistence with other plugins, rollback | [INSTALL.md](docs/INSTALL.md) |
| Controls and the settings rows | [CONTROLS.md](docs/CONTROLS.md) |
| Screenshots and GIFs | [SCREENSHOTS.md](docs/SCREENSHOTS.md) |
| Media builds, encoding and scope | [MEDIA_PROVENANCE.md](docs/MEDIA_PROVENANCE.md) |
| Rendering profiles, Auto, present-at-flush, measured results | [RENDERING.md](docs/RENDERING.md) |
| Mimicking the Switch in Eden (CPU quota bias for clips and A/B runs) | [EMULATOR_METHOD.md](docs/EMULATOR_METHOD.md) |
| Native character-select and arena display | [NATIVE_DISPLAY.md](docs/NATIVE_DISPLAY.md) |
| Dispatcher backoff | [DISPATCHER_BACKOFF.md](docs/DISPATCHER_BACKOFF.md) |
| How another plugin plugs in (menu rows, or its own ImGui windows) | [PLUGINS.md](docs/PLUGINS.md), [sdk/README.md](sdk/README.md) |
| Timing captures | [CAPTURES.md](docs/CAPTURES.md) |
| Configuration keys and boot flags | [CONFIGURATION.md](docs/CONFIGURATION.md) |
| Logs, event codes, crash diagnosis, Eden test lifecycle | [DEBUGGING.md](docs/DEBUGGING.md) |
| Performance measurement method and results | [PERFORMANCE.md](docs/PERFORMANCE.md), [LATENCY_METHODOLOGY.md](docs/LATENCY_METHODOLOGY.md) |
| Build, toolchain, release | [BUILD.md](docs/BUILD.md), [IMGUI_DEPENDENCY.md](docs/IMGUI_DEPENDENCY.md) |
| Validation ledger (every build, install and console result) | [VALIDATION.md](docs/VALIDATION.md), [STATUS.md](docs/STATUS.md) |
| Fork provenance and deliberate differences | [reference/README.md](reference/README.md), [FORK_PARITY.md](docs/FORK_PARITY.md) |

[firmware]: https://en-americas-support.nintendo.com/app/answers/detail/a_id/22525/~/nintendo-switch-system-updates-and-change-history#current
[atmosphere]: https://github.com/Atmosphere-NX/Atmosphere/releases/tag/1.12.1
[skyline]: https://github.com/skyline-dev/skyline/releases/tag/beta
