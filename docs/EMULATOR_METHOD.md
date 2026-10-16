# Mimicking the Switch in Eden

Eden on a desktop CPU does not show the frame drops the console shows. The
emulator's per-frame CPU work is a fraction of the 16.7 ms budget, so a
two-image profile never misses a refresh and the character-select stall, the
move drops and the effect of the dispatcher backoff are invisible. To make an
emulator clip or capture say something about the console, the emulator has to
be **biased** until its CPU budget per frame resembles the console's. This
page records how that is done for this project's clips and A/B runs.

## What the Switch experiences

From the console captures in [validation](VALIDATION.md) and the move ledger:

- The CPU is four Cortex-A57 cores at 1020 MHz (the Performance row **Clock**
  and the `# clock=` capture header record it since 0.5.3). The main thread's
  frame update is about 10 ms at idle in Training and 13.7 ms on the
  character select (16.3 ms while the hand moves); moves such as Kirby Hammer
  or Blazing End pushed the LLUltra work p90 to 12–16.5 ms in the 2026-10-15
  captures.
- The GPU at 576p takes 3.4–5.3 ms per frame under LLUltra; LessLag and
  Vanilla pipeline it behind the next frame's CPU work.
- A two-image profile (LessLag, LLUltra) has no third buffer to absorb a late
  present. If the frame's CPU work plus the present lands after the vblank,
  the next acquire waits a whole refresh: the exact 33 ms frames of the
  2026-10-14 console captures (character select at 49.9 fps before 0.4.0).
- The game's dispatcher spins on empty queues while a worker finishes a job;
  on the character select that is about 5 ms of every main-thread frame. With
  CPU to spare it costs nothing; without, it is the margin that makes the
  present late.

## What the bias is

A **CPU quota on Eden's systemd unit**, with a short period so the throttling
is felt inside every frame rather than once per 100 ms:

```powershell
systemctl --user set-property <eden unit> CPUQuota=120% CPUQuotaPeriodSec=5ms
```

`120 %` means the whole Eden process tree may use 1.2 cores' worth of time in
each 5 ms period. Eden's guest CPU thread then runs at a cadence close to the
console's, and the two-image stall reappears on the character select. The
settings this project has used:

| Quota | Period | Used for | Result |
| --- | --- | --- | --- |
| 175 % | 5 ms | Dispatcher backoff A/B/A on the character select, LessLag (0.3.1) | 52 → 59 fps with the hook, 18 % less CPU per frame; Vanilla and LLDoubles stayed at 60 |
| 120 % | 5 ms | FX Control effect campaign (2026-10-15), present-at-flush before/after clips | reproduces the console's character-select dip; every FX mode sits at the quota's ceiling, so On/Off/Reduced compare by worst interval |
| none | — | latency trials, control runs | 60 fps everywhere; the stall does not appear |

The quota is applied after the run starts (the unit name is in the launcher's
`run.json`), kept identical for the "before" and "after" state of an A/B, and
written into the notes of the run. If a scene shows no drop at 120 %, the
quota is tightened in steps (100 %, 90 %) until the "before" state drops and
the same value is used for "after"; the value used is stated with the clip.

## What the bias does and does not reproduce

- It reproduces the **CPU-side** budget: frame update time against the
  16.7 ms vblank, the dispatcher spin, the present position in the frame.
  Those are what dispatcher backoff and present-at-flush act on, so an A/B
  under quota is a fair test of their mechanism.
- It does **not** reproduce the console GPU (the desktop GPU finishes a 1080p
  frame in a fraction of the Switch's 576p time), memory bandwidth, the
  Horizon scheduler or thermal behaviour. Emulator frame rates are therefore
  not console numbers; a quota that gives 52 fps in Eden says nothing about
  the console's 49.9 except that the same mechanism is at work.
- Eden's own frame pacing adds noise. Clips use the Performance view's own
  fps line (the plugin's render-callback count, the same measurement as on
  the console), not Eden's status bar.

## Procedure used for the clips

1. Private display (headless weston + Xwayland, `tools/PrivateDisplay.ps1`),
   isolated Eden profile, the build under test staged in its `sdmc`, Vulkan,
   docked 1080p, speed limit 100 %, no Eden frame-rate or resolution tweaks.
2. Deluxe `config.ini`: `platform=emulator`, `render_mode=lesslag`,
   `auto_profile=false`, then the switch under test
   (`early_present=false` / `dispatcher_backoff=false` for the "before"
   clip, both `true` for "after"). The INI is read at start, so each state
   is its own Eden run.
3. Launch, apply the quota, load the game, open the Deluxe window on the
   **Performance** view and leave it open: the fps line and the `Backoff`
   row are part of the clip.
4. Record 6 s at 12 fps, 640 px wide, through the emulator's own GIF
   recorder (game frames, not a desktop capture): character select with the
   hand sweeping the roster; Training on the Training stage, LessLag, Pyra's
   Blazing End at a stopped CPU.
5. Copy `renderer-status.txt` after each clip (`early_presents`,
   `early_present_skips`, backoff counters, `clock`) and the recorder's frame
   and drop counts into the run notes.

The clips live under `docs/media/toggles-*.gif`; the run notes and the exact
quota are in the evidence root beside the captures.

## Result of the 2026-10-16 clips

Build `0.5.3-d10fce5eeb59`, quota 120 % / 5 ms for both states, LessLag,
docked 1080p, Eden GIF build on the private display. Read from the
Performance view in the clips and `renderer-status.txt` after each clip:

| Scene | Switches off | Switches on |
| --- | --- | --- |
| Character select, hand sweep | FPS 35, frame time 28.6 ms, Backoff `Off (ini)`, `early_presents=0` | FPS 58, 17.2 ms, Backoff 1 840/s 40 µs, `early_presents=4119` (one per present) |
| Training, Pyra Blazing End at a stopped CPU | FPS 29, 34.8 ms, `Off (ini)`, 0 | FPS 52, 19.2 ms, 1 738/s 50 µs, 10 205 |

The recorder reported 73 / 73 / 69 / 72 frames with 0, 0, 3 and 1 dropped
over the 6 s clips. The off state is harsher than the console's 49.9 fps
because the quota is a blunt instrument, not a calibrated Switch; the
direction and the mechanism are what the clips establish.

Caveat for every emulator run: the **Clock** row and the `# clock=` header
read the host CPU (about 4.2 GHz here, reported as `oc`); they mean
something only on the console.
