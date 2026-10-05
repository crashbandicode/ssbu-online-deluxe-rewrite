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
queue. Parsed configuration confirms each requested offline preset. No stage
or in-match behavior was verified. This does not isolate LessLag as the cause,
and it does not reproduce the console's Data Abort.

The test logs, configurations and plugin hashes are preserved locally at:

    /home/intpa/.local/share/ssbu-mod-backups/eden-switch-stack-20261005-183600/

The original seven-plugin rewrite/TM/SGPO setup was restored and all seven hashes
verified. The temporary original-Deluxe config was removed from Eden. The operator
is providing a custom Eden build with CLI control and screenshots for subsequent
testing; preserve its version and instructions before using these results as a
comparison baseline. No further Switch SD access occurred.

No source fix is currently established for the reported console crash.
