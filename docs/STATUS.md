# Implementation and fork parity

M0 is a buildable global-menu/FX/diagnostics foundation. It is not the reconstructed render engine.

| Behavior / fork change | Status |
|---|---|
| Quickplay input buffer, Auto/0–25 | Implemented without arena-only gate; live effect untested |
| Null latency pointer protection | Implemented at 13.0.5 site 0x16ccab8 |
| Manual profile survives auto selection | Tested portable policy; renderer not wired |
| Online Singles/Doubles defaults | Tested portable policy; renderer not wired |
| Offline-to-Quickplay guess | Corrected in policy: offline stays offline; runtime mode tracking pending |
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
| Global Player Info window | Embedded imgui-smash, settings/performance/hidden views; main-menu appearance confirmed in Eden |
| FPS / frame graph | Implemented from overlay callback intervals; not network timing |
| Streamer mode / peer diagnostics | Pending |
| FX On / Off | Targeted policy and live toggle confirmed; persistence fix and actual move visuals under test |
| FX Reduced | Reserved and unavailable; config rejects it; defensive policy preserves visuals |

## Reconstruction order

1. Validate M0 live with On/Auto, then targeted Off/manual buffer.
2. Port render ordering using install-once hooks and one frame snapshot, based on the newer backend.
   Verify installed-but-Vanilla against M0.
3. Add index/VSync transitions, then double/triple buffering with explicit NVN readback.
   Only then expose working LessLag/Ultra/Doubles choices.
4. Restore network telemetry, performance scaling and optional clock support; add Reduced as a separate policy.

Original snapshots and quickplay-changes.patch preserve exact fork changes.
Their presence is not feature completion.

## Compatibility

M0 installs no Layout::Draw, HUD scene-tick, match lifecycle, input-mapping, article,
camera or swapchain-mode hook. Its embedded ImGui backend interposes NVN bootstrap,
submit and acquisition for drawing the window. The user's overlay files are untouched.
Training Modpack input delay remains separate from the online network buffer.

Training Modpack shares EffectModule::req, req_follow, req_on_joint and req_screen.
These function hooks chain through originals. The target list excludes its generic death/score
effects and wing background. Both load orders/save-state transitions still need runtime tests.

Future engine site 0x3748128 is adjacent to SGPO's 0x374812c. Fable traced Skyline's near
inline-hook pool and argues both replace one word. Retain a runtime patch-width check in engine
acceptance; M0 touches neither.

No claim of universal online safety, emulator compatibility, reduced input lag or improved
frame rate is made from compilation.
