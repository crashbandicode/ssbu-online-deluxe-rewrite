# Online Deluxe rewrite

> **Disclaimer:** this project was developed with the assistance of AI
> (Claude, Codex and other coding agents) under the operator's direction and
> review. Every build recorded here was tested as described in
> [validation](docs/VALIDATION.md); nothing is claimed beyond those records.

A Rust/Skyline reconstruction of Online Deluxe for **Super Smash Bros. Ultimate 13.0.5**,
delivered as **one NRO**: rendering profiles that cut input latency, a global
Player Info window, the native character-select status line and network
display. No overclocking.

**Current version: 0.6.0** (2026-10-19). The Player Info window's first page is
redesigned: a `You` line, an `Opp` line for a live opponent, the five match-time
settings, a one-line fps footer and a chord hint, with the window's own toggles
on a new Window page. Not yet checked on the console. Every release on Forgejo
carries the NRO and its manifest; what each version changed and which console
checks are owed is in [STATUS.md](docs/STATUS.md), the evidence per build in
[VALIDATION.md](docs/VALIDATION.md).

> **Install only this NRO.** It replaces the original Online Deluxe fork, SsbuSync
> and the PIA manager; remove those three. With SsbuSync still loaded, Deluxe
> turns all of its hooks off and you are playing the old fork without noticing.
> [Install, what to remove, what to keep](docs/INSTALL.md).

![Cycling the overlay views](docs/media/overlay-cycle.gif)

| | |
| --- | --- |
| ![Settings window](docs/media/settings-window.png) <br> The Settings view in menus. | ![Training character select](docs/media/training-css.png) <br> The native status line on the character select. |
| ![Match with the compact strip](docs/media/match-settings-strip.png) <br> In a match the window becomes a compact two-row strip at the top left. | ![Character select after a match](docs/media/smash-css-after-match.png) <br> After a match the manual profile is held at Vanilla and the line reads `Vanilla*`. |

The window captures predate the 0.6.0 layout. More in [Screenshots](docs/SCREENSHOTS.md).

## What it does

- **Rendering profiles.** Vanilla, LessLag, LLUltra and LLDoubles trade buffered
  frames for latency; Auto picks one from the scene. The default is **Vanilla+**:
  Vanilla's three-image chain presenting at the flush (L+R+X toggles it).
  [Rendering profiles](docs/RENDERING.md)
- **Player Info window.** Settings, Plugins, Performance, Window and Debug pages
  on L+R+Down; a compact strip in matches; the native character-select line keeps
  your profile, buffer and the opponent's settings and RTT.
  [Controls](docs/CONTROLS.md), [native display](docs/NATIVE_DISPLAY.md)
- **Network display.** Opponent name, interface, settings and coloured RTT from
  the game's own measurement, shown only while their station is live; Stealth
  and Streamer switches.
- **Dispatcher backoff.** The game's idle dispatcher sleeps instead of spinning.
  [Dispatcher backoff](docs/DISPATCHER_BACKOFF.md)
- **Plugins.** Other mods add rows or a page of their own through the exported
  API and Rust SDK. Character FX lives in the separate
  [FX Control](https://git.syntaxbyexample.com/intpa/ssbu-fx-control) plugin.
  [Plugins](docs/PLUGINS.md), [`sdk/`](sdk/README.md)
- **Captures.** Opt-in character-select and whole-match timing captures to the
  card. [Captures](docs/CAPTURES.md)

## Optimizations added by this rewrite

Two things the original Online Deluxe does not do; both have an INI switch
(`dispatcher_backoff`, `early_present`, default on). Basics first:
[How Deluxe LessLag and LLUltra work](docs/LESSLAG_LLULTRA.md).

**Dispatcher backoff.** When the main CPU task keeps finding no work it can do,
the rewrite lets it sleep briefly, with growing pauses up to a small limit, so
other tasks get the core. Locks and checks stay as they are.
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

Hold **L+R** and tap **D-pad Down** to cycle the window: settings → plugins
(when any registered) → own plugin pages → performance → window → hidden.
**Up/Down** selects a row; **ZR + Left/Right** changes it, and the selected value
reads `< value >` while ZR is held. A plain Left/Right is ignored, because the
D-pad also taunts and moves menus. L+R and **Plus** opens Debug; L+R and **X**
toggles Vanilla+; L+R, **ZR** and **Y** toggles Streamer. Everything else is in
[Controls](docs/CONTROLS.md).

## Rendering profiles

| Profile | Textures | Presentation | Console | Emulator |
| --- | --- | --- | --- | --- |
| Vanilla | 3 | TwoBehind; Vanilla+ (the default) presents at the flush | 1080p, DRS off | 1080p, DRS off |
| LessLag | 2 | OneBehind, presents at the flush | 1080p, DRS on | 1080p, DRS off |
| LLUltra | 2 | Immediate, earlier flush, presents at the flush | 576p, DRS on; UltraRes raises it to 720p or 1080p | 1080p, DRS off |
| LLDoubles | 3 | OneBehind, presents at the flush | 1080p, DRS on | 1080p, DRS on |

Auto uses Vanilla in menus, offline matches and Training; online singles get
LessLag on console or LLUltra on emulator; online matches with more than two
fighters get LLDoubles. How Auto recognises an online match, the cleanup hold
(`Vanilla*`) and DRS are in [Rendering profiles](docs/RENDERING.md).

### Measured input delay

Captured frames from a synthetic input to the first visible response in Eden,
three trials per cell, on the 0.5.2 build (the earlier builds gave the same
counts):

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

Client-side rendering changes avoid gameplay and RNG modifications.
Linked-match results are evidence for the trials they cover, not proof that
every online scenario is desync-free.
