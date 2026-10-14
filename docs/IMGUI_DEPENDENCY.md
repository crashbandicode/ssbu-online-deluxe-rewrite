# imgui-smash dependency (embedded host)

The global overlay embeds the imgui-smash NVN backend in this NRO and uses it
when no standalone `libimgui_smash.nro` is loaded. Elite Legal Stages ships
that standalone plugin; when its exports match the pinned ImGui 19263 layout,
this NRO borrows that context and does not start a second backend. There is
no `imgui-api` crate and no `Layout::Draw` hook. The active backend interposes
`nvnBootstrapLoader` and draws inside the `nvnQueueSubmitCommands` wrapper, so
the overlay is available in every scene.

## Pinned artifact and sources

Since 0.3.8 the backend is built from the operator's Forgejo fork, whose
`deluxe` branch carries the former local renderer patch as a commit plus the
memory trims below. The compiled archive stays in the ignored
`target/native/imgui/` directory.

| Item | Value |
|---|---|
| Static archive | `libimgui_smash.a`, 13,851,556 bytes |
| SHA-256 | `8a06be16dbfd50d10bac81ee5bdd2ffa9cd3472773151cc45f1c2101541f8f7c` |
| Source | `https://git.syntaxbyexample.com/intpa/imgui-smash`, branch `deluxe`, commit `7f0e47c4a1699a714254798bf79041b1f2e49196` |
| Upstream | `Coolsonickirby/imgui-smash` at `bb47b205ed0be2fb864269ee2d37fc886b81221e` (fork `main`) |
| ImGui submodule | `2b415e642b002f4312ac02f6bfa955960f8bd3b6` (unchanged) |
| Historical patch | `patches/imgui-smash/renderer-bb47.patch` (`4e138e35…`), now fork commit `ae77450`; no longer applied by the tool |

Previous pin (0.3.0–0.3.7): archive `6c9591cc9f140ca6ce2efee2cdcaa0d8278b43319ea497b3f2434ce19d09ccca`,
17,181,302 bytes, built from upstream `bb47b205` plus the local patch.

The renderer correction prevents overlapping ImGui frames and gives each
in-flight overlay submission its own command, control, geometry and uniform
storage. Eight slots are reused only after a nonblocking fence check reports
completion. If all slots are busy, that overlay draw is skipped. Exhausted
command memory uses a bounded reserve to finish and discard the unsubmitted
draw safely.

### Fork branch `deluxe` (2026-10-14)

| Commit | Change |
|---|---|
| `ae77450` | Renderer correction (the former `renderer-bb47.patch`, applied verbatim) |
| `8ef779e` | Crate root replaced with the standalone companion crate (drops `imgui-api`, prefixes the bootstrap hook, links a pinned archive); reproduces archive `6c9591cc` and the installed standalone NRO `9a9e575c` byte for byte |
| `b923d90` | `IMGUI_DISABLE_DEMO_WINDOWS`; the ShowDemoWindow link removed; `imgui_show_demo_window_wrapper` stays exported as an empty function |
| `ef98524` | `IMGUI_DISABLE_DEBUG_TOOLS`; `igDebugLog`, `igDebugLogV`, `igShowFontSelector`, `igDebugTextEncoding`, `igDebugFlashStyleColor` kept as no-ops so the `ig*` set is unchanged |
| `5ec9928` | ProggyClean/ProggyForever default fonts dropped (the displayed font, JetBrains Mono NL 18 px, is unchanged) |
| `384b095`, `7f0e47c` | Companion crate pinned to archive `8a06be16`; archive linked `kind = "static"` with `ImGui::GetCurrentContext` so `cimgui.o` stays out of the standalone |

None of the disable macros changes a struct in `imgui.h` or `imgui_internal.h`,
so contexts stay compatible across old and new builds and the external-host
checks below still hold (`1.92.7 WIP (19263)` tag, `imgui_setup_context_export`
and `imgui_smash_add_on_draw_frame_wrapper` in text at the same addresses,
`text_offset == 0`, identical export set). Standalone companion built from
`7f0e47c`: `libimgui_smash.nro` 888,832 bytes (was 1,282,048), SHA-256
`a3f794b2d2af15d5f2c2868f22e81c865702524160640864604731b7afe6775d`, mapped
image 921,600 bytes (was 1,323,008). Deferred trims with their measured
costs (draw-slot budgets ≈ 3.75 MiB of pool memory, `ClearTexData()` 256 KiB,
R8 atlas 384 KiB, gc-sections for the Skyline link) are listed in the tangent
evidence `imgui-fork-1014/REPORT.md`.

## Preparing the archive

Docker and Git are required for a source build. The script uses the pinned
devkitA64 image recorded in `archive-pin.json`, the current user's uid/gid,
two CPUs and a 2 GiB container limit.

```text
/usr/bin/python3 tools/prepare_imgui.py
```

A local checkout of the pinned source and submodule avoids cloning:

```text
/usr/bin/python3 tools/prepare_imgui.py --upstream /path/to/imgui-smash
```

An already-built archive must match the same digest:

```text
/usr/bin/python3 tools/prepare_imgui.py --source /path/to/libimgui_smash.a
/usr/bin/python3 tools/test_prepare_imgui.py
```

The script clones the pinned commit of the pinned repository (applying a
patch only if the pin names one), builds from a clean source copy and
verifies the result before installing the archive, sidecar digest and
`provenance.json`. Deterministic `ar qcD` and `ranlib -D` rules produced
identical archives in two clean builds. An existing verified archive skips
the build. `build.rs` performs no downloads; host builds do not link the
archive.

## Renderer diagnostics

The existing five-second writer appends `imgui_frame_*` and `imgui_draw_*`
counters to `renderer-status.txt` when Deluxe owns the embedded backend.
It adds no background thread or per-frame file writes. `imgui_stats=not_embedded`
means those local counters do not describe the active host.

The former `PROBE_MENU_GPU_COMPLETION` probe and original-command-buffer
recording hooks have been removed. They were investigation tools and did not
describe the replacement's per-slot buffers. The analysis script still reads
their archived trace records; numeric event IDs have not been reassigned.

## What the archive provides and needs

Members: `cimgui`, `InputHelper`, `fsHelper`, `memoryHelper`,
`ImguiShaderCompiler`, `MemoryBuffer`, `MemoryPoolMaker`, `DrawSlots`,
`frame_guard`, `imgui_impl_nvn`,
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
  `libcss_preserve.nro` do not touch `nvnBootstrapLoader`
  or the NVN proc table. The ImGui backend does not wrap
  `nvnQueuePresentTexture`.
- The game text setter at `0x37a28a0` already has a Skyline hook in the tested
  combined installation. Its owner has not been identified; the earlier
  attribution to CSS Preserve was unverified. The native banner calls through
  this entry after checking its standard trampoline and untouched original
  instruction tail.
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
- On the October9 Switch build, the operator reports that Deluxe Debug's
  `L+R+Up` capture shortcut also opens Elite. Deluxe shows `Capturing (10 s)`,
  so both respond. The collected files show recording completed, with a complete
  callback CSV and truncated API CSV; the source audit identified per-fragment
  flush-requesting writes in export. Avoid repeated chord presses during collection.
  The shortcut overlap does not establish the export cause or explain the launch
  failures. Shortcut rebinding is separate from the buffered-export correction.

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

## Native display follow-up, 2026-10-07

Candidate `4a927dc20904722d` showed standalone Deluxe and Elite windows together
with native CSS text in solo Eden tests, both with and without G&W. One
concurrent two-instance run had invisible external windows despite valid
registrations and working input edits; its native text and the second peer's
embedded window remained visible. That run did not establish a G&W or native-pane cause. The follow-up below
addresses two defects in the backend; the intermittent symptom still needs
Switch confirmation. See [NATIVE_DISPLAY.md](NATIVE_DISPLAY.md).


## Renderer correction and cleanup, 2026-10-09

The original backend reproduced an ImGui assertion that a previous frame had
not ended. The replacement uses one nonblocking owner for the whole frame.
It also stops re-recording command, geometry and uniform memory that can still
belong to an outstanding GPU submission. The fence belongs to the submitting
queue, and polling it never waits for the GPU.

The guard-only experiment still lost a menu once, then passed a later repeat.
That limits causal claims: the source defects are established, but the
intermittent visibility symptom was not deterministic. The combined correction
has passed the recorded Eden transitions; Switch confirmation remains required.

The final rewrite is `7fdad40f10705afb`. The cleanup removes three superseded
probe modules, their original-command-buffer wrappers and one unused UI status
accessor. Shared memory-region/reserve and retirement helpers reduce duplicate
C++ logic. Compared with the pre-cleanup Rust snapshot and pinned upstream C++,
the renderer correction plus cleanup adds 40 net production source lines.
Tests and the source-build script are counted separately; temporary audit
harnesses stay outside the repository.

Required host and Skyline checks pass, including 247 Rust tests. Four focused
C++ helpers pass with warnings denied; the full upstream C++ build retains its
existing warnings. Two clean source builds produce the pinned archive byte for
byte. Raw run evidence and exact artifact hashes are recorded in
[VALIDATION.md](VALIDATION.md).

The external-host path uses the standalone NRO's backend. It cannot acquire
this embedded fix just by updating Deluxe. The patched standalone companion
uses the same archive and preserves Elite's exports. Diagnostic variants used
temporary counter threads; the release companion disables those features.
