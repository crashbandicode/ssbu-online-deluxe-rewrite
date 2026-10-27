# Media provenance

Where the images in [Screenshots](SCREENSHOTS.md) come from and what they do
and do not show. The observations themselves are recorded in
[Validation](VALIDATION.md#070--offline-ui-capture-review--2026-10-24).

## Capture identity

The current UI examples were captured on **2026-10-24** in an isolated, muted
Eden profile on Ubuntu, running Smash **13.0.5**. They replace the older 0.5.0
UI examples. Four files are from a later session; see
[Horizon-OC indicator media](#horizon-oc-indicator-media-090-candidate-2026-10-27).

- **Deluxe:** the released 0.7.0 NRO, build `0.7.0-9a7f81ce1937-1792670948`,
  tag `v0.7.0`, SHA-256
  `2ff456161c52522c6881956d0ad3a5a037a1a3fc74c8757a1e3a7afe49012cae`.
- **FX Control companion:** the separate
  [FX Control](https://git.syntaxbyexample.com/intpa/ssbu-fx-control) plugin
  supplies the FX Control and Stages pages. This fixture uses its 0.2.1
  development build `0.2.1-0c0d7e62c9d0-dirty-1792682041`, NRO SHA-256
  `8bf2bdb451266feba0e57a6b6996aceaf334b9cf190add0e8219aa9d8e1efdd8`.
- **Eden:** the running binary reports commit
  `10bcd2d849843b146a79c61a21df615e1bdcfcde`, SHA-256
  `64cf47519891440f4a5adfe4d3dfeff180fa4d90b61eb424ee1550e2a3fc3004`. The
  checkout alongside it is a different revision, `39ed41e`, and is not
  substituted for the binary's identity.
- **Fixture settings:**
  - manual **LessLag**, `platform=emulator`, input buffer Auto;
  - `vanilla_plus=false`, so the post-match label reads `Vanilla*` rather than
    the default `Vanilla+*`;
  - Stealth and Streamer initially off;
  - all pages visible, graph performance view, hints on;
  - FX starts and ends with its fighter settings On.

  These are demonstration settings, **not the shipped defaults**.

### Ally recording check

A follow-up isolated Windows run on Ally used the same Deluxe and FX binaries.
Its native GIF recorder captured the Deluxe Settings window in a three-second
sample (29 frames, one dropped; the GIF retains the full three seconds). Those
samples stay in the external evidence: the published close-ups above retain
the sharper Ubuntu source pixels. Copied input macros were not replay-tested.

The Ally executable's SHA-256 is
`45af318c93cbc00a2794421ee884ffbcb9dbec75bbdbdeef27af30d61cd2cbad`.
Its package identity names `289c4684e705bbb3cb25b3a10b5a99b7f5adfb7e`, while
the live status reports `stable`; those identifiers are recorded separately.

## Scope

These are offline emulator UI demonstrations:

- Displayed FPS and frame times belong to this fixture. They are not a Switch
  benchmark or an input-latency comparison.
- Not demonstrated: the online Opp summary, Auto's online `(Nf)` value and the
  console-only LLUltra resolution cycle.
- The unreleased revision-5 status cells on main after v0.7.0 are not in this
  release binary.

No new console run is claimed.

## Encoding

- **Source and scaling:** the source display is **1920×1080**. Whole-screen
  PNGs and the Training/match GIFs are reduced to **1280×720**.
- **Crops:** window and strip close-ups keep source pixels. Menu GIFs are
  cropped from the private display.
- **Frame rate:** menu GIFs were recorded at **10 images/s**. The encoder
  drops repeated consecutive frames but keeps their display time, so a GIF
  can have fewer frames than its duration suggests.
- **No redrawing:** no UI is redrawn or composited into these captures.

Raw screenshots, recordings, input and status receipts, encoding metadata and
review frames are kept outside the repository in the operator's evidence
directory `deluxe-media-refresh-1024/`.

## Current assets

All 29 are shown in [Screenshots](SCREENSHOTS.md). The four marked 0.9.0 come
from the Horizon-OC stand-in run described [below](#horizon-oc-indicator-media-090-candidate-2026-10-27).

| File | Size | Length | Shows |
| --- | --- | --- | --- |
| [overlay-cycle.gif](media/overlay-cycle.gif) | 480×400 | 14 s | L+R+Down page cycle |
| [settings-edit.gif](media/settings-edit.gif) | 480×200 | 10 s | ZR-guarded NetLatency edit (0.9.0, Training, with the OC clock) |
| [privacy-shortcuts.gif](media/privacy-shortcuts.gif) | 480×200 | 10 s | ZR+X / ZR+Y on the You row (0.9.0, Training, with the OC clock) |
| [oc-indicator.gif](media/oc-indicator.gif) | 480×200 | 10 s | ZR+L3 always/online, the OC clock (0.9.0) |
| [performance-window-oc.png](media/performance-window-oc.png) | 300×336 | | Performance close-up, Clock row with `OC online: idle` (0.9.0) |
| [fx-control-edit.gif](media/fx-control-edit.gif) | 480×340 | 8 s | Companion fighter-row edit |
| [training-strip.gif](media/training-strip.gif) | 1280×720 | 6 s | Training with Settings open |
| [match-strip.gif](media/match-strip.gif) | 1280×720 | 6 s | Match with the Settings strip |
| [main-menu.png](media/main-menu.png) | 1280×720 | | Main menu, window hidden |
| [settings-view.png](media/settings-view.png) | 1280×720 | | Settings page |
| [settings-window.png](media/settings-window.png) | 380×154 | | Settings close-up |
| [performance-view.png](media/performance-view.png) | 1280×720 | | Performance page |
| [window-options-window.png](media/window-options-window.png) | 380×174 | | Window Options close-up |
| [streamer-on.png](media/streamer-on.png) | 1280×720 | | Streamer and Stealth on |
| [debug-view.png](media/debug-view.png) | 1280×720 | | Debug on the main menu |
| [debug-window.png](media/debug-window.png) | 300×468 | | Debug close-up |
| [fx-control-view.png](media/fx-control-view.png) | 1280×720 | | Companion FX Control page |
| [fx-control-window.png](media/fx-control-window.png) | 380×258 | | FX Control close-up |
| [fx-stages-window.png](media/fx-stages-window.png) | 380×258 | | Companion Stages close-up |
| [training-css.png](media/training-css.png) | 1280×720 | | Training CSS, `You: LessLag \| Auto` |
| [smash-css-after-training.png](media/smash-css-after-training.png) | 1280×720 | | Ordinary CSS after Training |
| [debug-training.png](media/debug-training.png) | 1280×720 | | Debug in Training |
| [debug-window-training.png](media/debug-window-training.png) | 300×468 | | Training Debug close-up |
| [match-settings-strip.png](media/match-settings-strip.png) | 1280×720 | | Match, Settings strip |
| [match-strip-closeup.png](media/match-strip-closeup.png) | 376×42 | | Strip close-up |
| [match-performance.png](media/match-performance.png) | 1280×720 | | Match, Performance strip |
| [match-debug.png](media/match-debug.png) | 1280×720 | | Debug over the pause menu |
| [results-quit.png](media/results-quit.png) | 1280×720 | | No Contest after Quit |
| [smash-css-after-match.png](media/smash-css-after-match.png) | 1280×720 | | CSS after the match, `You: Vanilla* \| Auto` |

## Horizon-OC indicator media (0.9.0 candidate, 2026-10-27)

`settings-edit.gif`, `privacy-shortcuts.gif`, `oc-indicator.gif` and
`performance-window-oc.png` replace or extend the 0.7.0 set so the Horizon-OC
clock is visible. Eden cannot run Horizon-OC, so the test-only
[hoc-standin](../tools/hoc-standin/README.md) registered `hoc:clk` and answered
protocol v1; Deluxe's client, gate and drawing are real, no clock was changed,
and the Clock row's MHz is the emulator host's CPU.

- **Emulator:** Eden GIF build `9a5d704c9c` (release AppImage `c3fc1b3c…`),
  muted, fullscreen 1920×1080 on a private headless display, isolated copy of
  the `acceptance-fable` profile (protected files verified unchanged).
- **Plugins:** ARCropolis, Deluxe and the stand-in only. The GIFs use Deluxe
  build `0.8.0-93b4d7c27f56-1793139771`; the Performance close-up uses
  `0.8.0-ac503c0c48e7-1793140514`, which adds only the Performance wrap.
- **Settings:** the 0.7.0 media fixture INI (manual LessLag, emulator, Stealth
  and Streamer off, Vanilla+ off) plus `hoc_oc=online`.
- **Scene:** Training, Link against a CPU Bayonetta. ZR, X, Y and the D-pad
  move only the fighters there, so the crop stays readable; the 0.7.0 clips
  were on the main menu, where ZR opens the sidebar.
- **Capture:** the same timed input jobs as 0.7.0 (`privacy-shortcuts`,
  `settings-edit`) plus `oc-indicator`, 10 s at 10 images/s. Frames are Eden
  window screenshots (the skill's `capture_eden.py` method), because X11 screen
  capture returned only black on this host's headless display. The largest gap
  between frames was 0.28 s.
- **Encoding:** `record-ui-demo` `gif_tool.py frames`, crop `1440,0,480,200`
  (the window sits top right in Training), 10 frames/s, source pixels, 480 wide.

Evidence (receipts, raw frames, review sheets, stand-in and Deluxe logs) is in
the operator's evidence directory `horizon-oc-game-aware-tangent-20261027/eden-media/`.

## Historical media and diagrams

These are kept unchanged and are not part of the 0.7.0 UI session.

- **Latency clips:** the four Sephiroth `latency-*.gif` clips belong to the
  0.5.2 synthetic-input experiment in
  [Latency methodology](LATENCY_METHODOLOGY.md).
- **Feature-toggle comparisons:** the six `toggles-*` images and GIFs keep their
  original comparison scope in [Emulator method](EMULATOR_METHOD.md).
- **Diagrams:** the ten SVGs are conceptual illustrations, used in the
  [README](../README.md), [Controls](CONTROLS.md), [Rendering](RENDERING.md)
  and [LessLag and LLUltra](LESSLAG_LLULTRA.md).
  [privacy-indicators.svg](privacy-indicators.svg) and
  [oc-indicators.svg](oc-indicators.svg) (0.9.0, the Horizon-OC clock) redraw
  source geometry and are **not screenshots**.

None of these retained assets is relabeled as a new Switch measurement.
