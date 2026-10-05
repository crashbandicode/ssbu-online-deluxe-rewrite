# Implementation and fork parity

M0 is a buildable global-menu/FX/diagnostics foundation. It is not the reconstructed render engine.

| Behavior / fork change | Status |
|---|---|
| Quickplay input buffer, Auto/0–25 | Implemented without arena-only gate; live effect untested |
| Null latency pointer protection | Implemented at 13.0.5 site 0x16ccab8 |
| Manual profile survives auto selection | Tested portable policy; renderer not wired |
| Online Singles/Doubles defaults | Tested portable policy; renderer not wired |
| Offline-to-Quickplay guess | Corrected in policy: offline stays offline; FX-only scene/online-match tracking added, runtime validation pending |
| Reapply / repeated arena marking | External host removed; future engine has one authoritative state owner |
| 300 ImGui-callback transition grace | Global callback restored; future engine still needs explicit frame-boundary transitions |
| Stealth mode | No custom PIA telemetry/beacons emitted; ordinary game traffic continues |
| LessLag / Ultra / Doubles | Pending render-order, index/VSync and swapchain reconstruction; UI says Vanilla |
| v1.4.2 emulator crash fix | Static design reference only; not claimed implemented or reproduced |
| Singles clocks for Ultra/Doubles | Recorded requirement; no automatic clock changes in M0 |
| catch_unwind around overclock | Not copied: panic=abort cannot recover through it |
| Dynamic resolution / performance scaling | Pending; these were upstream features |
| Auto (Nf) display | Current window shows Auto; actual game buffer display pending |
| Native CSS menu | Removed after live font/clipping problems; no game-pane writes remain |
| Global Player Info window | Embedded imgui-smash, settings/performance/hidden views; main-menu/CSS/match behavior confirmed in Eden |
| FPS / frame graph | Implemented from overlay callback intervals; not network timing |
| Streamer mode / peer diagnostics | Pending |
| FX On / Off | Palutena/Sephiroth live toggles, persistence, TM save/load and SGPO accepted; expanded catalog awaits tests |
| FX online-only scope | INI defaults to online; all is the offline+online test override; live transitions untested |
| FX Reduced | Reserved and unavailable; config rejects it; defensive policy preserves visuals |

## Next work

The user chose [measured Reduced FX development](NEXT_SESSION.md) next: identify a
Switch dip, reproduce in Eden, fix one effect, test that candidate on Switch, repeat.
The expanded Off catalog must first pass tomorrow's move and scope smoke tests.

LessLag/Ultra/Doubles remain separate pending work: port render ordering with
install-once hooks/frame snapshots, verify installed-but-Vanilla, then validate
index/VSync and buffering transitions before exposing the profile choices.
Network telemetry, performance scaling and clocks are still pending.

Original snapshots and quickplay-changes.patch preserve exact fork changes.
Their presence is not feature completion.

## Compatibility

M0 installs no Layout::Draw, HUD scene-tick, input-mapping, article,
camera or swapchain-mode hook. FX scope observes scene entry and online delay setup;
it does not mutate scene or network state. Its embedded ImGui backend interposes NVN bootstrap,
submit and acquisition for drawing the window. The user's overlay files are untouched.
Training Modpack input delay remains separate from the online network buffer.

Training Modpack shares EffectModule::req, req_follow, req_on_joint and req_screen.
These function hooks chain through originals. The target list excludes its generic death/score
effects and wing background. User confirmed the current load order, menus, FX and Training save/load coexist on
the installed candidate. Reversed load order and the expanded candidate remain untested.

Future engine site 0x3748128 is adjacent to SGPO's 0x374812c. Fable traced Skyline's near
inline-hook pool and argues both replace one word. Retain a runtime patch-width check in engine
acceptance; M0 touches neither.

No claim of universal online safety, emulator compatibility, reduced input lag or improved
frame rate is made from compilation.
