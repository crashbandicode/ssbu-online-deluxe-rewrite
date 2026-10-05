# Fable 5.1 review of the first one-NRO milestone

Reviewed 5 October 2026 (claude-fable-5-1, xhigh, pair verified from the session transcript). Scope: `src/runtime/{mod,logging,hooks,input,ui,effects}.rs`, `src/diagnostics.rs`, `src/effects.rs`, `src/menu.rs`, checked against `reference/quickplay`, skyline-smash `bed642e`, nnsdk 0.6.2 and the overlay. Static review only.

## Must fix before any deployment

**1. Banner pointer chain dereferences one level too deep (crash or memory corruption).** `src/runtime/ui.rs:41` does `let pane = *(second as *const *mut Pane)`. In both sources the chain ends at `second`: the fork uses `l2 as *mut Pane` (`reference/quickplay/src/net/mod.rs:173-178`) and the local slider casts the same value directly (`local-latency-slider/src/lib.rs:38`). The extra load reads the pane's first word, its vtable pointer, which passes the `>= 0x10000` and alignment checks, so `set_text` at `ui.rs:95` then treats game read-only data as a `TextBox`. Fix: `Some(second as *mut Pane)`.

**2. GameCube style bit is wrong, so the menu is dead on GC controllers.** `src/runtime/input.rs:18` tests `1 << 6`; `NpadStyleGc` is `1 << 5` (the overlay's tested constant, `smash-gamepad-overlay/src/config.rs:38`; `1 << 6` is Palma). A GC pad falls through to `GetNpadFullKeyState` (`input.rs:33`), which does not report that style, so `buttons()` returns 0 and no hotkey ever fires on the user's main controller family.

**3. The banner chain is now dereferenced in every CSS scene without identity checks.** The fork only walked `owner+0xe58 -> +0x10` behind `is_valid_online_mode()`; the slider only in local online. Offline CSS variants never exercised this chain, and `banner()`'s magnitude/alignment tests cannot tell a pane from any other heap object. Read `(*pane).name` once, log it, and gate `set_text` on an expected name constant filled from that first run; keep `DISABLE_MENU` documented as the fallback.

## Should fix in this milestone

**4. Effect hooks install without the entry-word record the other hooks get.** `src/runtime/effects.rs:325-327` calls `install_hook!` eight times with no `preflight`. Training Modpack already function-hooks `req` twice, `req_on_joint`, `req_follow` and `req_screen` (`UltimateTrainingModpack/src/training/mod.rs:371,542,599,638,686`), so one side always relocates the other's far-hook stub, and plugin directory order decides which. Both orders work because every hook calls its original and the Off rewrite never touches hashes Training Modpack inspects, but record the first word at each entry before install so the log shows who wrapped whom. Do not reject a far stub here; chaining is intended.

**5. Parity ledger items, not bugs.** The slider no longer shows the game's auto value ("Auto (Nf)"). Only the first banner pane is written; the fork wrote two and reset font metrics.

## Verified correct

- All eight `EffectModule` link names and argument lists match skyline-smash `src/cpp.rs:5010-5514` exactly, including `req_follow`'s `fbjiiiibb` tail and `req_on_joint`'s two trailing vector pointers. `Hash40` as a `#[repr(C)]` `u64` wrapper passed by value is ABI-equivalent to `phx::Hash40`.
- Off mode rewrites only the 111 named hashes to `hash40("null")` and still calls the original; only `edge_fire3_screen2` is short-circuited to 0. `hash40` reproduces the CRC-32 check value with the length prefix.
- `set_online_latency` keeps the fork's x19 null guard; `apply_input_buffer` writes only for 0..=25 and leaves Auto untouched.
- Hotkeys behave as specified: L+R or ZL+ZR held, Down toggles, Up alternates the two rows, Left/Right edit; edge detection resets on CSS owner change.
- `preflight` correctly refuses sites already carrying a near `B`, a `BR`, or an `LDR` literal, which is exactly what a stale fork or old slider install would leave at 0x1a12f60 and 0x16ccab8.
- No `Layout::Draw`, no scene-loop or swapchain sites, no 0x374812c neighbourhood, no `nro_hook` dependency; `legacy_host_present()` refuses the old host. Diagnostics never block a game thread, and the panic hook chains to the previous hook.
- Render engine absent and labelled `render_backend=not-installed`; `policy::select_profile` is unused by runtime, as documented.

## Primary disposition

Fixed findings 1/2: removed the extra pane dereference; GC style uses bit 5.
Finding 3: added QueryMemory range/permission checks and exact txt1 name gate.
Names txt_all_title_00/01 and txt_vip_title_00/01 were independently read from the
user's local chara_select.layout.arc; no asset bytes are included. Unknown panes
skip both display and input handling. The callback now updates before original,
matching the inspected fork and avoiding post-transition pointer use.
Finding 4: shared effect entries record the first instruction without rejecting chaining.
Finding 5: Auto actual-value and second-pane gaps are in STATUS.md.
New pointer-walk regression tests cover the exact extra-dereference failure.

## Global window update

Live CSS evidence later found set_txt_title_00 and confirmed interaction, but the
user required the fork's global window. CSS ownership/font changes are now removed.
Fable's subsequent source review established that imgui-smash uses NVN bootstrap
and submit interposition, without Layout::Draw; it implemented the embedded host
glue. The primary implemented the table, controller state and config recovery.
See VALIDATION.md for runtime evidence and remaining limits. Banner findings above
describe the earlier milestone, not the current runtime.
