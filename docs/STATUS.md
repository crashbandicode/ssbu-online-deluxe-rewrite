# Implementation and validation status

The rewrite is one Rust/Skyline NRO for Smash 13.0.5. Auto is enabled by
default; it chooses Vanilla in menus, offline play and Training, LessLag for
console online singles, LLUltra for emulator online singles, and LLDoubles for
online matches with more than two fighters. Platform is an explicit INI setting.
No hardware overclocking is implemented.

| Feature | Implementation | Runtime evidence / remaining check |
| --- | --- | --- |
| Vanilla / LessLag / LLUltra | Live guarded transitions, persistent manual choice | Earlier linked Eden matches and transitions passed; historical frame measurements are tied to their exact binary |
| LLDoubles | Three textures, OneBehind, render sequencing, resolution scaling | b19 linked Auto/LLDoubles team match completed through results/CSS with matching observations |
| Auto | Scene, connection and fighter-count policy; preserves saved manual choice | Training/menu Vanilla, b19 linked doubles LLDoubles, d9 linked singles LLUltra and cleanup Vanilla passed |
| Resolution / DRS | Applied internal resolution shown separately from output size | Console-policy Eden Gigaflare tests passed for LessLag, LLUltra and LLDoubles; Vanilla remains off; console hardware untested |
| Fighter Main | Guarded read-only fighter status callback, bounded report tokens | Two count-only Training sessions passed approximate tick-frequency checks; count mode installs no cut-in hook or reports |
| Cut-in scaler | Original cut-in function chained, seven eligible fighter-tick cooldown | Luigi close-range Up-B cut-in passed in Eden: LLDoubles 1080→720→1080; Switch untested |
| Online input buffer | Auto or 0–25, retained null guard and Quickplay path | User menu/persistence checks; actual network buffer changes need live online coverage |
| Stealth | Default On, connection-latched beacon/payload suppression, reconnect notice | d9 fresh-On suppression/zero custom writes, pending-Off and saved Off after restart passed; earlier pending-On passed |
| Peer display | Selected station name, RTT/stability colors and announced settings | d9 remote Auto/2 buffers, profiles, names, green RTT and interface announcements passed in linked play |
| Ethernet / Wi-Fi | Local nifm type and peer announcement, otherwise Unknown | Never inferred from RTT; an emulator may report a virtual interface |
| Streamer / peer cycle | Persistent display privacy, menu and fork chord, identity-consistent peer selection | d9 exact saved INI reload and privacy passed; more than two network stations needs additional consoles |
| Global Player Info | Small top-left ImGui window in menus, CSS and play | Confirmed in Eden and by user; retained alongside the native banner |
| Native CSS / arena | Fork CSS pane path, room-ID arena hooks, saved-style Streamer restoration | CSS edits, closed-window display and match return verified in Eden; NSO arena and Switch testing pending. [Details](NATIVE_DISPLAY.md) |
| Embedded / standalone ImGui | Export/ABI detection, one active backend, namespaced hooks | Embedded and both standalone load orders passed earlier tests; 4a solo coexistence passed; one concurrent run's external windows were invisible, cause unresolved |
| Elite Legal Stages | Separate plugin, standalone ImGui retained | Both menus and settings persist; offline stages remain unrestricted as intended; NSO filtering untested |
| FX On / Off | Named visual requests only, live for newly requested effects | User Palutena/Sephiroth tests and TM save/load passed; final Gigaflare suppression observed; expanded catalog not fully runtime-certified |
| FX scope | Online-only default; all for offline+online tests | Offline Training preservation checked; full online-to-offline transition matrix pending |
| Reduced FX | Reserved, unavailable; never silently mapped to Off | Requires a reproducible Switch performance case |
| Software FPS boost | Not implemented | Live guest watchpoint identifies native-window swap interval; Eden uses it for composition-speed control. [Investigation](FPS_BOOST_INVESTIGATION.md); boost A/B measurements pending |

## Current candidate

Current candidate `4a927dc20904722d` passes 192 host tests and the required
host/Skyline checks/build. The fork's native CSS title and the global window
work together. Solo validation covers stable sizing, edits, Streamer restoration,
CSS re-entry and coexistence with standalone ImGui/Elite/TM/SGPO, also with G&W.
The preceding CSS-equivalent `84d48448ac91e65d` completed an offline match and
returned through results to CSS. Linked CSS showed both peers' names, profiles
and green RTT; no match was started in that new pair.

One full-stack peer's external ImGui windows were invisible during that linked
run, despite callbacks and edits working. Its native title and the other peer's
embedded window remained visible. Solo tests with and without G&W passed;
the paired anomaly is unresolved. Arena NSO and console tests remain pending.
[Native display](NATIVE_DISPLAY.md) records the evidence and differences.

The previous candidate `d9acd03611e1d1d6` passed 176 host tests, host/Skyline checks with
warnings denied, the no-default-features build, and the optimized Skyline build.
The matching NRO, optimized ELF, source, hashes, raw checks and runtime evidence
are preserved outside Git. This candidate passed a G&W LessLag-versus-Auto/Ultra
match through results and CSS, with the fix on only one peer, Elite enabled,
standalone ImGui/TM/SGPO on A and embedded ImGui on B. Both d9 test sessions
exited normally on both peers, without memory-limit or OOM events.

The preceding b19 candidate passed 175 tests. Its renderer is unchanged in d9;
only four display/test files differ. The manual console-LLUltra crop bug is fixed:
an observed game fullscreen crop reset now rearms the smaller crop on the next
tracked present.

Twenty-four b19 response trials passed visual/marker checks; d9 was not remeasured.
Normal-phase response counts were Vanilla 6, LessLag 3, LLUltra 2 and LLDoubles 4
(three each); delayed-phase LLUltra varied 3,2,3 and other modes stayed the same.
See the methodology for the one idle-animation threshold correction, clock
boundaries and the distinction between manual LLDoubles Training and doubles.

The first four-player attempt stopped at character select when the guest's
second controller did not join. A later b19 test used three host controllers
and one guest controller and completed a real four-fighter team match, results
and return to CSS. Auto selected LLDoubles; the G&W fix was present on A only.
Both peers agreed on observed damage, stocks and results.

## Compatibility boundaries

The corrected gamepad overlay passed the user's Kirby and Palutena Switch
matches and returns to character select. New profile/DRS changes remain
console-untested. Earlier asymmetric G&W-fix matches completed in LessLag and
LLUltra with matching visible results and return to CSS. These are bounded
observations, not simulation checksums or universal online-safety proof.

The rewrite installs no global Layout::Draw hook and does not patch SGPO's
`0x374812c` neighborhood. FX hooks chain original EffectModule requests and avoid
gameplay articles, hitboxes, cameras and RNG. Training Modpack's artificial input
delay is separate from Deluxe's online network buffer.

The guarded fighter-Main site must match the supported 13.0.5 code. Another
plugin already owning that site causes refusal; do not assume arbitrary
Smashline stacks are compatible. DRS reports unavailable triggers explicitly.

See [validation](VALIDATION.md) for exact tested artifacts,
[frame methodology](LATENCY_METHODOLOGY.md) for timing boundaries, and
[parity](FORK_PARITY.md) for deliberate differences from the fork.
