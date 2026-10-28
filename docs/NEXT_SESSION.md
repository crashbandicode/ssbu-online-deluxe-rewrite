# Resume and test plan

## 0.8.1 Horizon-OC release check — 2026-10-28

**Done.** The operator tested release `0.8.1-84da224d084f-1793155065` (NRO
`0c16374b…`) with `hoc_oc=online` on the console, including rematches. The
card's event logs agree:

- The profile applied for every online Singles match.
- It stayed applied across two rematches.
- Event 115 returned to `0b0110` about 15 s after the opponent's last sample.
- Offline Training and Doubles stayed at stock clocks.

Details are in [Validation](VALIDATION.md#081--console-rematches--2026-10-28).

## Earlier evidence — 2026-10-25

**0.8.0 Auto regression acceptance:** host tests and native byte verification
cover match-scoped observations, but a live online run is still needed. With
NetLatency on Auto, play consecutive matches whose buffers differ (for example
4f then 3f). The next stage should start plain `Auto`, then show its own count
during play, without waiting for match end. Check the Settings/strip and native
return banner; retain `events.log` with stage event 110 and observation event
113 (`b=0x16d81a8`) to establish timing. Check result retention, Offline/Training
clear, and manual-to-Auto staying plain until a fresh match. Record the release
manifest/build identity with the result. No 0.8.0 console pass is claimed yet.

The operator reports that Deluxe works in Eden and on an Erista Switch without
issues, including the current firmware/Atmosphère/Skyline stack
([versions and report](VALIDATION.md#operator-report--eden-and-erista-switch--2026-10-25)).
The released 0.7.0 offline UI review and the candidate's console LLUltra chord
are recorded below. The remaining checklist is feature-specific: it does not
mean the plugin has never been tested on console.
The published v0.6.2 notes also record the operator's online `Auto (Nf)` pass
on 2026-10-19; step 16 below is a retained comparison checklist, not a pending boot.
The operator confirms dispatcher backoff also helped on Switch (2026-10-25).

Use [Controls](CONTROLS.md) for the current UI. The older versioned baseline and
test matrix are historical references, not current card inventory or directions
to reinstall an older NRO. Live deployments and worker state remain outside this
repository.

## 0.7.0 acceptance — 2026-10-22

The operator tested installed candidate `2f5a…` on the Switch 2026-10-22 and
confirmed the LLUltra chord. Its build id is `0.6.2-d86fac2…`; the versioned
release is a separate artifact. Exact release and install identities are in
[VALIDATION.md](VALIDATION.md). Remaining console checks follow.

**Offline Eden UI review, 2026-10-24:** the released `2ff456…` NRO rendered the
compact Settings/icons, full companion/page cycle, Training window, match strips
and native banners; ZR-guarded NetLatency and You-row privacy round trips were
recorded. See [Screenshots](SCREENSHOTS.md) and [Validation](VALIDATION.md).
These observations do not complete the console, connected-peer or negative
shortcut checks below.

## Compact menu acceptance (carries into 0.7.0)

0.7.0 folds Stealth/Streamer into eye/TV icons beside You and
UltraRes into one console NetProfile LLUltra stop, labeled LLUltra / LLUltra+ /
LLUltra++.
The older version-specific rows below remain historical. Intermediate installs
(`5a2a5f05`, then `2f5a…`) are in [VALIDATION.md](VALIDATION.md). Current source
shows
`ZR+Left/Right edits   ZR+X Stealth   ZR+Y Streamer   L+R+Down next page   L+R+X Vanilla+/LLUltra`.
On the console, check:

- Settings opens with You selected; Up/Down walks You, the optional Opp line,
  NetLatency, NetProfile and wraps. There are no Stealth/Streamer rows. Both
  icons fit beside You at 720p and 1080p; the table marks it `> You` and the
  match strip `>You:`, followed by the same icons, then Net/Profile.
- With You selected, hold ZR and tap X/Y: Stealth/Streamer toggle once per tap.
  Closed eye=Stealth on, green filled TV=Streamer on. No toggle on Opp,
  NetLatency, NetProfile, another page, the hidden window, ZL instead of ZR, or
  both triggers together; an X/Y already held while moving onto You does not
  fire. Vanilla+'s L+R+X shortcut remains independent. The icon states are drawn
  in [privacy-indicators.svg](privacy-indicators.svg) (source geometry, not a
  screenshot).
- Changing Stealth during a connection shows an amber eye/asterisk and reconnect
  notice, even with Hints hidden. The session latch still changes on reconnect.
- Console NetProfile has one LLUltra stop at every saved resolution. Left or
  Right leaves it for the adjacent profile and remembers that resolution;
  entering it again shows the saved level. While manual console LLUltra is
  selected, L+R+X cycles 576 → 720 → 1080 → 576 in place and leaves the mode,
  Auto and Vanilla+ unchanged. The same chord still toggles Vanilla+ for Auto,
  the emulator and every other profile. Save/reload retains `render_mode=llultra`
  and `ultra_resolution`; Auto, cleanup labels and DRS keep their existing
  behavior.

Build/host evidence: [VALIDATION.md](VALIDATION.md). Live worktree/package state
and handoff are outside the repository in the evidence root's
`ship-compact-menu-1021/PLAN.md`. No runtime pass is implied by these pending checks.

## Released 0.6.2 baseline and historical checks

Updated 2026-10-19 for Deluxe 0.6.2 (Forgejo release v0.6.2; the NRO SHA-256 is in that release's manifest; 0.6.1 is Forgejo release v0.6.1 and booted on the console 2026-10-19, the operator confirming the title; 0.6.0 is Forgejo release v0.6.0 and booted on the console 2026-10-19, the operator reporting that it works; 0.5.7 is `282a2df`, Forgejo release v0.5.7, on the card since 2026-10-19 00:11Z, console boot pending; 0.5.6 is `fb2012a`, Forgejo release v0.5.6 and booted on the console; 0.5.5 is `6a9e61a`; 0.5.4 is `a63bc8c`). Read [STATUS.md](STATUS.md)
for what is implemented and which evidence exists, the newest entries of
[VALIDATION.md](VALIDATION.md) for the exact packages and gates, and `git log`.
Live operational state (card deployments and their receipts, Eden profiles,
worker sessions, the per-move ledger) lives in the operator's evidence root
outside this repository, not here.

## Where things stand

- 0.6.2 (2026-10-19, Forgejo release v0.6.2, NRO SHA-256 in that release's
  manifest): Auto shows the frame count the game chose, `Auto (4f)`, on the
  window's You line, the NetLatency row, the in-match strip's `Net:` field and
  the native character-select and arena `You:` line (`You: Auto:Van | Auto
  (4f)`); the lobby digit pane keeps plain `Auto`. The operator asked for it
  after booting 0.6.1. The number is the game's buffer byte read at the
  online-latency hook (event code 9, first value) while Auto is selected; it is
  cleared when the connection returns to Offline, and the label is plain `Auto`
  before the first online match of a session. Manual buffers, the Opp line, INI
  keys, event codes, chords and the plugin API are unchanged; the native You
  line's latency field is 10 characters wide (line cap 32 → 38). The online
  `Auto (Nf)` display was confirmed on Switch on 2026-10-19; see [Validation](VALIDATION.md).
- 0.6.1 (2026-10-19, Forgejo release v0.6.1, NRO SHA-256 in that release's
  manifest): retitles the Player Info window's last page **Window Options**.
  The operator booted 0.6.0 on the console the same day ("seems to work great.
  menu is much cleaner") and asked only for a real title on that page. The
  rows, chords, INI keys and plugin API are unchanged. Booted on the console on
  2026-10-19; the operator confirmed the title.
- 0.6.0 (2026-10-19, Forgejo release v0.6.0, NRO SHA-256 in that release's
  manifest): the redesign of the Player Info main page, because the operator
  called the old Settings page bloated. Settings is two columns with no opponent
  column: a read-only **You** line (`Vanilla+ | Auto | Wi-Fi`), an **Opp** line
  while the peer shows (`Mario | LessLag | 3 | 23ms | Ethernet`, announced facts
  only, ping-stability colour, ZR + Left/Right cycles the station), five rows
  (NetLatency, NetProfile with the Status folded in, Stealth, Streamer, UltraRes
  on a console), a one-line performance footer and a dim hint line (INI
  `hints`, default true). The page toggles and PerfView moved to a new
  **Window Options** page (titled just Window in 0.6.0 itself), the last page
  of the L+R+Down cycle, with a Hints row;
  PerfView now affects the Performance and Debug pages only. The selected value
  reads `< value >` while ZR is held. The in-match strip is `Net`, `Profile`,
  `Stealth`, `Stream`, `UltraRes` and a `>Opp:` line with the timing line.
  Nothing else changed (chords, Performance and Debug pages, plugin pages, the
  native banner, event codes, the peer exchange). Host-tested, then booted on
  the console on 2026-10-19: the operator said it works and the menu is much
  cleaner, and asked only for the page title that 0.6.1 adds (step 15 below;
   its checks were not reported one by one). The 0.6.0 Eden visual check was
   deferred at that point; the newer 0.7.0 offline UI review is recorded above.
- 0.5.6 (commit `fb2012a`, Forgejo release v0.5.6, NRO `df757ce2f8130fddd0c2ba69e35c0f2dd7eae5505365ce969ec88134a2fadad9`): Vanilla+,
  the default mode. With the new INI key `vanilla_plus` on (the default), the
  Vanilla profile keeps the game's three-image swap chain but presents the
  acquired image at the flush that carried that frame's last submission, one
  CPU update earlier than the game does, as the render-opts profiles already
  do. The NetProfile row, strip and native banner read `Vanilla+`
  (`Vanilla+*`, `Auto:Van+`); **L+R+X** (neither ZL nor ZR held) toggles it
  live and saves it; no separate NetProfile entry. Host gates: 237 + 1 tests,
  clippy with warnings denied, the no-default-features build. Justified by code
  reasoning and the 0.5.4 console API traces in Vanilla, and the first console
  boot the same evening showed 0 early-present mismatches over 89,500
  presents in menus, CSS and Training (see VALIDATION); the visual and
  latency check of step 12 is still the operator's call. The card held 0.5.6
  from 2026-10-18 (installed over 0.5.5, rollback archive kept) until 0.5.7
  replaced it.
- 0.5.7 (`282a2df`, Forgejo release v0.5.7, on the card, NRO `27cadcd719d549da5c0fc27cce9c7e752c6dcd17c20cebba443405ae039876c8`): the Streamer chord is L+R + **ZR** +
  Y only; the ZL variant does nothing. The evening's online session ended with
  `streamer=true` saved on the card without the operator meaning it, which is
  why the opponent's connection did not show; the card config was set back to
  `streamer=false` and `input_buffer=auto` by hand. Console check: L+R+ZL+Y
  changes nothing, L+R+ZR+Y flips the Streamer row.
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
- 0.5.5 (commit `6a9e61a`, Forgejo release v0.5.5, on the card for a few hours on 2026-10-18 and replaced by 0.5.6 without booting, NRO `cf57cf80f25f2e837deabaf859489dc00653692c269fb4a6e2069a6a1b9bf8c2`): the live-peer
  requirement for the operator's declined-rematch report. The opponent column,
  the strip's `Peer:` line, the Ping cell, the peer-dependent rows and the
  native CSS remote line (since 0.6.0 the Opp line and the strip's `>Opp:`
  line) need an RTT sample for the selected station within
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
  on the operator's card on 2026-10-17 over DBI MTP and booted on 2026-10-18;
  see its [validation record](VALIDATION.md).
- The historical card baseline also contained then-unverified changes: FX Control 0.2.1
  (`0c0d7e6`: page limited to the ten ledger fighters, EFF siblings removed,
  SDK pinned to `a63bc8c`), the gamepad overlay 0.1.2, the 0.5.3 work added since the last good boot (`48cce4f2`, 2026-10-16):
  `hide_pages` and the page rows, `perf_view`, `ultra_resolution`, the ZR hold
  for live edits, the clock probe and the L+R-only window chord; and the CFW
  update (Atmosphère 1.12.1, Horizon OC 2.6.0, hekate 6.5.4 kept). Before those
  updates, the known working pair was Deluxe `48cce4f2` + FX Control `f3d52a38`.
- FX Control 0.2.1 (`0c0d7e6`, Forgejo release v0.2.1) pins the plugin SDK to
  `a63bc8c`. Gated on 2026-10-18, checked in Eden next to 0.5.4 (registration
  with descriptor version 4, `99 0x0 0x4` in its install log) and installed on
  the card the same day over DBI MTP, together with the gamepad overlay 0.1.2
  (`0d25b0d`, log strings only). At that checkpoint, their console boot was pending;
  later general compatibility reporting is linked at the top of this document.
- The code-reduction campaign is paused by the operator. The effects
  measurement (numbered per-move Reduced variants on the console) is dropped,
  as above; FX Control 0.2.1 stays on the card with its variants unmeasured.

## Test plan, in order

This matrix retains the version-specific checks and original row numbers for
cross-references. Superseded layouts/chords are labelled by version; use the
compact-menu section above for 0.7.0. An old "on the card" statement describes
that entry's checkpoint, not the currently installed artifact.

### 1. First boot (console, operator; logs pulled over DBI MTP). 0.5.4 booted on 2026-10-18 and 0.5.6, 0.6.0 and 0.6.1 on 2026-10-19; rows the operator did not report one by one stay open

| # | Check | Pass | If it fails |
| --- | --- | --- | --- |
| 1 | CFW boots: hekate → sysMMC entry with the Horizon OC lines → HOME | HOME menu; HOC reports active | CFW problem, not plugins; the pre-update copies are in the evidence root |
| 2 | Game launch | Logo clears in normal time, title screen | Pull the crash report; roll Deluxe back to the card backup (0.5.3 `8dde0f2`, itself never booted), then to `48cce4f2` + FX `f3d52a38` |
| 3 | `install.log`, `events.log`, `hang-trace.log` | Clean init; FX Control registration (handle 1, descriptor 4); fighter hooks installed; event 111 clock reading; hang trace empty | FX not registered: deploy the SDK-5 FX build |
| 4 | Window chord | Hold L+R and tap Down to open/cycle the window; ZL+ZR does nothing anywhere, including the peer cycle | Chord removal regressed |
| 5 | Page cycle | Settings → FX Control → Window Options → hidden (card INI `hide_pages=plugins,performance`); since 0.6.0 the Window Options page (titled just Window in 0.6.0 itself) ends the cycle and carries PluginsPage / OwnPages / PerfPage (Shown or Hidden), PerfView and Hints (before 0.6.0 the first four were Settings rows); setting PerfPage to Shown on the Window Options page (ZR + Right; every value edit needs ZR) adds Performance live | — |
| 6 | Clock row and graph | Performance shows a MHz value and the frame-time plot; PerfView (on the Window Options page since 0.6.0) switches to a number (ZR + Left/Right) | A stock reading far from about 1020 MHz means the 1120 MHz threshold needs recalibrating ([VALIDATION](VALIDATION.md), clock probe entry) |
| 7 | FX Control page | Ten fighter rows with On / Off / Reduced; move rows read `Original` / `Variant N`; Inkling On / Off only; no EFF row | — |
| 8 | Training | LLUltra at 576p on the status line; graph live; a timing CSV with a `# clock` header written on exit | — |
| 9 | Character select | 60 fps on the fps line for about 30 s | The present-at-flush path regressed |
| 10 | Online, LessLag only | Switch NetProfile to LessLag, one Quickplay match, switch back | Operator rule: no LLUltra online match until LLUltra is stable offline |
| 11 | Declined rematch (needs the 0.5.5 NRO, which is on the card). Real NSO Quickplay: choose Rematch Yes while the opponent chooses No | The opponent column, the strip's `Peer:` line and the CSS remote line are gone within about 10 s (since 0.6.0: the Opp line, the strip's `>Opp:` line and the CSS remote line), and `events.log` has a `112` row (a = station index, b = silence in ms). In a normal match, through the stage load, results and the return to the character select, the opponent column (the Opp line since 0.6.0) does not flicker and no `112` row appears | Column still shown after 10 s with no `112` row: the departed station is still being sampled; pull `events.log` and the Debug view's Ping detail. `112` rows in ordinary play: the console's loading gaps exceed 10 s; measure them before changing `LIVE_MS` |
| 12 | Vanilla+ (0.5.6 or later; the card holds the `2f5a…` install, 0.7.0 source stamped 0.6.2, see [VALIDATION.md](VALIDATION.md)). With the shipped default `vanilla_plus=true`: menus, the character select and Training in Vanilla; the Performance page; `renderer-status.txt`; then L+R+X (neither ZL nor ZR, and not with manual console LLUltra selected, where the chord cycles the resolution) and a latency comparison | No visual fault in menus, CSS or Training (no tearing, stale frame or flicker); Performance keeps 60; `early_presents` advances in Vanilla with `early_present_mismatches=0`; L+R+X flips the NetProfile row text between `Vanilla+` and `Vanilla` and the `vanilla_plus` line in the saved `config.ini` (a `24` row in `events.log`); a menu and hot-swap latency comparison with the chord off against on feels the same or better | Visual fault, a mismatch count above 0 or a frame-rate drop: press L+R+X (or set `vanilla_plus=false` and restart), pull `renderer-status.txt` and `events.log`, and keep the 0.5.5 NRO as the rollback. The reasoning behind Vanilla+ is trace-based, not a console result |
| 13 | 1080p LLUltra under Horizon OC (0.5.6 or later; the card holds the `2f5a…` install). Training with the Training Modpack Input Display off (about 3 ms per frame), the Deluxe window hidden while performing, `CAPTURE_TIMING` for about 30 s per cell, one cell at 720p first and one at 1080p. Since 0.7.0: select manual LLUltra on NetProfile (ZR + Left/Right), then L+R+X steps LLUltra (576p) → LLUltra+ (720p) → LLUltra++ (1080p); then close the window. On 0.5.6–0.6.2 the same levels were the UltraRes row | The `Resolution` row reads 1080p; no one-second window below 59 frames, the seconds with the Deluxe window open (event code 23) excluded; the capture header reads `# clock=oc`, so it stays out of the move ledger and is read against the operator's uncaptured 720p observation. Expected: 60 | Drops at 1080p: cycle back to LLUltra+ (720p) or LLUltra (576p); that is a finding for the overclocked console, not a defect, and the default stays 576. Drops at 720p too: the operator's 720p statement does not reproduce; record the clock and the moves played. A boot or video fault: set `ultra_resolution=576` in the INI over MTP and restart |
| 14 | ZR hold in every scene (0.5.6 or later). Open the window (L+R+Down) on the title or main menu, then the character select, Training and a match, and on a row such as NetLatency or NetProfile press plain Left/Right, then ZR + Left/Right, then Up/Down; then close the window and use ZL + D-pad on the online character select pane | A plain Left/Right changes no value and writes no `config.ini` change in any scene; ZR + Left/Right changes it; Up/Down move the selection; ZL + D-pad on the closed-window native pane still edits | A plain press changes a value: the hold is not reaching that scene, so pull `events.log` and the scene's name. ZR + Left/Right does nothing: check that ZL is not also held (ZR alone unlocks the edit) |
| 15 | **Checked by the operator on the console 2026-10-19: the layout is OK; only the page title was changed (0.6.1, Window Options).** 0.6.0 Player Info layout (needs the 0.6.0 NRO). Open the window (L+R+Down) on the main menu, the character select, Training and in a match, with a live opponent for the Opp line (Quickplay or the LAN-Online harness); then cycle with L+R+Down through every page | Settings shows the **You** line (`<profile> \| <buffer> \| <interface>`), the **Opp** line only while a live opponent shows (`<name> \| <profile> \| <buffer> \| <ping> \| <interface>`, no `---` placeholders, painted in the ping's green, yellow or red), then exactly NetLatency, NetProfile (Status folded in, e.g. `LessLag \| Swap`), Stealth, Streamer and, on the console, UltraRes; no Name, Interface, Peer, Ping or Status row and no opponent column; the one-line footer (`60.0 fps \| 16.67 ms \| 1080p \| DRS on`) in the frame-time colour; the grey hint line wrapping to the window width. The cycle ends Performance → **Window Options** → hidden (Settings → FX Control → Window Options → hidden with `hide_pages=plugins,performance`); Window Options lists PluginsPage, OwnPages, PerfPage, PerfView and Hints, and Hints Hidden removes the hint line from Settings and Window Options and writes `hints=false` to `config.ini`. With ZR held the selected value reads `< value >` (e.g. `< Auto >`), without ZR it is plain; ZR + Left/Right on the Opp line cycles the remote station. In a match the strip's line 1 is `Net`, `Profile`, `Stealth`, `Stream`, `UltraRes` and line 2 is `>Opp:<name> \| <profile> \| <buffer> \| <ping> \| <interface>` followed by `FPS:.. FT:.. AVG:.. [####------]`; the Window Options page's strip shows `PlugPg`, `OwnPg`, `PerfPg`, `PerfView` and `Hints` | Any clipped, wrapped or overlapping text, a wrong colour, a missing or extra row, or a page out of order: photograph the window, note the scene and resolution, and pull `events.log`. `hints=false` in the INI (or the Hints row) removes the hint line if that line is the problem. The rollback is `pre-061-*` (0.6.0) or `pre-060-*` (0.5.7). Layout and colour were checked on the console by the operator on 2026-10-19; no Eden visual check has run |
| 16 | **Online `Auto (Nf)` display confirmed by the operator on 2026-10-19**, recorded in the [v0.6.2 release](https://git.syntaxbyexample.com/intpa/ssbu-online-deluxe-rewrite/releases/tag/v0.6.2). Remaining detailed comparison: NetLatency Auto, one Quickplay match, then the strip, window, native character-select line and main-menu reset | The strip, You summary, NetLatency and native You line agree on the game's N (at most 25); main-menu Offline resets them to plain `Auto`; Opp still shows the announced `Auto`; event code 9 carries N; manual values stay numeric. These details were not reported individually | On a mismatch, capture the affected label and scene and retain `events.log`; compare N against code 9. |

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
- Skyline version: the operator reports compatibility with the current runtime,
  beta `2eb226f` as checked 2026-10-25 ([version evidence](VALIDATION.md)). The
  earlier request to compare the upstream fork's older pin has no separate
  paired-version result recorded.
- Reduced effects for tournament-legal stages (Battlefield first): plan in the
  operator's evidence root; step 1 is a per-stage LLUltra capture set under
  the same acceptance rule as the moves.
- Optional and deferred since 2026-10-08: the high-FPS Palutena follow-up
  (400–450 fps emulator mods). Not a release gate.
