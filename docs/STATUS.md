# Implementation and validation status

Updated 2026-10-31 for **0.9.0**. Exact release/build and console evidence is in
[VALIDATION.md](VALIDATION.md). Earlier releases: 0.8.1 `84da224`, 0.8.0 `d92bc3d`, 0.7.0 `9a7f81c`, 0.6.2 (NRO SHA-256 in its
release manifest), 0.6.1 `9d0cec2`, 0.6.0 `a0dc5d6`, 0.5.7 `282a2df`,
0.5.6 `fb2012a`, 0.5.5 `6a9e61a`, 0.5.4 `a63bc8c`. The rewrite is one Rust/Skyline
NRO for Smash 13.0.5. Auto is enabled by default; it chooses Vanilla (Vanilla+
since 0.5.6) in menus,
offline play and Training, LessLag for console online singles, LLUltra for
emulator online singles, and LLDoubles for online matches with more than two
fighters. Platform is an explicit INI setting. Deluxe sets no clocks itself;
the clock probe (0.5.3) only records the clock state so that performance
captures can be classified, and since 0.8.1 `hoc_oc=online` tells the
game-aware Horizon-OC fork when Smash's saved profile may apply.

The operator reports testing in **Eden and on an Erista Switch without issues**,
with Switch firmware **23.0.1**, Atmosphère **1.12.1** and Skyline
**beta `2eb226f`** (latest checked 2026-10-25;
[version links and report](VALIDATION.md#operator-report--eden-and-erista-switch--2026-10-25)).
The 2026-10-24 offline UI review also covers the released 0.7.0 menu, privacy
shortcuts and strips. Specific measurements and online/negative-control checks
below remain distinct from that general compatibility report.

Publication routing: `origin` is Forgejo
(`git.syntaxbyexample.com/intpa/ssbu-online-deluxe-rewrite`), authoritative
and public; `github` is a downstream mirror that is not pushed automatically.
Nothing is pushed without the operator's explicit permission. Forgejo
releases carry the NRO and its SHA-256 manifest (see [Current: 0.9.0](#current-090)).

Evidence levels used below: **host** = Rust host tests, clippy with warnings
denied, the no-default-features build and the Skyline release build;
**Eden** = an emulator run on an isolated profile; **console** = a result from
the operator's Switch, recorded with its build hash in [VALIDATION.md](VALIDATION.md).

| Feature | Implementation | Runtime evidence / remaining check |
| --- | --- | --- |
| Vanilla / LessLag / LLUltra | Live guarded transitions, persistent manual choice; console LLUltra uses `ultra_resolution` (576 default, 720, 1080) with one LLUltra / LLUltra+ / LLUltra++ stop. L+R+X with NetProfile highlighted cycles the base resolution while manual console LLUltra is selected; DRS still steps down from it. [Rendering profiles](RENDERING.md#llultra-resolution-053-1080p-in-056). | Console 2026-10-15 (0.5.2): seventeen Training captures; Vanilla and LessLag hold 60 on every ledger move, LLUltra drops in 11 of 17 sessions. LessLag online singles held 60. Horizon OC at 720p held 60 (operator-reported 2026-10-18, no capture). The compact LLUltra resolution cycle was confirmed on `2f5a…` on 2026-10-22; a 1080p performance measurement remains unrecorded. |
| LLDoubles | Three textures, OneBehind, render sequencing, resolution scaling | b19 linked Auto/LLDoubles team match completed through results/CSS; Eden response trials keep the two-image saving after present-at-flush (0.5.2) |
| Auto | Scene, connection and fighter-count policy; preserves saved manual choice; Quickplay hold and stale demotion (0.4.2) | Training/menu Vanilla, b19 linked doubles LLDoubles, d9 linked singles LLUltra and cleanup Vanilla passed in Eden; 0.4.2 console results in VALIDATION.md |
| Present at the flush | Two-image LessLag presents the acquired image at the flush that submits it (0.4.0); extended to every committed render-opts profile in 0.5.2; `early_present=false` switches it off (0.5.3) | Console-verified for LessLag on the character select (49.9 → 59.85 fps, 0.4.0). LLUltra/LLDoubles extension: Eden response trials (savings of 3, 4 and 2 images against Vanilla, unchanged through the 0.5.4 candidate); console LLUltra captures above. Before/after clips at the console-calibrated 175 % CPU quota in [EMULATOR_METHOD.md](EMULATOR_METHOD.md) |
| Vanilla+ | Vanilla keeps the game's three-image ring but presents at the frame's last flush, one CPU update earlier (0.5.6). `vanilla_plus=true` is the default; L+R+X with NetProfile highlighted (only there since 0.9.0) toggles it except with manual console LLUltra selected, where it cycles resolution. Labels read `Vanilla+` / `Vanilla+*` / `Auto:Van+`; `early_present=false` disables it. [Rendering profiles](RENDERING.md#vanilla-056). | Host tests and a console boot on 2026-10-18: `early_presents` 89,483 = skips, `early_present_mismatches=0` over 89,500 presents in menus, CSS and Training. The 9–13 ms estimate comes from API timing, not an input-to-photon measurement. A dedicated visual/chord/latency A/B record remains open; the 0.7.0 UI capture fixture had Vanilla+ off. |
| Dispatcher backoff | Idle-pass sleep with growing pauses; `dispatcher_backoff=false` switches it off (0.5.3), `DISABLE_BACKOFF` unchanged | Eden before/after clips at the 175 % quota. The operator confirms it also helped on Switch (2026-10-25); an isolated numeric Switch A/B delta was not supplied. [Details](DISPATCHER_BACKOFF.md). |
| Resolution / DRS | Cut-in trigger (shipped); per-move catalog `drs_moves=` and the adaptive trigger `drs_adaptive=` with threshold/trigger/release keys (0.5.2, experiment). The built-in Sephiroth Gigaflare trigger was removed in 0.5.4 (commit 49fbfe1) | The cut-in trigger drops resolution on the console (ledger observation). The 0.5.2 triggers have been on the card since 2026-10-15 with no hardware result yet; the resolution-comparison cells were dropped from the test plan with the other FX cells (operator, 2026-10-18). LLUltra's base level is `ultra_resolution`, the NetProfile LLUltra / LLUltra+ / LLUltra++ stop above (the UltraRes row before 0.7.0) |
| Fighter Main | Guarded read-only fighter status callback, bounded report tokens | Two count-only Training sessions passed approximate tick-frequency checks; count mode installs no cut-in hook or reports |
| Cut-in scaler | Original cut-in function chained, seven eligible fighter-tick cooldown | Luigi close-range Up-B cut-in passed in Eden: LLDoubles 1080→720→1080; console captures show the cut-in zoom pushes (56–59 fps seconds under LLUltra, 2026-10-16) |
| Online input buffer | Auto or 0–25, retained null guard and Quickplay path. 0.8.0 samples the live input-consumption byte for `Auto (Nf)` with per-match validity; new matches start plain Auto, results retain the last count, Offline/Training clear it, and manual choices block Auto observation until the next match. | Host regression and native-byte checks; 0.8.0 live-online timing remains untested. The earlier v0.6.2 operator confirmation showed the label appeared, but did not validate freshness across consecutive matches. |
| Stealth | Default On, connection-latched beacon/payload suppression, reconnect notice | d9 fresh-On suppression/zero custom writes, pending-Off and saved Off after restart passed; earlier pending-On passed |
| Peer display | Remote facts only for a selected connected station that is also live: a measured RTT sample within 10 s (`pia_rtt::LIVE_MS`, a new binding gets a 10 s grace), applied to the opponent column, the strip's `Peer:` line, the Ping cell, the peer-dependent rows and the native CSS remote line; event 112 `PeerStale` when a bound station goes silent (0.5.5). Since 0.6.0 the Settings page's opponent column, Name / Interface / Peer / Ping rows and the strip's `Peer:` / `IF:` / `Ping:` / `PeerNet:` / `PeerProfile:` fields are one **Opp** summary line (and the strip's `>Opp:` line) with the same live rule. Name, RTT/stability colors and announced settings; sixteen keyed RTT stations behind one lock (0.5.4) | 9e9 offline/connected/disconnected and two-remote-peer cycling passed in Eden; the 0.5.4 station table and the 0.5.5 live rule are host-tested only. The declined-rematch persistence the operator reported on the console could not be reproduced in Eden (a LAN-Online decline ends the session); console verification of 0.5.5 pending |
| Ethernet / Wi-Fi | Local nifm type and peer announcement, otherwise Unknown | Never inferred from RTT; an emulator may report a virtual interface |
| Streamer / peer cycle | Persistent display privacy, identity-consistent peer selection; the window chord is L+R only since 0.5.3 (ZL+ZR does nothing). Since 0.9.0 Streamer is toggled by L+R+X with its You-line icon framed (ZR+Left/Right); ZR+Y with You selected (0.7.0–0.8.1) and the L+R+ZR+Y chord of 0.5.7–0.6.2 are retired | d9 saved INI/privacy passed; 9e9 cycled two remote peers in a three-station lobby. The L+R-only chord was used on the console on 2026-10-19 (the 0.6.0 and 0.6.1 boots) |
| Global Player Info | Compact Settings page: selectable **You** with Stealth eye, Streamer TV and (with the fork) OC clock icons, **Opp** only for a live peer, **NetLatency**, **NetProfile**, performance footer and hints. Since 0.9.0 (`a49ec48`/`866d8b2`; on the card as candidate `3ea3c49` since 2026-10-28, console check pending) ZR+Left/Right on You moves a light-blue frame between the icons and L+R+X toggles the framed one (L+R+X acts only on the highlighted row: You or NetProfile); ZR+X/Y/L3 are retired after accidental in-match ZR+L3 toggles. Manual console LLUltra is one resolution-aware profile stop. The vertical window sits right in Training/lobbies; ordinary matches use a two-line strip at top left. Since `dfa1608` (0.9.0, console check pending), lobby practice in a local-wireless or online waiting room counts as Training whatever stage it loads; before, a loaded lobby stage drew the strip (operator report and console log, 2026-10-28). Full details and the earlier 0.6.x layout: [Controls](CONTROLS.md). | Earlier layouts were operator-confirmed on console. Released 0.7.0 `2ff456…` has offline Eden evidence for Settings, privacy round trips, Training, match strips and banners (2026-10-24); the `2f5a…` console LLUltra chord was confirmed on 2026-10-22. The general Eden/Erista report is above; connected-peer and negative-control checks remain in [NEXT_SESSION.md](NEXT_SESSION.md). |
| Window pages and rows | **Window Options** is the final page before hidden and always remains in the L+R+Down cycle. It carries PluginsPage / OwnPages / PerfPage, PerfView and Hints. Every value edit needs ZR without ZL in every scene; Up/Down selects and `< value >` marks the held edit. [Controls](CONTROLS.md), [Configuration](CONFIGURATION.md). | Host tests, operator-confirmed 0.6.0/0.6.1 page/title, and the complete 0.7.0 offline Eden page cycle. The capture shows a ZR-guarded NetLatency round trip and companion FX edits. The full every-scene positive/negative matrix and console Clock check are not recorded individually. |
| Plugin menu API | `ssbu_online_deluxe_plugin_api_v1`; SDK and ABI crates remain 5.0.0, SDK sends descriptor version 4 (header and inline rows, 64 options). The 0.8.0 host accepts revisions 2–5 including passive status cells from `0d28adc`; 0.7.0 accepts only 2–4. [Plugins](PLUGINS.md) | Console 2026-10-16: FX Control's page registered with descriptor 4 on the first good 0.5.3 boot. Status-cell layout/navigation has host coverage; runtime validation remains pending. |
| Native CSS / arena | Fork CSS pane path, room-ID arena hooks, saved-style Streamer restoration | CSS edits, closed-window display and match return verified in Eden; two-peer arena and Background matches ran in Eden on 2026-10-18, but the `deluxe_rewrite_arena` and `deluxe_rewrite_background_matchmaking` hook sites are mis-addressed and never fire (open finding, below). [Details](NATIVE_DISPLAY.md) |
| Embedded / standalone ImGui | Export/ABI detection, one active backend, namespaced hooks | Load-order tests and the Oct 9 frame-ownership correction passed in Eden; limits in [ImGui dependencies](IMGUI_DEPENDENCY.md) |
| Elite Legal Stages | Separate plugin, standalone ImGui retained | Both menus and settings persist; offline stages unrestricted as intended; NSO filtering untested |
| Character FX | Moved to the separate [ssbu-fx-control](https://git.syntaxbyexample.com/intpa/ssbu-fx-control) plugin in 0.5.0; Deluxe has no effect hooks and exports the online context the plugin reads. FX Control 0.2.1 has per-move numbered Reduced variants for the ten ledger fighters and the SDK pinned to `a63bc8c`. | Clean 0.2.1 registration with descriptor 4 was checked against Deluxe 0.5.4 in Eden. The later `8bf2…` development menu was captured with Deluxe 0.7.0. On 2026-10-25 the operator reported FX testing in Eden/on Erista without issues; no build hash or Reduced performance acceptance accompanied that report. |
| Match and CSS capture | `CAPTURE_MATCH` / `CAPTURE_CSS` boot flags stream timing and API CSVs; one `capture.rs` pair since 0.5.4; CSV headers carry build and clock (`# clock_mhz`, `# clock`, `# hoc`) | Shipped in 0.4.0 and used for every console ledger capture; the 0.5.4 consolidation keeps the CSV output byte-identical in host tests, console parity still owed. [Captures](CAPTURES.md) |
| Clock probe | Horizon-OC file checks at boot, a 5 s add-latency MHz estimate, event code 111, status lines, Performance Clock row (0.5.3) | Console event log (2026-10-27 boot, read 2026-10-28): about 1020 or 905–915 MHz at stock, 1780–1789 MHz under Smash's Horizon-OC profile or the game's loading boost. The ~910 readings also appear in 2026-10-18 logs, before the game-aware fork existed ([Validation](VALIDATION.md)) |
| Horizon-OC online-only profile (0.8.1) | `hoc_oc=online` lets Smash's saved Horizon-OC profile apply only while the shared online session is live (Pia RTT samples, internet or local wireless, 15 s hold). Client of the game-aware hoc-clk fork (protocol v1, `hoc:clk` commands 100–103, API version still 2) on its own thread; handshake-gated toggle (ZR+L3 in 0.8.1; L+R+X on the framed You-line clock since 0.9.0, which also keeps a white clock in `always`), You-line clock mark and Performance Clock state; events 114/115; `DISABLE_HOC` | Host-tested (framing golden bytes against libnx's layout, handshake classification, gate, heartbeat, config, chord). Eden integration with the test-only `tools/hoc-standin` (2026-10-27): handshake present (event 114 `a=4`), Arm with the kernel PID and the SetApply heartbeat seen by the stand-in, grey clock, ZR+L3 and the Performance Clock state checked on screen; the run found and fixed Eden's broken TIPC presence reply (CMIF on `platform=emulator`) and a clipped Performance row ([Validation](VALIDATION.md)). Console (2026-10-27): the first card install (`cc6d8fdff592`) failed the handshake with `LimitReached`, because a game may hold only one `sm:` session and Smash's SDK owns it. Eden does not enforce that. Fixed in `788537d` by borrowing Smash's session. The operator confirmed the fixed build on console: the grey clock, ZR+L3 on and off, green during a live online match, and grey again after the match. Its event log, read back on 2026-10-28, agrees: handshake `a=4`, two toggle round trips, the profile applied from the first Pia sample through the match (1780–1789 MHz measured), and released 90 s after the last sample, the hold that candidate used. Release 0.8.1 (`84da224`) was tested by the operator on console on 2026-10-28, including rematches. Its event logs show stock clocks in offline Training and Doubles, the profile applied for seven online Singles matches (most readings 1770–1790 MHz) and kept across two rematches, and release about 15 s after the opponent's last sample ([Validation](VALIDATION.md#081--console-rematches--2026-10-28)). The fork is source-only on Forgejo, [intpa/horizon-oc-game-aware](https://git.syntaxbyexample.com/intpa/horizon-oc-game-aware) branch `game-aware/2.6.0` on Horizon-OC `2.6.0-rel` `7960dd9`; no release or binary yet |
| Shared online session (0.8.1) | One network-proven online signal for every consumer, live until 15 s after the last Pia RTT sample; Auto drops an earlier match's latency mark at a new match stage once that session has ended (event 107 `a=3`) | Console 2026-10-28 (0.8.1): the Horizon-OC gate stayed live across two rematches and released about 15 s after the opponent's last sample. Auto's use logged once, on the offline Doubles stage after a session ended (107 `a=3`); otherwise host-tested |
| ZL-gated native edits | Native pane D-pad edits need ZL held (0.4.1) | Host-verified; console results in VALIDATION.md |
| Coverage build | Cargo feature `coverage`, raw profile writer, event code 109, manual snapshots from the Debug view (0.5.2) | Eden only. 2026-10-17 trace: 1,192 instrumented functions, 1,047 executed, 145 never executed. 2026-10-18 LAN Online run (two modded peers, NSO scenes only): 893 executed, 299 never executed, 12 functions new against the 10-17 report; [DEBUGGING.md](DEBUGGING.md) |
| Removed in 0.5.4 | The opt-in peer observer, FPS++ (emulator-only diagnostic), the Gigaflare DRS trigger, five opt-in diagnostics, 21 helpers without a production caller; `peer_observer=` and `fps_boost=` are ignored legacy keys. Per-file reduction: `src/` 22,253 → 15,332 production lines (−31.1 %) | Host gates on every step; Eden latency, quota frame-rate, capture and plugin-page passes on the candidate. Console: the 0.5.4 first boot is the acceptance test. [FPS boost history](FPS_BOOST_INVESTIGATION.md) |

## Current: 0.9.0

Tagged `v0.9.0` on 2026-10-31. It releases the `fix/lobby-practice-window`
work that has run on the operator's Switch as candidate `3ea3c49` since
2026-10-28: the player modes as You-line icons (ZR+Left/Right frames Stealth,
Streamer or the OC clock, and L+R+X toggles the framed one; L+R+X acts only on
the highlighted row), waiting-room practice counted as Training, one log per
stage under `matches/`, present-pacing diagnostics (codes 117–119) and the
refresh-clock plugin's 120 Hz counters (code 120). The branch's source is
merged unchanged.

It is also the first release built with LTO. The library was both a cdylib
and an rlib, and Cargo skips LTO for that combination, so the profile's
`lto = "thin"` never applied. Resident size (`.text` + `.rodata` + `.data` +
`.bss`) is 1,480 → 1,172 KiB against the card's `3ea3c49` build, with the same
exports. Build, Eden check and card install: [Validation](VALIDATION.md).

## 0.8.1

Published on Forgejo on 2026-10-28 from clean tag `v0.8.1` / `84da224`.
Both public assets were downloaded and hash-verified, and the same NRO is
installed on the operator's Switch; exact build, hashes and the rollback path
are in [Validation](VALIDATION.md#081--public-release-assets-and-card-install--2026-10-28).

0.8.1 adds the online-only mode for the game-aware Horizon-OC fork
([source](https://git.syntaxbyexample.com/intpa/horizon-oc-game-aware), branch
`game-aware/2.6.0`). With `hoc_oc=online`, toggled by **ZR+L3** on Settings
in 0.8.1 (by L+R+X on the framed You-line clock since 0.9.0),
Smash's saved Horizon-OC profile applies only while the shared online session
is live: from the first Pia RTT sample (internet or local wireless) until 15 s
after the last one. A clock after the You-row privacy icons shows the state,
and the Performance **Clock** row repeats it. Deluxe sends no clock value and
never touches Horizon-OC's `config.ini`. With stock Horizon-OC, or none,
nothing changes. On a console the client borrows Smash's own `sm:` session,
because a game may hold only one. The same session signal lets Auto drop an
earlier match's latency mark (event 107 `a=3`).

The operator tested candidate `788537d`, which released 90 s after the last
sample, and then 0.8.1 with the 15 s hold on 2026-10-28, including rematches.
The event logs agree. See
[Validation](VALIDATION.md), [next checks](NEXT_SESSION.md), and the
[NRO and manifest](https://git.syntaxbyexample.com/intpa/ssbu-online-deluxe-rewrite/releases/tag/v0.8.1).

## 0.8.0

Published on Forgejo on 2026-10-25 from clean tag `v0.8.0` / `d92bc3d`.
Both public assets were downloaded and hash-verified; exact build and hashes
are in [Validation](VALIDATION.md#080--public-release-assets--2026-10-25).

0.8.0 fixes the stale `Auto (Nf)` observation across consecutive matches. It
samples the same game buffer byte at the guarded network-consumption site
`0x16d81a8`, with a match-generation token captured before reading. Every new
Singles/Doubles stage starts unknown, stale callbacks cannot cross a lifecycle
boundary, and valid observations can update during play. Results/rematch CSS
retain the last count; Offline/Training clear it. A manual-to-Auto change stays
plain Auto until a fresh match. Existing manual buffer application is unchanged.

It also includes the already-merged revision-5 passive plugin status cells and
navigation changes from `0d28adc`. SDK/table versions remain unchanged.
Host regression tests and the original 13.0.5 instruction bytes support the
fix; **live online timing and status-cell runtime validation remain pending**.
See [Validation](VALIDATION.md), [next checks](NEXT_SESSION.md), and the
[NRO and manifest](https://git.syntaxbyexample.com/intpa/ssbu-online-deluxe-rewrite/releases/tag/v0.8.0).

## 0.7.0

Tag `v0.7.0` and version commit `9a7f81c` are on Forgejo. The
[release NRO and manifest](https://git.syntaxbyexample.com/intpa/ssbu-online-deluxe-rewrite/releases/tag/v0.7.0)
were published on 2026-10-25; both public downloads match the verified clean
package. The earlier API access block is resolved. Exact artifact and evidence:
[validation](VALIDATION.md).

0.7.0 (2026-10-22) is the minor version for the compact Player Info menu and
in-place LLUltra resolution chord accepted after 0.6.2.
Settings keeps You/Opp summaries and two editable controls: **You** carries
font-independent Stealth eye and Streamer TV geometry
([privacy-indicators.svg](privacy-indicators.svg)); **ZR+X** / **ZR+Y** toggle
privacy only while the **You** row is selected (no Stealth/Streamer rows; the
match strip reuses the icons). Console **NetProfile** has one manual **LLUltra**
stop labeled **LLUltra** / **LLUltra+** / **LLUltra++** from the saved
`ultra_resolution`; Left/Right still leave for adjacent profiles and recall the
saved level on re-entry. While manual console LLUltra is selected, **L+R+X**
cycles 576p → 720p → 1080p → 576p in place and leaves Auto and Vanilla+
unchanged; every other profile keeps the Vanilla+ toggle on that chord. Config
keys, wire ABI, rendering code and the plugin API are unchanged from the
operator-tested candidate.

**Operator-tested install (recorded separately from the clean release package):**
on 2026-10-22 the operator installed candidate NRO SHA-256
`2f5a1d4e4d99c58e0b351fd3af72e4f7e1cf121e017357d130a18fb2b151fc01` (build id
`0.6.2-d86fac2a7ade-dirty-1792624156`, still stamped 0.6.2 in the binary) and
confirmed on the Switch the **LLUltra → LLUltra+ → LLUltra++ → LLUltra** cycle
with **L+R+X**. That package is not the authoritative **0.7.0** release asset;
the clean release is built separately from the `v0.7.0` version commit.
Host gates for the version-bumped tree are in
[VALIDATION.md](VALIDATION.md); compact-menu visual checks remain in
[NEXT_SESSION.md](NEXT_SESSION.md).

**Offline UI refresh, 2026-10-24:** the clean released `2ff456…` binary was
captured in Eden with the FX Control `8bf2…` companion. The compact page,
privacy icons/round trips, guarded NetLatency edit, complete page cycle,
Training window, match strips and native return banners are shown in
[Screenshots](SCREENSHOTS.md). This adds bounded emulator UI evidence; the
console, connected-peer and negative-control checks remain separate.

### Post-v0.7.0 status cells (included in 0.8.0)

Commit `0d28adc` (2026-10-24) adds descriptor revision 5 passive status cells
to plugin pages ([Plugins](PLUGINS.md#status-cells-080)). It first ships in
0.8.0; the v0.7.0 host refuses a revision 5 descriptor with `REGISTER_ERR_ABI`.
The SDK still sends revision 4. Runtime status-cell validation remains pending.

### 0.6.2

0.6.2 (2026-10-19; released as Forgejo v0.6.2 — see the v0.6.2 release manifest
for the NRO hash) makes Auto say which buffer the game chose: with NetLatency on
Auto the label reads `Auto (4f)` (`Auto (25f)` is the longest), on the window's
You line, the NetLatency row, the in-match strip's `Net:` field and the native
character-select and arena `You:` line. The operator asked for it after booting
0.6.1. The number is the game's own buffer byte, which the online-latency hook
(event code 9) reads before applying this mod's selection, so it is recorded only
while Auto is selected and is cleared when the connection returns to Offline;
before the first online match of a session the label stays plain `Auto`, and the
local-wireless lobby's one-digit pane keeps plain `Auto`. The native You line's
latency field grew from 4 to 10 characters (`YOU_LINE_MAX` 32 → 38, `TEXT_CAP`
692 → 698). Manual buffers, the peer exchange (the Opp line still shows the
opponent's announced `Auto`), the INI keys, the event codes, the chords and the
plugin API are unchanged; the original Online Deluxe showed only `Auto`, so this
is a deliberate difference ([FORK_PARITY.md](FORK_PARITY.md)). The
[published v0.6.2 notes](https://git.syntaxbyexample.com/intpa/ssbu-online-deluxe-rewrite/releases/tag/v0.6.2)
record that the operator tested it online on the Switch on 2026-10-19 and
`Auto (Nf)` showed correctly. Step 16 of [NEXT_SESSION.md](NEXT_SESSION.md)
retains the fuller comparison checklist, not a pending first boot.

### 0.6.1

0.6.1 (2026-10-19; released as Forgejo v0.6.1 — see the v0.6.1 release manifest
for the NRO hash) retitles the Player Info window's last page **Window Options**.
The operator booted 0.6.0 on the console the same day, said it works and the menu
is much cleaner, and asked for one change: a real name for the last page, which
0.6.0 titled just Window. The change is the page's ImGui title string; the rows,
chords, INI keys, event codes and the plugin API are as in 0.6.0, and the rest of
the code is the same. 0.6.1 was booted on the console on 2026-10-19 and the
operator confirmed the title.

### 0.6.0

0.6.0 (2026-10-19; Forgejo release v0.6.0; the NRO SHA-256 is in that release's
manifest) redesigns the Player Info window's first page. The operator called the
old Settings page bloated: one table of up to 14 rows in three columns (label,
You, Opponent) with Name, Interface, Peer, Ping, NetLatency, NetProfile, Status,
Stealth, Streamer, UltraRes, PluginsPage, OwnPages, PerfPage and PerfView,
followed on the same page by the 11-row Performance block and the 180-sample
frame-time plot. Now the Settings page has two columns and no opponent column:
a read-only **You** line (`<profile> | <buffer> | <interface>`), an **Opp** line
while the peer shows (`<name> | <announced profile> | <announced buffer> | <ping>
| <interface>`, facts the peer has not announced left out, painted in the ping's
stability colour, selectable so that ZR + Left/Right cycles the remote station),
five rows (NetLatency, NetProfile with the old Status row folded into its value,
Stealth, Streamer, UltraRes on a console), a one-line performance footer in the
frame-time colour and a dim hint line (INI `hints`, default true, also the Window
Options page's Hints row). The page toggles and PerfView moved to a new **Window
Options** page (0.6.0 itself titled it just Window; retitled in 0.6.1), the last
page of the L+R+Down cycle (settings → plugins → own pages → performance →
window options → hidden), which Settings and Window Options always keep in the
cycle; PerfView now affects the Performance and Debug pages only. While ZR is
held the selected row's value reads `< value >`. The in-match strip's first line
is `Net`, `Profile`, `Stealth`, `Stream` and `UltraRes`, and its second line is
the `>Opp:` summary followed by the timing line; the `Peer:`, `IF:`, `Ping:`,
`PeerNet:` and `PeerProfile:` fields are gone. Nothing else changed: the
Performance and Debug pages, plugin pages, the native CSS and arena banner, every
chord, the event codes and the peer exchange are as in 0.5.7, and the only new
INI key is `hints`. Host-tested, then **booted on the console on 2026-10-19**:
the operator reported that it works and the menu is much cleaner, and asked for
one change, a real title for the last page, which is 0.6.1. The checks in step
15 of the test plan in [NEXT_SESSION.md](NEXT_SESSION.md) were not reported one
by one; no Eden visual check had run at that 0.6.0 checkpoint. The later 0.7.0
offline review is recorded above. The
exact package and gates are in the 0.6.0 entry of
[VALIDATION.md](VALIDATION.md).

### 0.5.7

0.5.7 (2026-10-18; `282a2df`, Forgejo release v0.5.7, installed on the card 2026-10-19 00:11Z over 0.5.6, NRO `27cadcd719d549da5c0fc27cce9c7e752c6dcd17c20cebba443405ae039876c8`) puts the Streamer chord
behind ZR: L+R + **ZR** + Y toggles Streamer, the ZL variant does nothing.
Reason: after the evening's 0.5.6 online session the card's saved config had
`streamer=true` and the operator had never meant to hide the opponent; the
chord had flipped it unnoticed during window use (ZL is also Training's
controlled-fighter swap modifier). ZR is already the hold every window edit
needs. Host-tested; no console run of 0.5.7 yet.

### 0.5.6

0.5.6 (2026-10-18; commit `fb2012a`, Forgejo release v0.5.6, NRO `df757ce2f8130fddd0c2ba69e35c0f2dd7eae5505365ce969ec88134a2fadad9`) makes
**Vanilla+** the default mode, on the operator's decision of 2026-10-18: on by
default, toggled by L+R+X rather than a separate menu entry. With the new
INI key `vanilla_plus` on (the default), the Vanilla profile keeps the game's
three-image swap chain but presents the acquired image at the flush that
carried that frame's last submission, one CPU update earlier than the game
presents it, as LessLag (0.4.0), LLUltra and LLDoubles (0.5.2) already do. The
shipped default is therefore Vanilla+, and so is Auto's Vanilla choice outside
matches. The reasoning is code plus console traces: 0.5.4 in Vanilla at 60 fps
in Training shows PRESENT(idx), ACQUIRE(idx+1), about 6 ms, the single flush
span, about 9 ms of update, then the next present of the index acquired at the
previous loop head, so the acquired image is complete at the flush and
presenting there saves about 9–13 ms of display latency; the ring, the
acquire, the loop bookkeeping and the fighter simulation are untouched.
There is no new NetProfile entry; L+R+X (neither ZL nor ZR held) toggles the
flag and saves it, and `early_present=false` still disables every early
present.

Two more operator changes landed in the same release (request of 2026-10-18
21:16Z): enable 1080p LLUltra, and require ZR to change any value while the
menu is open, outside Training and matches too. The same request dropped the
full effects testing and reported that an overclock held 720p LLUltra at a
locked 60. **UltraRes 1080p:** `ultra_resolution=1080` and the UltraRes row
(576p → 720p → 1080p → 576p with Right, backwards with Left) let console
LLUltra render at the native 1080p, with dynamic resolution still stepping down
from it; the default stays 576 and emulator profiles ignore the key. The
motive is the operator's report that **Horizon OC: LLUltra at 720p held 60
(operator-reported, 2026-10-18, no capture)**; 1080p is the next cell to try.
**ZR hold for every window edit:** with the Deluxe window open, a Left/Right
value change on any Settings or plugin row needs ZR held in every scene (until
0.5.5 only while a fight was live); a plain Left/Right is ignored and Up/Down
still move the selection, so a menu D-pad press or a taunt cannot change a
setting. The window-closed native pane edits (ZL + D-pad) are unchanged.
**The FX measurement cells A–R are dropped** by the operator; the ledger stays
in the evidence root.

Host gates: 237 + 1 tests, clippy with warnings denied, the
no-default-features build. **Console:** installed on the card on 2026-10-18 over 0.5.5 (rollback archive kept) and booted the same evening: clean boot, Vanilla+ with 0 early-present mismatches over 89,500 presents in menus, CSS and Training, one online Singles match (details in the 0.5.6 VALIDATION entry); LLUltra 1080p and the ZR-hold round trip are unchecked.
The checks owed are the Vanilla+, 1080p LLUltra and ZR-hold rows of the test
plan in [NEXT_SESSION.md](NEXT_SESSION.md); the full account is the 0.5.6
entry of [VALIDATION.md](VALIDATION.md).

### 0.5.5

0.5.5 (2026-10-18; commit `6a9e61a`, Forgejo release v0.5.5, NRO `cf57cf80f25f2e837deabaf859489dc00653692c269fb4a6e2069a6a1b9bf8c2`) is a patch on
0.5.4 for the operator's console report that a declined rematch leaves the
opponent's player info on the screen of the side that chose Yes. Eden did not
reproduce it: in LAN Online a decline ends the Pia session, which clears the
peer, while the console's decline is assumed to be a graceful leave that
Deluxe handles only through the station-left hook, whose protocol gate drops
a leave with a foreign protocol id and which has no TTL behind it. The fix is
the live-peer requirement: the opponent column, the strip's `Peer:` line, the
Ping cell, the peer-dependent rows and the native CSS remote line need an RTT
sample for the selected station within 10 s (since 0.6.0 those places are the
Opp line and the strip's `>Opp:` line), and event code 112 records each
time a bound station goes silent. Host gates: 235 + 1 tests, clippy with
warnings denied, the no-default-features build; the Skyline build is the
release asset. **Not verified on the console**; installed on the card on
2026-10-18 over 0.5.4 and replaced by 0.5.6 the same evening without booting. The checks owed are step 11 of the
test plan in [NEXT_SESSION.md](NEXT_SESSION.md); the full account is the
0.5.5 entry of [VALIDATION.md](VALIDATION.md). The station-left hook is still
uninstrumented, so which leave variant the console sends stays unknown.

### 0.5.4 and the state carried over

Releases v0.4.0 to v0.5.1 and v0.5.4 are on Forgejo; 0.5.2 and 0.5.3 were
card builds without a release. 0.5.4 is the per-file reduction pass merged
from `reduction-candidate` (the merge was briefly labelled 0.6.0; that tag and
release were removed, and the 0.6.0 above is a later, unrelated release). Gates on `a63bc8c`: clippy with warnings denied, the
no-default-features build, 233 + 1 host tests, Skyline release build.

Historical card state on 2026-10-17: the 0.5.4 NRO had been installed over DBI MTP
but had not yet booted. It sat on top of other then-untested changes: FX Control
0.2.0 (`0cffbac`), the 0.5.3 builds since the last good boot (`hide_pages`
and page rows, `perf_view`, the clock probe, the L+R-only chord) and the CFW
update to Atmosphère 1.12.1 with Horizon OC 2.6.0. The last pair known to boot
at that checkpoint was Deluxe `48cce4f2` + FX Control `f3d52a38` (2026-10-16).
Deluxe 0.5.4 subsequently booted on 2026-10-18; this is not current card state.
The historical checklist is in
[NEXT_SESSION.md](NEXT_SESSION.md).

Console results so far, newest first:

- 2026-10-18 (operator-reported, uncaptured; Horizon OC 2.6.0): LLUltra at 720p
  held a locked 60. No capture exists, so the build, clock state and moves
  played are unrecorded and the observation does not enter the ledger. 1080p
  is the next cell to try (0.5.6 offers it).
- 2026-10-16 (`48cce4f2` + FX `f3d52a38`): booted normally after the bootstrap
  chaining fix; FX Control's page registered (descriptor 4). Kirby Hammer with
  effects On held 60 under LLUltra for a 474 s Training session, window open or
  closed; the Kirby row was never set to Reduced, so the variant cells were to
  be redone (dropped by the operator on 2026-10-18).
- 2026-10-15 (0.5.2 build `1525e3e0`): seventeen move captures; Vanilla and
  LessLag hold 60 everywhere, LLUltra drops on most moves because CPU work and
  GPU time must fit one vblank under its immediate present.
- 2026-10-15 (0.5.1): the render-opts teardown guard; the operator reported
  the build passed. A Training 50 fps reading was traced to the Training
  Modpack's Input Display (about 3 ms per frame), not to Deluxe.
- 2026-10-14 (0.4.0 to 0.4.2): present at the flush fixed the character-select
  stall (49.9 → 59.85 fps); the follow-ups (ZL-gated native edits, declined-
  rematch banner, Quickplay hold) are recorded in VALIDATION.md.

Open finding carried into the next work: the arena and background-matchmaking
hook sites (`deluxe_rewrite_arena` at `main+0x22d9fa0`,
`deluxe_rewrite_background_matchmaking` at `main+0x22da090`) never fire. The
2026-10-18 two-peer Eden run reached an arena lobby and a Background match
without either hook running, and read-only disassembly of 13.0.5 `main` puts
the first address in the middle of a scene-factory function and the second in
a factory whose class those flows do not create. Both addresses came from the
fork and have no instruction-word guard; the follow-up is to remove or
re-derive them and guard every fork-derived address. The modded-opponent
exchange is now covered (Eden, same run): it executes only with Stealth Off on
both peers, because Stealth On suppresses the beacon, and a new online session
is needed after the toggle. The code-reduction campaign is paused at the
operator's order, and the operator dropped the effects measurement cells
(per-move Reduced variants, then the resolution fallback) on 2026-10-18; the
move ledger stays in the evidence root.

## Historical candidates (October 8–10)

The October 8–10 candidates (`5c72`, `4c19`, `8328`, `b99f`, `d9ac`, `b19`,
the `c0e9`, `dbdb` and `b111` diagnostics) and their Eden and Switch evidence
are recorded in [VALIDATION.md](VALIDATION.md); the October 9 launch-crash
investigation is in [SWITCH_CRASH.md](SWITCH_CRASH.md) and the FX-era
Palutena and Sephiroth measurements moved with the FX documentation to the FX
Control repository. The content-fit HUD (`8328`) was the installed console
build until 0.4.0. The earlier text of this section is in the repository
history before the 0.5.4 update.

## Compatibility boundaries

The gamepad overlay passed the operator's Kirby and Palutena Switch matches and
returns to character select. The 0.5.2 resolution triggers have no hardware
result yet; present at the flush is console-verified for LessLag. Earlier
asymmetric Game & Watch-fix matches completed in LessLag and LLUltra with
matching visible results and return to CSS. These are bounded observations,
not simulation checksums or universal online-safety proof.

The rewrite installs no global Layout::Draw hook and does not patch SGPO's
`0x374812c` neighborhood. Character effects live in the FX Control plugin,
whose hooks chain original EffectModule requests and avoid gameplay articles,
hitboxes, cameras and RNG. Training Modpack's artificial input delay is
separate from Deluxe's online network buffer, and its Input Display costs about
3 ms per frame on the console.

The guarded fighter-Main site must match the supported 13.0.5 code. Another
plugin already owning that site causes refusal; do not assume arbitrary
Smashline stacks are compatible. DRS reports unavailable triggers explicitly.

See [validation](VALIDATION.md) for exact tested artifacts,
[frame methodology](LATENCY_METHODOLOGY.md) for timing boundaries,
[the emulator method](EMULATOR_METHOD.md) for the CPU-quota bias, and
[parity](FORK_PARITY.md) for deliberate differences from the fork.

## Compact-menu lineage

Earlier card-only candidates (`d9e1edd…`, `5a2a5f05…`) led to the accepted
`d86fac2` tree and the operator-tested `2f5a…` install documented in
[VALIDATION.md](VALIDATION.md). Cargo and the acceptance sections above now read
**0.7.0**. Historical package identities and
gates for the intermediate builds stay in VALIDATION without rewriting their
version stamps.
