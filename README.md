# Online Deluxe rewrite

A Rust/Skyline reconstruction of Online Deluxe for **Super Smash Bros. Ultimate 13.0.5**,
delivered as **one NRO**: rendering profiles that cut input latency, a global
Player Info window, the native character-select status line, network display
and scoped character-FX controls. No overclocking.

**Current release: 0.4.2.** Every release on Forgejo carries the NRO and its
manifest. Install notes: [docs/INSTALL.md](docs/INSTALL.md).

![Cycling the overlay views](docs/media/overlay-cycle.gif)

| | |
| --- | --- |
| ![Settings window](docs/media/settings-window.png) <br> The Settings view. | ![Training character select](docs/media/training-css.png) <br> The native status line on the character select. |

More in [Screenshots](docs/SCREENSHOTS.md).

## What it does

- **Rendering profiles.** Vanilla, LessLag, LLUltra and LLDoubles trade
  buffered frames for latency; Auto picks one from the scene. Since 0.4.0
  two-image LessLag presents at the flush, which removed the character-select
  stall on the console (49.9 → 59.85 fps on the hand sweep). [Rendering profiles](docs/RENDERING.md)
- **Player Info window and native status.** Settings, Performance, Debug and
  Plugins views on L+R+Down; the character select keeps your profile, buffer
  and the opponent's settings and RTT at the top. [Controls](docs/CONTROLS.md),
  [native display](docs/NATIVE_DISPLAY.md)
- **Network display.** Opponent name, interface, settings and colored RTT from
  the game's own measurement; Stealth and Streamer switches. [Controls](docs/CONTROLS.md)
- **Character FX.** On, Off or Reduced for named character effects, online
  only by default. [FX](docs/FX.md)
- **Dispatcher backoff.** One inline hook that lets the game's idle dispatcher
  sleep instead of spinning. [Dispatcher backoff](docs/DISPATCHER_BACKOFF.md)
- **Plugins page.** Other mods register rows through an exported API and the
  Rust SDK. [Plugins](docs/PLUGINS.md), [`sdk/`](sdk/README.md)
- **Captures.** Opt-in 30 s character-select captures and whole-match captures
  streamed to the card for frame analysis. [Captures](docs/CAPTURES.md)

## Controls

Hold **L+R** (or **ZL+ZR**) and tap **D-pad Down** to cycle the window:
settings → plugins (when any registered) → performance → hidden. In the window
**Up/Down** selects a row and **Left/Right** changes it. L+R and **Plus** opens
the Debug view. Everything else, including the ZL-gated native edits on the
character select, is in [Controls](docs/CONTROLS.md).

## Rendering profiles

| Profile | Textures | Presentation | Console | Emulator |
| --- | --- | --- | --- | --- |
| Vanilla | 3 | TwoBehind | 1080p, DRS off | 1080p, DRS off |
| LessLag | 2 | OneBehind, presents at the flush | 1080p, DRS on | 1080p, DRS off |
| LLUltra | 2 | Immediate, earlier flush | 576p, DRS on | 1080p, DRS off |
| LLDoubles | 3 | OneBehind | 1080p, DRS on | 1080p, DRS on |

Auto uses Vanilla in menus, offline matches and Training; online singles get
LessLag on console or LLUltra on emulator; online matches with more than two
fighters get LLDoubles. How Auto recognises an online match, the cleanup hold
(`Vanilla*`) and DRS are in [Rendering profiles](docs/RENDERING.md).

### Measured input delay

Captured-response trials in Eden on build `b19b7d94ae136e55`: frames from a
synthetic input to the first visible response, three trials per cell.

| Profile | Normal injection phase | Later injection phase | Frames saved vs Vanilla |
| --- | --- | --- | --- |
| Vanilla | 6, 6, 6 | 6, 6, 6 | — |
| LessLag | 3, 3, 3 | 3, 3, 3 | **3** |
| LLUltra | 2, 2, 2 | 3, 2, 3 | **3–4** |
| LLDoubles | 4, 4, 4 | 4, 4, 4 | **2** |

LLDoubles was measured as a manual profile in two-fighter Training. These are
emulator screenshots with synthetic input, not Switch input-to-photon
measurements, and they predate 0.4.0's present-at-flush change to LessLag,
which removed the console's character-select stall (49.9 → 59.85 fps on the
hand sweep) without changing the frame count claim. Method:
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
| Native character-select and arena display | [NATIVE_DISPLAY.md](docs/NATIVE_DISPLAY.md) |
| Character FX coverage and exclusions | [FX.md](docs/FX.md) |
| Dispatcher backoff | [DISPATCHER_BACKOFF.md](docs/DISPATCHER_BACKOFF.md) |
| Plugins page and SDK | [PLUGINS.md](docs/PLUGINS.md), [sdk/README.md](sdk/README.md) |
| Timing captures | [CAPTURES.md](docs/CAPTURES.md) |
| Configuration keys and boot flags | [CONFIGURATION.md](docs/CONFIGURATION.md) |
| Logs, event codes, crash diagnosis, Eden test lifecycle | [DEBUGGING.md](docs/DEBUGGING.md) |
| Performance measurement method and results | [PERFORMANCE.md](docs/PERFORMANCE.md), [LATENCY_METHODOLOGY.md](docs/LATENCY_METHODOLOGY.md) |
| Build, toolchain, release | [BUILD.md](docs/BUILD.md), [IMGUI_DEPENDENCY.md](docs/IMGUI_DEPENDENCY.md) |
| Validation ledger (every build, install and console result) | [VALIDATION.md](docs/VALIDATION.md), [STATUS.md](docs/STATUS.md) |
| Fork provenance and deliberate differences | [reference/README.md](reference/README.md), [FORK_PARITY.md](docs/FORK_PARITY.md) |

Client-side rendering and effect changes avoid gameplay and RNG modifications.
Linked-match results are evidence for the trials they cover, not proof that
every online scenario is desync-free.
