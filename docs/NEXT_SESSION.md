2026-10-14: this file describes the October 9–10 live-session state (bundles 8328cc88/4c19). The current release is 0.4.2; see docs/STATUS.md and docs/VALIDATION.md.

# Resume and test plan

Read [STATUS.md](STATUS.md), [VALIDATION.md](VALIDATION.md) and the latest entries
of docs/VALIDATION.md and `git log` before touching a live process. At the time
of writing a root handoff file owned the current sockets, deadlines, worktrees
and staging journals; it no longer exists in the repository. Older
run names in scripts are not live-session discovery.

## Current acceptance work

The latest local bundle is `8328cc88123bd432` under
`~/.local/share/ssbu-mod-backups/hud-content-fit-20261010/artifacts/`.
The operator liked f1ce's two-row layout and requested a narrower background.
8328 fits the box to content, retaining the left edge/text/bar/font/opacity.
All265hosttests and host/Skyline/package gates pass. Both actual1080p in-match
views are captured and parent-reviewed in that evidence root's screenshots/:
`hud-content-fit-1010a-settings-inmatch-settled.png` and
`hud-content-fit-1010a-performance-inmatch.png`. Use file:/// Markdown targets;
bare absolute-path links caused the earlier portal error. Run/worker resources
are closed, the fresh profile journal is RESTORED, and original bytes rehash
against backups. Read parent-closeout.json and the operator's evidence-root handoff
(outside the repository) for final primary
watcher closure. The operator subsequently approved these images;8328 is now installed
over DBI FTP, with final SHA readback at14:09Z on October10. At that time the newest
handoff and hud-content-fit-20261010/switch-ftp/deployment.json (installed_verified)
were the sources.
Latest INI/other plugins/HOC/boot files preserved; old4c19 locally/on-SD backed up.
FTP is closed and the operator may play. switch-ftp/SWITCH_CHECKLIST.md has the
ordered HUD/coexistence, export and peer checks. Wait for results; require fresh
Smash-closed/FTP-ready confirmation before later evidence collection. Do not restart
a closed benchmark or reuse an old staging journal.

Previous installed Switch baseline `4c1947b84add6114` is under
`~/.local/share/ssbu-mod-backups/menu-peer-fix-20261009/artifacts/`. It includes
buffered export62a and the new menu/peer-retirement changes, with261hosttests and
all host/Skyline/package gates passed. Coordinate a visual check of horizontal
Settings/transparency/content-fit height and a confirmed disconnect/no-rematch
then reconnect; preserve other connected lobby peers. The exact reported runtime
path remains unverified. The operator subsequently requested DBI USB MTP installation
with Smash closed:4c19 was installed and SHA-readback verified October9 at22:32Z.
The current INI is LessLag/input3/FX Off/all/Stealth Off; preserve its latest bytes.
Previous5c72 is backed up locally and on SD; eight other plugins and HOC/boot files
were preserved. Host MTP is released; no gameplay run has been claimed. See the
menu-peer-fix-20261009/switch-mtp receipts and the evidence-root handoff. Obtain fresh
Smash-closed/transfer-ready confirmation before further device access.

1. The isolated four-idle-Game-&-Watch absent/enabled/absent check is complete
   on Intel/ANV; see [the recorded results](GNW_FOUR_FIGHTER_CHECK.md). It showed
   smaller observed dips with the plugin, not a reproduction of the RTX slowdown
   or proof that every frame meets its deadline.
2. Right-side placement in Training and connected CSS/stage selection, left-side
   placement in ordinary matches, and removal of the opponent column after
   disconnect passed the recorded Eden checks. Console acceptance is pending.
3. Finish the remaining Reduced branches in the [runtime matrix](https://git.syntaxbyexample.com/intpa/ssbu-fx-control/src/branch/main/docs/FX_RUNTIME_MATRIX.md):
   Inkling forward-smash trace2 and the unobserved minor branches. Grounded
   triggered Sephiroth counter On/Reduced/On passed on c0; aerial/opposite-facing
   variants were not separately captured. Small-Flare,
   up/down-smash and full/dry-ink down-smash visual/request checks passed;
   roller WALK/AIR mappings were observed, with airborne-scene limits recorded. Verify
   On/Reduced/On restoration, protected cues and cleanup. Do not label an
   unchanged fallback as implemented reduction.
4. Run matched On/Off/Reduced comparisons under both LessLag and LLUltra using
   [the FX performance method](https://git.syntaxbyexample.com/intpa/ssbu-fx-control/src/branch/main/docs/FX_PERFORMANCE_METHOD.md). The goal is Off-like
   spike stability with recognizable cues. A thinner visual is not acceptance.
5. Validate the compact renderer correction on Switch after the final Eden
   package is accepted. Frame ownership and per-submission GPU storage replace
   the obsolete GPU-completion probe; the probe is no longer compiled. Use the
   exact final bundle and matching patched standalone host if Elite is installed.
   The intermittent baseline limits remain in [ImGui dependencies](IMGUI_DEPENDENCY.md).
6. Further network/three-client tests were waived by the operator on October 8.
   Preserve the existing single/doubles evidence and its artifact limits, but do
   not hold the next Switch candidate for a three-station match. A two-peer run
   may still be used specifically to validate the FX scope transition.
   On October 9 the operator additionally confirmed multiple people joining an
   Eden lobby had been tested by the Training Modpack agent; see the attributed
   coverage in VALIDATION.md. The newly reported disconnect/declined-rematch
   stale-opponent case still needs its own regression/runtime acceptance.
7. The peer-report follow-up completed 54 Palutena captures on c0: 18 warmups
   and 36 measured hit/whiff trials across Vanilla/LessLag/LLUltra and all FX
   modes. No consistent On-specific cost or Reduced benefit was established
   on this Intel/ANV host. Fable independently checked the hit traces; no
   hitbox/CPU attribution is justified. See [the measured results](https://git.syntaxbyexample.com/intpa/ssbu-fx-control/src/branch/main/docs/FX_EMULATOR_RESULTS.md).
   The peer clarified that the remaining report concerns 400–450 FPS mods.
   That optional reproduction is deferred until after operator Switch tests.
8. Reconcile docs, preserve optimized symbols and source, then publish accepted
   work to the authoritative Forgejo repository when authorized and update
   the release. As of 2026-10-14 `origin` is Forgejo
   (`git.syntaxbyexample.com/intpa/ssbu-online-deluxe-rewrite`); `github` is a
   second, downstream remote that is not pushed automatically. Do not assume
   release metadata is included in Git sync. Ship only
   the reviewed rewrite NRO after
   confirming Smash is closed; read and preserve the Switch's latest INI.

The previous Switch baseline was `5c72a0b54b8c8c95`, installed October 9 with
verified FTP readback. It retains standalone ImGui host `9a9e575c910c6471` and
the latest INI, both SHA-unchanged, and adds validated existing-stub chaining.
All 253 Rust host tests, warning-free host/Skyline Clippy, no-default build and
optimized packaging pass. The renderer is unchanged; embedded and standalone
Eden evidence in VALIDATION.md belongs to the previous `7fdad40f10705afb` build.
The operator initially reported successful close/reopen, then another crash.
Available diagnostics are preserved. The operator clarified vanilla-to-modded
relaunch failure and requested a repeat; before/after evidence is captured. The
repeat also had a bypassed failure. Operator transcribed "Unable to start software.
Return to the HOME Menu and try again". Both launches passed with injection paused
in comparison02; aftermath captured and owned flag restored/verified. Injection
is enabled again. Both rechallenge03 launches passed; aftermath and trace files
are collected. Recording finished, but API CSV export is incomplete. Source audit
found per-fragment native flush requests. Buffered-export candidate62a42df8d01d1738
has passed257hosttests, host/Skyline Clippy, no-default/release builds and exact
packaging. Its hardware export check remains pending; operator is free to play
while the independent crash/SaltyNX investigations continue. Coordinate fresh
Smash-closed/FTP-ready confirmation before transferring the local candidate.
Do not ask for another uninstrumented wait or shortcut attempt. Those comparisons
used5c72 before the DBI update recorded above.
Aftermath04 also contains two new SDK UserBreak reports from bypassed launches
after core injection. The original exception remains unidentified. The comparison
has not isolated the intermittent cause. All FTP connections are closed; obtain
fresh readiness before a later transfer and preserve the newer Vanilla/input3 INI.
Diagnostic-only candidates are not deployment targets.
The EFF transformer is unchanged from its earlier 22-test/five-stock-asset
acceptance. Consult the operator's evidence-root handoff for exact artifacts and
current process ownership.

The blue Special Zoom check is complete without a reproducible lag spike on
this machine. Preserve its appearance; the diagnostic request probe did not
observe the visible blue background and is not an accepted suppression path.
Keep `PROBE_CRITICAL_SCREEN` absent.

## Bounded emulator operation

Use the current CLI handoff:

```text
/home/intpa/eden-cli-handoff/updates/20261007-cli-controls/
/home/intpa/eden-cli-handoff/updates/20261007-cli-controls/source/docs/user/CLIControl.md
```

Use explicit isolated profiles, finite runtime and memory limits, and the exact
socket in the launch receipt. The user authorized two-hour sessions; set
`-Runtime 7200` (external launcher parameter) explicitly so the launcher's short default does not
interrupt a test series. Retain memory/no-swap caps and hang watchdogs.
Background mode is supported. Skip the intro.
Keep one emulator and no concurrent heavy builds for performance comparisons.
Do not restore a staging journal until its frontend has exited. Stop owned
supervisors, preserve raw evidence, and restore staged configuration/plugins
after each test.

FPS++ was measured (enabling the native-window composition-speed path
accelerated the match timer) and removed on 2026-10-17; there is no marker or
key to set. See [FPS_BOOST_INVESTIGATION.md](FPS_BOOST_INVESTIGATION.md) for
the history. No hardware overclocking.

## Switch boundary

The user reported that the previous released build worked after the G&W startup
fix, and earlier corrected-overlay Kirby/Palutena matches returned to CSS.
There is no special requirement for five matches.

The Switch received 5c72a0b54b8c8c95 and retained the matching ImGui host on October 9
with verified FTP readback. New right-placement,
peer-visibility and Reduced changes need their own console acceptance. The user
removed the G&W plugin; do not silently reinstall it with a rewrite update.
The October 9 reopen investigation includes distinct signatures: earlier
no-mod APM startup aborts (`2010-0011`, four stock modules), and the newer
`01791552627_01006a800016e000.log` Data Abort (`2168-0002`) with Skyline/rewrite
loaded and matching logs ending at unload-hook installation. The primary's exact
Skyline binary/stack reconstruction proves a zero-extended negative literal
offset, matching the faulting read address. Previous same-build hooks completed;
an existing absolute-jump stub is the likely address-sensitive trigger. Read
the durable hook-crash report linked from the evidence-root handoff. The narrow validated-stub
chain path and synchronous pre-hook capture are installed. Initial operator
success was followed by another crash. Read-only diagnostics are preserved in
`hook-chain-retry-20261009/FINDINGS.md` under the local backup root. Rewrite logs
match earlier successful hook installations, not the latest two Smash-title
processes in SaltyNX. Operator clarified PID144's successful L-bypassed vanilla
gameplay followed by normal modded relaunch failure (PID146); DBI via Album rules
out title takeover. The timeout is before bootstrap, with SVC result unchecked;
underlying cause/error screen remains unknown. User chose an unchanged-config
repeat; both `reproduce-01-baseline` and `reproduce-01-after` are now collected.
The operator retried after a bypassed failure, then had successful play and normal
relaunch failure. New PID149/153 thread-list timeouts surround injected PID151;
rewrite logs remain byte-identical, no new crash/fatal/ERPT names. Exact HOME-menu
"Unable to start software" text is supplied. Comparison02 passed with injection
paused and fresh rewrite startup; capture and restoration are complete. Injection
is enabled again;03 and04 aftermath collections are complete. FPS graph/shared
NVN audit completed (worker stopped): graph drawing follows startup, shared hooks
and an acquire-return ABI question require actual-chain verification, not speculative
changes for this early launch error. See the evidence-root handoff and the durable audit. Do not
rerun/overwrite snapshots01–04 or repeat the completed restore.
New after04 reports01791567103/PID166 and01791567116/PID169 show SDK UserBreak,
result0, UserExceptionHandler+0x2a8 and only four stock modules. SaltyNX core/NX-FPS
injection completed for both; rewrite logs are unchanged and no new timeout was
logged. Three failures were reported, but only two new crash reports were found;
do not invent a one-to-one mapping. No new fatal/ERPT names supplied the original
exception. These reports do not prove the earlier APM or relocation cause recurred.
Exact SDK analysis now proves the zero result is written by the default user-
exception callback before a secondary abort. Original exception data lies outside
the retained dumps; a future instrumented0x340-byte UserExceptionInfo capture is
the missing evidence, rather than another uninstrumented replay. See the durable
BYPASS_CRASH_AUDIT.md linked from the evidence-root handoff.
The separate export-fix worker's ownership was recorded in that handoff; older stopped
audit wrappers are not live work. Preserve all distinct failure evidence.

Kirby charged hammer connecting with Pac-Man on Battlefield also dips during
Special Zoom with game mods bypassed. SaltyNX remains active; the reported~50FPS
without OC/~40-something with OC lacks matched clocks/windows. This does not
establish a rewrite-specific issue, universal stock behavior or OC causality.
Keep Special Zoom/camera/hitlag intact. No new replay is currently requested.
`nst:oc` and background `sys-ftpd` are removed; HOC 2.5.1 remains by operator
request. Use foreground DBI FTP at the confirmed endpoint, not the old USB mount.
Official SaltyNX 2.1.2 is installed with all 15 files readback verified and HOC
hashes unchanged. The operator confirmed SaltyNX is working and the duplicate
listing is resolved. The old 1.8.1 module backup is outside the contents directory.
See the evidence-root handoff for the transaction receipt, rollback and process owners.

NSO arena display/filtering, actual VPN/high-RTT play and the first-generation
Switch's effect-specific dips remain distinct checks. Local LDN and emulator
success do not prove NSO compatibility or console performance.

Keep the test scope override:

```ini
effects_scope=all
```

The distributed default remains online-only. Preserve the user's other saved
settings when updating. Elite Legal Stages restricts NSO, not offline play.

### Optional after Switch testing — high-FPS Palutena follow-up (2026-10-08)
The operator's friend clarified that his emulator observation was at 400–450 FPS
with high-FPS mods, with reported dips of 10–20 FPS; he separately reported a few
FPS lost on Switch. This is not a reproduced result and the exact emulator,
hardware, patch, and settings remain unspecified. Existing capped 60 FPS results
do not establish the absence of a cost at that operating point.
Only after the operator's Switch tests, optionally compare warmed uncapped
On/Off/Reduced captures with the exact high-FPS mod and record frame times and
game-speed semantics. This is not a release gate and must not delay the new NRO.


Follow-up supplied by operator (2026-10-08, deferred):
- Bundle link: https://www.mediafire.com/file/mgwv1166wgyqscb/Pre+made+ready+eden+smash+build+version+13.0.4+wifi-safe.zip/file
- Peer says it includes reduced-effect mods, a latency slider with 400/450 FPS
  options, and the old delay mod.
- Peer workflow: enter Local Wireless, create a room, select 400 or 450 FPS,
  leave for local play; peer says that choice then applies elsewhere.
- Operator previously reproduced an Arsene issue by uncapping Eden, so reaching
  400 FPS may not be necessary to expose a measurable difference.
- Download via browser/Playwright if needed, inspect contents and hash artifacts
  before use in an isolated profile. No download or inspection has occurred yet.
  The archive filename says 13.0.4; our current game is 13.0.5. Verify actual
  components and their supported game versions before interpreting results.
  The filename's “wifi-safe” label is the distributor's claim, not acceptance.
This remains optional work AFTER the current menu/FX fixes and operator Switch
tests, as expressly requested. It is not an additional release gate.
