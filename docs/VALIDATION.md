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

## Live profiles and both asymmetric G&W modes, 2026-10-07

Tested NRO `ed5d12f84772149fcf86ea43ffa18efd657f6d27edefde3034eba8e647312c6e`,
ELF `5da2c5cd5ce801959faba5f9c04dbc9dabcf8dbc65e5c49e15eab1d6db32530d`.
The same candidate ran both tests below. Host Clippy with denied warnings,
no-default-features build, 110 host tests, Skyline Clippy and release build passed.

A `20261006-210611-b11161` had the rewrite, G&W fix `0883e69785e90b0e…`,
corrected SGPO `20e9e062c9ace623…`, Training Modpack and ARCropolis. B
`20261006-210611-b70a45` had no title overrides/plugins. Custom Eden was
`10bcd2d849843b146a79c61a21df615e1bdcfcde`, Smash 13.0.5, private localhost
room, Auto input buffer, FX On/all. Neither peer used VPN. A's diagnostic peer
observer was enabled and only observed packets. Demand-readback was disabled.

Both were three-stock, seven-minute Mr. Game & Watch versus Sephiroth matches.
The scripts exercised attacks, projectiles, jumps and KO/respawn; B then walked
off to finish the matches. They are not seven minutes of uninterrupted combat.

| Mode on A | Stage | Matching live damage, A/B | Matching result damage given, A/B | Outcome |
|---|---|---|---|---|
| LessLag | Kongo Jungle 64, Battlefield form | 5.6% / 26.4% | 26% / 5% | G&W wins on both; both return to CSS |
| LLUltra | Battlefield | 5.6% / 20.1% | 20% / 5% | G&W wins on both; both return to CSS |

G&W shader fingerprint `3869d88ad69eee6a` was verified, with RANGE slot 2
reductions and restoration logged. Rewrite Ultra guards recorded no bad callers
or alias mismatches. No desync was observed in either match. This establishes
bounded one-sided coexistence, not simulation-checksum equality, every move or
costume, Switch behavior, VPN/high-latency compatibility, or a performance gain.

The same session also switched Vanilla/LessLag/LLUltra in the main menu, at CSS
(nine successive changes), and during a third linked match (two complete cycles).
All selections reached Active. The match trace `2711481731398` captured 592
presents using indices 0/1/2, all with zero outstanding acquisitions after present,
and no skipped trace events. The CSS trace `2698991455799` recorded another 568
presents with zero outstanding acquisitions, but did not span all CSS changes.
Across the session, ten buffer changes committed; no code-47 refusal, code-14
panic or stalled-drain warning occurred. The third match was stopped after the
transition checks; it is not another complete-match acceptance.

The legacy trace `engine` and status `mode` fields describe the startup mode.
For live changes use `policy_committed`, `policy_effective`, the fresh active
texture count and transition events. Raw index-selection markers are before
the game's modulo-three instruction and can exceed two in Vanilla.

Both frontends exited normally, code 0, with no OOM events. Peak memory was
7,083,679,744 bytes for A and 6,448,177,152 for B, under 8/7 GiB limits and a
45-minute deadline. All staged files were restored with hash verification.
Switch and main user Eden files were untouched. Evidence, screenshots, mode
snapshots, trace summaries and restoration journal:
`~/.local/share/ssbu-mod-backups/peer-menu-gnw-e2-final-20261007/`.

INI persistence was separately exercised on preceding E1 candidate `bcb8…`:
LLUltra and the Stealth preference survived a full restart. Current host tests
cover the same save/recovery path. Stealth still displays Unavailable because
the transmit implementation is incomplete; saved preference is not proof of
network behavior.

## Stealth identity blocker and separate emulator crash

The read-only observer populated peer slot 1, but its one-shot map event was
dropped from the four-entry diagnostic queue. Subsequent sampled probes proved
the slot remained populated. Its received source ID still differs from the
transmit destination ID. Grok excluded a byte-order explanation; Fable's second
opinion found no proven conversion and recommends station-field and explicit
header-presence probes, or the fork manager's connection-ID method. No guessed
mapping or packet writes were added. Full Stealth and peer metadata remain open.

A prior diagnostic run `20261006-201956-a94e26` crashed in Eden's host GPU code
before fighter selection. Symbolization located `Tegra::DmaPusher::CallMethod`
accessing an unbound subchannel. The cause of that missing engine binding is
unresolved; it is not established as a G&W or rewrite fault. The current two
accepted matches did not reproduce it. No emulator workaround was shipped.

## Combined profiles and resolution candidate — 2026-10-07

NRO `6c6a64afca4073c91128d938141e143c8519ead41e530a26ca56f1c46bc02409`,
optimized ELF `74e484062f276da577b6d3900eba5bf05fa34a3233d84a47e1c16676b45b7c8a`.
169 host tests, host/Skyline Clippy with warnings denied, no-default-features
build and optimized Skyline build passed. Raw checks and source are in its
ignored debug bundle.

The earlier Stealth identity blocker described above is historical. The current
implementation uses the fork's validated connection-ID path; it does not guess
a conversion from receive headers. Prior candidate `410ac1f7` exchanged names,
interfaces and active profile settings with the original fork in a local-wireless
lobby. Its pending-On display was observed. That original-fork stack aborted
while loading a fighter, so this was not a complete match or final-candidate
fresh-Stealth-On acceptance.

### Fighter callback and Training coexistence

Count-only run `20261007-013838-0fd514` used Vanilla/emulator, standalone ImGui,
Elite, TM and corrected SGPO. Two five-second idle Training samples, separated
by returning to CSS and loading another Training session, counted 307 Main ticks
per fighter over 302 presents and 306 over 301. Both passed the predeclared
0.98–1.02 approximate-frequency band, with no rejected reads or slow ticks.
This is not exact per-frame uniqueness or module-unload proof. Count mode
installed no cut-in hook or resolution reports. The TM menu opened/closed and
SGPO remained visible. A preceding navigation attempt reached no accepted sample
and is retained separately.

### Manual profiles, Auto and Gigaflare

Run `20261007-014825-27814e` deliberately used `platform=console` in Eden to
exercise the console resolution table. Sephiroth charged neutral B for 4.5 seconds.

| Selected profile | Applied base | Gigaflare | Recovery |
| --- | --- | --- | --- |
| LLUltra | 576p / DRS on | 480p, crop 854×480 | 576p, crop 1024×576 |
| LessLag | 1080p / DRS on | 720p, crop 1280×720 | 1080p, crop 1920×1080 |
| Vanilla | 1080p / DRS off | No resolution report | Remained 1080p |
| LLDoubles | 1080p / DRS on | 720p | 1080p |

All four manual modes reached Active. Auto in offline Training selected Vanilla
while retaining the saved manual LLDoubles choice. The Doubles transition is
proven by events 749–756; periodic status publication missed that short interval.
Reading a status file every 50 ms does not make its roughly five-second
publication interval a 50 ms timing measurement.

FX Off suppressed the named Gigaflare request (`0x1210319015`, event 1355), and
the resolution report recovered afterward. The actual cut-in trigger was not
exercised. Nonfighter-category rejects during weapon articles are expected
filtering, not evidence of failed fighter reads.

Both Deluxe and Elite menus worked through the standalone ImGui host. Elite's
enabled setting persisted; offline Training continued. Both menus consume D-pad
edits when simultaneously open, so edit one at a time. This does not validate
Elite's NSO-only stage filter.

A manual console-LLUltra boot showed an undersized title image after the rewrite
had applied its crop. Later CSS/match images filled the window. Fable reviewed
the timeline; a later external crop reset is suspected, but its caller remains
unproven. The Auto/Vanilla menu path is unaffected. An observation-only diagnostic
is being prepared; no crop-behavior fix is claimed.

Both accepted single-console runs exited normally without OOM and their staged
files were restored. Raw screenshots, events, samples, acceptance records and
run journals are preserved under:

```text
dist/6c6a64afca4073c9/runtime-evidence/count-only/
dist/6c6a64afca4073c9/runtime-evidence/resolution-console/
```

### Four-player attempt, no match acceptance

A `20261007-020912-95094a` used this candidate, GNW fix `0883e697…`, SGPO
`20e9e062…`, TM, ARCropolis, Elite and standalone ImGui; B
`20261007-020912-f7b003` was a clean peer. Both joined a private local room.
Two local players joined A, but B's second local player did not join. The attempt
ended at Team Battle character select with three players. No match started;
this is not a doubles or desync result. Both exited normally without OOM,
peak 6,550,102,016 / 6,587,686,912 bytes, under 8/7 GiB caps. The staging journal
was restored exactly. No VPN or artificial network delay was used.
## Final b19 crop, response and cut-in checks — 2026-10-07

Artifact `b19b7d94ae136e55d2fe59136dd9ff2c40ce9436393a59984660ed3ab8c52cd7`
has matching optimized ELF `3f5e8407be2ef8f38037df645c69f32759d617ec6f7517fac4432de0fc18e397`.
All 175 host tests, host/Skyline Clippy with warnings denied, the no-default-features
build and optimized Skyline build passed. Source/build inputs were compared with
the packaged source archive before runtime testing.

An observation build proved the game reset a previously applied 1024×576 crop
to 1920×1080. The final fix forwards the original call unchanged and rearms only
an exact fullscreen reset on the tracked window with an accepted smaller crop.
The next tracked present reapplies it. Run `20261007-025442-132e12` showed events
177/178/179 in that order and a correctly filled title image, then full Gigaflare
576→480→576. It ended at its 600-second runtime limit, not a spontaneous crash;
no OOM kill, peak 8 GiB. The profile was restored.

Run `20261007-031616-b75dd6` collected 24 response trials on the same final NRO.
See [LATENCY_METHODOLOGY.md](LATENCY_METHODOLOGY.md#final-candidate-24-trials-2026-10-07)
for exact counts, image/trace validation, the retained idle-animation threshold
false positive and capture-copy race. The earlier preflight was stopped normally
to configure 360-pixel screenshots before emulation; it supplied no trials.

After the response trials, a second Training session used Luigi against Roy.
Close-range Up-B produced the visible critical-hit cut-in. Events 1193–1200 show
source-1 report start, applied 720/crop1280×720, report end and restored
1080/crop1920×1080 while manual LLDoubles was active. Screenshots preserve both
the cut-in and subsequent play. This verifies the actual trigger and recovery,
not a Switch performance improvement or every cut-in variation.

The solo run exited normally after 827.6 seconds, peak 8.55 GB within its 9 GiB
limit, with zero memory-limit reclaim events and zero OOM events. Its isolated
profile was restored. Evidence lives outside Git under:

```text
dist/b19b7d94ae136e55/runtime-evidence/crop-reset/
dist/b19b7d94ae136e55/runtime-evidence/response-and-cutin/
```

## Linked doubles and peer regression — 2026-10-07

The b19 pair `20261007-033126-622b58` / `20261007-033126-30b963`
completed a four-fighter team match on Moray Towers (Battlefield style).
Three local controllers joined A and one joined B. A carried the rewrite,
G&W fix `0883e697…`, corrected SGPO `20e9e062…`, TM, ARCropolis, Elite and
standalone ImGui; B had no Skyline/title mods. G&W and Sephiroth faced Wolf
and Byleth. Auto committed LLDoubles with three textures and DRS enabled.
Both sides agreed on observed hits, stocks and winning team, then returned
through results to character select. Full Gigaflare on B triggered A's
1080→720→1080 scaler. Both exited normally with no OOM.

The API trace `3123935492968` was captured during **Vanilla warmup**, not the
actual doubles match. Its 491 presents, three indices, TwoBehind relation and
zero outstanding acquisitions therefore do not prove the doubles pipeline.
Doubles acceptance uses the actual match's committed status, events and images.

The b19 pair `20261007-040044-b3b852` / `20261007-040044-1a740a` used
the rewrite on both peers, external ImGui on A and embedded ImGui on B. Names,
colored RTT, announced interface/profile, Streamer menu/chord and saved Stealth
On pending reconnect passed. G&W with the fix on A/LLUltra versus Sephiroth on
B/LessLag completed through matching winner/results and stage select. A reached
its 1800-second runtime limit while loading the next CSS; no final CSS acceptance
is claimed for this run. B exited normally; neither hit OOM. An earlier incidental
Ice Climbers match in the same session is not a G&W test.

```text
dist/b19b7d94ae136e55/runtime-evidence/four-fighter/
dist/b19b7d94ae136e55/runtime-evidence/two-rewrite-peer/
```

## Final display fix and Stealth/persistence acceptance — 2026-10-07

| Artifact | SHA-256 |
| --- | --- |
| NRO | `d9acd03611e1d1d644e5c1d56f0c80748e082cd46cd02f75ca3e62c3bffd838b` |
| Optimized ELF | `93da4bf258eb6a22f796ad9929fae4dcfef06cd45df7662e117505bf2ce6a25e` |
| Source archive | `6d7035650b1e285a645bdff6f1e9a88ed0cc690ba3d42eb0d197128b0433db2a` |

Build `0.1.0-03cad922da77-dirty-1791362447` passed 176 host tests, host
Clippy with warnings denied, the no-default-features build, the Skyline check
and optimized release packaging. The installed cargo-skyline 3.5.0 wrapper
requires `cargo skyline clippy -- -- -D warnings`; the single-separator attempt
rejected `-D` before checking. Raw final host/check logs are retained in the bundle.
Root compared all 55 packaged Rust/build input files with disk and reviewed
exactly four changed files versus b19: remote-buffer display/snapshot handling,
compact ASCII Auto labels and tests. No renderer/input/packet-writing change.
The 24 response trials remain measurements of b19, not this exact binary.

Two private localhost stations ran the final NRO. A had the complete mod stack
and compatible standalone ImGui; B had only the rewrite and its embedded host.
The first pair (`044519-4b92e6` / `044519-dff491`) loaded the exact INI files
saved through b19's menu, verified by SHA equality. A restored Ultra, buffer 2,
Stealth On and Streamer On; B restored LessLag, Auto buffer, Stealth On and
Streamer Off. Streamer hid the remote fields and its chord restored them.

Both learned/confirmed one native station. With fresh Stealth On, each logged
one suppressed beacon, **zero beacon writes and zero custom payload writes**.
Native name/RTT remained visible while unannounced interface/settings stayed
unknown. Saving Off showed `Off (reconnect)` and did not enable writes within
that connection. Both exited normally; complete logs/configs were archived
before restart, with no external INI edits between sessions.

The second pair (`045148-2c9fb4` / `045148-6e2cc5`) loaded saved Off and
exchanged custom metadata. At the first checkpoint A/B had one beacon each and
85/83 payload writes, with zero Stealth suppression. B displayed A's buffer 2;
A displayed B's Auto. Names, Ethernet announcements and green 17–20 ms RTT
were visible. The Ethernet value is the emulator's virtual nifm result.
No ambiguous-identity skips occurred. A few bounded lock-contention skips were
recorded; these are safe rejected attempts, not a claim of zero contention.

Elite's menu was opened with Deluxe and enabled before a regular linked match.
G&W on A used LessLag and the one-sided render fix; Sephiroth on B used Auto,
which selected LLUltra at match start. The G&W log confirms full shader/constants
verification and narrowed ranges restored after use. On Saffron City, both peers
showed Sephiroth at 12.6% after G&W's attacks and agreed on subsequent stocks,
G&W's victory, results and return to CSS. Cleanup committed Vanilla on both,
with the saved manual choice retained. The corrected controller overlay remained
visible on A. This was ordinary Smash; TM's save/load behavior was tested earlier.

All four d9 frontends exited normally with zero memory-limit and OOM events.
Fresh-On peaks were 6.08/5.24 GB; final match peaks were 6.21/5.92 GB under
8/7 GiB caps. All staging files were restored with original hashes.

```text
dist/d9acd03611e1d1d6/checks/
dist/d9acd03611e1d1d6/runtime-evidence/peer-and-gnw/
```

The bundle includes raw runs, checkpoints, screenshots, configuration receipts,
acceptance notes and a SHA manifest. Runtime folders can contain older captures;
attribute them by build ID and run window. The original packaging manifest's
`runtime_tested=false` is preserved as build-time metadata; the separate runtime
acceptance record supplies subsequent evidence.

These tests are bounded observations, not simulation checksums or universal
online-safety proof. No VPN/high-RTT, NSO filtering, four-network-station or new
Switch-rendering acceptance follows. Fable's final follow-up review was blocked
by the provider's model-access rejection; this final display review was performed
by root after Grok's implementation, without claiming a new Fable sign-off.


## Fork native display restoration, 2026-10-07

The fork's CSS title and arena room-ID code paths now accompany the existing
global Player Info window. Provenance and precise differences are in
[NATIVE_DISPLAY.md](NATIVE_DISPLAY.md). The window remains the editor.

Final artifact:

```text
NRO: 4a927dc20904722dc903afcb5c8548d80a5afffb3aeedd68958c76c3181c0518
ELF: 54a87dd43f7aaa3eb743dc73b20a6bc3cd61550549aae6c39c7c8acdc5dbe103
Bundle: dist/4a927dc20904722d/
```

192 host tests, host Clippy with warnings denied, no-default-features build,
Skyline Clippy with warnings denied and optimized package build passed. All 57
packaged Rust/build inputs match disk. The NRO and ELF hashes and matching
executable bytes were verified. One intermediate Skyline Clippy failure
(too_many_arguments) was corrected before packaging; its raw log is retained.

| Candidate / run | Observed result |
| --- | --- |
| `84d48448ac91e65d`, `20261007-101737-33eb02` | CSS window plus header; LessLag/buffer 0 edits; status persists with window hidden; Streamer restores original font/background; CSS re-entry; Olimar/Fox match through results and return to CSS; normal exit; 6,148,984,832-byte peak, no OOM |
| `4a927dc20904722d`, `20261007-103320-77c12b` + `20261007-103321-17d19e` | Linked native names/profile/green RTT in both directions; B embedded window visible; A standalone Deluxe and Elite windows invisible despite callbacks/input edits; no match started; below full speed before CSS, no OOM |
| `4a927dc20904722d`, `20261007-105849-510a19` | Solo standalone Deluxe and Elite windows plus native title, approximately 60 FPS; normal exit; 6,733,324,288-byte peak, no OOM |
| `4a927dc20904722d`, `20261007-110542-4550ab` | Same solo coexistence with G&W loaded; configured 600-second runtime limit stopped the frontend; 7,787,769,856-byte peak, no OOM |

The final build differs from 84 by the arena font/vertical-position correction
and deriving the diagnostic site inside the painter. No rendering-profile
behavior changed. Final solo screenshots show Elite Off; they prove menu
coexistence, not enabled NSO filtering. The earlier d9 enabled-Elite gameplay
result remains separately attributed to d9.

The paired external-window anomaly is unresolved. A root source/hook audit
found all three expected bootstrap registrations and external ImGui selection;
it did not establish which component caused the failure. Solo runs with and
without G&W both succeeded. Do not infer either a G&W fault or a fix from these
results. A future identical-pair comparison with `DISABLE_NATIVE` can isolate
the new native display without changing binary versions.

In the paired shutdown, A's frontend receipt reports exit 0 although its
supervisor tool returned 143. B remained in a game-stop confirmation flow;
after a 15-second helper timeout, the owned B service was explicitly stopped.
The G&W solo run ended at its configured runtime deadline before the manual
stop helper, which received ConnectionRefusedError. These stops are retained
as operational failures/limits, not reported as clean exits or match crashes.

All staging journals are restored with original hashes checked. No normal
Eden or Switch deployment occurred. Raw evidence is preserved outside Git:

```text
dist/4a927dc20904722d/runtime-evidence/native-display/
dist/4a927dc20904722d/checks/
dist/4a927dc20904722d/review/
dist/4a927dc20904722d/native-evidence-manifest.json
dist/84d48448ac91e65d/runtime-evidence/native-display/
dist/fps-investigation-20261007/
```

NSO arena, new Switch behavior and the concurrent external-window anomaly
remain unverified. No new latency, desync or FPS++ benefit is claimed by this
UI change. The FPS++ read-only trace is documented separately in
[FPS_BOOST_INVESTIGATION.md](FPS_BOOST_INVESTIGATION.md).
