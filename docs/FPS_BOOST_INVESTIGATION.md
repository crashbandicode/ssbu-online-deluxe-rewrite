# FPS++ investigation

## Result (2026-10-07)

The previously unidentified FPS-boost field is read by **nnSdk's native-window
buffer submission path** and forwarded as the queue-buffer **swap interval**.
It is not a main-module simulation counter. Eden explicitly interprets large
swap intervals as an extension for composition-speed control.

The original ssbusync backend writes `10000` when enabled and `1` when disabled.
In the delivered Eden source, `10000` requests a composition speed scale of
`10000 / 100 = 100`, while normalizing the actual buffer interval to `1`.
This shortens the requested compositor scheduling period. It does not request
CPU/GPU hardware clock changes, and it does not mean Smash renders 6000 unique
frames per second.

**Status:** implemented as a gated diagnostic and rejected as a normal latency
feature on the tested Eden build. The original fork's interval path accelerates
the game clock; it does not preserve normal match speed. Hardware clocks are
never changed.

Candidate `18dc6c99c5b751f7` was tested in one timed match with LessLag active:
Off, On, then Off again. The On screenshots show 10.38 game seconds elapsed
within a bracketed 3.002–5.299 host seconds. That is **1.959–3.457 times real
time**, with a midpoint estimate of **2.50 times**. Off midpoint estimates were
1.017 and 1.041, consistent with normal speed given the screenshot brackets.
These bounds exclude normal speed in the On condition.

Ordinary builds retain the saved FPS++ toggle but report **Unsupported** and
do not write the boost interval. Only the startup diagnostic marker below
enables the experimental path; do not use it for normal play or an online-safety
claim.

~~~text
sd:/ultimate/ssbu_online_deluxe_rewrite/ALLOW_FPS_BOOST
~~~

The applied suffix and peer announcement require actual applied/read-back state,
so an unsupported saved toggle is not advertised as enabled. The previous idea
that the field itself was a simulation-iteration counter is superseded by the
native-window trace below. Its downstream pacing effect nevertheless changes
observed game speed.

## Evidence chain

1. Upstream `saad-script/ssbu-online-deluxe/src/render/profile.rs` parses a trailing
   `+`, gates boost on emulator detection, and sends
   `EnvironmentFlags::FPS_BOOST_ENABLED` through `ultelier::sync_guest`.
   Its displayed suffix is `++`. The Quickplay fork inherits this feature.
2. The canonical ssbusync 1.4.2 backend's environment-consumer routine writes a
   `u32` through this chain (offsets relative to Smash 13.0.5 main):

   ```text
   outer = *(main + 0x6d43430)
   native_window = *(outer + 0x10)
   field = native_window + 0xf14
   *field = enabled ? 10000 : 1
   ```

3. An isolated vanilla Smash 13.0.5 run under the delivered Eden debugger
   resolved `main = 0x8036d000`, `outer = 0x1180470fa0`,
   `native_window = 0x1184cfc010`, and `field = 0x1184cfcf24`, containing `1`.
   These addresses are evidence from this run, not reusable fixed addresses.
4. A four-byte guest **read watchpoint** caught the reader at `nnSdk + 0x27360`
   (`0x87ef1360` in that run). Its instruction reads `[x21 + 0xf24]`, where
   `x21 = native_window - 0x10`, so both paths refer to the same field.
5. That value is copied into the submission argument at offset `0x2c`. A
   temporary instruction breakpoint immediately before the submission call
   verified the argument value `1`, the buffer slot, and the callee address.
   The callee at `nnSdk + 0x1ec50` serializes the input and invokes Binder
   transaction `7`, the queue-buffer transaction.
6. Eden's packed `QueueBufferInput` places `swap_interval` at `0x2c` and reads
   the flattened input directly. `BufferQueueProducer::QueueBuffer` copies it
   into the queued buffer item. `NormalizeSwapInterval` then handles it:

   ```cpp
   if (swap_interval >= 5) {
       *out_speed_scale = static_cast<float>(swap_interval) / 100.f;
       swap_interval = 1;
   }
   ```

   This excerpt omits the actual null-pointer guard for brevity; normalization
   also runs in paths that do not request the speed scale.

7. `VI::Conductor::GetNextTicks` divides its scheduling period by that scale.
   At a configured 100% speed and interval 1, the requested period changes
   from approximately 16.67 ms to 0.167 ms. This is a scheduling request,
   not a measured compositor rate. The separate `GetFramePeriodNs` work-duration
   hint clamps its result to at least 1 ms; execution overhead, fences, available
   buffers and host scheduling also constrain actual behavior.

The public upstream README claims about three frames of additional reduction
on Vanilla and about half a frame on LLUltra, with possible frame-time variance.
Those numbers remain **upstream claims**, separate from this rewrite's measured
LessLag/LLUltra results.

## Sources and reproducible evidence

- [Upstream profile implementation](https://github.com/saad-script/ssbu-online-deluxe/blob/main/src/render/profile.rs)
- [Upstream feature documentation](https://github.com/saad-script/ssbu-online-deluxe#%EF%B8%8F-render-profile-controls)
- Delivered Eden source: `~/eden-cli-handoff/source`, base commit
  `10bcd2d849843b146a79c61a21df615e1bdcfcde` plus the delivery's recorded changes.

  ```text
  src/core/hle/service/nvnflinger/graphic_buffer_producer.h
  src/core/hle/service/nvnflinger/graphic_buffer_producer.cpp
  src/core/hle/service/nvnflinger/buffer_queue_producer.cpp
  src/core/hle/service/nvnflinger/hardware_composer.cpp
  src/core/hle/service/vi/conductor.cpp
  src/core/debugger/gdbstub.cpp
  ```

- Root evidence directory (local, excluded from Git):

  ```text
  /tmp/delay-audit-01a10c4e-93fe-7db0-9d89-4bcd50bf2340/fps-investigation/
      rsp.py
      readwatch-main/receipt.json
      readwatch-mapped/receipt.json
      readwatch-mapped/reader-code.asm
      queue-input/receipt.json
      queue-input/producer-code.asm
  ```

  A frozen copy of the evidence and the supervisor receipt is retained under
  `dist/fps-investigation-20261007/`, with a SHA-256 manifest. Grok's resumed
  source-audit thread independently verified the native-window import and the
  queue-buffer layout/transaction mapping against the captured data.

- Eden run: `20261007-084809-393e51`, isolated profile B with title mods removed
  for the test; 8 GiB cap, zero swap, 900-second hard deadline. No boost-field
  write was made. Debugger watchpoints/breakpoint were removed and execution
  resumed after capture. Boot initially waited for the debugger's continue
  command, as the source specifies. The game subsequently ran at approximately
  60 FPS. After `game.stop`, the frontend exit exceeded the supervisor's
  responsiveness deadline; the supervisor closed the instance. This was not a
  successful graceful frontend exit or an observed in-match crash.
- Peak recorded memory: 5,319,225,344 bytes; no memory-limit or OOM events.
- Backup journal `~/.local/share/ssbu-mod-backups/fps-readwatch-20261007/journal.json`
  restored the title-mod trees and original configuration with hash checks.
  The user's normal profile and Switch were not changed.

## Requirements for a future usable FPS++ implementation

First separate guest 60 Hz game pacing from accelerated composition. The original
path does not do that. Then keep the renderer profile, 100% emulator speed, scene, input sequence,
host presentation settings and game mods fixed. Compare boost disabled, enabled,
and disabled again. First verify that the requested interval reaches the queue
and that cleanup restores the original value. Then measure:

1. Logical fighter updates and game-clock progress per wall-clock second.
2. Input receipt to first visible response, using the same phase-controlled
   method as [LATENCY_METHODOLOGY.md](LATENCY_METHODOLOGY.md).
3. Composition/submission/presentation timing, duplicate frames and frame-time
   distribution, with debugger stops removed from measurement runs.
4. An asymmetric linked match to check observable synchronization and cleanup.

Vanilla and LLUltra must be measured separately; the expected benefit depends
on the existing buffering policy. A backend implementation needs live-window
lifetime checks and explicit emulator opt-in. Identifying the mechanism does
not by itself validate those implementation details or compatibility.

## Timed-match evidence

The accepted timer trial and raw bracket calculations are retained outside Git:

~~~text
/tmp/delay-audit-01a10c4e-93fe-7db0-9d89-4bcd50bf2340/poll-real-fx/fps-timer2/
    clock-analysis.json
    receipts.json
~~~

The first `fps-timer` attempt is excluded. Fable independently read the accepted
screenshots, checked the timestamp bounds and reviewed the pacing path. A
lockstep opponent can mask local acceleration, so merely completing an
asymmetric network match would not validate this option.
