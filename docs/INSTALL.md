# Install

Close Smash before replacing files. Copy the rewrite NRO to:

```text
sd:/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libssbu_online_deluxe_rewrite.nro
```

Every Forgejo release carries the NRO and its `manifest.json` (SHA-256, build
id, module id). Skyline is required.

## What to remove, what to keep

This NRO implements the sync/render transitions and the peer-manager handshake
itself (`src/render_policy.rs`, `src/runtime/resolution.rs`,
`src/peer_transport.rs`, `src/runtime/pia_rtt.rs`). Leave out the old Online
Deluxe, SsbuSync, and PIA-manager plugins: if SsbuSync exports are already
present, the rewrite skips its hooks (`src/runtime/mod.rs` `legacy_host_present`).
Replace overlapping latency plugins. Keep dependencies needed by other installed
mods. Training Modpack and the gamepad overlay remain separate mods.

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
