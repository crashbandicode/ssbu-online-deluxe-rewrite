# Fork feature parity

Source of record: `reference/quickplay` at `35fdf7cf` (see `reference/README.md`).
Working-tree changes based on `6e003597fb2a`, reviewed 2026-10-06.
A feature is listed only when an `install()`
path or an exported function actually registers it. `docs/STATUS.md` is a
progress note, not this inventory.

`src/policy.rs` is a tested specification. Runtime never calls `select_profile`.
Do not treat those tests as shipped match behavior.

The rewrite is one NRO. Keep Training Modpack and SGPO coexistence: no
`Layout::Draw`, FX hooks call `original!`, and `llultra` reads SGPO's word at
`0x374812c` without patching it (`src/runtime/render_ultra.rs`).

## External dependencies

These are not implemented inside `reference/quickplay/src`. They are separate
components with public repositories, not a claim that the rewrite cannot run.
Whether a linked backend is usable is a review of that backend, not a
conclusion from the missing snapshot directory. The snapshot's `Cargo.toml`
patches `ultelier` to `vendor/smash-ultelier`, and that directory is absent
from the snapshot. A local checkout of the guest API is
`/tmp/delay-audit-01a10c4e-93fe-7db0-9d89-4bcd50bf2340/smash-ultelier/crates/sync-guest/src/lib.rs`
(`install`, `replace_env_flags`, resolution, dynamic-res push/pop, fps boost,
vsync, render opts). It is not part of the rewrite crate.

| Dependency | What the fork uses it for |
|---|---|
| `ultelier::sync_guest` | All live buffer/index/vsync/resolution/boost changes and `SsbuSyncConfig::vanilla()` at `render/mod.rs:132-135` |
| `ssbu_pia_interface` | Station list, RTT, interface type, stability, custom send/receive. Not in this tree |
| `libssbu_pia_manager` NRO | Separate plugin. Its `0x45` beacon is what other clients read as `is_modded`. Stealth patches that binary (`net/pia/manager_stealth.rs`) |
| `libnx_over.nro` plus `sd:/atmosphere/contents/00FF0000A11CE0FF` | Console overclock. Missing service shows error 70 (`render/mod.rs:54-66`) |
| `ssbusync_*` symbols | `mark_arena_mode_for_ssbusync` (`net/mod.rs:330-334`) and the per-frame enforcer (`:358-361`) |
| imgui-smash | Fork loads it through `imgui-api`. Rewrite uses a compatible standalone host when present, otherwise its embedded archive (`docs/IMGUI_DEPENDENCY.md`). Both standalone load orders passed the isolated Eden checks |

The rewrite does not print PIA `Connected Station` lines. Those come from the
original manager/fork callback path. Absence of that log is not stealth.

## Fork features that are registered

Defaults when `sd:/ultimate/ssbu_online_deluxe/config.toml` is missing
(`render/mod.rs:27-34`, `render/profile.rs:48-70`, `167-178`):

- `overclocker = true` (applied only on console; skipped on emulator)
- `stealth_mode = false`
- menu, offline singles, offline doubles: `Vanilla`
- online singles: `LessLag` on console, `LessLagUltra` on emulator
- online doubles: `LessLagDoubles`
- Auto mode starts on (`profile.rs:330`)
- A trailing `+` on a profile name enables FPS boost only when `is_emulator()` (`profile.rs:142-146`)

Profile bits the fork pushes through `sync_guest` (`profile.rs:203-240`, apply at `:491-522`):

| Preset | Buffer | Index | VSync | Render opts | Dynamic res | Default res |
|---|---|---|---|---|---|---|
| Vanilla | Triple | TwoBehind | on | off | off | 1920x1080 |
| LessLag | Double | OneBehind | off | on | on console, off emulator | 1920x1080 |
| LLUltra | Double | Immediate | off | on | on console, off emulator | 1024x576 console, 1080 emulator |
| LLDoubles | Triple | OneBehind | off | on | on | 1920x1080 |

Unspecified FPS-boost on a recommended profile inherits the previous selection
(`profile.rs:467-472`).

Selection UI (`ui/overlay/mod.rs:217-242`, `profile.rs:404-452`):

- Window chord: ZL+ZR+Down or L+R+Down (`ui/mod.rs:10-14`). Minus or Plus held
  with that chord opens the debug view (`overlay/mod.rs:581-587`).
- Configurable rows are NetLatency and NetProfile only.
- Left/Right change the selected row only when online and not in a connected
  real match (`allow_interact`, `overlay/mod.rs:222-224`).
- NetProfile cycle while manual: Vanilla, LessLag, LLUltra, LLDoubles, then
  Auto. From Auto, either direction drops to manual Vanilla. Boost chord
  L+R+ZL/ZR+X toggles FPS boost and is stored only on emulator (`profile.rs:410-417`, `:351-355`).
- Auto reapplies on station connect (`net/pia/mod.rs:140-141`, prints
  `AUTO SELECT RP`) and at match init (`profile.rs:546-563`). Match cleanup
  applies the menu profile (`:581-588`). NRO load applies the menu profile
  (`:591-597`). Each overlay frame reapplies if live flags drift (`:569-578`).
- Overclock, if the service loaded, uses `DockedProfile::Rest` outside a match
  and `DockedProfile::Singles` for Vanilla, LessLag, LLUltra, and LLDoubles
  (`profile.rs:524-541`). `catch_unwind` is around that call. The rewrite uses
  `panic = "abort"`, so that wrapper is not portable.

Opponent ping (`ui/overlay/mod.rs:309-321`, `ui/native/mod.rs:19-29`):

- Display is `StationConnectionManager` RTT plus Wifi `[Wifi]`, Ethernet
  `[Wired]`, or Emulated `[Emu]`. Color follows `NetworkStability`.
- Missing RTT prints `---`. This crate does not measure RTT.

Stealth (`render/mod.rs:37-38`, `net/pia/mod.rs:193-203`, `manager_stealth.rs:1-29`):

- Read once at `pia::install`. Not a menu row.
- True: do not register the 4-byte custom send hook; still register receive;
  scan the manager NRO and replace `mov w8, #0x45` with `mov w8, #0`.
- Incoming version `0` is ignored, same as a vanilla peer (`pia/mod.rs:162-167`).
- Versions 3 and 2 are parsed; version 2 clears boost bits 14 and 15 (`:101-113`).

Input buffer (`net/latency_slider/mod.rs:8-10`, `:146-183`):

- Hook `0x16ccab8`. Null pointer returns. Auto (`-1`) leaves the game byte.
  `0..=25` overwrites it. Active value is copied from selected at that hook.
  Match cleanup sets active to unknown (`i8::MIN`).
- The same left/right poll also runs on the local-wireless pane unless the
  overlay is interactable or streamer mode is on (`ui/native/mod.rs:221-238`).

Match class (`net/mod.rs`, all installed at `:374-390`):

- Quickplay: `0x22da160`, `0x22da090`. Arena menu: `0x22d9fa0`. Arena room:
  `0x1887b1c` / `0x18881f0`. Local wireless: `0x1bd45e0` stores `OnlineLocal`
  and does not become Quickplay.
- Main menu `0x235aaa0` resets to Offline unless status is already Quickplay.
- Stage setup `0x25d9288`: training or stage 311 is Training; more than two
  fighters is Doubles; result stages 310 and 354 clear the match.
- If a singles/doubles match is still Offline, the fork prints
  `UNTRACKED ONLINE MATCH DETECTED, ASSUMING QUICKPLAY` and forces Quickplay
  (`:223-231`). The rewrite does not do this.
- LDN host forces the CSS pane count to 2 once (`0x1a26200`, `:155-166`).
- LDN initialize/scan hooks only count networks (`net/ldn/mod.rs:41-42`).
- CSS `0x1a12f60` writes the native banner when online and the 300-frame
  transition grace has expired (`:46-66`, `:169-183`).

Native panes and streamer mode are live. `ui/native/mod.rs` `install()` is
commented (`:381-382`); the menu hooks above still call
`update_css_ui`, `update_local_online_ui`, and `update_online_arena_ui`.
Streamer mode is L+R+ZL/ZR+Y and replaces latency/ping text with the LDN
network count (`native/mod.rs:47-58`, `:221-228`). Station cycle is
ZL+ZR+Left/Right or L+R+Left/Right (`:32-44`).

Performance scaler (`perf_scaler/mod.rs:53-55`), called from match init (`:40-46`):

- Critical-hit cut-in hook `0x06a7100` plus a fighter main-line callback push
  and pop a dynamic-res report (`common/cut_in.rs:69-111`).
- Sephiroth/Edge fully charged shot does the same (`fighter/sephiroth.rs:40-60`).
- Both call `sync_guest`; there is no in-tree scaler math.

`dummy_connection` is off unless that Cargo feature is passed
(`net/pia/mod.rs:186-206`). A default build does not register it.

`ssbu_online_deluxe_set_render_config` (`render/mod.rs:41-51`) stores a new
config and reloads the overclocker. It does not itself apply a profile; the
next auto-select reads the stored config.

## Rewrite working tree, 2026-10-06

Startup (`src/runtime/mod.rs`) installs renderer, menu, FX, buffer, and Pia RTT, each skippable
with `sd:/ultimate/ssbu_online_deluxe_rewrite/DISABLE_<NAME>`: renderer, menu,
FX, buffer, Pia RTT. An existing `ssbusync_status` or `ssbusync_restrict_mark_arena_mode`
symbol skips every hook (`:109-112`, `:134-147`). Display version must be
`13.0.5`.

| Fork behavior | Rewrite | Evidence |
|---|---|---|
| Vanilla render | Completed. Non-vanilla patches are not installed | `runtime/renderer.rs:121-124` |
| LessLag / LLUltra startup modes | Completed as in-tree text patches, not `sync_guest`. Startup only. Status text says restart to change (`renderer.rs:29`) | `config.rs:116-118`; install stores `REQUEST` once (`renderer.rs:125-126`); `lesslag` adds render-opts sequencing; `llultra` adds immediate flush at `0x38701e0` and nops `0x3749b78` |
| `lesslag_core`, `doubles_core` | Partial isolation modes. Core skips render opts. `doubles_core` is 3 textures and is not LLDoubles | `config.rs:100-108` |
| Selectable NetProfile, Auto, match/menu apply, drift reapply | Missing. Menu edits FX and input buffer only (`menu.rs:43-56`). UI prints the boot mode (`runtime/ui.rs:220`) | `policy.rs:1-2` |
| Online singles/doubles defaults | Missing at runtime. The unused policy specifies console LessLag / emulator LLUltra / online doubles LLDoubles. Its Local case stays Vanilla, unlike the fork's OnlineLocal path; correct that specification before runtime integration (`policy.rs:26-37`) | Not called from `src/runtime` |
| Opponent ping | One fresh non-local station. The cell is the integer mean of that station's 100-sample ring, coloured Stable / Inconsistent / Unstable. You stays `---`. A second fresh station, another live protocol, or a stale sample stays uncolored `---`. No interface suffix, account name, or peer NetLatency / NetProfile | `pia_rtt.rs`, `runtime/ui.rs` |
| Stealth | Missing. No PIA send path exists to suppress. Full profile selection is still pending | no `pia` module |
| Input buffer Auto/0–25 | Completed and live. Same offset and null guard. Auto keeps the game byte | `runtime/hooks.rs:34-52`, `policy.rs:42-47` |
| Peer latency broadcast and active-vs-selected display | Missing | no station packet |
| Quickplay/Arena/Local class for profiles | Partial. The same five scene offsets only set an FX-online flag. Main menu always clears it, including during Quickplay | `runtime/online.rs:1-4`, `:40-63` |
| Offline match guessed as Quickplay | Not copied, on purpose, for FX. Do not revive it for Auto profiles unless that guess is explicitly wanted | `online.rs` comment vs `net/mod.rs:223-231` |
| LDN CSS pane force, native CSS/arena/room text, streamer mode, station cycle | Missing. Native pane writes were removed after font/clipping failures | `STATUS.md` statement; no calls in `src/runtime` |
| 300-frame CSS grace | Missing | fork `net/mod.rs:46-66` |
| Dynamic resolution, cut-in, Sephiroth scaler | Missing | no `perf_scaler` |
| FPS boost `+`, console clocks | Missing. Fork ultra/doubles still request the Singles docked profile | `profile.rs:524-537` |
| ssbusync arena mark | Not ported. Rewrite refuses to load beside the host | `runtime/mod.rs:109-112` |
| ImGui Player Info, FPS/frame graph | Completed. Standalone `libimgui_smash.nro` is used when its host exports match ImGui 19263; otherwise the embedded backend in this NRO. FPS is overlay frame interval, not network RTT | `runtime/overlay_host.rs`, `runtime/ui.rs:156-159` |
| FX On/Off, online scope | Rewrite-owned, not a fork feature. Live, and FX install also installs the scene flag | `runtime/effects.rs:339-340` |
| FX Reduced | Rejected by the parser. `FxMode::Reduced` exists and `effective()` turns it back into On. Not a registered mode | `config.rs:162-168`, `:24-28` |

`render_mode` in the INI is read at startup. Publishing a new config from the
menu (`runtime/ui.rs:163-164`) updates the atomic, but the renderer does not
reinstall patches. Changing the INI between matches does nothing until the
game process starts again.

Operator correction, not retested here: the rewrite's LessLag path has no known
menu stall. The stall belonged to original 1.4.1 menu-profile application.
Booting the rewrite with `render_mode=lesslag` still installs the match patches
before the menu, because install runs from `main`.

## Implementation order

Stay in this NRO. Do not add a second Deluxe plugin. Leave TM's effect hooks
chained and leave `0x374812c` unpatched.

1. Singles NetProfiles at a match boundary: manual Vanilla, LessLag, and
   LLUltra, plus Auto. Auto for a singles online match is LessLag on console
   and LLUltra on emulator, including connected local wireless (the fork's
   `is_valid_online_mode` includes `OnlineLocal`). Menu, training and offline
   stay Vanilla. The unused policy's Local case currently differs.
   Apply when the match becomes real, and return to Vanilla when it ends.
   Do not hot-edit the INI and do not reinstall hooks mid-frame. The current
   patches are process-lifetime text edits, so the transition design has to
   define a safe point (Fable is drafting that separately in
   `audittemp/fable-netprofiles-ping-design.md`). Until that boundary exists,
   a profile chosen in the CSS is not a feature.
2. Show the selected and the active profile in the existing Player Info row.
   Wire Left/Right only outside a connected match, matching `allow_interact`.
3. Opponent ping is the read-only Pia observer in the existing row. It is not
   `ssbu_pia_interface`. The fork-style stability colors are implemented;
   Wifi/Ethernet/Emu suffixes still need validated peer metadata.
   Several fresh peers, or a second live protocol, stay `unavailable` rather
   than picking a spectator. Stealth is still not a menu toggle.
4. Stealth as a startup bool, default false. It is meaningful only after a
   custom send hook exists. The original fork patches the separate manager's
   beacon. This one-NRO rewrite must instead own beacon generation and suppress
   its own beacon/custom sends when Stealth is enabled. Receive stays on.
   Do not describe "we send no PIA packet today" as implemented Stealth.
5. Input-buffer peer display and the CSS/local pane text. The local overwrite
   path already works.
6. After singles are selectable and tested: full LLDoubles (triple, one-behind,
   render opts, dynamic res, doubles detection from fighter count). `doubles_core`
   is not that preset. Reduced FX stays after singles and is independent of the fork.

Console overclock, FPS boost, cut-in/Sephiroth dynamic res, native banner
writes, and the Quickplay-guess fallback wait until the singles boundary is
real. Native pane writes should be reviewed against SGPO and Training Modpack
before they return.

## 2026-10-06 ImGui NVN chain

The first normal-order run showed menus but did not prove this NRO's NVN
bootstrap chain: both plugins installed one shared `nvn_bootstrap_hook`
address. Reversed order showed neither UI. Hook callbacks now use a
`deluxe_rewrite_` export prefix and local function binding; bootstrap install
also rejects a replacement address outside this NRO before `install_hook`.
Candidate `defda6fd` passed embedded, standalone-first, and standalone-last
Eden checks. Both standalone cases showed distinct hook addresses, nonzero
rewrite NVN observations, and both menus. See `VALIDATION.md` for artifacts
and limits. This does not establish Switch or Elite stage-filter behavior.

## 2026-10-06 RTT colour limits

The opponent ping row is the closed manager's 100-sample integer mean and the
thresholds ≤25 Stable, ≤100 Inconsistent, otherwise Unstable. Warmup divides
by the initialized count; a full ring divides by 99. The manager's warmup loop
also includes one unwritten zero. This rewrite sums only initialized samples,
so a steady value is Stable from the first sample. That is a deliberate
correction, not a claim that the closed NRO does the same.

Still missing, so the cells stay empty or `---`: account name, interface
suffix, peer NetLatency, and peer NetProfile. Full Stealth and selectable
NetProfiles are still pending. Several fresh stations stay unavailable rather
than an invented aggregate. Native CSS and arena banners are not reinstated.

## Experimental peer metadata diagnostics

`peer_observer=true` opts into two guarded, read-only hooks after startup.
The default is false. This is diagnostic logging only: it does not populate
names or interface cells, send packets, or implement Stealth. The optional
observer depends on the installed Pia lifecycle observers. Its exact artifact
and validation status belong in `VALIDATION.md`.

Source review of the original manager corrected two earlier hypotheses:
the wire constant ID is `Station+0x40`, not the manager's local sequence ID;
names are in subtype-2 response entries, not the subtype-1 prefix. The game
parser inherits omitted header fields, so a raw parser with absent fields
cannot substitute for the game's parsed object. Whether both LDN peers receive
usable subtype-2 responses still needs a captured trace.

## Tests to add

Existing host tests cover INI parse, the unused `select_profile` table, menu
FX/buffer edits, and `apply_input_buffer`. They do not prove a running mode.

- A profile published after `renderer::install` does not change `renderer::mode()`
  or the installed patch set.
- A defined match-start boundary applies Vanilla, LessLag, or LLUltra and the
  match-end boundary returns to Vanilla. Training stays Vanilla; connected
  local wireless uses the online recommendation, matching the fork.
- Emulator Auto singles resolve to LLUltra; console Auto singles resolve to
  LessLag. Doubles assertions stay pending until step 6.
- Buffer `auto` leaves a non-zero game byte; `0` and `25` replace it; a null
  pointer does not write.
- Ping formatting: a fresh mean is `{mean}ms` in the opponent cell and
  `{mean} ms` on the performance row. Empty and stale are `---` with no colour.
  Wifi/Ethernet/Emu suffixes stay absent until interface metadata exists.
- Stealth predicate: send registration is off, receive registration is on.
  Do not patch a manager binary in the test.
- Coexistence: ultra install does not write `0x374812c`; FX filters call
  through; no `Layout::Draw` install is added.
