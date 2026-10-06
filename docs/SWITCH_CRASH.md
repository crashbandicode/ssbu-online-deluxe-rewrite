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
Vanilla is a root-cause fix. The rewrite's measured modes currently select the
active texture count during window initialization and do not switch it live.

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