# Implementation and fork parity

The default remains the global-menu/FX/diagnostics foundation with Vanilla rendering.
Vanilla, LessLag and LLUltra are selectable live and persist to INI. The new
transitions passed linked Eden tests; their console behavior remains untested.

| Behavior / fork change | Status |
|---|---|
| Quickplay input buffer, Auto/0–25 | Implemented without arena-only gate; live effect untested |
| Null latency pointer protection | Implemented at 13.0.5 site 0x16ccab8 |
| Manual profile survives auto selection | Tested portable policy; renderer not wired |
| Online Singles/Doubles defaults | Tested portable policy; renderer not wired |
| Offline-to-Quickplay guess | Corrected in policy: offline stays offline; FX-only scene/online-match tracking added, runtime validation pending |
| Reapply / repeated arena marking | External host removed; future engine has one authoritative state owner |
| 300 ImGui-callback transition grace | Global callback restored; future engine still needs explicit frame-boundary transitions |
| Stealth mode | Preference toggles and persists, but UI says Unavailable. Transmit backend is blocked on peer identity reconciliation; no custom packet writes |
| LessLag rendering | Live `lesslag`: two textures, OneBehind, poll bypass and render_opts sequencing; Eden matches and captured-image response evidence; console untested |
| LLUltra rendering | Live `llultra`: adds Immediate index, guarded pre-present flush via entry hook 0x38701e0, triple original flush, later-flush suppression; Eden matches and captured-image response evidence; console untested |
| Component modes | `lesslag_core` (no render_opts) and `doubles_core` (three textures, OneBehind) retained for isolation tests |
| LLDoubles | Not implemented beyond `doubles_core`; DRS and multiplayer checks pending; scheduled after the Switch crash work |
| Texture-wait relocation / VSync-event pacing | Profiling-only in v1.4.1 and v1.4.2, off in shipped presets; intentionally not implemented |
| Frame pacer | Left enabled, as in the modern LessLag and LLUltra presets |
| Runtime profile transitions / selection menu | Implemented for Vanilla/LessLag/LLUltra with buffer and index drains, confirmed count readback, and pending/active UI; Eden menus/CSS/match tested |
| v1.4.2 emulator crash fix | Backend persistent-hook design studied; crash mechanism not reproduced or established |
| Singles clocks for Ultra/Doubles | Recorded requirement; no automatic clock changes in M0 |
| catch_unwind around overclock | Not copied: panic=abort cannot recover through it |
| Dynamic resolution / performance scaling | Pending; these were upstream features |
| Auto (Nf) display | Current window shows Auto; actual game buffer display pending |
| Native CSS menu | Removed after live font/clipping problems; no game-pane writes remain |
| Global Player Info window | Embedded imgui-smash, settings/performance/hidden views; main-menu/CSS/match behavior confirmed in Eden |
| FPS / frame graph | Implemented from overlay callback intervals; not network timing |
| Response marker | Opt-in INPUT_RECEIPT pixel marker and API-trace HID records; diagnostics only, off by default |
| Streamer mode / peer diagnostics | Pending |
| FX On / Off | Palutena/Sephiroth live toggles, persistence, TM save/load and SGPO accepted; expanded catalog awaits tests |
| FX online-only scope | INI defaults to online; all is the offline+online test override; offline Training preservation verified, live online transitions untested |
| FX Reduced | Reserved and unavailable; config rejects it; defensive policy preserves visuals |

## Next work

Current focus: persisted manual profiles, Stealth parity, and one-sided G&W
compatibility. Manual profiles and both requested G&W mode tests passed in Eden.
Stealth requires more read-only identity evidence before packet writes.
The corrected gamepad overlay previously passed the user's two Switch matches;
the new profile transitions have not yet been tested on Switch. Doubles is
deferred. Reduced FX still follows an actual Switch performance reproduction.

The custom CLI Eden delivery passed Vanilla and LessLag-core Training smoke tests
with NRO 5d47c70452ce73ba (source 42a60af). The later uncommitted candidate
123d47db700f5d19 booted `lesslag` and `llultra` into the same Sephiroth/Kazuya
Training match with Training Modpack and SGPO. Its observation-only and active
Ultra route guards recorded no bad callers or alias mismatches. Accepted
captured-image results and their limits are in PERFORMANCE.md.

Remaining rendering work is Auto selection, dynamic resolution and performance
scaling, console clocks, full LLDoubles, and console validation of the new modes.
Peer metadata and Stealth transmission are also pending. One-sided G&W-fix
matches in both LessLag and LLUltra completed without observed desync; see
[validation](VALIDATION.md) for exact artifacts, results and limits.

Original snapshots and quickplay-changes.patch preserve exact fork changes.
Their presence is not feature completion.

## Compatibility

The default installs no Layout::Draw, HUD scene-tick, input-mapping, article or
camera hook. All modes install the preflighted rendering hook set so that
menu changes do not reinstall hooks. The committed policy controls render-index,
render-poll and flush behavior. LessLag and LLUltra enable fighter
recording (0x374bb04), early-dispatch (0x374c600), post-scene (0x374cd64) and model
update (0x3ab590) hooks. FX scope observes scene entry and online delay setup;
it does not mutate scene or network state. Its embedded ImGui backend interposes NVN bootstrap,
submit and acquisition for drawing the window. The user's overlay files are untouched.
Training Modpack input delay remains separate from the online network buffer.

Training Modpack shares EffectModule::req, req_follow, req_on_joint and req_screen.
These function hooks chain through originals. The target list excludes its generic death/score
effects and wing background. User confirmed the current load order, menus, FX and Training save/load coexist on
the installed candidate. Reversed load order and the expanded candidate remain untested.

The legacy pre-present site 0x3748128 is adjacent to SGPO's 0x374812c; no mode
patches either. `llultra` instead hooks the called function's entry at 0x38701e0.
That hook keeps the original three arguments, checks the saved return address
against the known caller, the owner alias and the virtual flush target, and
returns through SGPO's original path. It also replaces the later flush call and
reset at 0x3749b78/0x3749b7c. Preflight requires the expected BL at 0x3748128,
and installation verifies that it and SGPO's word at 0x374812c stay unchanged.
The guards have recorded zero anomalies in Eden; that is not proof for other
scenes, Switch or other overlays.

No claim of universal online safety, emulator compatibility, reduced input lag or improved
frame rate is made from compilation.
