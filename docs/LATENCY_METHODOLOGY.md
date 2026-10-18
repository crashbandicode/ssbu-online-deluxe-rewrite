# Measuring frames removed by LessLag and LLUltra

This procedure compares the first **captured image that reacts to an input**
under otherwise identical conditions. A 60 FPS counter, a shorter render queue,
or a successful hook installation alone does not demonstrate lower input delay.

The final-candidate experiment and an earlier instrumented experiment are recorded
below. Compatibility tests do not automatically remeasure frame savings. See
[PERFORMANCE.md](PERFORMANCE.md#full-lesslagllultra-response-trials-2026-10-06)
for the original record and [VALIDATION.md](VALIDATION.md) for build-specific tests.

The 24-trial result is tied to `b19b7d94ae136e55`. The shipping candidate
`d9acd03611e1d1d6` changes four reviewed display/test files only (remote buffer
text and compact Auto labels); its render/input/transport algorithms are unchanged.
Its linked-match and menu regressions passed, but no response trials were repeated
on that exact binary. Keep this distinction when citing the measurements.
`d9acd03611e1d1d6` was the shipping candidate on 2026-10-07. Since 0.4.0,
two-image LessLag presents the acquired image at the flush that submits it
(`render_policy::present_at_flush`). The LessLag frame counts below come from
the earlier path. The 0.5.0 and 0.5.2 re-measurements below were taken on the
new path; since 0.5.2 LLUltra and LLDoubles present at the flush too.

## Measurement boundaries

| Event | Evidence | Meaning |
| --- | --- | --- |
| `t0` | Host monotonic timestamps around the CLI input request and acknowledgment | When the test submitted input to Eden; acknowledgment alone does not prove game delivery |
| `t1` | Input marker in the captured image, matched to the guest API trace by present serial | First existing plugin callback poll that observed stick-down; not necessarily the game's first input read |
| `t2` | First captured image visibly showing the fighter crouch | Observable response at the screenshot capture boundary |

Host timestamps and guest ticks belong to different clocks. Join records by
their identities; do not subtract a host timestamp from a guest tick. Screenshot
completion includes capture/readback/file overhead and is not physical display
time. At 60 Hz, one nominal frame is `1000 / 60 ≈ 16.667 ms`; that conversion
does not independently establish the actual wall time of a stalled frame.

The reported native six-frame Smash delay is background context, not a constant
to subtract. This experiment neither measured Homebrew Menu latency nor isolated
an additional fixed emulator delay. The baseline was **Vanilla rendering with
the same plugins installed**, not an unmodified copy of Smash.

## Controlled setup

Record the exact game version, NRO SHA-256, matching symbol/source bundle,
emulator executable hash/commit, configuration, plugin hashes and host hardware.
Use the same character, stage, camera, controller path, game speed, graphics
settings and background workload for every mode. Keep input buffer on Auto and
character effects original (FX On through 0.4.2; since 0.5.0 FX is the separate
ssbu-fx-control plugin, so leave it out or On). Warm the scene and shader cache
before collecting trials.

The recorded experiment used Smash 13.0.5, custom Eden `10bcd2d849`, Vulkan,
docked mode, 1× resolution and 100% speed. Sephiroth crouched against Kazuya in
Training with Training Modpack and the gamepad overlay installed. It used
synthetic CLI input, not a physical controller.

Use an isolated emulator profile, one bounded process for latency measurements,
and a memory/runtime limit. Separate linked-match/desync tests from latency
trials so the second emulator does not change the measurement workload.

## Capture procedure

1. Before boot, create an empty diagnostic marker file in that profile's SD tree:

   ```text
   sd:/ultimate/ssbu_online_deluxe_rewrite/INPUT_RECEIPT
   ```

   The marker uses the existing ImGui/HID callback. It draws two colored anchors,
   a 24-bit present serial, a Down bit and parity. It adds no gameplay hook.

2. Select the mode under test and wait until the renderer reports it **committed
   and active**. A saved or requested mode alone is insufficient during a live
   transition. Retain this status file:

   ```text
   sd:/ultimate/ssbu_online_deluxe_rewrite/renderer-status.txt
   ```

3. Start the optional guest API trace using the existing capture trigger:

   ```text
   sd:/ultimate/ssbu_online_deluxe_rewrite/CAPTURE_TIMING
   ```

   Preserve the resulting complete CSV. Input-marker records carry kinds 6 and
   7: present serial/raw stick Y and present serial/HID sampling counter. The HID
   counter is not a clock synchronization mechanism.

4. Capture consecutive native game images, named `000.png`, `001.png`, etc.
   The original experiment used 640×360 PNGs and a 1 ms completion poll. Desktop
   screenshots taken at arbitrary intervals cannot replace this sequence.
   After image 5 completes, inject stick-down, retaining request/ack timestamps.
   Keep the input active through the response, then release it. Capture 20 images
   per trial and keep all originals and the host event log.

5. Determine the first crouch image from the fighter's visible pose. Check the
   raw images; animation elsewhere, the input marker itself, and the gamepad
   overlay must not count as the fighter's response. Do not derive the response
   frame from the queue depth or assumed savings being tested.

6. Repeat with input about 8 ms later in the frame. Queue the next screenshot
   **before** waiting to inject: the earlier helper that waited first skipped
   image identities and produced invalid trials. Repeat across independent runs
   and alternate mode order where possible. Predeclare trial counts and exclusion
   rules, retain failed trials, and report each phase separately.

The historical capture helpers and their exact configuration are retained with
the evidence bundle. They contain machine-specific paths; inspect and adapt
those paths before reuse. They are experimental scripts, not a portable CLI API.

## Validation and calculation

The repository decoder requires Python with Pillow. From the repository root,
use the interpreter for that environment:

```powershell
$AnalysisPython = '/path/to/python-environment/bin/python'
& $AnalysisPython tools/response_marker.py '/path/to/trial'
& $AnalysisPython tools/analyze_render_trace.py '/path/to/api-trace.csv'
```

The image decoder rejects absent anchors, ambiguous cells, parity errors and
nonconsecutive present identities. Preserve a failed trial, but do not assign an
exact frame count across missing or duplicate identities. Also inspect the API
trace for gaps and confirm the serials surrounding the input and response.

For a valid sequence, let `i0 = 5`, `i_down` be the first Down-marker image, and
`i_pose` the first crouch image:

```text
R = i_pose - i0                 # captured response frames after injection
M = i_down - i0                 # frames to plugin input observation
D = serial_pose - serial_down  # present-serial difference, modulo 2^24
R = M + D                      # consistency check for consecutive images
savings = R_vanilla - R_mode   # compare the same injection-phase condition
```

`D = 0` means the marker and crouch first appear in the same captured image. It
does not mean zero input latency. The plugin can poll after the game's input
read, so `D` is phase-dependent and is not an independent pipeline-depth measure.

Keep genuine stalls in frame-count results when identities remain consecutive.
If reporting a separate steady-cadence wall-time summary, disclose its exclusion
rule and list excluded stalls. Report per-trial counts and distributions rather
than selecting the fastest trial. Small samples do not justify universal bounds.

## Accepted historical result

Measured NRO:

```text
123d47db700f5d196161b9fd1af673d15214657d2364862c5dc4ea095be120f0
```

| Mode | Normal-phase valid trials | `R` | `D` | Fewer captured response frames than Vanilla |
| --- | --- | --- | --- | --- |
| Vanilla | 4 of 5 | 6 | 3 | — |
| LessLag | 7, including 5 fresh repeats | 3 | 1 | 3 |
| LLUltra | 5 of 5 | 2 | 0 | 4 |

The nine delayed-phase trials yielded `R = 7,6,6` for Vanilla, `3,3,3` for
LessLag and `3,2,2` for LLUltra. One normal Vanilla trial had a missing present
identity and was excluded from exact counts. Three other trials had roughly
47–54 ms capture intervals with continuous identities; their frame counts remain
valid and their stalls are documented in PERFORMANCE.md.

These results support lower captured response delay in this setup. They do not
promise the same count for every input, establish Switch input-to-photon latency,
or measure LLDoubles. No overclocking behavior is required or implemented here.

Evidence is outside Git, including screenshots, CSVs, host timestamps, scripts,
reviews, per-trial results and a hash manifest:

```text
dist/123d47db700f5d19/runtime-evidence/
  response-summary.json
  evidence-sha256.json
  method/
  marked-vanilla/
  marked-lesslag-repeat/
  marked-ultra/
  phase8-vanilla/
  phase8-lesslag/
  phase8-ultra/
```

## Final candidate: 24 trials, 2026-10-07

NRO SHA-256:

```text
b19b7d94ae136e55d2fe59136dd9ff2c40ce9436393a59984660ed3ab8c52cd7
```

One isolated Eden `10bcd2d849` instance, Vulkan/docked/1×/100%, Sephiroth versus
Sora on the Training stage, 640×360 captures. Training Modpack, corrected gamepad
overlay, ARCropolis, Elite Legal Stages and standalone ImGui were installed.
FX was On/all and input buffer Auto. Profiles were selected manually and verified
Active in `policy_committed` before and after every group. Normal groups ran
Vanilla → LessLag → LLUltra → LLDoubles; delayed groups reversed that order.

| Mode | Normal `R` | Normal `M` / `D` | Delayed `R` | Delayed `M` / `D` |
| --- | --- | --- | --- | --- |
| Vanilla | 6,6,6 | 3 / 3 | 6,6,6 | 4 / 2 |
| LessLag | 3,3,3 | 2 / 1 | 3,3,3 | 3 / 0 |
| LLUltra | 2,2,2 | 2 / 0 | 3,2,3 | 2 / 1,0,1 |
| LLDoubles | 4,4,4 | 3 / 1 | 4,4,4 | 4 / 0 |

All 24 critical windows had consecutive present identities. Each identity joined
exactly one API kind-6 input record and one kind-7 sampling-counter record, and
the image Down bits matched the recorded stick values. All eight completed API
traces passed the decoder with zero skipped events.

Visual pose review was required. In delayed Vanilla trial 1, idle animation
crossed the automated difference threshold at image 6; the first visible crouch
was image 11. The raw automatic result is retained alongside the explicit manual
correction. Its critical window 4–12 is consecutive; an earlier gap between
images 0 and 1 lies outside that window. Normal Vanilla trial 1 contained a
36.1 ms capture interval with consecutive identities and remains included.

The capture helper initially copied CSV files before export finished. Those
partial copies were preserved; completed original exports were copied only after
verifying the partial bytes were their exact prefixes. Do not analyze files named
`*.partial-copy.csv`. The trace `engine=Vanilla` header and status `mode=vanilla`
describe startup mode in this build; live mode is established by the accompanying
`policy_committed`, `selected_profile` and Active status, not that header.

These trials support 3 fewer captured response frames for LessLag, 3–4 for
LLUltra and 2 for LLDoubles under these conditions. LLDoubles here is a manual
profile in two-fighter Training, not a four-fighter timing measurement. Small
synthetic-input samples on one host do not establish universal or console bounds.
Fable's requested independent review could not run because its provider denied
model access; the primary reviewed the images, serial joins and calculations.

Evidence, including all 480 original images, timestamps, traces, status snapshots,
scripts and the manual correction, is retained outside Git:

```text
dist/b19b7d94ae136e55/runtime-evidence/response-and-cutin/
```

## Re-measurement on 0.5.0, 2026-10-15

NRO SHA-256 (release v0.5.0, build `0.5.0-44830e5d9255-1792012237`):

```text
d9dd7df281671c12f14c4da191534650d4eac1856aaf5e4f886b97f6f7fcaf4f
```

Same method and the same Eden build (`10bcd2d849`), Vulkan/docked/1×/100%,
Sephiroth versus Kazuya (CPU Stop) on the Training stage, 640×360 captures,
`INPUT_RECEIPT` marker, input buffer Auto. Plugins next to Deluxe: Training
Modpack, gamepad overlay, nro_hook, param_hook; no FX Control. Modes were
selected manually and verified `policy_committed` + `engine_status=Active`
before every group (status files retained). Three trials per cell; LLDoubles
was not measured.

| Mode | Normal `R` (included trials) | Delayed `R` (included trials) | Fewer frames than Vanilla, normal / delayed |
| --- | --- | --- | --- |
| Vanilla | 6, 6 (2 of 3) | 6, 6, 6 (3 of 3) | — |
| LessLag | 3, 3, 3 (3 of 3) | 3, 3 (2 of 3) | 3 / 3 |
| LLUltra | 2, 2 (2 of 3) | 2, 3 (2 of 3) | 4 / 4, 3 |

Four of the 18 trials had one missing present identity inside the critical
window and are excluded from exact counts (their first-crouch images were
10, 7, 7 and 8, consistent with the included trials). Visual pose review
agreed with the automatic index in all 18. The three delayed Vanilla trials
share an API trace with four skipped events and are included on image counts
only. A first pass with Eden's `ui.screenshot_height` at 720 produced ~34 ms
captures and non-consecutive identities in every trial; it is retained as
invalid. The counts match the b19 table above: the 0.4.0 present-at-flush
change and the 0.5.0 FX removal did not change the captured response delay
in this setup. Evidence (360 images per pass, traces, status snapshots,
NOTES.md) is in the operator's evidence root under `latency-050-1015/`.

## Re-measurement on 0.5.2 present-at-flush, 2026-10-15

0.5.2 extended `render_policy::present_at_flush` from LessLag to LLUltra and
LLDoubles (commit `ce0145a`), so all three render-opts profiles now present the
acquired image at the flush that submits it. This pass asks whether that moved
the captured response counts, LLUltra's advantage over LessLag in particular.

NRO SHA-256 (build `0.5.2-ce0145a2155c-1792070522`, package `1525e3e0392c8403`):

```text
1525e3e0392c840306073762cb7e78670745fe0e925de013f660c901371cfdff
```

Same method and Eden build (`10bcd2d849`), Vulkan/docked/1×/100%, Sephiroth
versus Kazuya (CPU Stop) on the Training stage, `ui.screenshot_height=360`
(480 native 640×360 images; per-trial median capture interval 16.0 to 17.5 ms),
`INPUT_RECEIPT` marker, input buffer Auto, `platform=emulator`,
`auto_profile=false`. Plugins next to Deluxe: Training Modpack, gamepad overlay,
Arcropolis, css_preserve, nro_hook, param_hook; no FX Control. One private
profile copy and one uninterrupted Eden session (6 GiB cgroup cap). Normal
groups ran Vanilla, LessLag, LLUltra, LLDoubles; delayed groups ran in reverse.
Every group's committed mode and `engine_status=Active` were read from
`renderer-status.txt` before and after the group (status files retained). Three
20-image trials per cell, 24 trials, no replacements.

| Mode | Normal `R` | Normal `M` / `D` | Delayed `R` | Delayed `M` / `D` | Fewer frames than Vanilla, normal / delayed |
| --- | --- | --- | --- | --- | --- |
| Vanilla | 6, 6, 6 | 3 / 3 | 6, 6, 6 | 4 / 2 | — |
| LessLag | 3, 3, 3 | 2 / 1 | 3, 3, 3 | 3 / 0 | 3 / 3 |
| LLUltra | 2, 2, 2 | 2 / 0 | 2, 3, 3 | 2 / 0, 1, 1 | 4 / 4, 3, 3 |
| LLDoubles | 4, 4, 4 | 3 / 1 | 4, 4, 5 | 4 / 0, 0, 1 | 2 / 2, 2, 1 |

All 24 critical windows (image 4 through max(pose, Down)+1) had consecutive
present identities, so none was excluded; automatic and visually reviewed pose
indices agreed in all 24 and `R = M + D` held in all 24. Seven delayed-phase
trials (Vanilla 1 and 2, LessLag 1, LLUltra 1, LLDoubles 1 to 3) are not covered
by their API trace export, which holds 1800 present identities that begin after
those trials; they rest on the image marker's identities alone. Restricted to
trials whose identities join exactly one API kind-6 and one kind-7 record
(17 of 24), the delayed values are Vanilla 6, LessLag 3, 3, LLUltra 3, 3 and no
LLDoubles trial. Five of the eight traces carry the analyzer warning "Engine
state changed during capture." and one (LLUltra normal) one skipped event;
neither was investigated, no API-level A/B claim is made, and every group's
before/after `renderer-status.txt` agree on the committed profile.

Result: LessLag (3), Vanilla (6) and normal-phase LLUltra (2) and LLDoubles (4)
are unchanged from the b19 and 0.5.0 results, so present-at-flush did not shift
the captured-image counts. LLUltra still has one frame fewer than LessLag in the
normal phase (2 against 3, three of three trials). In the delayed phase the
LLUltra advantage is within the trial-to-trial spread: 2, 3, 3 against 3, 3, 3,
and on the API-covered trials both are 3. LLDoubles delayed shows a single
5 (consecutive identities, serials 35777 to 35783). These are captured-image
emulator frames from synthetic input; they say nothing about the console frame
time stall that present-at-flush targets and are not Switch input-to-photon
measurements.

Each included trial's native images are annotated in
[latency-vanilla.gif](media/latency-vanilla.gif),
[latency-lesslag.gif](media/latency-lesslag.gif) and
[latency-llultra.gif](media/latency-llultra.gif) (normal-phase trial 1 of each;
the counter starts at the first image after the injection and reaches `R` on the
first crouch image). [latency-llultra-later.gif](media/latency-llultra-later.gif)
is the LLUltra delayed-phase trial 2 (`R` = 3, `M` = 2, `D` = 1, API-covered):
the Down marker lights on the same image as in the normal phase, the crouch
appears one image later, so the saving against Vanilla is 3 frames instead of
4. Evidence, including all 480 images, per-trial analysis, the
adapted helpers, partial-copy verification and the staging receipt, is retained
outside Git in the operator's evidence root under `latency-052-1015/`, which
also holds the helper scripts. Deviations from the 0.5.0 run: a private copy of
its profile was used instead of the original, and the 6 GiB cgroup cap (8 GiB
before) reached its limit repeatedly (`memory.events max`) without an OOM kill
or a visible frame-rate drop.

## 0.5.4 research: pre-HID delay, 2026-10-18

All numbers in this section are **emulator evidence**, not Switch input-to-photon
measurements. Research branch `research/input-lag-1018` reproduced baseline
`1c184ff8dd2a` before testing the default-off interval request and adaptive delay.
The measured delay build is `0.5.4-3fea54be82ab-1792335022`; later documentation
commits do not change that measured binary.

The delay runs immediately before the original normal HID call at
`main+0x374cab8`, with a requested 4ms cap, 3ms margin and 32 clean-sample
warm-up. The word-guarded observer at `main+0x38704c4` reads the original
present/acquire helper's final extra-VSync output. It observes game accounting,
not physical scanout deadlines or causation.

Three trials per mode and phase were predeclared; none were replaced.
Each cell below is trial 1; 2; 3 as R/M/D. X(...) retains observed image-index
values but rejects exact counts because the critical present identities have a
gap. Baseline has 13/18 eligible trials; delay has 16/18.

| Mode | Phase | Baseline trials 1; 2; 3 (R/M/D) | Delay-enabled trials 1; 2; 3 (R/M/D) |
|---|---|---|---|
| Vanilla | normal | 6/3/3; 6/3/3; 6/3/3 | 6/3/3; 6/3/3; 6/3/3 |
| Vanilla | delayed | 7/4/3; 6/4/2; 6/4/2 | 6/4/2; 6/4/2; 7/4/3 |
| LessLag | normal | 3/2/1; 3/2/1; 3/2/1 | 3/2/1; 3/2/1; 3/2/1 |
| LessLag | delayed | X(3/2/1); 3/3/0; X(3/3/0) | 3/3/0; 3/3/0; 3/3/0 |
| LLUltra | normal | X(2/2/0); X(2/2/0); 2/2/0 | X(2/2/0); 2/2/0; 2/2/0 |
| LLUltra | delayed | X(3/3/0); 2/2/0; 3/2/1 | 2/2/0; X(2/2/0); 3/2/1 |

The normal eligible results remain Vanilla 6, LessLag 3 and LLUltra 2: no new
four/five-frame saving was demonstrated. Raw interval 0 requests left the
effective SDK queue interval at 1. Training's mode 0 publication and first fighter
consumption occurred in the same update; no removable whole-frame FIFO was found.

| Mode | Phase | Applied n | Actual sleep min/p50/p95/p99/max ms | Game extra VSync events | Extras paired with sleep |
|---|---|---|---|---|---|
| Vanilla | normal | 1551 | 3.853 / 4.079 / 4.116 / 4.127 / 4.193 | 6 | 4 |
| Vanilla | delayed | 1461 | 4.008 / 4.084 / 4.119 / 4.131 / 4.647 | 6 | 6 |
| LessLag | normal | 1509 | 3.386 / 4.082 / 4.116 / 4.129 / 4.171 | 7 | 4 |
| LessLag | delayed | 1684 | 4.010 / 4.083 / 4.115 / 4.123 / 4.219 | 5 | 1 |
| LLUltra | normal | 1670 | 0.640 / 4.069 / 4.112 / 4.126 / 5.739 | 2 | 2 |
| LLUltra | delayed | 1705 | 1.272 / 4.072 / 4.115 / 4.126 / 5.198 | 0 | 0 |

Each event table row covers one complete 30-second API export (zero skipped
events, 1800 present observations). The four trial windows without sleep were
Vanilla normal trial 3, Vanilla delayed trials 1 and 3 and LLUltra delayed trial 1; guards and warm-up can
suppress delay despite the setting. They remain in the declared matrix.
Actual sleep can exceed the request due to host scheduling; the longest observed
was 5.739ms. Baseline has no exact extra-VSync observer, so its event counts are
not available. These events do not establish that sleep caused a missed scanout.

Acquire-return spacing, p50/p95/p99/max milliseconds (emulator evidence):

| Mode | Phase | Baseline | Delay-enabled |
|---|---|---|---|
| Vanilla | normal | 16.666 / 16.853 / 17.173 / 17.931 | 16.668 / 16.830 / 16.967 / 17.729 |
| Vanilla | delayed | 16.666 / 16.846 / 17.050 / 20.281 | 16.668 / 16.814 / 16.892 / 17.463 |
| LessLag | normal | 16.670 / 16.904 / 17.417 / 18.299 | 16.664 / 16.837 / 16.965 / 17.890 |
| LessLag | delayed | 16.666 / 16.869 / 17.290 / 19.502 | 16.665 / 16.815 / 16.873 / 17.812 |
| LLUltra | normal | 16.668 / 16.846 / 17.674 / 32.933 | 16.662 / 16.828 / 16.900 / 18.093 |
| LLUltra | delayed | 16.666 / 16.839 / 17.111 / 17.858 | 16.663 / 16.815 / 16.882 / 17.574 |

These are chronological runs, not a randomized causal performance test.
The integer proxy can hide sub-frame improvements or change by one at a phase
boundary. It cannot establish a uniform physical frame saved. Immediate has no
deliberate OneBehind slot; LessLag retains that slot and its scheduling
tolerance. No absolute console button-to-photon floor or online parity is proved.

Host validation of the measured code: clippy all-targets with warnings denied,
no-default-features build, 239 unit plus 1 integration tests, and actual Skyline
package. Experiments remain separately default-off. The source package, optimized
symbols, NRO hashes, full timeline, lever ledger, raw captures and explicit
operator answer remain outside Git under the operator evidence root
`plugin-api-tangent-20261012/input-lag-1018/`: `REPORT.md`, `NOTES.md`,
`baseline-results.json`, `framedelay-v2-results.json`,
`frame-delay-run-metrics.json` and `artifacts/frame-delay-v2/`.

## Confirming on Switch

Repeat the controlled A/B comparison on the console using a high-speed camera
that sees an input indicator and the display together, or calibrated input and
display sensors. Hold controller, adapter, display mode, scene and console clock
settings constant. Measure the first visible response across repeated inputs and
phases. Report console results independently from emulator screenshots.

A Homebrew input-to-color application can characterize its own input/render
path, but subtracting its result from Smash does not automatically isolate the
emulator: the applications may poll, render and present differently. It is a
useful additional control only when those boundaries are independently measured.

Remove `INPUT_RECEIPT` after the experiment and restart to disable the diagnostic
strip. Retain the exact measured binary and matching symbols with the results.
