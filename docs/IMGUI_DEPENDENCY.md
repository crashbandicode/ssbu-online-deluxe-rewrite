# imgui-smash dependency (embedded host)

The global overlay embeds the imgui-smash NVN backend in this NRO and uses it
when no standalone `libimgui_smash.nro` is loaded. Elite Legal Stages ships
that standalone plugin; when its exports match the pinned ImGui 19263 layout,
this NRO borrows that context and does not start a second backend. There is
no `imgui-api` crate and no `Layout::Draw` hook. The active backend interposes
`nvnBootstrapLoader` and draws inside the `nvnQueueSubmitCommands` wrapper, so
the overlay is available in every scene.

## Pinned artifact and sources

| Item | Value |
|---|---|
| Static archive | `libimgui_smash.a`, 16,798,872 bytes |
| SHA-256 | `0042ff0592313228b0746ee9214091249f7c70a1797e1826d990c27ca2fe0e45` |
| Immutable source | fork commit `35fdf7cf5fea0f8249ca9fbb848c241b590bb483`, path `lib/libimgui_smash.a` (git blob `aff8692e11b881b99d11dd67dbcfacc24f1cd497`) |
| Host glue and headers | `Coolsonickirby/imgui-smash` commit `bb47b205ed0be2fb864269ee2d37fc886b81221e` (`src/lib.rs`, `include/imgui_smash.h`, `include/smash_types.h`, `src/cimgui.h`) |
| Backend config at that commit | command buffer id 1, precompiled shaders not forced, viewport 1280x720, input port 0, SVC logging unless a logger is set, `IM_ASSERT` = `SMASH_ASSERT` (aborts) |

The archive is third-party compiled code and is never committed. It lives in
the git-ignored `target/native/imgui/` directory together with a sidecar digest
and `provenance.json`.

## Preparing the archive

```text
/usr/bin/python3 tools/prepare_imgui.py --source <path-to-libimgui_smash.a>
/usr/bin/python3 tools/prepare_imgui.py            # fetches the pinned copy over HTTPS
```

The script verifies size and SHA-256 before installing, refuses non-HTTPS
URLs, and writes `libimgui_smash.a`, `libimgui_smash.a.sha256` and
`provenance.json`. `build.rs` never downloads: for `target_os = "switch"` it
requires the exact path, checks the size and the sidecar digest, then emits
`rustc-link-search` plus `rustc-link-lib=static:-bundle=imgui_smash`. Host
builds and tests ignore the archive entirely. A missing or mismatched archive
fails the Switch build with instructions.

## What the archive provides and needs

Members: `cimgui`, `InputHelper`, `fsHelper`, `memoryHelper`,
`ImguiShaderCompiler`, `MemoryBuffer`, `MemoryPoolMaker`, `imgui_impl_nvn`,
`imgui_nvn`, `imgui_smash`, `Logger`, ImGui core, demo, draw, tables, widgets.
It defines the C API (`imgui_smash_init`, `imgui_smash_bootstrap_hook`,
`imgui_smash_add_on_draw_frame`, `imgui_smash_context_setup`,
`imgui_smash_set_logger`, …), the `ig*` cimgui functions, and its own allocator
(`Mem::*`), so it imports no `operator new`.

Symbols resolved at NRO load (all game or nnSdk exports that the fork's host
already imported): `__cxa_guard_acquire/release`, `nn::fs::{OpenFile, ReadFile,
WriteFile, CreateFile, CreateDirectory, OpenDirectory, CloseDirectory,
GetEntryType, GetFileSize, SetFileSize, CloseFile}`, `nn::os::{GetSystemTick,
ConvertToTimeSpan}`, `nn::ro::LookupSymbol`,
`nn::gfx::detail::GlslcDll::{GetInstance, Finalize}`,
`nn::hid::{GetNpadStyleSet, GetNpadState (FullKey, Handheld, JoyDual),
InitializeKeyboard, InitializeMouse, GetKeyboardState, GetMouseState,
GetTouchScreenState<1>}`, `nn::diag::detail::AbortImpl`, `nn::util::VSNPrintf`,
`malloc`, `free`, `memcpy`, `memset`, `strlen`, `snprintf`, `vsnprintf`.
`__cxa_guard_*` resolve to the game's C++ runtime, not to Skyline's private
copy involved in the earlier `get_plugin_addresses` abort.

## Runtime behaviour of `src/runtime/overlay_host.rs`

Skyline's first `nn::fs::MountRom` waits until `LoadPluginsImpl` has
`LoadModule`'d every plugin and finished every plugin `main`
(`source/main.cpp` mount hook, `PluginManager.cpp` load loop then the main
loop). `imgui_smash_init` during that `main` only queues callbacks;
`InitImGui` creates the context later, once the NVN device, queue, and command
buffer exist (`imgui_nvn.cpp`). Detection is by export, not by file path.

`src/imgui_host.rs` chooses:

- **Embedded** when none of the standalone exports resolve. This NRO calls
  `imgui_smash_init`, installs its NVN renderer, and registers `draw` on its
  own cimgui. Unchanged from the previous host.
- **External** when `imgui_setup_context_export` and
  `imgui_smash_add_on_draw_frame_wrapper` both fall in one other NRO's text
  segment, and that header's ro segment contains `1.92.7 WIP (19263)`.
  The text mapping must start at the NRO header (`text_offset == 0`, which the
  Elite file uses: text `0x0/0xe8000`, ro `0xe8000/0x42000`, tag at file
  `0xf6083`). `imgui_get_context_export` is optional and, if present, must be
  in that same text segment. The Elite NRO does not export it. This path does
  not call this NRO's `imgui_smash_init` or `imgui_smash_bootstrap_hook`. It
  registers the setup callback and a draw thunk with the standalone. The thunk
  calls this NRO's `igSetCurrentContext` with the borrowed pointer, checks
  `igGetCurrentContext` matches, then runs the existing UI. `ig*` are direct
  links into this NRO's compiled cimgui. A readelf of this NRO showed those
  symbols are not dynamic exports. The standalone module can export the same
  names from its own compiled copy. `LookupSymbol` is not used for `ig*`,
  because that lookup would follow the export, not this NRO's copy.
- **Refuse** when any standalone export is present but the pair or the version
  tag is not. No `imgui_smash_init`, no draw callback, no `ig*` call. The NVN
  observation hook still chains to `original!()` so window setup keeps running.

The bootstrap hook records the `nvnBootstrapLoader` entry word (`HookEntryWord`,
site `0x200`) and a pre-existing far hook or branch as `Compatibility 4`.
External acceptance records `Compatibility 5` with detail `19263`. An unknown
ABI records `HookRejected` detail `8`. A null or unaligned borrowed context is
detail `5`. A `igSetCurrentContext` that does not stick is detail `6`.

Startup checkpoints: `overlay_host_begin`, then either
`overlay_host_bootstrap_hooked` / `overlay_host_ready`,
`overlay_host_external_chain` / `overlay_host_external_ready`, or
`overlay_host_refused_unknown_abi` / `overlay_host_entry_unreadable`.
Nothing is written per frame.

The embedded logger is still the lossy forwarder (first eight messages, then
one in 256). It is not installed on the external path, because that would
configure this NRO's copy rather than the standalone.

`ready()` is the embedded post-init callback, or the first external draw after
the borrowed context is current. `display_size()` reads the main viewport only
then, so it sees the same context the draw calls use. Otherwise it returns
1280x720.

## Coexistence and limits

- Training Modpack (`Layout::Draw`, layout-arc, fighter symbols), the overlay
  plugin (ui2d panes, 0x374812c, 0x1b6cc08, 0x1345558, 0x1d68b94) and
  `libcss_preserve.nro` (`set_text_string`) do not touch `nvnBootstrapLoader`
  or the NVN proc table. The ImGui backend does not wrap
  `nvnQueuePresentTexture`.
- The backend snapshots window textures in its `nvnWindowBuilderSetTextures`
  wrapper and wraps `nvnWindowAcquireTexture`; the in-tree engine chains acquisition and preserves that texture ownership.
  Live profile changes with the overlay enabled passed the recorded Eden tests.
- The backend's own `InputHelper` only reads FullKey, Handheld and JoyDual
  pads, so ImGui-internal input ignores GameCube controllers; the menu polls
  `nn::hid` itself.
- `IM_ASSERT` aborts the game: Begin/End and Push/Pop must balance in the draw
  callback, and `ig*` must not be called outside a frame.
- Both Deluxe and Elite consume D-pad input when their menus are open. Edit
  one menu at a time. Elite's stage restriction is NSO-only; offline stages
  remaining unrestricted is expected.

## Attribution and live evidence

The backend source carries the [GPL v2 license](licenses/imgui-smash-GPL-2.0.txt).
Host glue is adapted from Coolsonickirby/imgui-smash; table layout and navigation
follow the preserved MSFS-RyZe fork. The linked archive remains outside Git.

Historical candidate 2ca7c6991f9a7565 was 1,683,456 bytes. Its embedded backend initializes in
Eden with TM and SGPO installed. User confirmed the top-left window on main menu
and CSS, and FX changes persist across process restart. Other scene transitions
and console behavior remain separately tracked in VALIDATION.md.

## Export name collision (2026-10-06)

Initial runtime validation exposed a symbol collision. The normal-order
Eden run showed both menus but did not prove this NRO's NVN chain. Both plugins
installed the same `nvn_bootstrap_hook` replacement address; the ELF relocation
for that symbol was `GLOB_DAT`. Reversed order left both UIs absent while this
NRO's NVN observations ran. `-Bsymbolic-functions` on this link (ld.lld) binds
this NRO's own function references locally and does not stop a later plugin
from resolving an export that still uses the shared name. Every
`#[skyline::hook]` callback is therefore exported with a `deluxe_rewrite_`
prefix. Before `install_hook`, bootstrap installation logs the chain, this
NRO's module base, the replacement address, and that address's module base
(`loaded_nro`). A replacement that is not this NRO is `HookRejected` detail
`9` (`overlay_host_foreign_bootstrap`) and is not installed. Callback
registration stays the one embedded or external path above. There is no
`Layout::Draw` hook and no `0x374812c` patch.

Candidate `defda6fdf99e01e08906f4ae7e8f7df7cfab0d4fa1eb42b478e9ad6499146135`
then passed three isolated Eden runs: embedded, standalone before rewrite,
and standalone after rewrite. The external cases used distinct replacement
addresses in the two NROs, registered nonzero rewrite present observations,
and displayed both Deluxe and Elite Legal Stages menus. FX persistence was
also checked in the embedded and reversed-order runs. All three exited
normally and test profiles were restored. Raw evidence and run IDs are in
`VALIDATION.md`; actual Elite stage filtering and Switch coexistence remain
separate tests.
