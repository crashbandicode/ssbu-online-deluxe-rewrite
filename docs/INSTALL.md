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
| `libssbu_online_deluxe*.nro` (the fork, any filename variant such as `(nuevo)`) | **Remove** | Replaced by this NRO. On its own, without the PIA manager, it crashed at launch on the operator's Switch (2026-10-15, [Validation](VALIDATION.md#050--2026-10-14-character-fx-split-into-ssbu-fx-control-plugin-own-pages)). |
| `libssbusync.nro` | **Remove** | Replaced. If it's loaded, Deluxe switches off **all** its hooks (see the symptom check below). |
| `libssbu_pia_manager.nro` | **Remove** | Replaced by the built-in handshake and RTT reading. |
| `libnx_over.nro` | Not needed | The fork's overclock companion. Deluxe does no overclocking. Keep it only if another mod needs it. |
| `libsmashline_plugin.nro` | Not needed by Deluxe | Keep it if another mod needs it. |
| `libimgui_smash.nro` | **Remove** unless another mod needs it | Deluxe has ImGui built in. While this file is loaded, Deluxe draws with it instead: imgui-smash 1.1.0 draws names outside ASCII as the built-in ImGui does, older versions show `?` (see [ImGui](#imgui)). FX Control, CPU Pilot, Legal Stages + Loser Counterpick and TrainingRooms don't link ImGui; they add their rows or pages to Deluxe's window. |
| `libnro_hook.nro`, `libparam_hook.nro`, `libarcropolis.nro`, `libcss_preserve.nro` | Keep | Shared infrastructure other mods use. |
| FX Control, CPU Pilot, Legal Stages + Loser Counterpick, TrainingRooms, the gamepad overlay | Keep | Separate mods that work next to Deluxe. FX Control, CPU Pilot and Legal Stages + Loser Counterpick each add their own page to Deluxe's window; TrainingRooms adds a row to its Plugins page. |

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
- With it, its **FX Control** and **Stages** pages appear in Deluxe's window cycle, and it uses Deluxe's online
  tracking.
- Its settings are in `sd:/ultimate/ssbu_fx_control/config.ini`. Deluxe ignores the old `effects` and `effects_scope`
  keys (see [Configuration](CONFIGURATION.md)).

Replace any other character-FX plugin rather than running two.

## ImGui

**Built in.** Deluxe's menu statically links `libimgui_smash.a`, built from a pinned commit of our public
[imgui-smash](https://git.syntaxbyexample.com/intpa/imgui-smash) fork. The source, build recipe and digest are in
[ImGui dependencies](IMGUI_DEPENDENCY.md).

**Leave out the standalone `libimgui_smash.nro` unless another mod needs it.**
- Nothing of ours needs it. FX Control, CPU Pilot, Legal Stages + Loser Counterpick and TrainingRooms register their
  rows or pages with Deluxe's plugin menu and don't link ImGui themselves.
- When it's loaded, Deluxe doesn't start its own ImGui. If its exports and ImGui version (1.92.7 WIP, 19263) match,
  Deluxe draws in that plugin's ImGui; if they don't, Deluxe's window stays off. The check uses the exports, not the
  filename (`src/imgui_host.rs`, `src/runtime/overlay_host.rs`).
- Drawing in the standalone copy means its font and its renderer. imgui-smash 1.1.0 is current: it is built from the
  source Deluxe 0.10.4 links plus two fixes (a bounded swapchain texture table, and L3 + R3 gamepad navigation off
  by default), and since 0.10.5 Deluxe links exactly its archive, so its fonts are the built-in ones. It is still
  optional. v1.0.1 and earlier have only an ASCII font, so opponent names outside ASCII show `?`.

## Legal Stages + Loser Counterpick

A separate plugin ([elite-stages-counterpick](https://git.syntaxbyexample.com/intpa/elite-stages-counterpick)) that
replaces the original Elite Legal Stages NRO; remove that one. Its settings page, **Stages + Counterpick**, appears in
Deluxe's window.
- Its **legal stage pool** narrows the stages the game may pick to the ones you check; it never adds a stage.
- Its optional **counterpick** acts only on an accepted online 1v1 rematch under your Preferred Rules.

What it does, how it was tested and what isn't proven yet are in its own
[README](https://git.syntaxbyexample.com/intpa/elite-stages-counterpick#legal-stages--loser-counterpick).

## Eden

On Linux, Eden's default profile keeps the plugin folder at:

```text
~/.local/share/eden/sdmc/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/
```

Set `platform=emulator` in `config.ini` on Eden; Deluxe doesn't detect the emulator itself (see
[Configuration](CONFIGURATION.md)).

## Rollback

Keep the previous NRO; copying it back over the plugin path restores the earlier build. The operator's card installs
keep the replaced file under `sd:/ultimate/ssbu_online_deluxe_rewrite/backups/`. A Deluxe older than 0.5.0 has its own
FX hooks again, so remove FX Control before rolling back to one.
