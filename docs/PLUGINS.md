# Plugging into Online Deluxe

There are two ways another Skyline plugin can plug in. They are independent.

| You want | Use | Where |
| --- | --- | --- |
| Rows in Deluxe's menu (a toggle, a mode, a status) without writing any UI code | Deluxe's **plugin API** (`ssbu_online_deluxe_plugin_api_v1`) and the Rust SDK | this page, [sdk/README.md](../sdk/README.md) |
| Your own Dear ImGui windows drawn by the same renderer Deluxe uses | the **imgui-smash host** (`libimgui_smash.nro`) | [intpa/imgui-smash](https://git.syntaxbyexample.com/intpa/imgui-smash) |

## Menu rows in three steps

Your NRO describes its options once; Deluxe draws them on the Plugins page
(a vertical table outside matches, the compact two-row strip in a match),
asks you for the current value every frame and sends ZR + Left/Right as a
request (a plain Left/Right is ignored in the open window since Deluxe 0.5.6).
You own the behaviour and the saved state. The host is resolved at runtime, so
your NRO still loads when Deluxe is not installed.

1. **Depend on the SDK by path.** Clone the Forgejo repository at a release tag
   and add to `Cargo.toml`:

   ```toml
   [target.'cfg(target_os = "switch")'.dependencies]
   deluxe-plugin-sdk = { path = "../ssbu-online-deluxe-rewrite/sdk/sdk" }
   ```

2. **Describe the options and answer two callbacks.**

   ```rust
   use deluxe_plugin_sdk::{register, AtomicOptions, OptionSpec, PluginHandler,
                           PluginSpec, RequestStatus, RowLayout, Tone};

   static OPTIONS: AtomicOptions<1> = AtomicOptions::new([0]);
   static TONES: [Tone; 2] = [Tone::Neutral, Tone::Ok];

   static SPEC: PluginSpec = PluginSpec {
       id: "record",            // stable id, never shown
       label: "Record",         // shown; 24 bytes max
       options: &[OptionSpec {
           id: "enabled",
           label: "Enabled",
           choices: &["Off", "On"],
           config_key: Some("enabled"),      // host writes enabled=off|on …
           values: Some(&["off", "on"]),
           no_persist: false,
           tones: Some(&TONES),              // "On" draws green
           read_only: false,
           layout: RowLayout::Value,         // or OptionSpec::header() for a title row
       }],
       config_path: Some("sd:/ultimate/my_plugin/config.ini"), // … into this file
       own_page: false,  // true = a page of the window cycle titled "Record"
   };

   struct Handler;
   impl PluginHandler for Handler {
       fn current(&self, option: usize) -> Option<usize> { (&OPTIONS).current(option) }
       fn request(&self, option: usize, choice: usize) -> RequestStatus {
           (&OPTIONS).request(option, choice)   // or Pending / Rejected / Unavailable
       }
   }
   ```

3. **Register from `main`.** Skyline loads every NRO before running any
   `main`, so load order does not matter:

   ```rust
   #[skyline::main(name = "my_plugin")]
   pub fn main() {
       // read your own INI here and seed OPTIONS; the host never reads it back
       let result = register(&SPEC, Handler);
       skyline::println!("[my_plugin] register: {result:?}");
   }
   ```

That is the whole integration. On the shared Plugins page each plugin's rows
sit under a group row carrying its label (the strip shows the label, then the
option labels). A plugin with many rows (one per fighter, say) sets
`own_page: true` and gets its own page in the window cycle, titled with its
label and visited after the shared Plugins page; up to 64 options fit. Since
Deluxe 0.10.0 the player can take each own page out of the cycle on its own,
with a Shown / Hidden row titled with the plugin's label on Window Options
(saved by plugin id in `hide_plugin_pages`); the plugin needs no change.
`OptionSpec::header(id, label)` is a title row that Up/Down skip, for
grouping a long page. The SDK sends descriptor version 4 only; hosts from
Deluxe 0.5.3 through the released 0.7.0 accept versions 2, 3 and 4 under the
same rules (they share one layout), so plugins built against the 2.0.0 or
3.0.0 SDK keep registering, and an older host answers `RegisterError::Abi`
with no retry. 0.8.0 also accepts revision 5 ([below](#status-cells-080)). A plugin that
scopes its behaviour to online play reads
`deluxe_plugin_sdk::host_online_context()` instead of hooking the game's
scene sites itself: it returns the host's ready / online / match / training
bits, or `None` when Deluxe is absent or older than 0.5.0.

Plugins that put rows in Deluxe's window:

- [FX Control](https://git.syntaxbyexample.com/intpa/ssbu-fx-control), the
  first plugin built this way. It is the character-FX feature that Deluxe
  carried until 0.5.0, now a separate NRO with two own pages through the SDK.
  **FX Control** has a scope row, an all-fighters row, one row per covered
  fighter and, under a header per fighter, a value row per move selecting its
  Reduced variant (every non-header row is a `RowLayout::Value`). **Stages**
  has an All backgrounds row and an On / Off row per stage for its background
  visuals. The fighter rows' Online scope reads `host_online_context()`.
- CPU Pilot (ssbu-cpu-pilot): an own page, **CPU Pilot**, with the P1, Auto
  rematch and Auto save replay rows, through the SDK.
- TrainingRooms
  ([TrainingRoomsForUltimate](https://git.syntaxbyexample.com/intpa/TrainingRoomsForUltimate)):
  one row, **Replay capture**, under its **Training Rooms** group on the shared
  Plugins page, through the SDK.
- Legal Stages + Loser Counterpick
  ([elite-stages-counterpick](https://git.syntaxbyexample.com/intpa/elite-stages-counterpick)):
  an own page, **Stages + Counterpick**. Its legal stage pool limits the stages
  the game may pick to the ones you check. Its optimal counter pick (optional)
  affects an accepted online 1v1 rematch: under your preferred rules, it makes
  the game pick the loser's highest-ranked eligible stage. It doesn't use the
  SDK or link ImGui: it resolves `ssbu_online_deluxe_plugin_api_v1` itself and
  registers its page with status cells ([below](#status-cells-080)).

`request` is a request, not a command: return
`Pending` to apply a change later (the row shows `(pending)` until `current`
reports it), `Rejected` or `Unavailable` to refuse. A read-only option (or one
with a single choice) is a status row; give its choices tones to show
`Ready` / `Degraded` / `Failed` in green, yellow and red.

**L+R+X on a value row** (Deluxe 0.10.1) calls `request` with the choice
`current` already returns, once per press. A plugin that wants a second
action on a row can treat that as it (ssbu-cpu-pilot 0.2.1 steps the CPU level
on its P1 row); for any other plugin it is the request of the value it already
has and changes nothing. The host saves nothing for it and skips a row that is
waiting on a pending change. Older hosts never send it.

A status row is passive: Left/Right do nothing and `request` is never called.
Since 0.8.0 Up/Down also skip it (only an editable value row
holds the cursor, so a page with no editable row highlights nothing).

The complete example is [`sdk/examples/demo`](../sdk/examples/demo/src/lib.rs)
(three options, persisted INI, a time-driven status row). Limits, ABI layout,
error codes, threading rules and host persistence are in
[sdk/README.md](../sdk/README.md).

<a id="status-cells-unreleased"></a>

### Status cells (0.8.0)

Status cells, revision 5 and the passive-row navigation above first ship in
**0.8.0**, from commit `0d28adc`. The 0.7.0 host refuses a revision 5 descriptor
with `REGISTER_ERR_ABI`.

**Status cells (descriptor revision 5, `OPTION_FLAG_CELL`).** A run of 1 to 4
consecutive cell options is drawn as one table: each cell's label is a column
heading, separated by vertical bars, with a mark centered beneath it. The mark
comes from the tone of the choice `current` reports: `TONE_OK` draws a check
mark, any other tone a cross in that tone's color. Both are drawn as lines, so
no font glyph is needed, and the table shows no value text. `choice_tones` is
required (it is the ready/not-ready contract), and a cell cannot also be a
header or an inline row. Only a revision 5 descriptor may carry the bit; a
revision 2 to 4 descriptor that sets it is `REGISTER_ERR_INVALID`, and a host
built before revision 5 answers `REGISTER_ERR_ABI` before it registers
anything, so a plugin can then register the same rows as plain status rows
with a revision 4 descriptor. The SDK still sends revision 4 and has no cell
layout; a native plugin sets the flag in its own descriptor. Legal Stages +
Loser Counterpick is one: its **Pool | Results | Save** table is a run of
three cells in a revision 5 descriptor, and on `REGISTER_ERR_ABI` it registers
the same rows as plain status rows with revision 4. On the compact
in-match strip a cell run is one entry (`Pool:✓ | Results:✓ | Save:✗`, drawn
with the same marks), and the final strip page takes the passive rows that follow
the last editable row so they stay reachable.

## Window state (0.10.5)

Deluxe reads the controller through `nn::hid` and lets every press through to
the game, so a plugin whose own controller shortcuts overlap Deluxe's window
controls (the D-pad rows, ZR + Left/Right value edits) can ask whether the
window is up and stand its shortcuts down meanwhile. TrainingRooms (rc.9) and
the CPU pilot (0.2.4) do. The host exports:

```rust
#[repr(C)]
pub struct DeluxeOverlayStateV1 { pub size: u32, pub flags: u32 }   // size = 8
pub const DELUXE_WINDOW_VISIBLE: u32 = 1 << 0; // Deluxe's window is drawn (any page)
pub const DELUXE_ROWS_PAGE: u32      = 1 << 1; // the shown page has selectable/editable rows (D-pad and ZR+L/R act)
pub const DELUXE_DEBUG_VIEW: u32     = 1 << 2; // the Debug view is open
#[no_mangle]
pub unsafe extern "C" fn ssbu_online_deluxe_overlay_state_v1(out: *mut DeluxeOverlayStateV1) -> bool;
// Fills *out and returns true; false if out is null. Lock-free, safe to call from any game thread every frame.
```

- **Pages:** `DELUXE_ROWS_PAGE` is set on Settings, Window Options, the shared
  Plugins page and a plugin's own page; Performance and Debug have no rows.
- **Freshness:** the flags are the last draw's. A window that has not drawn for
  half a second reports `0`, so a stopped window never keeps another plugin's
  shortcuts off.
- **Consumers:** resolve the symbol with `nn::ro::LookupSymbol` once, and again
  on a miss at most once a second. A missing symbol means an older Deluxe:
  treat it as "no window". Never block on it.
- **Types:** `deluxe_plugin_abi` carries the struct, the bits,
  `OVERLAY_STATE_SYMBOL` / `OVERLAY_STATE_SYMBOL_C` and `OverlayStateFn`; the
  Rust SDK wraps the call as `deluxe_plugin_sdk::host_overlay_state() ->
  Option<u32>` with that lookup rule. Both crates stay at version 5.0.0, as
  when 0.8.0 added the revision-5 constants. No release tag has these yet
  (v0.10.4 doesn't); they are on `main`, which is 0.10.5.

## Your own windows through the imgui-smash host

Deluxe statically links `libimgui_smash.a`, built from
[intpa/imgui-smash](https://git.syntaxbyexample.com/intpa/imgui-smash) at
`9405a49`, now on that repository's `main` ([ImGui dependencies](IMGUI_DEPENDENCY.md)).
When a compatible standalone `libimgui_smash.nro` is installed, Deluxe reuses
that host instead, with that host's font (v1.0.1's is ASCII only, so names
outside ASCII show `?`); a standalone whose exports or ImGui version don't
match keeps Deluxe's window off ([Install](INSTALL.md#imgui)). A plugin that wants its own windows registers through the
host's exported `imgui_smash_*` callbacks and draws with matching Dear ImGui
code; the fork's
[developer guide](https://git.syntaxbyexample.com/intpa/imgui-smash/src/branch/main/docs/DEVELOPMENT.md)
has the steps, and its release carries the host NRO and the archive. Legal
Stages + Loser Counterpick, which replaces the original Elite Legal Stages NRO,
is not such a plugin: it links no ImGui and registers its settings page
through the plugin API above. Both integrations can run at once: Deluxe's
menu and your windows share one renderer and one controller.

## Verification

The Plugins page, tones and read-only rows were verified in Eden on
2026-10-12 with the demo NRO ([validation](VALIDATION.md), "Plugin value tones
and read-only rows"); pending requests have host tests only. On the console,
FX Control's own page registered with descriptor 4 on 2026-10-16
([Status](STATUS.md)); that record has no row edits, and tones, pending
requests and read-only rows have not been observed on hardware. Status cells
have host layout/registration coverage; runtime validation remains pending.
The window-state export (0.10.5) has host tests only; no Eden or console run
is recorded yet.
