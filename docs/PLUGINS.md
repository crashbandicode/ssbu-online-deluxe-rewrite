# Plugging into Online Deluxe

There are two ways another Skyline plugin can plug in. They are independent.

| You want | Use | Where |
| --- | --- | --- |
| Rows in Deluxe's menu (a toggle, a mode, a status) without writing any UI code | Deluxe's **plugin API** (`ssbu_online_deluxe_plugin_api_v1`) and the Rust SDK | this page, [sdk/README.md](../sdk/README.md) |
| Your own Dear ImGui windows drawn by the same renderer Deluxe uses | the **imgui-smash host** (`libimgui_smash.nro`, branch `deluxe`) | [intpa/imgui-smash](https://git.syntaxbyexample.com/intpa/imgui-smash) |

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
label and visited after the shared Plugins page; up to 64 options fit.
`OptionSpec::header(id, label)` is a title row that Up/Down skip, for
grouping a long page. The SDK sends descriptor version 4 only; hosts from
Deluxe 0.5.3 through the released 0.7.0 accept versions 2, 3 and 4 under the
same rules (they share one layout), so plugins built against the 2.0.0 or
3.0.0 SDK keep registering, and an older host answers `RegisterError::Abi`
with no retry. Revision 5 is unreleased ([below](#status-cells-unreleased)). A plugin that
scopes its behaviour to online play reads
`deluxe_plugin_sdk::host_online_context()` instead of hooking the game's
scene sites itself: it returns the host's ready / online / match / training
bits, or `None` when Deluxe is absent or older than 0.5.0.

[FX Control](https://git.syntaxbyexample.com/intpa/ssbu-fx-control) is the
first plugin built this way. It is the character-FX feature that Deluxe
carried until 0.5.0, now a separate NRO: an own page titled **FX Control**
with a scope row, an all-fighters row, one row per covered fighter and, under
a header per fighter, a value row per move selecting its Reduced variant
(every non-header row is a `RowLayout::Value`). Its Online scope reads
`host_online_context()`.

`request` is a request, not a command: return
`Pending` to apply a change later (the row shows `(pending)` until `current`
reports it), `Rejected` or `Unavailable` to refuse. A read-only option (or one
with a single choice) is a status row; give its choices tones to show
`Ready` / `Degraded` / `Failed` in green, yellow and red.

A status row is passive: Left/Right do nothing and `request` is never called.
On unreleased main (below) Up/Down also skip it (only an editable value row
holds the cursor, so a page with no editable row highlights nothing).

The complete example is [`sdk/examples/demo`](../sdk/examples/demo/src/lib.rs)
(three options, persisted INI, a time-driven status row). Limits, ABI layout,
error codes, threading rules and host persistence are in
[sdk/README.md](../sdk/README.md).

### Status cells (unreleased)

**Not in any release.** Status cells, revision 5 and the passive-row
navigation above exist only on main after v0.7.0 (commit `0d28adc`). The
released 0.7.0 host refuses a revision 5 descriptor with `REGISTER_ERR_ABI`.

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
layout; a native plugin sets the flag in its own descriptor. On the compact
in-match strip a cell run is one entry (`Pool:✓ | Results:✓ | Save:✗`, drawn
with the same marks), and the final strip page takes the passive rows that follow
the last editable row so they stay reachable.

## Your own windows through the imgui-smash host

Deluxe statically links the `deluxe` branch of
[intpa/imgui-smash](https://git.syntaxbyexample.com/intpa/imgui-smash) and,
when a compatible standalone `libimgui_smash.nro` is installed, reuses that host
instead. A plugin that wants its own windows (as Elite Legal Stages does) links
the host's exported `imgui_smash_*` functions and draws with Dear ImGui; the
fork's README has the steps and the release carries the host NRO and the
archive. Both integrations can run at once: Deluxe's menu and your windows
share one renderer and one controller.

## Verification

The Plugins page, pending requests and tones were verified in Eden on
2026-10-12 with the demo NRO ([validation](VALIDATION.md), "Plugin value tones
and read-only rows"). On the console, FX Control's own page registered with
descriptor 4 and its rows were edited in Training on 2026-10-16
([Status](STATUS.md)); tones, pending requests and read-only rows have not
been observed on hardware. Status cells have no validation record.
