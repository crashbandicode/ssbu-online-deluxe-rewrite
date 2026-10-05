# imgui-smash dependency (embedded host)

The global overlay embeds the imgui-smash NVN backend in this NRO. There is no
separate `libimgui_smash.nro`, no `imgui-api` crate, and no `Layout::Draw`
hook. The backend interposes NVN through one function hook on
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

- `install(draw)` refuses (returns `false`, records `HookRejected` at site
  `0x200`) when any standalone-host export resolves
  (`imgui_setup_context_export`, `imgui_get_context_export`,
  `imgui_smash_add_on_draw_frame_wrapper`), when the `nvnBootstrapLoader` entry
  is not readable, or on a second call.
- It records the entry word of `nvnBootstrapLoader` (`HookEntryWord`, site
  `0x200`) and flags a pre-existing far hook or branch as `Compatibility 4`
  before chaining through it.
- Startup checkpoints written to `install.log`: `overlay_host_begin`,
  `overlay_host_bootstrap_hooked`, `overlay_host_ready`, or one refusal phase.
  Nothing is written per frame.
- The backend logger is replaced by a lossy forwarder (first eight messages,
  then one in 256) that never trusts the buffer encoding.
- `ready()` turns true from the backend's post-init callback;
  `bootstrap_calls()` exposes how many loader calls passed through the hook;
  `display_size()` reads the main viewport size inside the draw callback and
  otherwise returns 1280x720.

## Coexistence and limits

- Training Modpack (`Layout::Draw`, layout-arc, fighter symbols), the overlay
  plugin (ui2d panes, 0x374812c, 0x1b6cc08, 0x1345558, 0x1d68b94) and
  `libcss_preserve.nro` (`set_text_string`) do not touch `nvnBootstrapLoader`
  or the NVN proc table. The future render engine's present/acquire sites are
  untouched because the backend deliberately does not wrap
  `nvnQueuePresentTexture`.
- The backend snapshots window textures in its `nvnWindowBuilderSetTextures`
  wrapper and wraps `nvnWindowAcquireTexture`; buffer-mode switching must be
  tested with the overlay enabled when the engine lands.
- The backend's own `InputHelper` only reads FullKey, Handheld and JoyDual
  pads, so ImGui-internal input ignores GameCube controllers; the menu polls
  `nn::hid` itself.
- `IM_ASSERT` aborts the game: Begin/End and Push/Pop must balance in the draw
  callback, and `ig*` must not be called outside a frame.
- Candidate NRO size is 1,683,456 bytes. Eden main-menu/CSS window and persistence
  checks passed; match/Training/console validation remains separate.

## Attribution and live evidence

The backend source carries the [GPL v2 license](licenses/imgui-smash-GPL-2.0.txt).
Host glue is adapted from Coolsonickirby/imgui-smash; table layout and navigation
follow the preserved MSFS-RyZe fork. The linked archive remains outside Git.

Candidate 2ca7c6991f9a7565 is 1,683,456 bytes. Its embedded backend initializes in
Eden with TM and SGPO installed. User confirmed the top-left window on main menu
and CSS, and FX changes persist across process restart. Other scene transitions
and console behavior remain separately tracked in VALIDATION.md.
