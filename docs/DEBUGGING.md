# Diagnosing crashes

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
- timing-<tick>.csv: opt-in ten-second render-callback traces; timing-status.txt
  reports capture/export failures. See [measurement procedure](PERFORMANCE.md).
- api-<tick>.csv: paired CPU spans around render APIs. These are not GPU or display
  times. Analyze with tools/analyze_render_trace.py.
- renderer-status.txt: actual active mode, texture counts/index masks and cumulative
  hook counters. Values are independent observations, not an atomic frame snapshot.
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

For controlled bisection, create empty DISABLE_MENU, DISABLE_FX, DISABLE_BUFFER or
DISABLE_RENDERER files
in the data directory and restart. They skip the corresponding hook family.
DISABLE_BUFFER disables input-buffer writes; the shared setup observer can remain
installed for FX scope. DISABLE_FX skips FX requests and scene tracking.
DISABLE_RENDERER skips experimental rendering text patches regardless of the INI
request; actual mode stays Vanilla. All four flags together, with
DUMP_PRESENTATION_HISTORY absent, leave diagnostic logging without those hooks.
Remove the disable flags to re-enable their hook families.
These differ from FX On, which keeps passthrough hooks installed.

| Code | Meaning |
|---|---|
| 1 / 2 | Boot / unsupported game version |
| 3 / 4 / 5 | Hook installing / installed / rejected |
| 6 / 7 | Config loaded / rejected |
| 8 | Setting changed |
| 9 / 10 | Network buffer applied / null pointer skipped |
| 11 | FX suppression count: first then every 128 |
| 12 / 13 | Sampled CSS entry / exit |
| 14 / 15 / 16 | Panic / heartbeat / compatibility or diagnostic flag |
| 17 | Shared function hook entry instruction before chaining |
| 18 / 19 | Accepted title pane / unsupported CSS pane or mapping |
| 20 | Sampled pane field: 1/2 chain pointers, 3–5 name bytes, 6 text buffer, 7 capacity/length, 8 font sizes |
| 21 / 22 | Global overlay entry / exit, sampled every 300 frames |
| 23 | View change: 0 hidden, 1 settings, 2 performance, 3 debug |
| 24 | Config saved: generation / effective FX mode |
| 25 | Overlay health: backend ready / bootstrap-call count |
| 26 | Embedded backend initialization complete |
| 27 | ACMD request seen while Off: hook ID / hash; first and every 256 per hook |
| 28 | ACMD suppression: original hash / (hook ID shifted 32 bits, suppression count low bits); first 32 and every 128 |
| 29 | ACMD stack refused: hook ID / argument type; first 8 only |
| 30 | FX online context changed: online boolean / source scene or latency-setup offset |
| 31 / 32 | Timing capture started / export finished: capture ID / export success (code 32) |

Codes 12/13 and 18–20 describe historical CSS builds; they are no longer emitted.
No game-owned pane or font is modified by the global window.

If the menu does not appear, inspect overlay_host_* install phases and code 25.
Zero bootstrap calls suggests the interposer never ran; ready=0 indicates incomplete
initialization. Codes 21/22 prove the draw callback runs; code 23 confirms the hotkey
changed views. Code 8 followed by 24 confirms a change was saved; inspect config.ini too.

Fixed offsets have source/version, text-bound and common-trampoline checks.
The new rendering sites additionally check seven original instruction words
verified from the live 13.0.5 code. Other hook families do not yet have that complete
signature coverage. Shared EffectModule function hooks deliberately chain. Live load-order validation
is separate from successful compilation.

The ImGui backend is embedded; do not also install a standalone imgui-smash NRO.
The rewrite detects its exports and declines to install a second host. NVN bootstrap
interposition chains other wrappers and records the entry word. See
[dependency details](IMGUI_DEPENDENCY.md); future LessLag/acquire changes need new tests.

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
