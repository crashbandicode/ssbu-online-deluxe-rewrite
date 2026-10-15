# Diagnosing crashes

![The Debug view of the Player Info window](media/debug-window.png)

The Debug view (L+R+Plus): build id, renderer, online tracker,
`Timing`, `Capture L+R+Up (30 s)`, `CSS capture` and `Match capture`, then the
live resolution, DRS, backoff, memory and frame-time rows. Captured in Eden on
2026-10-14 with 0.4.2, so it still shows the FX rows that 0.5.0 removed; the
whole-screen version and the view during Training are in [Screenshots](SCREENSHOTS.md).

Keep the exact dist/<NRO-hash-prefix>/ bundle for every installed build.
Packaging checks matching NRO/ELF executable bytes, saves both SHA-256 hashes,
the NRO module ID, compiler identity, complete source and native dependency provenance.
The optimized ELF retains DWARF and symbols.

Under sd:/ultimate/ssbu_online_deluxe_rewrite/:

- install.log: earliest startup phases, identity/base addresses, and synchronous before/after hook checkpoints.
  Previous boot: install.previous.log.
- events.log: background snapshots flushed every 500 ms. One previous log is retained;
  each is capped at 4 MiB.
- panic.log: best-effort Rust panic message/location and recent events. Retained until another panic.
- config.ini: saved settings; invalid input is reported and preserved. config.previous.ini
  retains the last good version and is recovered if replacement was interrupted.
  Switch RenameFile cannot overwrite an existing destination: save closes the temp file,
  parks the original in the backup, then promotes temp to the now-absent destination.
- timing-<tick>.csv: opt-in render-callback traces (30 s; a `CAPTURE_MATCH`
  capture runs to the end of the match, ten minutes at most); timing-status.txt
  reports capture/export failures. See [measurement procedure](PERFORMANCE.md).
  Both CSVs stream while the capture runs: header at arm time, rows every
  poll, `# engine/reason/skipped/sample_count` in the footer before `# complete=1`.
- api-<tick>.csv: paired CPU spans around render APIs. These are not GPU or display
  times. Analyze with tools/analyze_render_trace.py.
- renderer-status.txt: actual active mode, texture counts/index masks and cumulative
  hook counters, including `early_presents`, `early_present_skips` and
  `early_present_mismatches` for the present-at-flush path (0.4.0 LessLag;
  0.5.2 LLUltra and LLDoubles).
  Values are independent observations, not an atomic frame snapshot.
- presentation-history.txt: bounded VI probe results, only when the boot flag
  DUMP_PRESENTATION_HISTORY is present. No usable presented timestamps were obtained
  in the tested stock Eden startup window. Remove the flag for ordinary tests.

128 fixed-size events use a nonblocking try-lock. Game hooks do no file I/O; contention
increments a dropped counter. Startup install checkpoints are synchronous. The sink reports
overwritten sequences/drops. File/thread failures are reported to Skyline.
Up to the latest 500 ms may be absent after an abrupt crash; this is not a crash dump.

Startup phases begin before version/configuration queries and the event recorder.
Module addresses come from this NRO's hidden linker symbols (__text_start and
__bss_end), with a mapped-memory check before reading its header. Do not replace
this with skyline::info::containing_plugin: in the installed Skyline versions,
that helper enters a different C++ singleton from the loader's manager and caused
an nn::os::LockMutex abort during our first Eden test. The phase log isolated the
call; replacing it allowed the same mod stack to boot.

Keep backup NROs outside the entire skyline/plugins tree. Skyline recursively
tries every file as an NRO, including .nro.bak files. Its asynchronous console
output can lag the fault: the last printed plugin name is not proof of its cause.

The panic hook chains the previous mod's hook. panic=abort cannot recover through catch_unwind.
Hardware exceptions, GPU faults and hangs do not invoke it. Collect Atmosphere/emulator crash
logs too: module list, PC/LR/backtrace, game version, loaded NROs and reproduction steps.

    /usr/bin/python3 tools/symbolicate.py dist/<hash> 0x<plugin-relative-PC>
    /usr/bin/python3 tools/symbolicate.py dist/<hash> 0x<absolute-PC> --base 0x<plugin-base> --module-id <crash-module-id>

The script verifies bundle hashes and optional module ID, then uses LLVM/binutils addr2line.
Use the faulting plugin's base/ELF, not the game's main-module base.

For controlled bisection, create empty DISABLE_MENU, DISABLE_SCENE,
DISABLE_BUFFER, DISABLE_RENDERER, DISABLE_PIA, DISABLE_NATIVE or DISABLE_BACKOFF
files in the data directory and restart. They skip the corresponding hook
family. DISABLE_SCENE skips the scene tracker (main menu, Quickplay, arena,
local wireless, stage setup): Auto then never becomes ready, banners stay
hidden, the match capture never arms and the exported online context reports
nothing (it was part of the FX family until 0.4.2). DISABLE_BUFFER disables
input-buffer writes. DISABLE_RENDERER skips
experimental rendering text patches regardless of the INI request; actual mode
stays Vanilla. DISABLE_PIA skips RTT and prevents the optional peer observer
from installing. DISABLE_NATIVE skips the native banner hook. DISABLE_BACKOFF
skips the dispatcher idle backoff (code 102 `a` = 4). All seven flags together,
with DUMP_PRESENTATION_HISTORY absent, skip those hook families while retaining
logging.
Remove the disable flags to re-enable their hook families.
DISABLE_FX and the PROBE_CRITICAL_SCREEN marker are ignored since 0.5.0: the
FX hook family moved to the separate ssbu-fx-control plugin, which has its own
data directory and flags.

| Code | Meaning |
|---|---|
| 1 / 2 | Boot / unsupported game version |
| 3 / 4 / 5 | Hook installing / installed / rejected |
| 6 / 7 | Config loaded (a = 0, b = input buffer) / rejected |
| 8 | Setting changed: a = 0, b = input buffer |
| 9 / 10 | Network buffer applied / null pointer skipped |
| 12 / 13 | Sampled CSS entry / exit |
| 14 / 15 / 16 | Panic / heartbeat (a = 0, b = iteration) / compatibility or diagnostic flag |
| 17 | Shared function hook entry instruction before chaining |
| 18 / 19 | Accepted title pane / unsupported CSS pane or mapping |
| 20 | Sampled pane field: 1/2 chain pointers, 3–5 name bytes, 6 text buffer, 7 capacity/length, 8 font sizes |
| 21 / 22 | Global overlay entry / exit, sampled every 300 frames |
| 23 | View change: 0 hidden, 1 settings, 2 performance, 3 debug, 4 plugins |
| 24 | Config saved: generation / 0 |
| 25 | Overlay health: backend ready / bootstrap-call count |
| 26 | Embedded backend initialization complete |
| 30 | Online context changed (`OnlineContext`, `FxOnlineContext` before 0.5.0): online boolean / source scene or latency-setup offset |
| 31 / 32 | Timing capture started / export finished: capture ID / export success (code 32) |
| 33 / 34 / 35 | Renderer owner / caller / flush target diagnostics |
| 36 | Pia RTT sample/cleanup and foreign/drop counters |
| 48 | Render-opts teardown guard fired: a module unloaded between the frame's fighter recording and its post-scene hook, so the relocated model update and fighter worker were skipped and the game's own ordering runs for the next 180 frames. a = unload epoch, b = scene frame |
| 37 / 38 | Opt-in parsed peer observation / bounded escaped name sample |
| 39 / 40 | Peer observer drops / station identity mapping changes |
| 41–45 | Opt-in read-only outgoing packet/header diagnostics; never packet writes |
| 46 / 47 | Render-policy transition / refused transition or invalid post-setter count |
| 64 / 65 | Sampled station-map reachability flags / mixed source ID |
| 98 | Overlay command edge: a = HID button mask; b = previous view, resulting view, Npad id, style flags and command-edge flags (bit 0 Down, bit 1 Plus/Minus). Style 0 means no device |
| 99 | Plugin registered: a = handle (negative `REGISTER_ERR_*` code on error), b = accepted option count |
| 100 | Plugins-page Left/Right edge: a = plugin index (bits 0–15) and option index (16–31); b = requested choice (0–15) and the plugin's `REQUEST_*` answer (16–23) |
| 101 | Plugin INI persisted after a menu edit: a = plugin and option index as in 100; b = 1 success, 0 I/O error |
| 102 | Dispatcher backoff install: a = 1 installed, 2 other owner patched the site, 3 site words differ from 13.0.5, 4 disabled by flag, 5 eligibility gate first passed (b = tick); otherwise b packs the policy (`spin_passes`, `base_ns / 1000`, `max_ns / 1000`) |
| 103 | Dispatcher backoff counters, written only when they change: a = idle passes, b = sleeps (bits 0–47) and last sleep in µs (48–63) |
| 104 | CSS capture startup: a = hook mask (bit 0 dispatcher registration, 1 dependency wait, 2 UI job; 7 = arming); b bit 0 `CAPTURE_CSS` flag, bit 1 `GetThreadPriority` resolved, bit 2 `GetThreadCoreMask` resolved |
| 105 | Early present mismatch (two-image present-at-flush): a = 2 when the game then presented a different index than the early one, b = the game's index |
| 106 | `CAPTURE_MATCH`: a = 0 startup (b = flag present), 1 stage entered (b = scene status), 2 stage left (b = previous status), 3 a running 30 s capture ended for a starting match (b = its id) |
| 107 | Scene connection hold: a = 1 main-menu entry kept a stored Quickplay (b = hook address); a = 2 a Singles/Doubles stage under Quickplay without the online-latency hook was demoted to Offline (b = ticks since the stage load; the matching code 73 AutoContext row carries source `0x0ff11e`) |
| 108 | 0.5.2 dynamic-resolution trigger edge: a = source (1 move table, 2 adaptive) with the catalog move index in bits 8–15 and the fighter entry in bits 16–23 for moves; b = 1 push or 2 pop, the pushed level in bits 8–15, and for the adaptive source the triggering present interval in hundredths of a millisecond in bits 16–47. `renderer-status.txt` carries the settings (`drs_moves`, `drs_adaptive*`) and the counters (`adaptive_active`, `adaptive_observed`, `adaptive_pushes`, `adaptive_pops`, `move_active`, `move_pushes`, `move_pops`); code 82 also records each move edge with source 3 |

Codes 18–20 describe historical CSS builds; they are no longer emitted.
Codes 11, 27–29, 84–87 and 91–97 were the character-FX records. They moved
to the ssbu-fx-control plugin in 0.5.0 and are not reused here. In codes 6,
8, 15 and 24 the field that carried the FX mode is always 0 since 0.5.0.
Codes 12/13 mark character-select entry and exit (exit = one second without a CSS update).
No game-owned pane or font is modified by the global window.

If the menu does not appear, inspect overlay_host_* install phases and code 25.
Zero bootstrap calls suggests the interposer never ran; ready=0 indicates incomplete
initialization. Codes 21/22 prove the draw callback runs; code 23 confirms the hotkey
changed views. Code 8 followed by 24 confirms a change was saved; inspect config.ini too.

Fixed offsets have source/version, text-bound and common-trampoline checks.
The new rendering sites additionally check seven original instruction words
verified from the live 13.0.5 code. Other hook families do not yet have that complete
signature coverage. Live load-order validation
is separate from successful compilation.

The ImGui backend is embedded as a fallback. A compatible standalone imgui-smash
NRO is reused when present; unknown external ABIs are refused. Hook exports use
unique names to prevent dynamic symbol collisions. NVN bootstrap interposition
chains other wrappers and records the entry word. See
[dependency details](IMGUI_DEPENDENCY.md); future LessLag/acquire changes need new tests.

## Optional peer observer

`peer_observer=false` is the default. Set it to true before process startup
only for a controlled diagnostic run. Both parsed-message and station-index
instruction guards must match, and the Pia lifecycle hooks must be installed.
The optional pre-encryption send probe has its own instruction checks. This
does not enable peer metadata in the UI and never writes network packets.

The observer validates body bounds against the accepted packet length before
copying. A nonblocking gate protects its bounded queue and station generations;
file output occurs on the diagnostic thread after releasing the gate. Missed
observations are counted, and missed lifecycle notifications conservatively
clear mappings. Names are escaped, truncated diagnostic samples. Mixed IDs are
correlation values, not an anonymization guarantee. Observer events remain
subject to the ordinary log size and crash-flush limits above.

The map event queue can drop a one-shot update while retaining the in-memory
mapping. Absence of code 40 alone does not prove the station hook failed. The
64/65 reachability sample is diagnostic only and is not a coherent authorization
record for packet writes.

## Live render profiles

NetProfile changes save immediately. A `*` and `Swap` indicate that the request
has not fully committed. Inspect `policy_committed`, `policy_effective`,
`policy_requested`, `policy_buffer`, `policy_index`, `policy_known_active`,
`active_textures`, `policy_failed` and `policy_force_wait` in renderer-status.txt.
The legacy `mode` and CSV `engine` fields are the startup mode and can differ
from the active profile after a menu change.

Buffer changes force a wait, call the native setter at a later validated
boundary, drain original flushes and confirm a fresh count. A missing drain
marker after the setter keeps waiting with the safe modulus and emits one
code-46 kind-12 warning. It does not guess a rollback. A bad or persistently
wrong post-drain count is fatal and produces the normal panic diagnostics.
Before the setter, an unavailable boundary or dependency can refuse the change
without mutating the window.

CAPTURE_TIMING also records phase markers 12–17. Their result field is a hashed
thread label. Kind 17 index is outstanding tracked-window acquisitions after
present; zero is meaningful. Kind 12 is the selected value before the game's
modulo-three instruction. These are CPU-side observations, not displayed frames
or input-to-photon measurements.

With the `CAPTURE_CSS` boot flag (0.3.1) the trace also carries: kind 60
(index = dispatcher-backoff sleeps since the previous tracked present, result =
their summed actual microseconds), 61 (index = longest single sleep in µs,
result = idle streaks started), 62 (SDK `ThreadType` pointer halves, once per
thread per capture), 63 (index = `nn::os::GetThreadPriority`, result = ideal
core in the low byte and the affinity mask above it; -1 when the SDK symbol did
not resolve), 64 (UI aggregate child registered: index = worker | priority << 8,
result = thread label), 66 (span of a `main+0x354ccd0` dependency wait on any
thread; index = awaited counter's offset from the text base, `0x5333160` being
the UI barrier; result = thread label) and 77 (span of the UI job
`main+0x375e260` on its worker; result = thread label). A kind 66 row whose
label equals the kind 77 label is a wait nested inside the UI job. The label is
the same pure hash for every kind; `tools/analyze_render_trace.py` recomputes it
from kind 62 and reports a `css_capture` section. Event log code 104 gives the
hook mask (7 = all three) and which SDK symbols resolved; codes 12/13 mark
character-select entry and exit (exit = one second without a CSS update).

With the `CAPTURE_MATCH` boot flag the same capture pair is armed on every
stage load (the stage-setup hook classifies Singles, Doubles or Training;
label `match_singles`, `match_doubles` or `match_training`) and ended when
the game leaves the stage: the scene status returns to Inactive (results,
main menu) or the character-select update runs (the Training CSS keeps the
Training status, so this is what ends a Training capture; code 106 `a=2`
`b=0xC55`). The footer says `# reason=scene_end`; the ten-minute ceiling
gives `elapsed`. A stage that loads while a match capture is still running
ends it the same way and arms the next. Both CSVs stream from the
diagnostics thread, which polls every 250 ms while a capture runs; the trace
chunks pack `begin` relative to a per-chunk base, so a long capture keeps the
20-byte row. A 30 s capture still running when a match starts is ended
(event log code 106 `a=3`) and the match capture arms once its export is
done. `CAPTURE_MATCH` also installs the three `CAPTURE_CSS` hooks (rows
62–66 and 77) without arming on character-select entry; put both flags on
the card for both captures. Code 106: `a=0` startup (`b` = flag present),
`a=1` stage entered (`b` = status), `a=2` stage left (`b` = previous status),
`a=3` a running 30 s capture ended for a starting match (`b` = its id).
The Debug view's `Match capture` row shows the counts. `tools/analyze_timing.py`
accepts `scene_end`. Nothing in the hot path changes: no hook does I/O, and
the rows are the ones the CSS capture already records.

## Hang trace (0.3.2)

`hang-trace.log` in the data directory is always on and costs one atomic
increment per tracked hook call plus one short line every 5 s from the
diagnostics thread. It exists to answer, after a loading-screen freeze, whether
one of this plugin's hooks was mid-call and which liveness signals stopped.
The previous boot is kept as `hang-trace.previous.log`; the file rotates at 2 MiB.

- Header: build, `boot_tick` and `tick_hz`, so another plugin's log can be
  aligned by system tick. `t=` on every line is seconds since `boot_tick`.
- `[Heartbeat]` every 5 s: packed scene (`connection/status`), scene-ready
  bit, character-select visit open, API trace active, then (0.3.6) the
  kernel's memory accounting for the game process in bytes: `mem_used`,
  `mem_total`, `mem_free` and `heap_region` from `svcGetInfo` (used covers
  code, heap, stacks, every Skyline plugin and GPU-backed buffers; the same
  call works on Eden, so console and emulator headroom compare directly; the
  Debug view's Performance table shows the same numbers as a `Memory` row),
  then calls since boot for
  `present` (tracked window), `backoff_pass` (idle hook entries),
  `backoff_sleep`, the three `CAPTURE_CSS` hooks (`css_init`, `css_wait`,
  `css_ui_job`), `fighter_load`/`fighter_unload` (`common` module chain),
  `fighter_main` (per-fighter tick) and `cut_in`. A hook that entered and has
  not returned shows `[ACTIVE xN age_ms=…]`; a growing age across heartbeats
  means the game is blocked inside or under that hook. Count-only signals
  never show ACTIVE.
- `[HangTrace] first <hook>`: first call of each hook since the current scene
  began, with its tick. Scene changes print `[HangTrace] scene a -> b` and
  reset the first-firing marks.

Zero-code bisection for a freeze: delete `CAPTURE_CSS` first (removes the three
CSS hooks), then add `DISABLE_BACKOFF`, then the other `DISABLE_*` flags. The
heartbeat keeps running with every family disabled.

## Eden test lifecycle on this Ubuntu host

Use the test launcher instead of repeatedly opening the desktop shortcut:

    /usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -File ./tools/EdenTest.ps1 -Action Start
    /usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -File ./tools/EdenTest.ps1 -Action Status
    /usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -File ./tools/EdenTest.ps1 -Action Stop

Run each command separately. Start refuses an existing Eden process and creates
ssbu-deluxe-eden-test.service with MemoryHigh=8G, MemoryMax=12G and MemorySwapMax=1G.
Stop terminates the whole test cgroup, including a process left behind after its
window closes. Verify no Eden process remains before the next Start. The helper's
-EdenPath parameter overrides its default local AppImage path. Pass -GamePath with
the local game file to launch Smash directly using Eden's -g argument.
For unattended stock-Eden trials, specify -MaxRuntimeSeconds 900 (or another finite
bound). The default zero is for operator-controlled sessions. The incoming custom
Eden build will supply its own CLI control instructions; read its handoff before
adapting the launcher, and retain the same memory and single-instance safeguards.

These limits contain cgroup-accounted memory for this test, not the entire desktop
or all GPU-driver memory. They are test safeguards, not a memory-leak fix.
On 2026-10-05, the kernel OOM table showed eight simultaneous Eden processes and
zero free swap; repeated launches had accumulated old instances. Preserve logs
before restarting, since multiple instances can also write the same log files.
