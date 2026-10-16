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
drs_moves=none
drs_adaptive=false
drs_adaptive_threshold_ms=17.5
drs_adaptive_trigger_frames=6
drs_adaptive_release_frames=90
ultra_resolution=576
hide_pages=none
perf_view=graph
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
| `drs_moves` | `none`, `all`, or comma-separated move names (below) | 0.5.2 experiment: lower the internal resolution one level while a named move's status is active; see [Rendering profiles](RENDERING.md) |
| `drs_adaptive` | `true`, `false` | 0.5.2 experiment: lower the resolution one level after a run of presents slower than the threshold, restore after a run within budget |
| `drs_adaptive_threshold_ms` | `10.00`–`40.00` (up to two decimals) | Present interval that counts as a late frame; 16.67 is one vblank, the default 17.5 allows half a millisecond of jitter |
| `drs_adaptive_trigger_frames` | `1`–`60` | Consecutive late presents before the resolution drops (default 6, one tenth of a second) |
| `drs_adaptive_release_frames` | `1`–`600` | Consecutive presents within budget before the resolution is restored (default 90, 1.5 s) |
| `ultra_resolution` | `576`, `720` | 0.5.3 experiment: the internal resolution console LLUltra renders at (default `576`, the fork's choice); also the Settings row **UltraRes** on a console, which takes effect at once. Dynamic resolution still steps down from it. Emulator profiles and the other console profiles ignore it. Measures how much of LLUltra's budget the resolution drop buys; see [Rendering profiles](RENDERING.md) |
| `hide_pages` | `none`, or a comma list of `plugins`, `own_pages`, `performance` | 0.5.3: pages the window chord skips. `plugins,performance` makes the cycle Settings → a plugin's own page (FX Control) → hidden, so a measurement session reaches the plugin page in one tap and hides it in one more. Settings is always in the cycle and shows the three as the rows **PluginsPage**, **OwnPages**, **PerfPage** (Shown / Hidden), which toggle them live; Debug keeps its own chord. See [Controls](CONTROLS.md) |
| `perf_view` | `graph`, `number` | 0.5.3: the frame-time block at the bottom of the vertical window as the 180-sample plot (default) or the mean as text, which is less draw work while measuring; also the Settings row **PerfView**. The compact match strip is text either way |

## Dynamic-resolution experiment (0.5.2)

Both keys are off by default and only act where dynamic resolution is on,
that is on the console under LessLag, LLUltra and LLDoubles (Vanilla and the
emulator policy ignore them). Neither needs smashline. The move table reads
the fighter kind and status the Main hook already samples; the adaptive
trigger reads the present intervals the renderer already counts. The Debug
view shows both (`DRS moves`, `DRS adaptive`) and `renderer-status.txt`
carries the settings and the push/pop counters; every edge is event code 108.

Move names for `drs_moves`: `falcon_punch`, `pyra_side_b`, `pyra_neutral_b`,
`pyra_up_b`, `mythra_specials`, `palutena_explosive_flame`,
`ness_pk_thunder`, `lucas_pk_thunder`, `kirby_hammer`, `byleth_specials`,
`inkling_roller`. Example:

```ini
drs_moves=pyra_side_b,pyra_neutral_b,kirby_hammer
drs_adaptive=true
drs_adaptive_threshold_ms=17.5
```

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
