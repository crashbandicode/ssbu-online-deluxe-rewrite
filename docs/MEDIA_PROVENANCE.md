# Media provenance

Where the images in [Screenshots](SCREENSHOTS.md) come from, including their
builds, source intervals and transforms.

## Online match strip (0.10.8 candidate, 2026-11-16 UTC)

**Eden (untested on hardware).** The current `match-strip.gif`,
`match-settings-strip.png` and `match-strip-closeup.png` come from one real
cross-PC LAN Online Quickplay match, Smash 13.0.5, with two Sephiroths.
They replace only the canonical match-strip trio. The old
offline trio remains byte-identical under `*-v070-offline` filenames.

- **Captured side:** XPS A, DIAG build `0.10.8-a4411607be20-1794789531`,
  source `a4411607be20a104bd992e0c2428288aa8b5d633`, NRO SHA-256
  `b1eee4062db3b8672d03f788ddf00e9f6965066ab1a2eff9b1faa6632fb929c6`.
- **Peer:** Yoga B, production build `0.10.8-a4411607be20-1794789468`,
  NRO SHA-256
  `0434620d4702b8a36104c3b150f1c8c325893d6d2f327eddf1bcb0de7585a226`.
  The DIAG capture demonstrates its UI; it does not substitute for production
  binary validation.
- **Eden:** released AppImage SHA-256
  `2f315fc4b7d9658afef84f4ba3cb80a3d0b33292eaef66a802ef837222a247b9`.
  Both profiles were muted; the GDB stub and Horizon-OC stand-in were on.
  The stand-in changes no console clock.
- **Source:** retained `runtime-counter-fix-1/evidence-a/out/m-match-a/`
  `media-normal100/`, capture metadata SHA-256
  `6c0c4e01d5c1519a96aef614d50d364c36860d61ac8dfa7f6fb7cc8cf025b947`.
  The source capture-start receipt is 2026-11-16 01:10:50.739152 UTC.
  Pair setup, input/action receipts, actual match reviews and full cleanup
  remain with that run.
- **Timing:** the recorder requested 3 seconds at 10 images/s but returned
  30 frames over 8.198065 seconds, with `timing_warning=true`. This is not a
  10 fps recording. The published GIF uses frames 0000–0012 and frame 0013
  only as its end boundary: source interval 0.286197–3.791105 seconds,
  3.504908 seconds in total. Variable GIF delays follow consecutive source
  timestamps, rounded cumulatively to 10 ms; the encoded length is 3.50 s
  (13 frames, roughly 3.7 captured images/s). Later source frames overlap
  the deliberate 92% throttle and are outside this trim.
- **Pixels:** the source is already 1280×720. The GIF uses only palette
  conversion, with no resizing, interpolation, compositing or repainted UI.
  The full PNG is a byte copy of frame 0010 (2.975024 s, `Game 100%`).
  The close-up is its lossless source-pixel crop `x=65,y=22,w=451,h=27`,
  with no enlargement.
- **Scope:** a real online strip with Opp and Game fields. Displayed values
  are emulator observations, not a Switch benchmark or a latency comparison.
  These assets do not claim a new offline reshoot or the cancelled SD wait
  and rematch validation pair.

| File | Size | Length | SHA-256 |
| --- | --- | --- | --- |
| [match-strip.gif](media/match-strip.gif) | 1280×720 | 3.50 s | `024e62dbbd3013b81d8420b9a673516dc824a821b9494d45cecd6c051e73cc52` |
| [match-settings-strip.png](media/match-settings-strip.png) | 1280×720 | still | `0315ca5b992c745069870e6019843cc12ad1b9445d321cfbf77dcbe41648ab5c` |
| [match-strip-closeup.png](media/match-strip-closeup.png) | 451×27 | still | `21dbae648123017e3e6deea2d51d821a3e61519e11a932d322d52853d3009bdc` |

Full source-frame hashes, action receipts, transform script, encoded delays,
output hashes and first/middle/last review pixels are retained in the
operator's evidence directory
`deluxe-gamespeed/match-strip-media-0108/` (`TRIM-ENCODING.json`,
`source/`, `trim_encode.py`, `review-final/`). The initial full-source encode
and its review are retained separately; the final trim is authoritative.

## Game-speed dip and recovery (same 0.10.8 candidate)

**Eden (untested on hardware).** These additive assets use the same
2026-11-16 UTC paired run, XPS A DIAG build
`0.10.8-a4411607be20-1794789531` and Yoga B production peer identified
above. B was limited to 92%, then restored to 100%. The root reviewer
directly checked GIF first/middle/last as Game 100%, 93% (delay too low),
and 100%; the dip still reads 91% (delay too low).

- **Cuts:** normal100 frames 0004–0012, stable92 frames 0008–0016, then
  recover92 frames 0000–0008, each timed to its next captured frame.
  Setting actions and waits between the cuts are omitted; this is not a
  continuous recording of the speed-limit control action.
- **Encoding:** 1280×720 source geometry, 27 frames over 7.46 s, variable
  delays (maximum encoded cadence about 3.85 images/s). The source request
  for 10 images/s was not achieved. Palette conversion only; no crop,
  resize, interpolation, compositing or UI redraw.
- **Stills:** original PNG bytes from normal frame 0010, dip frame 0010
  and recovered frame 0008 (the GIF’s final frame), all 1280×720.
  The recovered frame was directly reviewed as Game 100%.
- **Evidence:** `deluxe-gamespeed/game-speed-media-0108/ENCODING.json`
  retains every selected frame, timestamp, hash and encoded delay;
  `VISUAL-REVIEW.json` binds the root’s pixel review. A rejected first
  encode that overlapped the next 80% throttle is retained externally.
  Action timestamps are also summarized in
  `match-strip-media-0108/SOURCE-ACTION-TIMELINE.json`.

| File | Length | SHA-256 |
| --- | --- | --- |
| [game-speed.gif](media/game-speed.gif) | 7.46 s | `1bc2112bd3eef29fb5363544a29e0299cc29121c63a2e4a22de0cd7c5bc5766c` |
| [game-speed-normal.png](media/game-speed-normal.png) | still | `0315ca5b992c745069870e6019843cc12ad1b9445d321cfbf77dcbe41648ab5c` |
| [game-speed-dip.png](media/game-speed-dip.png) | still | `d2a6df8f87a8b7d233ddb8f5488b1a18cb64c736c6dde67fb71692cb6b232c18` |
| [game-speed-recovered.png](media/game-speed-recovered.png) | still | `f0e932d45152b6398ade82a3ff8fb3efd5726033cd209147f20f38025874067e` |

<a id="capture-identity"></a>

## Historical offline capture identity

The historical offline UI examples were captured on **2026-10-24** in an
isolated, muted
Eden profile on Ubuntu, running Smash **13.0.5**. They replace the older 0.5.0
UI examples. Four files are from a later session; see
[Horizon-OC indicator media](#horizon-oc-indicator-media-081-candidate-2026-10-27).

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

## Historical offline scope

The 0.7.0/0.8.1 examples are offline emulator UI demonstrations:

- Displayed FPS and frame times belong to this fixture. They are not a Switch
  benchmark or an input-latency comparison.
- Not demonstrated: the online Opp summary, Auto's online `(Nf)` value and the
  console-only LLUltra resolution cycle.
- The revision-5 plugin status cells, first released in 0.8.0, are not in this
  0.7.0 binary.

No new console run is claimed.

## Historical offline encoding

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

## Historical offline assets

These retained examples are shown in [Screenshots](SCREENSHOTS.md). The four
marked 0.8.1 come
from the Horizon-OC stand-in run described [below](#horizon-oc-indicator-media-081-candidate-2026-10-27).

| File | Size | Length | Shows |
| --- | --- | --- | --- |
| [overlay-cycle.gif](media/overlay-cycle.gif) | 480×400 | 14 s | L+R+Down page cycle |
| [settings-edit.gif](media/settings-edit.gif) | 480×200 | 10 s | ZR-guarded NetLatency edit (0.8.1, Training, with the OC clock) |
| [privacy-shortcuts.gif](media/privacy-shortcuts.gif) | 480×200 | 10 s | ZR+X / ZR+Y on the You row (0.8.1, Training, with the OC clock); chords retired in 0.9.0, not re-recorded |
| [oc-indicator.gif](media/oc-indicator.gif) | 480×200 | 10 s | ZR+L3 always/online, the OC clock (0.8.1); chord retired and `always` clock added in 0.9.0, not re-recorded |
| [performance-window-oc.png](media/performance-window-oc.png) | 300×336 | | Performance close-up, Clock row with `OC online: idle` (0.8.1) |
| [fx-control-edit.gif](media/fx-control-edit.gif) | 480×340 | 8 s | Companion fighter-row edit |
| [training-strip.gif](media/training-strip.gif) | 1280×720 | 6 s | Training with Settings open |
| [match-strip-v070-offline.gif](media/match-strip-v070-offline.gif) | 1280×720 | 6 s | Match with the Settings strip |
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
| [match-settings-strip-v070-offline.png](media/match-settings-strip-v070-offline.png) | 1280×720 | | Match, Settings strip |
| [match-strip-closeup-v070-offline.png](media/match-strip-closeup-v070-offline.png) | 376×42 | | Strip close-up |
| [match-performance.png](media/match-performance.png) | 1280×720 | | Match, Performance strip |
| [match-debug.png](media/match-debug.png) | 1280×720 | | Debug over the pause menu |
| [results-quit.png](media/results-quit.png) | 1280×720 | | No Contest after Quit |
| [smash-css-after-match.png](media/smash-css-after-match.png) | 1280×720 | | CSS after the match, `You: Vanilla* \| Auto` |

## Horizon-OC indicator media (0.8.1 candidate, 2026-10-27)

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
  [oc-indicators.svg](oc-indicators.svg) (0.8.1, the Horizon-OC clock) redraw
  source geometry and are **not screenshots**.

None of these retained assets is relabeled as a new Switch measurement.
