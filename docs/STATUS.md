# Implementation and validation status

Updated 2026-10-19 for 0.6.1 (Forgejo release v0.6.1; the NRO SHA-256 is in that release's manifest; 0.6.0 is `a0dc5d6`, tag v0.6.0 at `fac09de`, Forgejo release v0.6.0, booted on the console 2026-10-19; 0.5.7 is `282a2df` + docs `a7e8427`, Forgejo release v0.5.7, on the card 2026-10-19 00:11Z; 0.5.6 is `fb2012a` + docs `6180325`, Forgejo release v0.5.6; 0.5.5 is `6a9e61a`, Forgejo release v0.5.5; 0.5.4 is `a63bc8c`). The rewrite is one Rust/Skyline
NRO for Smash 13.0.5. Auto is enabled by default; it chooses Vanilla (Vanilla+
since 0.5.6) in menus,
offline play and Training, LessLag for console online singles, LLUltra for
emulator online singles, and LLDoubles for online matches with more than two
fighters. Platform is an explicit INI setting. No hardware overclocking is
implemented; the clock probe (0.5.3) only records the clock state so that
performance captures can be classified.

Publication routing: `origin` is Forgejo
(`git.syntaxbyexample.com/intpa/ssbu-online-deluxe-rewrite`), authoritative
and public; `github` is a downstream mirror that is not pushed automatically.
Nothing is pushed without the operator's explicit permission. Every Forgejo
release carries the NRO and its SHA-256 manifest.

Evidence levels used below: **host** = Rust host tests, clippy with warnings
denied, the no-default-features build and the Skyline release build;
**Eden** = an emulator run on an isolated profile; **console** = a result from
the operator's Switch, recorded with its build hash in [VALIDATION.md](VALIDATION.md).

| Feature | Implementation | Runtime evidence / remaining check |
| --- | --- | --- |
| Vanilla / LessLag / LLUltra | Live guarded transitions, persistent manual choice; LLUltra's console internal resolution is `ultra_resolution` (576, 720 or, since 0.5.6, the native 1080; default 576) with the UltraRes row, which walks 576p → 720p → 1080p → 576p; dynamic resolution still steps down from it. [Rendering profiles](RENDERING.md#llultra-resolution-053-1080p-in-056) | Console 2026-10-15 (0.5.2 build): seventeen Training capture sessions, Vanilla and LessLag hold 60 on every ledger move, LLUltra alone drops (11 of 17 sessions fail "no second below 59"). LessLag online singles held 60. **Horizon OC: LLUltra at 720p held 60 (operator-reported, 2026-10-18, no capture); 1080p is the next cell to try.** The 576p row is not yet booted; the 1080p value is host-tested only (config parse, serialize, cycle and byte round trip) |
| LLDoubles | Three textures, OneBehind, render sequencing, resolution scaling | b19 linked Auto/LLDoubles team match completed through results/CSS; Eden response trials keep the two-image saving after present-at-flush (0.5.2) |
| Auto | Scene, connection and fighter-count policy; preserves saved manual choice; Quickplay hold and stale demotion (0.4.2) | Training/menu Vanilla, b19 linked doubles LLDoubles, d9 linked singles LLUltra and cleanup Vanilla passed in Eden; 0.4.2 console results in VALIDATION.md |
| Present at the flush | Two-image LessLag presents the acquired image at the flush that submits it (0.4.0); extended to every committed render-opts profile in 0.5.2; `early_present=false` switches it off (0.5.3) | Console-verified for LessLag on the character select (49.9 → 59.85 fps, 0.4.0). LLUltra/LLDoubles extension: Eden response trials (savings of 3, 4 and 2 images against Vanilla, unchanged through the 0.5.4 candidate); console LLUltra captures above. Before/after clips at the console-calibrated 175 % CPU quota in [EMULATOR_METHOD.md](EMULATOR_METHOD.md) |
| Vanilla+ | Vanilla keeps the game's three-image swap chain but presents the acquired image at the flush that carried that frame's last submission, one CPU update earlier than the game does (0.5.6). INI key `vanilla_plus` (default true, live); L+R+X toggles it and saves; the NetProfile row, strip and native banner read `Vanilla+` / `Vanilla+*` / `Auto:Van+`; `early_present=false` still disables it. Operator decision 2026-10-18: ship it as the default mode. [Rendering profiles](RENDERING.md#vanilla-056) | Host-tested only (237 + 1 tests). Reasoning from the 0.5.4 console API trace in Vanilla (the acquired image is complete at the flush; presenting there saves about 9–13 ms of display latency). First console boot 2026-10-18 evening: `early_presents` 89,483 = skips, `early_present_mismatches=0` over 89,500 presents in menus, CSS and Training; the visual check, the chord round trip and a latency comparison are still owed |
| Dispatcher backoff | Idle-pass sleep with growing pauses; `dispatcher_backoff=false` switches it off (0.5.3), `DISABLE_BACKOFF` unchanged | Eden before/after clips (sleeps per second and frame rate at the 175 % quota); the switch itself is not yet booted on the console |
| Resolution / DRS | Cut-in trigger (shipped); per-move catalog `drs_moves=` and the adaptive trigger `drs_adaptive=` with threshold/trigger/release keys (0.5.2, experiment). The built-in Sephiroth Gigaflare trigger was removed in 0.5.4 (commit 49fbfe1) | The cut-in trigger drops resolution on the console (ledger observation). The 0.5.2 triggers have been on the card since 2026-10-15 with no hardware result yet; the resolution-comparison cells were dropped from the test plan with the other FX cells (operator, 2026-10-18). LLUltra's base level is the UltraRes setting above |
| Fighter Main | Guarded read-only fighter status callback, bounded report tokens | Two count-only Training sessions passed approximate tick-frequency checks; count mode installs no cut-in hook or reports |
| Cut-in scaler | Original cut-in function chained, seven eligible fighter-tick cooldown | Luigi close-range Up-B cut-in passed in Eden: LLDoubles 1080→720→1080; console captures show the cut-in zoom pushes (56–59 fps seconds under LLUltra, 2026-10-16) |
| Online input buffer | Auto or 0–25, retained null guard and Quickplay path | User menu/persistence checks; actual network buffer changes need live online coverage |
| Stealth | Default On, connection-latched beacon/payload suppression, reconnect notice | d9 fresh-On suppression/zero custom writes, pending-Off and saved Off after restart passed; earlier pending-On passed |
| Peer display | Remote facts only for a selected connected station that is also live: a measured RTT sample within 10 s (`pia_rtt::LIVE_MS`, a new binding gets a 10 s grace), applied to the opponent column, the strip's `Peer:` line, the Ping cell, the peer-dependent rows and the native CSS remote line; event 112 `PeerStale` when a bound station goes silent (0.5.5). Since 0.6.0 the Settings page's opponent column, Name / Interface / Peer / Ping rows and the strip's `Peer:` / `IF:` / `Ping:` / `PeerNet:` / `PeerProfile:` fields are one **Opp** summary line (and the strip's `>Opp:` line) with the same live rule. Name, RTT/stability colors and announced settings; sixteen keyed RTT stations behind one lock (0.5.4) | 9e9 offline/connected/disconnected and two-remote-peer cycling passed in Eden; the 0.5.4 station table and the 0.5.5 live rule are host-tested only. The declined-rematch persistence the operator reported on the console could not be reproduced in Eden (a LAN-Online decline ends the session); console verification of 0.5.5 pending |
| Ethernet / Wi-Fi | Local nifm type and peer announcement, otherwise Unknown | Never inferred from RTT; an emulator may report a virtual interface |
| Streamer / peer cycle | Persistent display privacy, menu chord, identity-consistent peer selection; the window chord is L+R only since 0.5.3 (ZL+ZR does nothing) | d9 saved INI/privacy passed; 9e9 cycled two remote peers in a three-station lobby. The L+R-only chord is host-tested only, console pending |
| Global Player Info | Small ImGui window alongside native banners; right in Training and online lobbies, left in ordinary matches; one text builder, one style decision and one row model since 0.5.4. **0.6.0 main-page redesign:** the Settings page (title still Player Info) has two columns and no opponent column: a read-only **You** line (`Vanilla+ \| Auto \| Wi-Fi`), the **Opp** line while the peer shows (`Mario \| LessLag \| 3 \| 23ms \| Ethernet`, announced facts only, painted in the ping's stability colour, ZR + Left/Right cycles the remote station), five rows (NetLatency, NetProfile with the Status folded into its value, Stealth, Streamer, UltraRes on a console), a one-line performance footer (`60.0 fps \| 16.67 ms \| 1080p \| DRS on`) and a dim hint line (INI `hints`, default true). The 11-row Performance block and the plot are on the Performance page only. The in-match strip's first line is `Net`, `Profile`, `Stealth`, `Stream`, `UltraRes` and its second line the `>Opp:` summary plus the timing line | Placement user-confirmed on earlier builds; the 0.5.4 consolidation and the 0.6.0 layout are host-tested only, **console pending** (Settings in menus, CSS, Training and a match; the Opp line's colour with a live opponent; the strip in a match) |
| Window pages and rows | The **Window Options** page (0.6.0, titled just Window until 0.6.1), the last page of the L+R+Down cycle and always in it: `hide_pages` through the rows PluginsPage / OwnPages / PerfPage (0.5.3; Settings rows until 0.5.7), `perf_view` through the PerfView row (graph or number, now Performance and Debug pages only), and the new Hints row (`hints`); the Performance Clock row (0.5.3); ZR + Left/Right for every value edit (0.5.3 while a fight was live; **every scene since 0.5.6**, so a plain Left/Right in the open window is ignored in menus too, Up/Down still move the selection), with the selected value shown as `< value >` while ZR is held (0.6.0) | Host-tested (the overlay's `edit_hold` test; the runtime now sets the hold in every scene, with no new test; menu and config tests for the Window Options rows and `hints`); `hide_pages`, PerfView, Hints, the Window Options page and Clock are not yet booted. The ZR hold was exercised on the console on 2026-10-16 (variant edits in Training recorded in the FX log); the every-scene rule has not run on the console, and a menu round trip (plain Left ignored, ZR + Left changes, `< value >` marker visible) is owed |
| Plugin menu API | `ssbu_online_deluxe_plugin_api_v1`; SDK and ABI crates 5.0.0 send descriptor version 4 (header and inline rows, 64 options); the host accepts versions 2–4; plugins may add their own page (0.5.0). [Plugins](PLUGINS.md) | Console 2026-10-16: FX Control's page registered with descriptor 4 on the first good 0.5.3 boot; bootstrap hook chaining fixed the boot crash of the first v4 deploy. The 0.5.4 host consolidation is host-tested and Eden-checked (demo plugin and FX page on the candidate), console pending |
| Native CSS / arena | Fork CSS pane path, room-ID arena hooks, saved-style Streamer restoration | CSS edits, closed-window display and match return verified in Eden; two-peer arena and Background matches ran in Eden on 2026-10-18, but the `deluxe_rewrite_arena` and `deluxe_rewrite_background_matchmaking` hook sites are mis-addressed and never fire (open finding, below). [Details](NATIVE_DISPLAY.md) |
| Embedded / standalone ImGui | Export/ABI detection, one active backend, namespaced hooks | Load-order tests and the Oct 9 frame-ownership correction passed in Eden; limits in [ImGui dependencies](IMGUI_DEPENDENCY.md) |
| Elite Legal Stages | Separate plugin, standalone ImGui retained | Both menus and settings persist; offline stages unrestricted as intended; NSO filtering untested |
| Character FX | Moved to the separate [ssbu-fx-control](https://git.syntaxbyexample.com/intpa/ssbu-fx-control) plugin in 0.5.0; Deluxe has no effect hooks and exports the online context the plugin reads. FX Control 0.1.0 was released 2026-10-15; 0.2.1 (per-move numbered Reduced variants for the ten ledger fighters, SDK pinned to `a63bc8c`) was released 2026-10-18 and installed on the card | The plugin's own evidence is recorded in its repository. 0.2.1 registers with descriptor 4 against 0.5.4 in Eden; the console run is pending |
| Match and CSS capture | `CAPTURE_MATCH` / `CAPTURE_CSS` boot flags stream timing and API CSVs; one `capture.rs` pair since 0.5.4; CSV headers carry build and clock (`# clock_mhz`, `# clock`, `# hoc`) | Shipped in 0.4.0 and used for every console ledger capture; the 0.5.4 consolidation keeps the CSV output byte-identical in host tests, console parity still owed. [Captures](CAPTURES.md) |
| Clock probe | Horizon-OC file checks at boot, a 5 s add-latency MHz estimate, event code 111, status lines, Performance Clock row (0.5.3) | Host-tested only. The 1020 / 1785 MHz figures are a calculation from the A57 add latency, not a console measurement, until the first boot's event 111 is read |
| ZL-gated native edits | Native pane D-pad edits need ZL held (0.4.1) | Host-verified; console results in VALIDATION.md |
| Coverage build | Cargo feature `coverage`, raw profile writer, event code 109, manual snapshots from the Debug view (0.5.2) | Eden only. 2026-10-17 trace: 1,192 instrumented functions, 1,047 executed, 145 never executed. 2026-10-18 LAN Online run (two modded peers, NSO scenes only): 893 executed, 299 never executed, 12 functions new against the 10-17 report; [DEBUGGING.md](DEBUGGING.md) |
| Removed in 0.5.4 | The opt-in peer observer, FPS++ (emulator-only diagnostic), the Gigaflare DRS trigger, five opt-in diagnostics, 21 helpers without a production caller; `peer_observer=` and `fps_boost=` are ignored legacy keys. Per-file reduction: `src/` 22,253 → 15,332 production lines (−31.1 %) | Host gates on every step; Eden latency, quota frame-rate, capture and plugin-page passes on the candidate. Console: the 0.5.4 first boot is the acceptance test. [FPS boost history](FPS_BOOST_INVESTIGATION.md) |

## Current: 0.6.1

0.6.1 (2026-10-19; released as Forgejo v0.6.1 — see the v0.6.1 release manifest
for the NRO hash) retitles the Player Info window's last page **Window Options**.
The operator booted 0.6.0 on the console the same day, said it works and the menu
is much cleaner, and asked for one change: a real name for the last page, which
0.6.0 titled just Window. The change is the page's ImGui title string; the rows,
chords, INI keys, event codes and the plugin API are as in 0.6.0, and the rest of
the code is the same.

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
by one; no Eden visual check has run (the operator's yield order blocks it). The
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
**Vanilla+** the default mode, on the operator's decision of 2026-10-18
("ship it with the next version as vanilla+ and make it the default mode,
togglable by holding L+R+X instead of a separate scrolled entry"). With the new
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

Two more operator changes landed in the same release (message of 2026-10-18
21:16Z: "also enable 1080p llultra and make it so no matter what when the menu
is open you have to hold ZR to change even outside of training and matches.
I'm not going to do all the effects testing. an OC made 720p LLUltra locked
60"). **UltraRes 1080p:** `ultra_resolution=1080` and the UltraRes row
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

Card state on 2026-10-17: the 0.5.4 NRO was installed over DBI MTP and has
**not booted yet**. It sits on top of other never-booted changes: FX Control
0.2.0 (`0cffbac`), the 0.5.3 builds since the last good boot (`hide_pages`
and page rows, `perf_view`, the clock probe, the L+R-only chord) and the CFW
update to Atmosphère 1.12.1 with Horizon OC 2.6.0. The last pair known to boot
and play is Deluxe `48cce4f2` + FX Control `f3d52a38` (2026-10-16). The first
boot of 0.5.4 is the acceptance test; the checklist is in
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
