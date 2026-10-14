# Switch rollback and crash investigation, 2026-10-05

The original Skyline and ARCropolis 4.0.9 files were restored, hash-verified and
flushed to the Switch SD. Only hash-matched files introduced by the update were
removed. No rewrite build was deployed to the Switch. The SD has not been accessed
since the rollback; the operator can play using the restored setup.

Local evidence and rollback receipt:

    /home/intpa/.local/share/ssbu-mod-backups/switch-rollback-20261005-174501/

Original Skyline SHA-256:

    d3630815ee02f2502d072e382456ed043a4cfa6e642589bc313324cb852bbee1

Original ARCropolis SHA-256:

    8d18b5d85d7338df97a8a285049a3da213a7437c9755a5fd1db199ca3a4d968d

## The reported in-match crash

The October 5 report is a Data Abort, 2168-0002, on MainThread. PC is
main+0x3cedb4, LR main+0x15f6fe8 and the fault address is 0x18. The saved image
shows Kirby versus Wario on Duck Hunt, online, with the original Deluxe LessLag
profile. The operator reports a freeze followed by a crash shortly after play
started and similar failures on previous Skyline/ARCropolis update attempts.

An Opus 5.5/xhigh audit and a read-only 13.0.5 code probe from Eden establish the
immediate mechanism: a battle-object component update writes a 0x70-byte list
element through a null backing-buffer pointer. The registers and faulting store
agree. The function does not allocate the buffer itself. Why that buffer was null
is unresolved: allocation, initialization and lifetime are still alternatives.
Calling it a specific camera/effect component would be premature.

No plugin appears in the report's top 32 stack frames, and no resolved plugin hook
sits in the immediate crashing functions. This does not rule out an earlier
plugin-induced state change or memory corruption. The rewrite, Training Modpack
and SGPO were absent from the saved Switch plugin set.

The crash used the updated Skyline module (EAA2C23A...) and ARCropolis 4.1.1
(matched by embedded name/text size; its crash report build ID is zero). The
operator's repeated update association warrants a version bisection; it does not
identify which component caused the null buffer.

## Verified changes worth isolating

The updated Skyline binary matches the source behavior at
[2eb226f](https://github.com/skyline-dev/skyline/blob/2eb226fa9e4ab023dc00cb0db10ad4ec50ac4fa3/source/main.cpp).
It initializes a shared socket pool, then installs process-wide replacements for
both socket Initialize variants and Finalize which return success without calling
the original. The old binary initialized sockets too, but did not install these
replacements. The logger also changed. The normalized hook-engine and plugin-loader
code did not change. Socket lifecycle is a plausible online-specific variable,
but there is no demonstrated link to the null battle-object buffer.

ARCropolis changed resource/buffer handling in 4.0.10 and added native menu/scene
hooks in 4.1.0. The newer binary also uses SSL/network APIs for its boot updater.
[Release notes](https://github.com/Raytwo/ARCropolis/releases) describe the resource
and menu changes. At crash time its worker was sleeping; that neither implicates
nor exonerates resource changes earlier in loading.

## Separate older startup failures

Four October 4 reports match the saved Deluxe binary's API-version assertion in
smash-ultelier sync-guest install: the guest expected a different SsbuSync API from
the loaded host. These happened during startup on the old Skyline/ARCropolis pair.
The first October 4 report has the same general Rust-abort signature but a different
Deluxe build, so the exact assertion is not established for that report. The saved
October 5 host/guest API versions agree and the game reached a match. Those older
startup failures should not be conflated with the October 5 null-buffer crash.

## Controlled next tests

The console can remain on the restored pair. When testing resumes, change one
component at a time: original Skyline with ARCropolis 4.1.1, then updated Skyline
with ARCropolis 4.0.9. Keep the same fighters, stage, plugins and assets, and test
offline and online separately. Preserve the crash report and image for each
failure. Test the ARC boot-updater setting separately if that branch is implicated.
A Skyline build with only the socket replacements removed would be a further
diagnostic, not a proven fix.

Eden cannot reproduce Nintendo Switch Online conditions. A local reproduction
also requires the same assets, not merely the same NROs. The saved 16 crash-time
NROs and Skyline beta were tried in Eden with its existing assets and 4 GB memory
layout. The full stack stalled at Launching waiting for the unregistered nst:oc
overclock service. Removing only the peer's master-charge FX plugin did not help.
Restoring that plugin and excluding libnx_over allowed startup to continue after
dismissing the missing-overclock-service warning. The public fork checks for an
emulator using two fixed text-base addresses; this Eden run used another address
and its log explicitly reported is_emulator=false.

A temporary config with overclocker=false skipped that service and booted cleanly.
The warning names overclocker_enabled, but overclocker is the actual deserialized
field. The original fork was then tested twice with Kirby versus CPU Wario and
the same existing 1v1 (EU) rules: offline LessLag, then offline Vanilla. Both
reached character select and stalled on the pre-match loading splash. Both logs
show BufferState::Free assertions and "queue size is 3, waiting" in Eden's buffer
queue. The configuration confirmed each requested offline preset, but that did
not establish the active match preset: the later custom-Eden test below exposes
a forced Quickplay classification that can override the offline setting. Those
two earlier runs are not an accepted Vanilla-versus-LessLag comparison. Neither
reproduced the console's Data Abort.

The test logs, configurations and plugin hashes are preserved locally at:

    /home/intpa/.local/share/ssbu-mod-backups/eden-switch-stack-20261005-183600/

The original seven-plugin rewrite/TM/SGPO setup was restored and all seven hashes
verified. The temporary original-Deluxe config was removed from Eden. The operator
is providing a custom Eden build with CLI control and screenshots for subsequent
testing; preserve its version and instructions before using these results as a
comparison baseline. No further Switch SD access occurred.

No source fix is currently established for the reported console crash.

## Custom Eden isolation, 2026-10-06

A separate profile copied the working Eden NAND, SD, keys and configuration,
with independent save and SD paths. Fifteen crash-time plugins, updated Skyline
and ARCropolis4.1.1 were installed from local backups; only libnx_over was
excluded because Eden lacks nst:oc. The main rewrite/TM/SGPO profile and Switch
SD were untouched. The profile receipt is under eden-cli-handoff at
switch-profile-receipt-20261006-0306.json.

The first run set only offline singles to Vanilla. At stage setup the log said
UNTRACKED ONLINE MATCH DETECTED, ASSUMING QUICKPLAY, auto-selected LessLag and
began a Double/OneBehind transition. About16 ms later Eden emitted two
BufferState::Free assertions and stopped producing frames during loading.
Stage95 was automatically selected; no in-match scene was reached.

This has a direct source explanation: reference/quickplay/src/net/mod.rs
update_match_status turns every Offline Singles/Doubles state into Quickplay.
The hook is installed at stage presetup. match_init selects the online preset;
the registered ImGui callback's maybe_reapply_match_profile can apply the new
settings during rendering. It is not sufficient to set the offline preset for
a Vanilla control.

The corrected test set menu and both offline/online singles/doubles to Vanilla.
Its runtime log confirmed AUTO SELECT RP Vanilla. Kirby versus CPU9 Wario reached
Hollow Bastion (stage361), finished a65-second match and reached results without
the buffer assertion. Updated Skyline/ARCropolis and the fifteen plugins were
unchanged. Stage differed from the loading-stall run and the console report, so
this is a valid match-entry/completion control, not proof that the reported
Switch scene is stable.

Eden's assertion occurs in WaitForFreeSlotThenRelock: slots at or beyond the
current maximum buffer count must be Free. A reduction while an excluded slot
is still queued/acquired violates that check. This explains what the assertion
tests, not yet why the original host's transition failed to drain every owner.
The original host is closed source; neither disabling an assertion nor setting
Vanilla is a root-cause fix. As of 2026-10-06, the rewrite's measured modes
select the active texture count during window initialization and do not switch
it live (live guarded texture-count transitions exist now; see
[PERFORMANCE.md](PERFORMANCE.md)).

Saved logs, configurations and screenshots:
    /home/intpa/.local/share/ssbu-mod-backups/custom-eden-switch-20261006-0316/

The original console null-buffer crash remains unproven in Eden. No fix for it
is claimed from this separate buffer-queue failure.

### Menu-time transition and version matrix

Setting all scene profiles to LessLag moves the transition to the common-NRO
load before the main menu. This still stalled with the updated Skyline and
ARCropolis 4.1.1, including two BufferState::Free assertions. The host initializes
Vanilla first and then applies the menu profile; this is still a live transition,
not initial two-buffer window creation.

The saved old Skyline/ARCropolis 4.0.9 pair also stalled before the menu. The mixed
updated Skyline/old ARCropolis 4.0.9 pair likewise stalled; its log confirms the
Double/OneBehind transition and queue-drained/flush-done messages. Neither of
these two older-ARC runs contains the BufferState::Free assertion, so they
establish the same visible symptom, not the same proven internal failure.
Both finite watchdogs stopped the stalled game without a host OOM. This makes
the updated ARCropolis unnecessary for the observed emulator loading stall,
but does not explain the original console's in-match null-buffer Data Abort.

All three logs and configuration/hash receipts are retained in the evidence
directory above. No original binaries were patched and no files on the Switch
SD were accessed during these tests.

### Backend-only control

With updated Skyline/ARCropolis unchanged, replacing only the fork's v1.4.1
backend with canonical v1.4.2 allowed the all-LessLag menu transition to finish.
Two transient BufferState::Free assertions still appeared, yet index_manager
reported done and frame production continued. The assertion alone therefore
does not establish the cause of the persistent stall.

Kirby versus Wario on Small Battlefield (stage347) entered with logged LessLag,
ran beyond 81 seconds and completed to results. Thirty seconds of scripted
jump/attack/special inputs ran during gameplay. CLI shutdown exited normally,
without OOM. Saved host142 evidence includes the original configuration, all
plugin hashes, screenshots, input/status samples and log.

This is a diagnostic canonical-host substitution: its API worked with the fork
guest, but it does not carry the fork backend's two restriction bypasses. The
persistent-hook/per-frame render snapshot design already used by the rewrite
matches the later backend architecture. The exact Switch null-buffer failure
is still not reproduced or proven fixed by this emulator result.

### Private LDN and separate startup failure

Two independent custom-Eden profiles joined a password-protected, unlisted
two-member room through localhost. Each process had an 8 GiB memory cap, zero
swap allowance and a finite runtime. The initial in-game join timed out because
the host challenger prompt had not been accepted within the game's deadline.
Station recognition and modded-peer exchange were already present. Accepting
that prompt promptly allowed the connection; duplicate station warnings were
not evidence of the cause. Distinct game accounts were used for the successful
test, but account identity was not isolated as a cause of the earlier timeout.

Creating the second account also generated fresh ARCropolis per-user settings.
That startup failed before the title screen with an Eden SSL assertion, an
error applet and eventual host SIGSEGV. ARCropolis 4.1.1 creates the updater flag
in a new default configuration. Matching the new account's ARC configuration
to the known-working account, including disabled updating, the same preset and
legacy discovery, allowed a fresh frontend to boot. Because multiple settings
changed, this is not a single-setting proof or the console Data Abort.

Both peers entered Kirby versus Wario on Hollow Bastion (stage361). Runtime logs
confirm render_opts=true, Double/OneBehind and a completed transition at match
entry. Screenshots show more than 45 seconds of network gameplay, and a
30-second scripted Kirby input sequence completed. No console-style null-buffer
failure occurred. Both supervisors later stopped at their planned 900-second
limits with no OOM; the shutdown connection resets are not game-crash evidence.

Evidence is preserved in the same directory, under ldn-same-user,
ldn-new-user-updater and ldn-distinct-user-lesslag. The latter includes the
independent join audit. This remains a canonical-v1.4.2 backend diagnostic,
not a test of the unchanged original fork backend or Nintendo Switch Online.
### Matching stage and bike/hammer interaction

A further two-peer run selected Duck Hunt's Battlefield form on both stage-vote
screens. Runtime stage283 was consistent with the visible platform layout in the console crash image;
stage311 immediately before it was the separate waiting scene. Kirby versus
Wario ran for at least 223 seconds, with two photographed bike/hammer encounters
near the right ledge. The first bike hit Kirby for 6.1%; the second encounter
continued through a stock loss and respawn. Both peers continued producing
frames. Logs establish render_opts=true, Double/OneBehind and transition
completion at match entry. The console Data Abort did not occur.

Both games were stopped deliberately through the CLI. The host exited normally;
the guest had a room-lost Qt notice after the host closed, which was dismissed
before its normal exit. Neither process had an OOM. Evidence, config/plugin
hashes, command receipts, screenshots and both logs are archived under
ldn-duck-hunt-bike-hammer. This comparison still uses the canonical v1.4.2
backend; it does not prove the fork's original v1.4.1 backend is stable.
The crash screenshot establishes a bike collision with Kirby; it does not
establish a hammer item or Kirby hammer attack. Hammer charges were additional
test inputs, not a verified reconstruction of the preceding console inputs.
Battlefield form is an inference from the visible floating platform, not a
stage ID recovered from the console report.

### Original fork backend over LDN

Restoring the fork's original v1.4.1 host on both isolated peers, with Vanilla
menu/offline presets and LessLag online presets, allowed room entry and stage
voting. Both selected Duck Hunt Battlefield and Kirby/Wario. Both then stalled
during the transition into stage283, after the waiting scene. Logs contain
Double/OneBehind transitions and BufferState::Free assertions; neither reached
gameplay. Both watchdogs recorded over 120 seconds without a frame and stopped
the processes without OOM. This reproduces the loading failure over LDN, not
the Switch's later main+0x3cedb4 Data Abort. Evidence: ldn-original141-stall.

### Null-buffer investigation

Review of the captured 13.0.5 instruction window confirms that the faulting
function reads a container at component+0x18, clears its count at +0x10, and
writes 0x70-byte elements through the pointer at container+0x18. The crash
registers show that data pointer is null on the first element. This identifies
the failing write, not the allocation/freeing site or a culprit plugin.

The independent audit's symbol-string table is not accepted as proof of hook
registration, and searching binary data for literal offsets cannot exclude
computed hook addresses. It does not exonerate the FX plugins. Its Hammer-item
interpretation was also rejected on visual review. Useful next evidence is
the relocated vtable and its initialization/reset methods. The opt-in
DUMP_RENDER_PROBE diagnostic now includes nearby owner code, candidate vtables
and the caller prologue; it adds no hook or gameplay mutation.
The new probe succeeded in Eden. Relocated vtable main+0x4f69ec0 contains the
faulting fill function at slot +0x78. Slot +0x10 is main+0x3ce300 (always returns
zero); slot +0x70 is main+0x3ceaf0 (resets existing entries/counts without
allocating the list buffer). Initialization slot +0x28 is main+0x3ce310; at
+0x3ce57c/+0x3ce580 it copies the container pointer from initialization-data+8
into component+0x18. The allocation/lifetime investigation therefore needs
that initialization-data supplier, rather than an allocation inside the fill.

Probe package cacc08208cbf6fae passed clippy with warnings denied, the no-default
build, 45 host tests and the actual Skyline release build. It booted and wrote
all requested ranges, then Eden exited normally through the CLI. The diagnostic
marker was removed afterwards. This package only extends opt-in read ranges;
the response measurements remain attached to their tested 123d47db700f5d19 NRO.
Local raw evidence is under null-buffer-vtable-probe. No console fix is claimed.
## 2026-10-06: two new native-console crashes after rewrite deployment

Evidence: /home/intpa/.local/share/ssbu-mod-backups/switch-crash-20261006-121423.
These are distinct from the original Oct5 MainThread main+3cedb4 crash.
User reports regular Smash, probably Kirby, before the 3/2/1 countdown.
Latest Atmosphere screenshot shows Kirby/Wario silhouettes loading and the
rewrite window reporting Vanilla / FX On. Console clock differs from host;
use report filenames and installed build evidence, not FAT timestamps alone.

- Report 01791288254_01006a800016e000.log: MainThread DataAbort2168-0002,
  PC smash-gamepad-overlay+0xbb00, LR+0x9668; memory access is pane+0xb0.
  Deployed NRO SHAa64d4c... executable bytes match the local SGPO ELF
  (switch-crash-verified-symbols.json). GNU addr2line resolves PC to
  pane_utils::pane_name_matches, LR to the pane-validation iterator,
  stack+0x94cc to ResolvedHudSkin::is_valid, and +0x8f18 to update_runtime.
  Source validates cached raw pane pointers by dereferencing their names;
  it checks nullness but does not establish that the pane is still alive.
  Invalid cached-pane access is established. Its precise lifecycle cause
  (stale allocation versus another source of pointer corruption) is not.
- Report 01791288486_01006a800016e000.log: TaskWorker2 DataAbort2168-0002,
  PC main+0x37fe4, LR+0x37fdc, X0=0x40000000000, readaddress0x40000000010.
  Exact matching main build21450C647B8C5940ED5B68218EA128FD3C5D597D was dumped
  via isolated Eden's built-in dump_nso, then disassembled read-only.
  ldr x9,[x0,#0x10] follows38100(x23,1), which returns *(x23+16)
  using a six-entry offset table [8,16,24,32,40,48].
  Caller chain main+9a74c/+99a90/+a4e60 reaches VFX resource setup.
  Nearby same-path diagnostic string is
  "vfx system deleted the registered binary. resource id : %d."
  This identifies the resource subsystem, not the origin of corruption.
  The raw dump is a preserved NSO header plus UNCOMPRESSED mapped segments;
  header compression flags/file offsets are not rewritten by Eden.
  Disassembly is in main-load-crash.asm; private full dump remains under
  eden-cli-handoff/profile/data/eden/dump/01006A800016E000/nso/.

Both rewrite install logs confirm Vanilla, effectsOn/scopeAll/bufferAuto.
renderer-status reports no render patch calls, three active textures and
engine_statusVanilla. This rules out active LessLag/LLUltra text patches for
these boots; it does not exclude the rewrite's menu or FX passthrough hooks.
All seven SD plugin SHA256 values still matched the deployment receipt.

Evidence-preserving next test: only SGPO's NRO moved outside Skyline scanning,
to sd:/backup-deluxe-rewrite-20261006-114001/isolation-sgpo/.
Its HUD asset, other six plugins, cosmetics, and configuration remain unchanged.
Journal isolation-sgpo.json, verified/SDflushed. User asked to repeat the same
regular match. This is temporary isolation, not a shipped fix or proof of cause.
No Rust changes and no remote push.

FTP configuration checked without exposing credentials: port5000,
anonymous enabled, sysmodule420000000000000E boot2.flag present.
The initial known-neighbor probe found no responding FTP endpoint;
USB collection was used. IP address remains unknown.


## Post-results UI crash with SGPO NRO absent
The user clarified report 01791290269: the first match finished and results were shown;
the crash occurred on the black screen returning to character select. The NRO-only
isolation did not eliminate crashes. PC main+5986c reads a halfword at pane.ext_user_data_list+8;
the value loaded from pane+0xa8 is0x2000000, producing address0x2000008.
Exact main disassembly is preserved in ui-crash-verified-window.asm.
Recursive pane draws return through main+58908, Layout::Draw+4b674, and TM handle_draw.
This establishes a bad UI pane resource pointer, not the writer, lifetime or culprit.
The matching install.log still shows Vanilla, FXOn and no render backend installed.
Later collected events/status/config likewise show Vanilla, zero sequence/index/ultra calls,
and three active textures. Collection times and SHA256 are in remaining-log-collection.json.

One further isolation was applied over FTP: exact SHA82a674e7... HUD layout.arc (4,795,344 bytes)
was downloaded and verified, then moved to
sd:/backup-deluxe-rewrite-20261006-114001/isolation-sgpo/layout.arc.
The source is absent and the destination size matches; journal isolation-sgpo-layout.json.
SGPO NRO remains disabled, all other six plugins/settings unchanged.
User asked to fully restart Smash, finish one regular match Vanilla/FXOn, and return to CSS.
That test is pending. This reversible isolation is not a fix.
FTP sys-ftpd at192.168.0.226 port5000 supports anonymous; no credentials copied.
Fable follow-up still running in the resumed exact session; the MCP300s timeout is
front-door noise, not evidence the engine stopped. Send deadline approximately12:59UTC.

Fable follow-up completed; raw review and root qualifications are in the FTP evidence folder.
Root rejected its required-special-flag/missing-usd1 attribution: earlier draw path reaches the fault
without Pane+0x60 bits0/1. See picture-draw-full.asm and review-acceptance.txt. No patch based
on that unproven hypothesis. Worker wrapper stopped. Layout-only console test pending.

## Candidate lifetime testing, 2026-10-06

The first instrumented SGPO candidate (NRO SHA256
34b5525a5fae9ebf4a36f00f62dc26c21eac926179bbf161625c6fd80598a0a0) was rejected
during root-controlled Eden testing. Both 13.0.5 Finalize guards matched. The
non-draw HUD captured and updated on thread 0x1181034910, then scene update ran
on 0x892f2088. Its conservative owner-thread gate disabled further updates,
leaving a frozen overlay. A non-crashing run with a disabled overlay is not a pass.

One regular Kirby/Wario match on Town & City completed and returned through
results and stage select to CSS. At the next match start, the address-only
coverage watch reported stored=22, retired=22, live=0, overflow=false. This
supports the chosen Finalize boundary for that observed HUD teardown, not for
every native lifetime path. Match reset can clear cache entries before their
Finalize calls, so zero cache evictions alone does not disprove watch coverage.

Fable reviewed the multithread correction. Root retains the design: cached
pointer use and eviction share each cache's lock; Finalize takes it before the
engine cleanup and clears matching entries before releasing. Thread changes
are observations, not fatal errors. Positive bounded sleep replaces indefinite
busy spinning, and logging moves outside cache critical sections.

The fresh-capture helper at main+0x3776910 calls allocator main+0x392e290 and a
virtual lookup; root did not accept a lock-free assumption. Revised capture
resolves in the current engine callback before acquiring the HUD cache lock,
then publishes only if a retire epoch has not changed. The existing helper's
allocated-handle ownership also needs a separate audit; no deallocator is guessed.

Evidence and archived debug ELF:
~/.local/share/ssbu-mod-backups/switch-ftp-after-isolation-20261006/sgpo-lifetime-candidate/

Eden's host process received SIGBUS about 350ms after root requested systemd
stop. The service journal establishes that order. It is not counted as
reproduction of the Switch's guest UI exception. Peak host memory was about
6.98 GiB, with no OOM. The isolated profile was restored, and the Switch's
successful six-plugin configuration was not altered.

The asset-only post-results Switch crash remains unresolved. Static asset
checks and an Eden cycle do not establish native safety.


## Candidate2 emulator evidence, 2026-10-06

The revised SGPO candidate passed five regular Kirby/Wario Small Battlefield
match/results/stage-select/CSS cycles in isolated Eden. The exact full HUD
archive and seven-plugin stack were enabled, with rewrite Vanilla/FX On.
The overlay responded to A press and B press/release after reload. Starting
a sixth match recorded stored=110 / retired=110 / live=0 / overflow=false
for the five completed HUD lifetimes. No guest crash was observed.

NRO SHA256: 3a746514f3895da32374d1a7d1c5b9b7300c30b6ed08686cea480434180f251d.
Logs, screenshot series, command stream, debug ELF, manifest, and test limits:
~/.local/share/ssbu-mod-backups/switch-ftp-after-isolation-20261006/sgpo-lifetime-candidate2/root-test-results.json.

This addresses the cached-pane failure candidate; it does not resolve the
separate native crash with the SGPO NRO absent. The same updated Skyline and
ARCropolis were used in Eden and on Switch. Why that asset-related fault has
not appeared in these emulator runs is still unproven; neither a version
mismatch nor Training's caller frame establishes the cause.

The 20-minute service deadline ended the run while a stop-confirmation modal
was pending; subsequent SIGKILL was supervisor cleanup, not a guest crash.
Peak memory reached the 8 GiB cap with zero OOM. The isolated profile was
restored and the successful six-plugin Switch configuration left unchanged.
Native acceptance, Training-specific paths, standalone draw, skin switching
and all free-path coverage remain pending. Required full host Cargo gates
remain blocked by nnsdk's nightly requirement; no all-green claim.


## 2026-10-06T17:29Z initial corrected-overlay Switch validation passed
User completed TWO matches (Kirby, then Palutena), confirmed live overlay input
updates and both returns to CSS, then closed Smash. User declined further repeated
matches; two is accepted for this checkpoint. Do not ask for three/five more.
native-userdata-probe/test-result.json records exact scope/limits. Final evidence:
native-userdata-probe/final-overlay-two-matches/{smash-gamepad-overlay.log,
smash-gamepad-overlay-probe.log,targeted-collection.json}. New creports relative
to pre-install baseline: NONE. Probe sample normal, no suspect line. First match:
all22 watched addresses retired before second match start, live0, no overflow.
Second match last summary happens at match-end BEFORE teardown; no later summary
exists, so do not claim its final retirement count was observed. User saw CSS.
FTP collector encountered451 after historical report downloads; four relevant
current logs were recovered successfully using fresh sessions, manifest has0errors.

CORRECTION to previous Eden header-only notes: the completed archived eden2 probe
log DOES contain a normal base-pane sample, same LR5f540/vtable5240748/ext0/flags
as native. Earlier live reads were before that record appeared (no per-record
timestamp to locate it more precisely). No demonstrated native/Eden branch divergence.
Fable report fable-native-emu-probe-path.md archived under native-userdata-probe.
Root agrees with direct evidence and branch predicate; does NOT adopt claims that
post-NUL name bytes prove emulator-specific initialization (second native run also
zeros) or that an alpha predicate excludes an injected pane under possible memory
corruption. Both are at most conditional hypotheses. No parent-pointer dereference
or expanded logger is warranted absent a suspect record; preserve current tested NRO.
Fable exact6a30 wrapper stopped, own watcher reaped. Grok stopped. No code changes,
commits, or pushes since diagnostic acceptance. Native corrected overlay + probe
remain installed; PROBE_ONLY preserved in SD backup. All other plugins unchanged.
Switch HOME, Smash closed; all owned Eden stopped; both probe-run profiles restored.
Separate asset-only root cause still unproven/unreproduced. Initial two-match native
success supports lifecycle fix; do not call every historical crash resolved.
