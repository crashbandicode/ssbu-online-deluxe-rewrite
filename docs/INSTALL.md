# Install

Close Smash before replacing files. Copy the rewrite NRO to:

```text
sd:/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libssbu_online_deluxe_rewrite.nro
```

The [0.8.0 Forgejo release](https://git.syntaxbyexample.com/intpa/ssbu-online-deluxe-rewrite/releases/tag/v0.8.0)
carries the NRO and its `manifest.json` (SHA-256, build id, module id).
Skyline is required. The operator reports
Eden and Erista Switch compatibility with firmware **23.0.1**, Atmosphère
**1.12.1** and Skyline **beta `2eb226f`** (latest checked 2026-10-25;
[version links and evidence](VALIDATION.md#operator-report--eden-and-erista-switch--2026-10-25)).

## What to remove, what to keep

**You need this one NRO, not this NRO plus the old stack.** The original
Online Deluxe was three plugins working together: the fork NRO (menu and
profiles), SsbuSync (the render and buffer backend the fork drives through
imported symbols) and the PIA manager (the peer handshake). This NRO implements
all three parts itself (`src/render_policy.rs`, `src/runtime/resolution.rs`,
`src/peer_transport.rs`, `src/runtime/pia_rtt.rs`).

| File in `skyline/plugins/` | Action | Why |
| --- | --- | --- |
| `libssbu_online_deluxe*.nro` (the fork, any filename variant such as `(nuevo)`) | **Remove** | Replaced by this NRO. It also crashes at launch when its own companions are missing. |
| `libssbusync.nro` | **Remove** | Replaced. If it is loaded, Deluxe disables **all** of its hooks (`legacy_host_present` in `src/runtime/mod.rs`), prints `[Deluxe] Existing SsbuSync host detected; rewrite hooks skipped` and records event code 16 with `a` = 1. No window, no profiles: you are running the fork. |
| `libssbu_pia_manager.nro` | **Remove** | Replaced by the built-in handshake and RTT reading. |
| `libnx_over.nro` | Not needed | The fork's overclock companion. Deluxe does no overclocking. Keep only if another mod needs it. |
| `libsmashline_plugin.nro` | Not needed by Deluxe | Keep if another mod needs it (the old FX plugins did). |
| `libnro_hook.nro`, `libparam_hook.nro`, `libarcropolis.nro`, `libcss_preserve.nro` | Keep | Shared infrastructure other mods use. |
| `libimgui_smash.nro` | See below | Keep when Elite Legal Stages needs it. |
| Training Modpack, gamepad overlay, FX Control | Keep | Separate mods that work next to Deluxe. |

Replace overlapping latency plugins rather than stacking them. Keep
dependencies other installed mods need; do not remove those blindly. The fork's
own data directory `sd:/ultimate/ssbu_online_deluxe/` and SsbuSync's
`sd:/ultimate/ssbusync/` can stay; Deluxe reads neither.

**Symptom check.** If Deluxe installed but L+R+Down opens nothing, look at
`sd:/ultimate/ssbu_online_deluxe_rewrite/events.log` for code 16 with `a` = 1,
or at the Skyline log for the line above. That means SsbuSync is still loaded.

## Character FX (separate, optional)

Since 0.5.0 Deluxe has no character-FX hooks. On, Off and Reduced effects are
the separate [FX Control](https://git.syntaxbyexample.com/intpa/ssbu-fx-control)
plugin, installed as its own NRO in the same plugin directory. It is optional:
without it, every effect is the game's original. With it, its FX Control page
appears in Deluxe's window cycle and it uses Deluxe's online tracking. Its
settings are `sd:/ultimate/ssbu_fx_control/config.ini`; Deluxe ignores the old
`effects` and `effects_scope` keys (see [Configuration](CONFIGURATION.md)).
Replace any other character-FX plugin rather than running two.

## ImGui host

The menu statically links `libimgui_smash.a`, built from pinned public source
with our renderer patch. The source, build recipe and digest are recorded in
[ImGui dependencies](IMGUI_DEPENDENCY.md). When a compatible standalone
`libimgui_smash.nro` is installed, Deluxe reuses that host. Keep the standalone
plugin if **Elite Legal Stages** needs it; Deluxe's embedded copy does not
replace another plugin's dependency. Compatibility is checked through host
exports and the ImGui ABI, not just the filename. When the standalone host is
active, its renderer owns the GPU buffers; updating Deluxe alone does not update
that backend. Use the matching patched standalone host to receive the renderer
fix in an Elite installation.

Elite's stage filtering applies to **NSO**, not offline matches. Offline stages
remaining unrestricted is expected. Emulator coexistence tests do not validate
NSO matchmaking or its stage filter.

## Eden

For the normal Eden profile, the plugin directory is:

```text
/home/intpa/.local/share/eden/sdmc/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/
```

The automated test profiles under `eden-cli-handoff` are separate from that
path. Set `platform=emulator` in `config.ini` on Eden (see
[Configuration](CONFIGURATION.md)).

## Rollback

Keep the previous NRO. The deployment scripts used for the operator's card
archive the replaced file under
`sd:/ultimate/ssbu_online_deluxe_rewrite/backups/` before each install; copying
it back over the plugin path restores the earlier build. A Deluxe older than
0.5.0 has its own FX hooks again; remove FX Control before rolling back to one.
