# Fork feature parity

Source of record: `reference/quickplay` at `35fdf7cf` (see `reference/README.md`).
Current display candidate `4a927dc20904722d`, reviewed 2026-10-07.
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

## Rewrite implementation and evidence — 2026-10-07

The runtime calls `auto_policy` and the scene-driven `auto_driver`; the profile
matrix is no longer an unused specification. Defaults are Auto, Console, FX On
with online-only scope, Stealth On and Streamer Off. Explicit saved settings are
retained. Auto's target and the renderer's committed state are distinct.

| Fork behavior | Rewrite | Evidence / remaining boundary |
| --- | --- | --- |
| Vanilla, LessLag, LLUltra | One installed hook set, live guarded transitions, confirmed native buffer counts; Less/Ultra include render sequencing | Historical `ed5d12f8` linked matches and transitions; final combined result below |
| LLDoubles | Three textures, OneBehind, render sequencing and render-finished bypass; no Ultra early flush | b19 four-fighter linked match completed through results/CSS; A-only G&W fix versus clean B |
| Auto / match / menu policy | Console online singles LessLag, emulator singles LLUltra, online >2 fighters LLDoubles; menus/offline/Training Vanilla | Training/menu Vanilla, b19 linked doubles LLDoubles, d9 linked singles LLUltra and cleanup Vanilla passed |
| Manual profiles | Saved and menu-selectable, including during play; match cleanup requests Vanilla until the next match or a new explicit choice | Deliberate extension beyond the fork's connected-match edit restriction |
| Core experiments | INI-only `lesslag_core` / `doubles_core`, startup-only, no render sequencing | Retained diagnostics, not full presets |
| Input buffer | Auto or 0–25 at the original hook, including the Quickplay null guard; future setup calls apply edits | Preserved game hook; separate from rendering latency |
| RTT and stability colors | Selected station's game RTT, fork-style mean and colors; freshness/session identity checked before display | Accepted host/identity review; earlier local LDN displayed colored RTT |
| Names / peer settings | Validated connection-ID handshake and response entries; fork v2/v3 receive, v3 transmit only to confirmed modded peers | Earlier `410ac1f7` and original fork exchanged names, interfaces and Vanilla/Less/Ultra at menus/CSS; original fork aborted loading its fighter, so that was not a completed match |
| Interface | Local nifm Wi-Fi/Ethernet/Unknown; selected peer's announced type or Unknown | No RTT-based or text-base-based physical-interface inference |
| Stealth | Default On; suppress owned beacon and custom payload, retain receive; latched per connection, menu changes persist and show reconnect notice | Host-tested; Off interoperability and pending-On behavior observed on `410ac1f7`; fresh-On d9 suppressed its beacon with zero custom writes; saved Off stayed pending until reconnect |
| Peer cycling | Bounded station selection using one identity snapshot for name/interface/settings/ping | Host-tested; >2 network-station live test requires more than two local players |
| Streamer | Menu and fork chord; hides all remote rows and ping, persists to INI | d9 exact menu-saved INI reload and menu/chord privacy passed |
| Connection / match classification | Quickplay, Arena and Local stay distinct; fighter count classifies doubles; Training/result-stage rules retained | Stage/context guards required; does not guess an offline match is Quickplay |
| Resolution and DRS | Committed profile plus explicit platform table; guarded resolution function/register/crop path | Console-policy Eden full-Gigaflare scaling and restoration passed for Less/Ultra/Doubles; Vanilla stays off; b19 title fullscreen-crop reset fixed |
| Cut-in and Sephiroth Gigaflare scaler | Bounded visual resolution reports driven by guarded fighter Main and cut-in sites | Two approximate callback-count gates and Gigaflare trigger passed; Luigi Up-B cut-in scaling/restoration passed |
| Player Info / performance view | Global top-left ImGui table, peer/local columns, frame graph and transition status | Main/CSS/Training window confirmed with combined candidate and standalone host |
| Standalone ImGui | Reuse compatible host, embedded fallback otherwise | Embedded and both standalone load orders tested previously; Elite coexistence observed |
| Native CSS / Arena / LDN text | Fork CSS and arena display paths reinstated alongside the global window | CSS retains selected values after window closure; Streamer restores original styles. Arena uses the selected peer, not the full station list; NSO test pending. LDN lobby network-count replacement and native D-pad editing are not ported. [Details](NATIVE_DISPLAY.md) |
| 300-frame native-banner grace / LDN two-pane force | Not copied | Scene generations and validated pane identities replace retained unguarded pointers; no forced controller slots |
| SsbuSync arena marker | Not needed by the in-tree backend | Legacy host exports trigger a best-effort overlap refusal; do not install both stacks |
| FX On/Off and scope | Added by this rewrite, not inherited from the fork | Named visual requests only; user-confirmed Palutena/Sephiroth and TM/SGPO coexistence on prior candidates; full expanded catalog not runtime-certified |
| Reduced FX | Unavailable | Never silently converted to Off |
| Hardware overclocking | Deliberately excluded by user request | No clock management or nx_over dependency |
| Software FPS boost `+` | Not implemented | New live watchpoint identifies native-window swap interval and Eden's composition-speed extension; [mechanism documented](FPS_BOOST_INVESTIGATION.md), boost A/B measurements pending |

The renderer drains outstanding work before native buffer-count changes. It
confirms count/index state before Active. Missing drain markers retain a safe
wait; invalid post-mutation readback stops with diagnostics. Transitions never
reinstall hooks and never patch SGPO's `0x374812c` neighborhood.

## Deliberate differences

- Manual modes remain usable in Training for measurements. Auto uses Vanilla
  there. Main-menu entry clears stale connection context, and unknown offline
  matches are not relabeled Quickplay.
- Platform is an explicit setting. A nonstandard emulator address layout cannot
  silently choose console defaults.
- RTT warmup uses initialized samples, avoiding the original manager's unwritten
  zero. A full ring retains the fork's mean behavior and stability thresholds.
- Streamer is persistent display privacy; it does not alter packets or replace
  remote rows with the LDN network count.
- The independent window edits settings; the fork-style native title displays
  them while the window is closed. Original styles are captured once per pane
  identity and restored by Streamer. Ordinary game input remains unsuppressed.
- No hardware overclocking, unproven software FPS boost, or duplicate external
  backend is introduced.

## Dependency and compatibility boundaries

Skyline is required. The rewrite contains the sync/render logic and ImGui
fallback; it does not require an external SsbuSync host, PIA manager or Smashline.
Other mods may still need their own dependencies. Do not remove them blindly.

The compatible standalone ImGui host is discovered by exports and ABI, not by
filename. Namespaced hook callbacks and local function binding prevent the
two NVN bootstraps from resolving to the same replacement. Embedded, normal
standalone order and reversed order were tested on `defda6fd`; both menus and
distinct hook addresses were observed. See [dependency details](IMGUI_DEPENDENCY.md).

Elite Legal Stages plus standalone ImGui, TM, corrected SGPO and the rewrite
completed two ordinary offline matches on `ed5d12f8`. Both menus and the overlay
were visible during play; Elite settings persisted. Offline Random remained
unrestricted as intended. This does not test Elite's **NSO-only** filtering.

The guarded fighter-Main entry must match 13.0.5. A pre-existing branch/hook
refuses installation, so arbitrary Smashline stacks are not certified. In
count-only mode no cut-in hook or scaling reports are installed. Two Training
sessions passed approximate tick/present frequency gates; this is not exact
per-frame or module-unload proof. Normal mode rejects nonfighter weapon agents
before reading fighter state. Missing triggers are shown as unavailable.

`peer_observer=true` enables diagnostic observations. The actual handshake and
station map no longer depend on that diagnostic toggle; disabling logging does
not disable normal peer functionality. Transport hooks only annotate the
fork-compatible fields of validated traffic; they do not originate game inputs.

## Validation coverage

Current `4a927dc20904722d` passes 192 tests and required checks. The native CSS
display retains edits when Player Info is closed, restores the original title
under Streamer, and shows linked names/profiles/colored RTT. Solo standalone
Deluxe/Elite coexistence passed with and without the G&W fix. One concurrent
run had invisible external windows on the full-stack peer while native text
and the other peer's embedded window worked; that cause is unresolved.
Arena NSO and Switch validation remain pending. See
[NATIVE_DISPLAY.md](NATIVE_DISPLAY.md) for build-specific evidence.

Candidate `b19b7d94ae136e55` passed 175 host tests and all required host/Skyline
checks. It fixes the observed title crop reset and passed 24 controlled response
trials plus a real cut-in scaling/recovery test. A four-fighter linked team
match used Auto/LLDoubles, A-only G&W fix and clean B; observed damage, stocks,
results and return to CSS agreed.

A second b19 pair used the rewrite on both sides, external ImGui/full mod stack
on A and embedded ImGui on B. Names, green RTT, interfaces, profile changes,
Streamer menu/chord and Stealth Off-to-pending-On behaved as expected. An actual
G&W-versus-Sephiroth match (A LLUltra, B LessLag, fix only on A) completed through
results and stage select. A reached its configured runtime deadline during the
next CSS load, so that return is not certified. A discovered remote-buffer
placeholder bug is fixed in candidate `d9acd03611e1d1d6`.

The d9 candidate passed 176 host tests and all required builds/checks. Its
four changed source files contain display logic and tests only; rendering,
input and packet-writing algorithms are unchanged from b19. It passed exact
menu-saved INI reload, Streamer privacy, fresh Stealth On with a suppressed
beacon and zero custom writes, pending Off, and custom exchange after restart.
The remote buffer now displays Auto or the announced numeric value correctly.

A final d9 G&W-versus-Sephiroth match used A LessLag plus the G&W fix and B
Auto/LLUltra without that fix. Observed damage, stocks, winner, results and
return to CSS agreed. Elite was enabled on A with standalone ImGui, TM and
corrected SGPO; B used embedded ImGui. Both returned to committed Vanilla
after cleanup. These tests used two private localhost stations, not a VPN,
NSO or four network stations. All d9 frontends exited normally without OOM;
the isolated profiles were restored.

Earlier `ed5d12f8` tests completed asymmetric G&W-fix matches in both LessLag and
LLUltra against a clean peer, with matching damage/stocks/results and return to
character select. These are bounded observations, not simulation checksums,
full move coverage, NSO approval or a four-console/VPN test.

The [frame methodology](LATENCY_METHODOLOGY.md) and [performance record](PERFORMANCE.md)
keep the historical measured 3/4 captured-frame reduction tied to its exact
`123d47db` binary. Compatibility tests of later artifacts do not transfer that
measurement automatically. [VALIDATION.md](VALIDATION.md) records each build and
the remaining Switch tests.
