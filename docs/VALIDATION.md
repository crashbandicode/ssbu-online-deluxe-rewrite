# Validation record

## Approved content-fit HUD installed on Switch, 2026-10-10

- Operator approved the8328 screenshots, requested FTP installation, then freshly
  confirmed Smash closed. Foreground DBI FTP192.168.0.226:5000 worked.
- Fresh4c19 NRO, latest INI, all other plugin bytes and protected HOC/boot files
  backed up. Exact approved8328 staged under a non-NRO name and fully read back;
  old NRO moved outside the plugin loader, then staged candidate activated.
- Final installed readback SHA256
  `8328cc88123bd4320c550a95b5b3cc7c699110ab65c158b9f862975ec2f29534`
  verified14:09:35.905632Z. Remote old-NRO backup readback matches4c19. INI SHA
  `0617a94705ea8d2da3fff782161b5339f6a3880948a807894a01692bccd9c2d3`, all eight
  other plugins, five HOC/boot files and nine plugin filenames unchanged. FTP closed.
- Evidence: `~/.local/share/ssbu-mod-backups/hud-content-fit-20261010/switch-ftp/`,
  deployment.json installed_verified, before/ snapshot and readback/ binaries.
  Rollback on SD: `/ultimate/ssbu_online_deluxe_rewrite/backups/pre-hud-fit-8328cc88123bd432.nro`.
- No new hardware gameplay result yet. User will test the content-fit two-row HUD,
  menu coexistence/returns, export completion and actual peer departure/reconnect.
  The original pre-preview manifest is immutable; device-validation.json records
  deployment separately. No general launch-crash fix or performance gain claimed.

## Content-fit HUD candidate, 2026-10-10 — images reviewed

- Candidate `8328cc88123bd432`, build `0.1.0-2db70f90aaae-dirty-1791637336`,
  NRO SHA256 `8328cc88123bd4320c550a95b5b3cc7c699110ab65c158b9f862975ec2f29534`.
- User feedback accepted f1ce's two-row look and requested that its background
  stop at the content. The only behavior change is compact ImGui width0; previous
  positioning/adaptive-font bound, two rows, bar, alpha and other views remain.
  menu.rs changes are comments. No new native dependency or Cargo.lock change.
- 265hosttests, warnings-denied host/SkylineClippy, no-default and optimized target
  builds pass. Six fit-1010-* gate receipts are in slim-hud-preview-20261009/.
  All76archived sources match checkout; exact f1ce diff, NRO/ELF text and debug
  sections independently verified in the immutable8328 bundle.
- Parent opened original Settings `hud-content-fit-1010a-settings-inmatch-settled.png`
  (6:42.95) and Performance `hud-content-fit-1010a-performance-inmatch.png` (6:54.26).
  Actual offline Isabelle/Toon Link match,1920x1160 full-window captures/1080p output.
  Both have exactly two inline rows and the same left edge. Backgrounds end after
  Stream:Off / Out:1920x1080 respectively; no near-full-width empty rectangle.
- Earlier settings-final.png is Results and excluded. settings-inmatch.png has
  correct layout but still includes an emulator-pause interval in its180-sample
  timing history; after8s resumed play the settled capture replaces it. Other
  navigation/retry captures remain preserved and excluded from the final pair.
- Exact candidate/config/startup/module and screenshot SHA values verified.
  Config is console layout with FX Off/all, not a default-FX or performance test.
  The6GiB cap was reached with zero observed OOM kills. No connected-peer/720p or
  Switch-compatibility conclusion; operator visual feedback remains pending.
- Owned unit eden-control-58c1ff0dc02b.service exited normally0 after native
  game.stop/confirmation/ui.close; independently inactive/dead/MainPID0. Runtime,
  worker and finite gate/capture jobs closed; GPU lock available. Original NRO/INI/
  Qt files restored and independently rehashed against the fresh journal backups.
- Evidence: `~/.local/share/ssbu-mod-backups/hud-content-fit-20261010/`, including
  result.json, parent-review.json, profile-journal.json and run-evidence/.
  Scoped artifact runtime-validation.json supplements the immutable pre-preview
  manifest. Final parent-closeout.json records watcher cleanup. No Switch deployment.

## Slim-HUD screenshot candidate, 2026-10-09/10 — images reviewed

- Candidate `f1ce158107e826bf`, build `0.1.0-2db70f90aaae-dirty-1791588649`.
  NRO2428928bytes, SHA256
  `f1ce158107e826bf6ccff44f2e97abf94f145968e78224ca8521f7ee326840bb`.
- 265 host tests pass; host Clippy with warnings denied and no-default build pass.
  Parent repeated the host gates after review. TargetClippy first identified a
  redundant eighth argument; deriving FPS visibility from the existing Config
  fixed it without suppression. Final SkylineClippy/release pass.
- Parent also fixed remote-field lookup to use stable field identity rather than
  the selected row's decorated label. Only menu.rs/runtime/ui.rs differ from4c19.
  All76archived files match checkout; Cargo.lock/native provenance unchanged;
  NRO/ELF text matches and debug sections are present.
- Host-only runtime UI tests originally executed zero tests; this was corrected
  by testing portable format/fit helpers in menu.rs. Tests do not assert pixels.
- Raw baseline4c19 in-match images reproduce the tall grid. Baseline's owned unit needed forced-stop
  after teardown timeout; no OOM. Baseline screenshots are not f1ce evidence.
- Evidence and gate receipts:
  `~/.local/share/ssbu-mod-backups/slim-hud-preview-20261009/`.
  No Switch deployment of f1ce; no new disconnect/export/crash acceptance.

The G&W benchmark completed and released its reservation before f1ce preview launch
at00:15:26Z on October10. Exact SHA/build/module bound through staging/launch receipts
and install.log. Final Settings image at5:45.13 and Performance image at6:41.65 are
actual offline1v1 Isabelle/Greninja match scenes, full-window1920x1160 with reported
1920x1080 game output. Parent independently opened both originals: exactly two
inline HUD rows, segmented timing bar, no title/table. Near-full-width translucent
strip with text on the left. No connected-peer or720p image was collected.

Original PNGs, sidecars, result.json and parent-review.json:
`~/.local/share/ssbu-mod-backups/slim-hud-screenshots-20261009/`.
Separate2x detail crops are labelled derivatives; original screenshots are unchanged.
Countdown, wrong-view and Results captures are retained but excluded from the final
pair. Copied Switch config is FX Off/all; this run does not establish default FX On.
The6GiB cap was reached with zero OOM kills; screenshots are not performance evidence.
Owned Eden unit exited normally0, runtime/worker/watch resources closed, three
original profile files restored and independently rehashed. Operator visual feedback
remains pending. Original package manifest keeps its pre-preview runtime flag;
current scoped evidence is in runtime-validation.json, with prior checkpoint retained.

## Operator-reported multi-person Eden lobby test, 2026-10-09

The operator confirmed that multiple people joining the lobby in Eden had been
tested by the Training Modpack agent. This is attributed operator-reported
coverage; the message did not identify the candidate hash, exact participant
count or test receipt. Keep the earlier artifact-linked lobby/peer-cycle results
below as their own evidence. This confirmation does not establish that the newly
reported disconnect/declined-rematch stale-opponent case was exercised, nor does
it validate the current menu/peer-state candidate. No new emulator run was made
to record this update.

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
bundle are recorded in the operator's evidence-root handoff (outside the repository). This candidate has no live acceptance yet.

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
The latest package and any subsequent runtime check are recorded in the operator's evidence-root handoff (outside the repository).

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

## Connected-only opponent display — 2026-10-07

Candidate `9e9ce2ab036d3e7ce06b74bc3ff770c03c569b6583af038c93f1f5972878ff69`
passed 230 host tests, host Clippy with warnings denied, the no-default-features
build, Skyline Clippy and an optimized Skyline package. Main-menu screenshots
showed the local-only table on both the standalone full-stack peer and the
embedded-backend peer. Joining an identified station displayed the remote
column; failed admission/disconnect cleared it again. Unknown RTT does not
hide an otherwise connected station.

A third independent station on the Surface joined the same private LAN room.
The host selected B and then C, and each selection displayed that station's own
name/profile/interface/RTT. Native local-wireless D-pad editing changed the host
buffer from Auto to 0. All three fighters were selected and ready. The host's
start chord was not sent before the finite run deadlines, so **no three-station
match or desync acceptance is claimed**.

The host used Mr. Game & Watch and the corrected G&W plugin `bceef1f0`; B used
Sheik without it, C used Banjo & Kazooie without it. Selected policies were
LLDoubles, LessLag and Vanilla respectively. These describe lobby configuration,
not proven match behavior.

Local runs `20261007-184606-b9a2f2` and `20261007-184607-a19c6b` ended at their
2400-second runtime bounds, with no OOM events and peaks 5,931,536,384 and
6,097,330,176 bytes. Remote C's 1800-second bound ended its run; its emulator and
supervisor were verified absent. All staged profiles were restored. This used
the wired LAN; neither NSO nor a VPN/high-RTT path was tested.

~~~text
~/.local/share/ssbu-mod-backups/combined-peer-native-9e9-20261007/
/tmp/delay-audit-01a10c4e-93fe-7db0-9d89-4bcd50bf2340/peer-native-9e9/
/tmp/deluxe-surface-1007/peer1/archive-0b2d5643/  (Surface)
~~~

## HUD placement candidate — 2026-10-07

Candidate `a9c38567b4e368d2d7723d760a2b7852503cde3688ba419e221e1bf03a794dcd`
adds right-side placement for validated Training and online lobby scenes.
Settings, performance and debug views share the rule. Regular singles/doubles
and the main menu stay left; native game banners are not relocated. The
position uses the current viewport and configured outer width with a 16-pixel
right margin.

234 host tests, host Clippy with warnings denied, no-default-features build,
Skyline Clippy and optimized Skyline packaging passed. Runtime placement
acceptance is pending; build checks alone do not prove screen placement.

## Reduced visual checks so far — 2026-10-07

Candidate `829727230ef34139` passed a controlled Gigaflare On/Off/Reduced visual
comparison: Reduced removed the first large flash while retaining the later
purple blast. Training save-state repeats retained the observed 49.3% damage
and 8 hits. This is a visual/damage observation, not a measured speed gain.

Candidate `3335efb82e7633ad` passed Palutena up-air On/Reduced/Off/On captures:
Reduced retained the blue wing cue while removing the bright rays/glare; On
restored the full effect. The up-air parent is deliberately protected in Off,
so its full appearance in Off is expected. Variant-load/use events 92/93
corroborated the Reduced path. The attack whiffed; no damage or performance
comparison is claimed for this trial. Other families still need runtime checks.

### a9c Training placement — observed 2026-10-08 00:02 UTC

Root visually reviewed the GNW worker's `training-hud-right.png` under
`/tmp/gnw-four-training-1007/`. Player Info is anchored at the right of the game
viewport in Training, with only local settings and no opponent column. Four G&Ws
are visible. This accepts Training placement for that binary; an online lobby
and an ordinary-match left-side regression are still separate checks.

The screenshot is not a performance sample: the menu is open and P1 is in an
action pose. The absent/enabled/absent idle benchmark was still running.

## G&W idle performance sanity check

[Three-window stock G&W check](GNW_FOUR_FIGHTER_CHECK.md): the enabled NRO executed its verified hooks and had steadier sampled FPS on Intel/ANV; all medians were near 60. This does not reproduce the original RTX bottleneck or prove a strict 60 FPS lock.

## 833c Explosive Flame measurements

[Thirty measured trials plus six warmups](https://git.syntaxbyexample.com/intpa/ssbu-fx-control/src/branch/main/docs/FX_EMULATOR_RESULTS.md) completed across
LessLag/LLUltra and On/Off/Reduced at 1×. No repeatable On-over-Off cost was
established; Reduced's performance benefit remains unverified. Raw evidence is
preserved with hashes outside Git. Paused screenshots are excluded from timing.

## 2026-10-08: 24e candidate and longer stress session

- NRO SHA-256: `24e714e77b3eabf0af188aec1a115f9f058b61beb7cc9119a29466ca2c26fc01`.
- Optimized ELF SHA-256: `22795134b469c7c8d84dc767ebf9bba4e67236fca0284ba3188b2ffeb837bc3e`.
- 242 host tests pass with eight threads; 20 further complete parallel runs pass.
  A process-global peer-table test race was fixed using a shared test-only lock,
  including readers that could contend with the nonblocking path under test.
- Host all-target Clippy with warnings denied, no-default build, Skyline Clippy
  with warnings denied, and optimized Skyline packaging pass.
- Full-stack Eden boot at 2x host rendering resolution passes. Offline window
  has no opponent column. FPS++ saved On displays `On (Unsupported)` and reports
  applied=0; restored Off afterward. A first status read at 1.5 seconds was stale
  and rejected; the later fresh report agrees with the visible label.
- Run `20261007-235700-988c0a` uses a two-hour deadline and 10 GiB/no-swap limit
  with startup/hang guards. Higher resolution is synthetic stress, not proof of
  a Switch-equivalent bottleneck. Further FX and linked checks are pending.

## 2026-10-08 — Inkling b99f policy correction

The corrected policy preserves `inkling_slosher_splash2`, the empty-ink
down-smash body, and filters `inkling_slosher_trace`, an additional full-ink
trail. The full-ink splash stays visible. Forward-smash `pablo_trace2` policy
is unchanged; its missing runtime cull evidence is not considered fixed.
The generator reproduces the checked-in policy after rustfmt.

Artifact `dist/b99f087cf8ef0a9c`:

- NRO SHA-256: `b99f087cf8ef0a9c58a6ed7d60ae294a492b8f28183f4d182d090d81ebb9af0c`.
- ELF SHA-256: `882b6ef8ee578a459d88d21e0fabf6f5d881821f959f7a8607c50fa5642fae3b`.
- Source archive SHA-256: `2626352bea8cd631a34b25e91de5bf069746c21d6bc69410956cd3efe3eb6d31`.

246 host tests passed. Host Clippy (all targets, warnings denied),
no-default-features build, Skyline Clippy (warnings denied) and optimized
Skyline packaging passed. Specific runtime acceptance is pending.
The source archive precedes this documentation checkpoint. Nothing in this
entry claims a new release, push or Switch installation.


## b99 linked regression and c0 diagnostics — 2026-10-08

- Exact runtime NRO: `b99f087cf8ef0a9c58a6ed7d60ae294a492b8f28183f4d182d090d81ebb9af0c`.
  Two local Eden instances, CLI commit `10bcd2d849843b146a79c61a21df615e1bdcfcde`.
  A embedded ImGui, G&W fix `bceef1f09e4339149cdab8e7826d17492680add02307002b0a5d62033d1090c8`,
  FX Reduced/online; B standalone ImGui plus Elite, no G&W fix, FX On/all.
- Private room join succeeded when the host challenger prompt was accepted promptly.
  Earlier Surface stock guest attempts discovered the room but failed before gameplay;
  they are not three-station desync tests.
- One seven-minute G&W-versus-Sephiroth Final Destination match: confirmed LessLag,
  then a live switch to LLUltra on both peers. Matching final winner, two KOs and
  47% damage; both returned to stage selection. No disconnect observed.
- Complete paired capture IDs: LessLag A `5319903644070`, B `5319902971610`;
  LLUltra A `5325013204722`, B `5325017052606`. All eight CSVs end
  `complete=1`. These concurrent traces establish active paths, not GPU performance.
- A's captured request log culls `edge_fire3_burst` and retains
  `edge_fire3_burst2` with online-only scope. Both peers restore Vanilla/three
  textures during results cleanup while retaining the saved LLUltra choice.
- Window position: right in connected CSS/stage selection; left in ordinary match.
  Opponent column disappears after B closes. B's Stealth hides its announced
  profile/interface from A; A's announced Ethernet is visible to B.
- Both units were explicitly stopped after testing. Each reported a core dump
  during that requested shutdown; neither is classified as a spontaneous match crash.
  B's original 24 staged paths were restored and hash-verified.
- Durable raw archive:
  `~/.local/share/ssbu-mod-backups/b99-runtime-20261008/runtime-evidence.zip`,
  SHA-256 `55a56f0c5c8882e7f6a9549c1b7a8605337cca4a6a60c5ea8bcc743c4d2cef82`.
  Disconnect supplement:
  `9eeea2bfda3c65ef6a67d32a3508acfc22ff5e2e8c8ae133e605b2c0dc45c3aa`.
- New diagnostic-only NRO:
  `c0e9a625eb44a7359545cb3264b2e7e58d45cf15ea7c7d79ba721fbb5be8dce7`;
  ELF `880160b92d3956c5a9c1b6bc97e8be169592d483511afc71a028911db56cd987`;
  source archive `56b299cef4fd54bf3ca824963e8cd23e4f18807702104021ecc839c9efbf91af`.
  247 host tests, host/Skyline Clippy with warnings denied, no-default build and
  optimized Skyline package pass. Initial host Clippy rejected hex grouping in
  the new test; formatting was corrected and the check rerun successfully.
  No rendering-policy fix is claimed. Runtime diagnosis is ongoing.


## c0 online-only FX scope in offline Training (2026-10-08)

The c0 diagnostic candidate, embedded menu, Training Modpack and SGPO completed
three fresh-state Sephiroth-versus-Ridley Gigaflare captures with
`effects_scope=online`: On 5383529109398, Off 5383900612625 and Reduced
5384281487286. Both CSVs in each pair ended `complete=1`.
All 490 / 487 / 488 callback samples respectively retained the requested mode,
scope Online, effective-Off false and hidden Deluxe window. Full Gigaflare body
and wash remained visible. Reduced emitted code86 (restore) for
`edge_fire3_screen2`, `edge_fire3_burst` and `edge_fire3_burst2`; no suppression,
cull or sibling-substitution decision appeared in these capture windows.
These are visual and scope checks with paused screenshots, not frame-time trials.
They do not close the same-session online-to-offline transition matrix.

Raw files: audittemp/embedded-c0-a/scope-online-training; screenshots in the
adjacent visuals directory, prefix c0-scope-online-training.


## c0 grounded triggered-counter visual check (2026-10-08)

Two finite captures per mode, all timing/API pairs complete:
On 5406959689907 / 5407200212605; Reduced 5407543855417 /
5407781445372; returned On 5408115596411 / 5408354185051.
All callback rows retain the intended mode, scope All and hidden Deluxe menu.
Root reviewed the contact/burst/expiry pictures of trial0 in each mode:
Donkey Kong's jab visibly strikes Scintilla; the shield reacts, the counter
connects and expires, and all three show 20.2% / four hits. This is a grounded
right-facing fixture with TMP Jab/Always, not airborne or opposite-facing coverage.
Both Reduced traces log cull84 for edge_senkou_start and edge_senkou_hold,
keep85 for edge_senkou_shield. hold_attack was not observed. Success-layer names
pass through unchanged and the filter logger omits them; screenshots provide the
cue evidence. No frame-exact gameplay equivalence or performance claim.

Raw audittemp/c0-all-a/counter-on-reduced-return and adjacent visuals,
prefix c0-counter. Pilot1 missed and is excluded; pilot2 established the fixture.
Paused screenshots explain the low sampled FPS shown in some images.

## c0 peer-report performance follow-up and network attempt (October 8)

- 54 Palutena callback windows completed: 18 excluded warmups and 36 measured
  trials, Vanilla/LessLag/LLUltra crossed with On/Off/Reduced and hit/whiff.
  No consistent On-specific penalty or Reduced performance benefit established.
  [Full method, results and limits](https://git.syntaxbyexample.com/intpa/ssbu-fx-control/src/branch/main/docs/FX_EMULATOR_RESULTS.md).
- All accepted API slices cover the guest-input-anchored move plus margin.
  API traces retain capacity termination; only callback traces cover all ten
  seconds. Fable independently verified hit raw files, not the whiff matrix.
- Original TMP counter settings were restored byte-for-byte before Palutena.
  No effect-removal asset mods were installed; no gameplay changes were made.
- The private emulator room reached three members across Yoga A and Surface
  C/D. C discovered the native room and its identity/RTT appeared at the host
  challenger prompt, but native joins did not complete. No match or desync
  acceptance follows. D's first boot was 1.0.0: copied DisabledAddOns entries
  explicitly disabled its installed 13.0.5 update. Root removed only those
  entries after verifying D stopped. Corrected D boot remains untested.
- Surface C/D were stopped and released by 18:01:46 UTC; both reported core
  dumps during requested shutdown. Yoga A's requested stop completed with
  MainPID0 and Result=timeout. These are shutdown outcomes, not spontaneous
  in-match crash findings.
- Both Yoga test profiles were restored with verified backup hashes: all 24
  B paths and the 14 A parent/diagnostic paths. The Surface D fixture remains
  stopped and isolated for a future explicitly coordinated test window.
- Raw archive: `~/.local/share/ssbu-mod-backups/c0-all-runtime-20261008/evidence.zip`.
  It includes raw traces, per-file hashes, fixtures, helpers, Fable review and
  network screenshots/logs. Archive SHA-256:
  `01274ef1a5bd912157502e24a490ab6c1bd88f44f284ee01cfc665c8c55107e2`
  (195,165,945 bytes; CRC verified).
- No Rust changes or new NRO were produced in this follow-up; the existing c0
  build gates remain the applicable evidence. No new push, release or Switch
  write occurred. The user's Switch still has b99.


## Compact renderer acceptance, 2026-10-09

The compact patch is built from public imgui-smash `bb47b205` plus patch
`4e138e3513347326`. Two clean builds produced identical archive
`6c9591cc9f140ca6`. Four C++ helper tests compile with warnings denied. The
archive build still emits existing upstream warnings, which are distinct
from the warning-free host and Switch Rust Clippy checks.

The final rewrite `7fdad40f10705afbe5fe336ccb0a58543db935143a6601cc4d0045d8296ff12d`
pairs with ELF `a77fe5fd440d02ff761158f4363cc3026299d90af21c3de800861ef192ead3fa`.
All 247 host tests, no-default build, host/Skyline Clippy with warnings denied,
and optimized Skyline build pass. Source packaging includes the exact patch,
pin, toolchain and dependency license. No defined-symbol GLOB_DAT/JUMP_SLOT
relocations point back into this module.

- Embedded run `menu-compact-a` used the preceding `28ac474abf75f1aa` binary.
  Removing one unused status accessor after that build changed only build-ID
  immediates/string bytes in the executable; no other code or data changed.
  At 3× Eden resolution it passed main/CSS Settings and Performance,
  LLDoubles/LLUltra/LessLag edits, two screenshot-confirmed stage→CSS returns,
  Link/Ness Training with the window on the right, unobscured native pause,
  Training Modpack open/close, SGPO display and Training→CSS return.
  Earlier short-hold scripts did not leave CSS and are not counted as returns.
  The run was slower than real time (roughly 24–52 FPS), so this is visibility
  and coexistence evidence, not a performance comparison. Final counters:
  28,188 submissions, 28,180 reuses, three rejected overlapping frame entries,
  zero pending-slot skips, failed slots, geometry skips or command overflows.
  Normal exit after 882 s; peak memory 7,007,911,936 bytes, no cgroup OOM.
- External run `menu-compact-b` used final `7fdad40f10705afb` and patched
  standalone diagnostic `99bdfa66a6002826`, with Elite, TMP and SGPO.
  Bootstrap ownership and external-ready checkpoints matched the rewrite's
  module. Both windows appeared together on main, at the start of an actual
  Lucario/Duck Hunt match on Hollow Bastion, on results, and after return to
  CSS. All three profiles and a separate Rules→CSS return passed. Captures
  named `match-input` and `match-playing` are results-screen images, not
  gameplay evidence; `profiles-match-both` is the actual match-start image.
  At 1× the observed menu/match run was approximately 60 FPS. Final native
  counters: 29,286 submissions, 29,278 reuses, seven overlapping entries skipped,
  zero pending/failed/geometry/overflow counts. Counter files are independent
  snapshots. Normal exit after 511 s; peak 7,786,078,208 bytes, no cgroup OOM.
- Boundary run `menu-csmall` used isolated small-budget diagnostic
  `3c8c44bf13d3dcd2` and archive `12bd6129936ebee7`. It reached CSS while
  10,218 command-memory overflows were safely discarded/reset, with zero GPU
  submissions or failed slots and three overlapping entries skipped.
  The absent overlay is intentional in this diagnostic. Normal exit after
  227 s; peak 6,670,692,352 bytes, no cgroup OOM. It is never a deployable NRO.

For the compact embedded runs, use the counters in `renderer-status.txt`.
Older `imgui-renderer-status.txt` and `frame-guard-status.txt` files in that
profile are stale pre-cleanup evidence. The external diagnostic writes fresh
files labeled with the current archive digest. All three profile journals
were restored after shutdown.

Artifacts, raw logs, screenshots, source and SHA manifests are preserved under:

```text
/home/intpa/.local/share/ssbu-mod-backups/menu-backend-build-20261008/
  final-compact/7fdad40f10705afb/
  completed-runs/menu-compact-a/
  completed-runs/menu-compact-b/
  completed-runs/menu-csmall/
```

These trials do not prove a deterministic cure of every invisible-window
case. The original failure was intermittent, and a guard-only rollback also
passed a later repeat. Switch validation and input-to-display timing remain
separate. No Switch files were changed for this campaign.

The final standalone release companion is
`9a9e575c910c6471152585459d8611d66eff08b7e2058c4f2a63711628a905ed`
with ELF `1aa78f5e7b208c2963ccedeb95bfca6a0dbdde8be99201a5862618eee3e3ce75`.
Its diagnostic features are disabled. Host Clippy/no-default build and
Switch Clippy pass with warnings denied, including one compile with both
diagnostic features. The corrected release log is warning-free.
`menu-release-b` smoke-tested this exact NRO with final Deluxe: both menus on
main and CSS, Performance view and valid external registration, around 60 FPS.
It exited normally after 210 s, peak
6,535,589,888 bytes; its four-file journal was restored.
The old standalone counter files were not updated in this release and are
not measurements of this run. The longer match test above uses the diagnostic
build of the same native backend.

## Buffered diagnostic export candidate, 2026-10-09

Local candidate `62a42df8d01d1738` addresses the observed incomplete API CSV export,
not the separate launch failures. Installed5c72's capture129729755560 had a complete
602-sample callback CSV, but its API CSV stopped at partial row6308 of6602.
The pinned Switch filesystem implementation requests Flush for each unbuffered
formatting fragment. The correction uses64KiB buffered writes, explicit fallible
flush followed by File sync_all, and Exporting/Saved/Error ordering. Capture
semantics, CSV serializers and no-overwrite creation guards remain the same.

- Final full host suite:257passed, including4export regressions. Byte equality,
  bounded underlying writes, mixed-sign API fields, write/flush/status errors,
  phase ordering and file overwrite protection are covered.
- `cargo clippy --all-targets -- -D warnings`, `cargo build --no-default-features`,
  `cargo skyline clippy -- -- -D warnings`, and `cargo skyline build --release`
  passed on reviewed source. The private helper is compiled for Switch and tests;
  no host dead-code suppression is retained.
- Build `0.1.0-2db70f90aaae-dirty-1791570881`; NRO2420736bytes, SHA256
  `62a42df8d01d17386a045fccd0c080770779223e9f7b62ddba9a665cc1da2bae`.
- ELF SHA256 `842875e692f6363a8b984c45c54f5b03ef49613e887729517fcffb859521c935`;
  NRO/ELF executable bytes match and debug_info/debug_line/symtab are present.
- All76archived source files match checkout. Compared with installed5c72 source,
  only src/lib.rs and src/runtime/timing.rs changed, plus new src/capture_export.rs.
  Native ImGui dependency provenance is identical.
- Exact bundle and immutable gate receipts are outside the repository under
  `~/.local/share/ssbu-mod-backups/capture-export-fix-20261009/`.

Not deployed, not runtime-tested. Hardware export completion/performance remains
pending; neither these gates nor buffering establish a launch-crash fix. The
operator has been released to play while separate crash/source audits continue.

## Horizontal menu and peer-retirement candidate, 2026-10-09

Local bundle `4c1947b84add6114` retains buffered-export62a and adds:

- Horizontal Settings for ready Singles/Doubles with0.42 window-background alpha;
  existing Training/lobby placement and vertical diagnostic/performance tables remain.
- Internal-only Platform and emulator-only FPS++ display/navigation. Boost safety
  gates remain unchanged.
- Actual ImGui content-fit height, including wrapped local/opponent values and the
  plot, replacing fixed or estimated row-count heights.
- Identity-wide retirement on a confirmed station leave, including other aliases
  that previously kept its peer visible. Explicit subtype3/4 disconnects also clear
  aliases. Connected peers with Vanilla settings remain valid; other lobby peers
  survive a departure. Unknown RTT protocols cannot retire a current station just
  because their numeric indices overlap.

Final parent gates passed:261hosttests, `cargo clippy --all-targets -- -D warnings`,
`cargo build --no-default-features`, `cargo skyline clippy -- -- -D warnings`,
`cargo skyline build --release`, exact package and source verification. Switch-only
Clippy found two style lints in the draft; both were fixed without suppressions.
The reconnect test uses an actual synthetic disconnect sequence, not manual cleanup
of the stale index. Host tests do not claim live renderer/network behavior.

Build `0.1.0-2db70f90aaae-dirty-1791574315`, NRO SHA256
`4c1947b84add611414cd485c7aa62a4ccfc05db35f7b1e2a8bac7def4aed7db6`.
NRO/ELF executable bytes match, debug_info/debug_line/symtab are present, all76
archived source files match the reviewed checkout, and native dependency provenance
is unchanged. Six files differ from62a: menu.rs, overlay.rs, peer_transport.rs,
pia_rtt.rs (regression only), runtime/imgui.rs (existing wrap API bindings), and
runtime/ui.rs. Runtime RTT hook source and buffered-export sources match62a exactly.
Evidence: `~/.local/share/ssbu-mod-backups/menu-peer-fix-20261009/`.

At package verification this was not deployed or runtime-tested; installation is
recorded below. The specific declined-rematch path still needs
runtime confirmation of its emitted lifecycle events; no speculative protocol
cleanup is shipped to force the UI to disappear. The operator's earlier multi-
person Eden lobby test above remains attributed to the Training Modpack agent.
Crash investigation is separate.

### DBI USB MTP installation — 2026-10-09 22:32Z

The operator requested copying4c19 with Smash closed using DBI's USB MTP responder.
Its `1: SD Card` storage was identified independently of the old Hekate block mount.
Only the active rewrite NRO was replaced after verifying local and device backups.
Full device readback matched SHA256
`4c1947b84add611414cd485c7aa62a4ccfc05db35f7b1e2a8bac7def4aed7db6`,2420736bytes.

Previous5c72 is backed up locally and on SD at
`/ultimate/ssbu_online_deluxe_rewrite/backups/pre-menu-4c1947b84add6114.nro`;
the remote backup was independently read back and hash-verified. All eight other
plugins and six protected files match before/after hashes; plugin membership is
still nine files. The freshly saved INI is LessLag/input3/FX Off/all/Stealth Off,
SHA256 `0617a94705ea8d2da3fff782161b5339f6a3880948a807894a01692bccd9c2d3`.

Evidence: `~/.local/share/ssbu-mod-backups/menu-peer-fix-20261009/switch-mtp/`,
including `preflight.json`, `deployment.json` (`installed_verified`), local backups
and downloaded readbacks. Both bounded jobs passed; the host MTP mount was released.
This accepts the transfer, not menu/gameplay/disconnect/export behavior or a crash fix.

## Plugin registration API candidate, 2026-10-12

Local candidate `068a1cf6f418349b` adds the exported
`ssbu_online_deluxe_plugin_api_v1` table, the `sdk/abi` ABI crate (a root path
dependency), the host registry `src/plugin_api.rs`, and a Plugins view in the
chord cycle that appears only after a separate NRO registers. Settings,
Performance and Debug rendering are unchanged; the compact match strip reuses
the approved two-row layout for the Plugins page with four-field paging.

- Host suite: 277 unit tests plus `tests/plugin_sdk_roundtrip.rs` (the public
  Rust SDK registering through a table of the exported shape into the host
  registry, then stepping values through the real callbacks).
- `cargo clippy --all-targets -- -D warnings`, `cargo skyline clippy -- -- -D warnings`,
  `cargo build --no-default-features`, `cargo fmt --check` and
  `cargo skyline build --release` passed.
- Build `0.1.0-2db70f90aaae-dirty-1791806695`; NRO 2445312 bytes, SHA256
  `068a1cf6f418349bd6379ed6beaaa1ade084e377a25077b548e2ad5ea9e28e0c`; module
  `8b1648f683c5795940ee3bf283a03d4559f7e9da5c0499f8bd6804259e178d70`; ELF SHA256
  `aa1c1b97dab38a64797faaf81576192e3f710d3b4d1405f573c747fbdc33999a`. NRO/ELF
  executable bytes match; `nm -D` lists the export as a defined `T` symbol.
- Demo consumer `sdk/examples/demo` (`cargo skyline build --release`): NRO SHA256
  `80e023e2773b0a1734564563aa798e877f68ddd667e9b1b1548ef9a01471d0dc`; it imports
  only `nn::ro::LookupSymbol`, has no undefined reference to the host symbol and
  carries the symbol string in rodata. SDK crate: 13 host tests and 1 doctest.
- Evidence: `~/.local/share/ssbu-mod-backups/plugin-api-tangent-20261012/`
  (`artifacts/068a1cf6f418349b/`, `DESIGN.md`, `SDK_WORKER_REPORT.md`).

Not deployed, not loaded in Eden, not runtime-tested. Page rendering,
controller flow and live cross-NRO registration remain unverified; the demo's
log line has not been observed. No replay recording is included.

Follow-up the same day added optional host persistence (plugin-declared INI
path; `OptionDesc` 48 bytes, `PluginDesc` 72 bytes, diagnostic 101). Root
gates rerun: 280 host tests plus the round-trip test, host/Skyline Clippy with
warnings denied, no-default build, rustfmt and Skyline release pass. The
repackaged candidate's hashes are in the tangent evidence directory's
`artifacts/` manifests.

### Eden preview `plugins-page-1012a` (same day)

Candidate `9d08a711b271e0ac` plus the demo NRO
`8c45cd0238f460900f2b453360ba0fce362a28cf06ec5f95e2dcfbdefa3f6b43` were staged
into the isolated Eden profile (three originals backed up, journal RESTORED
afterwards; the owned unit exited with status 0). Observed on the main menu:
the separately built demo registered and the Plugins page listed `Demo` with
Enabled, Mode and the read-only Status row; the chord cycle ran Settings →
Plugins → Performance → Hidden; Right on Enabled showed On and Down then Left
on Mode showed C. Host persistence created
`sd:/ultimate/deluxe_plugin_demo/config.ini` with `enabled=on` and then
`mode=c`; events.log recorded code 100 (request Applied) followed by code 101
(persisted). The boot-time code 99 record was lost to the 128-entry ring before
the sink started; absence of 99 in a boot log is therefore not evidence either
way. Not observed: the compact in-match strip for the Plugins page (no match
was started; navigation entered the World of Light map and was abandoned to
avoid touching that save) and any hardware run.
Evidence: `~/.local/share/ssbu-mod-backups/plugin-api-tangent-20261012/`
(`result.json`, `screenshots/plugins-page-1012a-*.png`, `run-evidence/plugins-page-1012a/`).

### Eden match preview `plugins-match-1012b` (same day, private display)

Same candidate and demo NRO, launched on a private display (weston headless
with the Intel GL renderer plus Xwayland `:2`, no desktop window; Eden at
1280x720, 60 FPS). An Astra driver reached an offline Smash match, Shulk versus
a level 9 Yoshi CPU on Final Destination. Observed in the match: the compact
two-row Settings strip and the compact Plugins strip
(`>Demo.Enabled:Off   Demo.Mode:A   Demo.Status:Ready` over
`Plugins 1/1` and the timing line); Right on Enabled applied (codes 100 and
101) and the host wrote `enabled=on`; after the match ended the window returned
to the vertical table showing Enabled On. Not captured in this run: the compact
Performance strip (match over first), a second edit and the hidden state. The
profile was restored and the unit exited 0.
Screenshots: `screenshots/plugins-match-1012b-settings-match.png`,
`plugins-match-1012b-plugins-match.png`, `plugins-match-1012b-plugins-match-edit.png`;
archive `run-evidence/plugins-match-1012b/`.

### Plugin value tones and read-only rows — 2026-10-12

Candidate `e59fc9a6bd266790` (descriptor version 2: per-choice tones and
`OPTION_FLAG_READ_ONLY`; table and symbol unchanged; version 1 descriptors
still accepted). 282 host tests plus the SDK round-trip; host/Skyline Clippy
with warnings denied, no-default build, rustfmt and Skyline release pass. SDK:
22 tests including the version 1 fallback. Eden run `plugins-tones-1012c` on
the private display with the demo NRO
`2f9fcdd730b14ab22fc6c19cfc0a6fe9f61d072379b1cb30a695e43b8f004b60`: on the
main menu the Status row rendered Degraded in yellow and Ready in green and
Enabled turned green after Right; Right on the read-only Status row changed
nothing. In a Lucario-versus-CPU match the compact strip showed On in green
and Failed in red while the status cycled. Profile restored, unit exited 0.
Evidence: tangent root `result-tones-1012c.json`, `screenshots/plugins-tones-1012c-*`,
`run-evidence/plugins-tones-1012c/`. Not on hardware.

### Window chord fix — 2026-10-12

Hardware logs from 9d08 (code 98 records in `switch-mtp/collect-1012/events.log`)
showed every failed L+R+Down attempt with no Down edge: the D-pad registered a
frame before both bumpers, so the old "Down edge while the chord is held" rule
ignored it, while ZL+ZR+Down worked because the triggers register later. The
Debug attempts (L+R+Minus, ZL+ZR+Minus) never had Down in the same sample.
Commands now fire when the whole combination is held, in any order, once per
hold (`overlay::CommandLatch`); chord + Plus/Minus alone toggles Debug, and
Debug stays open while the modifier is held. 283 host tests pass with the new
regression; gates as above. Hardware retest pending. Note: because the game
and native CSS panes still see Down before the bumpers, a Down-first chord on
character select can nudge the native profile row once before the window
opens; the old gesture (bumpers first) avoids it.

### Character-select frame budget under LessLag — Eden measurements, 2026-10-12

Twenty-one captures on the private display (candidate d773, isolated profile):
at 1x every mode holds 60 on the CSS, so Eden does not reproduce the Switch
drop directly. A CPU quota of 175% on Eden's own unit does reproduce a
LessLag-only drop (Vanilla 60, LLDoubles 60, LessLag 53-57) while all modes
show the same share of slow frames and the same main-thread time; the
three-image modes recover with short frames and the two-image ring cannot.
Core modes under quota separate the components: two images without render
options still drop, three images without them hold, and render options add
about 0.45 ms of serialized main-thread work per CSS frame. GPU stress at 2x,
3x and 4x never singles out LessLag. The CSS is not heavier than a match on
the instrumented spans; the hardware difference is frame-time variance.
Report and raw captures: tangent evidence `css-budget/`. No code change; the
hardware ABAB (LessLag, LLDoubles, Vanilla on one CSS visit; `lesslag_core`
and `doubles_core` boots) is the decisive confirmation.

### Character-select slowdown: call-level attribution and dispatcher backoff — 2026-10-12

Astra xhigh traced the CSS frame on Eden with host `perf`, the dynarmic perf
map, per-thread schedstat and a diagnostic probe build (worktree
`work/css-trace-probe`, never deployed). Result (tangent evidence
`css-budget/astra/CSS_SLOW_FRAME_ATTRIBUTION.md`):

- The mod's own CSS paths are negligible (native handler 0.005 ms, banner
  0.085 ms, scene-manager wrapper 0.136 ms per frame).
- The main loop (`main+0x374b898`) waits through `0x354ccd0` on dependency
  counter `0x5333160`. All 600/600 registrations on that counter resolve to
  one job, `main+0x375e260`, an `nn::ui2d::Layout` animation update on
  worker 1. The main thread waits 4.83 ms per CSS frame for it (8.25 ms under
  the 175% quota) versus 0.91 ms (1.33 ms) in a live match.
- While waiting, the main thread runs the task dispatcher `main+0x354a6d0`,
  which polls six queues through `std::mutex` → `nn::os::LockMutex` →
  SDK critical section: 1,313 of 1,350 state samples show all six queues
  empty, one pending record and counter 1. The dispatcher uses its light
  event only when the pending vector is empty, so it spins. The lock
  Enter/Leave fast paths are 22% of all Eden samples unlimited and 27.8%
  under quota; the lock word is free in 98% of samples (churn, not contention).
- Intervention A/B/A on the same binary and boot (1 µs `SleepThread` about
  once per 171 idle passes): CSS LessLag 175% 52.23 / 59.31 / 51.89 fps and
  27.84 / 22.90 / 28.08 CPU ms/frame; Vanilla 59.32 / 60.10 / 59.85 and
  26.90 / 23.04 / 26.27; LLDoubles 59.55 / 60.03 / 58.33 and 26.44 / 23.13 /
  27.84; CSS LessLag unlimited 59.08 / 60.00 / 59.80; live match 175%
  57.43 / 59.35 / 59.90 and 20.62 / 18.74 / 19.32. Main-core CPU fell
  11.70 → 6.67 → 11.63 ms/frame while worker-1 CPU stayed 5.7–5.8 ms: the
  gain removes polling, not UI work. Screenshots
  `css-budget/astra/screenshots/{before,after}-css-lesslag-cpu175.png`
  (54.1 → 59.4 fps, 27.2 → 23.3 CPU ms/frame).

Implementation (this build): `src/dispatcher_backoff.rs` (pure policy, host
tests) and `src/runtime/dispatcher_backoff.rs` (inline hook at
`main+0x354ab48`, the `ldp x8, x9, [x24, #0xe0]` after the sixth empty probe).
Install verifies all five original words (`a94e2708 eb09011f 54000d01
f94003e0 9411f072`); a changed first word with the rest intact means another
plugin owns the site (`External`). The hook sleeps only on `MainThread`, only
when the enabled bit is set, all six queue sizes are zero, the pending vector
`[d+0xe0, d+0xe8)` is non-empty with every record's counter still > 0, and
the ready-resume vector `[d+0xf8, d+0x100)` is empty. Default policy: 16
idle passes spin, then 5 µs doubling to 50 µs on every idle pass;
`dispatcher_backoff.ini` overrides (`preset=measured` reproduces the Eden
cadence, `cooldown_us` adds Astra's optional limiter). `DISABLE_BACKOFF`
turns it off. Host gates: 293 tests, clippy host and Skyline with warnings
denied, no-default build, rustfmt.

Eden production A/B/A of the clean 0.3.0 NRO (`f22d94950be1b460`, build
0.3.0-8aeaa5391343-1791833455; Astra low driver, isolated profile, private
display, 1x, FX On, 175% quota with 5 ms period, 17–20 s API windows;
ledger `css-budget/prod-ab/STATUS.md`, report `PROD_AB.md`):

| Run | Hook | FPS | Eden CPU ms/frame | Backoff row |
| --- | --- | ---: | ---: | --- |
| CSS LessLag 175% (A) | `DISABLE_BACKOFF` | 55.59 | 26.55 | Off |
| CSS LessLag 175% (B) | default policy | 60.00 | 19.50 | 3068/s 50 us |
| CSS LessLag 175% (A) | `DISABLE_BACKOFF` | 54.15 | 26.83 | Off |
| CSS LessLag 175% | `preset=measured` | 59.03 | 22.92 | 2744/s 1 us |
| CSS Vanilla 175% | default policy | 60.01 | 20.08 | 2590/s 50 us |
| Live match LessLag 175% (Lucario/Kirby, Battlefield) | default policy | 59.84 | 16.34 | compact strip has no row; code 103 rate 1069/s, last 50 us |
| CSS LessLag unlimited | default policy | 60.0 | — | — |

Install log code 102 outcome 1 (installed) on every boot; no crash,
`Unavailable` or `External`. A first match attempt that crossed into the
results screen during capture was excluded and repeated. Verdict: the
production hook reproduces the diagnostic gain and exceeds it on the CSS
(flat 60 under quota); Vanilla and the match keep 60 with lower CPU. The
in-match compact Performance strip does not show the Backoff row; the
vertical table does.

### 0.3.1 — character-select capture and backoff eligibility gate — 2026-10-13

Hardware result of 0.3.0 (operator, manual LessLag, FX On): CSS idle 58–60 fps
(was 48–50) but 46 fps while the cursor hand moves, with the default ramp and
with `preset=measured`; 60 in a live match; a clean two-image boot reproduces
46 (`css-budget/hand/hardware/clean-boot/renderer-status.txt`: active 2, index
masks 0x3, generation 1). Astra xhigh (`css-budget/hand/CSS_HAND_ATTRIBUTION.md`)
could not reproduce the hand drop on Eden (60 moving in every configuration)
and attributed the menu's weight to one serial UI traversal job on pool-0
worker 1 (core 1, kernel priority 36): 1,765 Layout, 19,201 transform and
about 14,200 enabled-binding entries per frame against 426 / 3,610 / 967 in a
fight, already while idle. Hover adds at most 20 bindings, no storage read
burst, no observed nested cleanup wait, sync waits near zero. Open hardware
discriminators: the conditional cleanup wait at `main+0x37a2b24`, asset loads,
two-image fence reuse, completion pickup. Rejected: another timer preset,
earlier dispatch or a split wait, `all_threads=1` as default, CPU boost.
Prescription: `CSS_CAPTURE_CONTRACT.md`, implemented here.

Build (commit 00660a3, crate 0.3.1):

- Eligibility gate in `src/runtime/dispatcher_backoff.rs`: a sleep requires
  `SCENE_SEEN` (any `auto_driver::set_scene` call, i.e. a scene hook fired)
  and a tracked present within 250 ms (`nvn::queue_present`). Before that the
  Backoff row reads `Armed`; code 102 `a=5` is written once when the gate
  first passes. The clock is read only on passes that would sleep. Astra's
  boot audit showed the hook sleeping before the title with two pending
  records on Eden; this keeps startup and a stalled renderer on the original
  loop. It does not claim to fix the `main+0x37432cc` initialization fault,
  which is original game code after `nn::prepo::Initialize`.
- `CAPTURE_CSS` boot flag → `src/runtime/css_capture.rs`: three hooks on the
  five-word 13.0.5 fingerprints of `0x3549720` (dispatcher registration),
  `0x354ccd0` (dependency wait) and `0x375e260` (UI job), installed after the
  renderer family verified the shared sites; `nn::os::GetThreadPriority` and
  `GetThreadCoreMask` resolved through `nn::ro::LookupSymbol` (never linked,
  never set). CSS entry (native banner `0x1a12f60` hook) records code 12 and
  arms a 30 s capture labelled `css`; exit (one second without a CSS update)
  records code 13. Rows: 60/61 per-frame sleep count, summed and peak actual
  sleep µs, idle streaks; 62/63 thread pointer, priority, ideal core and
  affinity mask once per thread per capture; 64 UI child dispatch (worker,
  queue priority, thread); 66 every dependency wait with the counter's image
  offset and thread label; 77 UI job span. Code 104 reports the hook mask.
- Timing capture 30 s, 2048 samples; API trace 65536 events. The static
  initializer is now all zero (`Finish::Aborted` first) so both buffers live
  in `.bss`: the NRO is 2174976 B against 2465792 B for 0.3.0.
- `tools/analyze_render_trace.py` decodes the kinds and adds a `css_capture`
  section (waits by counter, UI job stats, nested waits on the UI thread,
  per-frame backoff distribution, thread table, dispatch pairs); 24 Python
  tests.

Gates: 294 host tests, `cargo clippy --all-targets -- -D warnings`,
`cargo skyline clippy -- -- -D warnings`, `cargo build --no-default-features`,
`cargo fmt --check`, `cargo skyline build --release`. Clean package
`artifacts/9511e8c0f7946bd5/` in the tangent evidence: build
`0.3.1-00660a3d63cc-1791859184`, NRO SHA-256
`9511e8c0f7946bd50434d78f7607283a84f13d299dfe6e6e2aad88c77b43e840`
(2174976 B, module `ca2cbd7e…8780`), ELF `55f94118…9608`.
Not installed on hardware (the Switch has f22d / 0.3.0); not run on Eden.
Forgejo release `v0.3.1` (id 246, tag on ed8de66) carries the clean NRO
(asset 68, 2174976 B, download read back as 9511e8c0…e840) and
`manifest.json` (asset 69).

Switch install of 0.3.1, 2026-10-13 11:48Z (operator: "ship 0.3.1 to the
switch"): DBI MTP single-writer transaction `switch-mtp-9511/` in the tangent
evidence, preflight on f22d, rollback archive
`backups/pre-css-capture-9511e8c0f7946bd5.nro` read back as f22d, target read
back as 9511e8c0…e840, other plugins and protected files unchanged; empty
`CAPTURE_CSS` created. First boot: `install.log` code 104 `a=7 b=7` (all three
hooks and both SDK symbols), code 102 `a=5` (gate opened on the first scene),
codes 12/31/32 (CSS entry, capture armed, export complete).

First hardware CSS capture (`css-budget/hand/hardware/controller-captures/switch-0.3.1-1013/`,
API trace 33,075 events, 1,273 callbacks, elapsed, zero skipped; Local
Wireless CSS because NSO demanded a system update; the hand moved ~3 s, held
still ~11 s, then moved ~16 s). Astra's independent decode
(`css-budget/hand/HARDWARE_CAPTURE_1013.md`, reproduces the repository
analyzer exactly): still 48.3 fps (present cadence 20.69 ms) against moving
38.8 fps (25.79 ms); of the 5.10 ms increase, the texture-ready sync wait
(kind 3, after `nvnWindowAcquireTexture`) grows 3.99 ms and the main thread's
wait on the UI barrier grows 0.84 ms. UI job 11.41 → 12.21 ms mean, p99
11.92 → 15.70 ms; only 1 of 619 moving jobs exceeded 16.67 ms. Zero nested
dependency waits inside the UI job (all 7,631 kind-66 rows are MainThread).
Thread table: UI worker SDK priority 8, ideal core 1, mask 0x2; MainThread
priority 16, ideal core 0, mask 0x1, so the backoff cannot donate core 1 time.
Backoff: 227–241 sleeps per frame, 12.0–12.7 ms summed, peak 77 µs; job
return → barrier return 32 µs. Surviving candidates: two-image image/sync
coupling (dominant observed increment) and UI computation (attribution
target); excluded: the `0x354ccd0` nested cleanup wait and any multi-ms
completion pickup. The operator rejected LLDoubles as a trial or fallback; the
next steps are a clean offline Smash-mode CSS control and a flag-gated
per-owner attribution probe on two-image LessLag.

### 0.3.2 — always-on hang trace for loading-screen freezes — 2026-10-13

Context: with 0.3.1 and the operator's other plugins installed, the Training
Modpack froze on a loading screen on the console (no crash report; the
modpack's own log stopped after the fighter allocations) while the same stack
entered Training on Eden. The modpack investigation asked this plugin for
timestamped liveness lines it could align with its heartbeat. The two most
recent Smash crash reports on the card (2026-10-12 20:20Z, 2026-10-13 11:31Z,
both before 0.3.1) are `training_modpack` BRK aborts, not Deluxe.

Build (commits 082a6dc code, 85a7563 docs, crate 0.3.2):

- `src/hang_trace.rs` (pure, host-tested) and `src/runtime/hang_trace.rs`:
  ten tracked hooks with lock-free call counters, in-flight counts and entry
  ticks: `present` (tracked window), `backoff_pass`, `backoff_sleep` (around
  `SleepThread`), `css_init` / `css_wait` / `css_ui_job` (around the
  `CAPTURE_CSS` originals), `fighter_load` / `fighter_unload` (`common`
  module chain), `fighter_main` (inline per-fighter tick body) and `cut_in`.
  Hooks touch atomics only; the diagnostics thread writes
  `hang-trace.log` (rotates at 2 MiB to `hang-trace.previous.log`): header
  with `boot_tick` and `tick_hz`, `[Heartbeat]` every 5 s with
  `[ACTIVE xN age_ms=…]` on any hook that entered and has not returned,
  `[HangTrace] first <hook>` once per scene, `[HangTrace] scene a -> b` from
  `auto_driver::store_scene`. Installed right after the version check so a
  hang during hook installation still leaves the header.
- Zero-code bisection order documented in `docs/DEBUGGING.md`: remove
  `CAPTURE_CSS`, then `DISABLE_BACKOFF`, then the other `DISABLE_*` flags.

Gates: 298 host tests (4 new), `cargo clippy --all-targets -- -D warnings`,
`cargo skyline clippy -- -- -D warnings`, `cargo build --no-default-features`,
`cargo fmt --check`, `cargo skyline build --release`. Clean package
`artifacts/73823f8f67d6a3b2/` in the tangent evidence: build
`0.3.2-85a756321310-1791911904`, NRO SHA-256
`73823f8f67d6a3b2d3a0de35814042fa1ecea293fef01dae0828cbc1f6eefd8d`
(2183168 B, module `63b8b5fb…a8fc`), ELF `54fe3aa1…2e8a`. Not installed on
hardware (the Switch has 9511e8c0 / 0.3.1); not run on Eden.
Forgejo release `v0.3.2` (id 356, tag on 022a7d0) carries the clean NRO
(asset 70, 2183168 B, download read back as 73823f8f…fd8d) and
`manifest.json` (asset 71).

Switch install of 0.3.2, 2026-10-13 17:26Z (operator: "ship 0.3.2 to the
switch"): DBI MTP transaction `switch-mtp-7382/` in the tangent evidence,
preflight on 9511e8c0, rollback archive
`backups/pre-hang-trace-73823f8f67d6a3b2.nro` read back as 9511e8c0, target
read back as 73823f8f…fd8d, other plugins and protected files unchanged.
`CAPTURE_CSS` left in place. Runtime result pending.

Switch procedure for the capture: put `CAPTURE_CSS` in the data directory,
boot with manual LessLag and FX On, enter the character select, keep the
hand still for ten seconds, move it continuously for twenty, wait for the
Debug view `Timing` row to say saved, quit to HOME, then pull
`timing-<id>.csv` and `api-<id>.csv` (label `css`). Also compare LessLag
against LLDoubles on the same CSS while moving (zero-code ring test).

### 0.3.3 — API trace buffer allocated per capture, not resident — 2026-10-13

Context: on the evening of 2026-10-13 the console (0.3.2 + Training Modpack
r31 + the OBS companion's trace build) failed four boots before the title.
Atmosphère reports `01791924547`, `01791926109`, `01791926153`, `01791926317`
(tangent `hang-1013/boots-evening/crash/`) fault at address 0 in unrelated
game threads: `BCATCacheStorage` inside `memset` (twice), `render dispatch`
in `nnosBreak`, MainThread at `cross2_Release.nss+0x38b6804`. No plugin frame
is on any crashing stack; Deluxe's thread is idle in each. Deluxe's own
`hang-trace.previous.log` from one of those boots shows `present=0` and every
counter at zero for 80 s with `ready=1`: the game never produced a frame while
every Deluxe hook was installed and idle. Reading: allocations are failing in
the game heap that Skyline plugins are loaded from, and the loaded set grew
that day (modpack 2494464 → 3379200 B, companion 172032 → 212992 B).

Deluxe's part, measured on the packaged ELFs (`readelf`-equivalent section
sizes, tangent `artifacts/`):

| Build | NRO | .text | .data | .bss | resident |
|---|---|---|---|---|---|
| 0.3.0 f22d | 2465792 | 1577024 | 470688 | 55864 | 2.10 MB |
| 0.3.1 9511e8c0 | 2174976 | 1583120 | 175864 | 2218856 | 3.98 MB |
| 0.3.2 73823f8f | 2183168 | 1590880 | 176136 | 2219288 | 3.99 MB |

The jump is `render_trace::CAPACITY` = 65536 × 32 B moved into `.bss` in
0.3.1 so a 30 s CSS capture never ends on capacity. 0.3.3 keeps the capacity
but allocates the buffer in `Trace::begin` (diagnostics thread, never a hook)
and frees it in `take_ready` after the CSV export; `observe` returns without
writing when no buffer exists. No capture, hook, flag or row changed.

Gates: 298 host tests, `cargo clippy --all-targets -- -D warnings`,
`cargo skyline clippy -- -- -D warnings`, `cargo build --no-default-features`,
`cargo fmt --check`, `cargo skyline build --release`. Clean package
`artifacts/e930c8f34d1752bf/` in the tangent evidence: build
`0.3.3-528137cdff51-1791927586`, NRO SHA-256
`e930c8f34d1752bf24b1819cc8b86322abef3c2303ef50f3ef8544a5ae9d25f3`
(2183168 B, module `047e3f9d…0575`), ELF `db3da70f…c2c9`; packaged ELF
sections .text 1590848 / .data 176136 / .bss 122152, resident 2.23 MB (was
3.99 MB). Forgejo release `v0.3.3` (id 357, tag on ab53b43) carries the clean
NRO (asset 72, 2183168 B, download read back as e930c8f3…25f3) and
`manifest.json` (asset 73).

Switch install of 0.3.3, 2026-10-13 ~22:10Z (operator: "ship 0.3.3 to the
switch, ftp is live"): DBI FTP transaction `switch-ftp-e930/` in the tangent
evidence (temp-name upload, readback, rename; one connection per step
because DBI's server drops long or overwriting STORs). Installed NRO read
back as 73823f8f…fd8d (0.3.2) before, saved to
`backups/pre-trace-heap-e930c8f34d1752bf.nro` on the card (verified) and
locally; target read back as e930c8f3…25f3 after; the other nine plugins
unchanged. Runtime result pending.

0.3.4 (same evening, operator question "won't doing the dump crash it once
activated?"): in 0.3.3 the per-capture allocation used `vec!`, so arming a
capture on a game heap that could not spare 2 MiB would have aborted the
process. 0.3.4 reserves the buffer with `Vec::try_reserve_exact` and makes
`Trace::begin` return false when that fails; `timing::Service::poll` already
treats a false `begin` as "timing capture could not be armed" (status line,
both captures finished and drained), so the game continues without a CSV.
Allocation still happens only on the diagnostics thread. Gates as for 0.3.3
(298 host tests, host and Skyline Clippy with warnings denied, no-default
build, fmt, Skyline release). Package `artifacts/f514bfe81f142541/`: build
`0.3.4-f1d570ccb074-1791929407`, NRO SHA-256
`f514bfe81f142541e5211bc2a9302581fc0d162159d111d5ebd56df41a2042de`
(2183168 B, module `88c1ecdb…3d81`), ELF `fb75fa44…9acb`, sections
unchanged from 0.3.3 (.bss 122152, resident 2.23 MB). Forgejo release
`v0.3.4` (id 358, tag on 40edc90): NRO asset 74 (download read back as
f514bfe8…42de), `manifest.json` asset 75. Not installed on hardware (the
Switch has 0.3.3).

0.3.5 (operator: "I'd do all of those, test and ship") removes the capture
buffer as a memory question altogether:

- Rows are packed to 20 B (begin and span relative to the capture start, both
  32-bit; a 30 s capture is 576 M ticks) and reconstructed to absolute ticks
  on export, so the CSV columns are unchanged.
- Hooks fill one of two 4096-row chunks (`render_trace::CHUNK`, 160 KiB in
  `.bss`, the whole resident cost); the diagnostics thread drains the other
  into the already-open `api-<id>.csv` every 500 ms. A chunk hands over only
  after the other was drained; rows that arrive while both are full are
  counted in `skipped`, never blocked on. Capture length is bounded by the
  30 s duration, no longer by capacity (`Finish::Capacity` stays for old CSVs).
- Nothing is copied at the end: `take_ready` returns a `Summary` (id, reason,
  skipped, event_count); the file gets its footer, flush and sync. The header
  is written when the capture arms, so a full card or name clash refuses the
  capture up front ("timing capture could not open …").
- CSV layout: `# engine`, `# reason`, `# skipped`, `# event_count` move from
  the header to the footer before `# complete=1`. `tools/analyze_render_trace.py`
  already reads metadata lines anywhere and only requires the final line, so
  it needs no change; a host test checks the streamed bytes equal the
  single-shot `write_csv`.
- Release profile `opt-level = "s"` (measured in a worktree on this commit's
  parent: `.text` 1590896 → 1399936 B, NRO 2183168 → 1998848 B; `"z"` would
  reach 1307376 B; fat LTO changes nothing with `codegen-units = 1`). `"s"`
  was chosen to keep loop codegen in hooked paths.

Gates: 300 host tests (4 trace tests rewritten, 2 new), host and Skyline
Clippy with warnings denied, no-default build, fmt, Skyline release. Package
`artifacts/6b22546e0a057901/`: build `0.3.5-ee936bc148d5-1791930600`, NRO
SHA-256 `6b22546e0a0579011f37c3ff3780e7a9f9955226195424ff02665ad142a65dfb`
(2002944 B, module `04397ac6…380a`), ELF `be8a3ca5…4c50`; sections `.text`
1401296 / `.rodata` 342848 / `.data` 179360 / `.bss` 286128 (two 4096-row
chunks), idle resident 2.21 MB (0.3.4: 2.23 MB; 0.3.2: 3.99 MB). A running
capture adds about 190 KB of heap (one-chunk scratch plus the 64 KiB file
buffer) where 0.3.3/0.3.4 added 2 MiB plus a 2 MiB copy at export.

Switch install of 0.3.5, 2026-10-13 ~23:25Z (operator: "test and ship",
"sd card is connected via usb, copy over when done"): hekate USB mass
storage, tangent `switch-ums-6b22/deployment.json`. Installed NRO read back
as e930c8f3…25f3 (0.3.3) before, saved to
`backups/pre-stream-trace-6b22546e0a057901.nro` on the card (verified) and
locally; copied under a temp name, renamed, read back as 6b22546e…5dfb;
other plugins untouched (the Training Modpack on the card is now a
2174976 B build, changed by its own agent). Runtime result pending.
Forgejo release `v0.3.5` (id 359, tag on 08eedf1): NRO asset 76 (download
read back as 6b22546e…5dfb), `manifest.json` asset 77.

0.3.6 (operator: "Yes build the memory probe") adds the measurement the
memory investigation lacked: every hang-trace heartbeat carries the kernel's
accounting for the game process, `mem_used`, `mem_total`, `mem_free` and
`heap_region` in bytes, read with `svcGetInfo` (TotalMemorySize,
UsedMemorySize, HeapRegionSize on the current process; inline assembly beside
the existing `QueryMemory` in `runtime/memory.rs`). The diagnostics thread
refreshes the three values every 500 ms into atomics; the Debug view's
Performance table shows them as a `Memory` row (`used/total MiB, free`).
No hook changed; the UI path makes no system call. Because the same call
works under Eden, the console and the emulator can now be compared for
headroom at title, character select, Training and the moment a boot fails.
Host tests cover the field formatting and MiB label (301 tests). Gates as
for 0.3.5. Package `artifacts/792d88778925c0e6/`: build
`0.3.6-c45b2f91b4ee-1791935605`, NRO SHA-256
`792d88778925c0e6963cf5bead29b1158f73f1b535c59948ae20fef93938c5e4`
(2002944 B, module `18fe2ed2…7101`), ELF `9dbc04b1…b5b4`; sections `.text`
1402016 / `.data` 179368 / `.bss` 286160 (resident 2.21 MB, unchanged).
Forgejo release `v0.3.6` (id 360, tag on ec2847f): NRO asset 78 (download
read back as 792d8877…c5e4), `manifest.json` asset 79.

Switch install of 0.3.6, 2026-10-14 ~00:40Z (operator: "ship 0.3.6 to the
switch"): hekate USB mass storage, tangent `switch-ums-792d/deployment.json`.
Installed NRO read back as 6b22546e…5dfb (0.3.5) before, saved to
`backups/pre-memory-probe-792d88778925c0e6.nro` on the card (verified) and
locally; copied under a temp name, renamed, read back as 792d8877…c5e4. The
plugin set on the card at that moment: ARCropolis 2076672, css_preserve,
nro_hook, param_hook 53248 each, gamepad overlay 151552, OBS companion
126976 (down from 212992), Deluxe 2002944, Training Modpack 1769472 (down
from 3379200), stage pack 1351680, imgui 1282048. Runtime result pending:
the first boot's heartbeats give the first `mem_used`/`mem_total` numbers.

0.3.7 (operator: "do all of them, test and ship") trims what the plugin holds
while the game runs, after a symbol-level pass over the 0.3.6 ELF:

- Diagnostics thread stack 2 MiB → 256 KiB (`logging::DIAGNOSTICS_STACK_BYTES`).
  The Switch std (`sys/thread/switch.rs`) gives every Rust thread a 2 MiB
  pthread stack from the game heap unless the builder sets one, and Deluxe
  never did. The thread's deepest frames are the file writers; its buffers
  (64 KiB `BufWriter`, drain scratch, log lines) are heap. About 1.75 MiB of
  heap back for the whole run. Hardware check: heartbeats keep arriving in
  `hang-trace.log` and no crash report names `deluxe-diagnostics`.
- Pia RTT station table (`pia_rtt::Session`, 253 slots × 100-sample ring,
  about 107 KB, non-zero initialised so it sat in `.data`) is no longer a
  static. `runtime::pia_rtt::STATE` is `Mutex<Option<Box<Session>>>`: the
  first RTT sample of the process sets `WANT_SESSION`, the next diagnostics
  poll (≤ 500 ms) allocates with `Session::new_boxed` (zeroed heap block plus
  three scalar writes; equality with `Session::new()` is tested) and the table
  stays for the rest of the process. Until then the sample hook counts its
  sample as dropped, the leave and cleanup hooks have nothing to clear, and
  the poll publishes the unavailable reading and empty board an empty table
  would (`missing_table_publishes_like_an_empty_one`). Offline the table
  never exists. `.data` 179368 → 69912.
- API trace chunks 4096 → 2048 rows (`render_trace::CHUNK`, 80 KiB `.bss`); a
  chunk still holds about 1.9 s at the 1,100 rows/s measured on the hardware
  CSS capture against the 500 ms drain.
- Callback capture buffer (`timing::Capture`, 2048 × 32 B) is allocated by
  `begin` on the diagnostics thread (`try_reserve_exact`; a refused allocation
  refuses the capture) and moved into the snapshot by `take_ready`, freed
  after the export. The render path still never allocates. `.bss` 286160 →
  138672.

Not changed here: the embedded imgui backend (about 545 KB of `.text` plus
demo windows, fonts and glyph tables), which is being trimmed in the Forgejo
fork `intpa/imgui-smash` (tangent `imgui-fork-1014/`) for both the embedded
archive and the standalone companion plugin.

Gates: 303 host tests (two new), host and Skyline Clippy with warnings
denied, no-default build, fmt, Skyline release. Sections `.text` 1402336 /
`.rodata` 343136 / `.data` 69912 / `.bss` 138672, resident image 1.95 MB
(0.3.6: 2.21 MB); with the thread stack the steady-state cost drops by about
2.0 MB, more than the whole 0.3.1 regression. Package
`artifacts/8562746a292942cf/`: build `0.3.7-61d022bdc3db-1791945438`, NRO
SHA-256 `8562746a292942cf23df19f9e6781cc37b623dda305337161b730e14813707ea`
(1896448 B, module `c4beb790…d5f7`), ELF `55b8208c…cf9f0`.

Switch install of 0.3.7, 2026-10-14 02:38Z (operator: "do all of them, test
and ship"; the card was still mounted over hekate USB mass storage from the
0.3.6 install): tangent `switch-ums-8562/deployment.json`. Installed NRO read
back as 792d8877…c5e4 (0.3.6) before, saved to
`backups/pre-runtime-trim-8562746a292942cf.nro` on the card (verified) and
locally; copied under a temp name, renamed, read back as 8562746a…07ea
(1896448 B). Other plugins untouched. Deluxe's log files as they were before
the install are in `switch-ums-8562/before-logs/`; they show the console has
not booted Smash since the 0.3.2 build (`hang-trace.log` and `install.log`
both begin `build=0.3.2-85a75632…`, newest crash report 21:18Z on 2026-10-13),
so 0.3.3, 0.3.5 and 0.3.6 never ran on hardware either. Runtime result pending:
the first boot decides the 256 KiB stack (heartbeats continuing in
`hang-trace.log`) and gives the first `mem_used` numbers of the trimmed build.
Forgejo release `v0.3.7` (id 363, tag on d41ad35): NRO asset 80 (download
read back as 8562746a…07ea), `manifest.json` asset 81.

Hardware result, 2026-10-14 (operator, verbatim: "so everything boots and
seems to work now"): the console booted with 0.3.7 and the current plugin
set, the first boot since the 0.3.2 build, so this is the first hardware run
of 0.3.3–0.3.7 together, including the 256 KiB diagnostics stack. Heartbeat
continuity and the first `mem_used` numbers are to be read from the card's
`hang-trace.log` at the next connection.

0.3.8 (operator: "fork imgui to forgejo and look for ways to save there next
and replace the standalone with our fork after testing it", then "ship it
when it's done and when the imgui agent is done") moves the ImGui backend to
the Forgejo fork `intpa/imgui-smash` (private; `main` = upstream `bb47b205`,
branch `deluxe` = renderer correction + companion crate + memory trims, head
`7f0e47c`; details and the deferred-trim table in `docs/IMGUI_DEPENDENCY.md`
and tangent `imgui-fork-1014/REPORT.md`). Archive 17,181,302 → 13,851,556 B
(`8a06be16…`); `archive-pin.json` points at the fork and applies no patch
(`prepare_imgui.py` keeps the patch step optional; `tools/test_prepare_imgui.py`
updated). Deluxe's own code is unchanged; the embedded backend loses the demo
and metrics windows, the debug tools and two unused fonts. Sections `.text`
1402336 → 1179744, `.rodata` 343120 → 225200, `.data` 69912 → 48864, `.bss`
138672 → 131568; resident image 1.95 → 1.59 MB. The standalone companion
built from the same fork commit is 888,832 B (was 1,282,048;
`a3f794b2d2af15d5f2c2868f22e81c865702524160640864604731b7afe6775d`), with the
same export set, the `1.92.7 WIP (19263)` tag and `text_offset == 0`, so the
external-host detection is unchanged.

Eden acceptance, 2026-10-14 (isolated profile, private display `:2`, Astra
low driver `imgui-fork-eden-driver-1014`, evidence `imgui-fork-1014/eden-test/`):

- External (trimmed standalone + Elite, `ext-1-shots/`): `install.log`
  `overlay_host_external_chain` → `overlay_host_external_ready`; Deluxe Debug
  view, settings view and FrameTime graph render cleanly on the main menu and
  the character select; Elite's `L+R+Up` menu lists its 11 stages cleanly;
  60 fps, CSS 59.85; no crash, no assert, settings unchanged.
- Embedded (no standalone, `emb-2-shots/`): `overlay_host_bootstrap_hooked`
  → `overlay_host_ready`; Debug and settings views clean; CSS 59.99 fps.
- Reversed order (standalone renamed `libzz_imgui_smash.nro` so it sorts
  after Deluxe, plus Elite, `rev-3-shots/`): `overlay_host_external_ready`
  again; Debug view, Elite's stage list and the CSS Debug view clean;
  CSS 60.01 fps; no crash or freeze.

The isolated profile's plugin set was restored exactly afterwards
(`eden-test/staging-journal.json`, `restore_verified`).

Gates: 303 host tests, host and Skyline Clippy with warnings denied,
no-default build, fmt, `tools/test_prepare_imgui.py`, Skyline release.
Package `artifacts/c247bd377a422023/`: build `0.3.8-5c97c9c935fe-1791974104`,
NRO SHA-256 `c247bd377a4220233c4bf56470d23a6ea267b752b99cfd02eb3b26b480557222`
(1531904 B, module `34009bd1…251b`), ELF `ef694416…cb3b`; `imgui-provenance.json`
records the fork commit and the `8a06be16` archive.

Switch install of 0.3.8 plus the trimmed standalone, 2026-10-14 10:36Z
(operator: "ship it when it's done and when the imgui agent is done. switch
is connected via usb"): DBI USB MTP, two-target transaction
`switch-mtp-038/deploy-038-mtp-1014.py` (preflight backed up every plugin and
the protected files; `deployment.json` `installed_verified`). Deluxe read back
as 8562746a…07ea (0.3.7) before, saved to
`backups/pre-imgui-trim-c247bd377a422023.nro`, replaced by copy from a staged
read-back, read back as c247bd37…7222; `libimgui_smash.nro` read back as
9a9e575c…05ed before, saved to
`backups/pre-imgui-trim-standalone-9a9e575c910c6471.nro`, replaced, read back
as a3f794b2…775d (888832 B). Plugin names and every other plugin unchanged.
Runtime result on hardware pending. Before the install the card's logs
(`switch-mtp-038/before-logs/`) showed the 0.3.7 boot: 1135 s, 153 heartbeats
(the 256 KiB diagnostics stack held), kernel probe `mem_used` 1,764,012,032 at
the title → 2,536,943,616 in Training of `mem_total` 3,444,572,160
(`mem_free` 907,628,544), and four complete CSS captures (ids 2570630842
Vanilla, 4928335010 LessLag with 1775 skipped rows, 10646616362 Mixed,
18255281875 Vanilla), handed to the Astra xhigh analysis
`css-hw-037-analysis-1014` (`css-budget/hw-0.3.7-1014/`). The operator still
sees frame-rate dips on the character select.
Forgejo release `v0.3.8` (id 364, tag on 45e79af): NRO asset 82 (download
read back as c247bd37…7222), `manifest.json` asset 83.

Hardware CSS analysis of the 0.3.7 boot, 2026-10-14 (Astra xhigh
`css-hw-037-analysis-1014`, read-only; report
`css-budget/hw-0.3.7-1014/ANALYSIS.md`, reproduction scripts and per-callback
CSVs beside it, inputs hashed in `inputs-sha256.json`). Findings:

- The footer engine label compares arm with export only; every capture but D
  contains renderer round trips (A: Vanilla → LessLag 5.7 s → LLUltra 15.8 s →
  LessLag 23.3 s → Vanilla 24.8 s; B: LessLag → Vanilla 9.5 s → LessLag
  22.5 s; C: Vanilla → LessLag 29.3 s), and B, C and D leave the CSS during
  the 30 s. Whole-file averages are therefore not a Vanilla-versus-LessLag
  benchmark; the stable interiors are.
- In genuine two-image LessLag on the CSS the dips are **texture-ready fence
  stalls after acquire**, not long acquire or present calls and not backoff
  pickup. Capture A's continuous CSS visit: Vanilla zero-input 60.00 fps with
  1.93 ms texture wait; LessLag zero-input 52.55 fps / 3.49 ms; LessLag with
  directional input 41.35 fps / 7.67 ms; Vanilla again 60.00 fps / 1.79 ms.
  The directional increment of 5.15 ms per callback is 4.18 ms texture wait
  and 0.81 ms main UI barrier (81 % of the increment is the fence). B's late
  genuine LessLag CSS sits at a 30.00 fps plateau with 16.8 ms texture wait
  per 33.3 ms callback, with or without input. A's 30 fps plateau at
  15.8–23.3 s is LLUltra, not LessLag evidence.
- Backoff: no nested dispatcher wait (kind 66) overlaps any UI job; dispatch
  → job 22–23 µs mean, job return → barrier return 32–34 µs mean, largest
  single sleep 189 µs. Leave backoff and clocks unchanged.
- Separate mechanisms, not the CSS plateau: 290–300 ms UI-job stalls paired
  with CSS departure (B/88, C/668, D/244, D/1398), 1.3 s uninstrumented
  hitches outside the CSS (B/313, D/466), and 12–56 ms "other infinite sync"
  waits in Vanilla outside the CSS. D's second CSS visit dips with short
  texture waits but longer UI wall and flush, so Vanilla is not hitch-free and
  is not a menu fallback.
- Streaming loss: both API holes start exactly on a 2048-row chunk boundary
  (A 25.58–25.88 s, B 6.33–7.71 s), so the loss is drain/overflow on the
  diagnostics loop, not callback loss (timing has zero skips in all four).
- Memory: process `mem_used` 2177–2462 MiB of 3285 MiB during the captures,
  free 823–1108 MiB; liveness, not a bandwidth or fragmentation measurement.

Ranked next steps, all in existing visual-request hooks and keeping genuine
two-image LessLag: (1) measurement integrity first (latch engine/scene
changes per capture, separate lock-contention from overflow skips, report
drain time and chunk high-water marks, collapse redundant per-frame field
rows); (2) `PROBE_RENDER_POLL` discriminator at the render-wait hook
main+0x387029c (restore the original zero-timeout poll while keeping two
images) as a matched baseline→probe→baseline experiment; (3) hide the
Settings overlay as a control (every capture had it visible); (4)
`DISABLE_LESSLAG_FIELDS` discriminator at main+0x384fa10, separately; (5)
frame/sync identities on the NVN wrappers to find the producer of the
pending fence. None is a shipping fix yet; no LLDoubles, Vanilla-on-menus,
clock or memory changes are recommended for the dips. Follow-up capture
acceptance: one CSS visit, 10 s still then 20 s repeatable directional
sweep, zero API overflow, engine and scene constant.

0.3.9 (operator: "go ahead and do the draw slot trims too, test and ship
with astra's findings") takes the first deferred fork trim. Fork commit
`c185303` (pin `77cdab7`): each of the backend's eight draw slots held a
256 KiB + 256 KiB command pool and a 64 KiB + 64 KiB control block of game
heap (5 MiB in all) for recordings that never exceeded 1,256 B of command
memory or 120 B of control memory across the stored Eden evidence. The
budgets are now 32 KiB + 32 KiB command memory (exactly one 64 KiB NX
memory-pool page) and 8 KiB + 8 KiB control memory per slot, 640 KiB in all;
the reserve still finishes and discards one oversized frame, and exceeding
base plus reserve remains an abort, which is why the Eden runs below read
the live counters. Budgets and rationale moved to
`command_memory_reserve.hpp`; `command_memory_reserve_test` asserts the
planned layout on the NX constants (pool exactly 65536, control 16384, also
on a 4 KiB page and with device minimums). Archive
`09b296a0f5c67641d229bb4a6d8b170f75281368da3957d666af424476bc2731`,
13,851,844 B, identical in two clean Docker builds; `archive-pin.json`,
`build.rs` and `tools/test_prepare_imgui.py` follow. Deluxe's own code is
unchanged. Standalone companion from `77cdab7`: 888,832 B,
`946de84f98546cb791640e40fbf5848ea3965c511631ef18b32e5c805b67d50e`, same
segment layout, exports, version tag and `text_offset == 0` as the 0.3.8
companion; a `draw-stats` build (`ef0e57bf…2c04`, 905,216 B, not shipped)
adds the 2 s logger thread that writes `imgui-renderer-status.txt`.

Eden acceptance, 2026-10-14 (isolated profile, private display `:2`, Astra
low driver `imgui-slots-eden-driver-1014`, brief and evidence in
`imgui-fork-1014/eden-test/`, `slots-N-shots/NOTES.md` per run):

- Run 1, external with the draw-stats companion + Elite (`slots-1-shots/`):
  `overlay_host_external_ready`; Deluxe Debug/Settings/Performance and
  Elite's stage list open together on the main menu and on the CSS, then a
  Shulk vs Wolf match on Small Battlefield with the compact strip (20 s,
  pause, quit, results, CSS return), all clean at 59.9–60.0 fps. Final
  `imgui-renderer-status.txt`: 29,658 submissions, 29,650 reuses, 0 pending
  skips, 0 failed, 0 geometry skips, **0 command overflows, max 1,448 B
  command / 120 B control** (a new maximum, 4.4 % of the 32 KiB initial
  budget). A shared Right press during the quit prompt flipped Stealth on;
  the driver restored it (config.ini `stealth=false`, `config.previous.ini`
  shows the transient). Eden hit its 8 GiB host cgroup cap during the match
  (`memory.events max 2470`, no OOM kill) and the game stayed at 60 fps;
  the unit's SIGBUS core-dump at `systemctl stop` is the same teardown
  pattern as all three 0.3.8 runs.
- Run 2, embedded (`slots-2-shots/`): `overlay_host_ready`; all views on
  the main menu and the CSS, three portrait hovers with Settings open,
  60.0 fps. Final `renderer-status.txt`: 13,201 submissions, 13,193 reuses,
  3 busy-frame rejections, 0 pending/failed/geometry/overflow, max 1,040 B
  command / 120 B control. Clean exit at stop, peak 6.25 GB.
- Run 3, reversed order (release companion as `libzz_imgui_smash.nro` +
  Elite, `slots-3-shots/`): `overlay_host_external_ready` again; Debug,
  Performance and Settings each open beside Elite's stage list on the main
  menu and on the CSS, Elite cursor moved Up/Down only, 59.95–60.0 fps;
  all settings unchanged (`config.ini` `stealth=false`); no crash, assert
  or window loss. The release companion has no counters; its slot code and
  archive are the ones measured in run 1.

The profile's plugin set was restored exactly afterwards
(`eden-test/staging-journal.json`, `restore_verified` 12:40Z).

Gates: 303 host tests, host and Skyline Clippy with warnings denied,
no-default build, rustfmt, `tools/test_prepare_imgui.py`; fork host tests
(`command_memory_reserve_test`, `draw_slot_rules_test`, `frame_guard_test`,
`frame_guard_concurrent_test`) with warnings denied; fork crate Clippy and
no-default build; Skyline release. Package `artifacts/8c20799f3974f45b/`:
build `0.3.9-4fd77a238f9e-1791980278`, NRO SHA-256
`8c20799f3974f45beaf28c2373982bcc546f08f6496ee58ed95852fd78496cca`
(1531904 B, module `8910f609…628b`), ELF `871d513f…d2fe`;
`imgui-provenance.json` records fork commit `c185303` and the `09b296a0`
archive. Fork branch `deluxe` (head `77cdab7`) and tag `deluxe-0.3.9` are
pushed to Forgejo; fork release `deluxe-0.3.9` (id 366) carries
`libimgui_smash.nro` (asset 88, read back as 946de84f…d50e) and
`libimgui_smash.a` (asset 89, read back as 09b296a0…2731).

Switch install of 0.3.9 plus the new standalone, 2026-10-14 12:41Z (same
order): DBI USB MTP, two-target transaction
`switch-mtp-039/deploy-039-mtp-1014.py` (preflight 12:41:22Z backed up every
plugin and the protected files; `deployment.json` `installed_verified`
12:41:56Z, 27 device commands, all exited). Deluxe read back as
c247bd37…7222 (0.3.8) before, saved to
`backups/pre-draw-slots-8c20799f3974f45b.nro`, replaced by copy from a
staged read-back, read back as 8c20799f…6cca; `libimgui_smash.nro` read back
as a3f794b2…775d before, saved to
`backups/pre-draw-slots-standalone-a3f794b2d2af15d5.nro`, replaced, read
back as 946de84f…d50e (888832 B). Plugin names, every other plugin, config.ini
and hekate_ipl.ini unchanged. Runtime result on hardware pending (the 0.3.8
install of 10:36Z had not booted yet either; the card's `hang-trace.log`
still ended with the 0.3.7 boot). Forgejo release `v0.3.9` (id 367, tag on
01132f5): NRO asset 90 (download read back as 8c20799f…6cca),
`manifest.json` asset 91.

Caveat for the CSS capture: the SD writes now happen during the measured
interval, from the diagnostics thread. Compare the next hardware CSS capture
against `switch-0.3.1-1013` before trusting a movement-penalty number from it.

This is a footprint change, not a
proven fix: the next console boot with 0.3.3 and the companion plugin rolled
back decides; the memory bisect is plugin removal, not Deluxe flags.

### LessLag render-poll candidate — 2026-10-14 (commit b044752, no version, no toggle)

Operator: "implement and test the fix Astra proposed for the fps drops",
then "build it without the version, prove it works in Eden and switch then
tag a new version once validated. no toggle, it either works or it doesn't
and will become the standard if it does or ripped out if it doesn't help".
`render_policy::skip_wait` now skips the game's timeout-zero render poll at
main+0x387029c only for LLUltra and the Doubles shapes; two-image LessLag
(and LessLagCore) run the original poll like Vanilla. `PROBE_RENDER_POLL`
still forces the poll for the policies that skip it. 304 host tests, host
and Skyline Clippy with warnings denied, no-default build, rustfmt, Skyline
release. Package `artifacts/021a03dee556fdc4/`: build
`0.3.9-b04475229832-1791982758`, NRO SHA-256
`021a03dee556fdc45bece92ab320c0f7e94f08d8449facfe421d2b7489015164`
(1531904 B). Crate version stays 0.3.9; no tag, no release.

Eden A/B (tangent `css-budget/render-poll-ab-1014/`: `PROTOCOL.md`,
`SUMMARY.md`, `RESULTS.md`, `compare.py`, `arms.json`, `captures/`,
`shots-*/`): isolated profile, embedded host, Deluxe alone, FX On, overlay
hidden, private display `:2`, Astra low drivers; Eden's systemd unit under
`CPUQuota` 175 % then 120 % with a 5 ms period; one 30 s `CAPTURE_TIMING`
window per arm, 10 s still then a 20 s left-stick sweep, on the
Training-mode character select with LessLag live (two textures confirmed in
`renderer-status.txt`). Findings:

- At 175 % neither build dips (0.3.0's backoff removed the CPU waste the
  earlier campaign measured there). At 120 % both dip: callback intervals
  over 20 ms were 8.7 % (A1, 0.3.9), 2.8 % (B1, candidate), 3.5 % (A2,
  0.3.9), 4.6 % (B2, candidate); cadence 55.8 / 58.5 / 58.1 / 57.7 fps.
  The candidate sits inside the reference spread: **no regression, no
  measurable gain on Eden**.
- The poll costs 0.001 ms per frame and returned ALREADY_SIGNALED in 5,276
  of 5,277 frames; Eden's texture-ready wait is 0.03 ms in every arm, so the
  emulator never reaches the fence stall the change targets (hardware
  Vanilla returned TIMEOUT from the same poll in 8–62 % of frames).
  The decision is the console's.
- Side finding: the regular Smash character select runs **Vanilla** after
  any Singles/Doubles/Training-classified scene until the profile is
  re-selected (`auto_policy::transition_cleanup` cleanup hold; the Smash
  submenu classifies as Doubles before the CSS, so a fresh boot → Smash →
  CSS is Vanilla as well). Hardware capture D matched this. LessLag CSS
  observations therefore come from re-selected or Training CSS visits.

Switch install of the candidate, 2026-10-14 13:49Z (operator: "prove it
works in Eden and switch"): DBI USB MTP, `switch-mtp-poll/deploy-poll-mtp-1014.py`
(`preflight.json` 13:49:38Z, `deployment.json` `installed_verified`
13:49:56Z). Deluxe read back as 8c20799f…6cca (0.3.9) before, saved to
`backups/pre-render-poll-021a03dee556fdc4.nro`, replaced by copy from a
staged read-back, read back as 021a03de…5164; standalone, other plugins,
config.ini and hekate_ipl.ini unchanged. The card's `hang-trace.log` and
`install.log` still end with the 0.3.7 boot, so the next console boot is
the first for 0.3.8, 0.3.9 and this candidate together. Hardware test:
LessLag on the Training CSS (Games & More → Training → start → Quit
Training → CSS) and on the regular CSS after re-selecting LessLag;
`CAPTURE_CSS` is still on the card, so each CSS visit records 30 s; compare
the kind-3 texture wait and the > 20 ms share with
`switch-mtp-038/before-logs/` (0.3.7 LessLag windows 41–52 fps, B 30 fps).
If it does not help, revert b044752 and reinstall 0.3.9 from the rollback.

**Hardware verdict, 2026-10-14 14:37Z: the candidate does not help, and it
is reverted (commit c0fa483) and off the card.** The console booted the
candidate (`switch-mtp-poll/boot-logs-1437/`, build
`0.3.9-b04475229832-1791982758`, 1110 s, `mem_used` 2.66 GB) and recorded
four CSS captures. The event log (`AutoRequest` code 74, `ProfileTransition`
code 46) dates them: capture 298481440895 at 81 s is genuine two-image
LessLag with the poll running, the operator still (0–11 s) then moving the
hand (12–30 s); 304026704152 at 370 s is LessLag after two Vanilla↔LessLag
re-selections; 312302874387 at 801 s is Vanilla under the cleanup hold
(`req=0 sel=3 hold=1` at 775.6 s); 313596223198 at 868 s starts under the
hold and commits LessLag at 877.7 s when the profile was re-selected.
Numbers (`RESULTS-hw.md`, `compare.py`):

| Capture | Live policy | Still: fps / > 20 ms / texture wait | Sweep: fps / > 20 ms / texture wait mean / p95 | Poll rows / results |
|---|---|---|---|---|
| 298481440895 | LessLag + poll | 59.4 / 1.0 % / 2.35 ms | **49.9 / 20.2 % / 4.56 / 16.4 ms** | 1593 / all 0 |
| 304026704152 | LessLag + poll | 50.6 / 18.5 % / 4.22 ms | 48.9 / 19.7 % / 5.72 / 16.8 ms | 1483 / all 0 |
| 312302874387 | Vanilla (hold), no input | 59.3 / 0.8 % / 1.70 ms | 58.5 / 0.4 % (no input) / 3.40 / 12.1 ms | 1590 / all 0 |
| 313596223198 | Vanilla → LessLag at 9.5 s | 52.6 / 4.9 % / 5.85 ms | 58.6 / 1.9 % / 4.59 / 10.7 ms | 1690 / 1688×0, 2×2 |

The poll returned ALREADY_SIGNALED in every LessLag frame (the sync it
checks is already complete in a two-image ring), so running it changes
nothing: the directional-input penalty is unchanged from the 0.3.7 captures
(LessLag windows 41–52 fps) and the texture-ready fence tail is still 16 ms.
Astra's rank-1 discriminator is therefore answered: removing the poll is
**not** what makes the fence miss. The remaining candidates from that list
are the hidden-overlay control, the `DISABLE_LESSLAG_FIELDS` discriminator
and frame/sync identities on the NVN wrappers. The card was returned to the
released 0.3.9 NRO at 14:39Z (`switch-mtp-poll-revert/deployment.json`
`installed_verified`; the candidate is archived as
`backups/candidate-render-poll-021a03dee556fdc4.nro`).

Display defect found by the operator on this boot: the native CSS banner
read "You: LessLag | 3" while the renderer ran Vanilla with three textures
under the cleanup hold (the menu's Engine row showed Vanilla). Manual mode
labelled the saved profile regardless of the live request. Fixed in
`auto_policy::profile_row_label`: a manual selection whose request is Vanilla
(only the cleanup hold produces that) is shown as `Vanilla*` in the banner,
the arena lines and the settings row; tests updated. Not yet on the card.

### Present-at-flush candidate — 2026-10-14 (commit 8687eee, no version, no toggle)

Operator, after the render-poll verdict: "letting the UI job overlap with the
previous frame's GPU work instead of serialising behind it — let's try that".
The console capture `switch-mtp-poll/boot-logs-1437/` (genuine LessLag)
gives the main thread's frame order relative to the game's submit: UI
dispatch 0.2 ms → UI job 0.2–10.7 ms (13.2 ms with the hand moving) with the
main barrier on it → other dependency waits to 13.7 ms → **present of the
frame submitted at 0 ms at 13.7 ms** (16.3 ms moving) → acquire 14.2 →
texture-ready wait to the vsync at 16.4 ms (fast) or **33.1 ms (slow)** →
late flush and flush with the next submit inside its span (1593 of 1595
frames). So the UI job already overlaps the GPU; the stall is the present
being issued only after the next frame's whole CPU update, so that a
hand-moving frame presents after the vsync and the two-image acquire waits
for the following one. Present index always equals the preceding acquire
(1592 of 1592).

Change: in committed idle two-image LessLag the flush hook
(`renderer.rs`) presents the acquired image at the end of the flush
(`nvn::present_at_flush`); the game's own present of that index, 13.7 ms
later, is skipped while its loop bookkeeping still runs, and the
per-present crop moves with the real present. `render_policy::
present_at_flush` gates it (not during a swap, drain, forced wait or after
a refusal; Vanilla, Ultra and Doubles keep the game's order). Counters
`early_presents`, `early_present_skips`, `early_present_mismatches` in
`renderer-status.txt`; code 105 records a mismatch. Trace: early present
= kind 1 result 2, skipped game present = kind 1 result 1. Build
`0.3.9-8687eee688c9-1791989848`, NRO
`aa214ec238e680b3a714837b7142351c1e567a1406762171696e96e98933a586`
(package `artifacts/aa214ec238e680b3/`). 305 host tests, Clippy host and
Skyline, no-default, fmt, Skyline release.

Eden P1, 2026-10-14 15:03Z (tangent `css-budget/present-at-flush-1014/`,
same protocol and 120 % quota as the render-poll arms, Training CSS with
Mario selected, Astra low driver): 1,786 early presents, 1,786 skipped
game presents, 0 mismatches; screenshots clean (one banner-entrance frame
with white sparkle quads, gone in the next frames; to be rechecked).
Early present lands 0.17 ms after the submit, the skipped game present
11.4 ms after it.

| Arm (120 %, whole 30 s) | Cadence fps | p95 ms | p99 ms | > 20 ms |
|---|---:|---:|---:|---:|
| A1 0.3.9 | 55.83 | 31.2 | 33.2 | 8.7 % |
| A2 0.3.9 | 58.13 | 18.3 | 32.6 | 3.5 % |
| B1 render-poll | 58.53 | 18.0 | 32.1 | 2.8 % |
| B2 render-poll | 57.67 | 19.5 | 32.7 | 4.6 % |
| **P1 present-at-flush** | **59.54** | **18.0** | **22.0** | **1.5 %** |
| **P2 present-at-flush** | **59.87** | **16.9** | **19.2** | **0.7 %** |

The 33 ms mode is gone on Eden in both arms: the first candidate whose p99
is not the doubled vsync. P2 (15:10Z, full Training path, Mario selected)
counted 13,634 early presents, 13,633 skipped game presents and 0
mismatches over the whole boot; sweep window 59.8 fps, 0.6 % > 20 ms,
p99 18.9 ms. Sweep coverage confirmed from the button column in both arms.

Switch install, 2026-10-14 15:06Z: DBI USB MTP, `switch-mtp-flush/
deploy-flush-mtp-1014.py` (`preflight.json` 15:05:43Z, `deployment.json`
`installed_verified` 15:06:01Z); Deluxe 8c20799f…6cca (0.3.9) → aa214ec2…
a586, rollback `backups/pre-present-flush-aa214ec238e680b3.nro`; other
plugins, config.ini and hekate_ipl.ini unchanged. Hardware test: LessLag
CSS (Training CSS, or re-select LessLag on the regular CSS) with hand
movement; `CAPTURE_CSS` records each visit; compare the kind-3 wait and the
> 20 ms share with `switch-mtp-poll/boot-logs-1437/` (candidate b044752,
same hardware day) and `switch-mtp-038/before-logs/` (0.3.7). Watch for
flicker, tearing or a stale frame on the CSS and in a match. If it does
not help or shows artifacts: revert 8687eee and reinstall the rollback.

### Match capture (`CAPTURE_MATCH`) — 2026-10-14 (commits aa7ae78, 210f83f)

Operator: "go ahead and build the match capture now and ship it so I can
share both results when I come back." The finite capture pair now records a
whole match: with `CAPTURE_MATCH` in the data directory at boot, every stage
load (stage-setup hook; Singles, Doubles, Training) arms `timing-<id>.csv` +
`api-<id>.csv` (label `match_*`), and the capture ends when the game leaves
the stage (`# reason=scene_end`): scene status back to Inactive, or the
character-select update running (210f83f; the Training CSS keeps the
Training status). Ten-minute ceiling (`elapsed`). Both CSVs stream from the
diagnostics thread, which polls every 250 ms while a capture runs; the
callback buffer is drained like the API chunks (the 2048-sample buffer bounds
nothing but a stalled drain), and API rows pack `begin` relative to a
per-chunk base (first row minus 2^31 ticks), so the 20-byte row survives
captures far beyond the former 223 s limit; a row outside its chunk's range
starts the other chunk instead of ending the capture. A 30 s capture still
running when a stage loads is ended for the match (code 106 `a=3`).
`CAPTURE_MATCH` also installs the three read-only `CAPTURE_CSS` hooks (rows
62–66, 77) without arming on CSS entry. Code 106; Debug row `Match capture`;
`tools/analyze_timing.py` accepts `scene_end` (duration check only for
`elapsed`). No hook does I/O; the render-path change is bookkeeping in
`Capture::observe` (first/last tick kept in the state instead of the buffer).

Eden smoke tests (isolated profile, embedded host, no CPU quota, Astra low
driver `match-capture-eden-driver-1014`, `css-budget/match-capture-1014/`):

- Run 1 (aa7ae78): Training stage load armed `capturing 15295074307251
  match_training` at 15:49:50Z; the files grew while the match ran (API CSV
  2.8 → 5.4 MB in the first 60 s); the Training exit to the CSS did **not**
  end it (status stayed Training), found and fixed in 210f83f. The partial
  files copied at 372 s (`runs/match-smoke-1/`): 607,954 API rows in strict
  sequence, no `end < begin`, no backward tick jump, begins 0.005–372.2 s
  after the id, kinds 1/2/3/5/12–17/29–34/60–64/66/77 (present rows twice
  per frame: the early present and the skipped game present), 21,447
  callback samples contiguous with increasing ticks over 358 s, one partial
  trailing line (copied mid-write). 60.0 fps throughout, no dialog or
  freeze.
- Run 2 (210f83f, `runs/match-smoke-2/`): two Training stage loads armed
  `match_training` captures 15306224507360 (15:59:32Z) and 15308367429258
  (16:01:17Z, re-entered from the Training CSS); both ended on the Training
  exit within the driver's 4 s poll (`# reason=scene_end`, events.log code
  106 `a=1`, 31, 106 `a=2 b=0xc55`, 32 for each), footers complete:
  159,579 API rows (6 skipped) + 5,715 callback samples over 96.3 s, and
  76,375 rows (1 skipped) + 2,741 samples over 45.9 s. `analyze_timing.py`:
  p50 16.67 ms, p99 17.7 / 17.6 ms, 26 / 5 intervals > 20 ms with the stage
  load and exit inside the window (max 347 / 227 ms). `analyze_render_trace.py`
  (now accepting `scene_end`): present rows with indices 0/1 and results
  1/2 only, so the present-at-flush path was live. 60 fps in Training, no
  dialog or freeze; the 47 fps sample at the second arm is the stage load.

Switch install, 2026-10-14 16:03Z: DBI USB MTP, `switch-mtp-match/
deploy-match-mtp-1014.py` (`preflight.json` 16:03:38Z, `deployment.json`
`installed_verified` 16:03:57Z); Deluxe aa214ec2…a586 (present-at-flush
candidate) → 4e4f2044…db56 (build `0.3.9-210f83fcc733-1791993370`, the same
candidate plus the match capture), rollback `backups/pre-match-capture-
4e4f2044edb3d3b5.nro`; `CAPTURE_MATCH` created in the data directory next
to the existing `CAPTURE_CSS`; other plugins, config.ini and hekate_ipl.ini
unchanged. Rollback chain on the card: 4e4f2044 → aa214ec2
(`pre-match-capture-…`) → 8c20799f 0.3.9 (`pre-present-flush-…`). Hardware
test: play matches with the flag; each stage load writes a `match_*` pair,
pull them with the CSS captures. No version tag until the console verdict.

### Console verdict and 0.4.0 — 2026-10-14 16:40Z

First boot of 4e4f2044 (16:11Z, config had drifted to Auto/Vanilla/2 during
the b044752 session via 23 menu saves the operator did not make) crashed on
MainThread in the game's `StandardAllocator::Initialize` during a scene
transition at t≈33–38 s, memory unmapped, no Deluxe frame in the chain,
Deluxe idle (`switch-mtp-match/boot-logs-1611/CRASH-1611.md`). Config
restored to manual LessLag, 3 frames, streamer off (readback verified);
the second boot ran 743 s with two online matches and did not crash.
Operator: "it booted, felt buttery smooth. fps reflected it too… this was
def a good version so I would ship it after confirming in the logs."

Hardware captures (`switch-mtp-match/boot-logs-1640/`, build 210f83f, same
console as `switch-mtp-poll/boot-logs-1437/`):

| capture | scene / engine | fps | p99 ms | > 20 ms |
|---|---|---|---|---|
| b044752 day, css sweep (reference) | CSS LessLag + poll | 49.9 | 33 | 20.2 % |
| 482801144422 css, sweep 10–30 s (1,169 directional samples) | CSS LessLag | **59.85** | **20.2** | **1.3 %** |
| 482801144422 css, still 0–10 s | CSS LessLag | 60.00 | 16.8 | 0.0 % |
| 484909232936 match_singles, 148 s | online match, LessLag | 59.66 | 16.9 | 0.4 % |
| 489614260886 match_singles, 263 s | online match, LessLag | 59.88 | 16.8 | 0.2 % |
| 484546224171 / 489237967027 match_training, 17 s each | Training incl. load | 53.3 / 53.4 | 83 / 80 | 5.3 / 5.1 % |
| 488410188899 css (cleanup hold, re-selected mid-capture) | CSS Vanilla → LessLag | 52.7 | 83 | 5.7 % |

Texture-wait p95 on the LessLag CSS sweep fell from 16.4 ms to 2.95 ms;
every LessLag frame shows the early present (result 2) and the skipped game
present (result 1), 0 mismatches (no code 105). The match frames over 20 ms
are the stage load at the start of each capture (max 249 / 139 ms). The
Training captures are 17 s long and dominated by the load. The second CSS
capture ran under the cleanup hold (Vanilla, 270 native presents) until the
profile was re-selected mid-capture, so it is not a LessLag measurement; it
confirms the `Vanilla*` state exists on the console after a match. The
`Mixed` engine label on the match captures was the export reading the
engine after the results-screen cleanup hold, fixed by sampling the engine
per poll (this release). Peak memory 2.50 GB of 3.44 GB.

Open items from this boot: the opponent window stays after the opponent
declines a rematch (operator report; `peer_transport::frame_from` keeps
`Selection::Station` while the observer view still says known); the scene
connection stayed `offline` through two online matches although the latency
hook fired (code 30), so Auto would have classified them offline Singles.

Release 0.4.0: present-at-flush (8687eee), `Vanilla*` banner (4f8bcfb),
match capture (aa7ae78, 210f83f), engine label fix. Tag `v0.4.0` on 1224ea0,
build `0.4.0-1224ea0f3521-1791998073`, NRO
`04c3008d0182a48eb81ede39ec9f3eec7f380adcc187198b66aacf6a494c6afb`
(1,531,904 B), module id `1bf822fd…dd82`, package `artifacts/04c3008d0182a48e/`;
Forgejo release 370 with the NRO and manifest, download readback verified.
Card: `switch-mtp-040/deploy-040-mtp-1014.py`, 4e4f2044 → 04c3008d, rollback
`backups/pre-0.4.0-04c3008d0182a48e.nro`, `CAPTURE_CSS` + `CAPTURE_MATCH`
kept, config manual LessLag / 3 frames.

### 0.4.1 — 2026-10-14: the three follow-ups from the 0.4.0 console boot

Operator: "okay go ahead and build those and ship the minor version to the
usb. I'll keep playing and when an fps drop occurs I will let you know."

- **Opponent window after a declined rematch.** The station-leave hook
  (`pia_rtt` `STATION_LEAVE_OFFSET`) calls `peer_transport::clear_station`,
  whose `apply_leaves` retired the opponent only through the constant id
  bound to the leaving index. That binding comes from the station-map hook,
  which is part of the opt-in peer observer (off in the operator's config), so
  the id was 0 and the opponent stayed on the banner through the single-peer
  fallback (`view_of`: one confirmed peer is shown without a selection) until
  the whole session ended. `apply_leaves` now retires the only confirmed peer
  when an unbound station leaves; two or more confirmed peers are left alone
  (test `an_unbound_station_leave_retires_the_only_confirmed_peer`). Not yet
  seen on the console; the leave hook firing on a declined rematch is the
  assumption (the 16:40Z boot had no peer-observer rows to confirm it).
- **Online matches classified `offline`.** Two Quickplay matches ran with
  the connection still Offline (no quickplay/arena scene hook fired; the
  latency hook did, code 30). `Scene` gains `marked_online` (the latency
  mark); `decide` uses `effective_connection()`, which counts a marked
  Offline stage as Quickplay for Auto only. Menus and the waiting room stay
  Vanilla, classified connections are never rewritten, manual selections
  ignore the mark, and `mark_match` re-evaluates the request. The first match
  of a session therefore swaps to LessLag when the latency hook fires (~3 s
  after the stage load) instead of never; later matches in the session are
  LessLag from the stage load (the mark clears only on the main menu). Why
  the scene hooks miss the entry remains open.
- **Config drift.** The 23 saves during the b044752 session were native-pane
  edits: on an online character select the fork-style D-pad editing changed
  the buffer (Left/Right) and the profile (Up/Down) while the operator moved
  the hand with the D-pad. Native edits now require ZL held (ZL+ZR stays the
  view chord; L+R chords unchanged); the Settings window still edits with the
  plain D-pad. Overlay tests updated; README controls table.

Gates: 308 host tests, host and Skyline Clippy with warnings denied,
no-default build, rustfmt. Tag `v0.4.1` on a466c1a, build
`0.4.1-a466c1a2662c-1791998997`, NRO
`3408557fb5925e7df3999683d7ca69a2873caded53033cbed1f3baff1f9cea37`
(1,531,904 B), package `artifacts/3408557fb5925e7d/`; Forgejo release 371
with the NRO and manifest, download readback verified. Card 17:32Z:
`switch-mtp-041/deploy-041-mtp-1014.py`, 04c3008d → 3408557f, rollback
`backups/pre-0.4.1-3408557fb5925e7d.nro`, flags and config unchanged. Host
verified only at install: the runtime delta from the console-verified 0.4.0 is
the three fixes above; the operator plays it next.

### 0.4.2 — 2026-10-14: why the scene hooks "missed" the online entry

Operator: "we should find why the hooks miss the entry." The events of the
b044752 session (`switch-mtp-match/boot-logs-1611/data/events.previous.log`,
code 73 `AutoContext` with the source hook) answer it:

| t (s) | context | source |
|---|---|---|
| 72.6 | quickplay / inactive | `0x22da160` (Online → Smash menu) |
| 307.8 | offline / inactive | `0x235aaa0` (main menu) |
| 406.8 | offline / training (waiting room) | `0x25d9288` (stage setup) |
| 422.6 | offline / singles (the match) | `0x25d9288` |
| 425.9 | — | `0x16ccab8` online-latency hook (code 30) |
| 860.4 | quickplay / inactive | `0x22da160` |
| 877.7 | quickplay / training | `0x25d9288` |

The Quickplay hook fires. The main-menu hook then reset the connection while
matchmaking continued in the background, and the waiting room and the match
started from the main menu without passing the Quickplay hook again. The fork
kept Quickplay across the main menu for exactly this reason (its comment: the
internal matchmaking transitions trigger this hook briefly); the rewrite
reset unconditionally. Secondary facts: the latency hook fires 3.1–3.3 s
after the stage load on every console match recorded; the Eden runs show
`0x22da160` firing on an offline menu (Games & More → Training produced
`quickplay/training`), so Quickplay from the menu hook alone is not proof of
an online match; the events ring (128 entries, 500 ms flush) drops rows in
stage-load bursts, so the 16:40Z session's missing menu rows are not evidence
of the hooks not firing.

Fix (this release): `auto_policy::main_menu_connection` keeps a stored
Quickplay on main-menu entry with no liveness check (anything else resets to
Offline; code 107 `a=1`), and a Singles/Doubles stage that has run five
seconds under a stored Quickplay with no online-latency mark is demoted to
Offline on the next diagnostics poll (500 ms cadence; `offline_match_due`,
`OFFLINE_CHECK_TICKS`; code 107 `a=2`, context source `0x0ff11e`). The mark
itself clears only when the connection is set to Offline, so a kept Quickplay
carries an earlier match's mark through the main menu. Auto therefore runs online matches as online from the stage load
(the 0.4.1 latency-mark fallback remains for the first match of a session that
somehow never passed the menu hook) and corrects a stale Quickplay five
seconds into an offline match instead of never. Manual profiles unaffected.
Gates: 309 host tests, host and Skyline Clippy with warnings denied,
no-default build, rustfmt. Tag `v0.4.2` on ddea22d (rewritten history),
build `0.4.2-ddea22d1d814-1792000378`, NRO
`87322eefca507870593c1ffbcdf70b5452a342bc1cf543b9ce43a692cad9c7e0`
(1,531,904 B), package `artifacts/87322eefca507870/`; Forgejo release 372
with the NRO and manifest, download readback verified. Card 17:54Z:
`switch-mtp-042/deploy-042-mtp-1014.py`, 3408557f → 87322eef, rollback
`backups/pre-0.4.2-87322eefca507870.nro`, flags and config unchanged.
Host-verified only at install; the console verdict on the Quickplay hold
comes from the next online session (`AutoContext` rows with sources
`0x235aaa0` and `0x0ff11e`, code 107).

Eden evidence from the documentation-media run (0.4.2 NRO, GIF build,
`docs-media-1014/`, events decoded by source): `0x22da160` fired at 162.7 s
on the offline path Games & More (connection quickplay with no network),
the Training stage followed at 227.7 s, and the main-menu hook at 437.9 s
and 542.8 s kept Quickplay (code 107 `a=1`) while the status went Inactive
and the cleanup hold engaged (`AutoRequest` hold bit, request Vanilla). The
regular Smash character select entered after the Training session was
classified Training by the CSS update (494.4 s): `is_training_mode()` still
reads true there, so the hold lifted and the banner read `LessLag | Auto`;
the Settings window at the main menu afterwards read `Vanilla*`. No
Singles/Doubles stage ran, so the stale-Quickplay demotion was not
exercised. Screenshots and GIFs from this run are in `docs/media/`.

A second media run (`docs-media-1014/NOTES.md` "Run 3") played an offline
Smash 1v1 against a level 9 CPU with 0.4.2: the compact two-row strip,
Performance and Debug views in the match, the results screen after quitting,
and the regular character select afterwards reading `You: Vanilla* | Auto`,
the first capture of the `Vanilla*` banner label (4f8bcfb) on a character
select after a match. The driver noted that the Plus in the Debug chord also
opens the game's pause menu in a match, which is the game's own binding.

Repository history: at the operator's request the `HANDOFF.md` file was
purged from every commit, tag and branch (`git filter-branch --index-filter
… --prune-empty --tag-name-filter cat -- --all`; bundle backup of the
pre-rewrite repository in the evidence root), and `AGENTS.md` no longer asks
for a handoff file in the repository. Every tag was re-pointed to its
rewritten commit; the Forgejo releases keep their assets.

### 0.5.2 — 2026-10-15: coverage build and the first Eden coverage campaign (offline and local wireless)

Operator request: light execution-coverage tracing of the Eden build to
inform code reduction without blind removal, with the rule that code is not
kept because a unit test exercises it. Implementation (commit "Coverage build:
instrumented variant with a Rust raw-profile writer"): cargo feature
`coverage`, `tools/Build.ps1 -Coverage`, per-package `-C instrument-coverage
-Zno-profiler-runtime` through `tools/coverage/config.toml`, and
`runtime::coverage` as the whole profiler runtime (defines
`__llvm_profile_runtime`, serialises the `__llvm_prf_*` sections as raw
profile format 10 through `coverage_profile`; `live.profraw` every 10 s,
`scene-NNN-P-S.profraw` at scene changes, `manual-NNN.profraw` on Debug
L+R+Left; event code 109). Host proof: a program built with the same flags and
the same writer merges in `llvm-profdata` and reports exact per-function
counts in `llvm-cov`. Gates: host clippy with and without the feature, 297
tests, no-default build, Skyline clippy with and without the feature, Skyline
build. Coverage package `artifacts/7df44f460e8f9aa3/` (build
`0.5.2-35ec596fc1a9-dirty-1792077376`, NRO 1,765,376 B, manifest
`"variant": "coverage"`, 1,392 instrumented functions, 4,676 counters).

Campaign (Eden CLI-control `10bcd2d849`, private display, `platform=emulator`,
manual LessLag, buffer 3, DRS flags off; game driven by an Astra low agent,
snapshots and staging by the primary): six scenarios — boot to Training;
the Player Info window in Training through every profile, buffer edit, view
cycle, Debug view, 30 s timing capture, manual dump and combat; Training
exit, main-menu Stealth/Streamer toggles and two Smash CPU matches; and a
two-peer local-wireless session (profiles A and B through an unlisted
password-protected emulator room on localhost, `CAPTURE_CSS` and
`CAPTURE_MATCH` present, SDK demo plugin installed on the host): lobby,
challenger accept, character select with both native status lines (18–21 ms
peer latency shown), a Sonic versus Lucario match on Final Destination with
the compact strip, Plugins 1/1 page, Performance and Debug views, about two
minutes of combat, the quit chord and clean room exits. The first LDN
attempt's host was OOM-killed by its 6 GiB cgroup cap while the match loaded
(kernel `oom-kill`, not a guest fault); the lobby/CSS profile was kept and the
second attempt ran at 8 GiB without incident. Cumulative snapshots were
differenced with `tools/coverage/profraw_diff.py`.

Result (`tools/coverage/report.py`, llvm-cov total 1,383 functions in the
map, 70.4 % regions, 69.0 % lines): 1,091 of 1,422 instrumented functions in
`src/` executed in at least one scenario (77 %); 331 never (107 closures, 22
const constructors/trait impls); 202 named never-executed functions spanning
2,723 physical lines; 2,188 lines never executed inside functions that ran.
The never-executed code is, in order of size: Nintendo Switch Online paths
(Arena room and station board, Quickplay hooks, about 750 lines; unreachable
in any emulator session), flag-gated diagnostics from closed investigations
(`DUMP_PRESENTATION_HISTORY`, `PROBE_FIGHTER_MODULE`, `DUMP_RENDER_PROBE`,
`PROBE_CROP`, the `dispatcher_backoff.ini` parser; about 740 lines), features
off in this configuration (FPS++, DRS and the 0.5.2 triggers; about 560
lines), plugin-API paths needing an option edit or an own-page plugin,
defensive and teardown paths, and 21 production helpers (115 lines) with no
production caller. Two groups of functions share identical aggregate counts
in every scenario (27 per present, 24 per diagnostics poll); equal totals,
not proof of shared or dispensable work. Analysis, tiers and the deduplicated
reduction inventory (Tier 1 unused helpers 115 source lines; Tier 2 opt-in
diagnostics and the backoff override file ≈ 1,100 source lines plus ≈ 180
test lines; together about 5 % of the non-test source) are in the evidence
root's `coverage-1015/REPORT.md`, reviewed by an Astra high session whose
nine findings (two FPS++ helpers wrongly listed as unused, `window_finalize`
wrongly grouped with the crop probe, an unsupported poller-gating claim,
double-counted totals) are applied; the report is evidence for review, not a
removal list. Scenario
gaps stated there: no NSO session, no first boot without `config.ini`, no
plugin option edit, no own-page plugin, no console run. Both Eden profiles
were restored afterwards (journal `RESTORED`, SHA-256 verified).

### 0.5.2 — 2026-10-15: response trials after present-at-flush for LLUltra and LLDoubles (Eden, captured images)

Operator request: re-measure the README input-delay table on the 0.5.2
present-at-flush build (commit ce0145a, package `artifacts/1525e3e0392c8403/`,
build `0.5.2-ce0145a2155c-1792070522`, NRO SHA-256
`1525e3e0392c840306073762cb7e78670745fe0e925de013f660c901371cfdff`), because
LLUltra's lead over LessLag might have shrunk. Method as in
LATENCY_METHODOLOGY.md: one uninterrupted CLI-control Eden `10bcd2d849`
(Vulkan/docked/1×/100%, `ui.screenshot_height=360`, 6 GiB cap) on a private copy
of the 0.5.0 profile (Deluxe only plus Training Modpack, gamepad overlay,
Arcropolis, css_preserve, nro_hook, param_hook; no FX Control), Sephiroth versus
Kazuya CPU Stop in Training, `INPUT_RECEIPT` marker, input buffer Auto,
`platform=emulator`, `auto_profile=false`. Four profiles × two injection phases
× three 20-image trials = 24 trials (480 native 640×360 images), normal-phase
order Vanilla, LessLag, LLUltra, LLDoubles, delayed order reversed; the
committed mode and `engine_status=Active` were read from `renderer-status.txt`
before and after every group. Captured-image response `R` (images from the
injection after image 5 to the first crouch image):

| Profile | Normal | Delayed | Saved vs Vanilla, normal / delayed |
| --- | --- | --- | --- |
| Vanilla | 6, 6, 6 | 6, 6, 6 | — |
| LessLag | 3, 3, 3 | 3, 3, 3 | 3 / 3 |
| LLUltra | 2, 2, 2 | 2, 3, 3 | 4 / 4, 3, 3 |
| LLDoubles | 4, 4, 4 | 4, 4, 5 | 2 / 2, 2, 1 |

All 24 critical windows had consecutive present identities (no exclusions, no
replacement trials); automatic and visually reviewed pose indices agreed in all
24 and `R = M + D` held in all 24. Present-at-flush did not change any captured
count versus the b19 and 0.5.0 runs; LLUltra still has one frame fewer than
LessLag in the normal phase (2 against 3, three of three) and is within one
frame of it in the delayed phase (2, 3, 3 against 3, 3, 3). Qualifications:
seven delayed-phase trials (Vanilla 1 and 2, LessLag 1, LLUltra 1, LLDoubles 1
to 3) are not covered by their API trace export (each export holds 1800
identities that start after those trials), so they rest on the marker's image
identities alone; on the 17 API-joined trials the delayed values are Vanilla 6,
LessLag 3, 3, LLUltra 3, 3 and no LLDoubles trial. Five traces carry "Engine
state changed during capture." and the LLUltra normal trace one skipped event;
not investigated, no API-level A/B claim. These are emulator captured-image
counts with synthetic input; they do not measure the console frame-time stall
that present-at-flush targets and do not establish Switch input-to-photon
latency. At the stop the unit recorded SIGBUS at the external
`systemctl --user stop` (after a clean `game.stop` and Eden's own clean
shutdown log), the known shutdown behaviour. Evidence outside Git:
`latency-052-1015/` (NOTES.md, per-trial analysis, 480 images, traces with
partial-copy verification, staging receipt); annotated GIFs
`docs/media/latency-{vanilla,lesslag,llultra}.gif` (normal-phase trial 1 of
each, frame counter from the first image after the injection to the crouch).

### 0.5.2 — 2026-10-15: present at the flush for LLUltra and LLDoubles (candidate for the console LLUltra drop)

Operator report: LLUltra "drops fps dramatically" on the console (scene not
yet specified, no capture yet). Code reading: `render_policy::present_at_flush`
was gated to LessLag, so LLUltra, the other two-image ring, still presented
each frame after the next frame's whole CPU update, exactly the mechanism
the 0.4.0 analysis traced on the console character select (update over
about 13.7 ms → missed vblank → the acquire waits for the following one →
33 ms frames). LLUltra's 576p lightens the GPU and cannot touch that.
Operator decision ("can we just apply the fix to LLUltra and Doubles too?"):
commit ce0145a extends the predicate to every committed, idle render-opts
profile (LessLag, Ultra, Doubles; Vanilla and every swap, drain, forced-wait
or refused state still refused) and the flush hook calls
`nvn::present_at_flush` whenever the policy allows instead of only on the
LessLag field path, so Ultra's three-call flush presents the acquired image
too. Doubles gains the earlier present with its third image as slack. Not
measured anywhere yet; the Ultra early flush and skipped late flush are
unchanged, so visual checks (anything recorded after the early present) are
part of the console test. Gates on ce0145a: host Clippy with warnings
denied, 293 host tests (the present-at-flush test now asserts Ultra and
Doubles), no-default-features build, `cargo fmt`, Skyline Clippy with
warnings denied, `tools/Build.ps1` release. Package
`artifacts/1525e3e0392c8403/`: build `0.5.2-ce0145a2155c-1792070522`, NRO
SHA-256 `1525e3e0392c840306073762cb7e78670745fe0e925de013f660c901371cfdff`
(1,470,464 B, module `6f2750f4…a860`). Card install over the first 0.5.2
build via `switch-mtp-052b/deploy-052b-mtp-1015.py`; `config.ini` on the
card is the LLUltra baseline (manual LLUltra, both DRS experiment flags off)
so the first console run measures this change alone. Acceptance: an LLUltra
Training capture with Input Display off, intervals and the texture-sync
waits compared against the 0.5.1 LessLag Training capture; `early_presents`
≈ `early_present_skips`, zero mismatches (code 105), no visual corruption.

### 0.5.2 — 2026-10-15: per-move and adaptive dynamic-resolution triggers (experiment, on the card)

Operator order after the Training analysis and the console A/B (Falcon Punch
"barely budged", Pyra's side special "dropped several frames, especially when
it connected"): "build the 0.5.2 experiment with both flags … make the
threshold where it triggers configurable in the ini as well". Commit 03bd237:
`src/drs_triggers.rs` (catalog of eleven named moves as fighter kind + status
sets from skyline-smash bed642e, `MoveSet` mask parsing, threshold parsing in
hundredths of a millisecond, the `Adaptive` over/under-budget state machine),
`src/perf_scaler.rs` (per-entry move slots and one adaptive slot on the same
report multiset as the cut-in and Gigaflare, `MainTick` input, counters),
`src/runtime/adaptive_drs.rs` (present-thread side: interval from the tracked
present ticks, 192 ticks per hundredth of a millisecond), config keys
`drs_moves`, `drs_adaptive`, `drs_adaptive_threshold_ms`,
`drs_adaptive_trigger_frames`, `drs_adaptive_release_frames` (all default
off/17.5/6/90, round-trip through the INI so menu saves keep them), event
code 108, `renderer-status.txt` settings and counters, Debug-view rows `DRS
moves` and `DRS adaptive`, compact-strip `Adp`. No smashline. Docs:
CONFIGURATION, DEBUGGING, RENDERING. Gates on 03bd237: host Clippy with
warnings denied, 293 host tests (eight new: catalog, mask, threshold, state
machine, move slots, adaptive slot, config round trip), no-default-features
build, `cargo fmt`, Skyline Clippy with warnings denied, `tools/Build.ps1`
release. Package `artifacts/9b9660ead3338d78/` in the tangent evidence: build
`0.5.2-03bd237d7d1c-1792068556`, NRO SHA-256
`9b9660ead3338d782dfcd27f03043efa1e55d351f47daf9d3d602ffc13239183`
(1,470,464 B, module `5a9e2e10…4e42`).

Card install, 2026-10-15 12:50Z: `switch-mtp-052/deploy-052-mtp-1015.py`
preflight and install over DBI USB MTP, `installed_verified` (the 0.5.0
release NRO archived on the card under `…/backups/pre-0.5.2-…/` and locally;
the other ten plugins and the protected files unchanged). The experiment
flags were then written to `config.ini` (previous copy saved as
`switch-mtp-052/config.ini.card-before`): `drs_moves=all`,
`drs_adaptive=true`, threshold 17.5 ms, 6 trigger frames, 90 release frames,
on the operator's manual LessLag. Not released, not tagged, no hardware
result yet. Acceptance per move or setting: a 30 s `CAPTURE_TIMING` on the
console with the trigger on and off, compared on intervals over 20 ms and on
the `Resolution` row; code 108 edges and the status counters show whether and
when each trigger fired.

### 0.5.1 — 2026-10-15: render-opts teardown guard, console test pending

Branch `post-scene-teardown-guard` (candidate `cc7f07f6…`, Eden evidence in
the 0.5.0 entry below: 22 Training → character-select exits without the
post-scene guest abort that hit the release five times in about 22 exits)
merged into main at 8aaef69 (`--no-ff`), version bumped to 0.5.1 at 231f4c8.
Change: `src/teardown_guard.rs` and `src/runtime/render_sequence.rs` skip
the relocated model update and the fighter worker on a frame in which a
fighter module unloaded (code 48 `RenderTeardownGuard`,
`teardown_guard_skips` in `renderer-status.txt`) and fall back to the game's
ordering for 180 scene frames; see `docs/DEBUGGING.md` and
`docs/PERFORMANCE.md`. Gates on 231f4c8: host Clippy with warnings denied,
284 host tests, no-default-features build, Skyline Clippy with warnings
denied, `tools/Build.ps1` release. Package
`artifacts/aee3852d905218e9/` in the tangent evidence: build
`0.5.1-231f4c82ec3a-1792064680`, NRO SHA-256
`aee3852d905218e922886d338e2540cd4dd379a25c23e5f4f32981b2a24642da`
(1,462,272 B, module `d60c4a2e…1fdf`).

Card install, 2026-10-15 11:45Z (operator: "test 0.5.1 before the
release"): `switch-mtp-051/deploy-051-mtp-1015.py` preflight and install
over DBI USB MTP, `installed_verified`; the 0.5.0 release NRO is archived on
the card under `ultimate/ssbu_online_deluxe_rewrite/backups/pre-0.5.1-
20261015T114532Z/` and locally; the other ten plugins, `config.ini` (manual
LessLag, Auto off) and the protected files are unchanged.

Hardware result, 2026-10-15 ~12:10Z (operator, verbatim: "0.5.1 passed"):
repeated Training → character-select exits and a LessLag match on this NRO
with the full plugin set, Skyline `9959ecb1…` (latest skyline-dev release),
no crash. This is the first console run of the teardown guard; the guard
counter and code-48 events on the card have not been read back yet. Released
as v0.5.1 (tag at 231f4c8, Forgejo release with the NRO and manifest).
Observed in the same session and NOT addressed by 0.5.1: in Training the
frame rate fell into the 50s and 40s during Kirby's charged hammer, while
the zoom (the cut-in that the scaler already covers) produced no drop. That
is a candidate for the 0.5.2 per-move dynamic-resolution experiment. Planned
next: the same build on upstream Online Deluxe's bundled Skyline `d3630815…`
so neither user group has to change Skyline.

Training frame-rate analysis from the card, 2026-10-15 ~12:30Z (operator:
"see why lesslag was in the 50s though without any effects"). The
`CAPTURE_MATCH` flag on the card recorded the Training session as
`timing-1794782036751.csv` / `api-1794782036751.csv` (label `match_training`,
build 0.5.1, 165 s, 8,609 frames; Kirby vs CPU Corrin per the modpack's
`nro_load` lines). Result: median render-callback interval 17.2 ms, mean
19.2 ms, 23.4 % of intervals over 20 ms, frame rate per 10 s between 45 and
59; the hang-trace heartbeats of the same boot show 47–59 presents/s in
Training. Reference: the previous Training capture on the card,
`timing-958174278187.csv` (build 0.5.0, 2026-10-14, Falco vs Ness, same
plugin set and capture flags): median 16.67 ms, p99 17.1 ms, 0.4 % over
20 ms, 60 fps; the previous boot's heartbeats (0.5.0, earlier the same day)
show Training at 60.0 presents/s. Decomposition from the API trace: kind 3 is
the infinite `nvnSyncWait` on the swapchain texture sync. On 0.5.0 it
returned 1 (waited, about 3 ms) on 99 % of frames, the signature of a frame
finishing early and idling to vblank. On 0.5.1 Training it returned 0
(texture already free, no wait) on 53 % of frames and waited about 6 ms on
the slow frames. Per-frame work excluding that wait: median 16.8 ms, p90
20.0 ms on 0.5.1 Training against 13.8 ms / 14.8 ms on the 0.5.0 capture,
and the 10 s medians swing between 14.6 and 18.6 ms with play. The game's
UI job (kind 77) rose only 7.0 → 7.5 ms and the worker dependency waits
(kind 66) are unchanged per frame. Reading: the scene's frame work exceeded
the 16.7 ms budget by about 3 ms and varied with activity; the two-image
LessLag ring has no slack, so every late frame becomes a missed vblank
(Vanilla's three-image ring would show the same work as 60 fps with one
more frame of latency). Not attributable to the 0.5.1 change from this data:
the guard fired twice in the session (`teardown_guard_skips=2`) and its only
per-frame effect is the 180-frame fallback after a transition (1,040 of
12,512 frames here). Differences from the 60 fps capture that the logs do
show: different fighters (Kirby and Corrin vs Falco and Ness) and the
Training Modpack menu with hitbox visualization, frame advantage, input
display, stage hazards and save-state autoload selected; whether those
toggles were the same on 2026-10-14 is not recorded. Discriminating tests
for the operator: the same Kirby Training scene (a) with hitbox
visualization off, (b) on Vanilla, (c) on the archived 0.5.0 NRO (one
restore command). Evidence copies: session scratchpad `card-051/`.

Resolved by the operator, 2026-10-15 ~12:55Z (verbatim: "It's not the
hitbox visualization it's the Input Display. When I switch it to off it
goes to 60 and stays locked there"): the Training Modpack's Input Display
overlay was the ~3 ms of extra frame work. With it off, LessLag Training
holds 60 on the console; the operator is taking the cost up with the
modpack's maintainers. Consequence for the fork/upstream comparison above:
the upstream run that "stayed locked" had the modpack removed from the card,
so it never carried that cost, and the earlier "ours in the 50s versus
upstream at 60" impression compared different plugin sets. Deluxe-side
lesson, unchanged: the two-image LessLag ring exposes any per-frame work
over 16.7 ms as a missed vblank; an adaptive resolution trigger (0.5.2
experiment) is the mitigation that does not depend on naming the cause.

### 0.5.0 — 2026-10-14: character FX split into `ssbu-fx-control`; plugin own pages

Operator: "project out the fx mods into their own repo … a plugin that users
can use to toggle effects on or off or reduced in its own window … an option
for each character as well as an 'all' option … make the menu a little
leaner for deluxe." Decisions: rows only for covered fighters plus All;
Reduced only where a variant exists; repo `intpa/ssbu-fx-control`; the menu
API stays in Deluxe (imgui-smash stays a renderer).

Deluxe side (commits f3fe436, f571a30, 40d2234, f9c294d, 0213533, b0ca667):
plugin API descriptor version 3 (`PLUGIN_FLAG_OWN_PAGE`, 32 options, export
`ssbu_online_deluxe_online_context_v1`, SDK 3.0.0 with v3→v2→v1 retries);
the scene tracker became the `SCENE` hook family (`DISABLE_SCENE`) instead of
installing from the FX family; timing CSV schema 2 without FX columns; then
the FX feature removed (14 source files, `host/eff-variant`, the family
generator, the FX policy JSON, five FX docs, the FX row, seven Debug rows,
FX event codes; 12,707 lines; code 30 renamed `OnlineContext`,
`auto_driver::context_online`); `effects` / `effects_scope` INI keys are
accepted and ignored. 280 host tests, 25 SDK + 2 ABI tests, 26 analyzer
tests, clippy host + Skyline, no-default build, Skyline release build. The NRO
is 1,462,272 B (0.4.2: 1,531,904 B).

Plugin side (`intpa/ssbu-fx-control` main ad1ac6c): verbatim import of the
FX sources from Deluxe f3fe436, then a per-fighter decision (move → fighter
type → compile-time name-prefix map), own INI with a 2 s quiet-period writer
thread, own events/install logs with Deluxe's code numbers, online scope from
Deluxe's export or the plugin's own latency + stage hooks when Deluxe is
absent (never both; Scope Online never applies when no signal exists). 61 host
tests + 18 EFF-variant tests, all gates. NRO `e5cf868a…` (581,632 B, build
`0.1.0-3e7d0ecb6828-1792010538`).

Eden combined run, 2026-10-14 20:55–21:20Z (GIF build, isolated profile, both
NROs staged, Astra low driver `fx-split-eden-driver-1014`, evidence
`fx-split-1014/`): Deluxe 0.5.0 `8b1c936f…` + FX Control `e5cf868a…`. The
window cycle ran Settings → **FX Control** (own page, titled with the plugin
label; `01-fx-page.png`) → Performance; the page listed Scope, All, the 14
fighters and `EFF variants: Ready`; Deluxe's Settings had no FX row
(`02-deluxe-settings.png`). FX Control's install log: `online_source=Host`
(Deluxe's export), config created with defaults, 22 hooks installed. Scope
set to All through the page and persisted (`fx-config-after-scope.ini`).
Training with Sephiroth, Gigaflare charged and released under On, Off and
Reduced (`gif-fx-sephiroth-gigaflare-{on,off,reduced}.gif`, 73 frames each,
0 dropped): On shows the large white/yellow/purple burst, Off has no burst
but keeps hit sparks, fire and the launch trail, Reduced keeps the purple
crescents and flames with less white washout; damage identical (49.3 %, 8
hits) in all three. Editing one fighter made the All row read `Unavailable`
(by design: mixed rows); All = Off then All = On restored every row
(`fx-config-final.ini`). 59.98–60.01 fps, no crash or dialog.

Released 2026-10-14 21:12Z: Forgejo release v0.5.0 (id 374) with NRO
`d9dd7df281671c12…` (1,462,272 B, build `0.5.0-44830e5d9255-1792012237`,
readback verified) and FX Control v0.1.0 (release id 375, NRO
`505dfecd68330201…`, 581,632 B, build `0.1.0-eb627b89e654-1792012419`, SDK
resolved from the Deluxe v0.5.0 tag). Both NROs are the Eden-tested code
rebuilt from the tagged commits; only the embedded build strings differ.

Card install 21:15Z over DBI USB MTP (`switch-mtp-050/deploy-050-mtp-1014.py`
preflight → install, readback verified): Deluxe `87322eef…` (0.4.2) replaced
by `d9dd7df2…`, `libssbu_fx_control.nro` added; rollback archive under
`ultimate/ssbu_online_deluxe_rewrite/backups/`; `config.ini` untouched
(manual LessLag, input_buffer 3, auto off, streamer off; its `effects` keys are
now ignored). Flags `CAPTURE_CSS` + `CAPTURE_MATCH` remain. Console run of
0.5.0 + FX Control not yet performed.

Eden GIF campaign for the FX Control README, 2026-10-14 21:27Z onward, on
the release NROs (Training, Scope All, 6 s clips per state): two Eden
instances died at the Training → character-select transition. Run 1
(22:49Z, after 48 clips): host SIGBUS with memory pinned at the 8 GB cgroup
cap, no guest panic logged. Run 2 (~23:30Z, after 66 clips): **guest abort**
— `SetTerminateResult 2001-0114` (kernel `InvalidHandle`) from
`nn::os::LockMutex` inside the game's model-update routine (`main+0x3ab5c4`)
called from `deluxe_rewrite_post_scene`, i.e. the LessLag render-opts path
that relocates the model update into the post-scene hook, while fighter NROs
were unloading. Evidence: evidence root `fx-gifs-1014/run-2/GUEST-PANIC.md`
and `guest-panic.txt`. Intermittent (the same transition succeeded many
times earlier that day); not reproduced on purpose; no fix made or tested.
The same render path is on the console in 0.5.0 (unchanged since
2026-10-06). Run 3 (14 GB cap, memory peak 8.19 GB) died 23:46Z with the
identical backtrace at the same transition after 75 clips, so host memory
pressure is not the trigger: three Eden deaths in about twelve Training →
character-select transitions this evening, all with FX Control 0.1.0, the
Training Modpack and the gamepad overlay staged next to Deluxe. Which of the
loaded plugins, or the GIF-build Eden, makes the race observable is not yet
established; a controlled Training → character-select cycling test with and
without FX Control is the next step before any mitigation. Later that night
the host-side Eden deaths (runs 1 and 4, and a latency run) were traced to
the private display service's 1 h runtime limit taking the X server down;
only runs 2 and 3 are guest aborts.

Input-delay re-measurement on the release NRO, 2026-10-15 00:35–01:45Z
(operator request after a PK Thunder recovery stayed at 60 fps under LessLag
on the console): same method and Eden build as the b19 table, 18 trials,
640×360 captures. Included trials: Vanilla R = 6, LessLag R = 3, LLUltra
R = 2 (normal) and 2–3 (delayed) — three and four fewer captured response
frames, unchanged from b19. Four trials excluded for one missing present
identity each; details in `docs/LATENCY_METHODOLOGY.md` ("Re-measurement on
0.5.0"). Emulator captured-image frames, not console input-to-photon.

Card experiment, 2026-10-15 01:53Z (operator order): the rewrite 0.5.0 and
FX Control NROs were removed from the card and the ORIGINAL fork
`libssbu_online_deluxe(nuevo).nro` (`21af1ac0…`, 1,441,792 B) with the
original `libimgui_smash.nro` (`61e19e1b…`, 1,261,568 B) installed over DBI
MTP, readback verified (`switch-mtp-fork-swap/deployment.json`); the three
replaced files are archived on the card under
`ultimate/ssbu_online_deluxe_rewrite/backups/fork-swap-20261015T015254Z/`
and locally in `switch-mtp-fork-swap/before/`; `restore` mode reverses it.
Purpose: compare the original fork's frame drops on PK Thunder recoveries
with the rewrite's.

Outcome, 2026-10-15 10:00–11:45Z (evidence root `switch-mtp-fork-deps/`,
`switch-mtp-fork-solo/`, `switch-mtp-upstream-143/`, crash reports in the
session scratchpad): the fork never reached a match on this console. Smash
first crashed at launch three times (Instruction Abort, PC 0, LR in the fork's
`main+0x838`) because the fork's dynamic imports `ssbu_pia_manager_*` were
unresolved; the swap had installed only the fork and its ImGui host. With
`libssbu_pia_manager.nro`, `libssbusync.nro` 1.3.0, `libnx_over.nro` and
`libsmashline_plugin.nro` added (the set that ran next to the fork until
2026-10-06) and the fork configured for offline LessLag, the game aborted at
the versus splash (User Break in game code, no plugin frame) the moment the
render options engaged; with Training Modpack, gamepad overlay, replay and
Elite removed it failed two boots (an init-time null identical to a
2026-10-13 boot fault seen with the rewrite installed, then a null fiber
stack in a fighter-script coroutine). The UPSTREAM mod,
saad-script/ssbu-online-deluxe v1.4.3 (release zip `5640c63a…`), installed as
its installer script would produce it (latest ARCropolis 4.1.1 = the card's
`92171b2f…`, nro_hook 0.4.0, imgui-smash 1.0.0, pia-interface 1.3.0,
smashline 1.6.8, upstream's own ssbusync 1.3.2, nx_over and Skyline
`d3630815…`; overclock sysmodule omitted, `overclocker = false`), crashed
once at boot while the card still had the older smashline `d472c596…` (null
in a task worker, no plugin frame) and then reached an offline LessLag match
after smashline was swapped to 1.6.8. The operator observed upstream holding
60 fps through special zooms by lowering the internal resolution; no
frame-time capture of that run exists. All swaps were restored in order
(11:41–11:42Z, each `restored_verified`): the card is back on the 0.5.0
plugin set, the rewrite's ImGui host `946de84f…` and Skyline `9959ecb1…`,
which is byte-identical to the latest skyline-dev release ("beta",
2026-04-12). For the record: the rewrite carries the same cut-in and
Gigaflare dynamic-resolution policy (`src/perf_scaler.rs`, "Dynamic
resolution" in `docs/RENDERING.md`), and the two Skyline builds are to be
tested with 0.5.1 so users need not change Skyline.

FX Control performance campaign in Eden, 2026-10-15 02:40–05:45Z (operator
request; results in the FX Control repository, `docs/FX_EMULATOR_RESULTS.md`
"Per-move frame drops under a 120 % CPU quota"): Eden's unit under
`CPUQuota=120%` / 5 ms period, Deluxe LessLag with the Performance view open,
27 moves × On/Off/Reduced, one 30 s `CAPTURE_TIMING` each. Deluxe-relevant
outcomes: (1) the post-scene guest abort recurred twice more on the release
NRO at Training exits (runs 5 and 6; five guest aborts in roughly 22 exits
over the night, all the same backtrace); (2) from 04:55Z the Eden profile
ran the teardown-guard candidate `cc7f07f6…` (branch
`post-scene-teardown-guard`, package `artifacts/6e74426068fed7a2/`): six
Training → character-select exits plus one boot, `teardown_guard_skips` 0 →
11 with a code-48 event each time, no abort and 60 fps between captures.
That is the first evidence for the guard; it is Eden-only, six exits, and
not a verified fix.

Cycling test on the candidate, 2026-10-15 05:50–06:20Z (evidence root
`fx-guard-test-1015/`, no CPU quota, same plugin stack): 16 Training →
character-select exits alternating Sephiroth, Pyra/Mythra, Inkling, Lucas,
Ganondorf, Palutena, Kazuya and Ness, with a main-menu round trip after
every fourth cycle. 16 of 16 exits reached the character select at 60 fps;
`teardown_guard_skips` and the code-48 count rose by exactly one across every
exit (11 → 43, the guard also fires on fighter loads); zero stalls or
aborts. Together with the campaign that is 22 exits without an abort on the
candidate against five aborts in about 22 exits on the release build the
same night, all in Eden. The candidate is not merged, not pushed, not on the
console; the operator decides.

### DBI USB MTP installation of f22d (0.3.0) — 2026-10-12 19:58Z

`switch-mtp-f22d/deploy-f22d-mtp-1012.py` preflight 19:55:47Z (installed
rewrite d773e7bf…2b17 as expected; all plugins and protected files backed
up locally) and install 19:58:08Z `installed_verified`: rollback archive
`/ultimate/ssbu_online_deluxe_rewrite/backups/pre-backoff-f22d94950be1b460.nro`
read back as d773; staged copy read back as f22d; target read back
f22d94950be1b4601ad9137a8d87acf6eccf2ec599eb48812e6f386f8e7403d8
(2465792 B, module cac8cc1b…ca31); other plugins, plugin names, config.ini
and protected files unchanged. Operator authorization recorded in
`operator-approval.json`. Hardware behaviour not yet observed: the Switch
test list is in the standalone repo's `docs/ANALYSIS.md` section 6.

### Pinned release v0.2.1 — 2026-10-12 (deferred persistence on PENDING)

Requested by the Training Modpack side: a `REQUEST_PENDING` answer no longer
queues the INI write. The host persists only when it observes `current`
equal to the pending choice (`Registry::reconcile_pending`, run once per
overlay draw even with the page hidden, and in `value`), or when a later
request on that option returns `REQUEST_APPLIED`; stepping a pending change
back to the live value cancels it and writes nothing. APPLIED behaviour is
unchanged. Three regressions cover pending→applied-by-plugin (one write),
pending→cancel (no write) and pending→applied-on-second-request (one write).
286 host tests plus the SDK round-trip; host and Skyline Clippy with warnings
denied; no-default build; rustfmt; Skyline release. Crate 0.2.1; ABI and SDK
crates unchanged at 2.0.0. Tag `v0.2.1` (commit aa8c33f) clean NRO: build
`0.2.1-aa8c33ff469b-1791823438`, SHA-256
`eb87feb1cbcbc2b2c4f4adbe15008d424def377fe2e1d854eaaf880c7ef69f88`,
2453504 bytes, module `8e8e9491d0eddd5891357113cf68ca83b8f5fe0f1a82f6717eedc875f455977f`,
ELF `d3e08612f7a6017c06d49e08bc7b35c0c938946a35f15bb333ce077d518d0b36`.
Not installed on hardware (the Switch has d773). Package: tangent evidence
`artifacts/eb87feb1cbcbc2b2/`. Forgejo release `v0.2.1` carries the clean NRO
and `manifest.json` as assets; release `v0.2.0` carries its NRO, manifest and
the demo plugin NRO `2f9fcdd730b14ab22fc6c19cfc0a6fe9f61d072379b1cb30a695e43b8f004b60`.
Every future tag ships its NRO the same way.

### Pinned release v0.2.0 — 2026-10-12

Tag `v0.2.0` (commit ae19c58, Forgejo `intpa/ssbu-online-deluxe-rewrite`) is
the consumer reference for the plugin API: `deluxe-plugin-abi` 2.0.0 and
`deluxe-plugin-sdk` 2.0.0. It adds pending-aware stepping on the Plugins page
(a second Left/Right takes a pending change back). Clean NRO from that tree:
build `0.2.0-ae19c58d9df6-1791821467`, SHA-256
`7585760b9bc4c63404ea7ff0a605de8afb642ea3db851f1c4ff2429add52ad78`,
2453504 bytes, module `829fa26224030349a7a84bfe48aab7a058e7c96c78cc2bedb855a5311409fb71`,
ELF `b695068c0d08924d6dc964712e13b9c089f10c1cafde8a2463d047ae7d3a9001`.
283 host tests plus the SDK round-trip; host and Skyline Clippy with warnings
denied; no-default build; rustfmt; Skyline release. Not installed on hardware
(the Switch has d773, which differs only by the pending-aware step and version
strings). Package: tangent evidence `artifacts/7585760b9bc4c634/`.

### DBI USB MTP installation of d773 — 2026-10-12 15:46Z

Operator authorized ("deploy d773 to the switch"); the live DBI MTP mount
implies Smash was closed. Preflight confirmed the installed rewrite was 9d08
and backed up the nine plugins and six protected files. The transaction wrote
`ultimate/ssbu_online_deluxe_rewrite/backups/pre-chord-tones-d773e7bfc5764ab7.nro`
(9d08, readback-verified), staged and copy-replaced the NRO and read every
file back: the rewrite is now
`d773e7bfc5764ab795a1d35839adc2a3595b3d6fd45e815b44cd84567b8b2b17`
(2453504 bytes, build 0.1.0-0922d28fd852-dirty-1791818952, module
`df24e776b86a09a2f342b68e17fc26ea2d9238cd1e68b392ba776e123a2bf1e6`); the INI
(`0617a94705ea8d2da3fff782161b5339f6a3880948a807894a01692bccd9c2d3`, the
operator's latest), eight other plugins, plugin names and protected files are
unchanged. Demo plugin not deployed. Evidence:
`~/.local/share/ssbu-mod-backups/plugin-api-tangent-20261012/switch-mtp-d773/`.
This accepts the transfer; the chord fix and tones are not yet observed on hardware.

### DBI USB MTP installation of 9d08 — 2026-10-12 14:00Z

The operator authorized the install with Smash closed and DBI's USB MTP
responder ready. The single-writer transaction (adapted from the 6c39 script)
backed up all nine plugins and six protected files, confirmed the installed
rewrite was still 6c39 immediately before writing, wrote the rollback archive
`ultimate/ssbu_online_deluxe_rewrite/backups/pre-plugin-api-9d08a711b271e0ac.nro`
with readback, staged the candidate with readback, copy-replaced the NRO and
read every file back: the rewrite is
`9d08a711b271e0acbb0b2be426c6892acf32eefe85e17deaccdd5050547ec40c`
(2449408 bytes, module `5baf03f205f8fb39ce81cffa31459681fa21e0c7dc7b062aadd503eb1aad5f3f`);
the INI (`c1272813e40bd694c7429c8103199141e8017341996f754309e54b2dbaa4158d`),
eight other plugins, plugin names and protected files are unchanged. All
device command processes exited. Only the rewrite NRO was written; the demo
plugin was not deployed, so on hardware the chord cycle stays
settings → performance → hidden until a plugin NRO registers.
Evidence: `~/.local/share/ssbu-mod-backups/plugin-api-tangent-20261012/switch-mtp/`
(`preflight.json`, `deployment.json`, `operator-approval.json`, before/after readbacks).
This accepts the transfer, not runtime behaviour on hardware.
