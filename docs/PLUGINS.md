# Plugging into Online Deluxe

There are two ways another Skyline plugin can plug in. They are independent.

| You want | Use | Where |
| --- | --- | --- |
| Rows in Deluxe's menu (a toggle, a mode, a status) without writing any UI code | Deluxe's **plugin API** (`ssbu_online_deluxe_plugin_api_v1`) and the Rust SDK | this page, [sdk/README.md](../sdk/README.md) |
| Your own Dear ImGui windows drawn by the same renderer Deluxe uses | the **imgui-smash host** (`libimgui_smash.nro`, branch `deluxe`) | [intpa/imgui-smash](https://git.syntaxbyexample.com/intpa/imgui-smash) |

## Menu rows in three steps

Your NRO describes its options once; Deluxe draws them on the Plugins page
(a vertical table outside matches, the compact two-row strip in a match),
asks you for the current value every frame and sends Left/Right as a request.
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
                           PluginSpec, RequestStatus, Tone};

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
       }],
       config_path: Some("sd:/ultimate/my_plugin/config.ini"), // … into this file
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

That is the whole integration. `request` is a request, not a command: return
`Pending` to apply a change later (the row shows `(pending)` until `current`
reports it), `Rejected` or `Unavailable` to refuse. A read-only option (or one
with a single choice) is a status row; give its choices tones to show
`Ready` / `Degraded` / `Failed` in green, yellow and red.

The complete example is [`sdk/examples/demo`](../sdk/examples/demo/src/lib.rs)
(three options, persisted INI, a time-driven status row). Limits, ABI layout,
error codes, threading rules and host persistence are in
[sdk/README.md](../sdk/README.md).

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
and read-only rows"). Hardware behaviour is not yet observed.
