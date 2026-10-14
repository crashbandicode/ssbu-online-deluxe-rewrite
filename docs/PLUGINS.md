# Plugins page

Other mods can add their own rows without writing menu code. A separately
built Skyline NRO resolves the exported `ssbu_online_deluxe_plugin_api_v1`
symbol at runtime and registers a plugin label plus one or more options, each
with a finite list of choices (for example `Record: Off / On`). Deluxe draws
them on the plugins page: a vertical table outside matches and the compact
two-row strip during a ready match, with paging for long lists.

Deluxe asks the plugin for the current value every frame and sends Left/Right
as a request; the plugin decides whether it applies now, later, or not at all,
and keeps its own saved settings. A plugin can colour its values (green, yellow,
red) per choice and mark a row read-only, so a status such as `Health: Failed`
shows in red on both layouts without being editable. A plugin that declares its
own `key=value` INI path gets that file updated by Deluxe after each accepted
menu edit, so the plugin only has to read it at startup. Deluxe's `config.ini`
is unchanged.

The ABI, the Rust SDK and a demo NRO live under [`sdk/`](../sdk/README.md). The
page is covered by host tests and was verified in Eden on 2026-10-12 (Plugins
page, pending requests, tones; see [validation](VALIDATION.md)); hardware
behaviour is not yet observed.
