# Next session: LessLag/LLUltra proof, Switch crash, then Doubles

The user's current order is: demonstrate objective response-frame savings for
full LessLag and LLUltra in custom Eden; then reproduce and fix the Switch
in-match crash in one or two Eden instances (LDN where needed); then Doubles.
Reduced FX follows an actual Switch performance reproduction. Earlier text that
deferred Doubles at the operator's request is superseded by this order.
Use HANDOFF.md for the exact bundle, installed files, live state and receipts.

## Environment

Use the verified custom Eden delivery at /home/intpa/eden-cli-handoff (READY,
4644 checksums verified, AppImage SHA starting fd4b91536e3d60f9, source base
10bcd2d849 plus its CLI patch). Read AGENT_HANDOFF.md and
source/docs/user/CLIControl.md there. Launch with
`launch-eden.ps1 -MemoryGiB 8 -Runtime 600`, then attach edenctl.py to the exact
socket printed by that run. Keep one bounded instance with a finite lifetime.
The older tools/EdenTest.ps1 is only for stock-baseline reproduction.

Check the installed NRO hash and startup identity against HANDOFF.md, and back
up the NRO and config before replacing either. `render_mode` and the INPUT_RECEIPT
and DUMP_ULTRA_ROUTE flags are read at boot; change them only with the game closed.
Restore `render_mode=vanilla` and remove both flags when a campaign ends.

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

## 2. Switch in-match crash

Reproduce it in Eden before changing code. Record the mode, scene, fighters,
online/LDN topology, the exact NRO hash and the crash evidence. The v1.4.2
release moved to persistent hooks and a per-frame render_opts snapshot, which
this rewrite also uses; that design difference is not proof of the crash
mechanism. A crash on externally stopping the emulator is a shutdown failure,
not this crash.

## 3. Doubles

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
