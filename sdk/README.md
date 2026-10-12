# Online Deluxe plugin API

A separately built Skyline NRO can add rows to the Online Deluxe menu without
shipping any UI code. The plugin describes its options, the host draws them and
asks the plugin to change them. The plugin owns its behavior and its saved state.

Descriptor version is 2: `OptionDesc` is 56 bytes and may carry per-choice tones
and a read-only flag. The exported host table is still `PluginApiV1` at table
version 1, under the unchanged symbol `ssbu_online_deluxe_plugin_api_v1`.
`table_matches` compares the table against that table version, not against the
descriptor version. An old host answers `REGISTER_ERR_ABI` to a version 2
descriptor; the Rust SDK retries once with a version 1 descriptor.

Layout of this directory:

| Path | Crate | Purpose |
| --- | --- | --- |
| `abi/` | `deluxe-plugin-abi` | `no_std` `#[repr(C)]` ABI shared by the host, the SDK and tests. Source of truth. |
| `sdk/` | `deluxe-plugin-sdk` | Rust consumer SDK: runtime host lookup, safe registration, `AtomicOptions`. |
| `examples/demo/` | `deluxe-plugin-demo` | Minimal NRO that registers three options and logs the result. |

Status of verification: the SDK is covered by host-side unit tests against a
fake host table, and the demo builds for `aarch64-skyline-switch`. How the
Plugins page looks and behaves on an emulator or on hardware has **not** been
verified in this work. Emulator and hardware behaviour of tones is not yet
verified. Treat the description of the page below as the intended behavior, not
an observed one.

## What the host shows

* The Plugins page joins the window chord cycle (L+R or ZL+ZR, then D-pad
  Down) as **settings → plugins → performance → hidden**. It is skipped while
  no plugin has registered, so users without plugins see no change.
* Outside a match: a vertical table with a header row per plugin and one row
  per option. Up/Down selects a row (the selection wraps), Left/Right asks the
  plugin for the previous or next choice. Lists longer than ten rows are
  clipped to a window around the selection with `...` markers.
* In a ready Singles/Doubles match: the same compact two-row strip Deluxe uses
  for Settings. Row one holds a page of up to four `Label:Value` fields that
  contains the selection (`>` marks it); row two shows `Plugins p/N` and the
  frame-time line. Long lists page rather than shrinking the text.
* Naming: a plugin with a single option is shown under the plugin label. A
  plugin with several options is shown as `Plugin.Option`.
* An option with exactly one choice is a read-only status row. So is any option
  flagged read-only (`OPTION_FLAG_READ_ONLY`, Rust `OptionSpec::read_only`):
  Left/Right do nothing, the host does not call `request`, and the value shown
  is whatever `current` reports. A multi-choice read-only row can display state
  such as Ready / Degraded / Failed.
* Each choice may carry a tone. The host draws the **current** choice's value
  text in that colour, in the vertical table and in the in-match strip:

  | Tone | Byte | Colour | Meaning |
  | --- | --- | --- | --- |
  | Neutral | 0 | default text | No extra emphasis. |
  | Ok | 1 | green | Something is working. |
  | Warn | 2 | yellow | Degraded, or waiting on attention. |
  | Error | 3 | red | Failed or unavailable. |

  Supply one tone per choice, or null / `None` for neutral on every choice.
  Only the current choice's tone is shown.
* `CHOICE_UNAVAILABLE` shows as `Unavailable`. After `REQUEST_PENDING`, the
  vertical row shows `Value (pending)` and the strip shows `Value~` until
  `current` reports the requested choice or another request is made.
  `REQUEST_REJECTED` and `REQUEST_UNAVAILABLE` change nothing on screen; the
  host logs them (diagnostic code 100) and keeps showing `current`.

Keep labels short. The strip is narrow and labels are limited to 24 bytes.

## Registration flow

1. Skyline loads every plugin NRO before it runs any plugin `main`, so the host
   exports are already resolvable from your `main`, in any load order.
2. Resolve the exported symbol `ssbu_online_deluxe_plugin_api_v1` with
   `nn::ro::LookupSymbol`. Do **not** link against it: a runtime lookup lets
   your NRO load, and keep working, when Deluxe is not installed. The SDK does
   this in `host()`.
3. Call the symbol (`extern "C" fn() -> *const PluginApiV1`). A null result means
   no compatible host. Check `struct_size` and `abi_version` before using the
   table (`table_matches` accepts table version 1 only; the SDK does this for
   you). That table version is not the descriptor version.
4. Call `table.register(&PluginDesc)`. It returns a non-negative handle or a
   negative `REGISTER_ERR_*` code. Registration happens once, normally from
   `main`.
5. If there is no host, carry on without a menu row.

## ABI rules

* **ids versus labels.** `id` strings are stable identifiers, never displayed:
  1 to 32 bytes of lowercase ASCII letters, digits and `_`. Plugin ids are unique
  across plugins; option ids are unique within a plugin. Labels (plugin, option
  and choice) are display text: non-empty UTF-8, at most 24 bytes, no control
  characters.
* **Limits.** At most 16 plugins per host, 8 options per plugin, 16 choices per
  option. Every plugin needs at least one option and every option at least one
  choice.
* **Strings are copied.** The host copies every string during `register`. The
  descriptor and its strings only need to live for that call.
* **Callbacks are retained.** `user`, `current` and `request` must stay valid
  for the life of the process. There is no unregister in version 1.
* **Threading.** Callbacks run on the host's overlay draw thread while the menu
  is open. They must be non-blocking, must not call back into the host API
  (no re-entrancy, including `register`) and must not assume a game thread.
  `current` runs once per visible option per frame; `request` runs once per
  accepted controller edge. Do shared-state work with atomics or short locks.
* **Persistence is yours by default.** Unless you opt in (see
  [Host persistence](#host-persistence-optional)), the host stores nothing for
  the plugin. Load your own settings at startup and report them through
  `current`; even with the opt-in, the host never reads the file back.
* **`current(user, option)`** returns the index of the option's current choice,
  or `CHOICE_UNAVAILABLE` (`u32::MAX`) when there is no value to show.
* **`request(user, option, choice)`** is a request, not a command. It returns:

  | Code | Meaning |
  | --- | --- |
  | `REQUEST_APPLIED` (0) | The change took effect. |
  | `REQUEST_PENDING` (1) | Accepted but applies later, for example after the match. |
  | `REQUEST_REJECTED` (2) | Refused in the current state. Nothing changed. |
  | `REQUEST_UNAVAILABLE` (3) | The feature behind this option is unavailable. |

  The host never assumes the change succeeded: it re-reads `current` on the next
  frame and shows that. For `REQUEST_PENDING`, have `current` keep returning the
  active choice until your change really applies; the host presents the request
  as Pending.
* **Register errors.** `REGISTER_ERR_NULL` (-1), `_ABI` (-2), `_INVALID` (-3,
  a string, count or id charset outside the limits), `_FULL` (-4), `_DUPLICATE`
  (-5, id already registered), `_CALLBACKS` (-6, a callback is null).
* **Host status.** `host_status()` returns bits; `HOST_STATUS_MENU_ACTIVE` is
  clear when the host skipped its menu hooks (unsupported game version, legacy
  SsbuSync host present, or the menu disabled), in which case registered rows
  cannot be shown. `host_build()` returns a NUL-terminated build
  string valid for the life of the process.

## Descriptor version 2 and the version 1 fallback

`PluginDesc` itself is still 72 bytes. Its `abi_version` selects the layout of
`options`:

| `abi_version` | Row type | Size | What it carries |
| --- | --- | --- | --- |
| 2 | `OptionDesc` | 56 | `choice_tones` and `OPTION_FLAG_READ_ONLY` |
| 1 | `OptionDescV1` | 48 | the same fields without tones |

The host export does not change with that number. `PluginApiV1` is still 32
bytes, its `abi_version` field is table version 1, and the symbol is still
`ssbu_online_deluxe_plugin_api_v1`. A plugin built against the old table keeps
resolving the same symbol.

A host built before descriptor version 2 returns `REGISTER_ERR_ABI` for a
version 2 descriptor. The Rust SDK then retries **once** with `abi_version = 1`,
an array of `OptionDescV1`, no tone pointer, and the read-only bit cleared
(a version 1 host rejects unknown flag bits). `user` and the callbacks stay the
same pointer. Only the final result leaks or drops the handler.
`negotiated_descriptor_version(handle)` is `Some(2)` when the first descriptor
was accepted and `Some(1)` when the retry was. Other languages should do that
same one retry themselves. Tones and read-only rows need a version 2 host; on
the fallback the row is still registered, just without colour and without the
read-only flag (a single-choice row is still read-only on every host).

`choice_tones` is null for neutral text, or `choice_count` bytes, each
`TONE_NEUTRAL` (0), `TONE_OK` (1), `TONE_WARN` (2) or `TONE_ERROR` (3). The Rust
SDK rejects a `tones` slice whose length is not `choices.len()` as
`RegisterError::Invalid` before it calls the host. Emulator and hardware
behaviour of tones is not yet verified.

## Host persistence (optional)

A plugin that keeps its settings in a simple `key=value` INI file can ask the
host to write menu edits into that file, so it does not need its own save code.
It is opt-in: without `config_path` nothing is ever written.

* **Opt in.** Set `PluginDesc::config_path` (Rust: `PluginSpec::config_path`) to
  the NUL-terminated path of your own INI, for example
  `sd:/ultimate/deluxe_plugin_demo/config.ini`. It must start with `sd:/`, be at
  most 255 bytes and contain no control characters. Null disables persistence.
* **Key.** `OptionDesc::config_key` (Rust: `OptionSpec::config_key`) is the INI
  key for the option; null uses the option `id`. Same charset as an id
  (lowercase ASCII, digits, `_`, 1 to 32 bytes).
* **Value.** `OptionDesc::choice_values` (Rust: `OptionSpec::values`) is
  `choice_count` NUL-terminated tokens, one per choice, written as the value
  (for example `off`/`on`). Null writes the decimal choice index (`0`, `1`, ...).
  Each token is 1 to 24 printable ASCII bytes with no whitespace, `=` or `#`.
  The Rust SDK requires exactly one token per choice.
* **When it is written.** After a menu edit your `request` callback answers with
  `REQUEST_APPLIED` or `REQUEST_PENDING`. The host rewrites the file on its own
  background thread, never on the overlay thread. It replaces the option's
  `key=value` line or appends one, keeps every other line (comments, unrelated
  keys) and writes atomically, leaving a `.previous.ini` backup beside the
  file. `REQUEST_REJECTED` and `REQUEST_UNAVAILABLE` write nothing.
* **What is never written.** Read-only options (a single choice, or
  `OPTION_FLAG_READ_ONLY` / `OptionSpec::read_only`), and any option with
  `OPTION_FLAG_NO_PERSIST` in `OptionDesc::flags` (Rust:
  `OptionSpec::no_persist`).
* **The host never reads the file.** Load it yourself at startup, before you
  register, and report the loaded choice through `current`. Ignore unknown keys
  and bad values so a hand-edited file cannot break the plugin.
* `PluginDesc::flags` (formerly `reserved`) must still be zero.

The demo registers this INI, so after toggling `Enabled` to On and `Mode` to B
the host leaves:

```ini
enabled=on
mode=b
```

(`status` is `no_persist`, so it never appears.) On the next start the demo
reads the file and seeds its options from it. The persistence behaviour has the
same verification status as the rest of this README: the SDK is unit tested
against a fake host table only; the host-side write path is described by the ABI
documentation, not observed here.

## Rust example

`Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]

[package.metadata.skyline]
titleid = "01006A800016E000"

[dependencies]
deluxe-plugin-sdk = { path = "path/to/ssbu-online-deluxe-rewrite/sdk/sdk" }

[target.'cfg(target_os = "switch")'.dependencies]
skyline = "=0.6.0"

[profile.release]
panic = "abort"
```

`src/lib.rs`:

```rust
use deluxe_plugin_sdk::{register, AtomicOptions, OptionSpec, PluginSpec};

// One atomic per option, holding the current choice index.
static OPTIONS: AtomicOptions<2> = AtomicOptions::new([0, 0]);

// `OptionSpec::new` / `PluginSpec::new` leave host persistence off.
static SPEC: PluginSpec = PluginSpec::new(
    "demo",
    "Demo",
    &[
        OptionSpec::new("enabled", "Enabled", &["Off", "On"]),
        OptionSpec::new("mode", "Mode", &["A", "B", "C"]),
    ],
);

#[skyline::main(name = "my-plugin")]
pub fn main() {
    // Err(HostMissing) just means Deluxe is not installed; keep running.
    let result = register(&SPEC, &OPTIONS);
    skyline::println!("[my-plugin] register: {result:?}");
}

// Elsewhere, read the player's choice: OPTIONS.get(0) == 1 means "On".
```

To let the host persist edits, fill the optional fields with literals:

```rust
use deluxe_plugin_sdk::Tone;

static ENABLED_TONES: [Tone; 2] = [Tone::Neutral, Tone::Ok];
static STATUS_TONES: [Tone; 3] = [Tone::Ok, Tone::Warn, Tone::Error];

static SPEC: PluginSpec = PluginSpec {
    id: "demo",
    label: "Demo",
    options: &[
        OptionSpec {
            id: "enabled",
            label: "Enabled",
            choices: &["Off", "On"],
            config_key: Some("enabled"),       // None = the option id
            values: Some(&["off", "on"]),      // None = decimal index
            no_persist: false,
            tones: Some(&ENABLED_TONES),       // None = neutral; one per choice
            read_only: false,
        },
        OptionSpec {
            id: "status",
            label: "Status",
            choices: &["Ready", "Degraded", "Failed"],
            config_key: None,
            values: None,
            no_persist: true,                  // never written
            tones: Some(&STATUS_TONES),
            read_only: true,                   // host does not call request
        },
    ],
    config_path: Some("sd:/ultimate/deluxe_plugin_demo/config.ini"),
};
// Read that file yourself at startup and seed OPTIONS before `register`.
// `OptionSpec::new` defaults `tones` to None and `read_only` to false.
```

The SDK checks these locally too: a `config_key` that is not a valid id, a
`values` or `tones` table whose length differs from `choices`, a value token
that is invalid, or an invalid `config_path` returns `RegisterError::Invalid`
without calling the host.

On success, `negotiated_descriptor_version(handle)` reports the descriptor
revision the host accepted (`Some(2)` or, after the automatic fallback,
`Some(1)`).

For anything beyond "store the index", implement `PluginHandler` yourself:

```rust
use deluxe_plugin_sdk::{PluginHandler, RequestStatus};

struct Recorder;

impl PluginHandler for Recorder {
    fn current(&self, option: usize) -> Option<usize> { /* index, or None */ }
    fn request(&self, option: usize, choice: usize) -> RequestStatus {
        /* RequestStatus::Applied | Pending | Rejected | Unavailable */
    }
}
```

`PluginHandler` must be `Send + Sync + 'static` and is leaked for the process
after a successful registration. The SDK range-checks the `option` and `choice`
indices the host passes and never forwards an out-of-range one: `request`
answers `REQUEST_REJECTED`, `current` answers `CHOICE_UNAVAILABLE`. A read-only
option is also answered `REQUEST_REJECTED` if the host calls `request`. It also
validates the spec locally and returns `RegisterError::Invalid` before calling
the host, including when no host is installed. Duplicate option ids within one
plugin are rejected locally as well.

`register` returns `RegisterError::HostMissing` when no host is present,
`HostIncompatible` when a table exists with a different size or table version,
and maps each `REGISTER_ERR_*` code to a named variant (`Unknown(code)`
otherwise). `REGISTER_ERR_ABI` is the one code the SDK retries, once, with a
version 1 descriptor before returning `RegisterError::Abi`.

`deluxe-plugin-demo` is a complete working example registering `enabled`,
`mode` and a read-only `status` option, with host persistence into
`sd:/ultimate/deluxe_plugin_demo/config.ini` and a startup loader for it.
`enabled` uses tones Neutral/Ok so "On" is green. `status` is the three choices
Ready / Degraded / Failed with tones Ok / Warn / Error, `read_only` and
`no_persist`; its `current` cycles those choices every five seconds from
startup. After `register` the demo logs `negotiated_descriptor_version` (2 on a
current host, 1 after the automatic fallback). Its Skyline entry point is
compiled only for the switch target, so it is an empty crate on a development
host (its INI parser and the status cycle are unit tested on the host).

## Building an NRO

From the plugin crate directory, with `cargo-skyline` installed:

```
cargo skyline build --release
```

The NRO lands at
`target/aarch64-skyline-switch/release/lib<crate_name>.nro` (the ELF is the
`.so` beside it). Use the same pinned Rust toolchain as the host repository.
`*.nro` files are gitignored; do not commit them.

## C sketch for other languages

Everything is C layout with natural alignment on a 64-bit target. Sizes:
`OptionDesc` 56 (descriptor version 2), `OptionDescV1` 48 (descriptor version 1),
`PluginDesc` 72, `PluginApiV1` 32. `PluginDesc.abi_version` is 2 when `options`
points at `OptionDesc`. The table's own `abi_version` stays 1. An old host
returns `REGISTER_ERR_ABI` (-2) for version 2; retry once with version 1 and
`OptionDescV1` (no `choice_tones`, and do not set `READ_ONLY`). Emulator and
hardware behaviour of tones is not yet verified.

```c
#include <stdint.h>

#define DELUXE_DESCRIPTOR_VERSION 2u
#define DELUXE_TABLE_VERSION 1u
#define DELUXE_CHOICE_UNAVAILABLE 0xFFFFFFFFu
#define DELUXE_OPTION_FLAG_NO_PERSIST (1u << 0)
#define DELUXE_OPTION_FLAG_READ_ONLY (1u << 1) /* descriptor version 2 only */
#define DELUXE_TONE_NEUTRAL 0u
#define DELUXE_TONE_OK 1u
#define DELUXE_TONE_WARN 2u
#define DELUXE_TONE_ERROR 3u

typedef uint32_t (*deluxe_current_fn)(void *user, uint32_t option);
typedef uint32_t (*deluxe_request_fn)(void *user, uint32_t option, uint32_t choice);

typedef struct OptionDesc {
    const char *id;              /* NUL-terminated, [a-z0-9_]{1,32}, unique in plugin */
    const char *label;           /* NUL-terminated, 1..24 bytes */
    const char *const *choices;  /* choice_count NUL-terminated labels */
    uint32_t choice_count;       /* 1..16 */
    uint32_t flags;              /* DELUXE_OPTION_FLAG_* bits, 0 when none */
    const char *config_key;      /* NULL = use id; else [a-z0-9_]{1,32} */
    const char *const *choice_values; /* NULL = write the decimal choice index; else
                                         choice_count NUL-terminated tokens, 1..24
                                         printable ASCII, no whitespace, '=' or '#' */
    const uint8_t *choice_tones; /* NULL = neutral; else choice_count TONE_* bytes.
                                    The current choice is drawn in that colour in
                                    both the vertical table and the in-match strip. */
} OptionDesc;                    /* sizeof == 56; PluginDesc.abi_version = 2 */

/* Descriptor version 1, for a host that returns REGISTER_ERR_ABI on version 2.
   Same prefix, no choice_tones. Do not set DELUXE_OPTION_FLAG_READ_ONLY.
   Rust spells choice_values as `*const c_char` and casts it to this pointer. */
typedef struct OptionDescV1 {
    const char *id;
    const char *label;
    const char *const *choices;
    uint32_t choice_count;
    uint32_t flags;
    const char *config_key;
    const char *const *choice_values;
} OptionDescV1;                  /* sizeof == 48; PluginDesc.abi_version = 1 */

typedef struct PluginDesc {
    uint32_t struct_size;        /* sizeof(PluginDesc) */
    uint32_t abi_version;        /* 2 = OptionDesc, 1 = OptionDescV1 */
    const char *id;              /* NUL-terminated, [a-z0-9_]{1,32}, unique across plugins */
    const char *label;           /* NUL-terminated, 1..24 bytes */
    const OptionDesc *options;   /* or OptionDescV1* when abi_version is 1 */
    uint32_t option_count;       /* 1..8 */
    uint32_t flags;              /* must be 0 */
    void *user;                  /* handed back to every callback */
    deluxe_current_fn current;   /* non-NULL */
    deluxe_request_fn request;   /* non-NULL */
    const char *config_path;     /* NULL, or "sd:/..." (at most 255 bytes) of your own
                                    key=value INI the host updates after menu edits */
} PluginDesc;                    /* sizeof == 72 */

typedef struct PluginApiV1 {
    uint32_t struct_size;        /* sizeof(PluginApiV1) == 32 */
    uint32_t abi_version;        /* DELUXE_TABLE_VERSION, not the descriptor version */
    int32_t (*reg)(const PluginDesc *desc);   /* >= 0 handle, < 0 REGISTER_ERR_* */
    uint32_t (*host_status)(void);            /* bit 0: menu hooks active */
    const char *(*host_build)(void);          /* NUL-terminated, process lifetime */
} PluginApiV1;

/* Exported by the host as "ssbu_online_deluxe_plugin_api_v1". Resolve it with
   nn::ro::LookupSymbol; do not link it. The symbol and the table layout are
   unchanged from table version 1. */
typedef const PluginApiV1 *(*deluxe_api_entry_fn)(void);

/* request results: 0 applied, 1 pending, 2 rejected, 3 unavailable.
   register errors: -1 null, -2 abi, -3 invalid, -4 full, -5 duplicate, -6 callbacks.
   tones: 0 neutral, 1 ok (green), 2 warn (yellow), 3 error (red). */
```

The Rust field is named `register`; it is renamed `reg` here only because
`register` is a reserved storage-class word in C. Field order and types are what
matter. Set `PluginDesc.abi_version = 2` for the 56-byte `OptionDesc`. On
`REGISTER_ERR_ABI`, send the same `PluginDesc` again with `abi_version = 1` and
`options` pointing at `OptionDescV1`.
