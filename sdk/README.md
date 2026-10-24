# Online Deluxe plugin API

A separately built Skyline NRO can add rows to the Online Deluxe menu without
shipping any UI code. The plugin describes its options, the host draws them and
asks the plugin to change them. The plugin owns its behavior and its saved state.

Descriptor version is 4 (Deluxe 0.5.3): `OptionDesc` is 56 bytes (per-choice
tones and the flags `NO_PERSIST`, `READ_ONLY`, `HEADER`, `INLINE`),
`PluginDesc` is 72 bytes with the plugin flag `PLUGIN_FLAG_OWN_PAGE` (a page of
the window cycle titled with the plugin label instead of a group on the shared
Plugins page), and a plugin may declare up to 64 options. Versions 2 and 3
share that layout and are accepted by the host under the same rules (see
[Descriptor version](#descriptor-version)). The exported host table is still
`PluginApiV1` at table version 1, under the unchanged symbol
`ssbu_online_deluxe_plugin_api_v1`; `table_matches` compares the table against
that table version, not against the descriptor version. A second export,
`ssbu_online_deluxe_online_context_v1` (`extern "C" fn() -> u32`), returns the
host's scene context as `ONLINE_CONTEXT_READY | ONLINE | MATCH | TRAINING`
bits; the SDK wraps it as `host_online_context()`, `None` on older hosts.

**Unreleased:** main after v0.7.0 (commit `0d28adc`) adds descriptor revision 5
(`CELL_ABI_VERSION`) with `OPTION_FLAG_CELL` passive status cells; that host
accepts revisions 2 to 5. The released v0.7.0 host accepts 2 to 4 and refuses
revision 5 with `REGISTER_ERR_ABI`. This SDK still sends revision 4 and has no
cell layout. See [Status cells](../docs/PLUGINS.md#status-cells-unreleased).

Pinned releases: git tag `v0.2.0` on the Forgejo repository
(`intpa/ssbu-online-deluxe-rewrite`) carries `deluxe-plugin-abi` 2.0.0 and
`deluxe-plugin-sdk` 2.0.0 (descriptor version 2); the 0.5.0 tag carries 3.0.0
(descriptor version 3); 0.5.3 carries 5.0.0 (descriptor version 4 only; the
4.0.0 crates of the earlier 0.5.3 builds still carried the retry chain; the
crates are unchanged at 5.0.0 through 0.7.0). Each
tag is the tree the matching host NRO was built from; the NRO's SHA-256 and
size are recorded in `docs/VALIDATION.md`. Consumers depend on the crates by
git tag (cargo finds the package by name anywhere in the repository) or clone
the tag and depend by path.

Layout of this directory:

| Path | Crate | Purpose |
| --- | --- | --- |
| `abi/` | `deluxe-plugin-abi` | `no_std` `#[repr(C)]` ABI shared by the host, the SDK and tests. Source of truth. |
| `sdk/` | `deluxe-plugin-sdk` | Rust consumer SDK: runtime host lookup, safe registration, `AtomicOptions`. |
| `examples/demo/` | `deluxe-plugin-demo` | Minimal NRO that registers three options and logs the result. |

Status of verification: the SDK is covered by host-side unit tests against a
fake host table, and the demo builds for `aarch64-skyline-switch`. The Plugins
page, pending requests and tones were verified in Eden on 2026-10-12 with the
demo NRO (`docs/VALIDATION.md`, "Plugin value tones and read-only rows").
On the console only FX Control's own-page registration and row edits are
recorded (2026-10-16, `docs/STATUS.md`); tones, pending requests and read-only
rows are not observed on hardware. A two-step guide with a minimal plugin
is in [docs/PLUGINS.md](../docs/PLUGINS.md); this file is the reference.

## What the host shows

* The Plugins page joins the window chord cycle (L+R, then D-pad Down; ZL+ZR
  stopped being a chord in Deluxe 0.5.3) as **settings → plugins → performance
  → window options → hidden** (the Window Options page is Deluxe 0.6.0's). It
  is skipped while no plugin has rows on it, so users without plugins see no
  change.
* A plugin registered with `PLUGIN_FLAG_OWN_PAGE` (Rust: `own_page: true`) is
  not grouped on the shared page: it gets its own page in the cycle, after
  the shared page and before Performance, titled with the plugin label; the
  page indicator reads `<Label> p/N`. Several own pages follow each other in
  registration order.
* The shared page lists, per plugin, a group row carrying the plugin label
  and then its option rows; an own page lists the plugin's option rows alone.
  Both views draw the same row list: group rows and header rows are label
  only and never selected, every other row is `Label` and the current value.
* Outside a match: a vertical table of those rows. Up/Down selects a row (the
  selection wraps and skips group and header rows), ZR + Left/Right asks the
  plugin for the previous or next choice (a plain Left/Right is ignored in the
  open window since Deluxe 0.5.6). Lists longer than ten rows are clipped to
  a window around the selection with `...` markers.
* In a ready Singles/Doubles match: the same compact two-row strip Deluxe uses
  for Settings. Row one holds a page of up to four fields that contains the
  selection: a group or header row as its label, every other row as
  `Label:Value`, `>` marking the selected one; row two shows `<title> p/N`
  and the frame-time line. Long lists page rather than shrinking the text.
* An option with exactly one choice is a read-only status row. So is any option
  flagged read-only (`OPTION_FLAG_READ_ONLY`, Rust `OptionSpec::read_only`):
  Left/Right do nothing, the host does not call `request`, and the value shown
  is whatever `current` reports. A multi-choice read-only row can display state
  such as Ready / Degraded / Failed.
* `OPTION_FLAG_HEADER` (Rust `OptionSpec::header(id, label)`,
  `RowLayout::Header`): the row shows only its label, flush with the page, as
  a group title for the rows under it. Up/Down skip it, Left/Right never reach
  it, `current` and `request` are never called for it and nothing is persisted.
  `choice_count` must be 0 and `choices`, `choice_values` and `choice_tones`
  null. `OPTION_FLAG_INLINE` is accepted and drawn as an ordinary value row;
  the Rust SDK no longer offers it. A row cannot be both header and inline.
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
* **Limits.** At most 16 plugins per host, 64 options per plugin (whatever
  the descriptor version; hosts before 0.5.3 took 32 for version 3 and 8 for
  version 2), 16 choices per option. Every plugin needs at least one option
  and every option other than a header at least one choice.
* **Strings are copied.** The host copies every string during `register`. The
  descriptor and its strings only need to live for that call.
* **Callbacks are retained.** `user`, `current` and `request` must stay valid
  for the life of the process. There is no unregister in version 1.
* **Threading.** Callbacks run on the host's overlay draw thread. They must
  be non-blocking, must not call back into the host API (no re-entrancy,
  including `register`) and must not assume a game thread. `current` runs
  once per visible option per frame while the Plugins page is shown, and once
  per frame for every option with a pending request even while the window is
  hidden (the host re-checks pending requests every draw so a change applied
  at match end is persisted); `request` runs once per accepted controller
  edge. Do shared-state work with atomics or short locks.
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

## Descriptor version

`PluginDesc` is 72 bytes and `options` always points at 56-byte `OptionDesc`
rows. `PluginDesc::abi_version` is 2, 3 or 4: the three revisions share that
layout and differed only in which flag bits and option counts a host would
take. Since Deluxe 0.5.3 the host reads all three under one rule set (the
version 4 rules: every flag bit, up to 64 options), so a plugin built against
the 2.0.0 or 3.0.0 SDK registers unchanged and a plugin may send any of the
three. Version 1 (a 48-byte row without tones, never released) and any other
number answer `REGISTER_ERR_ABI`.

The Rust SDK sends version 4 and calls the host exactly once. A host that
answers `REGISTER_ERR_ABI` predates 0.5.3; the SDK returns
`RegisterError::Abi`, drops the handler and does not retry. There is no
`negotiated_descriptor_version`: the accepted version is always the one sent.

The host export does not change with that number. `PluginApiV1` is still 32
bytes, its `abi_version` field is table version 1, and the symbol is still
`ssbu_online_deluxe_plugin_api_v1`. A plugin built against the old table keeps
resolving the same symbol.

`choice_tones` is null for neutral text, or `choice_count` bytes, each
`TONE_NEUTRAL` (0), `TONE_OK` (1), `TONE_WARN` (2) or `TONE_ERROR` (3). The Rust
SDK rejects a `tones` slice whose length is not `choices.len()` as
`RegisterError::Invalid` before it calls the host. Tones were verified in
Eden on 2026-10-12 (`docs/VALIDATION.md`, "Plugin value tones"); hardware
behaviour is not yet observed.

## Online context (optional, 0.5.0 hosts)

A plugin whose behaviour should differ online can share Deluxe's scene
tracker instead of hooking the game's scene sites itself (two plugins hooking
the same sites collide). The host exports
`ssbu_online_deluxe_online_context_v1`, an `extern "C" fn() -> u32` that reads
atomics and returns:

| Bit | Meaning |
| --- | --- |
| `ONLINE_CONTEXT_READY` (1) | the scene hooks are installed |
| `ONLINE_CONTEXT_ONLINE` (2) | Quickplay, Arena or local wireless, or the online-latency hook reached the current match |
| `ONLINE_CONTEXT_MATCH` (4) | a Singles or Doubles stage is loaded |
| `ONLINE_CONTEXT_TRAINING` (8) | the scene is Training (also the Quickplay waiting room) |

The Rust SDK wraps it as `deluxe_plugin_sdk::host_online_context() ->
Option<u32>`; `None` means no host or a host older than 0.5.0. Resolve it once
(the SDK caches the lookup) and read it when you need it; it is safe to call
from any thread.

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
* **When it is written.** After a menu edit your `request` callback answers
  `REQUEST_APPLIED`, the host rewrites the file on its own background thread,
  never on the overlay thread. It replaces the option's `key=value` line or
  appends one, keeps every other line (comments, unrelated keys) and writes
  atomically, leaving a `.previous.ini` backup beside the file.
  `REQUEST_REJECTED` and `REQUEST_UNAVAILABLE` write nothing.
* **Pending defers the write (host 0.2.1).** A `REQUEST_PENDING` answer
  persists nothing at the time of the edit. The host writes the value only
  when it later observes `current` reporting the requested choice (checked
  once per frame on the overlay thread, even while the Plugins page is
  hidden), or when a later request on the same option returns
  `REQUEST_APPLIED`. Stepping the row back to the live value cancels the
  pending change and writes nothing. A plugin that defers in-match edits
  therefore sees no host disk activity until it applies them.
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
use deluxe_plugin_sdk::{RowLayout, Tone};

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
            layout: RowLayout::Value,          // RowLayout::Header via OptionSpec::header
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
            layout: RowLayout::Value,
        },
    ],
    config_path: Some("sd:/ultimate/deluxe_plugin_demo/config.ini"),
    own_page: false,                           // true = its own page in the window cycle
};
// Read that file yourself at startup and seed OPTIONS before `register`.
// `OptionSpec::new` defaults `tones` to None, `read_only` to false and
// `layout` to `RowLayout::Value`.
```

The SDK checks these locally too: a `config_key` that is not a valid id, a
`values` or `tones` table whose length differs from `choices`, a value token
that is invalid, or an invalid `config_path` returns `RegisterError::Invalid`
without calling the host.

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
otherwise). No code is retried; `REGISTER_ERR_ABI` becomes `RegisterError::Abi`
after the one call.

`deluxe-plugin-demo` is a complete working example registering `enabled`,
`mode` and a read-only `status` option, with host persistence into
`sd:/ultimate/deluxe_plugin_demo/config.ini` and a startup loader for it.
`enabled` uses tones Neutral/Ok so "On" is green. `status` is the three choices
Ready / Degraded / Failed with tones Ok / Warn / Error, `read_only` and
`no_persist`; its `current` cycles those choices every five seconds from
startup. After `register` the demo logs the result and the descriptor version
it sent. Its Skyline entry point is compiled only for the switch target, so it
is an empty crate on a development host (its INI parser and the status cycle
are unit tested on the host).

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
`OptionDesc` 56, `PluginDesc` 72, `PluginApiV1` 32. `PluginDesc.abi_version`
is 4 (2 and 3 are accepted with the same layout and rules). The table's own
`abi_version` stays 1. A host before Deluxe 0.5.3 returns `REGISTER_ERR_ABI`
(-2) for a version 4 descriptor; a plugin may send version 3 or 2 to such a
host only with that host's narrower limits (32 or 8 options, no header rows,
and no own-page flag for version 2), which is not worth doing for new code.
Emulator and hardware behaviour of tones is not yet observed (Eden-verified
2026-10-12).

```c
#include <stdint.h>

#define DELUXE_DESCRIPTOR_VERSION 4u
#define DELUXE_TABLE_VERSION 1u
#define DELUXE_CHOICE_UNAVAILABLE 0xFFFFFFFFu
#define DELUXE_OPTION_FLAG_NO_PERSIST (1u << 0)
#define DELUXE_OPTION_FLAG_READ_ONLY (1u << 1)
#define DELUXE_OPTION_FLAG_HEADER (1u << 2)    /* label only, choice_count 0 */
#define DELUXE_OPTION_FLAG_INLINE (1u << 3)    /* accepted; drawn as a value row */
/* Unreleased (main after v0.7.0): only with PluginDesc.abi_version = 5. */
#define DELUXE_CELL_DESCRIPTOR_VERSION 5u
#define DELUXE_OPTION_FLAG_CELL (1u << 4)      /* passive status cell; tones required */
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
    uint32_t choice_count;       /* 1..16; 0 for a HEADER row */
    uint32_t flags;              /* DELUXE_OPTION_FLAG_* bits, 0 when none */
    const char *config_key;      /* NULL = use id; else [a-z0-9_]{1,32} */
    const char *const *choice_values; /* NULL = write the decimal choice index; else
                                         choice_count NUL-terminated tokens, 1..24
                                         printable ASCII, no whitespace, '=' or '#' */
    const uint8_t *choice_tones; /* NULL = neutral; else choice_count TONE_* bytes.
                                    The current choice is drawn in that colour in
                                    both the vertical table and the in-match strip. */
} OptionDesc;                    /* sizeof == 56 */

typedef struct PluginDesc {
    uint32_t struct_size;        /* sizeof(PluginDesc) */
    uint32_t abi_version;        /* DELUXE_DESCRIPTOR_VERSION (2 and 3 are accepted too) */
    const char *id;              /* NUL-terminated, [a-z0-9_]{1,32}, unique across plugins */
    const char *label;           /* NUL-terminated, 1..24 bytes */
    const OptionDesc *options;
    uint32_t option_count;       /* 1..64 */
    uint32_t flags;              /* 0, or DELUXE_PLUGIN_FLAG_OWN_PAGE (1u << 0) */
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
matter. Set `PluginDesc.abi_version = 4`; `REGISTER_ERR_ABI` means the host is
older than Deluxe 0.5.3.
