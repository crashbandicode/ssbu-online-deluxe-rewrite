# Measuring performance and LessLag

The rewrite currently uses Vanilla rendering. LessLag and Reduced FX remain
unimplemented. A constant 60 FPS does not establish low input latency, and a
callback interval is not a displayed-frame interval.

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
| Callback / present API invocation timing | Existing callback; future verified NVN wrapper | Render-thread scheduling and time inside a call |
| Presentation / vblank history | Validated `nn::vi::ListFrameHistory` data | Frames displayed late or repeated; scheduling relative to presentation |
| Input to visible response | High-speed camera with controller and display in shot, or calibrated hardware | End-to-end input latency |

Fable 5.1/xhigh reviewed the design. We rejected its initial interpretation of
`nvnQueuePresentTexture` call timing as actual presentation and its concurrent
plain-array ring design. The accepted first implementation is narrower: a finite,
locked callback capture. No present wrapper or `nn::vi` hook is installed yet.

The archived source declares `ListFrameHistory`, `GetLatestFrameNumber` and a
40-byte `FrameInfo`. The original binary imports those functions. The archive
captures its layer through a game offset; those offsets must not be copied into
13.0.5 without validation. An older SDK symbol map also lists `GetNativeWindow`.
A possible next step is a symbol hook on that function to obtain the layer, then
a bounded background history reader. Whether the game calls that symbol, the
record ABI, timestamp units and Eden support must be checked before trusting it.
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
  reported effective suppression and the diagnostic log recorded Gigaflare
  screen/burst/burst2 suppression. Training Modpack and SGPO remained loaded.
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
