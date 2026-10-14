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
the earlier path, and no response trials have been taken on the new one.

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
