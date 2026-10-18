# Resume and test plan

Updated 2026-10-18 for Deluxe 0.5.6 (commit pending; 0.5.5 is `6a9e61a`, Forgejo release v0.5.5; 0.5.4 is `a63bc8c`). Read [STATUS.md](STATUS.md)
for what is implemented and which evidence exists, the newest entries of
[VALIDATION.md](VALIDATION.md) for the exact packages and gates, and `git log`.
Live operational state (card deployments and their receipts, Eden profiles,
worker sessions, the per-move ledger) lives in the operator's evidence root
outside this repository, not here.

## Where things stand

- Main is 0.5.6 (commit `fb2012a`, Forgejo release v0.5.6, NRO `df757ce2f8130fddd0c2ba69e35c0f2dd7eae5505365ce969ec88134a2fadad9`): Vanilla+,
  the default mode. With the new INI key `vanilla_plus` on (the default), the
  Vanilla profile keeps the game's three-image swap chain but presents the
  acquired image at the flush that carried that frame's last submission, one
  CPU update earlier than the game does, as the render-opts profiles already
  do. The NetProfile row, strip and native banner read `Vanilla+`
  (`Vanilla+*`, `Auto:Van+`); **L+R+X** (neither ZL nor ZR held) toggles it
  live and saves it; no separate NetProfile entry. Host gates: 237 + 1 tests,
  clippy with warnings denied, the no-default-features build. Justified by code
  reasoning and the 0.5.4 console API traces in Vanilla, with **no console run
  yet**; step 12 below is the check. The card still holds 0.5.5 until the
  operator installs 0.5.6.
- 0.5.6 also carries two operator changes of 2026-10-18 21:16Z. **UltraRes
  1080p:** `ultra_resolution=1080` and the UltraRes row (576p → 720p → 1080p →
  576p) let console LLUltra render at the native 1080p, dynamic resolution
  still stepping down from it; the default stays 576. The operator reports that
  Horizon OC made 720p LLUltra hold a locked 60 (operator-reported, no
  capture), so 1080p under the overclock is the next cell (step 13).
  **ZR hold for every window edit:** with the window open a Left/Right value
  change needs ZR in every scene, a plain Left/Right is ignored and Up/Down
  still move the selection; the window-closed ZL native edits are unchanged
  (step 14). Both are host-tested only.
- The operator dropped the FX measurement cells A–R on 2026-10-18 ("I'm not
  going to do all the effects testing"); the per-move ledger stays in the
  evidence root (`MOVE-LEDGER.md`).
- The last released Deluxe is 0.5.5 (commit `6a9e61a`, Forgejo release v0.5.5, on the card since 2026-10-18 with its first boot pending, NRO `cf57cf80f25f2e837deabaf859489dc00653692c269fb4a6e2069a6a1b9bf8c2`): the live-peer
  requirement for the operator's declined-rematch report. The opponent column,
  the strip's `Peer:` line, the Ping cell, the peer-dependent rows and the
  native CSS remote line need an RTT sample for the selected station within
  10 s, and event code 112 (`PeerStale`) records a bound station going silent.
  Host gates: 235 + 1 tests, clippy with warnings denied, the
  no-default-features build. Eden could not reproduce the report (a LAN-Online
  decline ends the session), so only host tests stand behind the fix. Step 11
  below checks it on the console.
- 0.5.4 (`a63bc8c`) is the per-file reduction pass
  merged from `reduction-candidate`; Forgejo release v0.5.4 carries the NRO
  and manifest.
  Gates on that commit: clippy with warnings denied, the no-default-features
  build, 233 + 1 host tests, Skyline release build. The 0.5.4 NRO was installed
  on the operator's card on 2026-10-17 over DBI MTP and **has not booted yet**;
  the card run is the acceptance test.
- The card stacks other never-booted changes under it: FX Control 0.2.1
  (`0c0d7e6`: page limited to the ten ledger fighters, EFF siblings removed,
  SDK pinned to `a63bc8c`), the gamepad overlay 0.1.2, the 0.5.3 work added since the last good boot (`48cce4f2`, 2026-10-16):
  `hide_pages` and the page rows, `perf_view`, `ultra_resolution`, the ZR hold
  for live edits, the clock probe and the L+R-only window chord; and the CFW
  update (Atmosphère 1.12.1, Horizon OC 2.6.0, hekate 6.5.4 kept). The last
  pair known to boot and play is Deluxe `48cce4f2` + FX Control `f3d52a38`.
- FX Control 0.2.1 (`0c0d7e6`, Forgejo release v0.2.1) pins the plugin SDK to
  `a63bc8c`. Gated on 2026-10-18, checked in Eden next to 0.5.4 (registration
  with descriptor version 4, `99 0x0 0x4` in its install log) and installed on
  the card the same day over DBI MTP, together with the gamepad overlay 0.1.2
  (`0d25b0d`, log strings only). Neither has booted on the console yet.
- The code-reduction campaign is paused by the operator. The effects
  measurement (numbered per-move Reduced variants on the console) is dropped,
  as above; FX Control 0.2.1 stays on the card with its variants unmeasured.

## Test plan, in order

### 1. First boot of 0.5.4 (console, operator; logs pulled over DBI MTP)

| # | Check | Pass | If it fails |
| --- | --- | --- | --- |
| 1 | CFW boots: hekate → sysMMC entry with the Horizon OC lines → HOME | HOME menu; HOC reports active | CFW problem, not plugins; the pre-update copies are in the evidence root |
| 2 | Game launch | Logo clears in normal time, title screen | Pull the crash report; roll Deluxe back to the card backup (0.5.3 `8dde0f2`, itself never booted), then to `48cce4f2` + FX `f3d52a38` |
| 3 | `install.log`, `events.log`, `hang-trace.log` | Clean init; FX Control registration (handle 1, descriptor 4); fighter hooks installed; event 111 clock reading; hang trace empty | FX not registered: deploy the SDK-5 FX build |
| 4 | Window chord | L+R opens the window; ZL+ZR does nothing anywhere, including the peer cycle | Chord removal regressed |
| 5 | Page cycle | Settings → FX Control only (card INI `hide_pages=plugins,performance`); Settings ends with PluginsPage / OwnPages / PerfPage (Shown or Hidden) and PerfView; setting PerfPage to Shown (ZR + Right; every value edit needs ZR) adds Performance live | — |
| 6 | Clock row and graph | Performance shows a MHz value and the frame-time plot; PerfView switches to a number (ZR + Left/Right) | A stock reading far from about 1020 MHz means the 1120 MHz threshold needs recalibrating ([VALIDATION](VALIDATION.md), clock probe entry) |
| 7 | FX Control page | Ten fighter rows with On / Off / Reduced; move rows read `Original` / `Variant N`; Inkling On / Off only; no EFF row | — |
| 8 | Training | LLUltra at 576p on the status line; graph live; a timing CSV with a `# clock` header written on exit | — |
| 9 | Character select | 60 fps on the fps line for about 30 s | The present-at-flush path regressed |
| 10 | Online, LessLag only | Switch NetProfile to LessLag, one Quickplay match, switch back | Operator rule: no LLUltra online match until LLUltra is stable offline |
| 11 | Declined rematch (needs the 0.5.5 NRO, which is on the card). Real NSO Quickplay: choose Rematch Yes while the opponent chooses No | The opponent column, the strip's `Peer:` line and the CSS remote line are gone within about 10 s, and `events.log` has a `112` row (a = station index, b = silence in ms). In a normal match, through the stage load, results and the return to the character select, the opponent column does not flicker and no `112` row appears | Column still shown after 10 s with no `112` row: the departed station is still being sampled; pull `events.log` and the Debug view's Ping detail. `112` rows in ordinary play: the console's loading gaps exceed 10 s; measure them before changing `LIVE_MS` |
| 12 | Vanilla+ (needs the 0.5.6 NRO, not installed yet; the card holds 0.5.5). With the shipped default `vanilla_plus=true`: menus, the character select and Training in Vanilla; the Performance page; `renderer-status.txt`; then L+R+X (neither ZL nor ZR) and a latency comparison | No visual fault in menus, CSS or Training (no tearing, stale frame or flicker); Performance keeps 60; `early_presents` advances in Vanilla with `early_present_mismatches=0`; L+R+X flips the NetProfile row text between `Vanilla+` and `Vanilla` and the `vanilla_plus` line in the saved `config.ini` (a `24` row in `events.log`); a menu and hot-swap latency comparison with the chord off against on feels the same or better | Visual fault, a mismatch count above 0 or a frame-rate drop: press L+R+X (or set `vanilla_plus=false` and restart), pull `renderer-status.txt` and `events.log`, and keep the 0.5.5 NRO as the rollback. The reasoning behind Vanilla+ is trace-based, not a console result |
| 13 | 1080p LLUltra under Horizon OC (needs the 0.5.6 NRO; the card holds 0.5.5). Training with the Training Modpack Input Display off (about 3 ms per frame), the Deluxe window hidden while performing, `CAPTURE_TIMING` for about 30 s per cell, one cell at 720p first and one at 1080p. Set the UltraRes row with ZR + Right (576p → 720p → 1080p; ZR + Left once from the default reaches 1080p), then close the window | The `Resolution` row reads 1080p; no one-second window below 59 frames, the seconds with the Deluxe window open (event code 23) excluded; the capture header reads `# clock=oc`, so it stays out of the move ledger and is read against the operator's uncaptured 720p observation. Expected: 60 | Drops at 1080p: step the row back to 720p or 576p; that is a finding for the overclocked console, not a defect, and the default stays 576. Drops at 720p too: the operator's 720p statement does not reproduce; record the clock and the moves played. A boot or video fault: set `ultra_resolution=576` in the INI over MTP and restart |
| 14 | ZR hold in every scene (needs the 0.5.6 NRO). Open the window (L+R+Down) on the title or main menu, then the character select, Training and a match, and on a row such as NetLatency or UltraRes press plain Left/Right, then ZR + Left/Right, then Up/Down; then close the window and use ZL + D-pad on the online character select pane | A plain Left/Right changes no value and writes no `config.ini` change in any scene; ZR + Left/Right changes it; Up/Down move the selection; ZL + D-pad on the closed-window native pane still edits | A plain press changes a value: the hold is not reaching that scene, so pull `events.log` and the scene's name. ZR + Left/Right does nothing: check that ZL is not also held (ZR alone unlocks the edit) |

### 2. Measurement cells (dropped)

The operator dropped the FX measurement cells A–R on 2026-10-18 ("I'm not going
to do all the effects testing"); no result is owed for them. The per-move
ledger stays in the evidence root (`MOVE-LEDGER.md`).

### 3. Host side, around the console run

| # | Work | Pass |
| --- | --- | --- |
| 1 | FX Control re-pin to `a63bc8c`: cargo test, clippy with warnings denied, no-default-features, Skyline clippy and release build | Done 2026-10-18 on `0c0d7e6`, all clean |
| 2 | Eden check of FX Control 0.2.1 next to Deluxe 0.5.4 on the isolated profile | Done 2026-10-18: registers with descriptor 4; the page itself was not opened in that run |
| 3 | FX Control 0.2.1 release (NRO + manifest) and card deploy | Done 2026-10-18: Forgejo release v0.2.1; 0.2.1 and the overlay 0.1.2 installed over DBI MTP with rollback archives |

## Rules that still apply

- Never claim an inferred crash fix or untested game compatibility as
  verified; emulator evidence is labelled as such.
- Exclude the window-open seconds (event code 23) from every capture; captures
  taken under an overclock (`# clock=oc`) do not enter the ledger.
- FX lives only in the FX Control plugin; Deluxe carries no effect hooks. No
  global `Layout::Draw` hook; do not patch the overlay's `0x374812c`
  neighbourhood.
- The operator removed the Game & Watch plugin from the card on 2026-10-09; do
  not reinstall it with a Deluxe update. Other plugins, the INI and the
  Horizon OC / boot files are preserved byte for byte on every install.
- Push only to the Forgejo `origin`, and only with the operator's permission.
  `github` is a downstream mirror.

## Emulator operation

Isolated profiles, finite runtime and memory limits, one Eden at a time for
frame-rate comparisons, no concurrent heavy builds. The CPU-quota method that
makes the emulator show the console's drops is in
[EMULATOR_METHOD.md](EMULATOR_METHOD.md): a 175 % quota reproduces the
console's character-select frame rate with both switches off. The Clock row
reads the host clock in Eden and is meaningless there. The CLI control
documentation of the delivered Eden build is under
`/home/intpa/eden-cli-handoff/updates/20261007-cli-controls/`.

## Open threads

- Coverage trace (two Eden campaigns; the second, 2026-10-18, used Eden's LAN
  Online mode with two modded peers and executed 893 of 1,192 instrumented
  functions in NSO scenes). The modded-opponent exchange is covered: it runs
  only with Stealth Off on both peers and a new online session after the
  toggle ([CONFIGURATION.md](CONFIGURATION.md)). The hook sites behind
  `deluxe_rewrite_arena` (`main+0x22d9fa0`, in the middle of a scene-factory
  function) and `deluxe_rewrite_background_matchmaking` (`main+0x22da090`, a
  factory whose class the arena and Background flows do not create) never fire;
  both addresses were copied from the fork and are guarded only by `preflight`.
  Follow-up, not done in 0.5.4: remove or re-derive both sites and add
  instruction-word guards to every fork-derived address. Meanwhile arena and
  Background matches are classified through `stage_setup` and `mark_match`.
  Evidence (Eden only) is in the 2026-10-18 entry of
  [VALIDATION.md](VALIDATION.md).
- Follow-ups for the declined-rematch work (0.5.5): instrument the station-left
  hook (`.text+0x15f240`) with an event code per leave, accepted or dropped,
  carrying the leave's protocol id, the RTT table's protocol and the station
  index. One console match with a declined rematch then tells H1a (the hook
  never fires for a decline) from H1b (it fires and the protocol gate drops the
  leave); under H1b, accept a leave whose station matches the bound selection
  and keep the foreign-protocol guard for unbound indices. The Eden agent (the
  LAN-Online harness owner) is separately fixing the harness's decline path to
  behave like NSO's, so that an Eden run can exercise it.
- Skyline version: the card runs a newer skyline-dev build than the one
  upstream Online Deluxe pins; the operator wants 0.5.x verified on both.
- Reduced effects for tournament-legal stages (Battlefield first): plan in the
  operator's evidence root; step 1 is a per-stage LLUltra capture set under
  the same acceptance rule as the moves.
- Optional and deferred since 2026-10-08: the high-FPS Palutena follow-up
  (400–450 fps emulator mods). Not a release gate.
