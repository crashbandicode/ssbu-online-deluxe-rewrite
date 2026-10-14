# Configuration

Settings are created at:

```text
sd:/ultimate/ssbu_online_deluxe_rewrite/config.ini
```

Default configuration:

```ini
version=1
effects=on
effects_scope=online
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
| `effects` | `on`, `off`, `reduced` | Character FX policy; see [FX](FX.md) |
| `effects_scope` | `online`, `all` | Apply the FX policy only in recognized online contexts, or everywhere |
| `input_buffer` | `auto`, `0`–`25` | The game's online input buffer (NetLatency) |
| `render_mode` | `vanilla`, `lesslag`, `llultra`, `lldoubles` (also the restart-only experiments `lesslag_core`, `doubles_core`) | Saved manual profile; see [Rendering profiles](RENDERING.md) |
| `auto_profile` | `true`, `false` | Let the scene choose the profile |
| `platform` | `console`, `emulator` | Selects the Auto table and resolution policy |
| `stealth` | `true`, `false` | Suppress the recognition beacon and settings payload |
| `streamer` | `true`, `false` | Hide remote identities and restore the original native styles |
| `fps_boost` | `true`, `false` | Diagnostic experiment; the normal build reports Unsupported |
| `peer_observer` | `true`, `false` | Opt-in peer-observer diagnostics (station map, name rows) |

Use the explicit `platform=emulator` setting in Eden. For offline FX testing,
also set `effects_scope=all`. The [Eden test configuration](../config/eden-test.ini)
contains both overrides.

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
| `DISABLE_BACKOFF`, `DISABLE_NATIVE`, `DISABLE_MENU`, `DISABLE_SCENE`, `DISABLE_FX`, `DISABLE_BUFFER`, `DISABLE_RENDERER`, `DISABLE_PIA` | Bisection switches; see [Debugging](DEBUGGING.md) |
| `DUMP_PRESENTATION_HISTORY`, `DUMP_RENDER_PROBE` | Extra diagnostics; see [Debugging](DEBUGGING.md) |
