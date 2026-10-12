# Reduced FX emulator measurements

## Explosive Flame at normal resolution — 2026-10-08

This configuration did not establish a repeatable On-over-Off performance
penalty. Reduced's performance goal remains **unverified**; no percentage of
Off's benefit is claimed. Visual checks separately confirmed an intact flame
burst in Reduced, suppression in Off, and normal cleanup.

Tested artifact:

```text
NRO SHA-256: 833cc40bb7db272108684f96972f46103a3f876470dff9e1ecee974776c0f53d
Eden SHA-256: 0b41c3c0c78fccd53615895abf86a2fc3cdaa51bcf8b42cc7d4df0b732e6fc23
Game: 13.0.5
CPU: Intel Core Ultra 7 258V
Graphics: Intel/ANV, Vulkan, High accuracy, Immediate host presentation
Scale: 1×; guest internal resolution 1080p
Speed: Standard, 100%; FPS++ unapplied; DRS off
```

One Eden instance ran the complete series. Training used Palutena against an
idle Greninja, with a saved grounded position placing the leftward far flame
onscreen between the fighters. It did not hit either fighter. Training Modpack,
the corrected gamepad overlay, G&W fix, Elite Legal Stages and standalone ImGui
remained installed.

Each profile had one warmup per FX mode followed by five trials per mode.
Condition order was counterbalanced in five blocks. All 36 ten-second captures
completed without truncation. Thirty measured trials, excluding the six
predeclared warmups, passed metadata checks. Before/after snapshots kept the
committed profile and resolution unchanged; shader polling reported zero.
The policy log calls LLUltra `ultra`; its older `mode` field records the startup
mode and is not used as the live-policy check.

The move window starts at the first guest sample containing B and lasts 2.5
guest seconds. B plus left-stick input was received for approximately 400 ms.
The preceding 0.75 seconds supplies a local baseline. No screenshots, pause,
menu edits or setting changes occurred inside a capture. CLI status polling
ran every 0.5 seconds under every condition.

The table shows the **median of five per-trial p99 intervals**, in milliseconds.
These are two different CPU observations; neither measures displayed frames.

| Profile | Observation | On | Reduced | Off |
|---|---|---:|---:|---:|
| LessLag | Render callback | 17.47 | 17.76 | 17.60 |
| LessLag | Guest present-call starts | 18.79 | 19.31 | 19.84 |
| LLUltra | Render callback | 18.17 | 18.38 | 18.50 |
| LLUltra | Guest present-call starts | 18.24 | 18.62 | 18.59 |

The distributions overlap, including their move-window excess times. Occasional
slow intervals also appeared in Off. A single maximum or a near-60 average does
not isolate the effect's cost. The analysis helper's noise rule is descriptive,
not a significance test; this conclusion also follows from the raw repeated
trials. No trial was discarded merely because its baseline exceeded 16.67 ms.

Raw captures, per-trial settings/status snapshots, input receipts, interval
tables, scripts and SHA-256 manifest are preserved locally:

```text
/home/intpa/.local/share/ssbu-mod-backups/fx-palu-833c-20261008/
```

A controlled higher-resolution stress run can test whether the filter removes
measurable work, but it would be synthetic stress, not a reproduction of the
first-generation Switch's bottleneck. Switch performance and other effect
families remain separate acceptance work.

See [the methodology](FX_PERFORMANCE_METHOD.md) for the acceptance target and
the distinction between CPU cadence, GPU work and input-to-photon latency.


## Additional visual and diagnostic checks

The same 833c session checked full Gigaflare On/Off/Reduced, Shadow Flare
charge/projectile cues, Chrom's first side-B trail, Inkling's running roller and
Ike's full eruption. Reduced retained identifiable visuals and effects expired
after the move. Shadow Flare and roller cleanup also survived switching to On
while effects were active. Chrom's very short trail was inspected at 20% speed,
with 100% restored afterward; no timing claim uses those screenshots.

Ike's finite decision trace recorded `ike_volcano_add4` and `ike_volcano_add2`
as Reduced culls, while retaining the hold, volcano and middle-body requests.
On, Reduced and Off casts all produced the observed 10% self-damage and 39%
target damage. This is a narrow observation, not a simulation checksum.

Six unpaused full-Gigaflare pilot captures (one On/Off/Reduced per profile)
completed their ten-second windows at 1x. They do not establish a repeatable
On-versus-Off deficit or a Reduced performance benefit. The remaining normal
eruption bodies can still be substantial; counting removed emitters is not a
measurement of cost.

The Inkling visual comparison found that Off still showed the original roller
splash because its legacy name did not match the three actual request names.
That correction needs its own new-build runtime check. Earlier leftward roller
attempts ran off the Training platform and are excluded from cleanup acceptance.

Additional raw evidence, scripts and hashes are preserved at:

```text
/home/intpa/.local/share/ssbu-mod-backups/fx-lifecycle-833c-20261008/
```

## Higher-resolution Palutena pilot — 24e

At 2x host resolution on the same Intel/ANV machine, six excluded warmups and
six pilots completed: one On/Off/Reduced pilot under each of LessLag and
LLUltra. Training used idle Joker and a saved wide grounded arrangement with
the far flame fully visible between the fighters, missing the CPU. Earlier
offscreen fixture attempts were rejected before measurement.

Each ten-second trace completed without truncation, with zero sampled shader
compilation. Game-speed and graphics settings remained fixed. Move-window
guest present-call p99 values were, in On/Reduced/Off order:

| Profile | On | Reduced | Off |
| --- | ---: | ---: | ---: |
| LessLag | 18.70 ms | 20.15 ms | 18.67 ms |
| LLUltra | 18.45 ms | 18.05 ms | 18.52 ms |

This one-trial-per-condition pilot did not reproduce a clear On-over-Off cost.
It is neither a repeated-trial performance pass nor evidence that Reduced
regresses. No benefit ratio is claimed. The 2x scene is synthetic stress and
differs from the earlier 1x fixture; it is not a controlled resolution A/B.

## Higher-resolution Gigaflare pilot — 24e

Full Gigaflare was visually confirmed before the 2x series, with the saved
Sephiroth-versus-idle-Falcon fixture. Six warmups and six pilots completed across
both profiles, one pilot per FX mode per profile. The 7.5-second B-anchored
move windows all passed metadata checks without trace truncation.

| Profile | Present-call p99 On | Reduced | Off |
| --- | ---: | ---: | ---: |
| LessLag | 19.30 ms | 19.35 ms | 19.81 ms |
| LLUltra | 18.65 ms | 19.09 ms | 19.40 ms |

These pilots again did not show a clear On-over-Off cost; they establish no
Reduced performance benefit. Do not interpret the ordering of single trials
as a reliable ranking. The captures observe CPU-side cadence, not GPU completion.

The 24e Palutena and Gigaflare raw pilots, settings, receipts, analyzer and
SHA-256 manifest are preserved together:

```text
/home/intpa/.local/share/ssbu-mod-backups/fx-stress-24e-20261008/
```


## Blue Special Zoom, 2026-10-08

Candidate `7c43572918cdc473` reproduced Captain Falcon's connecting neutral-B
Special Zoom against idle Fox on the Training stage (31.5% / one hit). On,
Off and Reduced screenshots show the critical presentation. A fresh single
Eden process used Vulkan/ANV, accurate GPU, 1x, 100% speed, FPS++ Off and the
same full mod stack. Its 9 GiB/no-swap cgroup recorded no limit or OOM events.
The diagnostic `PROBE_CRITICAL_SCREEN` marker was absent.

After warming all three modes, five counterbalanced trials per FX mode and
render profile produced **30 complete captures** with no shader compilation,
trace gaps or menu visibility. The measurement is guest CPU present-entry
cadence, not GPU completion or displayed frames. The impact interval is
B-button receipt + 0.65–1.80 seconds, anchored from the raw timing CSV; it
approximates the observed zoom, not a logged background-request lifetime.
Host-time input leases differ by about one game frame between trials.

| Profile | FX | Median trial impact p99 | Worst impact interval | Impact intervals >25 ms |
| --- | --- | --- | --- | --- |
| LessLag | On | 18.97 ms | 19.64 ms | 0 / 345 |
| LessLag | Off | 18.91 ms | 20.85 ms | 0 / 345 |
| LessLag | Reduced | 19.06 ms | 24.01 ms | 0 / 345 |
| LLUltra | On | 18.62 ms | 20.04 ms | 0 / 343 |
| LLUltra | Off | 18.60 ms | 19.16 ms | 0 / 343 |
| LLUltra | Reduced | 18.38 ms | 19.28 ms | 0 / 343 |

**No repeatable critical-zoom lag spike was reproduced in this fixture.**
A preceding six-trial pilot had one 34.12 ms LessLag-On interval; the five
subsequent On repetitions did not reproduce that impact hitch. Occasional
longer intervals outside impact remain in the raw data, including a 35.47 ms
LessLag-Reduced recovery interval. They are not discarded or attributed to
zoom without evidence.

All modes retain the blue background. Code 96 recorded no allowlisted
critical-background requests through the installed boundaries, so the
diagnostic suppression path was not runtime validated. This is **not** a
background-present/background-absent comparison and does not establish zero
GPU cost, a Reduced benefit, or Switch performance. No zoom appearance change
was accepted. Red final-stock Finish Zoom is a different, untested fixture.

Raw `RESULTS.json`, all 30 API/timing CSV pairs, status/config snapshots and
the bounded driver are archived under the critical-zoom evidence directory
in `~/.local/share/ssbu-mod-backups/`. The earlier 24e pilot had baseline
variation and a CPU-heavy watcher; it is exploratory only. The six 7c pilot
CSV copies initially raced file export, were refreshed from finalized
originals and are recorded in `completion-refresh.json`; the repeated
driver waits for both completion markers before copying.

## Follow-up to the peer's LessLag/LLUltra report — c0, October 8

The user clarified that “with the mod” in the peer's report meant LessLag or
LLUltra. The claimed CPU hitbox cause remains a hypothesis. This follow-up did
not reproduce a consistent On-specific slowdown on this host, and it does not
establish an Off or Reduced performance benefit.

The same Intel Core Ultra 7 258V / Vulkan ANV host ran one Eden instance with
Smash 13.0.5, High GPU accuracy, 1x/1080p, 100% speed, inactive DRS and FPS++
Off. The embedded Deluxe window was hidden during capture. TMP, corrected
SGPO, ARCropolis and the fixed G&W plugin were present; standalone ImGui,
Elite and the peer's separate effect-removal assets were absent. There were
no sampled shader builds during the measured windows.

~~~text
NRO SHA-256:
c0e9a625eb44a7359545cb3264b2e7e58d45cf15ea7c7d79ba721fbb5be8dce7
Eden SHA-256:
0b41c3c0c78fccd53615895abf86a2fc3cdaa51bcf8b42cc7d4df0b732e6fc23
Raw evidence and per-file hashes:
/home/intpa/.local/share/ssbu-mod-backups/c0-all-runtime-20261008/evidence.zip
Summary inside archive:
c0-all-a/palu-36-measured-summary.json
~~~

Training used a stopped Ivysaur. A saved hit fixture connected Explosive Flame
for 18.2% / seven hits; a separate whiff fixture kept the visible explosion
between the fighters at zero damage. Each fixture covered Vanilla, LessLag
and LLUltra crossed with On, Off and Reduced. There were 54 ten-second callback
captures: 18 excluded warmups and **36 measured trials**, two per cell. The
second repetition reversed FX order. Mode/config snapshots, actual render-wait
skips and the guest B sample were checked. Input leases were timed by the host,
not frame-exact TAS. Screenshots were taken outside measured windows.

The analysis uses guest B receipt through B+2.5 seconds and a consistent
B-0.8-second pre-move baseline. The table reports the **worst present-entry
interval across two measured trials**, in milliseconds; it is descriptive,
not a ranking or significance test.

| Fixture | Profile | On | Off | Reduced |
| --- | --- | ---: | ---: | ---: |
| Hit | Vanilla | 19.690 | 19.009 | 18.913 |
| Hit | LessLag | 22.103 | 19.455 | 18.920 |
| Hit | LLUltra | 20.581 | 19.338 | 18.907 |
| Whiff | Vanilla | 19.904 | 20.423 | 19.080 |
| Whiff | LessLag | 18.460 | 18.670 | 18.898 |
| Whiff | LLUltra | 21.909 | 25.064 | 25.113 |

Hit trials contained 2,687 move-window present intervals, none over 25 ms.
Whiff trials contained 2,684, with two just over 25 ms: one LLUltra-Off and one
LLUltra-Reduced. Across both fixtures, all 5,386 callback intervals were below
25 ms. Neither the isolated maxima nor near-60 cadence proves every displayed
frame met its deadline.

The extra c0 diagnostics filled the API trace's 8,192-event buffer after
roughly 4.4–5.8 seconds. Every accepted API file retains its actual
`reason=capacity` metadata, and coverage through at least B+3 seconds was
required individually. Only the fully covered move/baseline slices are
reported. The callback files completed all ten seconds with no skipped
samples. The full-duration API analyzer's result was withheld; these are not
complete ten-second API measurements. Earlier hit/hit2 attempts that failed
the capacity gate are retained as failed evidence and excluded.

Fable 5.1 xhigh independently recomputed all 27 hit windows, including warmups,
and checked configuration, engine commitment, input anchors and API coverage.
Its review did not analyze the whiff matrix; the root reviewed that completed
matrix. The review supports a bounded negative result, not a general claim
that LessLag cannot slow an emulator.

Both hit and whiff create and check the attack's hitboxes. The hit additionally
changes hitlag, damage, reaction and impact visuals; positions, facing and
camera also differ between fixtures. Therefore comparisons are within each
fixture only. These CPU-side callback/present observations cannot attribute
cost to hitbox creation, measure GPU completion/scanout or predict Switch
performance. No gameplay or hitbox edits were made.

Further reproduction needs the peer's emulator/version, hardware, graphics
settings and scene, or a trace from the affected setup. Repeating the same
local fixture would not resolve those differences. Switch On/Off/Reduced
measurements remain necessary before claiming Reduced avoids its lag spikes.
