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

[Thirty measured trials plus six warmups](FX_EMULATOR_RESULTS.md) completed across
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
  [Full method, results and limits](FX_EMULATOR_RESULTS.md).
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

Caveat for the CSS capture: the SD writes now happen during the measured
interval, from the diagnostics thread. Compare the next hardware CSS capture
against `switch-0.3.1-1013` before trusting a movement-penalty number from it.

This is a footprint change, not a
proven fix: the next console boot with 0.3.3 and the companion plugin rolled
back decides; the memory bisect is plugin removal, not Deluxe flags.

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
