# Next session: effects measurement and Reduced

Stop point: user accepted the installed c1706bbe9589de97 candidate in Eden,
including FX On/Off/On, Training Modpack menu/save-load and gamepad overlay.
The expanded candidate is packaged separately and has not been installed/tested.
Use HANDOFF.md for the exact bundle and live state. Reduced is still unavailable.

1. Install the expanded NRO plus config/eden-test.ini (effects_scope=all) with Eden
   closed, retaining a backup of both old NRO and old config. Use tools/EdenTest.ps1
   and verify only one Eden instance. Check startup/log identity before testing.
2. Smoke-test the remaining table in FX.md, one character/move at a time:
   On, Off, On again; repeat in air/on ground when applicable. Check unrelated
   moves, general hit/shield/KO feedback, menus and Training save/load afterward.
   Separately verify default online-only scope leaves offline visuals intact.
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

The rewrite currently renders Vanilla. If the dip requires the original LessLag
engine, establish a matching rendering path before claiming a reproduction or a fix.
Do not compare different render profiles as if only the effect changed.
The new online scope itself needs arena/Quickplay/local-wireless entry/exit tests,
including online -> main menu -> offline match. All is only the test override;
the distributed default remains Online.

Do not copy generic effect deletion, article scaling, global victim timers or
color-reset suppression from the peer. These are separate behavior changes.
No performance improvement, LessLag completion or console compatibility is
established merely by a successful build.
