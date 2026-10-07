# Next session: Stealth identity, profile parity and console validation

LessLag/LLUltra captured-response evidence is accepted for the tested Eden setup.
The corrected overlay passed two user-run Switch matches. Manual profile
selection now works live and persists. One-sided G&W-fix matches passed in both
LessLag and LLUltra with matching results and return to CSS. See VALIDATION.md.
Stealth remains Unavailable: its preference persists, but its transmit backend
is blocked on peer identity reconciliation. Grok and Fable's second opinion did
not establish a safe mapping; preserve both reports and collect the read-only
evidence they identify before implementing packet writes. Do not take another
opinion hop on the same unresolved design. Auto, full peer metadata and console
profile-transition validation remain pending. Doubles is deferred by the latest
operator instruction. Reduced FX follows a reproducible Switch performance dip.
Use HANDOFF.md for the exact bundle, installed files, live state and receipts.

## Environment

Use the verified custom Eden delivery at /home/intpa/eden-cli-handoff (READY,
4644 checksums verified, AppImage SHA starting fd4b91536e3d60f9, source base
10bcd2d849 plus its CLI patch). Read AGENT_HANDOFF.md and
source/docs/user/CLIControl.md there. Launch with
`launch-eden.ps1 -MemoryGiB 8 -Runtime 600`, then attach edenctl.py to the exact
socket printed by that run. Keep one bounded instance with a finite lifetime.
Two isolated profiles are permitted for linked tests; use separate data trees,
memory limits and finite deadlines. The accepted pair used 8 GiB and 7 GiB.
The older tools/EdenTest.ps1 is only for stock-baseline reproduction.

Check the installed NRO hash and startup identity against HANDOFF.md, and back
up the NRO and config before replacing either. `render_mode` and the INPUT_RECEIPT
and DUMP_ULTRA_ROUTE flags are read at boot; external edits require a closed game.
The NetProfile menu applies supported mode changes live. Restore exact staged
files from each campaign's journal after collecting logs, with both games closed.

## 1. LessLag/LLUltra response proof — accepted for this Eden setup

PERFORMANCE.md records 25 valid count trials across normal and approximately
8 ms delayed injection phases. Normal-phase responses were Vanilla 6, LessLag 3,
Ultra 2 captured frames. Delayed-phase responses were Vanilla 6–7, LessLag 3,
Ultra 2–3; the exceptional Ultra trial had a capture/present stall and is retained.
API records match the in-image input-marker serials in all three modes.

Fable 5.1/xhigh reviewed the raw data and accepted this limited emulator claim.
The marker observes the plugin's existing input poll, not earliest HID receipt;
its separation from the pose is phase-dependent. No display, photon, native
console, or universal every-input frame-count claim follows. Extra release-side
trials would broaden coverage but are not a prerequisite to the next investigation.

## 2. Remaining crash and console checks

The corrected SGPO build passed the user's Kirby and Palutena matches and
returns to CSS. Do not call the new render-policy transitions console-tested.
A separate Eden GPU subchannel crash remains unexplained (VALIDATION.md).
For a recurrence, record the mode, scene, fighters,
online/LDN topology, the exact NRO hash and the crash evidence. The v1.4.2
release moved to persistent hooks and a per-frame render_opts snapshot, which
this rewrite also uses; that design difference is not proof of the crash
mechanism. A crash on externally stopping the emulator is a shutdown failure,
not this crash.

## 3. Doubles — deferred

`doubles_core` gives three textures with OneBehind but no render_opts. Full
LLDoubles also needs render_opts and its dynamic-resolution behavior, followed by
multiplayer performance and crash checks.

## Later: measured Reduced FX

1. Smoke-test the remaining FX.md table one character/move at a time: On, Off,
   On again, in the air and on the ground where applicable. Check unrelated
   moves, hit/shield/KO feedback, menus and Training save/load afterward.
   Ordinary offline Smash and online-to-offline scope transitions need coverage,
   including arena/Quickplay/local-wireless entry and exit. Online is the
   distributed default; all is only the test override.
2. Identify the exact effect that causes a Switch dip. Record console/model,
   game and plugin hashes, docked/handheld, resolution, clocks, render mode,
   stage, fighters/costumes, damage, camera, move/charge and repetition, with
   comparable FX On and Off traces or video.
3. Reproduce that scene in Eden, recording emulator version, host hardware and
   drivers, power profile, backend, resolution, speed, accuracy and shader cache.
   Vary one setting at a time and separate first-use shader stutter from
   repeatable cost. An arbitrary FPS cap is not a reproduction.
4. Implement Reduced for one confirmed offender, preserving its charge,
   trajectory, hit and size cues. Compare repeatable On/Off/Reduced runs, then
   test the same candidate on Switch. Accept only after improved pacing and
   correct feedback/coexistence there.

Full Gigaflare charge needs the longer 4.2-second hold; earlier 2.2-second
captures are not full-charge trials. Do not compare different render modes as if
only the effect changed. Do not copy the peer's generic effect deletion, article
scaling, global victim timers or color-reset suppression.

No performance improvement, console compatibility or crash fix follows from a
successful build.
