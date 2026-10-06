# Validation record

Development milestone, 2026-10-05:

- Optimized AArch64 Skyline NRO built.
- Host cargo clippy --all-targets -- -D warnings passed with zero warnings.
- Host cargo build --no-default-features passed.
- Host cargo test: 18 passed after review fixes, including pointer-walk regression coverage.
- Debug ELF has .debug_info, .debug_line and .symtab.
- Packager verified matching executable bytes.
- Known buffer/CSS hook addresses resolved to Rust source file/line.

Switch-target Clippy is unavailable in installed skyline-v3:
error: 'cargo-clippy' is not installed for the custom toolchain 'skyline-v3'.
Host Clippy excludes Switch-only code; the actual Skyline compiler checks those bodies.
No warning was emitted by that build.

## Eden startup investigation, 2026-10-05

- The original build aborted at startup. Runtime updates alone did not resolve it.
- Updated Eden and the user's mounted Switch SD to official Skyline beta (April 12,
  2026) and ARCropolis 4.1.1, including its resources, with verified backups.
  No rewrite NRO was installed on the Switch SD.
- Moved 21 historical .nro.bak files outside Eden's plugins tree after the new
  Skyline console log proved it was loading/executing them. No backup was deleted.
- Controlled baseline: seven existing plugins with the rewrite removed booted.
- Early synchronous phase logging isolated the abort after recorder initialization
  and before module bounds were returned by skyline::info::containing_plugin.
- Replaced that helper with module-local hidden linker symbols. User confirmed
  bundle 8efe12540cfc2b21 reaches the game. Logs contain the correct NRO module ID,
  all eight FX hooks, buffer installation, and background heartbeats.
- That boot exposed a separate menu-installation rejection: another mod had wrapped
  the text setter. Since we only call that setter, the hook-overwrite check was
  inappropriate. Added call-only validation; retained overwrite checks on hook sites.
- Corrected default-config creation: skyline-v3 reports missing native path 0x202
  as ErrorKind::Other. Confirm absence with GetEntryType before creating defaults.
- Candidate c5311b49dc66795c is installed in Eden. Its logs confirm default-config
  creation and successful CSS/FX/buffer hook installation. Menu activation and
  gameplay validation are still in progress.
- At actual character select, CSS entry/exit events are present but the title pane
  is rejected before input handling. Probe cb1eb92cebd70a50 adds sampled fields and
  reason codes without relaxing validation. The previous main-menu attempt did not
  exercise this callback. The user requested eventual menu access from all screens.
- At 17:27:52–17:28:30 UTC, a host-wide OOM caused a desktop logout. The kernel
  table showed eight concurrent Eden processes, several with 3–4 GiB resident, and
  exhausted 8 GiB swap. This is evidence of accumulated instances, not proof of a
  leak in one NRO. Logs/binary were preserved before temporarily quarantining it.
- Added tools/EdenTest.ps1: actual cgroup properties verified (12 GiB memory limit,
  1 GiB swap limit), and a second launch correctly refused the existing Eden PID.
  The same probe is restored for one guarded test; pane diagnosis remains pending.

Host Clippy, no-default-features build, 18 tests, and the actual Skyline build pass
after these source changes. This validates startup only, not full three-mod gameplay
compatibility, emulator performance or console behavior. Raw logs and install/update
receipts remain under target/, outside Git.

## First live matrix

Start with this mod alone, then Training Modpack plus SGPO; test both load orders.
Keep the exact build bundle.

1. Observer-only diagnostic flags: startup identity and log persistence.
2. On + Auto: baseline visuals and CSS/match/results/Training transitions.
3. Menu: each controller family, both rows, saving and restart.
4. Input buffer: Auto and explicit values in Quickplay/arena/local.
5. Each target move: Off suppression and newly spawned On restoration.
6. Shield/parry, smoke, KO, colors, projectiles and LAST_EFFECT consumers.
7. Training save-state/reset/menu and SGPO HUD in the same session.
8. Multi-fighter matches and repeated loads, separately console/emulator.
9. Compare frame times before claiming performance gains.

Diagnose failures from logs and addresses; do not add broad deletion or catch_unwind as a substitute.

## Global window acceptance, 2026-10-05

- User confirmed opening and FX Off in CSS build 24787e3d204edb6e, but its title
  text grew/clipped. Replaced the CSS pane path with the fork's top-left window.
- An intervening panic src\\lib.rs:243:75 matched installed libone_slot_eff.nro's
  fighter-name UTF-8 unwrap. Quarantining only that plugin let the next run reach CSS.
  Original NRO and logs are backed up; a permanent one-slot-effects fix is pending.
- Candidate 2ca7c6991f9a7565 embeds the pinned ImGui backend in one 1,683,456-byte
  NRO. Warning-free Skyline build, zero-warning host Clippy, no-default-features
  build and 22 host tests passed. The bundle includes matching ELF and full source.
- User confirmed the small window appears from the main menu, and FX switches Off.
  Code 26 confirms backend initialization; paired codes 21/22 confirm drawing.
- Fixed native RenameFile's refusal to overwrite config.ini using backup-and-replace
  with startup recovery. Live config.ini now contains effects=off; previous.ini has
  effects=on. A clean restart logged effects=Off, confirming persistence across process restart.
- Training Modpack updater is disabled in Eden; settings were backed up and runtime
  log confirms Disabled. Training Modpack and SGPO remain installed in these runs.

This does not establish all scene transitions, actual FX move behavior, full network
telemetry, Reduced FX, LessLag, console compatibility or performance gains.

User additionally confirmed the global window at character select and the normal
game title banner restored. Match/SGPO display test is pending.

## Accepted FX build and next candidate, 2026-10-05

Installed c1706bbe9589de97 (SHA256
c1706bbe9589de979460de2b3553c9bb188eca30b257355cb2d3aed452fc9225):
user confirmed the small global window in matches, Palutena/Sephiroth FX Off/On,
SGPO, Training Modpack menu open/close, and Training save/load preserving Off
with successful On restoration. Saved runtime evidence stays in that dist bundle.
No Switch or performance acceptance is implied.

The expanded candidate adds qualified actor/move tables, ten ACMD request variants,
both effect-name slots for flips, preserved projectile/hitbox cues, and online-only
INI scope (all for testing). Host tests: 32 passed; Clippy all-targets with
-D warnings and no-default-features build passed. Skyline build and exact final
bundle are recorded in HANDOFF.md. This candidate has no live acceptance yet.

Fable 5.1 xhigh reviewed context/stack ABIs and scope registration. Primary addressed
the review findings: preserve information-bearing cue names, guard install reads,
retain original Lua Int/Hash type, and refresh online state from delay setup even
when input-buffer changes are disabled. Main-menu reset remains unconditional.
Live Quickplay/arena/local-wireless transitions and unknown patched sites remain
validation items, as do the broad move catalog, both load orders and console tests.

Final clean-source bundle: dist/1087e7b01f288ef4, implementation commit
8b8eb34ac4fe224547254744c054f318c2bbcad1. NRO SHA256
1087e7b01f288ef4b29b7c0461109fc62c63ab107c59bc1f9e0a5a891ee02689.
Verified NRO/ELF executable match and retained .debug_info, .debug_line and .symtab.
The bundle includes the scope=all test INI. It has NOT replaced the accepted
c1706bbe Eden installation; no rewrite was deployed to Switch.

## Autonomous Eden continuation, 2026-10-05

The preceding no-install checkpoint was superseded by the user's instruction to
continue autonomously. Expanded bundle1087e7b01f288ef4 was installed and tested:
Sephiroth neutral-B and forward-smash suppression, On restoration, global window,
SGPO, and Training Modpack save/load/menu all worked in the exercised paths.
Off with scope=online retained offline Training visuals and emitted no suppression
events. Actual online transitions and the full move catalog remain untested.
Raw evidence is in that bundle's runtime-evidence directory.

Callback-capture bundlebf4df24b3b8cc176 is now installed in Eden, with On/all/Auto.
Five raw traces passed validation: idle, controlled emulator pause, debug shortcut,
and Training neutral-B with FX On and Off. Zero capture gaps, panic events or ACMD
stack-rejection events were recorded in the final run. An invalid UTF-16 request
was consumed without a repeated-error loop. See PERFORMANCE.md for exact clock
evidence and limits. These tests establish instrumentation operation, not a
performance benefit or LessLag behavior.

Host validation: 37 Rust tests and 4 Python tests passed; all-target host Clippy
with -D warnings and the no-default-features build passed. The actual optimized
Skyline release built without warnings and has matching NRO/ELF/source archives.
Target-only Rust is checked by the Skyline build and live tests, not host Clippy.
Eden was stopped after evidence collection. Original Training Modpack save states
were restored and their hash rechecked; no rewrite was installed on Switch.

## Render experiments and corrected charge coverage

Bundle 79790e610f0f24f8 exercised the opt-in LessLag core in Eden with Training
Modpack and SGPO. Readback confirmed three allocated/two active textures and
present/acquire indices 0/1. The global window, FX Off, and Training menu remained
operational. Core hooks executed; full LessLag sequencing and latency reduction
are not established. Doubles work was stopped at the operator's request.

The operator identified earlier 2.2-second neutral-B holds as too short to confirm
Gigaflare. Longer 4.2-second holds were visually confirmed, warmed, and used for
four On/Off/Off/On pilot trials without screenshot readback during the capture.
All callback/API traces completed with zero skipped records. No On-related dip
was reproduced. See PERFORMANCE.md for numbers, raw evidence paths and limits.

The bounded VI frame-history probe completed, but supplied no usable presented
timestamps in the tested Eden startup window. Temporary TAS and history settings
were restored. Capture files measure CPU observations, not displayed frames.

Final host review found and fixed escaped-newline test fixtures so invalid renderer
values and duplicate keys actually exercise their intended parser branches.
All-target Clippy with -D warnings and no-default-features build passed afterward.
The latest package and any subsequent runtime check are recorded in HANDOFF.md.

The saved original Switch stack was tested separately and restored afterward;
its Eden pre-match stall is documented in SWITCH_CRASH.md. That test did not load
the rewrite, TM or SGPO and did not reproduce the console's crash signature.

## Asymmetric local-wireless smoke test, 2026-10-06

Custom Eden commit10bcd2d849843b146a79c61a21df615e1bdcfcde ran two isolated
13.0.5 instances on one machine. Each had an8GiB memory limit and no swap.
A loaded rewrite d5b4464b508a98244056fa9b09b7a0b5c51f374313afee45177a6a307b70427c,
the corrected gamepad overlay20e9e062..., Training Modpack7.1.1 and ARCropolis4.1.1.
B had its entire title override directory and ultimate directory moved outside
the profile, so it ran without Skyline plugins or modded title assets.

A7-minute Palutena-versus-Sephiroth match on Small Battlefield completed through
sudden death and results. Both clients reported Sephiroth winning and the same
damage-given totals: Palutena6%, Sephiroth30%. Earlier paired screenshots showed
Palutena at30.2% after Shadow Flare on both clients. A returned to character
select; B returned to stage select.

FX changed from On to Off during the match on A only, then back On at results.
Paired screenshots at match clock5:13.86/5:13.85 show Gigaflare's large explosion
on cleanB and suppression on A. Palutena sideB also hit during Off; damage and
stock results remained consistent. Numeric RTT appeared15–27ms. This artifact
displayed the latest sample without the later mean/stability-color work.

No divergence or disconnection was observed during the match. This is a bounded
visual/state smoke test, not a simulation-checksum comparison, Nintendo-online
test, Switch test, or proof about every move. Render mode wasVanilla and input
bufferAuto; it does not validate LessLag/LLUltra multiplayer transitions.

Evidence and exact plugin hashes are outside Git:
~/.local/share/ssbu-mod-backups/desync-pia-20261006/
(result.json, a-plugins.json, run-a, run-b, runtime-final, screenshots).
Memory peaks were7,132,610,560bytes(A) and6,433,144,832bytes(B), with no OOM events.
B exited normally. A's40-minute watchdog expired after the results transition;
the planned stale-RTT screenshot following B's departure was not captured.
The staging journal is restored, including verified original A file hashes.
## ImGui host coexistence, 2026-10-06

NRO defda6fdf99e01e08906f4ae7e8f7df7cfab0d4fa1eb42b478e9ad6499146135
passed three bounded Eden runs: embedded host alone, standalone host loaded
before the rewrite, and standalone loaded after it. Both external runs displayed
Deluxe and Elite Legal Stages together. The reversed-order and embedded runs also
persisted an FX On-to-Off change. NVN present/acquire observations advanced in
all three runs.

The two external runs installed distinct bootstrap replacement addresses,
confirming that the shared hook-name collision was removed. Before the fix, both
plugins resolved the same exported callback; one plugin's implementation never
ran. Every rewrite hook now has a unique prefix, internal function references
bind locally, and bootstrap installation checks the replacement's module.

These checks cover boot, menus, toggling and NVN observation in Eden. They do not
establish native Switch compatibility or actual Elite stage filtering. The
standalone build was ImGui 1.92.7 WIP (19263), SHA-256
61e19e1b593826b1216228c6536cf2603b512154a8fecc548203e245bb86ed3e.
Unknown host ABIs are refused with diagnostics.

All three emulators exited normally and the isolated profile was restored.
Evidence: ~/.local/share/ssbu-mod-backups/imgui-unique-20261006/runtime/result.json,
with screenshots and complete run logs in each case directory.
## LLUltra and asymmetric G&W rendering, 2026-10-06

Two further pairs used the same custom Eden build and 13.0.5 game. A loaded
rewrite 9a0e76de16a7fe7af12da7fd47f6d6e38c3b971d242c0e561b60f0b31d805347,
corrected SGPO, Training Modpack and ARCropolis, with LLUltra/Auto buffer/FX On.
B had no title overrides or plugins. Both used private localhost rooms, 8 GiB
per process, no swap, and finite watchdogs.

The control pair (A 20261006-164547-429778, B 20261006-164547-836af6) had
no G&W plugin on either peer. A seven-minute Mr. Game & Watch versus Sephiroth
match on Small Battlefield completed with Sephiroth winning on both clients.
Settled Sephiroth damage was 16.6% on both; result damage-given totals were
G&W 16%, Sephiroth 15%. Green averaged RTT of 18–20 ms was visible. After B
disconnected, A displayed an uncolored "---". Both exited normally. Return to
character select was not visually confirmed in this control.

The asymmetric pair (A 20261006-172218-3d4587, B 20261006-172218-1b98c6)
added only A's libgnw_render_probe.nro, SHA-256
0883e69785e90b0e6324a803821e82a8cf4df529e6200c25c1e9658069620ade,
from source 4a4f63ad32efdc6e2c5395563bcbc0361296f75c. Demand-readback was
explicitly disabled. Logs confirmed the expected shader fingerprint,
verified=true, and RANGE slot=2 reductions with restored=true. LLUltra counters
advanced with zero bad callers and zero alias mismatches.

The same seven-minute matchup completed with Sephiroth winning on both clients.
Paired screenshots showed 14.6% Sephiroth damage mid-match and matching final
damage-given totals of G&W 29%, Sephiroth 0%. Both returned to stage selection
and then character select. No divergence or crash was observed in this match.
The scripted inputs exercised several moves, but not every requested animation
was captured. Regular-match KO/respawn, all costumes, role reversal, both peers
modded, and simulation checksums remain unverified. This is bounded coexistence
evidence, not an online-safe certification or a G&W performance measurement.

## Controlled latency relay, 2026-10-06

After the asymmetric match, B reconnected through a temporary localhost UDP
relay. It delayed each direction by 50 ms without editing packets. An independent
echo check measured 100.43–100.75 ms round trip with unchanged payloads.

Joining Smash with that delay already present failed before the host's
confirmation appeared. Joining through the same relay with zero delay succeeded.
Adding 50 ms per direction after the join produced about 123 ms in-game RTT,
with yellow and red stability indications captured during CSS/warm-up. No
deliberate jitter or packet loss was injected. Changing delay and host scheduling
can both affect variance; these screenshots are not a controlled test of each
color threshold.

The relay's 1000-second deadline expired during warm-up, producing a room-lost
dialog. No high-latency match completed. This was not a VPN route and does not
establish high-latency desync compatibility. Both Eden processes subsequently
exited normally, with no OOM events; memory peaks were 7,547,342,848 bytes (A)
and 6,820,466,688 bytes (B).

All test plugins/config files were restored from the staging journals, and B's
direct-connect port was restored to 24872. Main user profiles and Switch files
were not changed. Evidence:
~/.local/share/ssbu-mod-backups/desync-rtt-colors-20261006/
(gnw-control and gnw-asymmetric, including localhost-phase, full run logs,
screenshots, renderer counters, relay timing/configuration, and result receipts).

## Reviewed integration artifact, 2026-10-06

NRO `c924d9d1b2ed5b56bb7ec14b847b1b9d1ed627ecc2394be77144723fd0b101e0`
(2,203,648 bytes), ELF
`6cb922b0ec394ebbed1f2968ade1c61a9b82bf308ceca604d4f127d87d7ecbf6`.
Bundle `dist/c924d9d1b2ed5b56` retains matching executable bytes, optimized
debug symbols, full source, native provenance and raw check logs. Host Clippy
with denied warnings, no-default-features build, 89 tests, Skyline Clippy and
Skyline release build passed. All 38 Skyline hook exports have the unique
Deluxe prefix, with no preemptible callback relocations; required NN/NVN/Skyline
imports remain dynamic.

This adds the default-off peer diagnostic observer. Review tightened packet
bounds, coherent queue/map publication and lifecycle invalidation. It refuses
installation without the Pia lifecycle hooks. Missed lifecycle callbacks or
adoption of a new protocol conservatively clear diagnostic identities. It does
not send packets or populate the missing peer metadata UI. Its enabled runtime
path has not been tested yet.

Exact-artifact smoke run `20261006-184006-bd1296` used isolated Eden, LLUltra,
corrected SGPO, Training Modpack and the embedded host, with `peer_observer=false`.
It reached the main menu and displayed Player Info, with no opponent shown.
LLUltra present/preflush counters advanced, three allocated/two active textures,
zero bad callers and zero alias mismatches. It exited normally after 131 seconds,
peak memory 5,969,846,272 bytes, no OOM. This is startup/menu evidence, not a new
match or network test. Evidence and restored-profile hash receipt:
`~/.local/share/ssbu-mod-backups/observer-final-smoke-20261006/`.
