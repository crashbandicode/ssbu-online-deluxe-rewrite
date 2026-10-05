# Measuring performance and LessLag

The rewrite defaults to Vanilla rendering. The opt-in, restart-only `lesslag_core`
experiment has a two-texture ring and OneBehind flush/index behavior. Full LessLag
parity and Reduced FX remain unfinished. A constant 60 FPS does not establish low input latency, and a
callback interval is not a displayed-frame interval.

## Native latency baseline and the emulator hypothesis

The user identified the commonly documented native baseline of roughly six frames
(about 100 ms at 60 Hz). The original latency-slider explanation by a feature co-author
also reports six offline frames on a good monitor and distinguishes that from the
additional online buffer. Treat this as a reference observation, not a calibrated
constant to subtract from every result. [Original developer explanation](https://github.com/Naxdy/latency-slider-de/blob/master/README_orig.md#how-many-frames-does-it-actually-save).

Input sampling phase and controller/adapter polling can change the frame in which
an input is recognized; NaxGCC's author documents repeated controller experiments
using Training Modpack. [Polling experiment](https://gcc.naxdy.org/input-consistency.html#the-experiment).

Additional Eden latency is an unverified hypothesis for this host. Do not model
it as six guaranteed native frames plus a fixed emulator penalty. The emulator's
controller, scheduling and presentation paths differ from the console's, and the
whole measured result could differ in either direction. Establish four conditions:

| Platform | Baseline | Mod comparison |
| --- | --- | --- |
| Switch | Vanilla, FX On, offline | LessLag with other conditions fixed |
| Eden | Vanilla, FX On, offline | LessLag with other conditions fixed |

Report absolute input-to-visible-response latency and the within-platform change
in milliseconds and 60 Hz frame equivalents. Cross-platform differences describe
the complete setups unless controller, display and measurement conditions are
matched; current synthetic keyboard input is not equivalent to physical Switch
controller input. Record connection/polling, display refresh/game mode, compositor,
VSync and presentation settings. Keep online buffer experiments separate.
The callback traces below do not measure or confirm the six-frame native baseline.

## Available now: raw callback capture

An opt-in capture records ten guest-clock seconds of the existing ImGui render
callback, including callback sequence, raw system ticks, sampled HID buttons,
requested FX mode/scope and effective suppression state. It adds no game hook.
Storage is a fixed 1024-sample array; the callback uses `try_lock`, with explicit
skipped counts. Recording stops before the background logger exports a CSV.
No per-sample file write, allocation, or blocking lock occurs in the callback.

From Settings, use the existing debug shortcut (hold L+R and Plus or Minus, tap
Down), then hold L+R and tap Up to capture. `Timing` changes to `CSV saved` after
export. The debug view's capture shortcut uses L/R, not ZL/ZR.
The game's own input handling still receives these buttons; settle the scene
before the measured move. Menu operations must not be included in a move window.

For CLI-driven Eden tests, write an ASCII label (up to 48 letters, digits, `_`
or `-`) to `CAPTURE_TIMING` in the mod's data directory. The background thread
consumes that file and arms the capture. Wait for `timing-status.txt` to say
`capturing`, then perform the move. A repeated controller request during capture
is ignored. A file request left while busy waits for the next capture.

Files live at `sd:/ultimate/ssbu_online_deluxe_rewrite/timing-<tick>.csv`.
On this host, the data directory is:

    /home/intpa/.local/share/eden/sdmc/ultimate/ssbu_online_deluxe_rewrite

Run from the repository using the required PowerShell shell:

    /usr/bin/python3 ./tools/analyze_timing.py /path/to/timing-123.csv --json

The analyzer reports callback interval percentiles, maximum and counts above
20/25/33.333 ms. These thresholds are descriptive, not calibrated missed-frame
thresholds. Intervals spanning missing callback records are excluded and flagged.
Mixed settings are flagged. Truncated, unfinished and nonmonotonic traces are
rejected. The footer and declared sample count identify incomplete file writes.
The 30-second background timeout exports an explicitly aborted trace if the
callback does not finish the capture. That timeout also uses the guest clock.

The button field is a sample taken on the render callback. It does not timestamp
the physical button press or the game's simulation input read. Do not calculate
input-to-photon latency from it. Tick conversion is 19.2 MHz, matching the archived
`GetSystemTick` conversion in `reference/archive/src/profiling.rs`.

## Evidence required for LessLag

| Measurement | Source | Claim it can support |
| --- | --- | --- |
| Callback / present API invocation timing | Existing callback and chained NVN wrappers | CPU scheduling and time inside a call |
| Presentation history | Validated `nn::vi::ListFrameHistory` data | Compositor bookkeeping; platform semantics must be verified before making display claims |
| Input to visible response | High-speed camera with controller and display in shot, or calibrated hardware | End-to-end input latency |

Fable 5.1/xhigh reviewed the design. We rejected its initial interpretation of
`nvnQueuePresentTexture` call timing as actual presentation and its concurrent
plain-array ring design. The accepted first implementation is narrower: a finite,
locked callback capture. Chained NVN observations and an opt-in history probe have
since been added; neither is a substitute for an input-to-visible-response measurement.

The archived source declares `ListFrameHistory`, `GetLatestFrameNumber` and a
40-byte `FrameInfo`. The original binary imports those functions. The archive
captures its layer through a game offset; those offsets must not be copied into
13.0.5 without validation. An older SDK symbol map also lists `GetNativeWindow`.
The implementation now captures that layer through the resolved symbol and guards
its lifetime with a DestroyLayer hook. Live 13.0.5 code and SDK symbol mapping
validated the 40-byte record and conversion to 19.2 MHz system ticks. This SDK
copies present time into its vblank field, so these are not independent measurements.
Failure must report presentation data unavailable. A symbol name in an old map
alone is not proof of current runtime availability or engagement.

Present-call records cannot be matched to history by texture index alone: indices
are reused. Verify a frame-number association before reporting queue latency.
Likewise, distinct observed texture indices are only a lower bound on textures in
use unless the actual window configuration is measured.

For latency acceptance, compare Vanilla and LessLag using the same character,
move, game input buffer, display and controller. Film many button presses at
240 FPS or use a calibrated trigger/capture setup. Count to the same first visible
response and report the distribution and capture uncertainty. Keep the gamepad
overlay as a useful visual reference, not a substitute for the physical input
timestamp. Rendering can reduce latency while both conditions remain at 60 FPS.

## Controlled FX experiment

Gigaflare is a provisional first candidate because the fork's resolution scaler
explicitly targets it. It is not yet established here as a measured Switch
bottleneck. Use a fixed stage, fighter/CPU, position, damage, camera, render profile
and input sequence; restore the same Training Modpack state before each trial.
Warm the scene and move at least three times, then alternate On/Off in balanced
ABBA blocks with at least ten measured repetitions per condition. Capture menu
changes and state loads outside the move window. Preserve each raw trace, exact
NRO hash, configuration and trial label. Report per-trial results and variation;
do not infer improvement from one favorable maximum or a pooled average.

Use Off to estimate savings for the targeted layers. Failure of the peer's one
scaling strategy does not rule out other Reduced designs. Later compare Reduced
against both On and Off, preserving gameplay cues and confirming damage and state
behavior. A Vanilla result alone cannot verify a LessLag-specific dip.

On Eden, keep speed at 100%, one instance, the recorded docked/resolution settings
and asynchronous shaders off. Distinguish warm repeatable cost from first-use
shader compilation; warm-up alone is not proof of the cause of a spike. Record
host power mode, GPU load, settings and capture overhead. Current baseline uses
Immediate presentation; a separately recorded FIFO trial may be useful later.
Do not lower the game speed limit to manufacture a Switch-like dip. A host GPU
bottleneck under a higher-resolution stress test remains an emulator stress test,
not a demonstrated reproduction of the console problem.

Validate the guest clock against host wall time, including a controlled pause.
If a pause is absent from the guest trace, document that limitation. A video-based
repeat count additionally needs continuous expected motion, verified capture
timestamps and a control run to distinguish capture loss from game repeats.

## Eden instrumentation validation, 2026-10-05

Installed bundle `bf4df24b3b8cc176` (full NRO SHA-256
`bf4df24b3b8cc176cf7fd41f111266ab35f2400061797a9695695d538679fba2`)
exported five complete captures with zero skipped/missing records. Exact CSVs,
analyzer output, host timing receipts, selected emulator settings, screenshots
and logs are retained outside Git in that bundle's
`runtime-evidence/eden-timing-20261005/` directory.

- Idle Smash submenu: 602 samples, 10.016 guest seconds; request through observed
  export took 11.109 host seconds, including service/export/polling overhead.
- F4 pause control: a 4357.982 ms callback interval matched the approximately
  4355.459 ms separation between identically invoked pause/resume input helpers.
  These helper times bound the test; they are not exact Qt event timestamps.
  This Eden configuration's system-tick clock continued during the host pause.
- The debug-view L+R+Up shortcut exported a complete 601-sample trace.
- Training Sephiroth/Lucina: On and Off each exported a complete ten-second trace,
  with 132 nonzero HID samples for a 2.2-second neutral-B hold. The Off trace
  reported effective suppression. Full Gigaflare charge was not visually established;
  the operator identified the hold as too short. These are neutral-B capture checks,
  not confirmed Gigaflare trials. Training Modpack and SGPO remained loaded.
- An invalid UTF-16 request was consumed and reported once, with no repeated
  status-file writes. A later valid request succeeded.

These are capture acceptance tests, not an FX performance comparison: the match
trials were not warmed/repeated in balanced blocks and included screenshot
readback. No displayed-frame, LessLag or end-to-end latency claim follows.
Capture overhead against an independently measured baseline remains unquantified.

The first runtime export exposed a toolchain behavior in skyline-v3
`library/std/src/sys/fs/switch.rs`: `create_new(true)` rejects existing paths but
does not create missing ones unless `create(true)` is also set. The exporter now
sets both. Existing captures remain protected from replacement by this single
background writer. No global toolchain modification was made.

## Experimental core and API spans, 2026-10-05

Set render_mode=lesslag_core in config.ini while the game is closed. Vanilla remains
the distributed default. The experiment verifies each 13.0.5 instruction before
installing index, zero-timeout render-poll and flush hooks. It asks NVN for two
active textures immediately after window initialization, before first acquisition,
and enables its mode only after readback succeeds. Failed prerequisites keep
Vanilla behavior; an unsuccessful texture-count rollback stops with diagnostics.

This is a partial reconstruction: it omits the original render-task sequencing,
texture-wait relocation, pacing, dynamic resolution, clock changes and automatic
profile transitions. It does not touch SGPO's 0x374812c neighborhood. The remaining
CPU texture wait is intentional for this first experiment. Doubles work is parked.

Each CAPTURE_TIMING request now also writes api-<tick>.csv, a bounded 4096-event
capture using try_lock. It records CPU spans for present, acquire, the acquired
texture's CPU wait, zero-timeout waits and (core mode only) dispatch flush. Analyze
with python3 tools/analyze_render_trace.py /path/to/api-123.csv. The analyzer
rejects incomplete captures and reports skipped events. Spans can overlap; do not
sum them. Acquire's result is unavailable through the pinned backend's void ABI,
so -1 is recorded rather than fabricating a return value. The two ten-second
capture windows start on their respective first observations, not an identical
display frame. An occasional one-event difference is not itself a dropped frame.

renderer-status.txt gives actual mode, startup state, texture readback, index masks
and hook counters. In bundle 79790e610f0f24f8, LessLag core ran in Eden with three
allocated/two active textures, interval 1, present/acquire indices {0,1}, progressing
index/flush/poll counters, the global overlay and SGPO. Full console parity,
runtime transitions and end-to-end latency remain unverified.

An opt-in DUMP_PRESENTATION_HISTORY file installs symbol-bound layer lifetime
observers and runs a finite history probe. presentation-history.txt records raw
fields, query costs and explicit outcomes; no hook or query is installed without
the boot flag. The probe completed in Eden: history disabled gave empty data;
enable_buffer_history=true gave queued entries but zero presented timestamps and
GetLatestFrameNumber returned 0x1672. It ended as no_frames_yet. This is a negative
result for usable presentation data in the tested startup window.

Eden's current public implementation gates GetBufferHistory on that setting and
sets its presentation_time field inside QueueBuffer. That alone cannot establish
host display presentation. See [producer code](https://github.com/eden-emulator/mirror/blob/master/src/core/hle/service/nvnflinger/buffer_queue_producer.cpp)
and [history storage](https://github.com/eden-emulator/mirror/blob/master/src/core/hle/service/nvnflinger/buffer_queue_core.cpp).
The installed 0.2.1 runtime result is the evidence above; current main source is
context, not proof that every line matches that binary. Temporary history and TAS
settings were restored afterward. TAS keyboard recording was verified in a
temporary directory; playback-driven trials have not been validated.

### Full-charge Gigaflare pilot

The operator visually confirmed the longer hold as full Gigaflare. After three
longer-hold warm-ups, four trials used a 4.2-second B hold, the same Sephiroth/Fox
match and Training stage, CPU stopped, default reset-position chord before each
trial, and hidden Deluxe window. Input was synthetic keyboard input, not a
frame-indexed TAS replay. The installed cosmetic makes Fox look like Wolf.
No screenshots were captured during the measured windows. FX scope was all,
input buffer Auto, render mode lesslag_core, Eden Vulkan Balanced/docked/1x,
speed 100%, async shaders/presentation off; buffer history was enabled.

| Order | FX | Capture id | Callback p95 ms | Callback max ms | Flush p95 ms |
| --- | --- | --- | --- | --- | --- |
| 1 | On | 828799654813 | 17.085 | 17.411 | 1.427 |
| 2 | Off | 830147876733 | 17.087 | 21.917 | 1.556 |
| 3 | Off | 830764106999 | 17.089 | 24.505 | 1.502 |
| 4 | On | 831957155284 | 17.094 | 20.106 | 1.467 |

All traces completed without skipped callback/API records; present indices were
0/1. Raw traces, host receipts, analyzers' JSON and full-charge visual references
are retained in dist/79790e610f0f24f8/runtime-evidence/lesslag-core/ outside Git.
This small pilot did not show an On-related dip and cannot establish savings.
It is not the planned ten-repetition-per-condition experiment, a GPU benchmark,
or an input-latency comparison. Earlier 2.2-second tests, including filenames
containing gigaflare, are only short neutral-B instrumentation checks.
