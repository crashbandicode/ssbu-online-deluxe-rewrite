# Fork feature parity

Source of record for the original fork: `reference/quickplay` at `35fdf7cf`
(see `reference/README.md`). A feature is listed only when an `install()`
path or an exported function actually registers it. `docs/STATUS.md` is a
progress note, not this inventory.

`src/policy.rs` `select_profile` is the Auto scene table. `src/auto_policy.rs`
`decide` calls it when Auto is on and the scene is ready
(`src/runtime/auto_driver.rs` `snapshot`). Manual mode keeps the saved
`RenderMode`, except that after a match or Training ends the cleanup hold
requests Vanilla on the inactive scene until the next match or a newer
selection (shown as `Vanilla*`). The table tests are the selector specification; a completed
match is still required before treating a profile as runtime-accepted.

The historical diagnostic build
`c0e9a625eb44a735` passed 247 host tests, host Clippy after a hex-grouping
correction, the no-default build, and Skyline Clippy and packaging. That
package records flush-field widths and incoming-submit membership during a
finite API trace. It is not a rendering or menu-visibility change, and that
invisible-window reproduction was pending at that checkpoint. The Switch then had
`b99f087cf8ef0a9c` installed. Its historical runtime limits, including Reduced branches in
the FX runtime matrix (now `docs/FX_RUNTIME_MATRIX.md` in ssbu-fx-control), remain open.
Historical frame counts stay on the binary that produced them.

The rewrite is one NRO. Keep Training Modpack and SGPO coexistence: no
`Layout::Draw`, no effect hooks since 0.5.0 (they called `original!` until
then), and `llultra` reads SGPO's word at
`0x374812c` without patching it (`src/runtime/render_ultra.rs`).

## Rewrite changes 0.5.3 (2026-10-17): plugin API host and window pages consolidated

Host-tested only; the behaviour changes a plugin or a player can see:

1. Descriptor version 1 is refused with `REGISTER_ERR_ABI` (no known sender;
   only unreleased 2026-10-12 SDK builds sent it).
2. Descriptor versions 2 and 3 are validated under the version 4 rules (every
   flag bit, 64 options), which is strictly more permissive. The replay
   plugin (version 2, five rows) and FX Control 0.1.x copies (version 3) keep
   their rows unchanged.
3. `OPTION_FLAG_INLINE` rows draw as ordinary value rows; the SDK's
   `RowLayout::Inline` is gone (no plugin sends it; FX Control uses value
   rows since its strip scrolled off the console pane).
4. The shared Plugins page has a real group row per plugin: the vertical
   table is unchanged, but the in-match strip now shows the plugin label as a
   header segment followed by the plain option labels, where it showed
   `Plugin.Option` labels (or the plugin label alone for a one-option plugin).
5. On the shared page the selection index counts the group rows (row 0 is
   the first plugin's label, the first option is row 1); Up/Down skip them.
6. `Registry::value` is a pure getter; a pending request is cleared and
   persisted by `reconcile_pending`, which the overlay runs before every
   draw. A plugin applying a pending change between that call and the draw
   shows `(pending)` one frame longer.
7. The SDK (5.0.0) sends descriptor version 4 only and calls the host once;
   `negotiated_descriptor_version` and the version 3/2/1 retries are gone, and
   an old host answers `RegisterError::Abi` as before.
8. Code 99 (`PluginRegistered`) logs one line per registration (`a` handle or
   negative code, `b` option count, as before); the two separate accepted /
   refused prints became one.

Deluxe's own Settings, Performance and Debug drawing kept its text: the
Settings and plugin tables share one drawer, the strip fields share
`menu::strip_segments`, the Performance table and strip read their facts from
one `perf_facts` list, and the window frame is drawn by `draw_window`.

## Rewrite changes 0.5.0 (2026-10-14)

- Character FX (On / Off / Reduced and its scope) moved to the separate
  ssbu-fx-control 0.1.0 plugin. Deluxe has no effect hooks, no FX row and no
  FX keys; it ignores `effects` and `effects_scope` in an older INI. The
  scene tracker stays and is exported as the online context.
- Plugin API descriptor version 3: own pages in the window cycle, up to 32
  options, and the online-context export.

## Rewrite changes 0.3.0–0.4.2 (2026-10-13/14)

All are rewrite extensions or fixes with no fork counterpart. Per-release
evidence is in [VALIDATION.md](VALIDATION.md).

- Two-image LessLag present-at-flush (0.4.0): the acquired image is presented
  at the flush that submits it and the game's later present of that index is
  skipped (`render_policy::present_at_flush`; `early_presents`,
  `early_present_skips` and `early_present_mismatches` in renderer-status.txt;
  code 105 records mismatches).
- `Vanilla*` banner label under the cleanup hold (0.4.0).
- Opt-in `CAPTURE_MATCH` match capture (0.4.0): streamed timing and API CSVs
  for a whole match, ended by `scene_end` (`src/runtime/capture.rs` since
  2026-10-17, formerly `match_capture.rs`; code 106).
- ZL-gated native D-pad edits (0.4.1, `src/overlay.rs`).
- An unbound station leave retires the only confirmed peer (0.4.1,
  `peer_transport::retire_only_confirmed`).
- `Scene::marked_online` for Auto (0.4.1): a stage the online-latency hook
  reached counts as online.
- Main-menu entry keeps a stored Quickplay (no liveness check), and the
  diagnostics poll demotes a Singles/Doubles stage that has run five seconds
  under a stored Quickplay without the online-latency mark to Offline, which
  does rewrite the stored connection (0.4.2,
  `auto_policy::main_menu_connection`, `offline_match_due`, code 107).

## Earlier rewrite changes — 2026-10-12 (plugin value tones and read-only rows)

Candidate `e59fc9a6bd266790` adds descriptor version 2 to the plugin API:
an optional per-choice tone (neutral, green, yellow, red) that the host uses
for the current value's text in both the vertical Plugins table and the
in-match strip, and `OPTION_FLAG_READ_ONLY` for multi-choice status rows that
Left/Right never edit and whose value simply follows `current`. The exported
table and symbol are unchanged (table version 1), the host still accepts
version 1 descriptors, and the Rust SDK falls back to version 1 when an older
host answers the ABI error. Rendering outside the Plugins page is untouched.
282 host tests plus the SDK round-trip pass with the usual Clippy, no-default,
rustfmt and Skyline release gates. Emulator verification of the colours is
recorded in VALIDATION.md; no hardware run of this candidate yet.

## Earlier rewrite changes — 2026-10-12 (plugin registration API, host-tested)

Candidate `068a1cf6f418349b` adds a public registration API so separately
built Skyline NROs can add rows without ImGui code. This is a rewrite
extension with no fork counterpart. The host exports
`ssbu_online_deluxe_plugin_api_v1`; a consumer resolves it with
`nn::ro::LookupSymbol` (never a link-time import) and registers stable ids,
display labels, finite choice lists and `current`/`request` callbacks. The ABI
crate `sdk/abi` is shared by the host, the Rust SDK `sdk/sdk` and the demo NRO
`sdk/examples/demo`. Strings are copied at registration; `config.ini` is
unchanged. Persistence is plugin-owned, with an opt-in: a plugin that supplies
its own `sd:/` INI path gets the option's `key=value` line rewritten by the
host's background thread after an `APPLIED`/`PENDING` answer, atomically with
a `.previous.ini` backup; the host never reads that file (diagnostic 101).

A Plugins view (`View::Plugins = 4`, appended after Debug so recorded view
numbers hold) joins the chord cycle only after a registration:
settings → plugins → performance → hidden, with Plus/Minus still opening
Debug (since 0.6.0 the cycle ends performance → window options → hidden, see the
Player Info ledger row below). Outside a match it is a vertical table with a header row per plugin;
in a ready Singles/Doubles match it uses the approved compact two-row strip
unchanged in alpha, padding, content-fit background and left-edge rule, paging
four fields per row with a `Plugins p/N` indicator instead of shrinking text.
Settings, Performance and Debug rendering are untouched. New diagnostics 99
(registration) and 100 (request outcome).

280 host tests plus a new SDK round-trip integration test pass; host and
Skyline Clippy with warnings denied, the no-default build, rustfmt and the
optimized Skyline release build pass. The demo NRO builds and shows no
undefined import of the host symbol. The export is present in the packaged
ELF/NRO. Eden preview `plugins-page-1012a` (candidate `9d08a711b271e0ac`)
then showed the separately built demo NRO registered on the Plugins page, the
chord cycle including Plugins, Right/Left edits applied, and host persistence
writing the demo's INI (details in VALIDATION.md). Eden match preview
`plugins-match-1012b` then showed the compact Settings and Plugins strips in a
live Shulk-versus-CPU match with an in-match edit applied and persisted. Not
yet observed: any hardware run. The replay recorder itself is not part of
this work.
Evidence: `~/.local/share/ssbu-mod-backups/plugin-api-tangent-20261012/`.

## Earlier rewrite changes — 2026-10-10

Content-fit candidate `8328cc88123bd432` follows the operator's feedback on f1ce:
the two-row layout looks good, but the background should not extend across the
screen. Compact Settings/Performance now request zero ImGui window width so the
background fits content. The1840px design bound still determines the existing
left edge and adaptive-text guard; it is no longer the actual background width.
Rows, font logic, timing bar, alpha0.42, padding and outside-match/Debug behavior
are preserved. Only runtime/ui.rs changes behavior; menu.rs differences are comments.

265hosttests, warning-free host/SkylineClippy, no-default and target release builds
pass. All76archived source files match the packaged checkout, only the two reviewed
files differ from immutable f1ce, and native dependencies/Cargo.lock are unchanged.
Parent reviewed actual1080p offline match Settings and Performance captures: two
rows, same left edge, backgrounds fitted to their respective longest row. Exact
startup/build/module and screenshot hashes are bound in the evidence. The owned
preview exited normally0; original profile bytes were restored and rehashed.
The operator approved the screenshots;8328 was installed over foreground DBI FTP
October10 at14:09Z after fresh Smash-closed confirmation, with staged/final SHA
readback. Latest INI, other plugins and HOC/boot bytes were preserved; old4c19 is
backed up locally and outside the SD plugin loader. FTP closed. Hardware visual,
peer-departure/reconnect and export acceptance remain pending. No connected-peer,
720p or performance acceptance is inferred.
Evidence: `~/.local/share/ssbu-mod-backups/hud-content-fit-20261010/`.

### Hardware follow-up (acceptance still open)

The operator clarified that the old opponent name disappears on returning to
fighter select, but the HUD stays right there. Fresh read-only DBI MTP collection
confirms exact installed8328 NRO/build/module. Current logs show session cleanup
and a validated CSS title after the last Training classification without a scene
reset until main-menu entry. No post-cleanup peer-identity defect is established;
the existing peer-retirement behavior is retained.

The follow-up source now observes CSS only after the guarded pane validation.
The existing Training-mode symbol selects Training for genuine Training CSS and
Inactive otherwise; the stored connection is preserved. This closes the missing
scene-status writer without resetting native pane state on every frame. Portable
regressions cover regular CSS placement/menu cleanup and genuine Training CSS.

The operator also tried the full Debug gesture; no Debug transition or new capture
was recorded. New edge-only Code98 diagnostics carry sampled HID buttons, view
transition, controller id/style and command edges. Separate Plus/Minus and both
shoulder/trigger chords have regression coverage. Controls are not rebound and the
hardware failure is not yet diagnosed.

Candidate `6c396f10e8c4c3bb` passes269hosttests, warning-denied host/SkylineClippy,
no-default and optimized target builds. All76 archived files match the checkout;
only the seven reviewed CSS/input-diagnostic source files differ from installed8328.
Cargo.lock/native archive are unchanged, and matching NRO/ELF text and debug sections
are verified. The approved compact HUD layout is unchanged. The operator authorized
installation and confirmed Smash closed/DBI USB MTP ready;6c39 was installed with
exact readback at2026-10-10T22:24:13Z. Fresh15-file before/after checks prove only the
rewrite NRO changed; latest Reduced INI, eight other plugins, five boot/HOC files
and nine plugin names were preserved. Old8328 is backed up locally and outside the
plugin loader on SD. All device transfer processes exited. Hardware placement,
Debug/capture and compatibility acceptance remain pending. Exact hashes, six gate
receipts, source diff and runtime limits are in the candidate artifact directory;
device-validation.json links the new switch-mtp/deployment.json receipt.
Evidence: `~/.local/share/ssbu-mod-backups/peer-ui-followup-20261010/`.

### Earlier two-row candidate

Screenshot-review candidate `f1ce158107e826bf` replaces the rejected grid with two
unwrapped inline rows for ready Singles/Doubles Settings and Performance. Labels
and values share each row; the title/borders/large padding and ordinary match
graph are removed. A ten-segment frametime bar uses the existing real callback
samples. The strip is centered, translucent (alpha0.42), viewport-scaled and
content-height fitted. Outside matches and explicit Debug keep their prior layout.
Platform/FPS++ gates, peer retirement, export, hooks and native archive are preserved.
Only `src/menu.rs` and `src/runtime/ui.rs` differ from4c19.265hosttests, warning-free
host/SkylineClippy, no-default/release and exact76-file packaging checks pass.
Real1080p offline match screenshots now show exactly two inline rows in both views;
parent reviewed the originals and verified exact candidate startup/hash provenance.
The operator accepted the two-row appearance and requested the width change above;
f1ce is not on Switch. No connected-peer/720p
visual or performance/compatibility acceptance is implied. The test profile was
restored byte-for-byte and owned runtime exited normally.
Evidence: `~/.local/share/ssbu-mod-backups/slim-hud-preview-20261009/`.
Images/review: `~/.local/share/ssbu-mod-backups/slim-hud-screenshots-20261009/`.

### Previous installed baseline

Candidate `4c1947b84add6114` retains the buffered-export fix from
`62a42df8d01d1738`. It replaced5c72 on the Switch through authorized DBI USB MTP
at22:32Z with full SHA-256 readback. Previous5c72 is backed up locally and on SD;
the latest INI, eight other plugins and protected HOC/boot files were preserved.
The new menu and peer changes are source/build-validated; visual behavior, the
reported declined-rematch path and hardware export completion are not yet accepted.

- Ready Singles/Doubles use horizontal Settings with six columns, background
  alpha 0.42 and separate wrapped local/opponent values. Training, lobby and
  inactive Settings remain vertical; Performance and Debug remain vertical.
  ImGui fits the actual content height, including the existing frametime plot.
- Platform remains an explicit INI setting, hidden from the menu. FPS++ rows and
  navigation stops are emulator-only; diagnostic and apply gates are retained.
- Confirmed station departure retires that peer's identity and all station aliases;
  explicit kind `0x14` subtype 3/4 disconnects also clear aliases. Other connected
  peers remain available. Vanilla profile selection is not a disconnect signal.
- The transport is installed before the optional `peer_observer` diagnostic toggle.
  Unknown or foreign RTT protocols cannot retire a peer merely because their
  numeric station indices overlap. `src/runtime/pia_rtt.rs` matched 62a exactly
  until the 2026-10-17 cut (`work/pia-rtt-1017`): the station table now holds
  sixteen slots keyed by station index instead of one slot per possible index,
  lives in a plain `static Mutex<Session>` rather than a heap box allocated by
  the first poll, and the UI reads one `Mutex` of the last poll's publication
  and board instead of two seqlocks. Two observable differences: the first RTT
  sample of the process is stored instead of dropped, and more than sixteen
  simultaneously fresh stations evict the one with the oldest sample (arenas
  hold eight). Hooks, epoch and draw-generation gating, staleness, protocol
  adoption and the displayed values are unchanged.

All 261 host tests, warning-free host/Skyline Clippy, no-default and release builds,
and exact packaging checks passed. The final archive matches all 76 source files;
native dependency provenance and buffered-export sources are unchanged from 62a.
The six changed sources and gate receipts are recorded in
`~/.local/share/ssbu-mod-backups/menu-peer-fix-20261009/` and
[VALIDATION.md](VALIDATION.md). No runtime acceptance transfers from older builds.

The operator separately confirmed that the Training Modpack agent tested multiple
people joining an Eden lobby. That update supplied no exact candidate, participant
count or receipt; it is historical joining coverage, not a new run of this candidate
or acceptance of its disconnect/reconnect behavior.

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
| imgui-smash | Fork loads it through `imgui-api`. The rewrite now statically links an archive built from the operator's Forgejo fork `intpa/imgui-smash` (branch `deluxe`, upstream `Coolsonickirby/imgui-smash` at `bb47b205…` plus the renderer correction and memory trims). A compatible standalone host is reused when present; external setups need the matching corrected host. Source/build and Eden acceptance details are in `docs/IMGUI_DEPENDENCY.md` |

The rows above describe what the original fork called. This crate implements
the sync and peer-manager behavior in its own source and does not link
`ultelier`, `ssbusync`, or `libssbu_pia_manager`.

- Sync and render: `src/render_policy.rs` applies live buffer, index, and flush
  plans; `src/runtime/renderer.rs`, `src/runtime/render_sequence.rs`, and
  `src/runtime/render_ultra.rs` own the hooks; `src/runtime/resolution.rs` and
  `src/resolution.rs` own the guarded resolution and dynamic-resolution path.
  There is no `ultelier::sync_guest` call.
- Peer manager: `src/peer_transport.rs` owns the connection-id handshake and
  fork-compatible announcements; `src/runtime/pia_rtt.rs` reads Pia's
  already-computed RTT; `src/runtime/connection_transport.rs` installs the
  parsed-message, station-index and send-site hooks (until 2026-10-17 that file
  was `peer_observer.rs` and also carried the opt-in observer). `src/runtime/mod.rs`
  registers `pia_rtt::install` and `connection_transport::install` directly.
- If `ssbusync_status` or `ssbusync_restrict_mark_arena_mode` is already
  exported, `legacy_host_present` skips the rewrite's hooks. Leave the old
  Online Deluxe, SsbuSync, and PIA-manager plugins out of this stack.

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
  The rewrite reset unconditionally until 0.4.1, which lost the connection
  when background matchmaking passed through the main menu (console
  2026-10-14: Quickplay hook 72 s, main menu 308 s, match 422 s classified
  offline). Since 0.4.2 it matches the fork and adds what the fork lacked: a
  Singles/Doubles stage that runs five seconds under a menu-set Quickplay
  without the online-latency hook is demoted to Offline
  (`auto_policy::main_menu_connection`, `offline_match_due`, code 107).
- Stage setup `0x25d9288`: training or stage 311 is Training; more than two
  fighters is Doubles; result stages 310 and 354 clear the match.
- If a singles/doubles match is still Offline, the fork prints
  `UNTRACKED ONLINE MATCH DETECTED, ASSUMING QUICKPLAY` and forces Quickplay
  (`:223-231`). The rewrite does not relabel such a match: since 0.4.1 Auto
  (and FX until 0.5.0) treat a stage the online-latency hook reached as online without
  rewriting the stored connection (`Scene::marked_online`,
  `effective_connection`). Since 0.4.2 main-menu entry keeps a stored
  Quickplay, and the only write to the stored connection outside the scene
  hooks is the demotion of an unmarked Singles/Doubles stage to Offline five
  seconds after its load (`offline_match_due`).
- LDN host forces the CSS pane count to 2 once (`0x1a26200`, `:155-166`).
- LDN initialize/scan hooks only count networks (`net/ldn/mod.rs:41-42`).
- CSS `0x1a12f60` writes the native banner when online and the 300-frame
  transition grace has expired (`:46-66`, `:169-183`).

Native panes and streamer mode are live. `ui/native/mod.rs` `install()` is
commented (`:381-382`); the menu hooks above still call
`update_css_ui`, `update_local_online_ui`, and `update_online_arena_ui`.
Streamer mode is L+R+ZL/ZR+Y (the rewrite accepts only L+R+ZR+Y since
0.5.7) and replaces latency/ping text with the LDN
network count (`native/mod.rs:47-58`, `:221-228`). Station cycle is
ZL+ZR+Left/Right or L+R+Left/Right (`:32-44`).

Performance scaler (`perf_scaler/mod.rs:53-55`), called from match init (`:40-46`):

- Critical-hit cut-in hook `0x06a7100` plus a fighter main-line callback push
  and pop a dynamic-res report (`common/cut_in.rs:69-111`).
- Sephiroth/Edge fully charged shot does the same (`fighter/sephiroth.rs:40-60`).
- Both call `sync_guest`; the fork snapshot has no scaler math of its own. The rewrite's reports are `src/perf_scaler.rs` and `src/resolution.rs`.

`dummy_connection` is off unless that Cargo feature is passed
(`net/pia/mod.rs:186-206`). A default build does not register it.

`ssbu_online_deluxe_set_render_config` (`render/mod.rs:41-51`) stores a new
config and reloads the overclocker. It does not itself apply a profile; the
next auto-select reads the stored config.

## Rewrite implementation and artifact-linked evidence

The registration column follows the rewrite's `install()` paths. The historical
evidence column retains the named binaries from the 2026-10-07 record. The later
b99 match and c0 diagnostic package are in Validation coverage; the October 9
updates are above. None replaces earlier receipts or moves their frame counts
onto a newer NRO.

The runtime calls `auto_policy` and the scene-driven `auto_driver`; the profile
matrix is the Auto selector used by `decide`. Defaults are Auto, Console,
Stealth On and Streamer Off. Explicit saved settings are
retained. Auto's target and the renderer's committed state are distinct.

| Fork behavior | Rewrite | Evidence / remaining boundary |
| --- | --- | --- |
| Vanilla, LessLag, LLUltra | One installed hook set, live guarded transitions, confirmed native buffer counts; Less/Ultra include render sequencing; two-image LessLag presents at the flush since 0.4.0 (`present_at_flush`) | Historical `ed5d12f8` linked matches and transitions; final combined result below |
| LLDoubles | Three textures, OneBehind, render sequencing and render-finished bypass; no Ultra early flush | b19 four-fighter linked match completed through results/CSS; A-only G&W fix versus clean B |
| Auto / match / menu policy | Console online singles LessLag, emulator singles LLUltra, online >2 fighters LLDoubles; menus/offline/Training Vanilla | Training/menu Vanilla, b19 linked doubles LLDoubles, d9 linked singles LLUltra and cleanup Vanilla passed |
| Manual profiles | Saved and menu-selectable, including during play; match cleanup requests Vanilla until the next match or a new explicit choice | Deliberate extension beyond the fork's connected-match edit restriction |
| Core experiments | INI-only `lesslag_core` / `doubles_core`, startup-only, no render sequencing | Retained diagnostics, not full presets |
| Input buffer | Auto or 0–25 at the original hook, including the Quickplay null guard; future setup calls apply edits | Preserved game hook; separate from rendering latency |
| Auto input-buffer display (0.6.2) | The fork showed only `Auto` (its "active" value mirrored the selection). **Deliberate difference, added by this rewrite:** the hook (`0x16ccab8`, event code 9) reads the game's buffer byte before applying the selection; while Auto is selected that byte is what the game chose, and the label reads `Auto (4f)` (`Auto (25f)` is the longest) on the window's You line, the NetLatency row, the in-match strip's `Net:` field and the native character-select and arena `You:` line (`You: Auto:Van \| Auto (4f)`). The byte is recorded only while Auto is selected (a manual selection overwrites it) and cleared when the connection returns to Offline, so it stays through the results screen and the rematch character select of the same session; before the first online match it reads plain `Auto`. The native You line's latency field is 10 characters (the line cap 38, was 32), remote lines keep 4, and the local-wireless lobby pane and the peer exchange (the `Opp` line) still show plain `Auto` | Not booted on the console; console check is step 16 of the test plan in [NEXT_SESSION.md](NEXT_SESSION.md) |
| RTT and stability colors | Selected station's game RTT, fork-style mean and colors; freshness/session identity checked before display | Accepted host/identity review; earlier local LDN displayed colored RTT |
| Names / peer settings | Validated connection-ID handshake and response entries; fork v2/v3 receive, v3 transmit only to confirmed modded peers. Simplified 2026-10-17 (not a parity loss): the send hook applies the planned beacon and v3 bytes once and no longer saves and reverts them when a cleanup races the store; they are the rewrite's own announcement in its own packet, already addressed to the departing peer | Earlier `410ac1f7` and original fork exchanged names, interfaces and Vanilla/Less/Ultra at menus/CSS; original fork aborted loading its fighter, so that was not a completed match |
| Interface | Local nifm Wi-Fi/Ethernet/Unknown; selected peer's announced type or Unknown | No RTT-based or text-base-based physical-interface inference |
| Stealth | Default On; suppress owned beacon and custom payload, retain receive; latched per connection, menu changes persist and show reconnect notice | Host-tested; Off interoperability and pending-On behavior observed on `410ac1f7`; fresh-On d9 suppressed its beacon with zero custom writes; saved Off stayed pending until reconnect |
| Peer cycling | Bounded station selection using one identity snapshot for name/interface/settings/ping | 9e9 cycled two remote identities correctly in a three-station lobby; no three-station match completed |
| Streamer | Menu and fork chord; hides all remote rows and ping, persists to INI | d9 exact menu-saved INI reload and menu/chord privacy passed |
| Connection / match classification | Quickplay, Arena and Local stay distinct; fighter count classifies doubles; Training/result-stage rules retained | Stage/context guards required. Since 0.4.1 the latency mark changes Auto's effective connection without rewriting the stored one (`Scene::marked_online`); since 0.4.2 main-menu entry keeps a stored Quickplay and the diagnostics poll demotes an unmarked Singles/Doubles stage to Offline after five seconds (`offline_match_due`) |
| Resolution and DRS | Committed profile plus explicit platform table; guarded resolution function/register/crop path | Console-policy Eden full-Gigaflare scaling and restoration passed for Less/Ultra/Doubles while that trigger existed; Vanilla stays off; b19 title fullscreen-crop reset fixed |
| Cut-in scaler | Bounded visual resolution reports driven by the guarded fighter Main and cut-in sites | Two approximate callback-count gates passed; Luigi Up-B cut-in scaling/restoration passed |
| Sephiroth Gigaflare scaler | **Parity deliberately dropped 2026-10-17.** The fork scales on the full-charge shot (`fighter/sephiroth.rs:40-60`); the rewrite carried it until then and removed it | Console measurement did not justify it: Vanilla and LessLag hold 60 through the move on the Switch (2026-10-15 redo), so the trigger only cost a `WorkModule` read per Sephiroth tick. Eden Gigaflare evidence above is history |
| Player Info / performance view | Global ImGui table; connected-only remote column; right in Training/online lobbies, left in ordinary matches | 9e9 remote visibility passed offline/connected/disconnected; a9c right-side Training window confirmed; lobby check pending |
| Player Info main page (0.6.0) | The fork's page is one table (Name, Interface, Ping, ... rows) with an opponent column. **Deliberate difference, added by this rewrite:** the Settings page has two columns and no opponent column: a read-only `You` summary (`<profile> \| <buffer> \| <interface>`), an `Opp` summary while the peer shows (`<name> \| <announced profile> \| <announced buffer> \| <ping> \| <interface>`, unannounced facts left out, ping-stability colour, ZR + Left/Right cycles the station), five rows (NetLatency, NetProfile with the Status folded in, Stealth, Streamer, UltraRes on a console), a one-line performance footer and a dim hint line (`hints`, default true). The page toggles and PerfView moved to a Window Options page, the last page of the chord cycle (performance → window options → hidden); the in-match strip's first line shrank to `Net`, `Profile`, `Stealth`, `Stream`, `UltraRes` and its second line starts with `>Opp:`. The selected value reads `< value >` while ZR is held. The ` \| ` separator is the native character-select banner's | Host-tested, then booted on the console by the operator on 2026-10-19 (0.6.0: "seems to work great. menu is much cleaner"); the page is titled Window Options since 0.6.1. No Eden visual check. The `Opp` line keeps the 0.5.5 liveness rule below |
| Opponent liveness (0.5.5) | The fork showed the opponent while a station was bound. **Deliberate difference, added by this rewrite:** the opponent column (the `Opp` line since 0.6.0), the strip's `Peer:` line (`>Opp:`), the Ping cell, the peer-dependent rows and the native CSS remote line also require an RTT sample for the selected station within 10 s (`pia_rtt::LIVE_MS`; a new binding gets a 10 s grace); event code 112 | Host-tested only. The declined-rematch persistence it targets was reported on the console and not reproduced in Eden; console untested |
| Vanilla+ (0.5.6) | The fork presents Vanilla where the game does, after the next frame's CPU update. **Deliberate difference, added by this rewrite and on by default:** with `vanilla_plus=true` the Vanilla profile keeps the game's three-image ring but presents the acquired image at the flush that carried the frame's last submission (`render_policy::present_at_flush(decision, vanilla_plus)`). Plain **L+R+X** toggles it live (the fork's L+R+ZL/ZR+X boost chord stays inert), the row reads `Vanilla+` / `Auto:Van+`, `early_present=false` disables it, and the peer exchange still announces Vanilla | Host-tested only (237 + 1 tests). Justified by the 0.5.4 console API trace in Vanilla (the acquired image is complete at the flush), not by a console run of 0.5.6; console untested |
| ZR hold for window edits (0.5.3; every scene 0.5.6) | The fork changes the selected row on a plain Left/Right whenever the window is interactable (online and not in a connected real match, `allow_interact`, `overlay/mod.rs:222-224`). **Deliberate difference, added by this rewrite:** with the window open, a Left/Right value change on any Settings row or plugin row needs **ZR held without ZL**. 0.5.3 to 0.5.5 required it only while a fight was live; since 0.5.6 it is required in every scene (operator, 2026-10-18: "no matter what when the menu is open you have to hold ZR to change even outside of training and matches"). A plain press is ignored, Up/Down still move the selection, and the ZL-gated native pane edits with the window closed (0.4.1) are unchanged | Host-tested only (the overlay's `edit_hold` test; the runtime sets the hold in every scene). The live-fight form was exercised on the console on 2026-10-16; the every-scene form is console untested |
| LLUltra base resolution (0.5.3; 1080p 0.5.6) | The fork's console LLUltra renders at a fixed 1024x576 (table above). **Deliberate difference, added by this rewrite:** INI key `ultra_resolution` (`576` the default, `720`, and since 0.5.6 the native `1080`) and the console-only UltraRes row choose the base level; dynamic resolution steps down from it and emulator profiles ignore it. 1080 exists because the operator reports 720p LLUltra held 60 under Horizon OC | Host-tested only. The 720p report is the operator's statement, not a capture; no console capture at 720p or 1080p exists, so no performance acceptance is inferred |
| Standalone ImGui | Reuse compatible host, embedded fallback otherwise | Embedded and both standalone load orders tested previously; Elite coexistence observed |
| Native CSS / Arena / LDN text | Fork CSS and arena display paths reinstated alongside the global window | CSS retains selected values after window closure; Streamer restores original styles once (since 2026-10-17 the restore no longer re-fires on font drift alone). Arena lists up to eight known mapped remote stations (`arena_remotes`); NSO test pending. The lobby pane shows the input buffer and, in Streamer, the game's own captured count text (2026-10-17; the fork's `nn::ldn` Initialize/Scan hooks and bare scan count are not reproduced); native D-pad buffer editing (ZL held, 0.4.1) is implemented; 9e9 Auto-to-0 edit observed. [Details](NATIVE_DISPLAY.md) |
| 300-frame native-banner grace / LDN two-pane force | Not copied | Scene generations and validated pane identities replace retained unguarded pointers; no forced controller slots |
| SsbuSync arena marker | Not needed by the in-tree backend | Legacy host exports trigger a best-effort overlap refusal; do not install both stacks |
| FX On/Off and scope | Moved to ssbu-fx-control 0.1.0 / Deluxe 0.5.0. Added by this rewrite, not inherited from the fork; Deluxe now has no effect hooks and exports only the online context | Through 0.4.2: named visual requests only; user-confirmed Palutena/Sephiroth and TM/SGPO coexistence on prior candidates; full expanded catalog not runtime-certified. Further evidence belongs to the plugin |
| Reduced FX | Moved to ssbu-fx-control 0.1.0 / Deluxe 0.5.0. Was a live visual-layer policy with guarded lighter asset siblings | Through 0.4.2: several visual checks passed; full-family and Off-like performance acceptance pending. Never silently converted to Off |
| Hardware overclocking | Deliberately excluded by user request | No clock management or nx_over dependency |
| Plugin registration API / Plugins page | Added by this rewrite, not inherited from the fork; `src/plugin_api.rs`, `src/runtime/plugin_export.rs`, `sdk/` | Host tests, SDK round-trip, Eden `plugins-page-1012a` (registration, vertical edits, INI persistence) and Eden `plugins-match-1012b` (compact Settings/Plugins strips in a live match, in-match edit persisted); hardware still unverified |
| Software FPS boost `+` | **Parity deliberately dropped 2026-10-17.** Was a diagnostic implementation only; normal play reported Unsupported. The key, row, chord, marker and code 88 are gone; a peer's bit 14 still decodes as `++` | Emulator-only and never applied in normal play; the native-window composition-speed control accelerated game time in a measured match. [Evidence](FPS_BOOST_INVESTIGATION.md) stays as history |
| Peer observer (`peer_observer`) | **Removed 2026-10-17.** Added by this rewrite, not inherited from the fork: an opt-in ring, station map and send probe on the transport hooks (codes 37–45, 64, 65) | Never enabled on any card or Eden profile; the live handshake, station binding and announcements never depended on it |
| Dispatcher backoff | Added by this rewrite, not inherited from the fork; `src/dispatcher_backoff.rs`, `src/runtime/dispatcher_backoff.rs`, one inline hook at `main+0x354ab48`, all profiles and scenes, MainThread only | Eden A/B/A under a 175% CPU quota: CSS LessLag 52 → 59 fps, 18% less CPU/frame; Vanilla/LLDoubles CPU saving at 60; live match unchanged. Hardware benefit pending the Switch test. Same fix ships standalone as `ssbu-dispatcher-backoff`; one owner per site |

The renderer drains outstanding work before native buffer-count changes. It
confirms count/index state before Active. Missing drain markers retain a safe
wait; invalid post-mutation readback stops with diagnostics. Transitions never
reinstall hooks and never patch SGPO's `0x374812c` neighborhood.

## Deliberate differences

- Manual modes remain usable in Training for measurements. Auto uses Vanilla
  there. Main-menu entry clears stale connection context unless Quickplay is
  live (0.4.2). Unknown offline matches are not relabeled Quickplay in the
  stored connection; since 0.4.1 Auto (and FX until 0.5.0) treat a stage the
  online-latency hook reached as online (`Scene::marked_online`,
  `effective_connection`).
- Since 0.5.6 the default Vanilla is Vanilla+: it presents at the flush where
  the fork and the game present after the next CPU update. `vanilla_plus=false`
  (or L+R+X) restores the game's own order.
- Since 0.6.0 the Player Info Settings page is not the fork's table with an
  opponent column: it is the `You` / `Opp` summary lines, five rows, a one-line
  performance footer and a hint line, with the page toggles on a Window Options page
  (see the ledger row above). The fork's native character-select banner is
  unchanged and its ` | ` separator is reused.
- Platform is an explicit INI setting, hidden from the current menu. A nonstandard
  emulator address layout cannot silently choose console defaults.
- RTT warmup uses initialized samples, avoiding the original manager's unwritten
  zero. A full ring retains the fork's mean behavior and stability thresholds.
- Streamer is persistent display privacy and does not alter packets. Native
  lobby privacy restores the game's own count text instead of reproducing the
  fork's LDN scan count (2026-10-17); the small window is a separate path.
- The independent window edits settings; the fork-style native title displays
  them while the window is closed. Original styles are captured once per pane
  identity and restored by Streamer. Ordinary game input remains unsuppressed.
- No hardware overclocking, software FPS boost (removed 2026-10-17), or
  duplicate external backend is introduced.

## Dependency and compatibility boundaries

Skyline is required. Sync, resolution, and the peer-manager handshake live in
this crate, as listed under External dependencies. An exported SsbuSync host
causes `legacy_host_present` to skip these hooks, so the old Online Deluxe,
SsbuSync, and PIA-manager plugins stay out of this stack. Smashline is not a
dependency of this NRO. Other mods may still need their own plugins. Do not
remove those blindly.

The menu statically links `libimgui_smash.a`, rebuilt from pinned public
imgui-smash and Dear ImGui sources with the checked-in renderer patch.
The compatible standalone ImGui host is discovered by
exports and ABI, not by filename. Namespaced hook callbacks and local function
binding prevent the two NVN bootstraps from resolving to the same replacement.
Embedded, normal standalone order and reversed order were tested on `defda6fd`;
both menus and distinct hook addresses were observed. See
[dependency details](IMGUI_DEPENDENCY.md).

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

The opt-in `peer_observer` diagnostics were removed on 2026-10-17 (an existing
`peer_observer=` line is read and ignored). The handshake, station binding and
announcements never depended on that toggle. Transport hooks only annotate the
fork-compatible fields of validated traffic; they do not originate game inputs.

## Validation coverage

Historical `c0e9a625eb44a735` passed the checks named in the header. Its new events
were diagnostics only. A fresh single-instance capture confirmed the LessLag field
stores ran while the window was visible. At that checkpoint, reproducing the
concurrent-run failure remained pending; c0 was not a behavior fix. Later renderer
evidence is in [IMGUI_DEPENDENCY.md](IMGUI_DEPENDENCY.md). The latest entries
(0.4.0–0.4.2) are in [VALIDATION.md](VALIDATION.md).

b99 (`b99f087cf8ef0a9c`) was installed on the Switch at that checkpoint. On Eden it completed one
two-local-station Final Destination match, Game & Watch versus Sephiroth.
Both stations reported the same winner, two KOs, and 47% damage, then returned
to stage selection. LessLag and then LLUltra were each confirmed active. The
Game & Watch plugin `bceef` was on A only. A used FX Reduced with online scope.
B used FX On with scope all. Standalone ImGui and Elite were on B only. No
disconnect was observed. Both sides held cleanup and requested Vanilla at
results. The window was on the right for connected CSS and stage select, and
on the left during the match. The opponent column disappeared after B stopped.
There is no full-state or checksum proof. A three-station match is still not
done. High-RTT, VPN, NSO, and Switch performance are not accepted from this
match. b99 Reduced limits remain the open rows in
[FX_RUNTIME_MATRIX.md](https://git.syntaxbyexample.com/intpa/ssbu-fx-control/src/branch/main/docs/FX_RUNTIME_MATRIX.md), including the unobserved Inkling
forward-smash trace2 cull and the unobserved triggered Sephiroth counter body.

In the b99 concurrent run, the same-process embedded menu was visible on Vanilla and LLUltra and invisible on LessLag and LLDoubles. Finite traces show one draw and one submit per
present. The shared LessLag field stores are a lead. c0 can record whether
those stores ran. They also run in a visible, fresh single-instance c0 capture. The failure remains unexplained.

The prior 9e9 accepted connected-only peer visibility and three-station lobby
cycling, and that run ended before a match. Right-side Training placement is
confirmed. The native CSS display retains edits with the window closed and
restores styles under Streamer. Native LDN editing was observed; NSO arena
remains untested. Do not equate callback activity with visible pixels. See
[NATIVE_DISPLAY.md](NATIVE_DISPLAY.md) and [VALIDATION.md](VALIDATION.md).

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
