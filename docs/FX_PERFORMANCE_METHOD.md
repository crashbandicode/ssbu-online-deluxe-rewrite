# Reduced FX performance acceptance

The target is to approach **Off mode's frame-time stability under LessLag and
LLUltra**, while keeping a recognizable cue. Looking less busy is not sufficient.
Removing a decoration is useful only if it removes enough of the expensive work.

## Matched comparisons

For each move, record the exact NRO hash, game version, emulator build, CPU/GPU,
driver, graphics settings, render profile, FX scope, stage, fighters and inputs.
Use one emulator instance, 100% game speed, no FPS++ and no concurrent builds.
Retain the same mods and settings across On, Off and Reduced.

Warm the move and stage shaders before measuring. Save a Training state with
fixed position, facing, damage and charge conditions. Reload the same state
before each trial. Use the same recorded input, including the full hold for
Gigaflare. Do not compare different charge levels. Close menus and collect
screenshots outside timing windows when possible.

Run counterbalanced On, Off and Reduced blocks for both LessLag and LLUltra,
with at least five repetitions per condition when a spike can be reproduced.
Retain before/during/after intervals, not just idle FPS. Confirm from diagnostic
events that scope and profile were actually applied and that Reduced used its
intended path. Preserve move timing, damage/hits, visibility and post-effect
cleanup as separate checks.

## Critical-zoom reproduction

The operator clarified that the spike may occur during Special Zoom
(https://www.ssbwiki.com/Special_Zoom), rather than only the red final-stock
finishing screen. Ordinary attack captures do not establish critical-zoom
coverage. Verify an actual connecting move with its blue Special Zoom and keep
the red finishing-hit screen as a separate fixture. Retain a screenshot outside
the measured trials to identify which scene actually ran.

Keep the same stage, positions, damage, move and charge in On, Off and Reduced
under both LessLag and LLUltra. Keep intentional hitstop, camera timing and
cinematic slow motion unchanged. Measure presentation cadence around the hit;
a frozen animation during hitstop is not itself a dropped frame. Warm the
critical-zoom shaders as well as the move shaders.

The normal FX policy preserves critical-zoom backgrounds and bg_finishhit.
Therefore normal Off is not a comparator that removes those backgrounds. If all
three modes spike there, separate the move effects from the shared critical
presentation before changing policy. No removal of camera logic, hitstop,
gameplay state or simulation timing is implied by this measurement.

A diagnostic-only startup marker, `PROBE_CRITICAL_SCREEN`, allows the exact
`bg_criticalhit` / `sys_bg_criticalhit` names and their eight color variants
to be suppressed in scoped Off mode. It does not suppress the camera, hitstop,
slow motion, red finishing backgrounds or unrelated screen effects. Keep it
absent from user installations. Compare normal Off with diagnostic Off using
the same binary and fresh emulator processes, then repeat the original
condition to check drift. On and Reduced ignore this marker.

Diagnostic event 97 records the latched marker state at startup. Event 96
records each matched request during a finite timing capture (and the first 32
otherwise): argument A is the original Hash40; argument B uses bits 0–7 for
request path (1: screen, 2: screen-system, 3: ordinary) and bit 8 for actual
diagnostic suppression. A missing event or an unchanged screenshot does not
prove that the effect was suppressed.

## What the measurements mean

The plugin's bounded CAPTURE_TIMING trace records CPU-side guest present calls
at 19.2 MHz. Consecutive present-entry intervals can expose a game-side cadence
spike. They are **not** GPU completion, host display presentation, scanout or
physical input latency. The render-callback series is a separate observation
and must not be mislabeled as present timing.

Use actual per-frame intervals when available. Eden's smoothed status FPS can
corroborate a sustained drop; 1000/FPS is an effective period, not a recovered
individual-frame time series. A mean near60 does not prove that every frame
met its deadline.

For each condition report median, p95, p99 and worst interval, intervals over
16.67/20/25/33.33 ms, the longest consecutive run of slow intervals, and total
excess time above the frame budget during the move. Keep capture completeness,
dropped-event counts and observation boundaries in the result. Review raw plots
and repeated trials instead of treating one noisy maximum as decisive.

When On has a repeatable effect-specific cost above Off, also report the fraction
of that improvement retained by Reduced, using each matched block's excess time:

```text
retained_benefit = (excess_On - excess_Reduced) / (excess_On - excess_Off)
```

A value near 1 approaches Off; 0 retains none of Off's measured benefit.
Do not clamp negative or above-1 results, and do not compute this ratio when the
denominator is indistinguishable from run-to-run noise. Report the underlying
times and variation alongside it. A good ratio does not excuse missing cues or
remaining severe spikes; review both tail intervals and the move visually.

## Decision

- **On has no reproducible spike:** performance unverified. Do not claim Reduced
  fixes a Switch dip merely because the emulator stayed at60.
- **Off has the same spike:** the target effect may not be the bottleneck, the
  filter may not have reached the real request, or the measurement is dominated
  by another cost. Resolve that before tuning Reduced.
- **Off improves, Reduced does not:** Reduced fails the performance goal. Retain
  enough of the expensive body to recognize the move, but reduce its particles,
  lifetime, shader/overdraw work or other demonstrated cost further.
- **Reduced approaches Off across repeated matched trials while preserving the
  cue:** accept for the measured emulator configuration, then verify the same
  move on the first-generation Switch.

If emulator settings are changed to reproduce a dip, retain both the normal
setting result and the reproduction settings. Prefer accurate rendering and
a stage/scene that reproduces the actual move cost. Host contention, arbitrary
slow motion and a generally sub60 baseline are not substitutes for reproducing
the Switch effect-specific bottleneck.

The Switch comparison remains required for a Switch-performance claim.
No overclocking behavior is added to the rewrite.

Recorded experiments and limitations: [emulator FX results](FX_EMULATOR_RESULTS.md).

## Capacity-limited diagnostic traces

A trace ending with `reason=capacity` cannot support a full requested-duration
claim. In the c0 Palutena follow-up, the complete ten-second callback trace
provides the guest input anchor. An API slice is usable only when its actual
ordered, gap-free coverage independently includes the entire baseline and move
window plus a predeclared margin (B+3 seconds for the B..B+2.5-second window).
Keep the original capacity reason and skipped-event counts. Do not relabel
the file as elapsed or bypass the full-trace analyzer's rejection.

Report such observations as bounded CPU-side API intervals. They do not
measure GPU completion, displayed frames or input-to-photon latency. If the
coverage check fails, exclude the trial and retain the failed receipt.
