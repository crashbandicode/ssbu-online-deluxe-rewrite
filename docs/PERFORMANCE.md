# Measuring performance and LessLag

The rewrite defaults to Auto. Live profiles are Vanilla, LessLag, LLUltra, and
LLDoubles. Hooks install once. Later profile, texture-count, index, flush, and
resolution changes go through the guarded machine in `src/render_policy.rs`,
fed by `src/runtime/auto_driver.rs`. Since 0.5.0 Deluxe has no character-FX
hooks: On, Off and Reduced are the separate ssbu-fx-control plugin. Open
Reduced branches and the still-unshown Off-like frame-time benefit moved with
it, to `docs/FX_RUNTIME_MATRIX.md` and `docs/FX_PERFORMANCE_METHOD.md` in that
repository. The FX results below are dated records of 0.4.2 and earlier builds.

`lesslag_core` and `doubles_core` stay INI-only startup experiments. They are
separate from the four live profiles. Hardware clocks stay excluded. Software
FPS boost stays a gated diagnostic; the normal menu reports Unsupported.

A constant 60 FPS does not establish low input latency, and a callback interval
is not a displayed-frame interval. Every percentile, captured-image count, and
Gigaflare pilot below stays bound to the NRO named in that section. A later
compatibility pass does not inherit those numbers.

The c0 Palutena follow-up completed 36 measured hit/whiff trials across
Vanilla/LessLag/LLUltra and On/Off/Reduced. It did not establish a consistent
On-specific penalty. Capacity-limited API captures were accepted only for
individually verified complete move-window slices; no GPU, displayed-frame,
hitbox-cost or Switch claim follows. See [results and limitations](https://git.syntaxbyexample.com/intpa/ssbu-fx-control/src/branch/main/docs/FX_EMULATOR_RESULTS.md).

## Registered render profiles

`src/runtime/mod.rs` installs the renderer, then `resolution::install` and
`fighter_probe::install`. `src/runtime/renderer.rs` `install` preflights the
13.0.5 words, installs the index, render-wait, and flush hooks, then calls
`render_sequence::install` and `render_ultra::install`. `src/render_policy.rs`
keeps Vanilla, LessLag, LLUltra, and LLDoubles on one atomic word. A texture-count
change drains before commit. A transition does not reinstall hooks.

| Profile | Active textures | Index | Live flush when committed | Render opts | Resolution / dynamic resolution |
| --- | --- | --- | --- | --- | --- |
| Vanilla | 3 | TwoBehind | original once, fields off | No | 1080p, off |
| LessLag | 2 | OneBehind | original once, LessLag fields; since 0.4.0 presents the acquired image at the flush that submits it (`render_policy::present_at_flush`) and skips the game's later present of that index | Yes | console 1080p on; emulator 1080p off |
| LLUltra | 2 | Immediate | original three times, fields off, early flush; since 0.5.2 presents the acquired image at this flush like LessLag | Yes | console 576p on; emulator 1080p off |
| LLDoubles | 3 | OneBehind | original once, LessLag fields; since 0.5.2 presents the acquired image at this flush like LessLag | Yes | 1080p on, both platforms |

Sources: `src/render_policy.rs` `Profile` and `plan_flush`; `src/resolution.rs`
`defaults_for` and `delay_for_profile`; `src/config.rs` `RenderMode::textures`
and `render_opts`. LLDoubles is `Profile::Doubles`. `DoublesCore` is the
startup experiment, not this profile.

render_opts moves fighter model updates and fighter command recording after the
scene update and dispatches the render queue there, using one per-frame decision
shared by all four hooks. The queue lock and its +0x20 header byte follow the
game's own dispatcher rather than the archive's offset bug. LLUltra's pre-present
flush runs from an entry hook on 0x38701e0, guarded by the saved caller address,
owner alias and virtual flush target; SGPO's 0x374812c neighborhood is not patched.
The 0x38701e0 entry hook is installed in every mode and only flushes when
Immediate (LLUltra) is committed; in any other mode it counts the entry and
returns without flushing or changing the mode. There is no separate
observer flag (`src/runtime/render_ultra.rs`, `renderer.rs` `install`).

**Teardown guard (candidate, 2026-10-15).** The relocated model update runs
after the scene update, so on the frame that leaves Training or a match it
would touch fighters the scene update has just destroyed; four Eden runs on
2026-10-14 aborted there in `nn::os::LockMutex` (kernel `InvalidHandle`) with
`deluxe_rewrite_post_scene` on the stack (VALIDATION.md, "0.5.0"). The
module-unload hook now bumps an epoch; the fighter-recording hook snapshots
it per frame; when the post-scene hook sees the epoch changed within the
frame it skips the relocated model update and fighter worker (the deferred
queue is still dispatched), records code 48 and hands the next 180 frames
back to the game's own ordering (`src/teardown_guard.rs`,
`render_sequence.rs`; `teardown_guard_skips` in `renderer-status.txt`).
Render opts are therefore off for about three seconds after every scene
transition and otherwise unchanged. Not yet verified in Eden or on the
console.

The LessLag path reads the gate byte at `+0x1d18` and the dispatch index at
`+0x1d20`, then writes `+0x1d30` and `+0x1d28` when the gate is nonzero.
Diagnostic candidate `c0e9a625eb44a735` samples those fields and checks whether
the latest host EndRecording handle occurs in the incoming command list during
finite API captures. It passes 247 host tests and all required build checks.
The stores remain unchanged. In a fresh single-instance capture the window is
visible, the gate is one, and the wide field flips between zero and one. The
recorded host handle is present in all 273 sampled incoming lists. This does
not explain the earlier b99 concurrent-run invisibility in LessLag/LLDoubles
or observe the inner driver's appended UI handle.

Texture-wait relocation, the VSync-event wait, and the counter-thread patches
stay absent. v1.4.1 and v1.4.2 install them only when the profiling bit is set,
and the shipped presets leave it off. The frame pacer stays enabled.

## Local match note, outside the latency record

b99 (`b99f087cf8ef0a9c`) completed one two-local-station Final Destination match,
Game & Watch versus Sephiroth. Both stations reported the same winner, two KOs,
and 47% damage, then returned to stage selection. LessLag and then LLUltra were
each confirmed active. The Game & Watch plugin `bceef` was on A only. A used FX
Reduced with online scope. B used FX On with scope all. Standalone ImGui and
Elite were on B only. No disconnect was observed. Both sides held cleanup and
requested Vanilla at results. The window sat on the right during connected CSS
and stage select, and on the left during the match. The opponent column
disappeared after B stopped. This match has no full-state or checksum proof. A
three-station match is still undone. High-RTT, VPN, NSO, and Switch performance
are not accepted from it. b99 Reduced branch limits stay in
`docs/FX_RUNTIME_MATRIX.md` (now in ssbu-fx-control). The Switch already has b99 installed; that install
is not a console performance acceptance.

The sections from "Native latency baseline" through "Final renderer regression"
are the exact-artifact record for the NROs they name.

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
| Switch | Vanilla, original effects, offline | LessLag with other conditions fixed |
| Eden | Vanilla, original effects, offline | LessLag with other conditions fixed |

Report absolute input-to-visible-response latency and the within-platform change
in milliseconds and 60 Hz frame equivalents. Cross-platform differences describe
the complete setups unless controller, display and measurement conditions are
matched; current synthetic keyboard input is not equivalent to physical Switch
controller input. Record connection/polling, display refresh/game mode, compositor,
VSync and presentation settings. Keep online buffer experiments separate.
The callback traces below do not measure or confirm the six-frame native baseline.

## Available now: raw callback capture

An opt-in capture records thirty guest-clock seconds (ten before 0.3.1) of the
existing ImGui render callback, including callback sequence, raw system ticks,
sampled HID buttons and overlay view (schema 1, through 0.4.2, also carried
the requested FX mode/scope and effective suppression state). It adds no game hook. Storage is a 2048-sample buffer (1024 before 0.3.1)
that the diagnostics thread drains and streams to the CSV every poll; the
callback uses `try_lock`, with explicit skipped counts. Since 0.3.1 the
`CAPTURE_CSS` boot flag arms the same capture automatically on character-select
entry (label `css`) and adds the read-only dispatcher/UI rows described in the
README; the `CAPTURE_MATCH` boot flag arms it for a whole match (label
`match_*`, ended by the scene change, ten-minute ceiling).
No per-sample file write, allocation, or blocking lock occurs in the callback.

From Settings, use the existing debug shortcut (hold L+R and Plus or Minus, tap
Down), then hold L+R and tap Up to capture. The buffered-export source correction
shows `Capturing`, then `Exporting CSV`, then `CSV saved` only after both
CSVs are flushed/synchronized and the saved status is persisted. Errors show
`Export error`, even if writing the error status file also fails. The debug view's
capture shortcut uses L/R, not ZL/ZR.
The game's own input handling still receives these buttons; settle the scene
before the measured move. Menu operations must not be included in a move window.

For CLI-driven Eden tests, write an ASCII label (up to 48 letters, digits, `_`
or `-`) to `CAPTURE_TIMING` in the mod's data directory. The background thread
consumes that file and arms the capture. Wait for `timing-status.txt` to say
`capturing`, then perform the move. A repeated controller request during recording
or export is ignored. A file request left while busy waits for the next capture.

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
A wall-clock watchdog on the diagnostics thread (the capture's guest duration
plus ten seconds) ends a capture whose callback never finishes it and exports
it with `# reason=aborted`; a match capture ends with `scene_end` when the
game leaves the stage. The `# engine` footer is the profile at arm time, or
`Mixed` when the effective profile differed at any poll while recording.
The watchdog does not cancel blocked native filesystem calls. The CSV export uses a bounded
64KiB buffer per file with an explicit fallible flush before `File::sync_all`.

October9 Switch evidence on installed `5c72a0b54b8c8c95`: recording finished, but
the API CSV stopped mid-row6308 of6602 while the callback CSV was complete602
samples. The old status stayed Capturing through export. Skyline std requests
a native flush for every unbuffered formatting fragment; this motivated the
buffering/status correction above. Source/test/build evidence is separate from
device export acceptance. The new correction is not yet deployed. The operator
already tried repeated waits; do not request another uninstrumented wait.

The button field is a sample taken on the render callback. It does not timestamp
the physical button press or the game's simulation input read. Do not calculate
input-to-photon latency from it. Tick conversion is 19.2 MHz, matching the archived
`GetSystemTick` conversion in `reference/archive/src/profiling.rs`.

## Evidence required for LessLag

| Measurement | Source | Claim it can support |
| --- | --- | --- |
| Callback / present API invocation timing | Existing callback and chained NVN wrappers | CPU scheduling and time inside a call |
| Presentation history | Validated `nn::vi::ListFrameHistory` data | Compositor bookkeeping; platform semantics must be verified before making display claims |
| Captured composites carrying the INPUT_RECEIPT marker | Emulator screenshots decoded with tools/response_marker.py | Captured-image response frames in that emulator setup |
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

Since 0.5.0 the On/Off/Reduced switch is the ssbu-fx-control plugin, not
Deluxe; its current method is that repository's `docs/FX_PERFORMANCE_METHOD.md`.
This section is the method the 0.4.2 and earlier records followed.

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

This section records the `lesslag_core` experiment and bundle `79790e610f0f24f8`.
It is not a description of the current live profiles above.

Set render_mode=lesslag_core in config.ini while the game is closed. Vanilla remains
the distributed default of that experiment's INI. The experiment verifies each 13.0.5 instruction before
installing index, zero-timeout render-poll and flush hooks. It asks NVN for two
active textures immediately after window initialization, before first acquisition,
and enables its mode only after readback succeeds. Failed prerequisites keep
Vanilla behavior; an unsuccessful texture-count rollback stops with diagnostics.

That core build omitted the original render-task sequencing,
dynamic resolution, clock changes and automatic profile transitions. It does not
touch SGPO's 0x374812c neighborhood. The retained CPU texture wait matches the
shipped modern presets; its relocation is profiling-only in the modern hosts.
The later `lesslag` and `llultra` modes on that line of work added sequencing.
Current source registers sequencing, resolution, LLDoubles, and live transitions
together; see the registered-profile section.

Each CAPTURE_TIMING request also writes api-<tick>.csv, a capture using
try_lock (4096 events in this build; 8192 later; 65536 resident events in
0.3.1–0.3.4; since 0.3.5 the rows are packed to 20 B and streamed: hooks fill
one of two chunks (4096 rows in 0.3.5–0.3.6, 2048 rows = 80 KiB in `.bss`
since 0.3.7), and the diagnostics thread
drains the other into the open CSV every 250 ms while a capture runs (500 ms
otherwise), so the capture is bounded by
the 30 s duration, not by memory. The CSV header is written when the capture
arms and `engine`, `reason`, `skipped` and `event_count` move to the footer;
`skipped` now also counts rows lost when a chunk filled before its drain, so
a non-zero value still means "not an A/B trace"). It records CPU
spans for present, acquire, the acquired texture's CPU wait, zero-timeout waits
and (non-Vanilla modes only) dispatch flush. Analyze
with python3 tools/analyze_render_trace.py /path/to/api-123.csv. The analyzer
rejects incomplete captures and reports skipped events. Spans can overlap; do not
sum them. Acquire's result is unavailable through the pinned backend's void ABI,
so -1 is recorded rather than fabricating a return value. The two ten-second
capture windows start on their respective first observations, not an identical
display frame. An occasional one-event difference is not itself a dropped frame.

renderer-status.txt gives actual mode, startup state, texture readback, index masks
and hook counters. In bundle 79790e610f0f24f8, LessLag core ran in Eden with three
allocated/two active textures, interval 1, present/acquire indices {0,1}, progressing
index/flush/poll counters, the global overlay and SGPO. For that core bundle,
full console parity and end-to-end latency remain unverified. Live transitions
in current source are a separate, later registration.

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

## Custom CLI Eden acceptance, 2026-10-05/06

The delivery at /home/intpa/eden-cli-handoff was accepted after READY and all
4644 SHA256SUMS entries verified. AppImage SHA-256:
fd4b91536e3d60f9c7dacddaaa01a536d6999a16c4a985a411a6e7c8ed71598b.
Its source base is 10bcd2d849843b146a79c61a21df615e1bdcfcde plus the supplied
CLI patch; it is not the stock 0.2.1 binary. Do not pool their measurements.
All runs used NRO 5d47c70452ce73ba from source 42a60afbe814.

The CLI skipped the intro, navigated to Training, injected controller input,
read runtime state and captured native X11 screenshots. The Vanilla run checked
the Deluxe global window, SGPO, and opening/closing the Training Modpack menu.
LessLag core reached the same Training-stage Sephiroth/Ganondorf pairing with
SGPO and the global window. Both modes accepted live FX changes. Rendering
status showed three active textures in Vanilla and two in LessLag core, with
the latter's index/poll/flush counters advancing. These are functional checks
of NRO `5d47c70452ce73ba`. That artifact's full LessLag profile was still incomplete.

Vulkan, docked, 1x, speed 100%, Bilinear, asynchronous shader compilation and
presentation off. Custom source reports GPU accuracy High (UI label Accurate).
The custom build changed some defaults: the per-title scaling default loaded
Nearest even though the old INI stored Bilinear. It was explicitly restored to
Bilinear and persisted with `scaling_filter\default=false`. Changing the active
custom setting through settings.set and calling settings.save did not persist
that per-title override across restart; verify effective settings after loading
the game. No claim that the graphics configuration equals stock is made.

| Mode | FX / scope | Capture | Callback p95 ms | Max ms | Samples / API records |
| --- | --- | --- | --- | --- | --- |
| Vanilla | On / all | 914928122785 | 16.882 | 18.583 | 602 / 2403 |
| Vanilla | Off / all | 915986659695 | 16.927 | 18.598 | 602 / 2403 |
| LessLag core | On / all | 929403436991 | 16.946 | 17.959 | 602 / 2402 |
| LessLag core | Off / all | 930702295620 | 16.907 | 17.788 | 602 / 2402 |
| Renderer bypass | Off / online, offline Training | 940587468188 | 16.939 | 79.244 | 597 / 2385 |

Each capture completed with zero skipped/missing records and no analyzer warnings.
No screenshots ran during capture windows. The first four contain 252 or 253
successive callbacks with HID button mask 0x2 (B) set, spanning 4183–4200 ms between
first and last observations for a requested 4200 ms CLI lease. This corroborates
guest-visible input; the acknowledgment alone only proves frontend acceptance.
The two Off/all captures contain actual ACMD suppression events for hashes
0x10df40d690 and 0x11ea056d9f. This is stronger than observing the configured flag.

The bypass test left render_mode=lesslag_core in the INI and added
DISABLE_RENDERER before boot. Startup logged the bypass, actual mode remained
Vanilla, active textures were three, and render-patch counters stayed zero.
It also tested effects=off with effects_scope=online in Training against Dark Pit:
filtering stayed inactive, no suppression events occurred during capture, and
the visual effect remained visible in a separate screenshot. The 79 ms callback
gap is retained; this first-shot scope test is not a warmed performance trial,
and its cause was not isolated. The recovery flag was removed afterward.

These single smoke pairs do not establish an FX performance benefit or input
latency. CLI leases use a host timer, not frame scheduling; no latency estimate
is derived from them. Full traces, screenshots, input observations, effective
settings snapshots and custom-acceptance.json are retained under
dist/5d47c70452ce73ba/runtime-evidence/custom-* outside Git. The three supervisors
used 8 GiB RAM limits, zero swap and finite 900/600/600-second lifetimes; all ended
by their runtime limit with zero recorded OOM kills. Observed peak RAM stayed
below 7.3 GiB. An expected runtime_limit exit is not a game crash.

### Objective latency work after the CLI migration

Fable 5.1 xhigh reviewed the custom source; the primary checked and corrected the
measurement boundaries. The revised report is in the debug bundle's
review-evidence/custom-eden-measurement-review.md. Key acceptance requirements:

- Eden's OnFrameDisplayed is a composite callback. It runs even on a hidden-window
  early return and can precede asynchronous host presentation. TAS advances there,
  so a TAS index is not automatically a game-frame or display-frame index.
- Carry unique queue/layer/frame identity through the compositor, renderer and
  actual host present call. Matching repeated texture slots or nearby timestamps
  alone cannot establish causal correspondence. Guest parcel timestamps need
  runtime validation before treating them as a bridge between clocks.
- Identify the first frame whose content reacts to the input. An FX request marks
  simulation work, not the first visible reaction; inferring that frame from the
  very render delay being tested would be circular. Bounded frame-content capture
  or an independently synchronized capture is still required.
- Predeclare warm-up and independently evidenced interruptions. Retain genuine
  stalls, report distributions and paired differences, and keep runs independent
  when estimating uncertainty. Do not discard every FPS dip.

No emulator instrumentation patch or TAS playback validation is claimed here.
The next measurement should distinguish input availability, simulation response,
render/composite submission and physical display. The documented native six-frame
reference remains a reference, not a correction subtracted from these traces.

## Captured-image response measurement

Create an empty INPUT_RECEIPT file in the mod's data directory before boot. The
existing ImGui callback then draws a 28-cell strip at the bottom right: magenta
and cyan anchors, the low 24 bits of the NVN present serial (least significant
bit first), one Down bit (left stick Y below -16000) and parity over those 25
bits. It uses the existing HID read and adds no gameplay hook. Down therefore
marks the first poll by this callback that saw the input, not when HID received
it. Without the file nothing is drawn; normal users never see the strip.

While an API capture is running, each marker poll also records kind 6 (serial,
raw left Y) and kind 7 (serial, low 32 bits of the HID sampling counter) at the
same guest tick. The counter is not a clock bridge. Recording does no allocation
or file I/O in the callback or hooks; CAPTURE_TIMING exports raw 19.2 MHz ticks
afterward from the background thread.

`python3 tools/response_marker.py <directory>` decodes images named 000.png and
so on, and rejects missing anchors, ambiguous cells, parity failures and any
step other than one present per image. The current Eden trials took native
640×360 PNG screenshots at full rate with a 1 ms file poll, injected the stick
input right after image 5, and found the first image whose fighter region
changed to the crouch pose. The capture, ROI and batch helpers are machine-local
experiments, not a supported interface.

Three counts come from each valid trial: images from injection to the first
crouch image (R), images to the first Down marker, and presents between the
marker and the pose (D). R equals the marker count plus D.

## Full LessLag/LLUltra response trials, 2026-10-06

Accepted for the captured-image response in this setup after Fable 5.1/xhigh
review of the raw images, serials and API records. NRO 123d47db700f5d19 (SHA-256
123d47db700f5d196161b9fd1af673d15214657d2364862c5dc4ea095be120f0, build
0.1.0-5f556ad717f5-dirty-1791249872, source bundle retained); custom Eden
10bcd2d849 with its CLI patch, Vulkan, docked, 1x, 100% speed; game 13.0.5,
Sephiroth against Kazuya in Training; Training Modpack and SGPO installed;
FX On/all; input buffer Auto. Input was synthetic CLI stick-down. The baseline
is Vanilla rendering with the same mods, not unmodified Smash.

R is the first crouch image index minus image 5, the last image before injection.
D is the first crouch serial minus the first image carrying the plugin's Down
marker. Every accepted trial has consecutive identities through this window.

| Mode | Valid normal-phase trials | First Down image | First crouch image | R | D |
| --- | --- | --- | --- | --- | --- |
| vanilla | 4 of 5 | 8 | 11 | 6 | 3 |
| lesslag | 7 (2 earlier, 5 fresh) | 7 | 8 | 3 | 1 |
| llultra | 5 of 5 | 7 | 7 | 2 | 0 |

All valid normal-phase trials in a mode gave the same counts. The excluded
Vanilla trial has a present-identity gap (22180 to 22182) and is retained.
The first fresh LessLag trial had a 47 ms capture interval with continuous
identities: its frame count stands, but it is excluded from steady-cadence
wall-time summaries. The other measured intervals were about 16.7 ms.

The control phase injected about 8.24 ms after image 5, with the next screenshot
already queued so the deliberate delay did not itself skip an image:

| Mode | Trials | R, each trial | D, each trial |
| --- | --- | --- | --- |
| vanilla | 3 | 7, 6, 6 | 3, 2, 2 |
| lesslag | 3 | 3, 3, 3 | 0, 0, 0 |
| llultra | 3 | 3, 2, 2 | 1, 0, 0 |

All nine control trials have continuous critical-window identities. The first
LessLag control has a 48 ms capture interval. The first Ultra control has a 53.6 ms
capture interval and a 45.9 ms present interval in its API trace. These trials
remain in the counts and are excluded only from steady-cadence wall-time
summaries. The earlier control helper waited before queuing the next screenshot,
missed image identities, and produced invalid trials; those are retained too.

For normal-phase, steady-cadence trials, host acknowledgment to completed crouch
PNG was 98.7–101.3 ms Vanilla, 49.0–50.7 ms LessLag and 32.5–33.9 ms Ultra.
At the delayed phase it was 89.2–107.2, 40.7–41.1 and 25.6–26.1 ms respectively.
These are measurements on one host monotonic clock, including screenshot
completion; they are not physical display times. Conversion into milliseconds
from a nominal 60 Hz frame count is a separate, derived quantity.

The result supports three fewer captured response frames with LessLag and four
with Ultra at the normal phase. Delayed-input trials preserve the ordering with
up to one frame of variation; exact counts on every input are not guaranteed.
This neither confirms nor relies on the native six-frame reference.

The marker is the first existing plugin callback poll that observed Down. That
poll can occur after the game's own input read, so D is phase-dependent and is
not an independent pipeline-depth measurement. D=0 means the marker and pose
first appear in the same captured image, not zero latency. In the user's proposed
timeline, t0 is host input request/acknowledgment, t1 is this identified plugin
observation, and t2 is the first captured crouch image. Host and guest records
are joined by serial identity; their clocks are never subtracted.

The fresh LessLag trace 1076023080635 matches Down serials 11586, 11699, 11814,
11931, 12052. Ultra phase trace 1135661248312 matches 15767, 15878, 15992.
Vanilla/Ultra normal traces were also cross-checked. Slowed 2% LessLag/Ultra
trials corroborate their counts; the fresh slowed Vanilla run reached its
runtime limit before injection and is invalid. Release-side and other moves
would broaden coverage; console confirmation still requires console tests.

Raw PNGs, host records, API CSVs, configurations, logs, scripts and peer reviews
are retained outside Git under dist/123d47db700f5d19/runtime-evidence.
response-summary.json lists individual measurements; evidence-sha256.json
identifies retained files. Externally terminating the AppImage sometimes produced
SIGBUS exactly at shutdown; later CLI-stop runs exited cleanly. That is separate
from the reported Switch in-match crash.
## Final renderer regression, 2026-10-07

Candidate `b19b7d94ae136e55` repeated the receipt-marker and visible-crouch
experiment in 24 trials: three per profile at each of two input phases. The
normal-phase response counts were Vanilla 6, LessLag 3, LLUltra 2 and LLDoubles 4
captured images, consistently within each group. Delayed-phase counts were 6,
3, 2-3 and 4 respectively. These observations support a reduction of three
captured response frames with LessLag, three to four with LLUltra, and two with
manual LLDoubles in this Training setup.

All critical image windows were consecutive and their receipt serials matched
the guest HID and sampling-counter records. All eight completed API captures
had zero skipped events. Root visually reviewed all 24 pose onsets: one
automated threshold crossing was an idle-animation false positive, corrected
explicitly in the retained record. A 36.1 ms capture interval remains in the
data. These are bounded emulator response measurements, not Switch or physical
display timings, and the manual Training LLDoubles measurement is distinct
from linked doubles compatibility.

See [the methodology and full per-phase results](LATENCY_METHODOLOGY.md).
Raw evidence is retained outside Git:

```text
dist/b19b7d94ae136e55/runtime-evidence/response-and-cutin/
```

## Response trials re-measured on 0.5.2, 2026-10-15

NRO `1525e3e0392c8403` (SHA-256 `1525e3e0392c840306073762cb7e78670745fe0e925de013f660c901371cfdff`,
build `0.5.2-ce0145a2155c-1792070522`) repeated the receipt-marker and visible-crouch
experiment after LLUltra and LLDoubles began presenting at the flush (commit
`ce0145a`): 24 trials, three per profile at each of two input phases, same Eden
build, scene and 640×360 capture as the 0.5.0 re-measurement.

| Profile | Normal phase `R` | Later phase `R` |
| --- | --- | --- |
| Vanilla | 6, 6, 6 | 6, 6, 6 |
| LessLag | 3, 3, 3 | 3, 3, 3 |
| LLUltra | 2, 2, 2 | 2, 3, 3 |
| LLDoubles | 4, 4, 4 | 4, 4, 5 |

The counts match the b19 and 0.5.0 results: present-at-flush on LLUltra and
LLDoubles did not change the captured response frames, and LLUltra still has one
frame fewer than LessLag at the normal phase (2 against 3) while the two are
within one frame at the later phase. All 24 critical windows had consecutive
present identities; seven later-phase trials are not covered by their API trace
export. These are emulator captured-image counts, not the console frame-time
stall that change targets and not input-to-photon latency. Full per-trial table,
exclusions (none), trace caveats and deviations are in
[LATENCY_METHODOLOGY.md](LATENCY_METHODOLOGY.md#re-measurement-on-052-present-at-flush-2026-10-15);
annotated frames are in the README ([Measured input delay](../README.md#measured-input-delay)).
Evidence is retained outside Git in the operator's evidence root under
`latency-052-1015/`.
