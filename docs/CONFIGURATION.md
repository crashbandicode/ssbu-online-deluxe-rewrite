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
drs_moves=none
drs_adaptive=false
drs_adaptive_threshold_ms=17.5
drs_adaptive_trigger_frames=6
drs_adaptive_release_frames=90
ultra_resolution=576
hide_pages=none
perf_view=graph
early_present=true
dispatcher_backoff=true
research_present_interval_zero=false
research_input_trace=false
```

| Key | Values | Meaning |
| --- | --- | --- |
| `input_buffer` | `auto`, `0`–`25` | The game's online input buffer (NetLatency) |
| `render_mode` | `vanilla`, `lesslag`, `llultra`, `lldoubles` (also the restart-only experiments `lesslag_core`, `doubles_core`) | Saved manual profile; see [Rendering profiles](RENDERING.md) |
| `auto_profile` | `true`, `false` | Let the scene choose the profile |
| `platform` | `console`, `emulator` | Selects the Auto table and resolution policy |
| `stealth` | `true`, `false` | Suppress the recognition beacon and settings payload |
| `streamer` | `true`, `false` | Hide remote identities and restore the original native styles |
| `drs_moves` | `none`, `all`, or comma-separated move names (below) | 0.5.2 experiment: lower the internal resolution one level while a named move's status is active; see [Rendering profiles](RENDERING.md) |
| `drs_adaptive` | `true`, `false` | 0.5.2 experiment: lower the resolution one level after a run of presents slower than the threshold, restore after a run within budget |
| `drs_adaptive_threshold_ms` | `10.00`–`40.00` (up to two decimals) | Present interval that counts as a late frame; 16.67 is one vblank, the default 17.5 allows half a millisecond of jitter |
| `drs_adaptive_trigger_frames` | `1`–`60` | Consecutive late presents before the resolution drops (default 6, one tenth of a second) |
| `drs_adaptive_release_frames` | `1`–`600` | Consecutive presents within budget before the resolution is restored (default 90, 1.5 s) |
| `ultra_resolution` | `576`, `720` | 0.5.3 experiment: the internal resolution console LLUltra renders at (default `576`, the fork's choice); also the Settings row **UltraRes** on a console, which takes effect at once. Dynamic resolution still steps down from it. Emulator profiles and the other console profiles ignore it. Measures how much of LLUltra's budget the resolution drop buys; see [Rendering profiles](RENDERING.md) |
| `hide_pages` | `none`, or a comma list of `plugins`, `own_pages`, `performance` | 0.5.3: pages the window chord skips. `plugins,performance` makes the cycle Settings → a plugin's own page (FX Control) → hidden, so a measurement session reaches the plugin page in one tap and hides it in one more. Settings is always in the cycle and shows the three as the rows **PluginsPage**, **OwnPages**, **PerfPage** (Shown / Hidden), which toggle them live; Debug keeps its own chord. See [Controls](CONTROLS.md) |
| `early_present` | `true`, `false` | 0.5.3: present the acquired image at the flush that submitted it, the 0.4.0 fix for the two-image profiles' character-select stall (LessLag; LLUltra and LLDoubles since 0.5.2). `false` keeps the game's own present order everywhere, which is what upstream does. Read at start. The before/after is shown in [Optimizations added by this rewrite](../README.md#optimizations-added-by-this-rewrite) |
| `dispatcher_backoff` | `true`, `false` | 0.5.3: let the dispatcher idle-backoff hook sleep ([Dispatcher backoff](DISPATCHER_BACKOFF.md)). `false` leaves the hook installed but asleep, so the game spins as upstream does; the Performance row **Backoff** reads `Off (ini)`. Read at start. The `DISABLE_BACKOFF` flag still skips the install entirely |
| `research_present_interval_zero` | `true`, `false` | 2026-10-18 research experiment: after the first successful tracked NVN window init, resolve the raw `nvnWindowSetPresentInterval` and call it once with `0`. Default off; startup-only, with no Settings row. `renderer-status.txt` records setter availability, whether the call was issued, and `research_present_interval_zero_api_reported_readback`. The setter is void; the getter reports its cached/requested interval even if the inner surface clamps it. Separately, the runtime takes one exact-13.0.5, SDK-signature-validated, read-only Surface snapshot before and after the call, including its minimum, maximum, and effective fields. If the Surface signatures or mappings are unavailable, that snapshot reports unavailable independently of the NVN cached getter. No Surface field is written. Neither API readback nor the Surface snapshot establishes display behavior or physical no-vsync. |
| `research_input_trace` | `true`, `false` | Startup-only diagnostic request, default off and with no Settings row. When enabled, all seven 13.0.5 instruction words are preflighted before installing points around the HID manager (`main+0x374cab8`/`+0x374cabc`) and generic scene driver (`main+0x374cbd4`/`+0x374cbd8`); failure closes the experiment. Kinds 80–83 record the hashed calling thread at those points. During an active API trace only, kind 84 samples the current input-cache mode (`index`) and a bitmask of nonzero per-player override bytes (`result` bits 0–7). Companion kinds 85/86 bracket the selected cache producer at +0x135f874/+0x135f878; 87/88 carry player-0 mapped input low/high words after production; 89 marks mapped-input consumption immediately after +0x3f727c, and 90/91 carry the consumed low/high words with player index. Kinds 93/94 observe player-0 FIFO count/head after production. Phase/thread markers use the current normal-path HID iteration as index. These are observations of existing execution, not extra input reads or simulation calls; catch-up paths are not bracketed. It does not change input samples and is not a latency lever; trace hooks add overhead while a capture runs. `renderer-status.txt` reports the requested value and hook state. |
| `perf_view` | `graph`, `number` | 0.5.3: the frame-time block at the bottom of the vertical window as the 180-sample plot (default) or the mean as text, which is less draw work while measuring; also the Settings row **PerfView**. The compact match strip is text either way |

Legacy keys that earlier builds wrote are read and ignored, so an existing file
is not reset: `effects` and `effects_scope` (FX moved to ssbu-fx-control in
0.5.0), `fps_boost` and `peer_observer` (both features removed on 2026-10-17).
The next save drops them.

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
