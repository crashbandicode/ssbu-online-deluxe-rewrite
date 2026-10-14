# Configuration

Settings are created at:

```text
sd:/ultimate/ssbu_online_deluxe_rewrite/config.ini
```

Default configuration:

```ini
version=1
input_buffer=auto
render_mode=vanilla
auto_profile=true
platform=console
stealth=true
streamer=false
fps_boost=false
peer_observer=false
```

| Key | Values | Meaning |
| --- | --- | --- |
| `input_buffer` | `auto`, `0`–`25` | The game's online input buffer (NetLatency) |
| `render_mode` | `vanilla`, `lesslag`, `llultra`, `lldoubles` (also the restart-only experiments `lesslag_core`, `doubles_core`) | Saved manual profile; see [Rendering profiles](RENDERING.md) |
| `auto_profile` | `true`, `false` | Let the scene choose the profile |
| `platform` | `console`, `emulator` | Selects the Auto table and resolution policy |
| `stealth` | `true`, `false` | Suppress the recognition beacon and settings payload |
| `streamer` | `true`, `false` | Hide remote identities and restore the original native styles |
| `fps_boost` | `true`, `false` | Diagnostic experiment; the normal build reports Unsupported |
| `peer_observer` | `true`, `false` | Opt-in peer-observer diagnostics (station map, name rows) |

Use the explicit `platform=emulator` setting in Eden. The
[Eden test configuration](../config/eden-test.ini) contains that override.

`effects` and `effects_scope` are ignored since 0.5.0. Character FX is the
separate [FX Control](https://git.syntaxbyexample.com/intpa/ssbu-fx-control)
plugin, and its settings belong to `sd:/ultimate/ssbu_fx_control/config.ini`.
A 0.4.x file that still has the two keys loads with every other setting
intact; the next menu edit saves the file without them. Neither plugin copies
the old values into the FX Control file.

Menu edits apply live and are written by the background thread within about
half a second. External INI edits require restarting Smash. `config.previous.ini`
retains the previous valid configuration for recovery. An explicit
`stealth=false` stays false; missing Stealth settings default to true.

## Boot flags

Empty files in the same directory, read once at boot:

| File | Effect |
| --- | --- |
| `CAPTURE_CSS` | Arm a 30 s capture on every character-select entry ([Captures](CAPTURES.md)) |
| `CAPTURE_MATCH` | Record every match ([Captures](CAPTURES.md)) |
| `CAPTURE_TIMING` | Not a flag: write a label into it to start one capture |
| `DISABLE_BACKOFF`, `DISABLE_NATIVE`, `DISABLE_MENU`, `DISABLE_SCENE`, `DISABLE_BUFFER`, `DISABLE_RENDERER`, `DISABLE_PIA` | Bisection switches; see [Debugging](DEBUGGING.md). `DISABLE_FX` is ignored since 0.5.0 |
| `DUMP_PRESENTATION_HISTORY`, `DUMP_RENDER_PROBE` | Extra diagnostics; see [Debugging](DEBUGGING.md) |
