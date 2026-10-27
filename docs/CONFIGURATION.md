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
vanilla_plus=true
hints=true
```

| Key | Values | Meaning |
| --- | --- | --- |
| `input_buffer` | `auto`, `0`–`25` | The game's online input buffer (NetLatency) |
| `render_mode` | `vanilla`, `lesslag`, `llultra`, `lldoubles` (also the restart-only experiments `lesslag_core`, `doubles_core`) | Saved manual profile; see [Rendering profiles](RENDERING.md) |
| `auto_profile` | `true`, `false` | Let the scene choose the profile |
| `platform` | `console`, `emulator` | Selects the Auto table and resolution policy. Since 0.9.0 it also selects how the Horizon-OC client asks sm whether `hoc:clk` exists: TIPC on a console (the only framing Atmosphère accepts for that query), CMIF on an emulator (Eden's TIPC answer is always "absent"; see [hoc-standin](../tools/hoc-standin/README.md)) |
| `stealth` | `true`, `false` | Suppress the recognition beacon and settings payload. The peer settings exchange (the peer's announced settings on the Settings page's Opp line; the Peer row before 0.6.0) needs Stealth Off on both peers, and the value is latched per connection, so start a new online session after changing it |
| `streamer` | `true`, `false` | Hide remote identities and restore the original native styles |
| `drs_moves` | `none`, `all`, or comma-separated move names (below) | 0.5.2 experiment: lower the internal resolution one level while a named move's status is active; see [Rendering profiles](RENDERING.md) |
| `drs_adaptive` | `true`, `false` | 0.5.2 experiment: lower the resolution one level after a run of presents slower than the threshold, restore after a run within budget |
| `drs_adaptive_threshold_ms` | `10.00`–`40.00` (up to two decimals) | Present interval that counts as a late frame; 16.67 is one vblank, the default 17.5 allows half a millisecond of jitter |
| `drs_adaptive_trigger_frames` | `1`–`60` | Consecutive late presents before the resolution drops (default 6, one tenth of a second) |
| `drs_adaptive_release_frames` | `1`–`600` | Consecutive presents within budget before the resolution is restored (default 90, 1.5 s) |
| `ultra_resolution` | `576`, `720`, `1080` | Console LLUltra's base resolution (default `576`, the fork's choice). The 0.7.0 compact menu folds the former UltraRes row into one **NetProfile** stop: LLUltra / LLUltra+ / LLUltra++ mean 576p / 720p / 1080p. Left/Right leaves that stop in either direction and remembers the resolution; entering it recalls the saved value. While manual console LLUltra is selected, **L+R+X** cycles 576 → 720 → 1080 → 576 in place ([Controls](CONTROLS.md)). Existing INI files need no migration: `render_mode=llultra` and this key remain separate. `1080` is offered for overclocked consoles because the operator reports 720p held 60 under Horizon OC (statement, no capture). DRS still steps down from the base; emulator and other console profiles ignore the key. Other numbers are refused. See [Rendering profiles](RENDERING.md) |
| `hide_pages` | `none`, or a comma list of `plugins`, `own_pages`, `performance` | 0.5.3: pages the window chord skips. `plugins,performance` makes the cycle Settings → a plugin's own page (FX Control) → Window Options → hidden, so a measurement session reaches the plugin page in one tap and the Window Options page in one more. Settings and Window Options are always in the cycle. Since 0.6.0 the three are the rows **PluginsPage**, **OwnPages**, **PerfPage** (Shown / Hidden) of the **Window Options** page (until 0.5.7 they were Settings rows), which toggle them live; Debug keeps its own chord. See [Controls](CONTROLS.md) |
| `early_present` | `true`, `false` | 0.5.3: present the acquired image at the flush that submitted it, the 0.4.0 fix for the two-image profiles' character-select stall (LessLag; LLUltra and LLDoubles since 0.5.2). `false` keeps the game's own present order everywhere, which is what upstream does. Read at start. The before/after is shown in [Optimizations added by this rewrite](../README.md#optimizations-added-by-this-rewrite) |
| `vanilla_plus` | `true`, `false` | 0.5.6: Vanilla+. With it on (the default) the Vanilla profile keeps the game's own three-image swap chain but presents the acquired image at the flush that carried that frame's last submission, one CPU update earlier than the game presents it, as the render-opts profiles do (see [Vanilla+](RENDERING.md#vanilla-056)). `render_mode` defaults to `vanilla` and `auto_profile` to `true`, so the shipped default is Vanilla+, and Auto's Vanilla choice outside matches is Vanilla+ too. The NetProfile row, the strip and the native banner read `Vanilla+` (`Vanilla+*` in the cleanup hold, `Auto:Van+` in Auto) while it is on and plain `Vanilla` when off. Toggled live by **L+R+X** ([Controls](CONTROLS.md)), which saves it, except while manual console LLUltra is selected: that chord cycles `ultra_resolution` instead and leaves this flag unchanged. An external edit needs a restart. `early_present=false` still disables every early present, Vanilla+ included. The peer exchange still announces Vanilla. The first console boot (0.5.6) counted `early_present_mismatches=0` over 89,500 presents; the visual, chord and latency checks are owed ([Status](STATUS.md)) |
| `dispatcher_backoff` | `true`, `false` | 0.5.3: let the dispatcher idle-backoff hook sleep ([Dispatcher backoff](DISPATCHER_BACKOFF.md)). `false` leaves the hook installed but asleep, so the game spins as upstream does; the Performance row **Backoff** reads `Off (ini)`. Read at start. The `DISABLE_BACKOFF` flag still skips the install entirely |
| `perf_view` | `graph`, `number` | 0.5.3: the frame-time block at the bottom of the vertical Performance and Debug pages as the 180-sample plot (default) or the mean as text, which is less draw work while measuring; also the row **PerfView**, which is on the Window Options page since 0.6.0 (until 0.5.7 a Settings row). Since 0.6.0 the Settings page has no plot, only the one-line performance footer, so the key does not affect it. The compact match strip is text either way |
| `hints` | `true`, `false` | The dim chord hints under Settings and Window Options (including the ZR+X Stealth / ZR+Y Streamer shortcuts in the 0.7.0 compact menu, which act only with Settings' You row selected). Default `true`; an older file without the key keeps the hints. The Window Options **Hints** row toggles it live; an external edit needs a restart. A pending Stealth reconnect notice remains visible with hints off. See [Controls](CONTROLS.md) |
| `hoc_oc` | `always`, `online` | 0.9.0: when Horizon-OC may apply Smash's saved profile. Needs the game-aware hoc-clk fork; stock Horizon-OC ignores it and ZR+L3 does nothing. `always` (the default, and a missing key) is Horizon-OC's own behavior. `online` holds the profile back, using Horizon-OC's own global profile or stock clocks instead, except while the game is connected to another player (Pia RTT samples, internet or local wireless) and for 90 s after the last sample. The profile itself is never read or written by Deluxe, and no clock value is sent. Written only when `online`, so the default file above stays loadable by older builds; an older build resets a file that contains `hoc_oc=online`. Toggled live by **ZR+L3** on Settings ([Controls](CONTROLS.md)) |

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
| `DISABLE_BACKOFF`, `DISABLE_NATIVE`, `DISABLE_MENU`, `DISABLE_SCENE`, `DISABLE_BUFFER`, `DISABLE_RENDERER`, `DISABLE_PIA`, `DISABLE_HOC` | Bisection switches; see [Debugging](DEBUGGING.md). `DISABLE_FX` is ignored since 0.5.0 |
