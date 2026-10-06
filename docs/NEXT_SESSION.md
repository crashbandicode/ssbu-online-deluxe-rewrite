# Next session: effects measurement and Reduced

The user accepted the global menu, FX On/Off/On, Training Modpack menu/save-load
and gamepad overlay in Eden. Later work added the expanded FX catalog, bounded
timing capture and an experimental LessLag core. Use HANDOFF.md for the exact
bundle, completed tests and live state. Reduced is still unavailable. Doubles is
deferred at the operator's request.

1. Use the verified custom Eden delivery at /home/intpa/eden-cli-handoff; READY
   and all 4644 delivery checksums were verified. Read AGENT_HANDOFF.md and
   source/docs/user/CLIControl.md there. The AppImage SHA starts fd4b91536e3d60f9;
   source base is 10bcd2d849 with its supplied CLI patch. Launch with
   launch-eden.ps1 -MemoryGiB 8 -Runtime 600, then attach edenctl.py to the exact
   socket printed by that run. The older tools/EdenTest.ps1 is for stock-baseline
   reproduction only. Retain that baseline. Use the custom build's CLI
   controls/screenshots for subsequent tests, with one bounded instance and a
   finite lifetime. Check the installed rewrite NRO hash and startup identity
   against HANDOFF.md. Keep effects_scope=all for offline tests; rendering defaults
   to Vanilla. Back up files before replacing a build or config.
2. Smoke-test the remaining table in FX.md, one character/move at a time:
   On, Off, On again; repeat in air/on ground when applicable. Check unrelated
   moves, general hit/shield/KO feedback, menus and Training save/load afterward.
   Online-only scope was verified to preserve offline Training visuals; ordinary
   offline Smash and online-to-offline transitions still need coverage.
3. Establish which exact effect causes a dip on Switch. Record console/model,
   game/plugin hashes, docked/handheld, resolution, CPU/GPU/memory clocks, renderer
   profile, stage, fighters/costumes, damage, camera, move/charge and repetition.
   Record a baseline frame-time trace/video with FX On and comparable Off results.
   Do not assume every peer target is expensive.
4. Reproduce that same scene in Eden. Record Eden version, host CPU/GPU/drivers,
   power profile, graphics backend, docked/handheld, resolution scale, speed cap,
   CPU accuracy/multicore, shader/cache state and installed plugins. Begin from the
   current baseline; vary one setting at a time and separate cold shader stutter
   from a repeatable effect cost. An arbitrary FPS cap is not a reproduction.
5. Implement Reduced for ONE confirmed offender, retaining its recognizable charge,
   trajectory/hit/size cues. Compare repeatable On/Off/Reduced runs, then test the
   same candidate on Switch with the recorded setup and exact debug bundle.
6. Accept only after improved frame pacing and correct feedback/coexistence on
   Switch; then iterate through the remaining confirmed offenders.

The rewrite defaults to Vanilla. The opt-in lesslag_core is only a partial match
for the original LessLag engine; establish parity before claiming reproduction of
a LessLag-specific console dip. Full Gigaflare charge was visually confirmed by
the operator with the longer 4.2-second hold; earlier 2.2-second captures are not
accepted as full-charge trials.
Do not compare different render profiles as if only the effect changed.
The new online scope itself needs arena/Quickplay/local-wireless entry/exit tests,
including online -> main menu -> offline match. All is only the test override;
the distributed default remains Online.

Custom Eden acceptance is recorded in PERFORMANCE.md. Vanilla and LessLag core
both reached Training with complete paired callback/API captures and live FX
changes. DISABLE_RENDERER recovery was checked with a LessLag request still in
the INI. These are functional checks, not latency or performance acceptance.
For objective latency work, carry unique frame identity through the emulator's
queue/compositor/present path and identify the first changed content. The existing
OnFrameDisplayed callback also runs on a hidden-window early return; its name
does not make it display evidence. Do not count CLI milliseconds or TAS callbacks
as game frames without validating their relationship.

Do not copy generic effect deletion, article scaling, global victim timers or
color-reset suppression from the peer. These are separate behavior changes.
No performance improvement, LessLag completion or console compatibility is
established merely by a successful build.
