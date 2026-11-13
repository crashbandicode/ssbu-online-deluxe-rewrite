# Install

Close Smash before replacing files. Copy the rewrite NRO to:

```text
sd:/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libssbu_online_deluxe_rewrite.nro
```

Get it from the [latest Forgejo release](https://git.syntaxbyexample.com/intpa/ssbu-online-deluxe-rewrite/releases/latest),
which carries the NRO and its `manifest.json` (SHA-256, build id, module id). Skyline is required.

**That one NRO is all Deluxe needs.** Its menu has ImGui built in, so you don't need `libimgui_smash.nro` for
Deluxe (see [ImGui](#imgui) below).

**Tested on:**
- **0.10.4 on the operator's Switch, 2026-11-10:** firmware 23.0.1 and Atmosphère 1.12.1. The plugin's own install
  and event logs on the SD card record build `0.10.4-21c0009d228c-1794336309`.
- **Earlier builds:** Eden and Erista Switch, firmware 23.0.1, Atmosphère 1.12.1, Skyline beta `2eb226f`
  ([operator report, 2026-10-25](VALIDATION.md#operator-report--eden-and-erista-switch--2026-10-25)).
- **Not yet:** firmware 23.5.0 / Atmosphère 1.12.2.

## What to remove, what to keep

**You need this one NRO, not this NRO plus the old stack.** The original Online Deluxe was three plugins working
together:
- the fork NRO (menu and profiles);
- SsbuSync (the render and buffer backend the fork drives through imported symbols);
- the PIA manager (the peer handshake).

This NRO implements all three parts itself (`src/render_policy.rs`, `src/runtime/resolution.rs`,
`src/peer_transport.rs`, `src/runtime/pia_rtt.rs`).

| File in `skyline/plugins/` | Action | Why |
| --- | --- | --- |
| `libssbu_online_deluxe*.nro` (the fork, any filename variant such as `(nuevo)`) | **Remove** | Replaced by this NRO. It also crashes at launch when its own companions are missing. |
| `libssbusync.nro` | **Remove** | Replaced. If it's loaded, Deluxe switches off **all** its hooks and you are running the fork (see the symptom check below). |
| `libssbu_pia_manager.nro` | **Remove** | Replaced by the built-in handshake and RTT reading. |
| `libnx_over.nro` | Not needed | The fork's overclock companion. Deluxe does no overclocking. Keep it only if another mod needs it. |
| `libsmashline_plugin.nro` | Not needed by Deluxe | Keep it if another mod needs it (the old FX plugins did). |
| `libimgui_smash.nro` | Optional | Deluxe has ImGui built in. TrainingRooms and Elite Legal Stages draw their pages through Deluxe's menu, so they don't need it either. Keep it only if some other plugin needs it, and then use the newest build (see [ImGui](#imgui)). |
| `libnro_hook.nro`, `libparam_hook.nro`, `libarcropolis.nro`, `libcss_preserve.nro` | Keep | Shared infrastructure other mods use. |
| TrainingRooms, Elite Legal Stages, the gamepad overlay, FX Control, the CPU Pilot | Keep | Separate mods that work next to Deluxe; the ones with settings add their own page to Deluxe's window. |

Replace overlapping latency plugins rather than stacking them. Keep dependencies that other installed mods need;
don't remove them blindly. The fork's own data folder `sd:/ultimate/ssbu_online_deluxe/` and SsbuSync's
`sd:/ultimate/ssbusync/` can stay; Deluxe reads neither.

**Symptom check.** If Deluxe is installed but L+R+Down opens nothing, look in
`sd:/ultimate/ssbu_online_deluxe_rewrite/events.log` for code 16 with `a` = 1. The Skyline log has the matching line,
`[Deluxe] Existing SsbuSync host detected; rewrite hooks skipped`. Either one means SsbuSync is still loaded.

## Character FX (separate, optional)

Since 0.5.0 Deluxe has no character-FX hooks. On, Off and Reduced effects are the separate
[FX Control](https://git.syntaxbyexample.com/intpa/ssbu-fx-control) plugin, installed as its own NRO in the same
plugin folder.
- It's optional: without it, every effect is the game's original.
- With it, its FX Control page appears in Deluxe's window cycle, and it uses Deluxe's online tracking.
- Its settings are in `sd:/ultimate/ssbu_fx_control/config.ini`. Deluxe ignores the old `effects` and `effects_scope`
  keys (see [Configuration](CONFIGURATION.md)).

Replace any other character-FX plugin rather than running two.

## ImGui

**Built in.** Deluxe's menu statically links `libimgui_smash.a`, built from pinned public source with our renderer
patch. The source, build recipe and digest are in [ImGui dependencies](IMGUI_DEPENDENCY.md).

**The standalone `libimgui_smash.nro` is optional.**
- Nothing of ours needs it. Elite Legal Stages and TrainingRooms register their pages with Deluxe's plugin menu and
  don't link ImGui themselves.
- If another plugin of yours needs the standalone NRO, Deluxe reuses it instead of its own copy. Compatibility is
  checked through its exports and the ImGui ABI, not the filename.
- While the standalone copy is in use, its renderer owns the GPU buffers, so a renderer fix only arrives when you update
  that NRO too. Use the newest [imgui-smash](https://git.syntaxbyexample.com/intpa/imgui-smash) build: 1.0.2 as of
  2026-11-13. Its release isn't published yet; the newest published one is v1.0.1.

## Elite Legal Stages

A separate plugin ([Legal Stages + Loser Counterpick](https://git.syntaxbyexample.com/intpa/elite-stages-counterpick))
whose settings page, **Stages + Counterpick**, appears in Deluxe's window.
- Its **legal stage pool** narrows the stages the game may pick to the ones you check; it never adds a stage.
- Its optional **counterpick** acts only on an accepted online 1v1 rematch under your Preferred Rules.

What it does, how it was tested and what isn't proven yet are in its own
[README](https://git.syntaxbyexample.com/intpa/elite-stages-counterpick#legal-stages--loser-counterpick).

## Eden

For the normal Eden profile, the plugin folder is:

```text
/home/intpa/.local/share/eden/sdmc/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/
```

The automated test profiles under `eden-cli-handoff` are separate from that path. Set `platform=emulator` in
`config.ini` on Eden (see [Configuration](CONFIGURATION.md)).

## Rollback

Keep the previous NRO. The deployment scripts used for the operator's card archive the replaced file under
`sd:/ultimate/ssbu_online_deluxe_rewrite/backups/` before each install; copying it back over the plugin path restores
the earlier build. A Deluxe older than 0.5.0 has its own FX hooks again, so remove FX Control before rolling back to
one.
