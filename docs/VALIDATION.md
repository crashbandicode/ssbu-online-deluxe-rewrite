# Validation record

## 0.9.0 — release source and host gates — 2026-10-31

0.9.0 merges `fix/lobby-practice-window` (head `326236e`) into main. That
head reverts the branch's 10-29 telemetry, so the merged `src/`, `build.rs`
and `sdk/` are identical to `3ea3c49`, the candidate on the card since
2026-10-28 (entry below). The three "Unreleased" entries below are part of
0.9.0. Its working label 0.8.2 in the docs and source comments became 0.9.0
(comment text only).

**Build change.** `crate-type = ["cdylib"]`: as a cdylib and an rlib the
library got no LTO from Cargo, so `lto = "thin"` never applied.
`tests/plugin_sdk_roundtrip.rs` includes `src/plugin_api.rs` by path instead
of linking the rlib. Resident size (NRO header `.text` + `.rodata` + `.data`
+ `.bss`): 1,515,520 → 1,200,128 bytes against the `3ea3c49` card build; the
40 exported symbols are unchanged.

**Gates (merged tree).** 311 unit and 21 integration tests; clippy with
warnings denied, host and Skyline; the no-default-features build; the Skyline
release build. `cargo fmt --check` lists six files whose formatting predates
this release.

Release assets, the Eden check and the card install follow in their own
entries.

## Unreleased: L+R+X only on the highlight; refresh-clock counters from boot — 2026-10-28

The operator asked for two corrections to candidate `062bf65`:

- L+R+X must toggle only what is highlighted.
- The 120 Hz plugin's diagnostics should be observed from the start of a
  session, so the data exists without a restart.

**Changes.**

| Commit | Change |
| --- | --- |
| `866d8b2` | L+R+X acts on the framed mode with You highlighted, and on Vanilla+ (or the manual console LLUltra resolution) with NetProfile highlighted. It does nothing on any other row, page, or with the window hidden; `062bf65` flipped Vanilla+ from all of those. The hint line names the rule. |
| `791f22b` | Every 5 s from boot the diagnostics thread reads the refresh-clock `status.txt` and records code 120: interval VSync events and game ticks, refresh, swap interval, normalizing / valid / observe-only bits, mode transitions and docked target, plus the interval's system ticks. The rows reach `events.log` and the running stage log. Deluxe only reads the file. |

The refresh-clock plugin was not changed. Its `telemetry.flag` was already on
the card, so its status file updates every second from launch.

**Gates.**

- `cargo clippy --all-targets -- -D warnings` is clean.
- `cargo build --no-default-features` succeeds.
- `cargo test`: 311 passed, including a parse of the 2026-10-28 card status
  file.
- The Skyline build had no warnings.

**Card install (DBI, 2026-10-28 20:05:15Z, `installed_verified`).** Installed
at the operator's request.

- Build `0.8.1-3ea3c490c9dc-1793217844`, NRO SHA-256
  `a0a4c9a04409bea8a6225995c9c31ccbd9f275a3a8e991e22e3357a37406b79b`
  (1,474,560 bytes).
- It replaced candidate `062bf65` (`0e58e76a…`). The config `20de5c13…`
  (`hoc_oc=online`), the hoc-clk fork, the FX Control and Horizon-OC configs
  and the other plugins were unchanged.
- The refresh-clock folder holds `status.txt` and `telemetry.flag`.
- **Rollback:** copy `backups/pre-highlight-deluxe-1028.nro` over the plugin.
- Receipt: `usb-highlight-1028/deployment.json`.

Not run yet on the console.

## Unreleased: You-line mode focus and per-stage logs — 2026-10-28

**Why.**

- The 2026-10-28 log showed ZR+L3 switching `hoc_oc` three times during
  online Singles matches. The operator confirmed none were intended.
- The operator asked to consolidate the You-line shortcuts:
  - ZR+D-pad Left/Right picks a mode icon, highlighted in light blue;
  - L+R+X toggles it;
  - L+R+Down keeps opening the window and stepping pages.
- The operator also asked for one log per match, and for everything the 120 Hz
  checks need to be logged up front, so the A/B fits on one boot.

**Controls (`a49ec48`).**

- The per-mode chords are retired: ZR+X Stealth, ZR+Y Streamer, ZR+L3 OC.
- On the Settings You row, ZR+Left/Right moves the focus
  (`menu::YouIcon`): Stealth → Streamer → OC, wrapping. OC is a stop only
  while the clock shows.
- `runtime/ui.rs` draws the focus as a light-blue rounded frame,
  `LIGHT_BLUE` rgb(0.45, 0.78, 1.0).
- L+R+X (no trigger held) toggles the framed mode. On other rows, and with
  the window hidden, L+R+X keeps the Vanilla+ / LLUltra-resolution behaviour.
- The clock now stays in `hoc_oc=always`, drawn white with a hub
  (`MarkTone::Always`), so it can be switched back.
- The hint line names the single chord.
- The strip's width allowance counts the clock.
- Host tests cover the focus walk, OC skipping, held-press edges, the retired
  chords on every view and row, and the OC toggle only while the clock shows.

**Per-stage logs (`8d965ba`).** `matches/NNNNNN-<scene>-<stage>.log` for
every Training, Singles or Doubles stage, described in
[DEBUGGING.md](DEBUGGING.md).

- Header: settings, render profile, textures and present interval, clock, and
  the refresh-clock status.
- Body: the stage's events.
- Footer: present counts, a present-spacing histogram, pacing outliers,
  reacquires, the end render state, and the refresh-clock status again.
- `matches/index.txt` keeps 60; no directory listing is used.
- A replayed events batch is skipped by sequence.

**Gates.**

- `cargo clippy --all-targets -- -D warnings` is clean.
- `cargo build --no-default-features` succeeds.
- `cargo test`: 309 passed.
- The Skyline build compiled the runtime with no warnings.

**Card install (DBI, 2026-10-28 19:53:16Z, `installed_verified`).** Installed
at the operator's request.

- Build `0.8.1-062bf65b0227-1793217135`, NRO SHA-256
  `0e58e76a35ac3f46a625754903f46f180dbade5a2f532999b5ad52d387adda05`
  (1,474,560 bytes), from package `DELUXE-build-lobby/0e58e76a35ac3f46`.
  `runtime_tested=false`.
- It replaced release 0.8.1 `0c16374b…`, which read back identical before the
  write.
- Unchanged on read-back: the other plugins, the hoc-clk fork `c4c28125…`, and
  the FX Control and Horizon-OC configs.
- **Rollback:** copy `ultimate/ssbu_online_deluxe_rewrite/backups/pre-lobby-deluxe-1028.nro`
  over the plugin.
- Config write: since the in-match switches to `always` were accidental,
  `hoc_oc=online` was appended to the card's `config.ini`.
  - `c720724e…` (416 bytes) became `20de5c13…` (430 bytes), read back
    verified. That is the same file as the earlier `hide_pages` write:
    `vanilla_plus=false`, `hoc_oc=online`.
  - The original is saved as `backups/pre-hoc-online-1028.ini`.
- Receipts: `usb-lobby-1028/deployment.json` and `usb-lobby-1028/hoc1.json`.

**Not tested.** No Eden or console run yet. The light-blue frame, white clock,
per-stage files, lobby window and pacing events have not been seen running.
The console checks are in
[next checks](NEXT_SESSION.md#one-boot-checks-you-line-controls-lobby-window-120-hz--2026-10-28).

## 120 Hz tearing report — logs show no cause — 2026-10-28

The operator saw what looked like tearing while playing at 120 Hz (SaltyNX
and FPSLocker, docked 120 Hz, 60 FPS target, interval 2) with the
refresh-clock plugin `0.0.2-dev`.

**Evidence.**

- Read over DBI, with nothing written (`usb-logs-1028/read1.json`): this
  session's `events.log`, `renderer-status.txt`, 15 `CAPTURE_MATCH` /
  `CAPTURE_CSS` capture pairs, and refresh-clock `status.txt`.
- A read-only research worker analysed them (report
  `usb-logs-1028/TEARING-ANALYSIS.md`). Its present statistics were rechecked
  independently on three captures (`usb-logs-1028/check_presents.py`).

**What the logs show.** The quoted values were confirmed by the independent
recheck.

- Profiles:
  - The Quickplay and local-wireless matches ran Vanilla+ (three images; boot
    settings had `vanilla_plus: true`).
  - The later matches ran manual LessLag (two images), with Horizon-OC
    applied in most of them.
- 69,892 actual presents across 14 captures.
  - Median spacing 16.67 ms in every capture. In the long LessLag Singles,
    p99 was 16.84 ms and 16.88 ms.
  - Gaps under 12 ms were 0.21%, clustered at character-select motion and
    stage exits.
- No acquire returned the image presented last.
- Every early present had its duplicate skipped; `early_present_mismatches=0`.
- The texture-sync wait still ran on LessLag (median 7.86 ms, none over
  16.67 ms).
- refresh-clock was normalising as designed: 124 VSync events to 62 game
  ticks in 1.03 s, with refresh 120, interval 2 and target 60. Its only hook
  is the VSync-count worker; it never presents or picks a buffer.
- The once-per-second `telemetry.flag` status write shows no periodic stall
  in the captures.

**Conclusion.**

- The CPU-side present sequence was regular in both Vanilla+ and LessLag.
- The captures record API calls, not scanout. A tear inside the display mode
  or on screen would leave this trace unchanged, so the report is neither
  confirmed nor explained.
- Also not established: which save turned `vanilla_plus` off (code 24 did
  not name keys) and the present interval during each capture.

**Diagnostics added** (candidate `ae69cce`, see [DEBUGGING.md](DEBUGGING.md)):

| Code | Records |
| --- | --- |
| 117 | Present spacing outside 14–19 ms |
| 118 | An acquire of the image presented last |
| 119 | Present-interval changes |
| 24 | Each save's changed keys (b is now a mask) |

Codes 117 and 118 are rate limited; `renderer-status.txt` adds exact
`present_pace_outliers` and `present_reacquired_last` totals.

**Gates.**

- Clippy clean.
- `--no-default-features` build succeeds.
- `cargo test`: 302 passed.
- The Skyline build has no warnings: package
  `DELUXE-build-lobby/47fc0dc468b8bffc`, build
  `0.8.1-ae69cce7e744-1793214821`, NRO `47fc0dc4…`.
- The first diagnostics commit failed the Skyline build (an unsafe call in
  runtime code that host builds do not compile); `ae69cce` fixed it.

The console A/B plan is in
[next checks](NEXT_SESSION.md#lobby-practice-window-and-120-hz-tearing-checks--2026-10-28).

## Unreleased: lobby practice window — 2026-10-28

The operator reported that the Player Info window became the in-match strip
in the local-wireless lobby, and in the Quickplay lobby after loading a
different stage. The card's `events.log` for release `0.8.1-84da224d084f`
(read over DBI, nothing written; receipt `usb-logs-1028/read1.json`) shows
the cause.

| Time | Connection (code 73) | Stage (code 110) | Classified |
| --- | --- | --- | --- |
| 68277.9 s | Quickplay | `0x0` | Training (Training-mode bit set) |
| 68530.4 s | Local | `0x0` | Doubles |
| 68659.5 s | Local | `0x3` | Doubles |
| 68917.4 s | Offline (lobby, see below) | `0x3` | Doubles |
| 69155.0 s / 69166.4 s | Offline (lobby) | `0xe`, `0x1a` | Doubles |
| 69445.9 s | Offline (lobby) | `0x1a` | Doubles |

- **Cause.** Stage setup classified Training only from the game's
  Training-mode bit or the default waiting-room stage 311 (`0x137`).
  - Lobby practice can load any stage with more than two fighters, and its
    Training-mode bit was clear.
  - Those stages fell through to the fighter count and became Doubles.
  - The UI draws the strip for a ready Singles or Doubles scene
    (`menu::menu_shape`).
- **Fix (`dfa1608`).** Stage setup also reads the game's scene-mode values
  that the operator's Training Modpack fork and smash-gamepad-overlay v0.1.3
  use for the waiting room: initialized, melee `0x0d`, and mode `0x02010000`
  (local) or `0x0701xxxx` (online).
  - The reads are read-only and armed only when
    `app::smashball::is_training_mode` matches 13.0.5 byte for byte.
  - The fork's source notes that the game's practice-scene setup selects
    melee `0x0d` and the normal match setup never does.
  - Lobby practice then classifies as Training. That gives the window and,
    under Auto, the Training profile.
  - New code 116 records the raw values at every stage load.
- **Gates.**
  - `cargo clippy --all-targets -- -D warnings` is clean.
  - `cargo build --no-default-features` succeeds.
  - `cargo test`: 298 passed, including four new predicate and
    classification tests.
  - Skyline package `DELUXE-build-lobby/e7ae1fa0d5c48fa3`: build
    `0.8.1-dfa1608b3f73-1793213529`, NRO `e7ae1fa0…`, 1,462,272 bytes,
    `runtime_tested=false`.
  - Superseded by `0.8.1-ae69cce7e744-1793214821` (NRO `47fc0dc4…`), the
    same fix plus the diagnostics in the entry above.
- **Not tested.** No console or Eden run of the candidate yet; see
  [next checks](NEXT_SESSION.md#lobby-practice-window-and-120-hz-tearing-checks--2026-10-28).
- **Other observations from the same log.**
  - After the local-wireless visit, Deluxe never saw the second Quickplay
    entry, so the connection stayed Offline in that lobby. The online
    latency hook marked each match online at its start (code 30, source
    `0x16ccab8`). This is unchanged and not yet explained.
  - ZR+L3 switched `hoc_oc` three times, each during an online Singles match:
    to always at 69077.4 s, back to online at 69118.9 s, and to always at
    69221.0 s. The saved config therefore omits `hoc_oc` (always is the
    default), and the profile now applies offline too.

## 0.8.1 — console rematches — 2026-10-28

The operator tested rematches on the console with the same release and card,
and they worked: the clock stayed green between matches against the same
opponent.

Two logs were read over DBI afterwards, with nothing written. Both headers
name `0.8.1-84da224d084f-1793155065`, and the config is unchanged
(`99665630…`).

- `events.previous.log`: SHA-256 `107c00db…`, 40,533 bytes, 43647–44623 s.
- `events.log`: SHA-256 `0eb8d214…`, 260,564 bytes, 47334–55557 s.

The earlier boot was offline Training only. It shows handshake `a=4`, gate
`0b0110` throughout, and no profile.

The later boot ran offline Training and four offline Doubles stages at stock
clocks (107 `a=2`), then six online Singles matches in four online sessions:

| Session | Gate on (115 `0b1111`) | Online Singles (event 110) | Gate off (115 `0b0110`) |
| --- | --- | --- | --- |
| 1 | 52599.6 s | `0x2a` 52615.6–52877.1 s | 52910.2 s; 112 at 52907.1 s, 10,006 ms silent |
| 2 | 53009.7 s | `0x15b` 53022.0–53286.5 s | 53312.8 s; 112 at 53309.6 s, 10,010 ms silent |
| 3 | 53418.3 s | `0x0` 53430.7–53697.6 s, rematch `0x28` 53726.6–53989.3 s | 54012.1 s |
| 4 | 54057.7 s | `0x6b` 54071.5–54315.6 s, rematch `0x15b` 54340.1–54613.9 s | 54645.9 s |

- **Rematches.** Each rematch stage loaded 29.0 s and 24.5 s after the
  previous match ended. Event 115 did not change in either gap, so the profile
  stayed applied across the rematch.
- **Release timing.**
  - In sessions 1 and 2, event 112 bounds the last sample as in the entry
    below. The releases came 13.1–16.1 s and 13.2–16.2 s after it, which is
    consistent with the 15 s hold.
  - Sessions 3 and 4 logged no event 112. The gate went grey 22.8 s and
    32.0 s after their last match ended.
- **Measured CPU while applied.**
  - Most readings were 1770–1790 MHz.
  - Four single readings were lower with no gate change: 1646 MHz at
    53571.0 s, 1015 MHz at 53766.2 s (the next, at 53773.9 s, read 1787 MHz),
    1652 MHz at 53859.3 s, and 1592 MHz at 54337.1 s.
  - The probe is a 5 s estimate, and the log does not explain these readings.
- **Gate off.** Stock readings were 822–1025 MHz. Higher readings
  (1557–1789 MHz) came only at boot or while loading, with no event 115 row,
  as in the entries below.

External receipt: `usb-online-1027/read4.json`, with the decoded timelines
`read4-hoc-events.txt` and `read4-hoc-events-previous.txt`.

## 0.8.1 — console run of the 15 s release — 2026-10-28

The operator tested the installed release `0.8.1-84da224d084f-1793155065` (NRO
`0c16374b…`) with `hoc_oc=online` on the console, and it worked as expected.
The checks were:

- the clock turns grey about 15 s after leaving an online match;
- a rematch keeps it green;
- the window cycles only Settings and Window Options.

The card's `events.log` (SHA-256 `3ddd2788…`, 58,314 bytes) was read over DBI
afterwards, with nothing written. Its header names that build and the loaded
config: `hide_pages` all true, `hoc_oc: Online`. Times are seconds since boot.

| Time | Event | Measured CPU (event 111) |
| --- | --- | --- |
| 41746.6 s | 114 `a=4` (fork present), `caps=0x1`, protocol 1; 115 `0b0110` (armed, online mode), grey | |
| 41885–42642 s | offline Training stages `0x134` and `0x4` (event 110) | 1016–1022 MHz and 906–911 MHz (stock) |
| 42674.6 s | 115 `0b1111` (applied, online live), green, generation 10 | |
| 42676.8 s | Training stage `0x137`, the online wait | 1787 MHz at 42684.8 s |
| 42689.3–42848.4 s | online Singles stage `0x15f` | 1788 MHz, 1785 MHz |
| 42870.0 s | 112 `PeerStale`, station 1 silent for 10,002 ms | |
| 42874.2 s | 115 `0b0110`, grey, generation 11 | 1019 MHz at 42877.5 s |
| 42881.8 s | offline Doubles stage `0x2c`; 107 `a=3` drops the earlier match's latency mark, then 107 `a=2` at 42887.2 s | 1019 MHz |

- **Release timing.** The log does not time the last raw Pia sample.
  - The poll keeps a publication measured for up to 3 s after its sample
    (`pia_rtt::STALE_MS`). Event 112 puts the last measured publication at
    about 42860.0 s, so the last sample came between about 42857.0 and
    42860.0 s.
  - The release at 42874.2 s is therefore 14.2–17.2 s after that sample.
  - A 15 s hold requires the sample at or before 42859.2 s, which lies in
    that range.
- **Readings with the gate off.** Two higher readings come before any gate
  change: 1542 MHz at 41740.2 s, before the handshake, and 1660 MHz at
  41871.1 s, while loading. No event 115 row accompanies either.
- **Not in this log:**
  - The log holds one online match. The rematch case is recorded in the entry
    above.
  - The page cycle is not logged.
- **Config.** The card's `config.ini` (`99665630…`) differs from the one
  written before the run only in `vanilla_plus=true`, an in-game change by the
  operator.

External receipt: `usb-online-1027/read3.json`, with the decoded timeline
`read3-hoc-events.txt`.

## 0.8.1 — public release assets and card install — 2026-10-28

Installed and published at the operator's request. The
[Forgejo release](https://git.syntaxbyexample.com/intpa/ssbu-online-deluxe-rewrite/releases/tag/v0.8.1)
contains the NRO and `manifest.json`. Both were built from clean tag `v0.8.1`
at `84da224d084f5f7c08ac9501e86f071d483fcd3e`.

- Build ID: `0.8.1-84da224d084f-1793155065`.
- NRO: **1,462,272 bytes**, SHA-256
  `0c16374b11d3d592d79b98f1435a0acf3f339c836fbc91eb7be6ee40087b48d0`.
- Manifest SHA-256:
  `f67c1b8b6fd692a24ddc47d0486194951a5c70b146415b761a096ea0093596ca`.
- All **77** archived source inputs match the tag, and **1,138,752**
  executable bytes match the optimized debug ELF. The ImGui archive matches
  its pin (`09b296a0…`).
- Both assets were downloaded without authentication and hash-verified after
  publication at **2026-10-28T02:42:05Z** (release 633, assets 221/222).

**Card install (DBI, 02:40:54Z, state `installed_verified`).**

- The card's Deluxe NRO changed from the candidate `778e94a8…`
  (`0.8.0-788537dd6f6d-1793142603`) to the release NRO `0c16374b…`, which read
  back identical.
- Unchanged on read-back: the other plugins, the hoc-clk fork `c4c28125…`,
  and the Deluxe, FX Control and Horizon-OC configs.
- The Deluxe config already had the operator's page change of the same
  session: `hide_pages=plugins,own_pages,performance`.
- **Rollback:** copy `ultimate/ssbu_online_deluxe_rewrite/backups/pre-081-deluxe-1028.nro`
  (`778e94a8…`) over the plugin.
- At install the 15 s release had not yet run on the console. The
  published manifest's `runtime_tested=false` describes the build at
  publication. The console run is recorded in the entry above.

External receipts: `usb-081-1028/deployment.json`,
`release-081/PACKAGE-VERIFIED.json` and `release-081/PUBLICATION.json`. The
retained bundle with the NRO, debug ELF and source archive is
`DELUXE-build-081/0c16374b11d3d592/`.

## 0.8.1 — Horizon-OC online-only profile, 15 s release — 2026-10-28

0.8.1 releases the Horizon-OC online-only profile recorded in the three
entries below. After the console test the operator asked for the profile to
drop about 15 s after a disconnect unless a player reconnects.
`online_session::HOLD_TICKS` changed from 90 s to 15 s. The labels that said
0.9.0 now say 0.8.1, because only this work has landed since `v0.8.0`.

**Console event log of the 90 s candidate.** The log for
`0.8.0-788537dd6f6d-1793142603` covers its first boot, 42–1598 s. It was read
over DBI and nothing was written. Times are seconds since boot.

| Time | Event | Measured CPU (event 111) |
| --- | --- | --- |
| 56.0 s | 114 `a=4` (fork present), `caps=0x1`, protocol 1; 115 `0b0110` (armed, online mode), grey | |
| 106.1 s / 108.1 s | 115 `0` then `0b0110`: ZR+L3 off and on | |
| 240.7 s / 245.7 s | 115 `0` then `0b0110`: ZR+L3 off and on | 1789 MHz at 245.4 s, 991 MHz at 252.5 s |
| 335.5 s | gate unchanged (idle) | 1782 MHz, then 912 MHz at 342.8 s: the game's loading boost |
| 396.6 s | 115 `0b1111` (applied, online live), green | 1789 MHz at 399.9 s |
| 413.7–524.9 s | online Singles stage `0x6b` (event 110) | 1780 MHz at 487.7 s |
| 578.1 s | Training stage `0x134`, still inside the 90 s hold | 1789 MHz at 580.4 s |
| 628.3 s | 115 `0b0110`, grey: 90 s after the last sample at about 538 s | 905–915 MHz from 630.6 s to the end |

- About 910 MHz is a stock reading of the probe. Console logs from 2026-10-18,
  before the game-aware fork existed, alternate between about 1020 and about
  910 MHz at stock clocks.
- The previous boot ran the pre-fix build `0.8.0-cc6d8fdff592`. It logged 114
  `a=1 b=0x10801` and 115 `0b0100` with error `0x10801`, and Training ran at
  1787 MHz: the profile applied offline, as the entry below describes.
- With 0.8.1 the 628.3 s release would come about 15 s after the last
  sample, near 553 s.

**Why 15 s is safe.**

- 15 s is longer than the 10 s a connected peer may go without a sample
  (`pia_rtt::LIVE_MS`), which already covers loading gaps.
- A new sample inside the window keeps the session live.
- Auto's carried-mark check uses the same signal. If a real online match
  arrives after the session lapsed, it re-marks itself through the latency
  hook in 3.1–3.3 s. In the log above, the next opponent's samples began
  17 s before the match stage loaded.

**Gates on the 0.8.1 tree:**

- `cargo clippy --all-targets -- -D warnings` is clean.
- `cargo build --no-default-features` succeeds.
- `cargo test`: 294 passed, including `hold_is_fifteen_seconds`.

The Skyline build, NRO hash and release state are recorded with the release
assets. The 15 s hold was later observed on the console (first entry above),
not in Eden.

## Unreleased Horizon-OC online-only profile — console handshake failure and sm-session fix — 2026-10-27

Operator report after the card install of `0.8.0-cc6d8fdff592` with the
hoc-clk fork `c4c28125…`: no clock, and ZR+L3 did nothing.

- **Evidence:** the card's `events.log` (read over DBI) holds the only boot of
  that build. The first handshake recorded event 114 `a=0x1 b=0x10801` and
  event 115 `a=0 b=0x1_0001_0801`.
- **Cause:** `0x10801` is the kernel's `LimitReached`, returned by Deluxe's own
  `svcConnectToNamedPort("sm:")`.
  - Atmosphère's pm gives the Application resource limit `SessionCountMax = 1`
    (`stratosphere/pm/source/impl/pm_spec.cpp`).
  - The kernel only expands limits for system processes (`kern_k_client_port.cpp`).
  - Smash's SDK holds that one session for the whole process: `nn::sm::Finalize`
    is `mov x0, xzr; ret`.
  - Eden does not enforce the limit, so the Eden integration below could not
    show this.
- **Effect:** the client classified Horizon-OC as absent. No clock was drawn,
  ZR+L3 was ignored, and the fork was never armed, so the saved profile kept
  applying as in stock Horizon-OC.
- **Ruled out:** the service-access list. Skyline's `exefs/main.npdm` on that
  card grants the client wildcard `*` in both ACI and ACID, so sm's
  `GetServiceHandle` check allows `hoc:clk`.
- **Fix (`788537d`):** the client borrows Smash's registered `sm:` session.
  - It reads the session the way `nn::sm::GetServiceHandle` reaches it: the
    slot behind its `adrp`/`ldr`, the session object (checked by vtable), and
    the handle `nn::sm::Initialize` stored at `+0xc`.
  - Both functions must first hash to Smash 13.0.5's SDK, build id `5fd4f531…`.
  - HasService is still asked first. The SDK's `TryConnectToHipcService` and
    `IsHipcServiceRegistered` are abort stubs, and a deferred
    `GetServiceHandle` would stall every SDK thread's sm traffic on the
    shared session.
  - Host tests derive the pinned offsets from the SDK's instruction words.
- **Build and install:** `0.8.0-788537dd6f6d-1793142603`, NRO `778e94a8…`,
  installed over DBI and read back at 23:13Z. The other plugins, the fork,
  and the Deluxe, FX Control and Horizon-OC configs were unchanged.
- **Not run:** an Eden smoke test of the new route, because another Eden
  session was active.
- **Console result:** tested by the operator after a reboot. The test covered:
  - the grey clock while idle;
  - ZR+L3 switching the mode off and on;
  - the green (applied) clock in a live online match;
  - the return to grey after the match.

  The card's event log for this build was read back on 2026-10-28; see the
  0.8.1 entry above.

## Unreleased Horizon-OC online-only profile — Eden integration with the hoc-standin — 2026-10-27

Run at the operator's request as a sanity integration test. Eden GIF build
`9a5d704c9c`, isolated profile, plugins ARCropolis + Deluxe + `tools/hoc-standin`
(test-only; registers `hoc:clk` and answers protocol v1, no clock control).

1. **First boot, Deluxe `0.8.0-cc6d8fdff592`:** the stand-in registered
   `hoc:clk`, yet Deluxe recorded event 114 `a=1 b=0` (no service). A stand-in
   probe showed Eden's TIPC `AtmosphereHasService` reply
   `0000fe5c 00000002 00000000 00000000` (present = false) for `hoc:clk`,
   `nothere` and the built-in `fsp-srv` alike; the same query over CMIF
   answered present. Atmosphère accepts that query only over TIPC
   (`sm_user_service.cpp`), so the fix keeps TIPC on consoles and uses CMIF
   when `platform=emulator` (`dd1862e`).
2. **Second boot, Deluxe `0.8.0-93b4d7c27f56`:** stand-in log: session
   accepted, first use of commands 0, 100, 101 with kernel PID 81, `armed`,
   then 102 heartbeats. Deluxe: event 114 `a=4 b=0x10001` (fork present,
   caps 1, version 1) and event 115 `a=0x6 b=1` (armed, `hoc_oc` online, not
   applying, generation 1). Screenshots show the grey clock after the privacy
   icons and `ZR+L3 OC` in the hint line; ZR+L3 removed and restored the clock
   (mode always, then online), Stealth/Streamer and the NetLatency edit behaved
   as before.
3. **Performance page:** the Clock fact was clipped at the window edge, hiding
   the gate state (Memory too). Facts now wrap (`ac503c0`); the re-check with
   `0.8.0-ac503c0c48e7` reads `4739 MHz OC, HOC none, / OC online: idle`
   (emulator host MHz).

Not covered: the green (applied) state, which needs Pia samples from a
connected player, and every console behavior. Evidence: operator evidence
directory `horizon-oc-game-aware-tangent-20261027/eden-media/` (receipts, logs,
frames, review sheets).

## Unreleased Horizon-OC online-only profile — card install — 2026-10-27

Installed at the operator's request. Card install over DBI USB MTP
(console `XAW10006803734`), finished 2026-10-27T20:15:12Z, state
`installed_verified`.

- **Deluxe:** build `0.8.0-cc6d8fdff592-1793130065` (feature branch
  `feature/hoc-online-oc`; code identical at `76b5784`), NRO **1,458,176
  bytes**, SHA-256
  `b4a796195bd70b6f8bab8323e1a070011338661cf09f04dfbad3915d8967261d`. It
  replaced v0.8.0 (`77c3ba0d…23cb7`). Card rollback:
  `ultimate/ssbu_online_deluxe_rewrite/backups/pre-hocga-deluxe-1027.nro`.
- **hoc-clk:** the game-aware fork
  ([intpa/horizon-oc-game-aware](https://git.syntaxbyexample.com/intpa/horizon-oc-game-aware)
  `28d0e351`, Horizon-OC `2.6.0-rel` `7960dd9` + protocol v1), `exefs.nsp`
  SHA-256 `c4c28125cff7e17c30118e8595f14c224e3b85d9dda195b91004fbcfb7952576`.
  It replaced the official Horizon-OC 2.6.0 `exefs.nsp` (`9b5ae2fd…14204`,
  byte-identical to the release `dist.zip`). Card rollback:
  `ultimate/ssbu_online_deluxe_rewrite/backups/pre-hocga-hocclk-exefs-1027.nsp`
  (copy it back to `atmosphere/contents/00FF0000636C6BFF/exefs.nsp` and
  reboot).
- **Unchanged, read back byte for byte:** Horizon-OC `config.ini`
  (`4dbfd996…`, never written), the Deluxe and FX `config.ini`, the ARCropolis
  config, hoc-clk `toolbox.json` and the other twelve plugins.
- **Unchanged by listing:** the plugin, hoc-clk, `flags` (the 0-byte
  `boot2.flag`), `config/horizon-oc` and `ultimate/mods` directories.

Activation needs a console reboot (the sysmodule) and a Smash start (the
plugin). `hoc_oc` is absent, so the default `always` keeps Horizon-OC's own
behavior until ZR+L3. Receipts and local backups are outside the repository:
`horizon-oc-game-aware-tangent-20261027/usb-install-1027/` (`preflight.json`,
`deployment.json`, `before/`). Installed only; nothing has run yet, so
`runtime_tested=false`.

## 0.8.0 — public release assets — 2026-10-25

The [Forgejo release](https://git.syntaxbyexample.com/intpa/ssbu-online-deluxe-rewrite/releases/tag/v0.8.0)
contains the NRO and `manifest.json`, built from clean tag `v0.8.0` at
`d92bc3d12ebc3bbbe640a9624d8df8a60ba3b430`.

- Build ID: `0.8.0-d92bc3d12ebc-1792963475`.
- NRO: **1,445,888 bytes**, SHA-256
  `77c3ba0dff6c0fce7dd1f293610edbf1791f277e18de9eff19ea5e1a2c223cb7`.
- Manifest SHA-256:
  `0ea5d864a5fbdc7492fe0c821848516da343bad69be7e27b59bb43ba1aa6bb9c`.
- All **73** archived source inputs match the tag; **1,130,256** executable
  bytes match the optimized debug ELF. Native dependency and archive hashes
  were checked before publication.
- Both assets were downloaded without authentication and hash-verified after
  publication at **2026-10-25T21:25:37Z** (release 479, assets 204/205).

External receipts: `deluxe-auto-080/PACKAGE-VERIFIED.json` and
`deluxe-auto-080/PUBLICATION.json`; retained NRO, debug ELF and source archive:
`deluxe-auto-080/out/77c3ba0dff6c0fce/`. No device was deployed or run;
`runtime_tested=false` remains accurate for this release. Gates and their scope
are below.

## 0.8.0 — Auto observation freshness — 2026-10-25

The operator reported the preceding match's `Auto (Nf)` count persisting through
the next match. The old observation was session-scoped, cleared only on Offline;
the UI consumers already rebuilt their labels on each draw/update. Historical
latency-hook traces below include callbacks about three seconds after stage
load, so the investigation does **not** establish that the old hook ran only
after matches.

The fix reads the buffer at its network-update consumption site `0x16d81a8`
(`ldrb w9,[x19,#0x41]`) rather than relying on the calculation hook's snapshot.
The preserved Smash 13.0.5 executable verifies that this is the same field the
original calculation stores at `0x16ccaa0`; existing manual hook `0x16ccab8`
remains after that store/clamp. The new hook requires the exact 20-byte window
starting at `0x16d81a4`, reads only during the callback, and retains no pointer.

A single packed atomic state carries generation, eligibility, activity and
count. Stage entry starts a fresh unknown observation even for consecutive
Singles stages; results close sampling but retain the count. Offline, Training,
connection-mode changes and rejected stage context clear it. Manual selection/application invalidates
Auto observation until the next match. A token captured before the memory read
rejects publication after a lifecycle boundary. Diagnostic 113 records changed
counts, with stage event 110 providing the match boundary.

Completed on the 0.8.0 source tree:

- `cargo test`: **264 unit tests + 1 SDK integration test**, all passed.
  Eleven new Auto-state tests cover consecutive 4f→3f matches, unknown startup,
  within-match updates, stale tokens, result retention, manual-to-Auto suppression,
  boundaries and contending observations. Existing status-cell tests also pass.
- `cargo clippy --all-targets -- -D warnings`: passed, zero warnings.
- `cargo build --no-default-features`: passed.
- `cargo skyline clippy -- -- -D warnings`: passed, zero warnings.
- `tools/Build.ps1`: actual optimized Skyline release build passed.
- Changed Rust files pass rustfmt; `git diff --check` passes.

An initial compile rejected `u64::from` in const functions; lossless widening
casts fixed the toolchain incompatibility before the successful checks above.
External gate logs, native inspection, review and package/publication receipts
are under `deluxe-auto-080/` in the operator's evidence root.

**No 0.8.0 live-console or emulator run is claimed.** In particular the actual
network-callback timing and consecutive online match display still need the
[runtime check](NEXT_SESSION.md#earlier-evidence--2026-10-25). The new release
also includes the previously merged revision-5 plugin status cells; their
runtime acceptance is separate from passing host coverage.

## 0.7.0 — public release assets — 2026-10-25

The [Forgejo release](https://git.syntaxbyexample.com/intpa/ssbu-online-deluxe-rewrite/releases/tag/v0.7.0)
now includes `libssbu_online_deluxe_rewrite.nro` and `manifest.json`. The
existing `v0.7.0` tag and archived clean package were used unchanged; no retag
or rebuild from post-tag main was performed. All 71 archived source inputs
match the tag, 1,124,304 executable bytes match the ELF, and the package/native
dependency hashes were reverified before upload.

Both public downloads were read back and hash-verified:

- NRO SHA-256: `2ff456161c52522c6881956d0ad3a5a037a1a3fc74c8757a1e3a7afe49012cae`.
- Manifest SHA-256: `e01e26bd713e82bc5cf8b393afd79f9465a92dbcb42aa165d0029a28e3bd45b9`.

The earlier API access block no longer occurred with the established publisher.
The unchanged manifest's `runtime_tested=false` is its packaging-time field;
the later exact-binary Eden capture and operator reports are recorded below.
External receipt: `docs-notes-fx-1025/release-070/PUBLICATION.json`.

## Operator report — Eden and Erista Switch — 2026-10-25

The operator reports that Deluxe has been tested in Eden and on an Erista
Switch without issues observed, and works with the latest firmware, Atmosphère
and Skyline. The upstream versions checked on 2026-10-25 are:

- [Switch firmware 23.0.1](https://en-americas-support.nintendo.com/app/answers/detail/a_id/22525/~/nintendo-switch-system-updates-and-change-history#current), released 2026-09-30.
- [Atmosphère 1.12.1](https://github.com/Atmosphere-NX/Atmosphere/releases/tag/1.12.1), released 2026-10-16.
- [Skyline beta](https://github.com/skyline-dev/skyline/releases/tag/beta), released 2026-04-12, at commit [`2eb226fa9e4ab023dc00cb0db10ad4ec50ac4fa3`](https://github.com/skyline-dev/skyline/commit/2eb226fa9e4ab023dc00cb0db10ad4ec50ac4fa3). Skyline's runtime release uses a `beta` tag, not the Rust crate's version number.

This operator report is the basis for the README's testing and compatibility
summary. The release links identify the upstream versions; they are not new
agent-run compatibility tests. No plugin build hash or per-feature checklist
was supplied with this report, so build-specific evidence remains separate below.

The operator also confirmed that **dispatcher backoff's Eden benefit carried
over to the Switch**. This is a positive console observation; no isolated
numeric Switch A/B delta accompanied it. The 52 → 59 fps and 18% CPU-per-frame
figures in [Dispatcher backoff](DISPATCHER_BACKOFF.md) remain Eden measurements.

## 0.7.0 — offline UI capture review — 2026-10-24

The released NRO `2ff456161c52522c6881956d0ad3a5a037a1a3fc74c8757a1e3a7afe49012cae`
(`0.7.0-9a7f81ce1937-1792670948`) booted in an isolated, muted Eden profile
on Ubuntu with Smash 13.0.5 and the separate FX Control `8bf2bdb451266feb…`
companion. This is the clean release artifact, distinct from the Switch's
operator-tested `2f5a…` candidate below. No new console run is claimed.

Observed from actual screenshots and recordings:

- Compact Settings renders You with eye/TV icons, NetLatency and NetProfile,
  the performance footer and hints. The offline fixture has no Opp row.
- Down selects NetLatency; held ZR+Right changes Auto to 0, with `< 0 >`
  visible while held; ZR+Left restores Auto. You-row ZR+X/Y show the closed
  Stealth eye and green Streamer TV, then restore both off.
- L+R+Down visits Settings, FX Control, Stages, Performance, Window Options,
  hidden and Settings again. The two FX pages come from the companion.
  Captain Falcon's companion row cycles On → Off → Reduced → On; the All
  row is `Unavailable` during mixed settings.
- Training keeps the vertical window on the right; an ordinary offline match
  uses the compact Settings/Performance strips at the top left. Debug shows
  build `9a7f81ce1937`, Timing Idle and CSS/match capture flags off. Both
  L+R+Minus and L+R+Plus were used; Plus also opens the game pause menu.
- Training and the next ordinary character-select banner read
  `You: LessLag | Auto`. Quitting the Pyra/CPU Mario match on Hollow Bastion
  produces No Contest; the following character select reads
  `You: Vanilla* | Auto`, with manual LessLag saved and the cleanup hold active.

The fixture explicitly uses manual LessLag, `platform=emulator`,
`vanilla_plus=false` and privacy initially off; these are not shipped defaults.
The source display is 1920×1080. [Media provenance](MEDIA_PROVENANCE.md) records the
binary identities, transforms and media provenance. External evidence is in
`deluxe-media-refresh-1024/`: `capture-setup/`, `jobs/`, `raw/`,
`MEDIA-DRIVE.jsonl`, `decoded/`, `encoded/MANIFEST.json` and `encode-review/`.

This is bounded visual/control evidence, not the full negative-control suite
in [Next session](NEXT_SESSION.md), an online-peer test, a console LLUltra
resolution test or a performance/latency comparison. The post-v0.7.0 revision-5
status cells are not in this binary and were not exercised. Historical latency
and switch-mechanism media retain their original build/measurement scope.

**Ally recording follow-up:** the same two NROs also booted with Smash 13.0.5
in an isolated Windows portable profile. The native GIF recorder captured the
Deluxe Settings window; its three-second sample has 29 frames, one dropped,
and preserves three seconds of playback. No input macro was replay-tested.
These samples are evidence only; the published media retain the sharper
Ubuntu captures. The owned process exited normally with code 0, its task was
removed, 1,022 protected shared files remained byte-identical, and the private
settings/plugins were restored. Receipts are in `ally-runtime/` under the same
external evidence directory; executable identity is in
[Media provenance](MEDIA_PROVENANCE.md#ally-recording-check).

## 0.7.0 — release validation — 2026-10-22

**Scope.** Minor release of the accepted compact You icons, one console
NetProfile LLUltra stop, in-place **L+R+X** resolution cycle and You-row privacy
gate. Root product version **0.6.2 → 0.7.0**; SDK/ABI crates remain **5.0.0**.
The version bump preserves the candidate's Rust implementation. No fresh 0.7.0
console run is claimed.

**Clean release package.** Tag `v0.7.0` points to version commit
`9a7f81ce19374b1f783d35845c02b3fe3a5745ab`. Skyline release build
`0.7.0-9a7f81ce1937-1792670948`, NRO 1,441,792 bytes, SHA-256
`2ff456161c52522c6881956d0ad3a5a037a1a3fc74c8757a1e3a7afe49012cae`.
All 71 archived build inputs match the tagged commit; 1,124,304 executable
bytes independently match the optimized ELF. The native dependency hash,
module ID, source archive and manifest also verified. All 69 non-version
inputs match the operator-tested candidate; only root Cargo version entries
changed. Symbols and receipts are outside the repository in
`release-070-1022/out/2ff456161c52522c/` and `PACKAGE-VERIFIED.json`.

**Publication history:** version commit and tag were pushed to Forgejo on
2026-10-22. The NRO/manifest upload was initially blocked: the client was rejected by Cloudflare with
HTTP 403 / error 1010 before release creation. Publication completed on
2026-10-25 with the same archived package, as recorded above. No new console
installation was performed as part of publication.

**Operator-tested `2f5` archive (separate from the clean 0.7.0 package above).**
Installed/readback-verified 2026-10-22 (`usb-install-1022/VERIFICATION.json`):
NRO SHA-256 `2f5a1d4e4d99c58e0b351fd3af72e4f7e1cf121e017357d130a18fb2b151fc01`,
build id `0.6.2-d86fac2a7ade-dirty-1792624156` (binary still reports 0.6.2).
The operator confirmed on the Switch manual console **LLUltra** with **L+R+X**
advancing **LLUltra → LLUltra+ → LLUltra++ → LLUltra** (2026-10-22). That is
runtime evidence for the installed dirty candidate only, not for an unpublished
**0.7.0** release artifact.

**Host gates on the 0.7.0-versioned tree:** **246 unit + 1 SDK integration
tests passed**, warning-free host and Skyline Clippy, and the
no-default-features build. Logs recorded
under the evidence root `release-070-1022/` (`cargo-test.log`,
`cargo-clippy.log`, `cargo-build-no-default-features.log`,
`cargo-skyline-clippy.log`). `git diff --check` also passed.

## LLUltra in-place chord — 2026-10-21

Uncommitted source on main `d86fac2a7ade`. Manual console LLUltra is one profile
stop: Left or Right leaves it either direction and the saved resolution is
recalled on reentry, on console and emulator. While that manual console
selection is current, L+R+X cycles 576 → 720 → 1080 → 576 in place and leaves
the mode, Auto, and Vanilla+ unchanged. Auto, emulator LLUltra, and every other
profile keep the Vanilla+ toggle. Privacy ZR+X/Y, hold, and a face button
pressed before the shoulders stay gated as before. Labels, the config schema,
wire ABI, and rendering code are unchanged.

Host gates on this source: **246 unit tests + 1 SDK integration test passed**,
host Clippy `--all-targets -- -D warnings`, `build --no-default-features`,
Skyline Clippy `-- -- -D warnings`, edition-2021 rustfmt of the changed Rust
files, and the Skyline release build. Build
`0.6.2-d86fac2a7ade-dirty-1792624156`, NRO 1,441,792 bytes, SHA-256
`2f5a1d4e4d99c58e0b351fd3af72e4f7e1cf121e017357d130a18fb2b151fc01`.
`text_verified` true, `runtime_tested` false. Symbols, manifest, source
archive, and diff are outside the repository under
`llultra-cycle-1021/out/2f5a1d4e4d99c58e/`.

At packaging on 2026-10-21 this candidate was not deployed. It was subsequently
installed/readback-verified on 2026-10-22 at 10:22Z and the operator confirmed
the chord; see the 0.7.0 acceptance record above. The compact-menu candidate
below is the earlier package.

## Unreleased compact menu — 2026-10-21

Candidate based on 0.6.2/main `302c095`, uncommitted. Settings now has You/Opp
summaries and two editable controls. You carries font-independent eye/TV
geometry; open-window ZR+X/Y toggle Stealth/Streamer. The eye retains an amber
asterisk and reconnect notice when the connection latch differs. The match strip
uses the same icons. Console NetProfile folds existing resolution settings into
LLUltra / LLUltra+ / LLUltra++; entry recalls the saved resolution, Auto and
cleanup behavior remain, emulator has one Ultra stop. Native banner text shares
the label helper. Config keys/mode bytes, peer protocol and plugin ABI are unchanged.

Primary verification on the final source: **242 unit tests + 1 SDK integration
test passed**, host Clippy `--all-targets -- -D warnings`,
`build --no-default-features`, Skyline Clippy `-- -- -D warnings`, changed Rust
files' rustfmt check, and actual Skyline release build/package passed.
Tests cover privacy press/hold and hidden-window behavior, compact navigation,
resolution cycling/recall, config round trips, Auto/cleanup/emulator labels, and
unchanged peer resolution/boost bits. Independent Cursor Grok4.7 source review
accepted the diff; it did not independently run these tests.

Build `0.6.2-302c09574b3e-dirty-1792606778`, NRO 1,441,792 bytes, SHA-256
`d9e1edd1eca0dbe04e6f024405046d98cf92698c759819501ae21947fb3a670d`.
NRO/ELF executable text matching verified by the packager. Optimized symbols,
source archive, native-dependency provenance and manifest are outside the repo
under the evidence root's `compact-menu-1021/d9e1edd1eca0dbe0/`.

**Not deployed or runtime visually tested.** No new console compatibility or
stock-clock FPS claim.

Follow-up source change, same day: the privacy shortcuts now require the
Settings **You** row, which becomes the first selectable row (`> You` in the
table, `>You:` in the strip). ZR held with ZL released, each new X/Y press
toggles Stealth/Streamer once; a held button does not repeat or fire on entering
You, and other rows, pages and the hidden window have none. Candidate
`d9e1edd1eca0dbe0` above still has the any-open-page shortcuts. The corrected
source passes **243 unit tests + 1 SDK integration test**, host Clippy with
warnings denied, the no-default-features build, Skyline Clippy with warnings
denied, the Skyline release build, changed-source rustfmt and diff checks.
The new test explicitly covers entering/leaving You while X/Y remain held;
privacy gating is checked across every view and row. These are source/build
checks, not runtime visual acceptance.
Corrected build `0.6.2-302c09574b3e-dirty-1792617097`, NRO 1,441,792 bytes,
SHA-256 `5a2a5f05a996775aae4dcbd4d5804883c4d48c4b1358020443107b93df2f999f`,
is packaged under external `compact-player-row-1021/5a2a5f05a996775a/` with
matching NRO/ELF executable text. Its archived source differs from the previous
compact candidate only in menu selection, overlay gating and runtime UI.
[privacy-indicators.svg](privacy-indicators.svg) redraws the icon geometry from
source; it is not a screenshot. The pending visual/control checks are in
[NEXT_SESSION.md](NEXT_SESSION.md); the external `compact-player-row-1021/PLAN.md`
records the corrected candidate's exact operational state and Ally review launch.

**Ally Eden review and approval, 2026-10-21.** The `5a2a5f05` NRO booted in Eden
on the Ally (Smash 13.0.5, loading complete, title and Settings/Player Info
rendered); the review screenshot shows the You row with the eye and TV icons.
This is a bounded boot and visible-UI observation only: no control, FPS or
gameplay acceptance. The operator then approved it for release.

**Installed on Switch, 2026-10-21 21:45:07Z.** At the operator's request, so
it could be used right away, the same `5a2a5f05` NRO
(1,441,792 bytes, build `0.6.2-302c09574b3e-dirty-1792617097`) was installed
over DBI USB MTP. It replaced 0.6.2 SHA-256
`59dab7a8cd60d8d770d7a7c4f30bb668b174854d666e72086ed06e5c238768c4`; the
installed readback matches `5a2a5f05`. The other eleven plugins, the Deluxe and
FX `config.ini` files and `bootloader/hekate_ipl.ini` are byte-identical before
and after. Rollback on the card:
`/ultimate/ssbu_online_deluxe_rewrite/backups/pre-compact-5a2a5f05a996775a.nro`
(readback matches `59dab7a8`); the staged copy
`compact-5a2a5f05a996775a.stage` stays outside the plugin loader. Receipts are
in the evidence root's `ship-compact-menu-1021/usb-install-1021/`
(`deployment.json` state `installed_verified`). No console boot, control or FPS
result is claimed for this install. Version stays 0.6.2; no tag or release.

**Hint-text follow-up (source only).** After the install, the operator asked to
drop the redundant "You selected" prefix. `HINT_LINE` now reads
`ZR+Left/Right edits   ZR+X Stealth   ZR+Y Streamer   L+R+Down next page   L+R+X Vanilla+`.
The You-row gate is unchanged. The installed `5a2a5f05` binary still shows the
previous wording. The follow-up package and source/build receipts stay in external
`ship-compact-menu-1021/`; it is not copied to the Switch automatically.

## Approved content-fit HUD installed on Switch, 2026-10-10

- Operator approved the8328 screenshots, requested FTP installation, then freshly
  confirmed Smash closed. Foreground DBI FTP192.168.0.226:5000 worked.
- Fresh4c19 NRO, latest INI, all other plugin bytes and protected HOC/boot files
  backed up. Exact approved8328 staged under a non-NRO name and fully read back;
  old NRO moved outside the plugin loader, then staged candidate activated.
- Final installed readback SHA256
  `8328cc88123bd4320c550a95b5b3cc7c699110ab65c158b9f862975ec2f29534`
  verified14:09:35.905632Z. Remote old-NRO backup readback matches4c19. INI SHA
  `0617a94705ea8d2da3fff782161b5339f6a3880948a807894a01692bccd9c2d3`, all eight
  other plugins, five HOC/boot files and nine plugin filenames unchanged. FTP closed.
- Evidence: `~/.local/share/ssbu-mod-backups/hud-content-fit-20261010/switch-ftp/`,
  deployment.json installed_verified, before/ snapshot and readback/ binaries.
  Rollback on SD: `/ultimate/ssbu_online_deluxe_rewrite/backups/pre-hud-fit-8328cc88123bd432.nro`.
- No new hardware gameplay result yet. User will test the content-fit two-row HUD,
  menu coexistence/returns, export completion and actual peer departure/reconnect.
  The original pre-preview manifest is immutable; device-validation.json records
  deployment separately. No general launch-crash fix or performance gain claimed.

## Content-fit HUD candidate, 2026-10-10 — images reviewed

- Candidate `8328cc88123bd432`, build `0.1.0-2db70f90aaae-dirty-1791637336`,
  NRO SHA256 `8328cc88123bd4320c550a95b5b3cc7c699110ab65c158b9f862975ec2f29534`.
- User feedback accepted f1ce's two-row look and requested that its background
  stop at the content. The only behavior change is compact ImGui width0; previous
  positioning/adaptive-font bound, two rows, bar, alpha and other views remain.
  menu.rs changes are comments. No new native dependency or Cargo.lock change.
- 265hosttests, warnings-denied host/SkylineClippy, no-default and optimized target
  builds pass. Six fit-1010-* gate receipts are in slim-hud-preview-20261009/.
  All76archived sources match checkout; exact f1ce diff, NRO/ELF text and debug
  sections independently verified in the immutable8328 bundle.
- Parent opened original Settings `hud-content-fit-1010a-settings-inmatch-settled.png`
  (6:42.95) and Performance `hud-content-fit-1010a-performance-inmatch.png` (6:54.26).
  Actual offline Isabelle/Toon Link match,1920x1160 full-window captures/1080p output.
  Both have exactly two inline rows and the same left edge. Backgrounds end after
  Stream:Off / Out:1920x1080 respectively; no near-full-width empty rectangle.
- Earlier settings-final.png is Results and excluded. settings-inmatch.png has
  correct layout but still includes an emulator-pause interval in its180-sample
  timing history; after8s resumed play the settled capture replaces it. Other
  navigation/retry captures remain preserved and excluded from the final pair.
- Exact candidate/config/startup/module and screenshot SHA values verified.
  Config is console layout with FX Off/all, not a default-FX or performance test.
  The6GiB cap was reached with zero observed OOM kills. No connected-peer/720p or
  Switch-compatibility conclusion; operator visual feedback remains pending.
- Owned unit eden-control-58c1ff0dc02b.service exited normally0 after native
  game.stop/confirmation/ui.close; independently inactive/dead/MainPID0. Runtime,
  worker and finite gate/capture jobs closed; GPU lock available. Original NRO/INI/
  Qt files restored and independently rehashed against the fresh journal backups.
- Evidence: `~/.local/share/ssbu-mod-backups/hud-content-fit-20261010/`, including
  result.json, parent-review.json, profile-journal.json and run-evidence/.
  Scoped artifact runtime-validation.json supplements the immutable pre-preview
  manifest. Final parent-closeout.json records watcher cleanup. No Switch deployment.

## Slim-HUD screenshot candidate, 2026-10-09/10 — images reviewed

- Candidate `f1ce158107e826bf`, build `0.1.0-2db70f90aaae-dirty-1791588649`.
  NRO2428928bytes, SHA256
  `f1ce158107e826bf6ccff44f2e97abf94f145968e78224ca8521f7ee326840bb`.
- 265 host tests pass; host Clippy with warnings denied and no-default build pass.
  Parent repeated the host gates after review. TargetClippy first identified a
  redundant eighth argument; deriving FPS visibility from the existing Config
  fixed it without suppression. Final SkylineClippy/release pass.
- Parent also fixed remote-field lookup to use stable field identity rather than
  the selected row's decorated label. Only menu.rs/runtime/ui.rs differ from4c19.
  All76archived files match checkout; Cargo.lock/native provenance unchanged;
  NRO/ELF text matches and debug sections are present.
- Host-only runtime UI tests originally executed zero tests; this was corrected
  by testing portable format/fit helpers in menu.rs. Tests do not assert pixels.
- Raw baseline4c19 in-match images reproduce the tall grid. Baseline's owned unit needed forced-stop
  after teardown timeout; no OOM. Baseline screenshots are not f1ce evidence.
- Evidence and gate receipts:
  `~/.local/share/ssbu-mod-backups/slim-hud-preview-20261009/`.
  No Switch deployment of f1ce; no new disconnect/export/crash acceptance.

The G&W benchmark completed and released its reservation before f1ce preview launch
at00:15:26Z on October10. Exact SHA/build/module bound through staging/launch receipts
and install.log. Final Settings image at5:45.13 and Performance image at6:41.65 are
actual offline1v1 Isabelle/Greninja match scenes, full-window1920x1160 with reported
1920x1080 game output. Parent independently opened both originals: exactly two
inline HUD rows, segmented timing bar, no title/table. Near-full-width translucent
strip with text on the left. No connected-peer or720p image was collected.

Original PNGs, sidecars, result.json and parent-review.json:
`~/.local/share/ssbu-mod-backups/slim-hud-screenshots-20261009/`.
Separate2x detail crops are labelled derivatives; original screenshots are unchanged.
Countdown, wrong-view and Results captures are retained but excluded from the final
pair. Copied Switch config is FX Off/all; this run does not establish default FX On.
The6GiB cap was reached with zero OOM kills; screenshots are not performance evidence.
Owned Eden unit exited normally0, runtime/worker/watch resources closed, three
original profile files restored and independently rehashed. Operator visual feedback
remains pending. Original package manifest keeps its pre-preview runtime flag;
current scoped evidence is in runtime-validation.json, with prior checkpoint retained.

## Operator-reported multi-person Eden lobby test, 2026-10-09

The operator confirmed that multiple people joining the lobby in Eden had been
tested by the Training Modpack agent. This is attributed operator-reported
coverage; the message did not identify the candidate hash, exact participant
count or test receipt. Keep the earlier artifact-linked lobby/peer-cycle results
below as their own evidence. This confirmation does not establish that the newly
reported disconnect/declined-rematch stale-opponent case was exercised, nor does
it validate the current menu/peer-state candidate. No new emulator run was made
to record this update.

Development milestone, 2026-10-05:

- Optimized AArch64 Skyline NRO built.
- Host cargo clippy --all-targets -- -D warnings passed with zero warnings.
- Host cargo build --no-default-features passed.
- Host cargo test: 18 passed after review fixes, including pointer-walk regression coverage.
- Debug ELF has .debug_info, .debug_line and .symtab.
- Packager verified matching executable bytes.
- Known buffer/CSS hook addresses resolved to Rust source file/line.

Switch-target Clippy is unavailable in installed skyline-v3:
error: 'cargo-clippy' is not installed for the custom toolchain 'skyline-v3'.
Host Clippy excludes Switch-only code; the actual Skyline compiler checks those bodies.
No warning was emitted by that build.

## Eden startup investigation, 2026-10-05

- The original build aborted at startup. Runtime updates alone did not resolve it.
- Updated Eden and the user's mounted Switch SD to official Skyline beta (April 12,
  2026) and ARCropolis 4.1.1, including its resources, with verified backups.
  No rewrite NRO was installed on the Switch SD.
- Moved 21 historical .nro.bak files outside Eden's plugins tree after the new
  Skyline console log proved it was loading/executing them. No backup was deleted.
- Controlled baseline: seven existing plugins with the rewrite removed booted.
- Early synchronous phase logging isolated the abort after recorder initialization
  and before module bounds were returned by skyline::info::containing_plugin.
- Replaced that helper with module-local hidden linker symbols. User confirmed
  bundle 8efe12540cfc2b21 reaches the game. Logs contain the correct NRO module ID,
  all eight FX hooks, buffer installation, and background heartbeats.
- That boot exposed a separate menu-installation rejection: another mod had wrapped
  the text setter. Since we only call that setter, the hook-overwrite check was
  inappropriate. Added call-only validation; retained overwrite checks on hook sites.
- Corrected default-config creation: skyline-v3 reports missing native path 0x202
  as ErrorKind::Other. Confirm absence with GetEntryType before creating defaults.
- Candidate c5311b49dc66795c is installed in Eden. Its logs confirm default-config
  creation and successful CSS/FX/buffer hook installation. Menu activation and
  gameplay validation are still in progress.
- At actual character select, CSS entry/exit events are present but the title pane
  is rejected before input handling. Probe cb1eb92cebd70a50 adds sampled fields and
  reason codes without relaxing validation. The previous main-menu attempt did not
  exercise this callback. The user requested eventual menu access from all screens.
- At 17:27:52–17:28:30 UTC, a host-wide OOM caused a desktop logout. The kernel
  table showed eight concurrent Eden processes, several with 3–4 GiB resident, and
  exhausted 8 GiB swap. This is evidence of accumulated instances, not proof of a
  leak in one NRO. Logs/binary were preserved before temporarily quarantining it.
- Added tools/EdenTest.ps1: actual cgroup properties verified (12 GiB memory limit,
  1 GiB swap limit), and a second launch correctly refused the existing Eden PID.
  The same probe is restored for one guarded test; pane diagnosis remains pending.

Host Clippy, no-default-features build, 18 tests, and the actual Skyline build pass
after these source changes. This validates startup only, not full three-mod gameplay
compatibility, emulator performance or console behavior. Raw logs and install/update
receipts remain under target/, outside Git.

## First live matrix

Start with this mod alone, then Training Modpack plus SGPO; test both load orders.
Keep the exact build bundle.

1. Observer-only diagnostic flags: startup identity and log persistence.
2. On + Auto: baseline visuals and CSS/match/results/Training transitions.
3. Menu: each controller family, both rows, saving and restart.
4. Input buffer: Auto and explicit values in Quickplay/arena/local.
5. Each target move: Off suppression and newly spawned On restoration.
6. Shield/parry, smoke, KO, colors, projectiles and LAST_EFFECT consumers.
7. Training save-state/reset/menu and SGPO HUD in the same session.
8. Multi-fighter matches and repeated loads, separately console/emulator.
9. Compare frame times before claiming performance gains.

Diagnose failures from logs and addresses; do not add broad deletion or catch_unwind as a substitute.

## Global window acceptance, 2026-10-05

- User confirmed opening and FX Off in CSS build 24787e3d204edb6e, but its title
  text grew/clipped. Replaced the CSS pane path with the fork's top-left window.
- An intervening panic src\\lib.rs:243:75 matched installed libone_slot_eff.nro's
  fighter-name UTF-8 unwrap. Quarantining only that plugin let the next run reach CSS.
  Original NRO and logs are backed up; a permanent one-slot-effects fix is pending.
- Candidate 2ca7c6991f9a7565 embeds the pinned ImGui backend in one 1,683,456-byte
  NRO. Warning-free Skyline build, zero-warning host Clippy, no-default-features
  build and 22 host tests passed. The bundle includes matching ELF and full source.
- User confirmed the small window appears from the main menu, and FX switches Off.
  Code 26 confirms backend initialization; paired codes 21/22 confirm drawing.
- Fixed native RenameFile's refusal to overwrite config.ini using backup-and-replace
  with startup recovery. Live config.ini now contains effects=off; previous.ini has
  effects=on. A clean restart logged effects=Off, confirming persistence across process restart.
- Training Modpack updater is disabled in Eden; settings were backed up and runtime
  log confirms Disabled. Training Modpack and SGPO remain installed in these runs.

This does not establish all scene transitions, actual FX move behavior, full network
telemetry, Reduced FX, LessLag, console compatibility or performance gains.

User additionally confirmed the global window at character select and the normal
game title banner restored. Match/SGPO display test is pending.

## Accepted FX build and next candidate, 2026-10-05

Installed c1706bbe9589de97 (SHA256
c1706bbe9589de979460de2b3553c9bb188eca30b257355cb2d3aed452fc9225):
user confirmed the small global window in matches, Palutena/Sephiroth FX Off/On,
SGPO, Training Modpack menu open/close, and Training save/load preserving Off
with successful On restoration. Saved runtime evidence stays in that dist bundle.
No Switch or performance acceptance is implied.

The expanded candidate adds qualified actor/move tables, ten ACMD request variants,
both effect-name slots for flips, preserved projectile/hitbox cues, and online-only
INI scope (all for testing). Host tests: 32 passed; Clippy all-targets with
-D warnings and no-default-features build passed. Skyline build and exact final
bundle are recorded in the operator's evidence-root handoff (outside the repository). This candidate has no live acceptance yet.

Fable 5.1 xhigh reviewed context/stack ABIs and scope registration. Primary addressed
the review findings: preserve information-bearing cue names, guard install reads,
retain original Lua Int/Hash type, and refresh online state from delay setup even
when input-buffer changes are disabled. Main-menu reset remains unconditional.
Live Quickplay/arena/local-wireless transitions and unknown patched sites remain
validation items, as do the broad move catalog, both load orders and console tests.

Final clean-source bundle: dist/1087e7b01f288ef4, implementation commit
8b8eb34ac4fe224547254744c054f318c2bbcad1. NRO SHA256
1087e7b01f288ef4b29b7c0461109fc62c63ab107c59bc1f9e0a5a891ee02689.
Verified NRO/ELF executable match and retained .debug_info, .debug_line and .symtab.
The bundle includes the scope=all test INI. It has NOT replaced the accepted
c1706bbe Eden installation; no rewrite was deployed to Switch.

## Autonomous Eden continuation, 2026-10-05

The preceding no-install checkpoint was superseded by the user's instruction to
continue autonomously. Expanded bundle1087e7b01f288ef4 was installed and tested:
Sephiroth neutral-B and forward-smash suppression, On restoration, global window,
SGPO, and Training Modpack save/load/menu all worked in the exercised paths.
Off with scope=online retained offline Training visuals and emitted no suppression
events. Actual online transitions and the full move catalog remain untested.
Raw evidence is in that bundle's runtime-evidence directory.

Callback-capture bundlebf4df24b3b8cc176 is now installed in Eden, with On/all/Auto.
Five raw traces passed validation: idle, controlled emulator pause, debug shortcut,
and Training neutral-B with FX On and Off. Zero capture gaps, panic events or ACMD
stack-rejection events were recorded in the final run. An invalid UTF-16 request
was consumed without a repeated-error loop. See PERFORMANCE.md for exact clock
evidence and limits. These tests establish instrumentation operation, not a
performance benefit or LessLag behavior.

Host validation: 37 Rust tests and 4 Python tests passed; all-target host Clippy
with -D warnings and the no-default-features build passed. The actual optimized
Skyline release built without warnings and has matching NRO/ELF/source archives.
Target-only Rust is checked by the Skyline build and live tests, not host Clippy.
Eden was stopped after evidence collection. Original Training Modpack save states
were restored and their hash rechecked; no rewrite was installed on Switch.

## Render experiments and corrected charge coverage

Bundle 79790e610f0f24f8 exercised the opt-in LessLag core in Eden with Training
Modpack and SGPO. Readback confirmed three allocated/two active textures and
present/acquire indices 0/1. The global window, FX Off, and Training menu remained
operational. Core hooks executed; full LessLag sequencing and latency reduction
are not established. Doubles work was stopped at the operator's request.

The operator identified earlier 2.2-second neutral-B holds as too short to confirm
Gigaflare. Longer 4.2-second holds were visually confirmed, warmed, and used for
four On/Off/Off/On pilot trials without screenshot readback during the capture.
All callback/API traces completed with zero skipped records. No On-related dip
was reproduced. See PERFORMANCE.md for numbers, raw evidence paths and limits.

The bounded VI frame-history probe completed, but supplied no usable presented
timestamps in the tested Eden startup window. Temporary TAS and history settings
were restored. Capture files measure CPU observations, not displayed frames.

Final host review found and fixed escaped-newline test fixtures so invalid renderer
values and duplicate keys actually exercise their intended parser branches.
All-target Clippy with -D warnings and no-default-features build passed afterward.
The latest package and any subsequent runtime check are recorded in the operator's evidence-root handoff (outside the repository).

The saved original Switch stack was tested separately and restored afterward;
its Eden pre-match stall is documented in SWITCH_CRASH.md. That test did not load
the rewrite, TM or SGPO and did not reproduce the console's crash signature.

## Asymmetric local-wireless smoke test, 2026-10-06

Custom Eden commit10bcd2d849843b146a79c61a21df615e1bdcfcde ran two isolated
13.0.5 instances on one machine. Each had an8GiB memory limit and no swap.
A loaded rewrite d5b4464b508a98244056fa9b09b7a0b5c51f374313afee45177a6a307b70427c,
the corrected gamepad overlay20e9e062..., Training Modpack7.1.1 and ARCropolis4.1.1.
B had its entire title override directory and ultimate directory moved outside
the profile, so it ran without Skyline plugins or modded title assets.

A7-minute Palutena-versus-Sephiroth match on Small Battlefield completed through
sudden death and results. Both clients reported Sephiroth winning and the same
damage-given totals: Palutena6%, Sephiroth30%. Earlier paired screenshots showed
Palutena at30.2% after Shadow Flare on both clients. A returned to character
select; B returned to stage select.

FX changed from On to Off during the match on A only, then back On at results.
Paired screenshots at match clock5:13.86/5:13.85 show Gigaflare's large explosion
on cleanB and suppression on A. Palutena sideB also hit during Off; damage and
stock results remained consistent. Numeric RTT appeared15–27ms. This artifact
displayed the latest sample without the later mean/stability-color work.

No divergence or disconnection was observed during the match. This is a bounded
visual/state smoke test, not a simulation-checksum comparison, Nintendo-online
test, Switch test, or proof about every move. Render mode wasVanilla and input
bufferAuto; it does not validate LessLag/LLUltra multiplayer transitions.

Evidence and exact plugin hashes are outside Git:
~/.local/share/ssbu-mod-backups/desync-pia-20261006/
(result.json, a-plugins.json, run-a, run-b, runtime-final, screenshots).
Memory peaks were7,132,610,560bytes(A) and6,433,144,832bytes(B), with no OOM events.
B exited normally. A's40-minute watchdog expired after the results transition;
the planned stale-RTT screenshot following B's departure was not captured.
The staging journal is restored, including verified original A file hashes.
## ImGui host coexistence, 2026-10-06

NRO defda6fdf99e01e08906f4ae7e8f7df7cfab0d4fa1eb42b478e9ad6499146135
passed three bounded Eden runs: embedded host alone, standalone host loaded
before the rewrite, and standalone loaded after it. Both external runs displayed
Deluxe and Elite Legal Stages together. The reversed-order and embedded runs also
persisted an FX On-to-Off change. NVN present/acquire observations advanced in
all three runs.

The two external runs installed distinct bootstrap replacement addresses,
confirming that the shared hook-name collision was removed. Before the fix, both
plugins resolved the same exported callback; one plugin's implementation never
ran. Every rewrite hook now has a unique prefix, internal function references
bind locally, and bootstrap installation checks the replacement's module.

These checks cover boot, menus, toggling and NVN observation in Eden. They do not
establish native Switch compatibility or actual Elite stage filtering. The
standalone build was ImGui 1.92.7 WIP (19263), SHA-256
61e19e1b593826b1216228c6536cf2603b512154a8fecc548203e245bb86ed3e.
Unknown host ABIs are refused with diagnostics.

All three emulators exited normally and the isolated profile was restored.
Evidence: ~/.local/share/ssbu-mod-backups/imgui-unique-20261006/runtime/result.json,
with screenshots and complete run logs in each case directory.
## LLUltra and asymmetric G&W rendering, 2026-10-06

Two further pairs used the same custom Eden build and 13.0.5 game. A loaded
rewrite 9a0e76de16a7fe7af12da7fd47f6d6e38c3b971d242c0e561b60f0b31d805347,
corrected SGPO, Training Modpack and ARCropolis, with LLUltra/Auto buffer/FX On.
B had no title overrides or plugins. Both used private localhost rooms, 8 GiB
per process, no swap, and finite watchdogs.

The control pair (A 20261006-164547-429778, B 20261006-164547-836af6) had
no G&W plugin on either peer. A seven-minute Mr. Game & Watch versus Sephiroth
match on Small Battlefield completed with Sephiroth winning on both clients.
Settled Sephiroth damage was 16.6% on both; result damage-given totals were
G&W 16%, Sephiroth 15%. Green averaged RTT of 18–20 ms was visible. After B
disconnected, A displayed an uncolored "---". Both exited normally. Return to
character select was not visually confirmed in this control.

The asymmetric pair (A 20261006-172218-3d4587, B 20261006-172218-1b98c6)
added only A's libgnw_render_probe.nro, SHA-256
0883e69785e90b0e6324a803821e82a8cf4df529e6200c25c1e9658069620ade,
from source 4a4f63ad32efdc6e2c5395563bcbc0361296f75c. Demand-readback was
explicitly disabled. Logs confirmed the expected shader fingerprint,
verified=true, and RANGE slot=2 reductions with restored=true. LLUltra counters
advanced with zero bad callers and zero alias mismatches.

The same seven-minute matchup completed with Sephiroth winning on both clients.
Paired screenshots showed 14.6% Sephiroth damage mid-match and matching final
damage-given totals of G&W 29%, Sephiroth 0%. Both returned to stage selection
and then character select. No divergence or crash was observed in this match.
The scripted inputs exercised several moves, but not every requested animation
was captured. Regular-match KO/respawn, all costumes, role reversal, both peers
modded, and simulation checksums remain unverified. This is bounded coexistence
evidence, not an online-safe certification or a G&W performance measurement.

## Controlled latency relay, 2026-10-06

After the asymmetric match, B reconnected through a temporary localhost UDP
relay. It delayed each direction by 50 ms without editing packets. An independent
echo check measured 100.43–100.75 ms round trip with unchanged payloads.

Joining Smash with that delay already present failed before the host's
confirmation appeared. Joining through the same relay with zero delay succeeded.
Adding 50 ms per direction after the join produced about 123 ms in-game RTT,
with yellow and red stability indications captured during CSS/warm-up. No
deliberate jitter or packet loss was injected. Changing delay and host scheduling
can both affect variance; these screenshots are not a controlled test of each
color threshold.

The relay's 1000-second deadline expired during warm-up, producing a room-lost
dialog. No high-latency match completed. This was not a VPN route and does not
establish high-latency desync compatibility. Both Eden processes subsequently
exited normally, with no OOM events; memory peaks were 7,547,342,848 bytes (A)
and 6,820,466,688 bytes (B).

All test plugins/config files were restored from the staging journals, and B's
direct-connect port was restored to 24872. Main user profiles and Switch files
were not changed. Evidence:
~/.local/share/ssbu-mod-backups/desync-rtt-colors-20261006/
(gnw-control and gnw-asymmetric, including localhost-phase, full run logs,
screenshots, renderer counters, relay timing/configuration, and result receipts).

## Reviewed integration artifact, 2026-10-06

NRO `c924d9d1b2ed5b56bb7ec14b847b1b9d1ed627ecc2394be77144723fd0b101e0`
(2,203,648 bytes), ELF
`6cb922b0ec394ebbed1f2968ade1c61a9b82bf308ceca604d4f127d87d7ecbf6`.
Bundle `dist/c924d9d1b2ed5b56` retains matching executable bytes, optimized
debug symbols, full source, native provenance and raw check logs. Host Clippy
with denied warnings, no-default-features build, 89 tests, Skyline Clippy and
Skyline release build passed. All 38 Skyline hook exports have the unique
Deluxe prefix, with no preemptible callback relocations; required NN/NVN/Skyline
imports remain dynamic.

This adds the default-off peer diagnostic observer. Review tightened packet
bounds, coherent queue/map publication and lifecycle invalidation. It refuses
installation without the Pia lifecycle hooks. Missed lifecycle callbacks or
adoption of a new protocol conservatively clear diagnostic identities. It does
not send packets or populate the missing peer metadata UI. Its enabled runtime
path has not been tested yet.

Exact-artifact smoke run `20261006-184006-bd1296` used isolated Eden, LLUltra,
corrected SGPO, Training Modpack and the embedded host, with `peer_observer=false`.
It reached the main menu and displayed Player Info, with no opponent shown.
LLUltra present/preflush counters advanced, three allocated/two active textures,
zero bad callers and zero alias mismatches. It exited normally after 131 seconds,
peak memory 5,969,846,272 bytes, no OOM. This is startup/menu evidence, not a new
match or network test. Evidence and restored-profile hash receipt:
`~/.local/share/ssbu-mod-backups/observer-final-smoke-20261006/`.

## Live profiles and both asymmetric G&W modes, 2026-10-07

Tested NRO `ed5d12f84772149fcf86ea43ffa18efd657f6d27edefde3034eba8e647312c6e`,
ELF `5da2c5cd5ce801959faba5f9c04dbc9dabcf8dbc65e5c49e15eab1d6db32530d`.
The same candidate ran both tests below. Host Clippy with denied warnings,
no-default-features build, 110 host tests, Skyline Clippy and release build passed.

A `20261006-210611-b11161` had the rewrite, G&W fix `0883e69785e90b0e…`,
corrected SGPO `20e9e062c9ace623…`, Training Modpack and ARCropolis. B
`20261006-210611-b70a45` had no title overrides/plugins. Custom Eden was
`10bcd2d849843b146a79c61a21df615e1bdcfcde`, Smash 13.0.5, private localhost
room, Auto input buffer, FX On/all. Neither peer used VPN. A's diagnostic peer
observer was enabled and only observed packets. Demand-readback was disabled.

Both were three-stock, seven-minute Mr. Game & Watch versus Sephiroth matches.
The scripts exercised attacks, projectiles, jumps and KO/respawn; B then walked
off to finish the matches. They are not seven minutes of uninterrupted combat.

| Mode on A | Stage | Matching live damage, A/B | Matching result damage given, A/B | Outcome |
|---|---|---|---|---|
| LessLag | Kongo Jungle 64, Battlefield form | 5.6% / 26.4% | 26% / 5% | G&W wins on both; both return to CSS |
| LLUltra | Battlefield | 5.6% / 20.1% | 20% / 5% | G&W wins on both; both return to CSS |

G&W shader fingerprint `3869d88ad69eee6a` was verified, with RANGE slot 2
reductions and restoration logged. Rewrite Ultra guards recorded no bad callers
or alias mismatches. No desync was observed in either match. This establishes
bounded one-sided coexistence, not simulation-checksum equality, every move or
costume, Switch behavior, VPN/high-latency compatibility, or a performance gain.

The same session also switched Vanilla/LessLag/LLUltra in the main menu, at CSS
(nine successive changes), and during a third linked match (two complete cycles).
All selections reached Active. The match trace `2711481731398` captured 592
presents using indices 0/1/2, all with zero outstanding acquisitions after present,
and no skipped trace events. The CSS trace `2698991455799` recorded another 568
presents with zero outstanding acquisitions, but did not span all CSS changes.
Across the session, ten buffer changes committed; no code-47 refusal, code-14
panic or stalled-drain warning occurred. The third match was stopped after the
transition checks; it is not another complete-match acceptance.

The legacy trace `engine` and status `mode` fields describe the startup mode.
For live changes use `policy_committed`, `policy_effective`, the fresh active
texture count and transition events. Raw index-selection markers are before
the game's modulo-three instruction and can exceed two in Vanilla.

Both frontends exited normally, code 0, with no OOM events. Peak memory was
7,083,679,744 bytes for A and 6,448,177,152 for B, under 8/7 GiB limits and a
45-minute deadline. All staged files were restored with hash verification.
Switch and main user Eden files were untouched. Evidence, screenshots, mode
snapshots, trace summaries and restoration journal:
`~/.local/share/ssbu-mod-backups/peer-menu-gnw-e2-final-20261007/`.

INI persistence was separately exercised on preceding E1 candidate `bcb8…`:
LLUltra and the Stealth preference survived a full restart. Current host tests
cover the same save/recovery path. Stealth still displays Unavailable because
the transmit implementation is incomplete; saved preference is not proof of
network behavior.

## Stealth identity blocker and separate emulator crash

The read-only observer populated peer slot 1, but its one-shot map event was
dropped from the four-entry diagnostic queue. Subsequent sampled probes proved
the slot remained populated. Its received source ID still differs from the
transmit destination ID. Grok excluded a byte-order explanation; Fable's second
opinion found no proven conversion and recommends station-field and explicit
header-presence probes, or the fork manager's connection-ID method. No guessed
mapping or packet writes were added. Full Stealth and peer metadata remain open.

A prior diagnostic run `20261006-201956-a94e26` crashed in Eden's host GPU code
before fighter selection. Symbolization located `Tegra::DmaPusher::CallMethod`
accessing an unbound subchannel. The cause of that missing engine binding is
unresolved; it is not established as a G&W or rewrite fault. The current two
accepted matches did not reproduce it. No emulator workaround was shipped.

## Combined profiles and resolution candidate — 2026-10-07

NRO `6c6a64afca4073c91128d938141e143c8519ead41e530a26ca56f1c46bc02409`,
optimized ELF `74e484062f276da577b6d3900eba5bf05fa34a3233d84a47e1c16676b45b7c8a`.
169 host tests, host/Skyline Clippy with warnings denied, no-default-features
build and optimized Skyline build passed. Raw checks and source are in its
ignored debug bundle.

The earlier Stealth identity blocker described above is historical. The current
implementation uses the fork's validated connection-ID path; it does not guess
a conversion from receive headers. Prior candidate `410ac1f7` exchanged names,
interfaces and active profile settings with the original fork in a local-wireless
lobby. Its pending-On display was observed. That original-fork stack aborted
while loading a fighter, so this was not a complete match or final-candidate
fresh-Stealth-On acceptance.

### Fighter callback and Training coexistence

Count-only run `20261007-013838-0fd514` used Vanilla/emulator, standalone ImGui,
Elite, TM and corrected SGPO. Two five-second idle Training samples, separated
by returning to CSS and loading another Training session, counted 307 Main ticks
per fighter over 302 presents and 306 over 301. Both passed the predeclared
0.98–1.02 approximate-frequency band, with no rejected reads or slow ticks.
This is not exact per-frame uniqueness or module-unload proof. Count mode
installed no cut-in hook or resolution reports. The TM menu opened/closed and
SGPO remained visible. A preceding navigation attempt reached no accepted sample
and is retained separately.

### Manual profiles, Auto and Gigaflare

Run `20261007-014825-27814e` deliberately used `platform=console` in Eden to
exercise the console resolution table. Sephiroth charged neutral B for 4.5 seconds.

| Selected profile | Applied base | Gigaflare | Recovery |
| --- | --- | --- | --- |
| LLUltra | 576p / DRS on | 480p, crop 854×480 | 576p, crop 1024×576 |
| LessLag | 1080p / DRS on | 720p, crop 1280×720 | 1080p, crop 1920×1080 |
| Vanilla | 1080p / DRS off | No resolution report | Remained 1080p |
| LLDoubles | 1080p / DRS on | 720p | 1080p |

All four manual modes reached Active. Auto in offline Training selected Vanilla
while retaining the saved manual LLDoubles choice. The Doubles transition is
proven by events 749–756; periodic status publication missed that short interval.
Reading a status file every 50 ms does not make its roughly five-second
publication interval a 50 ms timing measurement.

FX Off suppressed the named Gigaflare request (`0x1210319015`, event 1355), and
the resolution report recovered afterward. The actual cut-in trigger was not
exercised. Nonfighter-category rejects during weapon articles are expected
filtering, not evidence of failed fighter reads.

Both Deluxe and Elite menus worked through the standalone ImGui host. Elite's
enabled setting persisted; offline Training continued. Both menus consume D-pad
edits when simultaneously open, so edit one at a time. This does not validate
Elite's NSO-only stage filter.

A manual console-LLUltra boot showed an undersized title image after the rewrite
had applied its crop. Later CSS/match images filled the window. Fable reviewed
the timeline; a later external crop reset is suspected, but its caller remains
unproven. The Auto/Vanilla menu path is unaffected. An observation-only diagnostic
is being prepared; no crop-behavior fix is claimed.

Both accepted single-console runs exited normally without OOM and their staged
files were restored. Raw screenshots, events, samples, acceptance records and
run journals are preserved under:

```text
dist/6c6a64afca4073c9/runtime-evidence/count-only/
dist/6c6a64afca4073c9/runtime-evidence/resolution-console/
```

### Four-player attempt, no match acceptance

A `20261007-020912-95094a` used this candidate, GNW fix `0883e697…`, SGPO
`20e9e062…`, TM, ARCropolis, Elite and standalone ImGui; B
`20261007-020912-f7b003` was a clean peer. Both joined a private local room.
Two local players joined A, but B's second local player did not join. The attempt
ended at Team Battle character select with three players. No match started;
this is not a doubles or desync result. Both exited normally without OOM,
peak 6,550,102,016 / 6,587,686,912 bytes, under 8/7 GiB caps. The staging journal
was restored exactly. No VPN or artificial network delay was used.
## Final b19 crop, response and cut-in checks — 2026-10-07

Artifact `b19b7d94ae136e55d2fe59136dd9ff2c40ce9436393a59984660ed3ab8c52cd7`
has matching optimized ELF `3f5e8407be2ef8f38037df645c69f32759d617ec6f7517fac4432de0fc18e397`.
All 175 host tests, host/Skyline Clippy with warnings denied, the no-default-features
build and optimized Skyline build passed. Source/build inputs were compared with
the packaged source archive before runtime testing.

An observation build proved the game reset a previously applied 1024×576 crop
to 1920×1080. The final fix forwards the original call unchanged and rearms only
an exact fullscreen reset on the tracked window with an accepted smaller crop.
The next tracked present reapplies it. Run `20261007-025442-132e12` showed events
177/178/179 in that order and a correctly filled title image, then full Gigaflare
576→480→576. It ended at its 600-second runtime limit, not a spontaneous crash;
no OOM kill, peak 8 GiB. The profile was restored.

Run `20261007-031616-b75dd6` collected 24 response trials on the same final NRO.
See [LATENCY_METHODOLOGY.md](LATENCY_METHODOLOGY.md#final-candidate-24-trials-2026-10-07)
for exact counts, image/trace validation, the retained idle-animation threshold
false positive and capture-copy race. The earlier preflight was stopped normally
to configure 360-pixel screenshots before emulation; it supplied no trials.

After the response trials, a second Training session used Luigi against Roy.
Close-range Up-B produced the visible critical-hit cut-in. Events 1193–1200 show
source-1 report start, applied 720/crop1280×720, report end and restored
1080/crop1920×1080 while manual LLDoubles was active. Screenshots preserve both
the cut-in and subsequent play. This verifies the actual trigger and recovery,
not a Switch performance improvement or every cut-in variation.

The solo run exited normally after 827.6 seconds, peak 8.55 GB within its 9 GiB
limit, with zero memory-limit reclaim events and zero OOM events. Its isolated
profile was restored. Evidence lives outside Git under:

```text
dist/b19b7d94ae136e55/runtime-evidence/crop-reset/
dist/b19b7d94ae136e55/runtime-evidence/response-and-cutin/
```

## Linked doubles and peer regression — 2026-10-07

The b19 pair `20261007-033126-622b58` / `20261007-033126-30b963`
completed a four-fighter team match on Moray Towers (Battlefield style).
Three local controllers joined A and one joined B. A carried the rewrite,
G&W fix `0883e697…`, corrected SGPO `20e9e062…`, TM, ARCropolis, Elite and
standalone ImGui; B had no Skyline/title mods. G&W and Sephiroth faced Wolf
and Byleth. Auto committed LLDoubles with three textures and DRS enabled.
Both sides agreed on observed hits, stocks and winning team, then returned
through results to character select. Full Gigaflare on B triggered A's
1080→720→1080 scaler. Both exited normally with no OOM.

The API trace `3123935492968` was captured during **Vanilla warmup**, not the
actual doubles match. Its 491 presents, three indices, TwoBehind relation and
zero outstanding acquisitions therefore do not prove the doubles pipeline.
Doubles acceptance uses the actual match's committed status, events and images.

The b19 pair `20261007-040044-b3b852` / `20261007-040044-1a740a` used
the rewrite on both peers, external ImGui on A and embedded ImGui on B. Names,
colored RTT, announced interface/profile, Streamer menu/chord and saved Stealth
On pending reconnect passed. G&W with the fix on A/LLUltra versus Sephiroth on
B/LessLag completed through matching winner/results and stage select. A reached
its 1800-second runtime limit while loading the next CSS; no final CSS acceptance
is claimed for this run. B exited normally; neither hit OOM. An earlier incidental
Ice Climbers match in the same session is not a G&W test.

```text
dist/b19b7d94ae136e55/runtime-evidence/four-fighter/
dist/b19b7d94ae136e55/runtime-evidence/two-rewrite-peer/
```

## Final display fix and Stealth/persistence acceptance — 2026-10-07

| Artifact | SHA-256 |
| --- | --- |
| NRO | `d9acd03611e1d1d644e5c1d56f0c80748e082cd46cd02f75ca3e62c3bffd838b` |
| Optimized ELF | `93da4bf258eb6a22f796ad9929fae4dcfef06cd45df7662e117505bf2ce6a25e` |
| Source archive | `6d7035650b1e285a645bdff6f1e9a88ed0cc690ba3d42eb0d197128b0433db2a` |

Build `0.1.0-03cad922da77-dirty-1791362447` passed 176 host tests, host
Clippy with warnings denied, the no-default-features build, the Skyline check
and optimized release packaging. The installed cargo-skyline 3.5.0 wrapper
requires `cargo skyline clippy -- -- -D warnings`; the single-separator attempt
rejected `-D` before checking. Raw final host/check logs are retained in the bundle.
Root compared all 55 packaged Rust/build input files with disk and reviewed
exactly four changed files versus b19: remote-buffer display/snapshot handling,
compact ASCII Auto labels and tests. No renderer/input/packet-writing change.
The 24 response trials remain measurements of b19, not this exact binary.

Two private localhost stations ran the final NRO. A had the complete mod stack
and compatible standalone ImGui; B had only the rewrite and its embedded host.
The first pair (`044519-4b92e6` / `044519-dff491`) loaded the exact INI files
saved through b19's menu, verified by SHA equality. A restored Ultra, buffer 2,
Stealth On and Streamer On; B restored LessLag, Auto buffer, Stealth On and
Streamer Off. Streamer hid the remote fields and its chord restored them.

Both learned/confirmed one native station. With fresh Stealth On, each logged
one suppressed beacon, **zero beacon writes and zero custom payload writes**.
Native name/RTT remained visible while unannounced interface/settings stayed
unknown. Saving Off showed `Off (reconnect)` and did not enable writes within
that connection. Both exited normally; complete logs/configs were archived
before restart, with no external INI edits between sessions.

The second pair (`045148-2c9fb4` / `045148-6e2cc5`) loaded saved Off and
exchanged custom metadata. At the first checkpoint A/B had one beacon each and
85/83 payload writes, with zero Stealth suppression. B displayed A's buffer 2;
A displayed B's Auto. Names, Ethernet announcements and green 17–20 ms RTT
were visible. The Ethernet value is the emulator's virtual nifm result.
No ambiguous-identity skips occurred. A few bounded lock-contention skips were
recorded; these are safe rejected attempts, not a claim of zero contention.

Elite's menu was opened with Deluxe and enabled before a regular linked match.
G&W on A used LessLag and the one-sided render fix; Sephiroth on B used Auto,
which selected LLUltra at match start. The G&W log confirms full shader/constants
verification and narrowed ranges restored after use. On Saffron City, both peers
showed Sephiroth at 12.6% after G&W's attacks and agreed on subsequent stocks,
G&W's victory, results and return to CSS. Cleanup committed Vanilla on both,
with the saved manual choice retained. The corrected controller overlay remained
visible on A. This was ordinary Smash; TM's save/load behavior was tested earlier.

All four d9 frontends exited normally with zero memory-limit and OOM events.
Fresh-On peaks were 6.08/5.24 GB; final match peaks were 6.21/5.92 GB under
8/7 GiB caps. All staging files were restored with original hashes.

```text
dist/d9acd03611e1d1d6/checks/
dist/d9acd03611e1d1d6/runtime-evidence/peer-and-gnw/
```

The bundle includes raw runs, checkpoints, screenshots, configuration receipts,
acceptance notes and a SHA manifest. Runtime folders can contain older captures;
attribute them by build ID and run window. The original packaging manifest's
`runtime_tested=false` is preserved as build-time metadata; the separate runtime
acceptance record supplies subsequent evidence.

These tests are bounded observations, not simulation checksums or universal
online-safety proof. No VPN/high-RTT, NSO filtering, four-network-station or new
Switch-rendering acceptance follows. Fable's final follow-up review was blocked
by the provider's model-access rejection; this final display review was performed
by root after Grok's implementation, without claiming a new Fable sign-off.


## Fork native display restoration, 2026-10-07

The fork's CSS title and arena room-ID code paths now accompany the existing
global Player Info window. Provenance and precise differences are in
[NATIVE_DISPLAY.md](NATIVE_DISPLAY.md). The window remains the editor.

Final artifact:

```text
NRO: 4a927dc20904722dc903afcb5c8548d80a5afffb3aeedd68958c76c3181c0518
ELF: 54a87dd43f7aaa3eb743dc73b20a6bc3cd61550549aae6c39c7c8acdc5dbe103
Bundle: dist/4a927dc20904722d/
```

192 host tests, host Clippy with warnings denied, no-default-features build,
Skyline Clippy with warnings denied and optimized package build passed. All 57
packaged Rust/build inputs match disk. The NRO and ELF hashes and matching
executable bytes were verified. One intermediate Skyline Clippy failure
(too_many_arguments) was corrected before packaging; its raw log is retained.

| Candidate / run | Observed result |
| --- | --- |
| `84d48448ac91e65d`, `20261007-101737-33eb02` | CSS window plus header; LessLag/buffer 0 edits; status persists with window hidden; Streamer restores original font/background; CSS re-entry; Olimar/Fox match through results and return to CSS; normal exit; 6,148,984,832-byte peak, no OOM |
| `4a927dc20904722d`, `20261007-103320-77c12b` + `20261007-103321-17d19e` | Linked native names/profile/green RTT in both directions; B embedded window visible; A standalone Deluxe and Elite windows invisible despite callbacks/input edits; no match started; below full speed before CSS, no OOM |
| `4a927dc20904722d`, `20261007-105849-510a19` | Solo standalone Deluxe and Elite windows plus native title, approximately 60 FPS; normal exit; 6,733,324,288-byte peak, no OOM |
| `4a927dc20904722d`, `20261007-110542-4550ab` | Same solo coexistence with G&W loaded; configured 600-second runtime limit stopped the frontend; 7,787,769,856-byte peak, no OOM |

The final build differs from 84 by the arena font/vertical-position correction
and deriving the diagnostic site inside the painter. No rendering-profile
behavior changed. Final solo screenshots show Elite Off; they prove menu
coexistence, not enabled NSO filtering. The earlier d9 enabled-Elite gameplay
result remains separately attributed to d9.

The paired external-window anomaly is unresolved. A root source/hook audit
found all three expected bootstrap registrations and external ImGui selection;
it did not establish which component caused the failure. Solo runs with and
without G&W both succeeded. Do not infer either a G&W fault or a fix from these
results. A future identical-pair comparison with `DISABLE_NATIVE` can isolate
the new native display without changing binary versions.

In the paired shutdown, A's frontend receipt reports exit 0 although its
supervisor tool returned 143. B remained in a game-stop confirmation flow;
after a 15-second helper timeout, the owned B service was explicitly stopped.
The G&W solo run ended at its configured runtime deadline before the manual
stop helper, which received ConnectionRefusedError. These stops are retained
as operational failures/limits, not reported as clean exits or match crashes.

All staging journals are restored with original hashes checked. No normal
Eden or Switch deployment occurred. Raw evidence is preserved outside Git:

```text
dist/4a927dc20904722d/runtime-evidence/native-display/
dist/4a927dc20904722d/checks/
dist/4a927dc20904722d/review/
dist/4a927dc20904722d/native-evidence-manifest.json
dist/84d48448ac91e65d/runtime-evidence/native-display/
dist/fps-investigation-20261007/
```

NSO arena, new Switch behavior and the concurrent external-window anomaly
remain unverified. No new latency, desync or FPS++ benefit is claimed by this
UI change. The FPS++ read-only trace is documented separately in
[FPS_BOOST_INVESTIGATION.md](FPS_BOOST_INVESTIGATION.md).

## Connected-only opponent display — 2026-10-07

Candidate `9e9ce2ab036d3e7ce06b74bc3ff770c03c569b6583af038c93f1f5972878ff69`
passed 230 host tests, host Clippy with warnings denied, the no-default-features
build, Skyline Clippy and an optimized Skyline package. Main-menu screenshots
showed the local-only table on both the standalone full-stack peer and the
embedded-backend peer. Joining an identified station displayed the remote
column; failed admission/disconnect cleared it again. Unknown RTT does not
hide an otherwise connected station.

A third independent station on the Surface joined the same private LAN room.
The host selected B and then C, and each selection displayed that station's own
name/profile/interface/RTT. Native local-wireless D-pad editing changed the host
buffer from Auto to 0. All three fighters were selected and ready. The host's
start chord was not sent before the finite run deadlines, so **no three-station
match or desync acceptance is claimed**.

The host used Mr. Game & Watch and the corrected G&W plugin `bceef1f0`; B used
Sheik without it, C used Banjo & Kazooie without it. Selected policies were
LLDoubles, LessLag and Vanilla respectively. These describe lobby configuration,
not proven match behavior.

Local runs `20261007-184606-b9a2f2` and `20261007-184607-a19c6b` ended at their
2400-second runtime bounds, with no OOM events and peaks 5,931,536,384 and
6,097,330,176 bytes. Remote C's 1800-second bound ended its run; its emulator and
supervisor were verified absent. All staged profiles were restored. This used
the wired LAN; neither NSO nor a VPN/high-RTT path was tested.

~~~text
~/.local/share/ssbu-mod-backups/combined-peer-native-9e9-20261007/
/tmp/delay-audit-01a10c4e-93fe-7db0-9d89-4bcd50bf2340/peer-native-9e9/
/tmp/deluxe-surface-1007/peer1/archive-0b2d5643/  (Surface)
~~~

## HUD placement candidate — 2026-10-07

Candidate `a9c38567b4e368d2d7723d760a2b7852503cde3688ba419e221e1bf03a794dcd`
adds right-side placement for validated Training and online lobby scenes.
Settings, performance and debug views share the rule. Regular singles/doubles
and the main menu stay left; native game banners are not relocated. The
position uses the current viewport and configured outer width with a 16-pixel
right margin.

234 host tests, host Clippy with warnings denied, no-default-features build,
Skyline Clippy and optimized Skyline packaging passed. Runtime placement
acceptance is pending; build checks alone do not prove screen placement.

## Reduced visual checks so far — 2026-10-07

Candidate `829727230ef34139` passed a controlled Gigaflare On/Off/Reduced visual
comparison: Reduced removed the first large flash while retaining the later
purple blast. Training save-state repeats retained the observed 49.3% damage
and 8 hits. This is a visual/damage observation, not a measured speed gain.

Candidate `3335efb82e7633ad` passed Palutena up-air On/Reduced/Off/On captures:
Reduced retained the blue wing cue while removing the bright rays/glare; On
restored the full effect. The up-air parent is deliberately protected in Off,
so its full appearance in Off is expected. Variant-load/use events 92/93
corroborated the Reduced path. The attack whiffed; no damage or performance
comparison is claimed for this trial. Other families still need runtime checks.

### a9c Training placement — observed 2026-10-08 00:02 UTC

Root visually reviewed the GNW worker's `training-hud-right.png` under
`/tmp/gnw-four-training-1007/`. Player Info is anchored at the right of the game
viewport in Training, with only local settings and no opponent column. Four G&Ws
are visible. This accepts Training placement for that binary; an online lobby
and an ordinary-match left-side regression are still separate checks.

The screenshot is not a performance sample: the menu is open and P1 is in an
action pose. The absent/enabled/absent idle benchmark was still running.

## G&W idle performance sanity check

[Three-window stock G&W check](GNW_FOUR_FIGHTER_CHECK.md): the enabled NRO executed its verified hooks and had steadier sampled FPS on Intel/ANV; all medians were near 60. This does not reproduce the original RTX bottleneck or prove a strict 60 FPS lock.

## 833c Explosive Flame measurements

[Thirty measured trials plus six warmups](https://git.syntaxbyexample.com/intpa/ssbu-fx-control/src/branch/main/docs/FX_EMULATOR_RESULTS.md) completed across
LessLag/LLUltra and On/Off/Reduced at 1×. No repeatable On-over-Off cost was
established; Reduced's performance benefit remains unverified. Raw evidence is
preserved with hashes outside Git. Paused screenshots are excluded from timing.

## 2026-10-08: 24e candidate and longer stress session

- NRO SHA-256: `24e714e77b3eabf0af188aec1a115f9f058b61beb7cc9119a29466ca2c26fc01`.
- Optimized ELF SHA-256: `22795134b469c7c8d84dc767ebf9bba4e67236fca0284ba3188b2ffeb837bc3e`.
- 242 host tests pass with eight threads; 20 further complete parallel runs pass.
  A process-global peer-table test race was fixed using a shared test-only lock,
  including readers that could contend with the nonblocking path under test.
- Host all-target Clippy with warnings denied, no-default build, Skyline Clippy
  with warnings denied, and optimized Skyline packaging pass.
- Full-stack Eden boot at 2x host rendering resolution passes. Offline window
  has no opponent column. FPS++ saved On displays `On (Unsupported)` and reports
  applied=0; restored Off afterward. A first status read at 1.5 seconds was stale
  and rejected; the later fresh report agrees with the visible label.
- Run `20261007-235700-988c0a` uses a two-hour deadline and 10 GiB/no-swap limit
  with startup/hang guards. Higher resolution is synthetic stress, not proof of
  a Switch-equivalent bottleneck. Further FX and linked checks are pending.

## 2026-10-08 — Inkling b99f policy correction

The corrected policy preserves `inkling_slosher_splash2`, the empty-ink
down-smash body, and filters `inkling_slosher_trace`, an additional full-ink
trail. The full-ink splash stays visible. Forward-smash `pablo_trace2` policy
is unchanged; its missing runtime cull evidence is not considered fixed.
The generator reproduces the checked-in policy after rustfmt.

Artifact `dist/b99f087cf8ef0a9c`:

- NRO SHA-256: `b99f087cf8ef0a9c58a6ed7d60ae294a492b8f28183f4d182d090d81ebb9af0c`.
- ELF SHA-256: `882b6ef8ee578a459d88d21e0fabf6f5d881821f959f7a8607c50fa5642fae3b`.
- Source archive SHA-256: `2626352bea8cd631a34b25e91de5bf069746c21d6bc69410956cd3efe3eb6d31`.

246 host tests passed. Host Clippy (all targets, warnings denied),
no-default-features build, Skyline Clippy (warnings denied) and optimized
Skyline packaging passed. Specific runtime acceptance is pending.
The source archive precedes this documentation checkpoint. Nothing in this
entry claims a new release, push or Switch installation.


## b99 linked regression and c0 diagnostics — 2026-10-08

- Exact runtime NRO: `b99f087cf8ef0a9c58a6ed7d60ae294a492b8f28183f4d182d090d81ebb9af0c`.
  Two local Eden instances, CLI commit `10bcd2d849843b146a79c61a21df615e1bdcfcde`.
  A embedded ImGui, G&W fix `bceef1f09e4339149cdab8e7826d17492680add02307002b0a5d62033d1090c8`,
  FX Reduced/online; B standalone ImGui plus Elite, no G&W fix, FX On/all.
- Private room join succeeded when the host challenger prompt was accepted promptly.
  Earlier Surface stock guest attempts discovered the room but failed before gameplay;
  they are not three-station desync tests.
- One seven-minute G&W-versus-Sephiroth Final Destination match: confirmed LessLag,
  then a live switch to LLUltra on both peers. Matching final winner, two KOs and
  47% damage; both returned to stage selection. No disconnect observed.
- Complete paired capture IDs: LessLag A `5319903644070`, B `5319902971610`;
  LLUltra A `5325013204722`, B `5325017052606`. All eight CSVs end
  `complete=1`. These concurrent traces establish active paths, not GPU performance.
- A's captured request log culls `edge_fire3_burst` and retains
  `edge_fire3_burst2` with online-only scope. Both peers restore Vanilla/three
  textures during results cleanup while retaining the saved LLUltra choice.
- Window position: right in connected CSS/stage selection; left in ordinary match.
  Opponent column disappears after B closes. B's Stealth hides its announced
  profile/interface from A; A's announced Ethernet is visible to B.
- Both units were explicitly stopped after testing. Each reported a core dump
  during that requested shutdown; neither is classified as a spontaneous match crash.
  B's original 24 staged paths were restored and hash-verified.
- Durable raw archive:
  `~/.local/share/ssbu-mod-backups/b99-runtime-20261008/runtime-evidence.zip`,
  SHA-256 `55a56f0c5c8882e7f6a9549c1b7a8605337cca4a6a60c5ea8bcc743c4d2cef82`.
  Disconnect supplement:
  `9eeea2bfda3c65ef6a67d32a3508acfc22ff5e2e8c8ae133e605b2c0dc45c3aa`.
- New diagnostic-only NRO:
  `c0e9a625eb44a7359545cb3264b2e7e58d45cf15ea7c7d79ba721fbb5be8dce7`;
  ELF `880160b92d3956c5a9c1b6bc97e8be169592d483511afc71a028911db56cd987`;
  source archive `56b299cef4fd54bf3ca824963e8cd23e4f18807702104021ecc839c9efbf91af`.
  247 host tests, host/Skyline Clippy with warnings denied, no-default build and
  optimized Skyline package pass. Initial host Clippy rejected hex grouping in
  the new test; formatting was corrected and the check rerun successfully.
  No rendering-policy fix is claimed. Runtime diagnosis is ongoing.


## c0 online-only FX scope in offline Training (2026-10-08)

The c0 diagnostic candidate, embedded menu, Training Modpack and SGPO completed
three fresh-state Sephiroth-versus-Ridley Gigaflare captures with
`effects_scope=online`: On 5383529109398, Off 5383900612625 and Reduced
5384281487286. Both CSVs in each pair ended `complete=1`.
All 490 / 487 / 488 callback samples respectively retained the requested mode,
scope Online, effective-Off false and hidden Deluxe window. Full Gigaflare body
and wash remained visible. Reduced emitted code86 (restore) for
`edge_fire3_screen2`, `edge_fire3_burst` and `edge_fire3_burst2`; no suppression,
cull or sibling-substitution decision appeared in these capture windows.
These are visual and scope checks with paused screenshots, not frame-time trials.
They do not close the same-session online-to-offline transition matrix.

Raw files: audittemp/embedded-c0-a/scope-online-training; screenshots in the
adjacent visuals directory, prefix c0-scope-online-training.


## c0 grounded triggered-counter visual check (2026-10-08)

Two finite captures per mode, all timing/API pairs complete:
On 5406959689907 / 5407200212605; Reduced 5407543855417 /
5407781445372; returned On 5408115596411 / 5408354185051.
All callback rows retain the intended mode, scope All and hidden Deluxe menu.
Root reviewed the contact/burst/expiry pictures of trial0 in each mode:
Donkey Kong's jab visibly strikes Scintilla; the shield reacts, the counter
connects and expires, and all three show 20.2% / four hits. This is a grounded
right-facing fixture with TMP Jab/Always, not airborne or opposite-facing coverage.
Both Reduced traces log cull84 for edge_senkou_start and edge_senkou_hold,
keep85 for edge_senkou_shield. hold_attack was not observed. Success-layer names
pass through unchanged and the filter logger omits them; screenshots provide the
cue evidence. No frame-exact gameplay equivalence or performance claim.

Raw audittemp/c0-all-a/counter-on-reduced-return and adjacent visuals,
prefix c0-counter. Pilot1 missed and is excluded; pilot2 established the fixture.
Paused screenshots explain the low sampled FPS shown in some images.

## c0 peer-report performance follow-up and network attempt (October 8)

- 54 Palutena callback windows completed: 18 excluded warmups and 36 measured
  trials, Vanilla/LessLag/LLUltra crossed with On/Off/Reduced and hit/whiff.
  No consistent On-specific penalty or Reduced performance benefit established.
  [Full method, results and limits](https://git.syntaxbyexample.com/intpa/ssbu-fx-control/src/branch/main/docs/FX_EMULATOR_RESULTS.md).
- All accepted API slices cover the guest-input-anchored move plus margin.
  API traces retain capacity termination; only callback traces cover all ten
  seconds. Fable independently verified hit raw files, not the whiff matrix.
- Original TMP counter settings were restored byte-for-byte before Palutena.
  No effect-removal asset mods were installed; no gameplay changes were made.
- The private emulator room reached three members across Yoga A and Surface
  C/D. C discovered the native room and its identity/RTT appeared at the host
  challenger prompt, but native joins did not complete. No match or desync
  acceptance follows. D's first boot was 1.0.0: copied DisabledAddOns entries
  explicitly disabled its installed 13.0.5 update. Root removed only those
  entries after verifying D stopped. Corrected D boot remains untested.
- Surface C/D were stopped and released by 18:01:46 UTC; both reported core
  dumps during requested shutdown. Yoga A's requested stop completed with
  MainPID0 and Result=timeout. These are shutdown outcomes, not spontaneous
  in-match crash findings.
- Both Yoga test profiles were restored with verified backup hashes: all 24
  B paths and the 14 A parent/diagnostic paths. The Surface D fixture remains
  stopped and isolated for a future explicitly coordinated test window.
- Raw archive: `~/.local/share/ssbu-mod-backups/c0-all-runtime-20261008/evidence.zip`.
  It includes raw traces, per-file hashes, fixtures, helpers, Fable review and
  network screenshots/logs. Archive SHA-256:
  `01274ef1a5bd912157502e24a490ab6c1bd88f44f284ee01cfc665c8c55107e2`
  (195,165,945 bytes; CRC verified).
- No Rust changes or new NRO were produced in this follow-up; the existing c0
  build gates remain the applicable evidence. No new push, release or Switch
  write occurred. The user's Switch still has b99.


## Compact renderer acceptance, 2026-10-09

The compact patch is built from public imgui-smash `bb47b205` plus patch
`4e138e3513347326`. Two clean builds produced identical archive
`6c9591cc9f140ca6`. Four C++ helper tests compile with warnings denied. The
archive build still emits existing upstream warnings, which are distinct
from the warning-free host and Switch Rust Clippy checks.

The final rewrite `7fdad40f10705afbe5fe336ccb0a58543db935143a6601cc4d0045d8296ff12d`
pairs with ELF `a77fe5fd440d02ff761158f4363cc3026299d90af21c3de800861ef192ead3fa`.
All 247 host tests, no-default build, host/Skyline Clippy with warnings denied,
and optimized Skyline build pass. Source packaging includes the exact patch,
pin, toolchain and dependency license. No defined-symbol GLOB_DAT/JUMP_SLOT
relocations point back into this module.

- Embedded run `menu-compact-a` used the preceding `28ac474abf75f1aa` binary.
  Removing one unused status accessor after that build changed only build-ID
  immediates/string bytes in the executable; no other code or data changed.
  At 3× Eden resolution it passed main/CSS Settings and Performance,
  LLDoubles/LLUltra/LessLag edits, two screenshot-confirmed stage→CSS returns,
  Link/Ness Training with the window on the right, unobscured native pause,
  Training Modpack open/close, SGPO display and Training→CSS return.
  Earlier short-hold scripts did not leave CSS and are not counted as returns.
  The run was slower than real time (roughly 24–52 FPS), so this is visibility
  and coexistence evidence, not a performance comparison. Final counters:
  28,188 submissions, 28,180 reuses, three rejected overlapping frame entries,
  zero pending-slot skips, failed slots, geometry skips or command overflows.
  Normal exit after 882 s; peak memory 7,007,911,936 bytes, no cgroup OOM.
- External run `menu-compact-b` used final `7fdad40f10705afb` and patched
  standalone diagnostic `99bdfa66a6002826`, with Elite, TMP and SGPO.
  Bootstrap ownership and external-ready checkpoints matched the rewrite's
  module. Both windows appeared together on main, at the start of an actual
  Lucario/Duck Hunt match on Hollow Bastion, on results, and after return to
  CSS. All three profiles and a separate Rules→CSS return passed. Captures
  named `match-input` and `match-playing` are results-screen images, not
  gameplay evidence; `profiles-match-both` is the actual match-start image.
  At 1× the observed menu/match run was approximately 60 FPS. Final native
  counters: 29,286 submissions, 29,278 reuses, seven overlapping entries skipped,
  zero pending/failed/geometry/overflow counts. Counter files are independent
  snapshots. Normal exit after 511 s; peak 7,786,078,208 bytes, no cgroup OOM.
- Boundary run `menu-csmall` used isolated small-budget diagnostic
  `3c8c44bf13d3dcd2` and archive `12bd6129936ebee7`. It reached CSS while
  10,218 command-memory overflows were safely discarded/reset, with zero GPU
  submissions or failed slots and three overlapping entries skipped.
  The absent overlay is intentional in this diagnostic. Normal exit after
  227 s; peak 6,670,692,352 bytes, no cgroup OOM. It is never a deployable NRO.

For the compact embedded runs, use the counters in `renderer-status.txt`.
Older `imgui-renderer-status.txt` and `frame-guard-status.txt` files in that
profile are stale pre-cleanup evidence. The external diagnostic writes fresh
files labeled with the current archive digest. All three profile journals
were restored after shutdown.

Artifacts, raw logs, screenshots, source and SHA manifests are preserved under:

```text
/home/intpa/.local/share/ssbu-mod-backups/menu-backend-build-20261008/
  final-compact/7fdad40f10705afb/
  completed-runs/menu-compact-a/
  completed-runs/menu-compact-b/
  completed-runs/menu-csmall/
```

These trials do not prove a deterministic cure of every invisible-window
case. The original failure was intermittent, and a guard-only rollback also
passed a later repeat. Switch validation and input-to-display timing remain
separate. No Switch files were changed for this campaign.

The final standalone release companion is
`9a9e575c910c6471152585459d8611d66eff08b7e2058c4f2a63711628a905ed`
with ELF `1aa78f5e7b208c2963ccedeb95bfca6a0dbdde8be99201a5862618eee3e3ce75`.
Its diagnostic features are disabled. Host Clippy/no-default build and
Switch Clippy pass with warnings denied, including one compile with both
diagnostic features. The corrected release log is warning-free.
`menu-release-b` smoke-tested this exact NRO with final Deluxe: both menus on
main and CSS, Performance view and valid external registration, around 60 FPS.
It exited normally after 210 s, peak
6,535,589,888 bytes; its four-file journal was restored.
The old standalone counter files were not updated in this release and are
not measurements of this run. The longer match test above uses the diagnostic
build of the same native backend.

## Buffered diagnostic export candidate, 2026-10-09

Local candidate `62a42df8d01d1738` addresses the observed incomplete API CSV export,
not the separate launch failures. Installed5c72's capture129729755560 had a complete
602-sample callback CSV, but its API CSV stopped at partial row6308 of6602.
The pinned Switch filesystem implementation requests Flush for each unbuffered
formatting fragment. The correction uses64KiB buffered writes, explicit fallible
flush followed by File sync_all, and Exporting/Saved/Error ordering. Capture
semantics, CSV serializers and no-overwrite creation guards remain the same.

- Final full host suite:257passed, including4export regressions. Byte equality,
  bounded underlying writes, mixed-sign API fields, write/flush/status errors,
  phase ordering and file overwrite protection are covered.
- `cargo clippy --all-targets -- -D warnings`, `cargo build --no-default-features`,
  `cargo skyline clippy -- -- -D warnings`, and `cargo skyline build --release`
  passed on reviewed source. The private helper is compiled for Switch and tests;
  no host dead-code suppression is retained.
- Build `0.1.0-2db70f90aaae-dirty-1791570881`; NRO2420736bytes, SHA256
  `62a42df8d01d17386a045fccd0c080770779223e9f7b62ddba9a665cc1da2bae`.
- ELF SHA256 `842875e692f6363a8b984c45c54f5b03ef49613e887729517fcffb859521c935`;
  NRO/ELF executable bytes match and debug_info/debug_line/symtab are present.
- All76archived source files match checkout. Compared with installed5c72 source,
  only src/lib.rs and src/runtime/timing.rs changed, plus new src/capture_export.rs.
  Native ImGui dependency provenance is identical.
- Exact bundle and immutable gate receipts are outside the repository under
  `~/.local/share/ssbu-mod-backups/capture-export-fix-20261009/`.

Not deployed, not runtime-tested. Hardware export completion/performance remains
pending; neither these gates nor buffering establish a launch-crash fix. The
operator has been released to play while separate crash/source audits continue.

## Horizontal menu and peer-retirement candidate, 2026-10-09

Local bundle `4c1947b84add6114` retains buffered-export62a and adds:

- Horizontal Settings for ready Singles/Doubles with0.42 window-background alpha;
  existing Training/lobby placement and vertical diagnostic/performance tables remain.
- Internal-only Platform and emulator-only FPS++ display/navigation. Boost safety
  gates remain unchanged.
- Actual ImGui content-fit height, including wrapped local/opponent values and the
  plot, replacing fixed or estimated row-count heights.
- Identity-wide retirement on a confirmed station leave, including other aliases
  that previously kept its peer visible. Explicit subtype3/4 disconnects also clear
  aliases. Connected peers with Vanilla settings remain valid; other lobby peers
  survive a departure. Unknown RTT protocols cannot retire a current station just
  because their numeric indices overlap.

Final parent gates passed:261hosttests, `cargo clippy --all-targets -- -D warnings`,
`cargo build --no-default-features`, `cargo skyline clippy -- -- -D warnings`,
`cargo skyline build --release`, exact package and source verification. Switch-only
Clippy found two style lints in the draft; both were fixed without suppressions.
The reconnect test uses an actual synthetic disconnect sequence, not manual cleanup
of the stale index. Host tests do not claim live renderer/network behavior.

Build `0.1.0-2db70f90aaae-dirty-1791574315`, NRO SHA256
`4c1947b84add611414cd485c7aa62a4ccfc05db35f7b1e2a8bac7def4aed7db6`.
NRO/ELF executable bytes match, debug_info/debug_line/symtab are present, all76
archived source files match the reviewed checkout, and native dependency provenance
is unchanged. Six files differ from62a: menu.rs, overlay.rs, peer_transport.rs,
pia_rtt.rs (regression only), runtime/imgui.rs (existing wrap API bindings), and
runtime/ui.rs. Runtime RTT hook source and buffered-export sources match62a exactly.
Evidence: `~/.local/share/ssbu-mod-backups/menu-peer-fix-20261009/`.

At package verification this was not deployed or runtime-tested; installation is
recorded below. The specific declined-rematch path still needs
runtime confirmation of its emitted lifecycle events; no speculative protocol
cleanup is shipped to force the UI to disappear. The operator's earlier multi-
person Eden lobby test above remains attributed to the Training Modpack agent.
Crash investigation is separate.

### DBI USB MTP installation — 2026-10-09 22:32Z

The operator requested copying4c19 with Smash closed using DBI's USB MTP responder.
Its `1: SD Card` storage was identified independently of the old Hekate block mount.
Only the active rewrite NRO was replaced after verifying local and device backups.
Full device readback matched SHA256
`4c1947b84add611414cd485c7aa62a4ccfc05db35f7b1e2a8bac7def4aed7db6`,2420736bytes.

Previous5c72 is backed up locally and on SD at
`/ultimate/ssbu_online_deluxe_rewrite/backups/pre-menu-4c1947b84add6114.nro`;
the remote backup was independently read back and hash-verified. All eight other
plugins and six protected files match before/after hashes; plugin membership is
still nine files. The freshly saved INI is LessLag/input3/FX Off/all/Stealth Off,
SHA256 `0617a94705ea8d2da3fff782161b5339f6a3880948a807894a01692bccd9c2d3`.

Evidence: `~/.local/share/ssbu-mod-backups/menu-peer-fix-20261009/switch-mtp/`,
including `preflight.json`, `deployment.json` (`installed_verified`), local backups
and downloaded readbacks. Both bounded jobs passed; the host MTP mount was released.
This accepts the transfer, not menu/gameplay/disconnect/export behavior or a crash fix.

## Plugin registration API candidate, 2026-10-12

Local candidate `068a1cf6f418349b` adds the exported
`ssbu_online_deluxe_plugin_api_v1` table, the `sdk/abi` ABI crate (a root path
dependency), the host registry `src/plugin_api.rs`, and a Plugins view in the
chord cycle that appears only after a separate NRO registers. Settings,
Performance and Debug rendering are unchanged; the compact match strip reuses
the approved two-row layout for the Plugins page with four-field paging.

- Host suite: 277 unit tests plus `tests/plugin_sdk_roundtrip.rs` (the public
  Rust SDK registering through a table of the exported shape into the host
  registry, then stepping values through the real callbacks).
- `cargo clippy --all-targets -- -D warnings`, `cargo skyline clippy -- -- -D warnings`,
  `cargo build --no-default-features`, `cargo fmt --check` and
  `cargo skyline build --release` passed.
- Build `0.1.0-2db70f90aaae-dirty-1791806695`; NRO 2445312 bytes, SHA256
  `068a1cf6f418349bd6379ed6beaaa1ade084e377a25077b548e2ad5ea9e28e0c`; module
  `8b1648f683c5795940ee3bf283a03d4559f7e9da5c0499f8bd6804259e178d70`; ELF SHA256
  `aa1c1b97dab38a64797faaf81576192e3f710d3b4d1405f573c747fbdc33999a`. NRO/ELF
  executable bytes match; `nm -D` lists the export as a defined `T` symbol.
- Demo consumer `sdk/examples/demo` (`cargo skyline build --release`): NRO SHA256
  `80e023e2773b0a1734564563aa798e877f68ddd667e9b1b1548ef9a01471d0dc`; it imports
  only `nn::ro::LookupSymbol`, has no undefined reference to the host symbol and
  carries the symbol string in rodata. SDK crate: 13 host tests and 1 doctest.
- Evidence: `~/.local/share/ssbu-mod-backups/plugin-api-tangent-20261012/`
  (`artifacts/068a1cf6f418349b/`, `DESIGN.md`, `SDK_WORKER_REPORT.md`).

Not deployed, not loaded in Eden, not runtime-tested. Page rendering,
controller flow and live cross-NRO registration remain unverified; the demo's
log line has not been observed. No replay recording is included.

Follow-up the same day added optional host persistence (plugin-declared INI
path; `OptionDesc` 48 bytes, `PluginDesc` 72 bytes, diagnostic 101). Root
gates rerun: 280 host tests plus the round-trip test, host/Skyline Clippy with
warnings denied, no-default build, rustfmt and Skyline release pass. The
repackaged candidate's hashes are in the tangent evidence directory's
`artifacts/` manifests.

### Eden preview `plugins-page-1012a` (same day)

Candidate `9d08a711b271e0ac` plus the demo NRO
`8c45cd0238f460900f2b453360ba0fce362a28cf06ec5f95e2dcfbdefa3f6b43` were staged
into the isolated Eden profile (three originals backed up, journal RESTORED
afterwards; the owned unit exited with status 0). Observed on the main menu:
the separately built demo registered and the Plugins page listed `Demo` with
Enabled, Mode and the read-only Status row; the chord cycle ran Settings →
Plugins → Performance → Hidden; Right on Enabled showed On and Down then Left
on Mode showed C. Host persistence created
`sd:/ultimate/deluxe_plugin_demo/config.ini` with `enabled=on` and then
`mode=c`; events.log recorded code 100 (request Applied) followed by code 101
(persisted). The boot-time code 99 record was lost to the 128-entry ring before
the sink started; absence of 99 in a boot log is therefore not evidence either
way. Not observed: the compact in-match strip for the Plugins page (no match
was started; navigation entered the World of Light map and was abandoned to
avoid touching that save) and any hardware run.
Evidence: `~/.local/share/ssbu-mod-backups/plugin-api-tangent-20261012/`
(`result.json`, `screenshots/plugins-page-1012a-*.png`, `run-evidence/plugins-page-1012a/`).

### Eden match preview `plugins-match-1012b` (same day, private display)

Same candidate and demo NRO, launched on a private display (weston headless
with the Intel GL renderer plus Xwayland `:2`, no desktop window; Eden at
1280x720, 60 FPS). An Astra driver reached an offline Smash match, Shulk versus
a level 9 Yoshi CPU on Final Destination. Observed in the match: the compact
two-row Settings strip and the compact Plugins strip
(`>Demo.Enabled:Off   Demo.Mode:A   Demo.Status:Ready` over
`Plugins 1/1` and the timing line); Right on Enabled applied (codes 100 and
101) and the host wrote `enabled=on`; after the match ended the window returned
to the vertical table showing Enabled On. Not captured in this run: the compact
Performance strip (match over first), a second edit and the hidden state. The
profile was restored and the unit exited 0.
Screenshots: `screenshots/plugins-match-1012b-settings-match.png`,
`plugins-match-1012b-plugins-match.png`, `plugins-match-1012b-plugins-match-edit.png`;
archive `run-evidence/plugins-match-1012b/`.

### Plugin value tones and read-only rows — 2026-10-12

Candidate `e59fc9a6bd266790` (descriptor version 2: per-choice tones and
`OPTION_FLAG_READ_ONLY`; table and symbol unchanged; version 1 descriptors
still accepted). 282 host tests plus the SDK round-trip; host/Skyline Clippy
with warnings denied, no-default build, rustfmt and Skyline release pass. SDK:
22 tests including the version 1 fallback. Eden run `plugins-tones-1012c` on
the private display with the demo NRO
`2f9fcdd730b14ab22fc6c19cfc0a6fe9f61d072379b1cb30a695e43b8f004b60`: on the
main menu the Status row rendered Degraded in yellow and Ready in green and
Enabled turned green after Right; Right on the read-only Status row changed
nothing. In a Lucario-versus-CPU match the compact strip showed On in green
and Failed in red while the status cycled. Profile restored, unit exited 0.
Evidence: tangent root `result-tones-1012c.json`, `screenshots/plugins-tones-1012c-*`,
`run-evidence/plugins-tones-1012c/`. Not on hardware.

### Window chord fix — 2026-10-12

Hardware logs from 9d08 (code 98 records in `switch-mtp/collect-1012/events.log`)
showed every failed L+R+Down attempt with no Down edge: the D-pad registered a
frame before both bumpers, so the old "Down edge while the chord is held" rule
ignored it, while ZL+ZR+Down worked because the triggers register later. The
Debug attempts (L+R+Minus, ZL+ZR+Minus) never had Down in the same sample.
Commands now fire when the whole combination is held, in any order, once per
hold (`overlay::CommandLatch`); chord + Plus/Minus alone toggles Debug, and
Debug stays open while the modifier is held. 283 host tests pass with the new
regression; gates as above. Hardware retest pending. Note: because the game
and native CSS panes still see Down before the bumpers, a Down-first chord on
character select can nudge the native profile row once before the window
opens; the old gesture (bumpers first) avoids it.

### Character-select frame budget under LessLag — Eden measurements, 2026-10-12

Twenty-one captures on the private display (candidate d773, isolated profile):
at 1x every mode holds 60 on the CSS, so Eden does not reproduce the Switch
drop directly. A CPU quota of 175% on Eden's own unit does reproduce a
LessLag-only drop (Vanilla 60, LLDoubles 60, LessLag 53-57) while all modes
show the same share of slow frames and the same main-thread time; the
three-image modes recover with short frames and the two-image ring cannot.
Core modes under quota separate the components: two images without render
options still drop, three images without them hold, and render options add
about 0.45 ms of serialized main-thread work per CSS frame. GPU stress at 2x,
3x and 4x never singles out LessLag. The CSS is not heavier than a match on
the instrumented spans; the hardware difference is frame-time variance.
Report and raw captures: tangent evidence `css-budget/`. No code change; the
hardware ABAB (LessLag, LLDoubles, Vanilla on one CSS visit; `lesslag_core`
and `doubles_core` boots) is the decisive confirmation.

### Character-select slowdown: call-level attribution and dispatcher backoff — 2026-10-12

Astra xhigh traced the CSS frame on Eden with host `perf`, the dynarmic perf
map, per-thread schedstat and a diagnostic probe build (worktree
`work/css-trace-probe`, never deployed). Result (tangent evidence
`css-budget/astra/CSS_SLOW_FRAME_ATTRIBUTION.md`):

- The mod's own CSS paths are negligible (native handler 0.005 ms, banner
  0.085 ms, scene-manager wrapper 0.136 ms per frame).
- The main loop (`main+0x374b898`) waits through `0x354ccd0` on dependency
  counter `0x5333160`. All 600/600 registrations on that counter resolve to
  one job, `main+0x375e260`, an `nn::ui2d::Layout` animation update on
  worker 1. The main thread waits 4.83 ms per CSS frame for it (8.25 ms under
  the 175% quota) versus 0.91 ms (1.33 ms) in a live match.
- While waiting, the main thread runs the task dispatcher `main+0x354a6d0`,
  which polls six queues through `std::mutex` → `nn::os::LockMutex` →
  SDK critical section: 1,313 of 1,350 state samples show all six queues
  empty, one pending record and counter 1. The dispatcher uses its light
  event only when the pending vector is empty, so it spins. The lock
  Enter/Leave fast paths are 22% of all Eden samples unlimited and 27.8%
  under quota; the lock word is free in 98% of samples (churn, not contention).
- Intervention A/B/A on the same binary and boot (1 µs `SleepThread` about
  once per 171 idle passes): CSS LessLag 175% 52.23 / 59.31 / 51.89 fps and
  27.84 / 22.90 / 28.08 CPU ms/frame; Vanilla 59.32 / 60.10 / 59.85 and
  26.90 / 23.04 / 26.27; LLDoubles 59.55 / 60.03 / 58.33 and 26.44 / 23.13 /
  27.84; CSS LessLag unlimited 59.08 / 60.00 / 59.80; live match 175%
  57.43 / 59.35 / 59.90 and 20.62 / 18.74 / 19.32. Main-core CPU fell
  11.70 → 6.67 → 11.63 ms/frame while worker-1 CPU stayed 5.7–5.8 ms: the
  gain removes polling, not UI work. Screenshots
  `css-budget/astra/screenshots/{before,after}-css-lesslag-cpu175.png`
  (54.1 → 59.4 fps, 27.2 → 23.3 CPU ms/frame).

Implementation (this build): `src/dispatcher_backoff.rs` (pure policy, host
tests) and `src/runtime/dispatcher_backoff.rs` (inline hook at
`main+0x354ab48`, the `ldp x8, x9, [x24, #0xe0]` after the sixth empty probe).
Install verifies all five original words (`a94e2708 eb09011f 54000d01
f94003e0 9411f072`); a changed first word with the rest intact means another
plugin owns the site (`External`). The hook sleeps only on `MainThread`, only
when the enabled bit is set, all six queue sizes are zero, the pending vector
`[d+0xe0, d+0xe8)` is non-empty with every record's counter still > 0, and
the ready-resume vector `[d+0xf8, d+0x100)` is empty. Default policy: 16
idle passes spin, then 5 µs doubling to 50 µs on every idle pass;
`dispatcher_backoff.ini` overrides (`preset=measured` reproduces the Eden
cadence, `cooldown_us` adds Astra's optional limiter). `DISABLE_BACKOFF`
turns it off. Host gates: 293 tests, clippy host and Skyline with warnings
denied, no-default build, rustfmt.

Eden production A/B/A of the clean 0.3.0 NRO (`f22d94950be1b460`, build
0.3.0-8aeaa5391343-1791833455; Astra low driver, isolated profile, private
display, 1x, FX On, 175% quota with 5 ms period, 17–20 s API windows;
ledger `css-budget/prod-ab/STATUS.md`, report `PROD_AB.md`):

| Run | Hook | FPS | Eden CPU ms/frame | Backoff row |
| --- | --- | ---: | ---: | --- |
| CSS LessLag 175% (A) | `DISABLE_BACKOFF` | 55.59 | 26.55 | Off |
| CSS LessLag 175% (B) | default policy | 60.00 | 19.50 | 3068/s 50 us |
| CSS LessLag 175% (A) | `DISABLE_BACKOFF` | 54.15 | 26.83 | Off |
| CSS LessLag 175% | `preset=measured` | 59.03 | 22.92 | 2744/s 1 us |
| CSS Vanilla 175% | default policy | 60.01 | 20.08 | 2590/s 50 us |
| Live match LessLag 175% (Lucario/Kirby, Battlefield) | default policy | 59.84 | 16.34 | compact strip has no row; code 103 rate 1069/s, last 50 us |
| CSS LessLag unlimited | default policy | 60.0 | — | — |

Install log code 102 outcome 1 (installed) on every boot; no crash,
`Unavailable` or `External`. A first match attempt that crossed into the
results screen during capture was excluded and repeated. Verdict: the
production hook reproduces the diagnostic gain and exceeds it on the CSS
(flat 60 under quota); Vanilla and the match keep 60 with lower CPU. The
in-match compact Performance strip does not show the Backoff row; the
vertical table does.

### 0.3.1 — character-select capture and backoff eligibility gate — 2026-10-13

Hardware result of 0.3.0 (operator, manual LessLag, FX On): CSS idle 58–60 fps
(was 48–50) but 46 fps while the cursor hand moves, with the default ramp and
with `preset=measured`; 60 in a live match; a clean two-image boot reproduces
46 (`css-budget/hand/hardware/clean-boot/renderer-status.txt`: active 2, index
masks 0x3, generation 1). Astra xhigh (`css-budget/hand/CSS_HAND_ATTRIBUTION.md`)
could not reproduce the hand drop on Eden (60 moving in every configuration)
and attributed the menu's weight to one serial UI traversal job on pool-0
worker 1 (core 1, kernel priority 36): 1,765 Layout, 19,201 transform and
about 14,200 enabled-binding entries per frame against 426 / 3,610 / 967 in a
fight, already while idle. Hover adds at most 20 bindings, no storage read
burst, no observed nested cleanup wait, sync waits near zero. Open hardware
discriminators: the conditional cleanup wait at `main+0x37a2b24`, asset loads,
two-image fence reuse, completion pickup. Rejected: another timer preset,
earlier dispatch or a split wait, `all_threads=1` as default, CPU boost.
Prescription: `CSS_CAPTURE_CONTRACT.md`, implemented here.

Build (commit 00660a3, crate 0.3.1):

- Eligibility gate in `src/runtime/dispatcher_backoff.rs`: a sleep requires
  `SCENE_SEEN` (any `auto_driver::set_scene` call, i.e. a scene hook fired)
  and a tracked present within 250 ms (`nvn::queue_present`). Before that the
  Backoff row reads `Armed`; code 102 `a=5` is written once when the gate
  first passes. The clock is read only on passes that would sleep. Astra's
  boot audit showed the hook sleeping before the title with two pending
  records on Eden; this keeps startup and a stalled renderer on the original
  loop. It does not claim to fix the `main+0x37432cc` initialization fault,
  which is original game code after `nn::prepo::Initialize`.
- `CAPTURE_CSS` boot flag → `src/runtime/css_capture.rs`: three hooks on the
  five-word 13.0.5 fingerprints of `0x3549720` (dispatcher registration),
  `0x354ccd0` (dependency wait) and `0x375e260` (UI job), installed after the
  renderer family verified the shared sites; `nn::os::GetThreadPriority` and
  `GetThreadCoreMask` resolved through `nn::ro::LookupSymbol` (never linked,
  never set). CSS entry (native banner `0x1a12f60` hook) records code 12 and
  arms a 30 s capture labelled `css`; exit (one second without a CSS update)
  records code 13. Rows: 60/61 per-frame sleep count, summed and peak actual
  sleep µs, idle streaks; 62/63 thread pointer, priority, ideal core and
  affinity mask once per thread per capture; 64 UI child dispatch (worker,
  queue priority, thread); 66 every dependency wait with the counter's image
  offset and thread label; 77 UI job span. Code 104 reports the hook mask.
- Timing capture 30 s, 2048 samples; API trace 65536 events. The static
  initializer is now all zero (`Finish::Aborted` first) so both buffers live
  in `.bss`: the NRO is 2174976 B against 2465792 B for 0.3.0.
- `tools/analyze_render_trace.py` decodes the kinds and adds a `css_capture`
  section (waits by counter, UI job stats, nested waits on the UI thread,
  per-frame backoff distribution, thread table, dispatch pairs); 24 Python
  tests.

Gates: 294 host tests, `cargo clippy --all-targets -- -D warnings`,
`cargo skyline clippy -- -- -D warnings`, `cargo build --no-default-features`,
`cargo fmt --check`, `cargo skyline build --release`. Clean package
`artifacts/9511e8c0f7946bd5/` in the tangent evidence: build
`0.3.1-00660a3d63cc-1791859184`, NRO SHA-256
`9511e8c0f7946bd50434d78f7607283a84f13d299dfe6e6e2aad88c77b43e840`
(2174976 B, module `ca2cbd7e…8780`), ELF `55f94118…9608`.
Not installed on hardware (the Switch has f22d / 0.3.0); not run on Eden.
Forgejo release `v0.3.1` (id 246, tag on ed8de66) carries the clean NRO
(asset 68, 2174976 B, download read back as 9511e8c0…e840) and
`manifest.json` (asset 69).

Switch install of 0.3.1, 2026-10-13 11:48Z (at the operator's request): DBI MTP single-writer transaction `switch-mtp-9511/` in the tangent
evidence, preflight on f22d, rollback archive
`backups/pre-css-capture-9511e8c0f7946bd5.nro` read back as f22d, target read
back as 9511e8c0…e840, other plugins and protected files unchanged; empty
`CAPTURE_CSS` created. First boot: `install.log` code 104 `a=7 b=7` (all three
hooks and both SDK symbols), code 102 `a=5` (gate opened on the first scene),
codes 12/31/32 (CSS entry, capture armed, export complete).

First hardware CSS capture (`css-budget/hand/hardware/controller-captures/switch-0.3.1-1013/`,
API trace 33,075 events, 1,273 callbacks, elapsed, zero skipped; Local
Wireless CSS because NSO demanded a system update; the hand moved ~3 s, held
still ~11 s, then moved ~16 s). Astra's independent decode
(`css-budget/hand/HARDWARE_CAPTURE_1013.md`, reproduces the repository
analyzer exactly): still 48.3 fps (present cadence 20.69 ms) against moving
38.8 fps (25.79 ms); of the 5.10 ms increase, the texture-ready sync wait
(kind 3, after `nvnWindowAcquireTexture`) grows 3.99 ms and the main thread's
wait on the UI barrier grows 0.84 ms. UI job 11.41 → 12.21 ms mean, p99
11.92 → 15.70 ms; only 1 of 619 moving jobs exceeded 16.67 ms. Zero nested
dependency waits inside the UI job (all 7,631 kind-66 rows are MainThread).
Thread table: UI worker SDK priority 8, ideal core 1, mask 0x2; MainThread
priority 16, ideal core 0, mask 0x1, so the backoff cannot donate core 1 time.
Backoff: 227–241 sleeps per frame, 12.0–12.7 ms summed, peak 77 µs; job
return → barrier return 32 µs. Surviving candidates: two-image image/sync
coupling (dominant observed increment) and UI computation (attribution
target); excluded: the `0x354ccd0` nested cleanup wait and any multi-ms
completion pickup. The operator rejected LLDoubles as a trial or fallback; the
next steps are a clean offline Smash-mode CSS control and a flag-gated
per-owner attribution probe on two-image LessLag.

### 0.3.2 — always-on hang trace for loading-screen freezes — 2026-10-13

Context: with 0.3.1 and the operator's other plugins installed, the Training
Modpack froze on a loading screen on the console (no crash report; the
modpack's own log stopped after the fighter allocations) while the same stack
entered Training on Eden. The modpack investigation asked this plugin for
timestamped liveness lines it could align with its heartbeat. The two most
recent Smash crash reports on the card (2026-10-12 20:20Z, 2026-10-13 11:31Z,
both before 0.3.1) are `training_modpack` BRK aborts, not Deluxe.

Build (commits 082a6dc code, 85a7563 docs, crate 0.3.2):

- `src/hang_trace.rs` (pure, host-tested) and `src/runtime/hang_trace.rs`:
  ten tracked hooks with lock-free call counters, in-flight counts and entry
  ticks: `present` (tracked window), `backoff_pass`, `backoff_sleep` (around
  `SleepThread`), `css_init` / `css_wait` / `css_ui_job` (around the
  `CAPTURE_CSS` originals), `fighter_load` / `fighter_unload` (`common`
  module chain), `fighter_main` (inline per-fighter tick body) and `cut_in`.
  Hooks touch atomics only; the diagnostics thread writes
  `hang-trace.log` (rotates at 2 MiB to `hang-trace.previous.log`): header
  with `boot_tick` and `tick_hz`, `[Heartbeat]` every 5 s with
  `[ACTIVE xN age_ms=…]` on any hook that entered and has not returned,
  `[HangTrace] first <hook>` once per scene, `[HangTrace] scene a -> b` from
  `auto_driver::store_scene`. Installed right after the version check so a
  hang during hook installation still leaves the header.
- Zero-code bisection order documented in `docs/DEBUGGING.md`: remove
  `CAPTURE_CSS`, then `DISABLE_BACKOFF`, then the other `DISABLE_*` flags.

Gates: 298 host tests (4 new), `cargo clippy --all-targets -- -D warnings`,
`cargo skyline clippy -- -- -D warnings`, `cargo build --no-default-features`,
`cargo fmt --check`, `cargo skyline build --release`. Clean package
`artifacts/73823f8f67d6a3b2/` in the tangent evidence: build
`0.3.2-85a756321310-1791911904`, NRO SHA-256
`73823f8f67d6a3b2d3a0de35814042fa1ecea293fef01dae0828cbc1f6eefd8d`
(2183168 B, module `63b8b5fb…a8fc`), ELF `54fe3aa1…2e8a`. Not installed on
hardware (the Switch has 9511e8c0 / 0.3.1); not run on Eden.
Forgejo release `v0.3.2` (id 356, tag on 022a7d0) carries the clean NRO
(asset 70, 2183168 B, download read back as 73823f8f…fd8d) and
`manifest.json` (asset 71).

Switch install of 0.3.2, 2026-10-13 17:26Z (at the operator's request): DBI MTP transaction `switch-mtp-7382/` in the tangent evidence,
preflight on 9511e8c0, rollback archive
`backups/pre-hang-trace-73823f8f67d6a3b2.nro` read back as 9511e8c0, target
read back as 73823f8f…fd8d, other plugins and protected files unchanged.
`CAPTURE_CSS` left in place. Runtime result pending.

Switch procedure for the capture: put `CAPTURE_CSS` in the data directory,
boot with manual LessLag and FX On, enter the character select, keep the
hand still for ten seconds, move it continuously for twenty, wait for the
Debug view `Timing` row to say saved, quit to HOME, then pull
`timing-<id>.csv` and `api-<id>.csv` (label `css`). Also compare LessLag
against LLDoubles on the same CSS while moving (zero-code ring test).

### 0.3.3 — API trace buffer allocated per capture, not resident — 2026-10-13

Context: on the evening of 2026-10-13 the console (0.3.2 + Training Modpack
r31 + the OBS companion's trace build) failed four boots before the title.
Atmosphère reports `01791924547`, `01791926109`, `01791926153`, `01791926317`
(tangent `hang-1013/boots-evening/crash/`) fault at address 0 in unrelated
game threads: `BCATCacheStorage` inside `memset` (twice), `render dispatch`
in `nnosBreak`, MainThread at `cross2_Release.nss+0x38b6804`. No plugin frame
is on any crashing stack; Deluxe's thread is idle in each. Deluxe's own
`hang-trace.previous.log` from one of those boots shows `present=0` and every
counter at zero for 80 s with `ready=1`: the game never produced a frame while
every Deluxe hook was installed and idle. Reading: allocations are failing in
the game heap that Skyline plugins are loaded from, and the loaded set grew
that day (modpack 2494464 → 3379200 B, companion 172032 → 212992 B).

Deluxe's part, measured on the packaged ELFs (`readelf`-equivalent section
sizes, tangent `artifacts/`):

| Build | NRO | .text | .data | .bss | resident |
|---|---|---|---|---|---|
| 0.3.0 f22d | 2465792 | 1577024 | 470688 | 55864 | 2.10 MB |
| 0.3.1 9511e8c0 | 2174976 | 1583120 | 175864 | 2218856 | 3.98 MB |
| 0.3.2 73823f8f | 2183168 | 1590880 | 176136 | 2219288 | 3.99 MB |

The jump is `render_trace::CAPACITY` = 65536 × 32 B moved into `.bss` in
0.3.1 so a 30 s CSS capture never ends on capacity. 0.3.3 keeps the capacity
but allocates the buffer in `Trace::begin` (diagnostics thread, never a hook)
and frees it in `take_ready` after the CSV export; `observe` returns without
writing when no buffer exists. No capture, hook, flag or row changed.

Gates: 298 host tests, `cargo clippy --all-targets -- -D warnings`,
`cargo skyline clippy -- -- -D warnings`, `cargo build --no-default-features`,
`cargo fmt --check`, `cargo skyline build --release`. Clean package
`artifacts/e930c8f34d1752bf/` in the tangent evidence: build
`0.3.3-528137cdff51-1791927586`, NRO SHA-256
`e930c8f34d1752bf24b1819cc8b86322abef3c2303ef50f3ef8544a5ae9d25f3`
(2183168 B, module `047e3f9d…0575`), ELF `db3da70f…c2c9`; packaged ELF
sections .text 1590848 / .data 176136 / .bss 122152, resident 2.23 MB (was
3.99 MB). Forgejo release `v0.3.3` (id 357, tag on ab53b43) carries the clean
NRO (asset 72, 2183168 B, download read back as e930c8f3…25f3) and
`manifest.json` (asset 73).

Switch install of 0.3.3, 2026-10-13 ~22:10Z (at the operator's request, with
DBI's FTP server running): DBI FTP transaction `switch-ftp-e930/` in the tangent
evidence (temp-name upload, readback, rename; one connection per step
because DBI's server drops long or overwriting STORs). Installed NRO read
back as 73823f8f…fd8d (0.3.2) before, saved to
`backups/pre-trace-heap-e930c8f34d1752bf.nro` on the card (verified) and
locally; target read back as e930c8f3…25f3 after; the other nine plugins
unchanged. Runtime result pending.

0.3.4 (same evening; the operator asked whether the dump would crash once
activated): in 0.3.3 the per-capture allocation used `vec!`, so arming a
capture on a game heap that could not spare 2 MiB would have aborted the
process. 0.3.4 reserves the buffer with `Vec::try_reserve_exact` and makes
`Trace::begin` return false when that fails; `timing::Service::poll` already
treats a false `begin` as "timing capture could not be armed" (status line,
both captures finished and drained), so the game continues without a CSV.
Allocation still happens only on the diagnostics thread. Gates as for 0.3.3
(298 host tests, host and Skyline Clippy with warnings denied, no-default
build, fmt, Skyline release). Package `artifacts/f514bfe81f142541/`: build
`0.3.4-f1d570ccb074-1791929407`, NRO SHA-256
`f514bfe81f142541e5211bc2a9302581fc0d162159d111d5ebd56df41a2042de`
(2183168 B, module `88c1ecdb…3d81`), ELF `fb75fa44…9acb`, sections
unchanged from 0.3.3 (.bss 122152, resident 2.23 MB). Forgejo release
`v0.3.4` (id 358, tag on 40edc90): NRO asset 74 (download read back as
f514bfe8…42de), `manifest.json` asset 75. Not installed on hardware (the
Switch has 0.3.3).

0.3.5 (requested by the operator) removes the capture
buffer as a memory question altogether:

- Rows are packed to 20 B (begin and span relative to the capture start, both
  32-bit; a 30 s capture is 576 M ticks) and reconstructed to absolute ticks
  on export, so the CSV columns are unchanged.
- Hooks fill one of two 4096-row chunks (`render_trace::CHUNK`, 160 KiB in
  `.bss`, the whole resident cost); the diagnostics thread drains the other
  into the already-open `api-<id>.csv` every 500 ms. A chunk hands over only
  after the other was drained; rows that arrive while both are full are
  counted in `skipped`, never blocked on. Capture length is bounded by the
  30 s duration, no longer by capacity (`Finish::Capacity` stays for old CSVs).
- Nothing is copied at the end: `take_ready` returns a `Summary` (id, reason,
  skipped, event_count); the file gets its footer, flush and sync. The header
  is written when the capture arms, so a full card or name clash refuses the
  capture up front ("timing capture could not open …").
- CSV layout: `# engine`, `# reason`, `# skipped`, `# event_count` move from
  the header to the footer before `# complete=1`. `tools/analyze_render_trace.py`
  already reads metadata lines anywhere and only requires the final line, so
  it needs no change; a host test checks the streamed bytes equal the
  single-shot `write_csv`.
- Release profile `opt-level = "s"` (measured in a worktree on this commit's
  parent: `.text` 1590896 → 1399936 B, NRO 2183168 → 1998848 B; `"z"` would
  reach 1307376 B; fat LTO changes nothing with `codegen-units = 1`). `"s"`
  was chosen to keep loop codegen in hooked paths.

Gates: 300 host tests (4 trace tests rewritten, 2 new), host and Skyline
Clippy with warnings denied, no-default build, fmt, Skyline release. Package
`artifacts/6b22546e0a057901/`: build `0.3.5-ee936bc148d5-1791930600`, NRO
SHA-256 `6b22546e0a0579011f37c3ff3780e7a9f9955226195424ff02665ad142a65dfb`
(2002944 B, module `04397ac6…380a`), ELF `be8a3ca5…4c50`; sections `.text`
1401296 / `.rodata` 342848 / `.data` 179360 / `.bss` 286128 (two 4096-row
chunks), idle resident 2.21 MB (0.3.4: 2.23 MB; 0.3.2: 3.99 MB). A running
capture adds about 190 KB of heap (one-chunk scratch plus the 64 KiB file
buffer) where 0.3.3/0.3.4 added 2 MiB plus a 2 MiB copy at export.

Switch install of 0.3.5, 2026-10-13 ~23:25Z (at the operator's request, with
the SD card connected over USB): hekate USB mass
storage, tangent `switch-ums-6b22/deployment.json`. Installed NRO read back
as e930c8f3…25f3 (0.3.3) before, saved to
`backups/pre-stream-trace-6b22546e0a057901.nro` on the card (verified) and
locally; copied under a temp name, renamed, read back as 6b22546e…5dfb;
other plugins untouched (the Training Modpack on the card is now a
2174976 B build, changed by its own agent). Runtime result pending.
Forgejo release `v0.3.5` (id 359, tag on 08eedf1): NRO asset 76 (download
read back as 6b22546e…5dfb), `manifest.json` asset 77.

0.3.6 (requested by the operator) adds the measurement the
memory investigation lacked: every hang-trace heartbeat carries the kernel's
accounting for the game process, `mem_used`, `mem_total`, `mem_free` and
`heap_region` in bytes, read with `svcGetInfo` (TotalMemorySize,
UsedMemorySize, HeapRegionSize on the current process; inline assembly beside
the existing `QueryMemory` in `runtime/memory.rs`). The diagnostics thread
refreshes the three values every 500 ms into atomics; the Debug view's
Performance table shows them as a `Memory` row (`used/total MiB, free`).
No hook changed; the UI path makes no system call. Because the same call
works under Eden, the console and the emulator can now be compared for
headroom at title, character select, Training and the moment a boot fails.
Host tests cover the field formatting and MiB label (301 tests). Gates as
for 0.3.5. Package `artifacts/792d88778925c0e6/`: build
`0.3.6-c45b2f91b4ee-1791935605`, NRO SHA-256
`792d88778925c0e6963cf5bead29b1158f73f1b535c59948ae20fef93938c5e4`
(2002944 B, module `18fe2ed2…7101`), ELF `9dbc04b1…b5b4`; sections `.text`
1402016 / `.data` 179368 / `.bss` 286160 (resident 2.21 MB, unchanged).
Forgejo release `v0.3.6` (id 360, tag on ec2847f): NRO asset 78 (download
read back as 792d8877…c5e4), `manifest.json` asset 79.

Switch install of 0.3.6, 2026-10-14 ~00:40Z (at the operator's request): hekate USB mass storage, tangent `switch-ums-792d/deployment.json`.
Installed NRO read back as 6b22546e…5dfb (0.3.5) before, saved to
`backups/pre-memory-probe-792d88778925c0e6.nro` on the card (verified) and
locally; copied under a temp name, renamed, read back as 792d8877…c5e4. The
plugin set on the card at that moment: ARCropolis 2076672, css_preserve,
nro_hook, param_hook 53248 each, gamepad overlay 151552, OBS companion
126976 (down from 212992), Deluxe 2002944, Training Modpack 1769472 (down
from 3379200), stage pack 1351680, imgui 1282048. Runtime result pending:
the first boot's heartbeats give the first `mem_used`/`mem_total` numbers.

0.3.7 (requested by the operator) trims what the plugin holds
while the game runs, after a symbol-level pass over the 0.3.6 ELF:

- Diagnostics thread stack 2 MiB → 256 KiB (`logging::DIAGNOSTICS_STACK_BYTES`).
  The Switch std (`sys/thread/switch.rs`) gives every Rust thread a 2 MiB
  pthread stack from the game heap unless the builder sets one, and Deluxe
  never did. The thread's deepest frames are the file writers; its buffers
  (64 KiB `BufWriter`, drain scratch, log lines) are heap. About 1.75 MiB of
  heap back for the whole run. Hardware check: heartbeats keep arriving in
  `hang-trace.log` and no crash report names `deluxe-diagnostics`.
- Pia RTT station table (`pia_rtt::Session`, 253 slots × 100-sample ring,
  about 107 KB, non-zero initialised so it sat in `.data`) is no longer a
  static. (Superseded 2026-10-17: the table is sixteen keyed slots, about
  7 KB, and is a plain static again; `WANT_SESSION`, `new_boxed` and the
  dropped first sample are gone. See the `pia_rtt.rs` cut entry under 0.5.3.) `runtime::pia_rtt::STATE` is `Mutex<Option<Box<Session>>>`: the
  first RTT sample of the process sets `WANT_SESSION`, the next diagnostics
  poll (≤ 500 ms) allocates with `Session::new_boxed` (zeroed heap block plus
  three scalar writes; equality with `Session::new()` is tested) and the table
  stays for the rest of the process. Until then the sample hook counts its
  sample as dropped, the leave and cleanup hooks have nothing to clear, and
  the poll publishes the unavailable reading and empty board an empty table
  would (`missing_table_publishes_like_an_empty_one`). Offline the table
  never exists. `.data` 179368 → 69912.
- API trace chunks 4096 → 2048 rows (`render_trace::CHUNK`, 80 KiB `.bss`); a
  chunk still holds about 1.9 s at the 1,100 rows/s measured on the hardware
  CSS capture against the 500 ms drain.
- Callback capture buffer (`timing::Capture`, 2048 × 32 B) is allocated by
  `begin` on the diagnostics thread (`try_reserve_exact`; a refused allocation
  refuses the capture) and moved into the snapshot by `take_ready`, freed
  after the export. The render path still never allocates. `.bss` 286160 →
  138672.

Not changed here: the embedded imgui backend (about 545 KB of `.text` plus
demo windows, fonts and glyph tables), which is being trimmed in the Forgejo
fork `intpa/imgui-smash` (tangent `imgui-fork-1014/`) for both the embedded
archive and the standalone companion plugin.

Gates: 303 host tests (two new), host and Skyline Clippy with warnings
denied, no-default build, fmt, Skyline release. Sections `.text` 1402336 /
`.rodata` 343136 / `.data` 69912 / `.bss` 138672, resident image 1.95 MB
(0.3.6: 2.21 MB); with the thread stack the steady-state cost drops by about
2.0 MB, more than the whole 0.3.1 regression. Package
`artifacts/8562746a292942cf/`: build `0.3.7-61d022bdc3db-1791945438`, NRO
SHA-256 `8562746a292942cf23df19f9e6781cc37b623dda305337161b730e14813707ea`
(1896448 B, module `c4beb790…d5f7`), ELF `55b8208c…cf9f0`.

Switch install of 0.3.7, 2026-10-14 02:38Z (at the operator's request; the
card was still mounted over hekate USB mass storage from the
0.3.6 install): tangent `switch-ums-8562/deployment.json`. Installed NRO read
back as 792d8877…c5e4 (0.3.6) before, saved to
`backups/pre-runtime-trim-8562746a292942cf.nro` on the card (verified) and
locally; copied under a temp name, renamed, read back as 8562746a…07ea
(1896448 B). Other plugins untouched. Deluxe's log files as they were before
the install are in `switch-ums-8562/before-logs/`; they show the console has
not booted Smash since the 0.3.2 build (`hang-trace.log` and `install.log`
both begin `build=0.3.2-85a75632…`, newest crash report 21:18Z on 2026-10-13),
so 0.3.3, 0.3.5 and 0.3.6 never ran on hardware either. Runtime result pending:
the first boot decides the 256 KiB stack (heartbeats continuing in
`hang-trace.log`) and gives the first `mem_used` numbers of the trimmed build.
Forgejo release `v0.3.7` (id 363, tag on d41ad35): NRO asset 80 (download
read back as 8562746a…07ea), `manifest.json` asset 81.

Hardware result, 2026-10-14 (tested by the operator; everything booted and
worked): the console booted with 0.3.7 and the current plugin
set, the first boot since the 0.3.2 build, so this is the first hardware run
of 0.3.3–0.3.7 together, including the 256 KiB diagnostics stack. Heartbeat
continuity and the first `mem_used` numbers are to be read from the card's
`hang-trace.log` at the next connection.

0.3.8 (requested by the operator: fork ImGui to Forgejo, look for savings
there, replace the standalone with the fork once tested, and ship when the
ImGui work is done) moves the ImGui backend to
the Forgejo fork `intpa/imgui-smash` (private; `main` = upstream `bb47b205`,
branch `deluxe` = renderer correction + companion crate + memory trims, head
`7f0e47c`; details and the deferred-trim table in `docs/IMGUI_DEPENDENCY.md`
and tangent `imgui-fork-1014/REPORT.md`). Archive 17,181,302 → 13,851,556 B
(`8a06be16…`); `archive-pin.json` points at the fork and applies no patch
(`prepare_imgui.py` keeps the patch step optional; `tools/test_prepare_imgui.py`
updated). Deluxe's own code is unchanged; the embedded backend loses the demo
and metrics windows, the debug tools and two unused fonts. Sections `.text`
1402336 → 1179744, `.rodata` 343120 → 225200, `.data` 69912 → 48864, `.bss`
138672 → 131568; resident image 1.95 → 1.59 MB. The standalone companion
built from the same fork commit is 888,832 B (was 1,282,048;
`a3f794b2d2af15d5f2c2868f22e81c865702524160640864604731b7afe6775d`), with the
same export set, the `1.92.7 WIP (19263)` tag and `text_offset == 0`, so the
external-host detection is unchanged.

Eden acceptance, 2026-10-14 (isolated profile, private display `:2`, Astra
low driver `imgui-fork-eden-driver-1014`, evidence `imgui-fork-1014/eden-test/`):

- External (trimmed standalone + Elite, `ext-1-shots/`): `install.log`
  `overlay_host_external_chain` → `overlay_host_external_ready`; Deluxe Debug
  view, settings view and FrameTime graph render cleanly on the main menu and
  the character select; Elite's `L+R+Up` menu lists its 11 stages cleanly;
  60 fps, CSS 59.85; no crash, no assert, settings unchanged.
- Embedded (no standalone, `emb-2-shots/`): `overlay_host_bootstrap_hooked`
  → `overlay_host_ready`; Debug and settings views clean; CSS 59.99 fps.
- Reversed order (standalone renamed `libzz_imgui_smash.nro` so it sorts
  after Deluxe, plus Elite, `rev-3-shots/`): `overlay_host_external_ready`
  again; Debug view, Elite's stage list and the CSS Debug view clean;
  CSS 60.01 fps; no crash or freeze.

The isolated profile's plugin set was restored exactly afterwards
(`eden-test/staging-journal.json`, `restore_verified`).

Gates: 303 host tests, host and Skyline Clippy with warnings denied,
no-default build, fmt, `tools/test_prepare_imgui.py`, Skyline release.
Package `artifacts/c247bd377a422023/`: build `0.3.8-5c97c9c935fe-1791974104`,
NRO SHA-256 `c247bd377a4220233c4bf56470d23a6ea267b752b99cfd02eb3b26b480557222`
(1531904 B, module `34009bd1…251b`), ELF `ef694416…cb3b`; `imgui-provenance.json`
records the fork commit and the `8a06be16` archive.

Switch install of 0.3.8 plus the trimmed standalone, 2026-10-14 10:36Z
(at the operator's request, once the ImGui work was done, with the Switch
connected over USB): DBI USB MTP, two-target transaction
`switch-mtp-038/deploy-038-mtp-1014.py` (preflight backed up every plugin and
the protected files; `deployment.json` `installed_verified`). Deluxe read back
as 8562746a…07ea (0.3.7) before, saved to
`backups/pre-imgui-trim-c247bd377a422023.nro`, replaced by copy from a staged
read-back, read back as c247bd37…7222; `libimgui_smash.nro` read back as
9a9e575c…05ed before, saved to
`backups/pre-imgui-trim-standalone-9a9e575c910c6471.nro`, replaced, read back
as a3f794b2…775d (888832 B). Plugin names and every other plugin unchanged.
Runtime result on hardware pending. Before the install the card's logs
(`switch-mtp-038/before-logs/`) showed the 0.3.7 boot: 1135 s, 153 heartbeats
(the 256 KiB diagnostics stack held), kernel probe `mem_used` 1,764,012,032 at
the title → 2,536,943,616 in Training of `mem_total` 3,444,572,160
(`mem_free` 907,628,544), and four complete CSS captures (ids 2570630842
Vanilla, 4928335010 LessLag with 1775 skipped rows, 10646616362 Mixed,
18255281875 Vanilla), handed to the Astra xhigh analysis
`css-hw-037-analysis-1014` (`css-budget/hw-0.3.7-1014/`). The operator still
sees frame-rate dips on the character select.
Forgejo release `v0.3.8` (id 364, tag on 45e79af): NRO asset 82 (download
read back as c247bd37…7222), `manifest.json` asset 83.

Hardware CSS analysis of the 0.3.7 boot, 2026-10-14 (Astra xhigh
`css-hw-037-analysis-1014`, read-only; report
`css-budget/hw-0.3.7-1014/ANALYSIS.md`, reproduction scripts and per-callback
CSVs beside it, inputs hashed in `inputs-sha256.json`). Findings:

- The footer engine label compares arm with export only; every capture but D
  contains renderer round trips (A: Vanilla → LessLag 5.7 s → LLUltra 15.8 s →
  LessLag 23.3 s → Vanilla 24.8 s; B: LessLag → Vanilla 9.5 s → LessLag
  22.5 s; C: Vanilla → LessLag 29.3 s), and B, C and D leave the CSS during
  the 30 s. Whole-file averages are therefore not a Vanilla-versus-LessLag
  benchmark; the stable interiors are.
- In genuine two-image LessLag on the CSS the dips are **texture-ready fence
  stalls after acquire**, not long acquire or present calls and not backoff
  pickup. Capture A's continuous CSS visit: Vanilla zero-input 60.00 fps with
  1.93 ms texture wait; LessLag zero-input 52.55 fps / 3.49 ms; LessLag with
  directional input 41.35 fps / 7.67 ms; Vanilla again 60.00 fps / 1.79 ms.
  The directional increment of 5.15 ms per callback is 4.18 ms texture wait
  and 0.81 ms main UI barrier (81 % of the increment is the fence). B's late
  genuine LessLag CSS sits at a 30.00 fps plateau with 16.8 ms texture wait
  per 33.3 ms callback, with or without input. A's 30 fps plateau at
  15.8–23.3 s is LLUltra, not LessLag evidence.
- Backoff: no nested dispatcher wait (kind 66) overlaps any UI job; dispatch
  → job 22–23 µs mean, job return → barrier return 32–34 µs mean, largest
  single sleep 189 µs. Leave backoff and clocks unchanged.
- Separate mechanisms, not the CSS plateau: 290–300 ms UI-job stalls paired
  with CSS departure (B/88, C/668, D/244, D/1398), 1.3 s uninstrumented
  hitches outside the CSS (B/313, D/466), and 12–56 ms "other infinite sync"
  waits in Vanilla outside the CSS. D's second CSS visit dips with short
  texture waits but longer UI wall and flush, so Vanilla is not hitch-free and
  is not a menu fallback.
- Streaming loss: both API holes start exactly on a 2048-row chunk boundary
  (A 25.58–25.88 s, B 6.33–7.71 s), so the loss is drain/overflow on the
  diagnostics loop, not callback loss (timing has zero skips in all four).
- Memory: process `mem_used` 2177–2462 MiB of 3285 MiB during the captures,
  free 823–1108 MiB; liveness, not a bandwidth or fragmentation measurement.

Ranked next steps, all in existing visual-request hooks and keeping genuine
two-image LessLag: (1) measurement integrity first (latch engine/scene
changes per capture, separate lock-contention from overflow skips, report
drain time and chunk high-water marks, collapse redundant per-frame field
rows); (2) `PROBE_RENDER_POLL` discriminator at the render-wait hook
main+0x387029c (restore the original zero-timeout poll while keeping two
images) as a matched baseline→probe→baseline experiment; (3) hide the
Settings overlay as a control (every capture had it visible); (4)
`DISABLE_LESSLAG_FIELDS` discriminator at main+0x384fa10, separately; (5)
frame/sync identities on the NVN wrappers to find the producer of the
pending fence. None is a shipping fix yet; no LLDoubles, Vanilla-on-menus,
clock or memory changes are recommended for the dips. Follow-up capture
acceptance: one CSS visit, 10 s still then 20 s repeatable directional
sweep, zero API overflow, engine and scene constant.

0.3.9 (requested by the operator: the draw-slot trims as well, tested and
shipped with Astra's findings) takes the first deferred fork trim. Fork commit
`c185303` (pin `77cdab7`): each of the backend's eight draw slots held a
256 KiB + 256 KiB command pool and a 64 KiB + 64 KiB control block of game
heap (5 MiB in all) for recordings that never exceeded 1,256 B of command
memory or 120 B of control memory across the stored Eden evidence. The
budgets are now 32 KiB + 32 KiB command memory (exactly one 64 KiB NX
memory-pool page) and 8 KiB + 8 KiB control memory per slot, 640 KiB in all;
the reserve still finishes and discards one oversized frame, and exceeding
base plus reserve remains an abort, which is why the Eden runs below read
the live counters. Budgets and rationale moved to
`command_memory_reserve.hpp`; `command_memory_reserve_test` asserts the
planned layout on the NX constants (pool exactly 65536, control 16384, also
on a 4 KiB page and with device minimums). Archive
`09b296a0f5c67641d229bb4a6d8b170f75281368da3957d666af424476bc2731`,
13,851,844 B, identical in two clean Docker builds; `archive-pin.json`,
`build.rs` and `tools/test_prepare_imgui.py` follow. Deluxe's own code is
unchanged. Standalone companion from `77cdab7`: 888,832 B,
`946de84f98546cb791640e40fbf5848ea3965c511631ef18b32e5c805b67d50e`, same
segment layout, exports, version tag and `text_offset == 0` as the 0.3.8
companion; a `draw-stats` build (`ef0e57bf…2c04`, 905,216 B, not shipped)
adds the 2 s logger thread that writes `imgui-renderer-status.txt`.

Eden acceptance, 2026-10-14 (isolated profile, private display `:2`, Astra
low driver `imgui-slots-eden-driver-1014`, brief and evidence in
`imgui-fork-1014/eden-test/`, `slots-N-shots/NOTES.md` per run):

- Run 1, external with the draw-stats companion + Elite (`slots-1-shots/`):
  `overlay_host_external_ready`; Deluxe Debug/Settings/Performance and
  Elite's stage list open together on the main menu and on the CSS, then a
  Shulk vs Wolf match on Small Battlefield with the compact strip (20 s,
  pause, quit, results, CSS return), all clean at 59.9–60.0 fps. Final
  `imgui-renderer-status.txt`: 29,658 submissions, 29,650 reuses, 0 pending
  skips, 0 failed, 0 geometry skips, **0 command overflows, max 1,448 B
  command / 120 B control** (a new maximum, 4.4 % of the 32 KiB initial
  budget). A shared Right press during the quit prompt flipped Stealth on;
  the driver restored it (config.ini `stealth=false`, `config.previous.ini`
  shows the transient). Eden hit its 8 GiB host cgroup cap during the match
  (`memory.events max 2470`, no OOM kill) and the game stayed at 60 fps;
  the unit's SIGBUS core-dump at `systemctl stop` is the same teardown
  pattern as all three 0.3.8 runs.
- Run 2, embedded (`slots-2-shots/`): `overlay_host_ready`; all views on
  the main menu and the CSS, three portrait hovers with Settings open,
  60.0 fps. Final `renderer-status.txt`: 13,201 submissions, 13,193 reuses,
  3 busy-frame rejections, 0 pending/failed/geometry/overflow, max 1,040 B
  command / 120 B control. Clean exit at stop, peak 6.25 GB.
- Run 3, reversed order (release companion as `libzz_imgui_smash.nro` +
  Elite, `slots-3-shots/`): `overlay_host_external_ready` again; Debug,
  Performance and Settings each open beside Elite's stage list on the main
  menu and on the CSS, Elite cursor moved Up/Down only, 59.95–60.0 fps;
  all settings unchanged (`config.ini` `stealth=false`); no crash, assert
  or window loss. The release companion has no counters; its slot code and
  archive are the ones measured in run 1.

The profile's plugin set was restored exactly afterwards
(`eden-test/staging-journal.json`, `restore_verified` 12:40Z).

Gates: 303 host tests, host and Skyline Clippy with warnings denied,
no-default build, rustfmt, `tools/test_prepare_imgui.py`; fork host tests
(`command_memory_reserve_test`, `draw_slot_rules_test`, `frame_guard_test`,
`frame_guard_concurrent_test`) with warnings denied; fork crate Clippy and
no-default build; Skyline release. Package `artifacts/8c20799f3974f45b/`:
build `0.3.9-4fd77a238f9e-1791980278`, NRO SHA-256
`8c20799f3974f45beaf28c2373982bcc546f08f6496ee58ed95852fd78496cca`
(1531904 B, module `8910f609…628b`), ELF `871d513f…d2fe`;
`imgui-provenance.json` records fork commit `c185303` and the `09b296a0`
archive. Fork branch `deluxe` (head `77cdab7`) and tag `deluxe-0.3.9` are
pushed to Forgejo; fork release `deluxe-0.3.9` (id 366) carries
`libimgui_smash.nro` (asset 88, read back as 946de84f…d50e) and
`libimgui_smash.a` (asset 89, read back as 09b296a0…2731).

Switch install of 0.3.9 plus the new standalone, 2026-10-14 12:41Z (same
order): DBI USB MTP, two-target transaction
`switch-mtp-039/deploy-039-mtp-1014.py` (preflight 12:41:22Z backed up every
plugin and the protected files; `deployment.json` `installed_verified`
12:41:56Z, 27 device commands, all exited). Deluxe read back as
c247bd37…7222 (0.3.8) before, saved to
`backups/pre-draw-slots-8c20799f3974f45b.nro`, replaced by copy from a
staged read-back, read back as 8c20799f…6cca; `libimgui_smash.nro` read back
as a3f794b2…775d before, saved to
`backups/pre-draw-slots-standalone-a3f794b2d2af15d5.nro`, replaced, read
back as 946de84f…d50e (888832 B). Plugin names, every other plugin, config.ini
and hekate_ipl.ini unchanged. Runtime result on hardware pending (the 0.3.8
install of 10:36Z had not booted yet either; the card's `hang-trace.log`
still ended with the 0.3.7 boot). Forgejo release `v0.3.9` (id 367, tag on
01132f5): NRO asset 90 (download read back as 8c20799f…6cca),
`manifest.json` asset 91.

Caveat for the CSS capture: the SD writes now happen during the measured
interval, from the diagnostics thread. Compare the next hardware CSS capture
against `switch-0.3.1-1013` before trusting a movement-penalty number from it.

This is a footprint change, not a
proven fix: the next console boot with 0.3.3 and the companion plugin rolled
back decides; the memory bisect is plugin removal, not Deluxe flags.

### LessLag render-poll candidate — 2026-10-14 (commit b044752, no version, no toggle)

The operator asked for the fix Astra proposed for the FPS drops. It was to be
built without a version, proven in Eden and on the Switch, and then tagged as
a new version. It has no toggle: it becomes the standard if it helps and is
removed if it doesn't.
`render_policy::skip_wait` now skips the game's timeout-zero render poll at
main+0x387029c only for LLUltra and the Doubles shapes; two-image LessLag
(and LessLagCore) run the original poll like Vanilla. `PROBE_RENDER_POLL`
still forces the poll for the policies that skip it. 304 host tests, host
and Skyline Clippy with warnings denied, no-default build, rustfmt, Skyline
release. Package `artifacts/021a03dee556fdc4/`: build
`0.3.9-b04475229832-1791982758`, NRO SHA-256
`021a03dee556fdc45bece92ab320c0f7e94f08d8449facfe421d2b7489015164`
(1531904 B). Crate version stays 0.3.9; no tag, no release.

Eden A/B (tangent `css-budget/render-poll-ab-1014/`: `PROTOCOL.md`,
`SUMMARY.md`, `RESULTS.md`, `compare.py`, `arms.json`, `captures/`,
`shots-*/`): isolated profile, embedded host, Deluxe alone, FX On, overlay
hidden, private display `:2`, Astra low drivers; Eden's systemd unit under
`CPUQuota` 175 % then 120 % with a 5 ms period; one 30 s `CAPTURE_TIMING`
window per arm, 10 s still then a 20 s left-stick sweep, on the
Training-mode character select with LessLag live (two textures confirmed in
`renderer-status.txt`). Findings:

- At 175 % neither build dips (0.3.0's backoff removed the CPU waste the
  earlier campaign measured there). At 120 % both dip: callback intervals
  over 20 ms were 8.7 % (A1, 0.3.9), 2.8 % (B1, candidate), 3.5 % (A2,
  0.3.9), 4.6 % (B2, candidate); cadence 55.8 / 58.5 / 58.1 / 57.7 fps.
  The candidate sits inside the reference spread: **no regression, no
  measurable gain on Eden**.
- The poll costs 0.001 ms per frame and returned ALREADY_SIGNALED in 5,276
  of 5,277 frames; Eden's texture-ready wait is 0.03 ms in every arm, so the
  emulator never reaches the fence stall the change targets (hardware
  Vanilla returned TIMEOUT from the same poll in 8–62 % of frames).
  The decision is the console's.
- Side finding: the regular Smash character select runs **Vanilla** after
  any Singles/Doubles/Training-classified scene until the profile is
  re-selected (`auto_policy::transition_cleanup` cleanup hold; the Smash
  submenu classifies as Doubles before the CSS, so a fresh boot → Smash →
  CSS is Vanilla as well). Hardware capture D matched this. LessLag CSS
  observations therefore come from re-selected or Training CSS visits.

Switch install of the candidate, 2026-10-14 13:49Z (the operator asked for
proof in Eden and on the Switch): DBI USB MTP, `switch-mtp-poll/deploy-poll-mtp-1014.py`
(`preflight.json` 13:49:38Z, `deployment.json` `installed_verified`
13:49:56Z). Deluxe read back as 8c20799f…6cca (0.3.9) before, saved to
`backups/pre-render-poll-021a03dee556fdc4.nro`, replaced by copy from a
staged read-back, read back as 021a03de…5164; standalone, other plugins,
config.ini and hekate_ipl.ini unchanged. The card's `hang-trace.log` and
`install.log` still end with the 0.3.7 boot, so the next console boot is
the first for 0.3.8, 0.3.9 and this candidate together. Hardware test:
LessLag on the Training CSS (Games & More → Training → start → Quit
Training → CSS) and on the regular CSS after re-selecting LessLag;
`CAPTURE_CSS` is still on the card, so each CSS visit records 30 s; compare
the kind-3 texture wait and the > 20 ms share with
`switch-mtp-038/before-logs/` (0.3.7 LessLag windows 41–52 fps, B 30 fps).
If it does not help, revert b044752 and reinstall 0.3.9 from the rollback.

**Hardware verdict, 2026-10-14 14:37Z: the candidate does not help, and it
is reverted (commit c0fa483) and off the card.** The console booted the
candidate (`switch-mtp-poll/boot-logs-1437/`, build
`0.3.9-b04475229832-1791982758`, 1110 s, `mem_used` 2.66 GB) and recorded
four CSS captures. The event log (`AutoRequest` code 74, `ProfileTransition`
code 46) dates them: capture 298481440895 at 81 s is genuine two-image
LessLag with the poll running, the operator still (0–11 s) then moving the
hand (12–30 s); 304026704152 at 370 s is LessLag after two Vanilla↔LessLag
re-selections; 312302874387 at 801 s is Vanilla under the cleanup hold
(`req=0 sel=3 hold=1` at 775.6 s); 313596223198 at 868 s starts under the
hold and commits LessLag at 877.7 s when the profile was re-selected.
Numbers (`RESULTS-hw.md`, `compare.py`):

| Capture | Live policy | Still: fps / > 20 ms / texture wait | Sweep: fps / > 20 ms / texture wait mean / p95 | Poll rows / results |
|---|---|---|---|---|
| 298481440895 | LessLag + poll | 59.4 / 1.0 % / 2.35 ms | **49.9 / 20.2 % / 4.56 / 16.4 ms** | 1593 / all 0 |
| 304026704152 | LessLag + poll | 50.6 / 18.5 % / 4.22 ms | 48.9 / 19.7 % / 5.72 / 16.8 ms | 1483 / all 0 |
| 312302874387 | Vanilla (hold), no input | 59.3 / 0.8 % / 1.70 ms | 58.5 / 0.4 % (no input) / 3.40 / 12.1 ms | 1590 / all 0 |
| 313596223198 | Vanilla → LessLag at 9.5 s | 52.6 / 4.9 % / 5.85 ms | 58.6 / 1.9 % / 4.59 / 10.7 ms | 1690 / 1688×0, 2×2 |

The poll returned ALREADY_SIGNALED in every LessLag frame (the sync it
checks is already complete in a two-image ring), so running it changes
nothing: the directional-input penalty is unchanged from the 0.3.7 captures
(LessLag windows 41–52 fps) and the texture-ready fence tail is still 16 ms.
Astra's rank-1 discriminator is therefore answered: removing the poll is
**not** what makes the fence miss. The remaining candidates from that list
are the hidden-overlay control, the `DISABLE_LESSLAG_FIELDS` discriminator
and frame/sync identities on the NVN wrappers. The card was returned to the
released 0.3.9 NRO at 14:39Z (`switch-mtp-poll-revert/deployment.json`
`installed_verified`; the candidate is archived as
`backups/candidate-render-poll-021a03dee556fdc4.nro`).

Display defect found by the operator on this boot: the native CSS banner
read "You: LessLag | 3" while the renderer ran Vanilla with three textures
under the cleanup hold (the menu's Engine row showed Vanilla). Manual mode
labelled the saved profile regardless of the live request. Fixed in
`auto_policy::profile_row_label`: a manual selection whose request is Vanilla
(only the cleanup hold produces that) is shown as `Vanilla*` in the banner,
the arena lines and the settings row; tests updated. Not yet on the card.

### Present-at-flush candidate — 2026-10-14 (commit 8687eee, no version, no toggle)

After the render-poll verdict, the operator asked to try letting the UI job
overlap the previous frame's GPU work instead of serialising behind it.
The console capture `switch-mtp-poll/boot-logs-1437/` (genuine LessLag)
gives the main thread's frame order relative to the game's submit: UI
dispatch 0.2 ms → UI job 0.2–10.7 ms (13.2 ms with the hand moving) with the
main barrier on it → other dependency waits to 13.7 ms → **present of the
frame submitted at 0 ms at 13.7 ms** (16.3 ms moving) → acquire 14.2 →
texture-ready wait to the vsync at 16.4 ms (fast) or **33.1 ms (slow)** →
late flush and flush with the next submit inside its span (1593 of 1595
frames). So the UI job already overlaps the GPU; the stall is the present
being issued only after the next frame's whole CPU update, so that a
hand-moving frame presents after the vsync and the two-image acquire waits
for the following one. Present index always equals the preceding acquire
(1592 of 1592).

Change: in committed idle two-image LessLag the flush hook
(`renderer.rs`) presents the acquired image at the end of the flush
(`nvn::present_at_flush`); the game's own present of that index, 13.7 ms
later, is skipped while its loop bookkeeping still runs, and the
per-present crop moves with the real present. `render_policy::
present_at_flush` gates it (not during a swap, drain, forced wait or after
a refusal; Vanilla, Ultra and Doubles keep the game's order). Counters
`early_presents`, `early_present_skips`, `early_present_mismatches` in
`renderer-status.txt`; code 105 records a mismatch. Trace: early present
= kind 1 result 2, skipped game present = kind 1 result 1. Build
`0.3.9-8687eee688c9-1791989848`, NRO
`aa214ec238e680b3a714837b7142351c1e567a1406762171696e96e98933a586`
(package `artifacts/aa214ec238e680b3/`). 305 host tests, Clippy host and
Skyline, no-default, fmt, Skyline release.

Eden P1, 2026-10-14 15:03Z (tangent `css-budget/present-at-flush-1014/`,
same protocol and 120 % quota as the render-poll arms, Training CSS with
Mario selected, Astra low driver): 1,786 early presents, 1,786 skipped
game presents, 0 mismatches; screenshots clean (one banner-entrance frame
with white sparkle quads, gone in the next frames; to be rechecked).
Early present lands 0.17 ms after the submit, the skipped game present
11.4 ms after it.

| Arm (120 %, whole 30 s) | Cadence fps | p95 ms | p99 ms | > 20 ms |
|---|---:|---:|---:|---:|
| A1 0.3.9 | 55.83 | 31.2 | 33.2 | 8.7 % |
| A2 0.3.9 | 58.13 | 18.3 | 32.6 | 3.5 % |
| B1 render-poll | 58.53 | 18.0 | 32.1 | 2.8 % |
| B2 render-poll | 57.67 | 19.5 | 32.7 | 4.6 % |
| **P1 present-at-flush** | **59.54** | **18.0** | **22.0** | **1.5 %** |
| **P2 present-at-flush** | **59.87** | **16.9** | **19.2** | **0.7 %** |

The 33 ms mode is gone on Eden in both arms: the first candidate whose p99
is not the doubled vsync. P2 (15:10Z, full Training path, Mario selected)
counted 13,634 early presents, 13,633 skipped game presents and 0
mismatches over the whole boot; sweep window 59.8 fps, 0.6 % > 20 ms,
p99 18.9 ms. Sweep coverage confirmed from the button column in both arms.

Switch install, 2026-10-14 15:06Z: DBI USB MTP, `switch-mtp-flush/
deploy-flush-mtp-1014.py` (`preflight.json` 15:05:43Z, `deployment.json`
`installed_verified` 15:06:01Z); Deluxe 8c20799f…6cca (0.3.9) → aa214ec2…
a586, rollback `backups/pre-present-flush-aa214ec238e680b3.nro`; other
plugins, config.ini and hekate_ipl.ini unchanged. Hardware test: LessLag
CSS (Training CSS, or re-select LessLag on the regular CSS) with hand
movement; `CAPTURE_CSS` records each visit; compare the kind-3 wait and the
> 20 ms share with `switch-mtp-poll/boot-logs-1437/` (candidate b044752,
same hardware day) and `switch-mtp-038/before-logs/` (0.3.7). Watch for
flicker, tearing or a stale frame on the CSS and in a match. If it does
not help or shows artifacts: revert 8687eee and reinstall the rollback.

### Match capture (`CAPTURE_MATCH`) — 2026-10-14 (commits aa7ae78, 210f83f)

The operator asked for the match capture to be built and shipped. The finite
capture pair now records a
whole match: with `CAPTURE_MATCH` in the data directory at boot, every stage
load (stage-setup hook; Singles, Doubles, Training) arms `timing-<id>.csv` +
`api-<id>.csv` (label `match_*`), and the capture ends when the game leaves
the stage (`# reason=scene_end`): scene status back to Inactive, or the
character-select update running (210f83f; the Training CSS keeps the
Training status). Ten-minute ceiling (`elapsed`). Both CSVs stream from the
diagnostics thread, which polls every 250 ms while a capture runs; the
callback buffer is drained like the API chunks (the 2048-sample buffer bounds
nothing but a stalled drain), and API rows pack `begin` relative to a
per-chunk base (first row minus 2^31 ticks), so the 20-byte row survives
captures far beyond the former 223 s limit; a row outside its chunk's range
starts the other chunk instead of ending the capture. A 30 s capture still
running when a stage loads is ended for the match (code 106 `a=3`).
`CAPTURE_MATCH` also installs the three read-only `CAPTURE_CSS` hooks (rows
62–66, 77) without arming on CSS entry. Code 106; Debug row `Match capture`;
`tools/analyze_timing.py` accepts `scene_end` (duration check only for
`elapsed`). No hook does I/O; the render-path change is bookkeeping in
`Capture::observe` (first/last tick kept in the state instead of the buffer).

Eden smoke tests (isolated profile, embedded host, no CPU quota, Astra low
driver `match-capture-eden-driver-1014`, `css-budget/match-capture-1014/`):

- Run 1 (aa7ae78): Training stage load armed `capturing 15295074307251
  match_training` at 15:49:50Z; the files grew while the match ran (API CSV
  2.8 → 5.4 MB in the first 60 s); the Training exit to the CSS did **not**
  end it (status stayed Training), found and fixed in 210f83f. The partial
  files copied at 372 s (`runs/match-smoke-1/`): 607,954 API rows in strict
  sequence, no `end < begin`, no backward tick jump, begins 0.005–372.2 s
  after the id, kinds 1/2/3/5/12–17/29–34/60–64/66/77 (present rows twice
  per frame: the early present and the skipped game present), 21,447
  callback samples contiguous with increasing ticks over 358 s, one partial
  trailing line (copied mid-write). 60.0 fps throughout, no dialog or
  freeze.
- Run 2 (210f83f, `runs/match-smoke-2/`): two Training stage loads armed
  `match_training` captures 15306224507360 (15:59:32Z) and 15308367429258
  (16:01:17Z, re-entered from the Training CSS); both ended on the Training
  exit within the driver's 4 s poll (`# reason=scene_end`, events.log code
  106 `a=1`, 31, 106 `a=2 b=0xc55`, 32 for each), footers complete:
  159,579 API rows (6 skipped) + 5,715 callback samples over 96.3 s, and
  76,375 rows (1 skipped) + 2,741 samples over 45.9 s. `analyze_timing.py`:
  p50 16.67 ms, p99 17.7 / 17.6 ms, 26 / 5 intervals > 20 ms with the stage
  load and exit inside the window (max 347 / 227 ms). `analyze_render_trace.py`
  (now accepting `scene_end`): present rows with indices 0/1 and results
  1/2 only, so the present-at-flush path was live. 60 fps in Training, no
  dialog or freeze; the 47 fps sample at the second arm is the stage load.

Switch install, 2026-10-14 16:03Z: DBI USB MTP, `switch-mtp-match/
deploy-match-mtp-1014.py` (`preflight.json` 16:03:38Z, `deployment.json`
`installed_verified` 16:03:57Z); Deluxe aa214ec2…a586 (present-at-flush
candidate) → 4e4f2044…db56 (build `0.3.9-210f83fcc733-1791993370`, the same
candidate plus the match capture), rollback `backups/pre-match-capture-
4e4f2044edb3d3b5.nro`; `CAPTURE_MATCH` created in the data directory next
to the existing `CAPTURE_CSS`; other plugins, config.ini and hekate_ipl.ini
unchanged. Rollback chain on the card: 4e4f2044 → aa214ec2
(`pre-match-capture-…`) → 8c20799f 0.3.9 (`pre-present-flush-…`). Hardware
test: play matches with the flag; each stage load writes a `match_*` pair,
pull them with the CSS captures. No version tag until the console verdict.

### Console verdict and 0.4.0 — 2026-10-14 16:40Z

First boot of 4e4f2044 (16:11Z, config had drifted to Auto/Vanilla/2 during
the b044752 session via 23 menu saves the operator did not make) crashed on
MainThread in the game's `StandardAllocator::Initialize` during a scene
transition at t≈33–38 s, memory unmapped, no Deluxe frame in the chain,
Deluxe idle (`switch-mtp-match/boot-logs-1611/CRASH-1611.md`). Config
restored to manual LessLag, 3 frames, streamer off (readback verified);
the second boot ran 743 s with two online matches and did not crash.
The operator tested it: it booted and played smoothly, the FPS readout agreed,
and it was approved for release once the logs confirmed it.

Hardware captures (`switch-mtp-match/boot-logs-1640/`, build 210f83f, same
console as `switch-mtp-poll/boot-logs-1437/`):

| capture | scene / engine | fps | p99 ms | > 20 ms |
|---|---|---|---|---|
| b044752 day, css sweep (reference) | CSS LessLag + poll | 49.9 | 33 | 20.2 % |
| 482801144422 css, sweep 10–30 s (1,169 directional samples) | CSS LessLag | **59.85** | **20.2** | **1.3 %** |
| 482801144422 css, still 0–10 s | CSS LessLag | 60.00 | 16.8 | 0.0 % |
| 484909232936 match_singles, 148 s | online match, LessLag | 59.66 | 16.9 | 0.4 % |
| 489614260886 match_singles, 263 s | online match, LessLag | 59.88 | 16.8 | 0.2 % |
| 484546224171 / 489237967027 match_training, 17 s each | Training incl. load | 53.3 / 53.4 | 83 / 80 | 5.3 / 5.1 % |
| 488410188899 css (cleanup hold, re-selected mid-capture) | CSS Vanilla → LessLag | 52.7 | 83 | 5.7 % |

Texture-wait p95 on the LessLag CSS sweep fell from 16.4 ms to 2.95 ms;
every LessLag frame shows the early present (result 2) and the skipped game
present (result 1), 0 mismatches (no code 105). The match frames over 20 ms
are the stage load at the start of each capture (max 249 / 139 ms). The
Training captures are 17 s long and dominated by the load. The second CSS
capture ran under the cleanup hold (Vanilla, 270 native presents) until the
profile was re-selected mid-capture, so it is not a LessLag measurement; it
confirms the `Vanilla*` state exists on the console after a match. The
`Mixed` engine label on the match captures was the export reading the
engine after the results-screen cleanup hold, fixed by sampling the engine
per poll (this release). Peak memory 2.50 GB of 3.44 GB.

Open items from this boot: the opponent window stays after the opponent
declines a rematch (operator report; `peer_transport::frame_from` keeps
`Selection::Station` while the observer view still says known); the scene
connection stayed `offline` through two online matches although the latency
hook fired (code 30), so Auto would have classified them offline Singles.

Release 0.4.0: present-at-flush (8687eee), `Vanilla*` banner (4f8bcfb),
match capture (aa7ae78, 210f83f), engine label fix. Tag `v0.4.0` on 1224ea0,
build `0.4.0-1224ea0f3521-1791998073`, NRO
`04c3008d0182a48eb81ede39ec9f3eec7f380adcc187198b66aacf6a494c6afb`
(1,531,904 B), module id `1bf822fd…dd82`, package `artifacts/04c3008d0182a48e/`;
Forgejo release 370 with the NRO and manifest, download readback verified.
Card: `switch-mtp-040/deploy-040-mtp-1014.py`, 4e4f2044 → 04c3008d, rollback
`backups/pre-0.4.0-04c3008d0182a48e.nro`, `CAPTURE_CSS` + `CAPTURE_MATCH`
kept, config manual LessLag / 3 frames.

### 0.4.1 — 2026-10-14: the three follow-ups from the 0.4.0 console boot

The operator asked for these fixes to be built and the minor version installed
over USB, and kept playing to watch for FPS drops.

- **Opponent window after a declined rematch.** The station-leave hook
  (`pia_rtt` `STATION_LEAVE_OFFSET`) calls `peer_transport::clear_station`,
  whose `apply_leaves` retired the opponent only through the constant id
  bound to the leaving index. That binding comes from the station-map hook,
  which is part of the opt-in peer observer (off in the operator's config), so
  the id was 0 and the opponent stayed on the banner through the single-peer
  fallback (`view_of`: one confirmed peer is shown without a selection) until
  the whole session ended. `apply_leaves` now retires the only confirmed peer
  when an unbound station leaves; two or more confirmed peers are left alone
  (test `an_unbound_station_leave_retires_the_only_confirmed_peer`). Not yet
  seen on the console; the leave hook firing on a declined rematch is the
  assumption (the 16:40Z boot had no peer-observer rows to confirm it).
- **Online matches classified `offline`.** Two Quickplay matches ran with
  the connection still Offline (no quickplay/arena scene hook fired; the
  latency hook did, code 30). `Scene` gains `marked_online` (the latency
  mark); `decide` uses `effective_connection()`, which counts a marked
  Offline stage as Quickplay for Auto only. Menus and the waiting room stay
  Vanilla, classified connections are never rewritten, manual selections
  ignore the mark, and `mark_match` re-evaluates the request. The first match
  of a session therefore swaps to LessLag when the latency hook fires (~3 s
  after the stage load) instead of never; later matches in the session are
  LessLag from the stage load (the mark clears only on the main menu). Why
  the scene hooks miss the entry remains open.
- **Config drift.** The 23 saves during the b044752 session were native-pane
  edits: on an online character select the fork-style D-pad editing changed
  the buffer (Left/Right) and the profile (Up/Down) while the operator moved
  the hand with the D-pad. Native edits now require ZL held (ZL+ZR stays the
  view chord; L+R chords unchanged); the Settings window still edits with the
  plain D-pad. Overlay tests updated; README controls table.

Gates: 308 host tests, host and Skyline Clippy with warnings denied,
no-default build, rustfmt. Tag `v0.4.1` on a466c1a, build
`0.4.1-a466c1a2662c-1791998997`, NRO
`3408557fb5925e7df3999683d7ca69a2873caded53033cbed1f3baff1f9cea37`
(1,531,904 B), package `artifacts/3408557fb5925e7d/`; Forgejo release 371
with the NRO and manifest, download readback verified. Card 17:32Z:
`switch-mtp-041/deploy-041-mtp-1014.py`, 04c3008d → 3408557f, rollback
`backups/pre-0.4.1-3408557fb5925e7d.nro`, flags and config unchanged. Host
verified only at install: the runtime delta from the console-verified 0.4.0 is
the three fixes above; the operator plays it next.

### 0.4.2 — 2026-10-14: why the scene hooks "missed" the online entry

The operator asked why the hooks miss the entry. The events of the
b044752 session (`switch-mtp-match/boot-logs-1611/data/events.previous.log`,
code 73 `AutoContext` with the source hook) answer it:

| t (s) | context | source |
|---|---|---|
| 72.6 | quickplay / inactive | `0x22da160` (Online → Smash menu) |
| 307.8 | offline / inactive | `0x235aaa0` (main menu) |
| 406.8 | offline / training (waiting room) | `0x25d9288` (stage setup) |
| 422.6 | offline / singles (the match) | `0x25d9288` |
| 425.9 | — | `0x16ccab8` online-latency hook (code 30) |
| 860.4 | quickplay / inactive | `0x22da160` |
| 877.7 | quickplay / training | `0x25d9288` |

The Quickplay hook fires. The main-menu hook then reset the connection while
matchmaking continued in the background, and the waiting room and the match
started from the main menu without passing the Quickplay hook again. The fork
kept Quickplay across the main menu for exactly this reason (its comment: the
internal matchmaking transitions trigger this hook briefly); the rewrite
reset unconditionally. Secondary facts: the latency hook fires 3.1–3.3 s
after the stage load on every console match recorded; the Eden runs show
`0x22da160` firing on an offline menu (Games & More → Training produced
`quickplay/training`), so Quickplay from the menu hook alone is not proof of
an online match; the events ring (128 entries, 500 ms flush) drops rows in
stage-load bursts, so the 16:40Z session's missing menu rows are not evidence
of the hooks not firing.

Fix (this release): `auto_policy::main_menu_connection` keeps a stored
Quickplay on main-menu entry with no liveness check (anything else resets to
Offline; code 107 `a=1`), and a Singles/Doubles stage that has run five
seconds under a stored Quickplay with no online-latency mark is demoted to
Offline on the next diagnostics poll (500 ms cadence; `offline_match_due`,
`OFFLINE_CHECK_TICKS`; code 107 `a=2`, context source `0x0ff11e`). The mark
itself clears only when the connection is set to Offline, so a kept Quickplay
carries an earlier match's mark through the main menu. Auto therefore runs online matches as online from the stage load
(the 0.4.1 latency-mark fallback remains for the first match of a session that
somehow never passed the menu hook) and corrects a stale Quickplay five
seconds into an offline match instead of never. Manual profiles unaffected.
Gates: 309 host tests, host and Skyline Clippy with warnings denied,
no-default build, rustfmt. Tag `v0.4.2` on ddea22d (rewritten history),
build `0.4.2-ddea22d1d814-1792000378`, NRO
`87322eefca507870593c1ffbcdf70b5452a342bc1cf543b9ce43a692cad9c7e0`
(1,531,904 B), package `artifacts/87322eefca507870/`; Forgejo release 372
with the NRO and manifest, download readback verified. Card 17:54Z:
`switch-mtp-042/deploy-042-mtp-1014.py`, 3408557f → 87322eef, rollback
`backups/pre-0.4.2-87322eefca507870.nro`, flags and config unchanged.
Host-verified only at install; the console verdict on the Quickplay hold
comes from the next online session (`AutoContext` rows with sources
`0x235aaa0` and `0x0ff11e`, code 107).

Eden evidence from the documentation-media run (0.4.2 NRO, GIF build,
`docs-media-1014/`, events decoded by source): `0x22da160` fired at 162.7 s
on the offline path Games & More (connection quickplay with no network),
the Training stage followed at 227.7 s, and the main-menu hook at 437.9 s
and 542.8 s kept Quickplay (code 107 `a=1`) while the status went Inactive
and the cleanup hold engaged (`AutoRequest` hold bit, request Vanilla). The
regular Smash character select entered after the Training session was
classified Training by the CSS update (494.4 s): `is_training_mode()` still
reads true there, so the hold lifted and the banner read `LessLag | Auto`;
the Settings window at the main menu afterwards read `Vanilla*`. No
Singles/Doubles stage ran, so the stale-Quickplay demotion was not
exercised. Screenshots and GIFs from this run are in `docs/media/`.

A second media run (`docs-media-1014/NOTES.md` "Run 3") played an offline
Smash 1v1 against a level 9 CPU with 0.4.2: the compact two-row strip,
Performance and Debug views in the match, the results screen after quitting,
and the regular character select afterwards reading `You: Vanilla* | Auto`,
the first capture of the `Vanilla*` banner label (4f8bcfb) on a character
select after a match. The driver noted that the Plus in the Debug chord also
opens the game's pause menu in a match, which is the game's own binding.

Repository history: at the operator's request the `HANDOFF.md` file was
purged from every commit, tag and branch (`git filter-branch --index-filter
… --prune-empty --tag-name-filter cat -- --all`; bundle backup of the
pre-rewrite repository in the evidence root), and `AGENTS.md` no longer asks
for a handoff file in the repository. Every tag was re-pointed to its
rewritten commit; the Forgejo releases keep their assets.

### 0.5.2 — 2026-10-15: console move captures — Vanilla and LessLag hold 60, LLUltra alone drops

Console, build `1525e3e0` (`0.5.2-ce0145a2155c`), FX Control installed with
every effect on, Training Modpack Input Display off, `drs_moves=none`,
`drs_adaptive=false`. Seventeen `CAPTURE_MATCH` Training sessions (label
`match_training`, 82–367 s each, one per move of the performance-floor list:
Pyra side/neutral/up B, Mythra specials, Falcon Punch, Palutena side B, Ness
PK Thunder and PK Fire, Lucas PK Thunder, Kirby Hammer, Byleth specials,
Inkling Roller, Sephiroth, plus four unnamed), each switching Vanilla →
LessLag → LLUltra through the Deluxe window with the window hidden while
the move was repeated against a stopped CPU. Segments split on the code 46
kind-3 commits; seconds with the window open (code 23), within 3 s of a
commit, overlapping a cut-in push (code 82 source 1) or inside the quit
transition excluded. Analysis script `tools/frame_windows.py` (per-second
callback counts, worst sliding one-second window, resolution edges marked)
plus per-frame reading of `api-<id>.csv`; evidence and per-session table in
the tangent evidence root, `card-captures-1015-redo/ANALYSIS.md` and
`MOVE-LEDGER.md`.

Result. Vanilla: every segment holds 60 after one 51–57 second 1–3 s into
the capture (a single 55–140 ms gap at match start). LessLag: 60 in all 16
segments; the only sub-59 seconds are three 53–54 seconds on cut-in pushes
in one session. The first attempt the same evening, which read as
"LessLag drops 20–49 % of seconds", had the Deluxe window open while
fighting; with those seconds excluded it is clean too. LLUltra: fails the
"no second below 59" rule in 11 of 17 sessions — stretches of 33–41
callbacks per second made of alternating 19 ms and 31–33 ms frames (Kirby
hammer eleven seconds in a row, Falcon Punch 8 of 14 s, Pyra side B 9 of
25 s); Palutena side B, both PK Thunders, Inkling Roller and the Sephiroth
session pass.

Mechanism, from the per-frame API trace. LLUltra (Immediate) acquires and
waits for the texture at the start of the frame, runs the CPU update and
flushes + presents at the present entry, so the image can flip at the very
next vblank: the frame has to fit **CPU work + GPU time ≤ 16.7 ms**.
LessLag and Vanilla present at the late flush right after the vblank wait
and pipeline the GPU behind the next frame's CPU work: **max(work, GPU) ≤
16.7 ms**. Measured: CPU work from the vblank to the present entry is
9.2–10.4 ms (p50) in LLUltra and Vanilla and 11.5–12.1 ms in LessLag
(relocated model update); in LLUltra's half-rate seconds it is 10–12 ms
(p50) and 11.7–16.5 ms (p90), the extra lying between the fighter-record
and post-scene marks (scene/effect update; UI job and worker waits
unchanged). A frame that presents 6.9 ms before the vblank held, one that
presented 4.3 ms before it missed, so the GPU time at 576p is 3.4–5.3 ms
and any frame over about 11.5 ms of CPU work misses; the two-image ring
then waits 19–20 ms for the next acquire (the `nvnSyncWait` max span,
against 5–8 ms in every other profile). Resolution is not the separator
(LessLag at 1080p holds, LLUltra at 576p drops).

Consequences. The per-move effect cost drops no frame in Vanilla or LessLag
on the console in this scene, so the measured-drop part of the
performance-floor plan concerns LLUltra only. The shipped levers trim the
GPU term by about 1–1.5 ms (adaptive or move-table DRS 576p → 480p), enough
for the moves with work p90 ≈ 12 ms and not for Falcon Punch or the Kirby
hammer; Reduced effects are the only lever on the CPU term and are unproven
(the Eden quota numbers did not fall for Pyra, Palutena, Falcon). A
render-policy fallback — present at the late flush for a run of frames after
an acquire wait over about 10 ms, i.e. LessLag's response for the duration of
the move — would hold 60 without per-move knowledge; not built. Gaps: Falcon
Punch has no LessLag segment, no full-charge Gigaflare edge (code 82 source
2) was recorded, four sessions are unnamed, no online match was captured.

### 0.5.2 — 2026-10-15: coverage build and the first Eden coverage campaign (offline and local wireless)

Operator request: light execution-coverage tracing of the Eden build to
inform code reduction without blind removal, with the rule that code is not
kept because a unit test exercises it. Implementation (commit "Coverage build:
instrumented variant with a Rust raw-profile writer"): cargo feature
`coverage`, `tools/Build.ps1 -Coverage`, per-package `-C instrument-coverage
-Zno-profiler-runtime` through `tools/coverage/config.toml`, and
`runtime::coverage` as the whole profiler runtime (defines
`__llvm_profile_runtime`, serialises the `__llvm_prf_*` sections as raw
profile format 10 through `coverage_profile`; `live.profraw` every 10 s,
`scene-NNN-P-S.profraw` at scene changes, `manual-NNN.profraw` on Debug
L+R+Left; event code 109). Host proof: a program built with the same flags and
the same writer merges in `llvm-profdata` and reports exact per-function
counts in `llvm-cov`. Gates: host clippy with and without the feature, 297
tests, no-default build, Skyline clippy with and without the feature, Skyline
build. Coverage package `artifacts/7df44f460e8f9aa3/` (build
`0.5.2-35ec596fc1a9-dirty-1792077376`, NRO 1,765,376 B, manifest
`"variant": "coverage"`, 1,392 instrumented functions, 4,676 counters).

Campaign (Eden CLI-control `10bcd2d849`, private display, `platform=emulator`,
manual LessLag, buffer 3, DRS flags off; game driven by an Astra low agent,
snapshots and staging by the primary): six scenarios — boot to Training;
the Player Info window in Training through every profile, buffer edit, view
cycle, Debug view, 30 s timing capture, manual dump and combat; Training
exit, main-menu Stealth/Streamer toggles and two Smash CPU matches; and a
two-peer local-wireless session (profiles A and B through an unlisted
password-protected emulator room on localhost, `CAPTURE_CSS` and
`CAPTURE_MATCH` present, SDK demo plugin installed on the host): lobby,
challenger accept, character select with both native status lines (18–21 ms
peer latency shown), a Sonic versus Lucario match on Final Destination with
the compact strip, Plugins 1/1 page, Performance and Debug views, about two
minutes of combat, the quit chord and clean room exits. The first LDN
attempt's host was OOM-killed by its 6 GiB cgroup cap while the match loaded
(kernel `oom-kill`, not a guest fault); the lobby/CSS profile was kept and the
second attempt ran at 8 GiB without incident. Cumulative snapshots were
differenced with `tools/coverage/profraw_diff.py`.

Result (`tools/coverage/report.py`, llvm-cov total 1,383 functions in the
map, 70.4 % regions, 69.0 % lines): 1,091 of 1,422 instrumented functions in
`src/` executed in at least one scenario (77 %); 331 never (107 closures, 22
const constructors/trait impls); 202 named never-executed functions spanning
2,723 physical lines; 2,188 lines never executed inside functions that ran.
The never-executed code is, in order of size: Nintendo Switch Online paths
(Arena room and station board, Quickplay hooks, about 750 lines; unreachable
in any emulator session), flag-gated diagnostics from closed investigations
(`DUMP_PRESENTATION_HISTORY`, `PROBE_FIGHTER_MODULE`, `DUMP_RENDER_PROBE`,
`PROBE_CROP`, the `dispatcher_backoff.ini` parser; about 740 lines), features
off in this configuration (FPS++, DRS and the 0.5.2 triggers; about 560
lines), plugin-API paths needing an option edit or an own-page plugin,
defensive and teardown paths, and 21 production helpers (115 lines) with no
production caller. Two groups of functions share identical aggregate counts
in every scenario (27 per present, 24 per diagnostics poll); equal totals,
not proof of shared or dispensable work. Analysis, tiers and the deduplicated
reduction inventory (Tier 1 unused helpers 115 source lines; Tier 2 opt-in
diagnostics and the backoff override file ≈ 1,100 source lines plus ≈ 180
test lines; together about 5 % of the non-test source) are in the evidence
root's `coverage-1015/REPORT.md`, reviewed by an Astra high session whose
nine findings (two FPS++ helpers wrongly listed as unused, `window_finalize`
wrongly grouped with the crop probe, an unsupported poller-gating claim,
double-counted totals) are applied; the report is evidence for review, not a
removal list. Scenario
gaps stated there: no NSO session, no first boot without `config.ini`, no
plugin option edit, no own-page plugin, no console run. Both Eden profiles
were restored afterwards (journal `RESTORED`, SHA-256 verified).

### 0.5.3 — 2026-10-17: native banner and Player Info window cut to one text builder, one style decision and one row model (host tests, Skyline build)

Same file-by-file reduction order as the two cuts below. Plan: evidence root
`reduction-1017/PLAN-native-ui.md`. Commits a21394c (native banner, host and
runtime) and 160efc0 (window, plus `pia_rtt::Shown<T>`) on `work/native-ui-1017`
from `reduction-candidate` 93116e7.

Production code lines (code-audit classifier, comments/blanks/test modules
excluded): `src/native_banner.rs` 507 → 256 (tests 532 → 248, 18 → 9 tests);
`src/runtime/native_banner.rs` 1,358 → 689; `src/runtime/ui.rs` 1,051 → 820;
`src/pia_rtt.rs` 641 → 637; `src/runtime/pia_rtt.rs` 230 → 226; `src/menu.rs`
430 and `src/overlay.rs` 297 unchanged. Net −1,159. The plan's host (≤ 260)
and runtime (≤ 700) targets are met; the window's ≤ 600 is not: every view,
row, strip, chord and record is kept, and about 220 of the remaining lines are
ImGui calls, record encodings and struct literals that rustfmt lays out one
argument per line (`fn_call_width` 60, `struct_lit_width` 18).

Removed, host: `LineParts`, `PaintInput`, `PaintOutcome`, `BannerPaint`,
`Observe`, `observe_style`, `telemetry_text`, `with_room_id`, `arena_status`,
`banner_paint`, `font_for`, `material_for`, `colors_for`, `font_choice`,
`arena_y`, `style_matches`, `show_native_remote`, `local_wireless_text`,
`visible_network_count`, `store_scan_count`, `ldn_symbols_usable`,
`LDN_INITIALIZED`, `LDN_INITIALIZE` / `LDN_GET_STATE` / `LDN_SCAN`,
`bound_bytes`, `main_data_end`, `is_arena_room_name` (the `ARENA_ROOM_NAME`
const remains), `remote_line_max()` (now the `REMOTE_LINE_MAX` const);
`local_count_active` lost its `symbols_ready` and `identity_ok` parameters.
Added: `status_text`, `paint`, `PaneKind`, `Style`, `Styled`, an owned
`RemoteLine`, `local_lobby_text`. Runtime banner: `install_ldn`,
`symbol_mapped`, `hook_symbol`, `on_ldn_initialize`, `on_ldn_scan`,
`network_state`, `lookup_symbol`, the `LDN_*` / `ORIGINAL_*` / `GET_STATE` /
`SCAN_COUNT` / `LDN_READY` statics, `snapshot`, `arena_parts`, `arena_text`,
`ArenaLine` / `ArenaParts`, `empty_slot`, `store_text`, `store_bytes`,
`slot_text`, `names_equal`, `find_slot`, `address_slot`, `free_slot`,
`clear_arena`, `clear_arena_state`, `pane_name_at`, `parent_of`,
`pane_from_link`, `mapped_span`, `raw_name`, `pack_half`; `apply_pos_y` is
folded into `apply_style`, `read_material` / `write_material` into one
`material`, `copy_name` is `pane_name`. Window: `SettingCell`,
`CompactLines`, `setting_cells`, `settings_strip`, `marked`,
`compact_settings_lines`, `compact_performance_lines`, `compact_plugin_lines`,
`draw_settings_vertical`, `draw_plugins_vertical`, `draw_rows` (now
`draw_table`), `draw_compact_lines` (now `draw_strip`), `draw_debug` (now
`debug_rows`), `plugin_page` (now `plugin_rows`), `latest_frame_time` (folded
into `timing_line`). `pia_rtt`: `current_publication()` and `current_board()`
return `Shown<T>`; `present_published`, `present_station` and
`aggregate_stability` take it instead of three generation numbers.

Behaviour changes, recorded in `docs/FORK_PARITY.md` and
`docs/NATIVE_DISPLAY.md`: Streamer's restore of a pane is one-shot and no
longer re-fires when only the live font drifted from the saved one; the
local-wireless lobby pane goes through the same per-pane capture as the title
panes, shows the input buffer behind the marker outside Streamer and the game's
own captured count text in Streamer (the fork's `nn::ldn` hooks and bare scan
count are gone). Diagnostics only: the install records `HookRejected` for a
refused CSS or arena site (was a log line); a rejected pane open records one
`PaneRejected` and one `PaneField(address, vtable)` instead of four fields;
the chained text setter records one `Compatibility` row.

Gates: 233 host tests plus the SDK round-trip, host clippy with warnings
denied, no-default build, rustfmt, Skyline release build
`0.5.3-160efc058ba2-1792252181` (NRO 1,437,696 bytes, SHA-256
`e2b0a365a1876921…`, the same size as the ecb9923 candidate). Not run in Eden
or on the console. Owed before the branch merges: CSS title and Streamer
restore, the local-wireless lobby pane in both Streamer states, the arena pane
(NSO), and the window's vertical tables and ready-match strip.

### 0.5.3 — 2026-10-17: Tier 1 code reduction — 21 helpers with no production caller removed (host tests, Skyline build)

Operator rule from the coverage campaign: code is not kept because a unit
test exercises it. Commit 38179f7 (a Fable worktree agent, reviewed by the
primary) removes the 21 production helpers that `coverage-1015/REPORT.md`
§4 Tier 1 found referenced only from tests or from nothing: `menu::
platform_label`/`net_profile_value`, `render_policy::profile_text`,
`native_banner::local_update_accepted`/`displayed_network_count`/`walk_pane`/
`text_fits` (and the `GraphNode` struct only `walk_pane` used), `pia_rtt::
next_draw_generation` and the `Session::accept_sample/leave/cleanup/poll`
wrappers (the `*_for` variants with an explicit `StationPick` are the
production API), the `render_trace` const decoders `core_info_parts`/
`pointer_from_halves`/`flush_narrow_parts`/`flush_plan_parts`, `resolution::
Policy::dropped_transitions` and the free `update_settings`/`clear_reports`,
`peer_transport::shown_peer`, `peer_passive::packet_prefix_accepted`. 154
production lines gone, 284 deletions against 126 insertions overall. No
behaviour change: no call site outside `mod tests` existed. Tests: four
assertions/tests that specified nothing but a retired helper were deleted;
the rest were rewritten against the production path (`profile_status`/
`awaiting_index` for the profile label cases, `*_for(..., StationPick::
Absent)`, `peer_frame().peer`, `transition_dropped`, and `perf_scaler`'s
fixtures now use `resolution::reset_scene`/`retarget` instead of the removed
wrappers — same asserted state; `reset_scene` also sets `dirty`, which no
test observes); the render-trace decoders and the pane-walk oracle moved
into their files' test modules so the encoder and `CSS_TITLE_WALK` tests
still hold. One correction to the report: the resolution wrappers did have
callers, in `perf_scaler.rs` tests. `PIA_MAGIC` gained a host-only
`allow(dead_code)` (its production user `runtime/peer_observer.rs` is
Switch-only). Gates: `cargo clippy --all-targets -- -D warnings`, 308 host
tests, `cargo build --no-default-features`, Skyline release package
`artifacts/9edfcedb1abb12cc/` (build `0.5.3-38179f792544-dirty-1792196741`,
"dirty" from an untracked worktree directory since removed; NRO 1,486,848 B,
SHA-256 `9edfcedb1abb12cca229b33f4f6346e708e93a9bb1de88b89725e0c87221cad6`).
Not run in Eden or on the console; nothing in the change executes at run
time. The NSO coverage campaign that would give the Tier 3 (Arena/Quickplay)
evidence started the same evening on the LAN-NSO Eden but stopped at the
Online tile (error 2306-0502: the fixture's companion service was not
listening on 8443/8444); its partial profiles are in the evidence root's
`coverage-nso-1016/`.

### 0.5.4 — 2026-10-17: reduction-candidate merged into main; whole-product coverage trace closed with what it has

Operator order: stop the reduction work with what it had, merge the code into
`main` and install it on the Switch. `reduction-candidate` (code tip
962eb3d, ledger 4a2006a) merged into `main` with a merge commit (Cargo
version bump only; no code change since the candidate package
`5357da2626a98fb9` that carried the latency rerun below). The merge was first
labelled 0.6.0 (tag `v0.6.0`, Forgejo release 384, card install
`switch-mtp-060/`) by misreading the order; the operator corrected it to a
patch version within the hour, so the
version is 0.5.4 (commit on top of the merge), the `v0.6.0` tag and release
were removed and the card reinstalled (`switch-mtp-054/`). Gates on the merge:
`cargo clippy --all-targets -- -D warnings` clean, `cargo build
--no-default-features` ok, host tests 233 + 1 pass, `cargo skyline build
--release` ok (package in the evidence root `artifacts/`, hash in its
manifest and in the release). The 0.5.4 NRO goes to the operator's card over
DBI MTP in the same transaction style as the earlier installs; the card run
is the acceptance test, not claimed here.

Coverage trace of the coverage variant of the same source (package
`72d91b51b540bf26`, Eden only): offline host and guest scenarios (Training,
window pages, plugin pages, menus, CSS, Streamer), then LAN-NSO runs
`nso-b3` to `nso-b6` on the Yoga (Quickplay versus the Ally on Online Deluxe
0.3.0, two public arenas including the Streamer room-pane check, Background
matchmaking from Training, Co-op Quickplay) and a Surface peer `surf-a1`
(Smash menu only). Merged result (`coverage-1017/report-final/`): 1,192
instrumented functions, 1,047 executed in at least one scenario, 145 never
executed (68 closures, 13 constructors/trait impls; 813 physical lines; 73
with no live reference). Interim before the arenas: 1,024 executed. Never
executed and worth a follow-up: the hook sites behind `deluxe_rewrite_arena`
and `deluxe_rewrite_background_matchmaking` never fired in any arena or
Background scenario; `peer_transport::settings_label` / `profile_bits`
(the modded-opponent exchange) never ran because no two coverage-build
peers ever reached a match: the candidate and coverage builds abort at
startup on the Ally's Windows Eden (`nn::fs::RenameDirectory` → `nn::diag`
abort from the plugin; Linux Eden and the console are fine), and the
Linux Surface peer paired with the Yoga (companion probe `result=true`
once the Yoga advertised its wired address) but every Quickplay ended in
2618-0513/2618-0502 with ufw active on both Linux hosts and no sudo to
change it. Both are open findings, not fixed in 0.5.4.

### 0.5.4 — 2026-10-18: LAN-Online coverage campaign (Eden, two modded peers) — Stealth gates the peer exchange; the arena and Background hook sites are wrong

Second NSO coverage campaign, on the Yoga, using Eden's new automatic "LAN
Online" mode: two Eden instances on one PC, profiles `a` and `b` under
`/home/intpa/eden-nso-autolan`, no companion service and no fixture. That
removes the blockers of the 2026-10-17 trace (the companion needing distinct
source IPs, ufw between the two Linux hosts). Coverage build
`0.5.4-1c184ff8dd2a-1792323967`, package `artifacts/9a275b4005651f6d` (NRO
SHA-256 `9a275b40…`). `src/` and the Cargo files at `1c184ff` are identical to
release 0.5.4 (`a63bc8c`; `git diff a63bc8c 1c184ff -- src Cargo.toml
Cargo.lock` is empty), so this is the shipped code plus the coverage writer.
Both peers ran it, so every match below was between two modded peers. All of
this is emulator evidence; no console run and no source change are part of this
entry.

Reached: three Quickplay 1v1 matches, one with a Yes rematch; a public arena
played for two rounds (create, name entered through the keyboard applet, join,
two rounds, Streamer On and Off, guest leave, host close); a Background
Matchmaking match (peer `b`); a Co-op 2v2 that formed with four fighters and
ended after about a minute with the game's "disconnected due to inactivity"
(the driver's input cadence, not the network); and a final Quickplay match
with Stealth Off on both peers. Frame rate read 59–60 in every match, an
emulator reading and not a console figure.

Result (`coverage-1018/report-final/`): 1,192 instrumented functions, 893
executed, 299 never executed. The run held only NSO scenes (no offline,
Training or character-select scenarios), so the 299 is not comparable to the
145 never-executed of 2026-10-17. Against that report
(`report-final/compare-1017.md`), 12 functions that had never run executed
today, and 133 ran in neither campaign.

**Finding 1 (a run condition, not a defect): the peer settings exchange needs
Stealth Off.** The modded-opponent exchange (`peer_transport::settings_label`,
`profile_bits`, `shown`, the `note_rtt` closure, `retire_only_confirmed`,
`runtime::connection_transport::body_addr`) executed only in the Stealth-Off
scenario: 46,356 `settings_label` calls on `a` and 52,788 on `b`, against 0 in
every earlier scenario. `stealth` defaults to `true`, which suppresses the
recognition beacon, so neither peer is confirmed as modded and `payload_write`
skips. The session latches the saved value, so after Stealth is switched Off a
new online session is needed. With it off, the Settings page showed the peer
as `Ethernet, Auto, LessLag` with 12–13 ms ping (the interface is the
emulator's reported one). This corrects the reading in the 0.5.4 entry above:
the exchange had not run on 2026-10-17 because no two modded peers reached a
match, and it did not run in this campaign's first scenarios because both
peers were on the default Stealth On. The condition is now stated in
CONFIGURATION.md.

**Finding 2 (correctness): the arena and Background scene hooks are
mis-addressed.** `deluxe_rewrite_arena` (`main+0x22d9fa0`) and
`deluxe_rewrite_background_matchmaking` (`main+0x22da090`) in
`src/runtime/online.rs` executed 0 times again, although the arena lobby was
entered on both peers and a Background match was played. Read-only
disassembly of the extracted 13.0.5 `main` shows that `0x22d9fa0` is
`ldr x19, [sp, #0x10]` in the epilogue of a scene-factory function that
starts at `0x22d9ef0` (the next on-grid factory entry is `0x22d9fc0`), that
is, in the middle of a function. `0x22da090` is a real factory entry (it
allocates 0xf0 bytes, vtable `0x50b7c70`), but its class is not created in
the arena or Background flows that were exercised. Both addresses were copied
from the fork (`reference/quickplay/src/net/mod.rs`) and are guarded only by
`preflight`: there is no instruction-word match, unlike `STAGE_SETUP` and the
arena panes. Consequence: arena and Background matches get no connection kind
from these two hooks and are classified through `stage_setup` and
`mark_match`. `note_arena_name` at 0 is the healthy result (it only logs an
unexpected room-id pane name). Because the LAN Online harness removes the old
blockers, this is a plugin finding and not a harness limitation. Not fixed
here. Recommended follow-up: remove or re-derive both sites, and add
instruction-word guards to every fork-derived address.

Evidence outside Git: the operator's evidence root `coverage-1018/`
(`FINDINGS.md`, `report-final/`, `report-final/compare-1017.md`); Eden
profiles under `/home/intpa/eden-nso-autolan`.

### 0.5.5 — 2026-10-18: declined-rematch peer persistence; live-peer requirement

Operator report from the console (real NSO): after Rematch Yes on one side and
No on the other, the Yes side keeps showing the opponent's player info. The Eden attempt below is emulator
evidence; the console behaviour, both of the report and of the fix, is
untested here, and no console log of the incident is part of this entry.

**Eden attempt: not reproduced.** Seven trials on the Yoga, 16:50–19:10Z, two
Eden instances in LAN Online mode (profiles `a` and `b` under
`/home/intpa/eden-nso-autolan`), the 0.5.4 release NRO (`3c8e4f52…`, the card
build) on both, a card-like INI (`render_mode=lesslag`, `platform=console`,
Stealth at its default On), Quickplay Solo. Trial 7 ran the exact sequence: A
pressed Yes at about 19:05:18Z, B pressed No at about 19:05:40Z. On A, within
about 20 s, the game showed "A communication error has occurred." in the
waiting room, `events.log` recorded `68 TransportCleanup` (a = 1) at the
decline, the Player Info window held only the "You" column and the Debug view
had no `Ping detail` row. The other exits tried (disconnect, inactivity, kick;
trials 1, 3 and 4) also cleared the peer, and they also end the session
(code 68). The harness cannot reach the console's path: in LAN Online a decline
tears the Pia session down, and the session-end hook (`.text+0x15ec90`,
`clear_session`) is the path that already clears the peer. The console's
decline is assumed to be a graceful leave, where the opponent leaves and the
session continues (the working assumption; no console log of it has been
read). Until the harness's decline behaves that way, an Eden run of a declined
rematch is not evidence for or against the fix below. The attempt also cost
about two hours of harness trouble (a `lan_online_tls` mismatch between the
profiles, an OOM kill under a 6.5 GiB cap, a 20-minute online ban after
repeated self-destructs); those lessons are in the findings file.

**Code reasoning (no run).** A graceful leave reaches Deluxe only through the
station-left hook (`.text+0x15f240`, `deluxe_rewrite_observe_station_left`).
`Table::station_left` (`src/pia_rtt.rs`) returns false when the leave's
protocol id is zero or differs from the RTT table's protocol, a deliberate
guard (test `foreign_and_unknown_protocol_leaves_do_not_retire_a_current_station`),
and the hook drops the false case silently. Every retirement path starts from
an accepted leave (`clear_station`): the 0.4.1 declined-rematch change
(`peer_transport::retire_only_confirmed`) only handles an accepted leave for a
station with no bound constant id. Nothing else retires a bound station: its
name, `index_id` binding and confirmed peer carry no TTL and persist until the
next session end or protocol adoption, and no scene change or window view
clears them. So a leave that never reaches the hook (H1a) or reaches it with a
protocol id the gate rejects (H1b) leaves the opponent displayed. Neither
hypothesis is confirmed: no event code covered the leave hook, so the card's
`events.log` cannot tell them apart.

**Fix: the live-peer requirement, chosen by the operator.** Rather than guess
which leave the console sends, the opponent display now requires the selected
station to be live: a measured RTT publication for it within
`pia_rtt::LIVE_MS` = 10 000 ms. A newly bound station gets a 10 s grace (the
poll stamps the binding), so it shows at once, and each measured publication
renews the stamp. The rule gates the opponent column of the Player Info
window, the in-match strip's `Peer:` line and its IF, Ping, PeerNet and
PeerProfile facts, the Ping cell, the peer-dependent Settings rows and the
native CSS banner's remote line. Arena roster lines are unchanged (they read
the per-station board). The station-left protocol gate is unchanged. New event
code 112 `PeerStale` (a = station index, b = silence in ms), one row per
live-to-silent transition, never per frame. Code: `src/pia_rtt.rs`
(`LIVE_MS`, `SYSTEM_TICKS_PER_MS`, the pure `station_live(last_seen_tick,
now_tick, ticks_per_ms)` with unit tests; a zero stamp, an unknown tick rate
or a clock that went backwards read as not live), `src/runtime/pia_rtt.rs`
(`track_liveness` in the 500 ms diagnostics poll, `station_live(selection)`),
`src/peer_transport.rs` (`opponent_column_visible(streamer, selection,
live)`), `src/runtime/ui.rs`, `src/runtime/native_banner.rs`,
`src/diagnostics.rs`. Version 0.5.4 → 0.5.5 (Cargo.toml and Cargo.lock).

**Host gates** (commit `6a9e61a`): `cargo test` 235 + 1 pass, `cargo clippy
--all-targets -- -D warnings` clean, `cargo build --no-default-features` ok.
Skyline package `artifacts/cf57cf80f25f2e83/` in the evidence root, build id
`0.5.5-6a9e61afdac8-1792351203`, NRO SHA-256
`cf57cf80f25f2e837deabaf859489dc00653692c269fb4a6e2069a6a1b9bf8c2` (1,437,696 B),
ELF `d2c3605a…`. Released as Forgejo v0.5.5 (release 394, NRO and manifest
attached, download readback matched) and installed on the card the same
evening over the 0.5.4 NRO, which had booted once (operator, 2026-10-18
afternoon: menus, overlay and Training fine; FX cells not run); the 0.5.4 NRO
is kept as `backups/pre-055-cf57cf80f25f2e83.nro`. 0.5.5 was replaced on the card by 0.5.6 the same evening without booting.

**Eden smoke of the 0.5.5 NRO (emulator evidence, 19:21–19:40Z).** Both
LAN-Online profiles staged with `cf57cf80…`, same card-like INI. One full
Quickplay Solo match with the window open on Settings: the strip read
`Peer:intpater IF:Unknown Ping:12ms` at 1:13 and at 0:32 remaining, the
Player Info window at the results screen and on the "Rematch?" prompt still
showed the opponent column (`intpater / Unknown / 12ms`), and no `112` row was
written during the match, the results or the prompt. So the live-peer rule
does not hide a live opponent and the 10 s grace covers the match flow. The
decline itself again ended the LAN session (B's prompt timed out, A's waiting
room showed the communication error, `68 TransportCleanup`, only the "You"
column afterwards), so the rule's hiding branch was not exercised in Eden; it
is covered by the unit tests only until the console run. A first smoke
attempt in the same boot was void (B dropped for inactivity 14 s into the
match). Logs: `rematch-1018/smoke-055/profraw/al-{a6,b6}/final/`, screenshots
`rematch-1018/screens/smoke2-*.png`.

**Owed on the console** (none of it done): with the 0.5.5 NRO on the card,
(1) choose Rematch Yes while the opponent declines on real NSO: the
opponent's column, the strip's `Peer:` line and the CSS remote line must
disappear within about 10 s, and `events.log` must show a `112` row for that
station; (2) play a normal match through the stage load, results and return
to the character select, and with the window open, to confirm the opponent
column does not flicker (a `112` row in ordinary play would mean the 10 s is
too short for the console's loading gaps); (3) the leave hook still has no
event code, so H1a against H1b stays open until it is instrumented. A
separate Eden task, owned by the LAN-Online harness agent, makes the LAN
runtime's decline behave like NSO's so that Eden can exercise this path.

Evidence outside Git: `~/.local/share/ssbu-mod-backups/plugin-api-tangent-20261012/rematch-1018/`
(`FINDINGS.md`, `screens/`, `profraw/al-a5/final/` and `al-b5/final/`, the
endgame and keepalive logs).

### 0.6.2 — 2026-10-19: Auto shows the frame count the game chose

- Request (operator, after the 0.6.1 console boot): have Auto show the chosen
  buffer, as in `Auto (4f)`.
- Change: the online-latency hook (`runtime/hooks.rs`, fork hook `0x16ccab8`,
  event code 9) already reads the game's buffer byte before applying our
  selection. New `policy::observed_auto_buffer(automatic, selected)` returns
  that byte only while the selection is Auto (a manual value overwrites the
  byte, so later calls would echo ours); the runtime keeps it in `AUTO_BUFFER`
  (`runtime/mod.rs`: `note_auto_buffer`, `auto_buffer`, `clear_auto_buffer`)
  and clears it when `auto_driver::set_scene` moves the connection to Offline,
  so it survives the results screen and the rematch character select of the
  same session. `native_banner::local_latency_label(buffer, auto_frames)`
  renders `Auto (4f)` / `Auto` / the manual number; callers are the window
  rows and You line (`runtime/ui.rs`), the native CSS/arena You line
  (`runtime/native_banner.rs`) and the lobby text (always `None`: the
  local-wireless pane is one digit wide). Native You line latency field
  `YOU_LATENCY_MAX` 4 → 10, `YOU_LINE_MAX` 32 → 38, `TEXT_CAP` 692 → 698
  (`PANE_SCAN_MAX` still under 1024). Peer exchange, INI, event codes, chords
  and the plugin API unchanged.
- Host tests: `policy::auto_buffer_is_observed_only_while_auto_is_selected`
  (new), `native_banner::status_text_keeps_marker_room_you_and_remote_lines`
  extended (`Auto (4f)`, `Auto (25f)`, manual stays a number, the native You
  line keeps `Auto (25f)` whole, `YOU_LINE_MAX == 38`).
- Gates on this commit: `cargo test` 240 + 1 passed; `cargo clippy
  --all-targets -- -D warnings` clean; `cargo build --no-default-features` ok;
  `cargo skyline build --release` ok (the runtime callers compile only for the
  Switch target). Build id, NRO SHA-256 and size: the v0.6.2 release manifest.
- Console follow-up: the [published v0.6.2 release notes](https://git.syntaxbyexample.com/intpa/ssbu-online-deluxe-rewrite/releases/tag/v0.6.2)
  record installation at 2026-10-19 13:01Z, a same-day online Switch run and the
  operator's confirmation that `Auto (Nf)` shows correctly online. Build
  `0.6.2-302c09574b3e-1792414221`, NRO SHA-256
  `59dab7a8cd60d8d770d7a7c4f30bb668b174854d666e72086ed06e5c238768c4`.
  This supersedes this entry's original "not booted" note. It does not report
  every location, main-menu reset and event-code comparison individually.

### 0.6.1 — 2026-10-19: the last page is titled Window Options

- Change: the last page of the L+R+Down cycle, which 0.6.0 titled just
  `Window`, is titled **Window Options** (`src/runtime/ui.rs`, the ImGui
  title string passed to `igBegin`). No other code change; the rows
  (PluginsPage, OwnPages, PerfPage, PerfView, Hints), the cycle, the INI keys,
  the event codes and the plugin API (descriptor 4, SDK/ABI crates 5.0.0) are
  as in 0.6.0.
- Why: the operator tested 0.6.0 on the console the same day. It worked and
  the menu was cleaner; the one requested change was a real name for the third
  page, which was titled only "window".
- Gates on this commit: `cargo test` 239 + 1 passed; `cargo clippy
  --all-targets -- -D warnings` clean; `cargo build --no-default-features` ok;
  `cargo skyline build --release` through `tools/Build.ps1 -Package` ok. Build
  id, NRO SHA-256 and size: the v0.6.1 release manifest on Forgejo.
- Console follow-up: booted on 2026-10-19; the operator confirmed the Window
  Options title and that it works (also recorded in the published v0.6.2 notes linked
  above). The layout is the operator-tested 0.6.0 layout; only the title differs.

### 0.6.0 — 2026-10-19: Main page redesign (You/Opp lines, five rows, Window page)

The operator found the 0.5.7 main screen bloated, with too much
information. An inventory of the Settings view confirmed it: 14
settings rows in three columns (Name, Interface, Peer, Ping, NetLatency,
NetProfile, Status, Stealth, Streamer, UltraRes, PluginsPage, OwnPages,
PerfPage, PerfView), four of them configuration of the window itself with a
meaningless `---` in the opponent column, Ping shown twice (Settings row and
Performance block), the 11-row Performance block and the 180-sample plot on
the same page, and no on-screen hint for any chord, which is how Streamer
flipped unnoticed before 0.5.7. The operator approved the whole proposal as
0.6.0.

Change (`src/menu.rs`, `src/overlay.rs`, `src/runtime/ui.rs`, `src/config.rs`,
`src/runtime/mod.rs`):

- Settings page: a `You` summary line (`Vanilla+ | Auto | Wi-Fi`), an `Opp`
  summary line while the peer shows (`Mario | LessLag | 3 | 23ms | Ethernet`,
  unannounced facts left out, painted in the ping's stability colour, Left/Right
  with ZR cycles the station as the Peer row did), then NetLatency, NetProfile
  with the engine status folded in (`LessLag | Swap`; an empty or `Vanilla`
  status shows the profile alone), Stealth, Streamer and, on console, UltraRes.
  Two columns; the opponent column, and the Name, Interface, Status and Ping
  rows, are gone. Under the table a one-line footer in the frame-time colour
  (`60.0 fps | 16.67 ms | 1080p | DRS on`) and a dim hint line
  (`ZR+Left/Right edits   L+R+Down next page   L+R+X Vanilla+   L+R+ZR+Y Streamer`).
  While ZR is held the selected value reads `< value >`.
- Window page (new `View::Window`, value 6, appended so recorded view numbers
  hold): PluginsPage, OwnPages, PerfPage, PerfView and the new Hints row. Last
  in the chord cycle and always in it, so the toggles stay reachable:
  Settings → (Plugins) → (own pages) → Performance → Window → hidden;
  `hide_pages=plugins,performance` gives Settings → FX Control → Window → hidden.
  The row model is one list (`ROW_PEER` … `ROW_HINTS`, 11 rows) gated by page
  (`RowGate::window`), so Up/Down wrap within a page and the Settings handler
  can never reach a Window row.
- The full Performance block and the plot are drawn on the Performance and
  Debug pages only; `perf_view` affects those.
- In-match strip: Settings line 1 is `Net`, `Profile`, `Stealth`, `Stream`,
  `UltraRes`; line 2 is `>Opp:<name> | <profile> | <buffer> | <ping> | <if>`
  and the timing line. The Window page has its own strip (the page fields and
  `Hints`, then the timing line). The `Peer:Hidden` line under Streamer is gone.
- INI: `hints=true|false` (default true; a file without the key keeps the
  line). Every other key, every chord, the event codes, the peer exchange and
  the native banner are unchanged. The separator ` | ` is the native banner's;
  the hint and marker text are ASCII because the window's font is the ImGui
  host's.

Host gates: 239 + 1 tests (new: `window_page_rows_toggle_the_hidden_pages_perf_view_and_hints`,
`summary_lines_footer_and_edit_marker_read_as_designed`,
`hints_default_on_round_trip_and_reject_other_values`; the view-cycle, page-skip
and plugin-page tests walk through Window), `cargo clippy --all-targets -- -D warnings`,
`cargo build --no-default-features`, `cargo skyline build --release`. Not
exercised: the drawn layout (Eden is under the operator's yield order). Released
as Forgejo v0.6.0 with the NRO and manifest; the NRO SHA-256 is in that
release's manifest and in the evidence root (`switch-mtp-060/`).

Owed on the console: the Settings page in menus, the character select,
Training and a match (You line, Opp line with a live opponent in the ping
colour, five rows, footer, hint line); the Window page after Performance and
its five rows editing live; `< value >` while ZR is held and plain without;
the strip's two lines in a match; a Streamer round trip hiding the Opp line.

Console 2026-10-19 (operator's boot of the 0.6.0 NRO `4741645e…`): tested by
the operator; it worked and the menu was cleaner. The one change requested was a real title for the last page, done in 0.6.1 (Window
Options). Checklist rows 12–16 were not individually reported.

### 0.5.7 — 2026-10-18: Streamer chord behind ZR

After the evening's first 0.5.6 online session the operator reported that
the opponent's connection did not show in the menu at all. The card's saved
`config.ini` (pulled over MTP afterwards, `switch-mtp-056/logs-online-1018/`)
had `streamer=true`; the previous boot's `config.previous.ini` had it too,
while the afternoon's 0.5.4 config had `streamer=false`. Streamer hides the
opponent column, the strip's peer facts and the native CSS remote line by
design, so the display was correct for its setting; the setting itself had
flipped unnoticed, most plausibly through the L+R + ZL/ZR + Y chord during
window use (ZL is also Training's controlled-fighter swap modifier, and the
session shows many window-chord presses). The live-peer rule was not
involved: the log has no code 112, the handshake and confirm fired, 494 RTT
samples arrived and the session cleaned up at the end.

Change: the Streamer chord is L+R with **ZR** held, then Y, the same hold
every window edit has needed since 0.5.6; the ZL variant and ZL+ZR together
do nothing (`src/overlay.rs`, `edit_modifier`). No INI change. The operator's
card config was set back to `streamer=false` and `input_buffer=auto` by hand
over MTP the same evening (receipt `switch-mtp-056/CONFIG-EDIT-2245.json`).
Host gates: 237 + 1 tests (the Streamer chord test now asserts the ZL variant
is inert), `cargo clippy --all-targets -- -D warnings`, `cargo build
--no-default-features`, Skyline package NRO SHA-256 `27cadcd719d549da5c0fc27cce9c7e752c6dcd17c20cebba443405ae039876c8`.
Released as Forgejo v0.5.7 (`282a2df`, docs `a7e8427`; NRO + manifest attached, download
read back) and installed on the card 2026-10-19 00:11Z over 0.5.6 with the rollback archive
`ultimate/ssbu_online_deluxe_rewrite/backups/pre-057-27cadcd719d549da.nro` (= 0.5.6, read back
before the write) and a verified after-readback; the saved config was not touched. Owed on the
console: L+R+ZL+Y changes nothing, L+R+ZR+Y toggles the Streamer row.

### 0.5.6 — 2026-10-18: Vanilla+ (default), UltraRes 1080p, ZR-hold for every window edit

Three changes in one release: Vanilla+ (present at the flush in the game's
three-image shape, on by default), the native 1080p option for console LLUltra
and the ZR hold for every window edit, plus a plan change (the FX measurement
cells are dropped). Vanilla+ is the Decision through Code paragraphs below; the
other items follow them.

**Decision.** Operator, 2026-10-18 about 20:50Z: ship it in the next version
as Vanilla+, on by default and toggled by L+R+X rather than a separate menu
entry. So the change is a flag, not a fifth profile: new
INI key `vanilla_plus`, default true, live. With it on, the Vanilla profile keeps
the game's own three-image swap chain but presents the acquired image at the
flush that carried that frame's last submission, one CPU update earlier than the
game presents it, exactly as the render-opts profiles do (LessLag since 0.4.0,
LLUltra and LLDoubles since 0.5.2). `render_mode` defaults to `vanilla` and
`auto_profile` to `true`, so the shipped default is now Vanilla+, and Auto's
Vanilla choice outside matches is Vanilla+ too. No console run of 0.5.6 exists;
nothing here is a console result for this build.

**Why it should be safe: code reasoning plus console traces of the previous
build.** The console API trace of 0.5.4 in Vanilla, Training at 60 fps, from the
2026-10-18 first-boot logs (`switch-mtp-055/logs-firstboot-1018/deluxe/api-3207722793418.csv`
in the evidence root), shows per loop iteration: PRESENT(idx), then
ACQUIRE(idx+1), then about 6 ms, then a single FLUSH span (the frame's last
submission, 4 µs after the late-flush marker), then about 9 ms of update, then
the next PRESENT of the index that was acquired at the previous loop head. So
in the game's three-image shape the acquired image is already complete at that
flush, and presenting it there saves the 9 ms or so of update that the game
waits through first, about 9–13 ms of display latency in the trace. The ring,
the acquire, the game's loop bookkeeping and the fighter simulation are not
touched. The 0.5.2 LessLag traces (`api-2214438549910.csv`) show the same index
relationship for the early present. The runtime presents `PENDING_INDEX`, the
index the loop-head acquire returned (`src/runtime/nvn.rs::present_at_flush`),
and the game's own present of that index is skipped through the existing
`EarlyPresent` and mismatch accounting (event code 105, unchanged). This is an
inference from the traces; the frame-time and latency effect of the change has
not been measured.

**Code.** `src/config.rs` (the `vanilla_plus` field, default true, parse slot
17, serialized last after `dispatcher_backoff`, tests for the default, the
round trip, rejected values and duplicates), `src/render_policy.rs`
(`present_at_flush(decision, vanilla_plus)` also allows `Profile::Vanilla` when
the flag is on; tests), `src/runtime/renderer.rs` (the flush hook passes the
flag), `src/runtime/mod.rs` (the `VANILLA_PLUS` atomic, `settings()`,
`publish()`, `vanilla_plus_enabled()`), `src/overlay.rs` (chord **L+R+X** with
neither ZL nor ZR held toggles the flag live and saves it, `ConfigSaved`, code
24; L+R+ZL/ZR+X, the fork's retired FPS++ chord, still does nothing; test),
`src/auto_policy.rs::profile_row_label(…, vanilla_plus)` (tests) with callers in
`src/runtime/ui.rs`, `src/runtime/native_banner.rs` and a `src/menu.rs` test,
and `config/default.ini` (`vanilla_plus=true`). UI: no new NetProfile entry; the
NetProfile row, the in-match strip's `Profile:` field and the native CSS
banner's `You:` field read `Vanilla+` (manual), `Vanilla+*` (cleanup hold) and
`Auto:Van+` (Auto) with the flag on, and plain `Vanilla`, `Vanilla*` and
`Auto:Van` with it off. `early_present=false` still disables every early
present, Vanilla+ included. The peer exchange still announces Vanilla, because
the ring shape is unchanged. Version 0.5.5 → 0.5.6 (Cargo.toml and Cargo.lock).

**UltraRes 1080p** (requested by the operator, 2026-10-18 21:16Z).
Code: `src/config.rs` adds `UltraResolution::P1080` (key `1080`, label `1080p`,
`Level::P1080`); the Settings row's `cycle` walks 576p → 720p → 1080p → 576p
with Right and backwards with Left (it swapped two values before); the INI
error reads `ultra_resolution must be 576, 720 or 1080`. 1080 had been dropped
at the operator's request when the option was added in 0.5.3 and is now back at
the operator's request. `src/runtime/mod.rs` stores the choice as a byte
(`ULTRA_RES`, `UltraResolution::to_byte` / `from_byte`: 0 = 576, 1 = 720,
2 = 1080) instead of the old 720 boolean (`ULTRA_720`). The default stays 576;
only console LLUltra reads it, emulator profiles and the other console profiles
ignore the key, and dynamic resolution still steps down from the chosen base
(1080p → 720p → 576p, 576p and below → 480p), so at 1080p only the cut-in, move
table and adaptive triggers lower it. The config test covers all three values
(parse, serialize round trip, byte round trip, cycle in both directions,
labels) and rejects `900`, `480`, `720p`, `1080p`, an empty value and `true`.
Reasoning, **operator-reported and uncaptured:** an overclock held 720p
LLUltra at a locked 60 (Horizon OC 2.6.0, installed 2026-10-16; the operator's statement,
no capture, so the build, clock state and moves played are unrecorded and it is
not a result of this repository). The 0.5.3 analysis put LLUltra's limit at
CPU work plus GPU time inside one vblank (GPU about 3.4–5.3 ms at 576p); if an
overclock lets 720p fit, the native level is the next thing to try. Whether
1080p fits is unmeasured, on a stock or an overclocked console. Host-tested
only.

**ZR hold for every window edit** (operator, same request: holding ZR is
required to change a value whenever the menu is open, including outside
Training and matches). `src/runtime/ui.rs` now sets `ui.controls.edit_hold =
true` in every scene; through 0.5.5 it was set only while the scene was ready
and not Inactive, that is while a fight was live (the 0.5.3 rule). With the
window open, a Left/Right value change on any Settings row or plugin row needs
**ZR held without ZL**; a plain Left/Right is ignored and Up/Down still move
the selection. Reason: menu D-pad presses and taunts must never change a
setting. ZR and not ZL because ZL + D-pad Left/Right swaps the controlled
fighter in Training. Unchanged: the window-closed native pane edits (ZL +
D-pad on the online character select and arena pane, 0.4.1). The gate is the
existing overlay test `edit_hold_needs_zl_for_value_changes_but_not_for_navigation`;
the runtime change is the one assignment and has no new test, and the
`Overlay::edit_hold` documentation was updated. Host-tested only; the fork
edits on a plain press, so this is a deliberate difference
([parity](FORK_PARITY.md)).

**FX measurement cells dropped** (operator, same request: the full effects
testing will not be done). Cells A–R of the test plan (Kirby, Pyra and
Falcon On and Reduced baselines and variants, the `drs_adaptive` and
`drs_moves=all` resolution comparisons, the leftover LessLag and Gigaflare
cells and the online LLUltra cell) are no longer planned and no result is owed
for them. The move ledger stays in the evidence root as `MOVE-LEDGER.md`; the
option and variant code is unchanged.

**Host gates:** `cargo test` 237 + 1 pass, `cargo clippy --all-targets -- -D
warnings` clean, `cargo build --no-default-features` ok, for the whole 0.5.6
tree. Skyline package NRO SHA-256 `df757ce2f8130fddd0c2ba69e35c0f2dd7eae5505365ce969ec88134a2fadad9`. Released as Forgejo v0.5.6 (release
395, NRO and manifest attached, download readback matched) and installed on the card the same
evening over 0.5.5 (`backups/pre-056-df757ce2f8130fdd.nro` keeps 0.5.5). **First console boot of 0.5.6 (same evening, operator; logs pulled over MTP to `switch-mtp-056/logs-online-1018/`):** clean boot, hooks installed, clock 1642–1788 MHz under Horizon OC. Vanilla+ ran in the menus, the character select and Training: `renderer-status.txt` ended at `present_calls=89500 early_presents=89483 early_present_skips=89483 early_present_mismatches=0`, so on hardware too every early present was matched by one skipped game present. One online Quickplay Singles match ran in manual LessLag with a locked render cadence (`timing-3592420657006.csv`: 217 s, p50 16.667 ms, p99 16.871 ms, 40 intervals above 20 ms mostly at the start and the results transition, two mid-match hitches of 63 and 77 ms, 5 one-second windows below 59 of 217) while the hang trace shows the simulation stalling and catching up (fighter `main` ticks 106–168 per second against the nominal 120, presents flat at 60): what the operator reported as the most unstable match was the connection, with a manual `input_buffer=3`, not frame drops; the API trace shows no GPU or sync stall. The missing opponent column was `streamer=true` in the saved config (see the 0.5.7 entry). LLUltra 1080p and the ZR-hold round trip were not exercised.

**Eden smoke of the Vanilla+ build (emulator evidence, 21:02–21:17Z).** The
Vanilla+ code of `ee5c6ce` (dirty-tree package `96a03725…`, before the 1080p
and ZR commits) on LAN-Online profile a with manual Vanilla, `vanilla_plus`
default, platform console, one Eden with an 8 GiB cap, an Astra-low driver
on the menus. `renderer-status.txt` at the end: `present_calls=60699
early_presents=59203 early_present_skips=59202 early_present_mismatches=0`,
`mode=vanilla`, `engine_status=Active`, three-image index masks: every early
present was matched by one skipped game present, and the only frames without
one are the ones while the flag was off. The native character-select banner
read `You: Vanilla+ | Auto`; L+R+X on the CSS saved `vanilla_plus=false`
(`24 ConfigSaved` twice), the banner read `You: Vanilla | Auto`, and in the
12 s with the flag off there were 908 presents but 54 early presents (those
before the press); L+R+X again restored both. 60 fps on the main menu, the
CSS hand sweep, 30 s of Training and a Training hot swap (Link → Yoshi,
first Training frame about 14 s after Plus); no tearing, stale frame or
flicker in any reviewed screenshot; no `105`, `48` or panic row. Evidence:
`~/.local/share/ssbu-mod-backups/plugin-api-tangent-20261012/vanillaplus-1018/`
(`SMOKE.md`, `screens/`, `profraw/al-a7/final/`, `renderer-status-final.txt`).
The 1080p option cannot be exercised in Eden (emulator profiles ignore the
key) and the ZR hold is host-tested; both are console checks.

**Owed on the console** (none of it done): with the 0.5.6 NRO on the card,
(1) menus, the character select and Training in Vanilla+ show no visual fault:
no tearing, stale frame or flicker; (2) the Performance page keeps 60;
(3) `renderer-status.txt` shows `early_presents` advancing in Vanilla with
`early_present_mismatches=0`; (4) L+R+X flips the NetProfile row text between
`Vanilla+` and `Vanilla` and the saved `vanilla_plus` line in `config.ini`;
(5) a subjective menu and hot-swap latency comparison with the chord off
against on; (6) 1080p LLUltra at 60 under Horizon OC, with a capture: Training,
`CAPTURE_TIMING`, UltraRes set to 1080p, no one-second window below 59 outside
the Deluxe-window-open seconds, the capture's `# clock=oc` header kept out of
the ledger; a 720p capture under the same overclock would turn the operator's
uncaptured 720p statement into a baseline; (7) the ZR-hold round trip in a menu:
open the window on the title or main menu, plain Left/Right changes no value
(the row stays put, no `config.ini` write), ZR + Left/Right changes it, Up/Down
still move the selection, and the same in the character select, Training and a
match.

### 0.5.3 — 2026-10-17: latency response trials repeated on candidate package `5357da2626a98fb9` (Eden) — frame reduction unchanged

The operator asked for the test to be repeated to confirm the reduction
candidate keeps the same input-latency frame reduction. Same method,
Eden, isolated profile and scripts as the benchmark entry below (evidence
root `bench-latency-5357-1017/`: `RESULTS.md`, `summary-table.md`,
`reviewed-trials.json`, `trials/`, `ui/`, `DRIVE-LOG-BENCH.md`,
`staging-receipt.json` restored and verified). Every API CSV header carries
`build=0.5.3-962eb3d1d19f-1792252459`.

Included trials, R = images from injection to the first crouch image, normal
phase / delayed phase 8: Vanilla 6, 6, 6, 6 / 7, 6, 6, 7; LessLag 3, 3, 3, 3 /
3, 3, 3, 3; LLUltra 2, 2, 2 / 3, 2, 2; LLDoubles 4, 4, 4 / 4, 4, 4. Savings
against the same-session Vanilla are three, four and two images, unchanged
from 0.5.2 and from the first candidate run; every included trial satisfies
R = M + D. Eight trials were excluded for a nonconsecutive present identity in
the critical window (host capture hiccups) and replaced by extra groups; no
manual pose overrides. The Vanilla delayed phase read 7 on two of four
trials (present delta 3 instead of 2 on the three-image ring), variance of
the Vanilla baseline that the earlier runs also showed once. The quota
frame-rate, capture and plugin-page checks were not repeated on this
package; their code paths changed only in the window drawing and the pane
text, which the console run covers.

### 0.5.3 — 2026-10-17: per-file reduction pass complete — candidate package `5357da2626a98fb9` (host tests, Skyline build)

The three per-file plans (`peer_transport.rs`, `pia_rtt.rs`, native banner +
window; entries below) are merged on `reduction-candidate` 962eb3d. Totals
against `main` be82f50 (code-audit classifier, comments/blanks/test modules
excluded): Deluxe `src/` 22,253 → 15,332 production lines (−6,921, −31.1 %);
`sdk/` 861 → 754; test lines inside `src/` 11,008 → 7,886; 68 → 60 files.
Per file: `peer_transport.rs` 1,388 → 739, `pia_rtt.rs` 1,010 → 637,
`runtime/pia_rtt.rs` 364 → 226, `native_banner.rs` 544 → 256,
`runtime/native_banner.rs` 1,364 → 689, `runtime/ui.rs` 1,182 → 820
(operator targets of ≈ 700 / ≈ 400 / ≈ 800 for the three groups: the first met,
the second and third not; what remains is guard windows, consumer-facing
types, ImGui call plumbing and event-record packings, and each entry below
lists the features that would have to go to reach the figure). Full table:
evidence root `reduction-1017/LOC-BY-FILE.md`.

Gates on 962eb3d: 233 host tests + SDK round-trip, host clippy with warnings
denied, no-default build, rustfmt, Skyline release build
`0.5.3-962eb3d1d19f-1792252459`: NRO 1,437,696 bytes, SHA-256
`5357da2626a98fb9776237774cecf473280272d2c2e3fc48deffe2511bda78b5`, module
`e79d250c843c3725…`, ELF `32768a2ad5a9f002…`. Package: evidence root
`artifacts/5357da2626a98fb9/`; staged with the unchanged FX Control candidate
`0c7e626d…` in `candidate-1017/` for the operator's Switch run. The SDK and
ABI did not change in this pass, so the FX Control build from the benchmark
entry still pairs with it.

Behaviour changes carried by this pass, all recorded in `docs/FORK_PARITY.md`
and `docs/NATIVE_DISPLAY.md`: one-shot announcement write (no revert on a
racing cleanup); a contention tick on busy UI snapshots; the first RTT sample
is kept; more than sixteen fresh RTT stations evict the stalest; one-shot
Streamer restore on the native panes; the local-wireless lobby pane shows the
input buffer behind the marker and restores the game's own count text in
Streamer (the fork's `nn::ldn` hooks are not reproduced). Not run in Eden or
on the console. The console run owed from the benchmark entry now also has to
show: CSS and arena pane text and Streamer restore, the lobby pane in both
Streamer states, the window's tables and in-match strip, and the transport
handshake / RTT / names / announce.

### 0.5.3 — 2026-10-17: `pia_rtt.rs` cut to sixteen keyed stations and one publication lock (host tests, Skyline build)

Plan: evidence root `reduction-1017/PLAN-pia-rtt.md` (items A–J). Commit
e7fce5c on `work/pia-rtt-1017` (from `reduction-candidate` a6e4a01).

Production code lines (code-audit classifier): `src/pia_rtt.rs` 990 → 641
(tests 623 → 379, 27 → 20 tests, one new: a seventeenth fresh station evicts
the stalest); `src/runtime/pia_rtt.rs` 370 → 230;
`src/runtime/connection_transport.rs` 305 → 291 and `src/runtime/hooks.rs`
66 → 81 (one `guard_matches` instead of two copies). Net −488. The plan's
host target of ≤ 500 was not reached: about 160 of the remaining lines are
the four 64-byte guard windows and the declared types the consumers use
(`Observation`, `PingGap`, `PingRgba`, `RttIdentity`, `Publication`,
`StationBoard`, `PresentedPing`, …), and rustfmt lays every kept
five-parameter signature (`present_published`, `present_station`,
`aggregate_stability`, `Publication::from_reading`) out one parameter per
line. Removed: the 253-slot table (`STATION_SLOTS`, ~107 KB) for
`STATION_CAP = 16` slots keyed by station index, `Session::new_boxed`,
`StationBoard::empty`, the `PING_*` colour consts, `format_*_ping`,
`rtt_published`, `stability_rank`, the `REASON_*` consts with `reason_of` /
`gap_of` (`PingGap` is `repr(u8)` and gains `Off`), the unread counters
`rejected_negative` / `rejected_index` / `rejected_overflow` / `leaves`,
`Ring.sum` and the test-only `latest`, `fresh_stamp` / `stamp_for` /
`fresh_peer`, the four `accept_*_for` entry points (one `Session::accept`
with the update closure), the `Default` impls; in the runtime `WANT_SESSION`,
the boxed `Option<Session>`, the two seqlocks (`PUB_*`, `BOARD_*`, five
`[AtomicU64; 8]` arrays) for one `Mutex<Shown>` written by the poll and read
with `try_lock`, and the "station table allocated" log line. Every
consumer-facing name and signature is unchanged (`native_banner.rs` tests use
`ping_rgba(Stability::Stable)` for the deleted `PING_STABLE`).

Behaviour changes, recorded in `docs/FORK_PARITY.md`: the first RTT sample
of the process is stored instead of dropped while the table was being
allocated, and more than sixteen simultaneously fresh stations evict the one
with the oldest sample (arenas hold eight). The ~7 KB `Session` is a static
again (`.data`), replacing the 107 KB heap table of 0.3.7.

Gates: 242 host tests + SDK round-trip, host clippy with warnings denied,
no-default build, rustfmt, Skyline release build `0.5.3-e7fce5ca3833-1792249232`
(NRO 1,445,888 bytes, SHA-256 `4076fe8275b91b2a…`; +4 KB over the
`peer_transport.rs` cut's `f5b60530…`, page rounding of the static table).
Not run in Eden or on the console; the owed console checks for the transport
(handshake, RTT, names, announce) now also cover this cut.

### 0.5.3 — 2026-10-17: `peer_transport.rs` cut to its production surface (host tests, Skyline build)

Operator order: plan each file separately and reduce its code without losing
functionality or just moving the code to other files. Plan:
evidence root `reduction-1017/PLAN-peer-transport.md`. Commit 7e6097e on
`reduction-candidate` (plus ec250b4 docs, 26620e4 rustfmt-only fixes in
`clock_probe.rs`, `coverage_profile.rs`, `runtime/clock_probe.rs`).

Production code lines (code-audit classifier, comments/blanks/test modules
excluded): `src/peer_transport.rs` 1,378 → 739 (tests 1,132 → 499, 25 → 8
tests covering the same behaviours); `src/peer_passive.rs` 261 → 258;
`src/runtime/connection_transport.rs` 342 → 305; `src/runtime/pia_rtt.rs`
360 → 370 (the station list that `publication_targets` used to build). Net
−669. Removed: the `View` intermediate and its u8 code tables
(`settings_code`, `label_from_code`, `display_latency`), the four separate
gate-entry copies (one `with_table` remains), `PublicationTargets`,
`RequestParse`/`ConnectionRequest`/`Direction`/`PendingStore`, `AnnounceMode`
(a 1:1 copy of `render_policy::Profile`), `encode_v3` (inlined),
`TransportCounters`/eleven counter statics (one `[AtomicU64; 10]` indexed by
`Counter`), `spare_tag_offset` (shares `peer_passive::beacon_tag_offset`),
`Table.epoch`, the test-only production functions, and in the runtime the
`SavedBody` save/revert path. Every consumer-facing name and the `ShownPeer`
fields are unchanged; `settings_label` now returns the final `++` label.

Behaviour changes, both recorded in `docs/FORK_PARITY.md` (Names / peer
settings row): the send hook applies the planned beacon and v3 bytes once and
no longer saves and reverts them when a cleanup races the store (they are the
rewrite's own announcement in its own packet, already addressed to the
departing peer); and UI snapshots that meet a busy gate now count a
`Contentions` tick (surfaces in `TransportSkip`), which they did not before.
No packet or display behaviour changed.

Gates: 249 host tests + SDK round-trip, host clippy with warnings denied,
no-default build, rustfmt, Skyline release build `0.5.3-26620e452923-1792247628`
(NRO 1,441,792 bytes, SHA-256 `f5b60530b6005163…`; the 4 KB growth over
`3f8855a2…` is page rounding, the `.text` shrank). Not run in Eden or on the
console; the owed console checks for the transport (handshake, RTT, names,
announce) now also cover this cut.

### 0.5.3 — 2026-10-17: reduction candidate ecb9923 benchmarked in Eden (latency, quota frame rate, captures, plugin pages)

The operator asked for the frame-reduction and correction benchmarks to be
rerun. Candidate package `artifacts/3f8855a2bec44ecc/` (build `0.5.3-ecb99238f02b-1792210040`,
NRO 1,437,696 B) with the FX Control candidate `artifacts-fx/0c7e626d64f35128/`
(SDK 5.0.0). All four passes are Eden measurements with Astra low driving the game
and the primary running the scripts; evidence root `bench-reduction-1017/` (RESULTS.md,
summary-table.md, reviewed-trials.json, trial images and CSVs, screenshots, receipts).

- **Latency response trials, the README table (frame reduction).** Method and tools of
  the 0.5.2 run (LATENCY_METHODOLOGY.md; Eden CLI-control `10bcd2d849`, isolated
  profile, Training, Sephiroth vs Kazuya CPU Stop, `INPUT_RECEIPT`, 640×360 captures,
  four profiles × two injection phases × three 20-image trials, replacement groups for
  the two groups with a nonconsecutive window). Included R, 0.5.2 in brackets: Vanilla
  6, 6, 6 / delayed 6, 6, 6, 7, 6 [6, 6, 6 / 6, 6, 6]; LessLag 3, 3, 3 / 3, 3, 3, 3
  [3, 3, 3 / 3, 3, 3]; LLUltra 2, 2, 2 / 2, 2, 2 [2, 2, 2 / 2, 3, 3]; LLDoubles 4, 4, 4 /
  4, 4, 4 [4, 4, 4 / 4, 4, 5]. Three trials excluded for a nonconsecutive present
  identity (raw R 3, 3, 7), one manual pose review (reference image caught the fighter
  settling; crouch at image 11, R 6). The savings (3 / 4 / 2 captured frames) are
  unchanged. Profile switches needed ZR held (the 0.5.3 live-fight rule, which the
  0.5.2 run predates).
- **Frame rate under the console-calibrated quota.** Same Eden, profile and INI as the
  2026-10-16 clips, `CPUQuota=175%`: character select 59.971 / 60.042 / 59.980 fps,
  Training (Pyra vs Terry specials) 59.962 / 60.049 / 60.015 fps, against 59.98 / 60.04
  / 60.02 and 59.98 / 60.06 / 59.99 on 2026-10-16. Performance page 60 and 58 fps,
  Backoff 266/s and 1554/s at 50 µs.
- **Capture correctness.** With `CAPTURE_CSS` and `CAPTURE_MATCH` present the
  consolidated writer produced five CSS captures (30 s, 1,798–1,800 samples), a Training
  capture (144.5 s), a Singles match (99.7 s) and the attract-mode Doubles demo (26.5 s),
  every file footer `# complete=1`, and the unchanged `tools/analyze_timing.py` and
  `tools/analyze_render_trace.py` parsed all of them; the latency run's ten 30 s
  captures likewise. The group script learned to wait for the footer, because the new
  writer streams from arm time. Console repeat of one timing, one CSS and one match
  capture still owed for the parity claim.
- **Plugin pages.** SDK 5 demo and FX Control candidate loaded together: FX registered
  `99 0x1 0x4`; page cycle Settings → Plugins → FX Control → Performance → hidden;
  Demo edits persisted (`enabled=on`, `mode=b`), FX fighter toggle and move variant
  persisted and reverted through the INI, headers skipped by the selection. Not
  regressions: the demo's Status row cycles Ready → Degraded → Failed by design; Training
  draws the vertical tables (the compact strip is for ready Singles/Doubles); the tall FX
  own page overlaps Training's "Total Damage" label at the right edge (pre-existing).
  A version-2 registration in Eden was not run (host test covers the replay plugin's
  shape).

Profiles restored and verified after each session; no Eden, display or unit of another
agent was touched. The candidate is not on the card; the operator's Switch run with the
FX passes is the acceptance.

### 0.5.3 — 2026-10-17: captures and traces consolidated (host tests and Python analyzers only)

Operator order after the capture plan (evidence root
`reduction-1017/PLAN-captures.md`, a read-only investigation of 690dc57):
consolidate the "captures and traces" bucket towards the plan's floor while
keeping every capability the console used in the 15 Oct campaign. Worktree
branch from main 74991cf.

- **Module layout.** `src/capture.rs` (pure, host-tested) replaces
  `src/timing.rs`, `src/render_trace.rs` and `src/capture_export.rs`: the
  constants and the kept kind constants (12–17, 27, 66, 77), `thread_label`,
  `image_offset`, `request_label`, `Finish`, one `Ring<T: Entry>` with
  `Sample` (callback row) and `Event` (API row) as its two 32-byte row
  types, `Stream`-keyed `header`/`rows`/`footer`, and the status phases.
  `src/runtime/capture.rs` replaces `runtime/timing.rs`,
  `runtime/match_capture.rs` and the arming, visit and status parts of
  `runtime/css_capture.rs`: one `PENDING` request byte (match > controller
  > CSS > `CAPTURE_TIMING` file), one `FLAGS` byte for `CAPTURE_CSS` and
  `CAPTURE_MATCH`, `note_css_frame` (code 12, CSS arming and the match
  end-by-CSS; `online.rs` no longer calls a separate `note_css`, because the
  same character-select hook reaches `note_css_frame` on every update,
  before the pane walk that gated the old call), `note_stage`/`note_scene`,
  the three Debug rows and the `Service` that streams both CSVs.
  `runtime/css_capture.rs` keeps only the two hooks (dependency wait, UI
  job) and their install; `runtime/api_trace.rs` gains `point`.
  `outstanding_after_present` moved to `render_policy.rs` with its test;
  `logging::rotate` is shared with the hang trace.
- **Dropped by name.** Trace kinds 60/61 (the per-frame backoff rows,
  `backoff_frame_events`, `frame_rows`, the `FRAME_*` aggregates and the
  timed branch in `dispatcher_backoff.rs`, the `nvn.rs` call), 62/63
  (`describe_thread`, `GET_PRIORITY`/`GET_CORE_MASK`, `THREADS`, `core_info`),
  64 and the `INIT_WORKER` hook (`child_of_ui_aggregate`, `dispatch_info`,
  `SUBMIT_WORKER`/`UI_AGGREGATE`), 28 (`contains_resizing`, `resize_event`,
  `RESIZE_COUNT`; the scan of every backend log line is gone), 29–34
  (`FlushFields`, `FlushPlanDiagnostic`, `flush_field_events`,
  `read_flush_fields`; the field store in the flush hook stays, gated on
  `+0x1d18` directly), `ticks_to_us`, `pointer_halves`, `bits_u32`; event
  code 89 (`MenuOrder`, reserved by comment); `Finish::Capacity` (a full
  ring counts rows as `skipped` and never ends a capture); the two-chunk
  20-byte `.bss` packing (`Packed`, `Chunk`, `take_chunk`,
  `CHUNK_HALF_RANGE`); the `[HangTrace] first <hook>` lines (`first_line`,
  `note_first`, `first_tick`, `reported_first`) and the `css_init`
  heartbeat field; the duplicate `rotate` and `TICK_HZ`; the test-only
  production code (`Snapshot` ×2, `write_csv` ×2, `u32_bits`, `Default` ×2,
  `export_ready_csvs`, `controller_request_busy`, `write_buffered`,
  `Stream::closed`).
- **Race fix (plan §4.3; inferred, never observed in a capture).** The old
  `take_ready` discarded rows still in the buffer but reported `total`, so a
  hook that finished the capture between the poll's `drain` and its
  `take_ready` lost its last rows while the footer counted them, and the
  analyzers would have rejected the file. `Ring::take_ready(out)` now hands
  the final rows over under the same lock that counted them (test
  `take_ready_hands_over_the_final_rows_so_the_count_matches`). Drains swap
  buffers, O(1) under the lock, instead of copying.
- **Byte-identical.** `timing-<id>.csv` (schema 2) and `api-<id>.csv`
  (schema 1) including the clock lines after the column line, pinned by the
  golden test `golden_csv_bytes_for_both_streams`; the `timing-status.txt`
  strings; the heartbeat line minus `css_init=`; the four triggers (Debug
  chord, `CAPTURE_TIMING` label, `CAPTURE_CSS` entry, `CAPTURE_MATCH`
  stage); the `Timing`, `CSS capture` and `Match capture` rows (the CSS
  row's mask wording is now 0x3); the export (`create_new`, 64 KiB buffer,
  flush, `sync_all`, Exporting before any CSV work, Saved only after both
  footers and the saved status). `tools/analyze_timing.py`,
  `tools/analyze_render_trace.py`, `tools/frame_windows.py` and their tests
  are unchanged (30 passed, one pre-existing skip). Code 104's `a` is now
  the two-bit mask (bit 0 wait, bit 1 job; 3 = both) and `b` keeps only bit
  0; `diagnostics.rs` and DEBUGGING.md say so.
- **Behaviour changes called out.** Capacity no longer ends a callback
  capture; API rows are skipped only when the 4096-row ring is full (the
  old two-chunk headroom or better); the API storage moved from 80 KiB of
  resident `.bss` to the heap at arm (0 idle; 272 KiB peak, as before; a
  low heap now refuses the arm visibly with `timing capture could not be
  armed`); a match request arriving under a 30 s capture replaces, rather
  than queues behind, a pending controller request; an arm that fails to
  open the API file no longer also creates the timing file. The ring is
  generic over the row type (the plan's second option in §3.3 item 4), so
  `buttons` keeps all 64 bits instead of being truncated to 32.
- **Lines** (production = before `#[cfg(test)] mod tests`, base 74991cf):
  bucket 2,436 → 1,579 (−35 %). `capture.rs` 483 (plan about 240; was
  `timing.rs` 364 + `render_trace.rs` 630 + `capture_export.rs` 91),
  `runtime/capture.rs` 562 (plan about 300; was `runtime/timing.rs` 446 +
  `match_capture.rs` 105 + about 70 of `css_capture.rs`),
  `runtime/api_trace.rs` 45 → 52 (plan about 35), `runtime/css_capture.rs`
  333 → 115 (plan about 60), `hang_trace.rs` 162 → 148 (plan about 75),
  `runtime/hang_trace.rs` 260 → 219 (plan about 110). Elsewhere −84:
  `renderer.rs` −38, `dispatcher_backoff.rs` −31, `overlay_host.rs` −17,
  `response_probe.rs` −6, `nvn.rs` −3, `logging.rs` +1, `diagnostics.rs` +4,
  `render_policy.rs` +6. Net −941 production lines; `git diff --stat`
  against 74991cf: 31 files, +1,612 / −3,172. The plan's floor of about 820
  was not reached: its §3.3 estimates undercounted rustfmt'd, documented
  Rust by about 2× (every §3.3 item is implemented, nothing beyond the §3.1
  kept capabilities remains, and the §3.6 cuts were declined), and the
  number was not chased by stripping comments or compressing formatting.
- Host tests 274 → 265 (20 deleted with the old modules, 10 new in
  `capture.rs`, one moved to `render_policy.rs`, the five `hang_trace.rs`
  tests adapted to nine hooks and no first lines). Gates: `cargo clippy
  --all-targets -- -D warnings`, `cargo test` (265 unit + 1 integration),
  `cargo build --no-default-features`, `cargo fmt -- --check` (clean except
  `clock_probe.rs`, `coverage_profile.rs` and `runtime/clock_probe.rs`,
  which were already unformatted on main and are left alone), `python3 -m
  unittest discover -s tools -p "test_*.py"`. The Skyline build is the
  parent's: `runtime/` compiles only there, and nothing here is claimed as
  runtime-verified. Console parity is still owed: one `CAPTURE_TIMING` 30 s
  capture, one `CAPTURE_CSS` capture and one Training `CAPTURE_MATCH` on the
  card, each analysed with the unchanged tools, plus continuing
  `hang-trace.log` heartbeats.
### 0.5.3 — 2026-10-17: plugin API host and window pages consolidated; SDK 5.0.0 (host tests only)

Operator order after the reduction plan
(`reduction-1017/PLAN-plugin-api.md` in the evidence root, §4a, §4b, §5, §7,
§8). Branch off 74991cf. Nothing here is runtime-verified.

- **Version policy.** The host accepts descriptor versions 2, 3 and 4 under
  one rule set (the version 4 rules: every option flag, the own-page plugin
  flag, 64 options) and refuses 1 with `REGISTER_ERR_ABI`
  (`abi::MIN_ABI_VERSION = 2`, `descriptor_version_supported = 2..=4`). The
  three versions share one 56-byte row layout, so this is strictly more
  permissive for every known sender: smash-replay-to-mp4 (version 2, five
  rows, SDK pin `fa479944`) and FX Control 0.1.x copies (version 3) keep their
  rows unchanged. Version 1 was only ever sent by unreleased 2026-10-12 SDK
  builds. The per-version flag and limit tables, `OptionDescV1`/`widen`,
  `MAX_OPTIONS_V3`/`V2` are gone.
- **Host rewrite** (`src/plugin_api.rs`): one `RowKind { Value, Status,
  Header }` replaces the `locked`/`header`/`inline` bools; one `copy_str`
  replaces `bounded_cstr`/`owned_id`/`owned_label`/`owned_checked`;
  `parse_descriptor` returns a `Plugin` (no `ParsedPlugin`); `Registry::value`
  is a pure getter (pending is cleared and persisted only by
  `reconcile_pending`, run before every draw); `PageRow { Group, Option }`,
  `rows_for`, `selectable`, `move_selection` and one `wrap` replace
  `PluginRow`, `plugin_rows`, `plugin_page_rows`, `cycle_choice`,
  `move_plugin_row`, `row_selectable`, `move_selectable_row`,
  `clamp_selectable_row`; `compact_page`/`vertical_window` merged into
  `menu::window`, `compact_segments` moved to `menu::strip_segments`,
  `compact_label`/`compact_line`/`inline_pieces`/`count`/`persisted_value`
  dropped. Pending-request handling stays (the replay plugin's "Stop rec" row
  answers `Pending`, verified in its source).
- **Window pages** (`src/runtime/ui.rs`, Switch-only, compiled by reasoning):
  one `menu::Row` model feeds the vertical table and the strip for Settings
  cells and plugin rows alike; `draw_rows` is the one 2- or 3-column drawer
  (`draw_settings_vertical` and `draw_plugins_vertical` build its input);
  `plugin_page` asks `current` only for the windowed rows; `rgb` + `GREEN` /
  `YELLOW` / `RED` / `GREY` consts; `setting_cells` is closure-driven with one
  `dash()` and a toggles table, returning the strip's remote facts instead of
  the strip finding cells by label; `perf_facts(strip)` is the one source for
  the Performance table and strip; `draw` keeps input, polling and records and
  `draw_window` the frame and body dispatch; `auto_driver::snapshot()` runs
  once per draw; the quiet-ping copy is gone (`performance` takes
  `Option<&PresentedPing>`). Rendered text is unchanged except as listed
  below.
- **Visible changes** (also in `docs/FORK_PARITY.md`): version 1 refused;
  versions 2/3 read under the version 4 rules; `OPTION_FLAG_INLINE` draws as
  a value row; the shared Plugins page has a group row per plugin, so the
  strip shows the plugin label as a header segment and then the plain option
  labels (was `Plugin.Option`, or the plugin label alone for a one-option
  plugin) and the selection index counts group rows (first option is row 1);
  a pending change applied between `reconcile_pending` and the draw shows
  `(pending)` one frame longer; code 99 logs one line per registration.
- **SDK and ABI 5.0.0** (removed public items): the SDK sends version 4 only
  and calls the host once; the version 3/2/1 retry chain, `v1_option_descs`,
  `NEGOTIATED_VERSIONS`/`negotiated_descriptor_version`, `needs_version_4` and
  `RowLayout::Inline` are gone; `option_flags` lost its version parameter.
  The exported symbol, `TABLE_VERSION`, the 72-byte `PluginDesc`, the 56-byte
  `OptionDesc`, every flag bit, tone byte, error code and `ONLINE_*` are byte
  for byte as before; `ssbu_online_deluxe_online_context_v1` and the
  `PluginApiV1` table are untouched. Demo: drops the
  `negotiated_descriptor_version` log.
- **FX Control must change** (per the plan's §5; done by the parent in
  `/home/intpa/ssbu-fx-control`): `Cargo.toml:22` bump the SDK `rev` to this
  commit; `src/runtime/plugin.rs:7-10` drop `negotiated_descriptor_version`
  from the import and `:38` record `b` as
  `u64::from(deluxe_plugin_sdk::abi::ABI_VERSION)` (or drop it; code
  `PluginRegistered`'s `b` meaning changes, update its diagnostics doc if
  present) and `:52` the println; `src/menu.rs:346` delete the
  `needs_version_4` assertion and `:347-350` replace the `RowLayout::Inline`
  loop with "every non-header row is `RowLayout::Value`". Runtime behaviour
  does not change (FX Control already needs a version 4 host); ship FX
  Control and Deluxe together. The replay plugin needs nothing.
- **Production lines** (physical lines before the first `#[cfg(test)]`, as
  the plan counts): `src/plugin_api.rs` 903 → 664 (plan ≈300; the plan's
  figure left out the ≈200-line `Registry` it keeps and rustfmt's layout of
  `parse_descriptor`); `src/runtime/plugin_export.rs` 138 → 135 (plan ≈100;
  `flush_persistence` and the online export are untouched by order);
  `src/runtime/ui.rs` 1,235 → 1,170 in all, plugin drawing ≈220 → ≈165
  (plan ≈60) and Deluxe's own drawing ≈1,015 → ≈1,005 (plan ≈790: every §4b
  idea is applied, but rustfmt expands the table-driven cells and fact lists
  back to one field per line); `src/overlay.rs` 369 → 371 (plugin navigation
  27 → 22; plan ≈30 for the whole bucket); `src/menu.rs` 479 → 531
  (`CompactField`/`compact_fields_line` 30 out, `Row`/`strip_segments`/
  `strip_line`/`window` 82 in; plan ≈45); `sdk/abi/src/lib.rs` 345 → 306;
  `sdk/sdk/src/lib.rs` 828 → 703; `tests/plugin_sdk_roundtrip.rs` 243 → 236.
  `git diff --stat 74991cf`: 15 files, +1,578 / −2,143.
- **Tests.** Host 274 → 275 (`plugin_api` 17 incl. the new
  `replay_plugin_version_two_descriptor_registers_with_rows_tones_and_pending`:
  a raw version 2 descriptor shaped like the replay plugin's five rows, two
  persisted toggles, a six-choice toned status row, a tone-only row, an
  editable `no_persist` row answering `Pending`; `menu` gains
  `windows_page_the_strip_and_centre_the_table` and the `strip_rows_…` rewrite
  of the `CompactField` tests; overlay row indices shift by the group row),
  plus the SDK round-trip (1), SDK 23 + 1 doc-test, ABI 2. Gates: `cargo
  clippy --all-targets -- -D warnings`, `cargo test --workspace`, `cargo
  build --no-default-features`, `cargo fmt -- --check` (clean on every touched
  file; the four pre-existing unformatted files are left alone), `cargo test`
  in `sdk/sdk` and `sdk/abi`, `cargo build` of the demo. The Skyline build of
  the NRO and the demo NRO are the parent's.
- **Still owed:** an Eden run with the demo plugin as the replay plugin's
  stand-in (built at `e4ef935`, SDK 2.0.0, sends version 2: shared page with
  its group row, tones, read-only rows, `config_path` writes) and with the FX
  Control page (own page, headers, 33 rows, persistence of `scope` and the
  fighter rows), and the console. Claim nothing beyond the host tests until
  then.

### 0.5.3 — 2026-10-17: removed the opt-in peer observer, FPS++ and the Gigaflare DRS trigger (host tests only)

Operator order after the coverage campaign: three features go, with no other
behaviour change. Branch `work/reduction-1017` from main 690dc57.

- **Opt-in peer observer** (`peer_observer`, default false, never enabled on
  any card or Eden profile). `src/peer_observer.rs` and
  `src/runtime/peer_observer.rs` are gone. The runtime file was not
  observer-only: it also carried the live connection transport (the
  parsed-message hook feeding `observe_incoming`, the station-index hook
  binding stations, the pre-encryption send hook committing announcements,
  the nifm interface byte and the code 66–72 counters). Those parts moved
  unchanged to `src/runtime/connection_transport.rs`, and the hook sites,
  guards, `mix_id`, `kind_of`, `header_span` and `message_body_range` they
  read to `src/connection_transport.rs`. Removed: the observation ring, the
  station map and its generation table, the `invalidate_index` /
  `invalidate_session` calls from `runtime/pia_rtt.rs`, the station-probe
  flags, the send-site probe (`diagnose_send`, `SendBuild`, the 17-event
  report, `stamp_send_seq`), name decoding, `inherit_header`, and the
  `drain_station_probe` / `drain_send` publishers. Event codes 37–45, 64 and
  65 were only ever recorded by the observer (verified: codes 41–45 reached
  the log only through `publish_send`, which ran under the flag) and are
  retired by comment in `diagnostics.rs`; 66–72 stay. The stale
  `peer_transport.rs` comment claiming the station-map hook was part of the
  observer is corrected (it always installed on its guard alone).
- **FPS++** (`fps_boost`, `ALLOW_FPS_BOOST`, emulator-only, reported
  Unsupported in every normal build). `src/fps_boost.rs` and
  `src/runtime/fps_boost.rs` are gone, with the NVN window
  initialize/finalize/present callbacks, the Settings row (`ROW_FPS`; the
  `ROW_*` constants are renumbered, `ROW_COUNT` 11 → 10), the Debug-view
  row, the L+R+ZL/ZR+X chord, the `++` suffix on the local profile label in
  the window and both native banners, the `fps_boost_status` and
  `fps_boost_applied` lines of `renderer-status.txt`, and event code 88
  (retired by comment). `Announce.boost_applied` and the `announce_from`
  parameter are gone, so our v3 payload never sets fork bit 14; a peer's bit
  14 is still decoded as `++` (test `peer_bit_14_still_decodes_as_boost_and_is_never_sent`).
- **Sephiroth Gigaflare DRS trigger.** `perf_scaler::sephiroth_tick`, the
  `edge` slots, `EDGE_KIND` / `EDGE_SPECIAL_N_SHOOT` / `EDGE_CHARGE_KIND` /
  `CHARGE_FULL`, `MainTick.charge` and the `WorkModule` charge read in
  `runtime/fighter_main.rs::read_tick` are gone; `on_main` now returns the
  cut-in action and the move-table pair. The cut-in trigger, the 0.5.2 move
  table (entry, kind and status are still sampled every tick) and the
  adaptive trigger are untouched. Code 82 keeps source 1 (cut-in) and 3
  (move table); source 2 is retired by comment. Reason: the 2026-10-15
  console redo had Vanilla and LessLag hold 60 through the move.
- **Legacy keys.** `config::parse` rejects unknown keys and the runtime would
  then reset the user's INI, and existing cards carry `peer_observer=` and
  `fps_boost=` lines. Both are now ignored legacy keys like `effects` /
  `effects_scope` (any value, any order, duplicates allowed) and the writer
  no longer emits them; test
  `removed_peer_observer_and_fps_boost_keys_are_ignored_and_not_written_back`.
- **Lines** (`git diff --stat main`: 31 files, +885 / −4,507; src alone
  −4,434). Dedicated modules, split at their `#[cfg(test)]` boundary:
  peer observer 1,981 production + 735 test lines deleted, of which 536
  production lines (147 host + 389 runtime) are the re-homed transport, plus
  91 new transport tests; FPS++ 805 production + 473 test lines deleted.
  Shared-file edits (approximate, by hunk): peer observer about 57 production
  and 20 test lines, FPS++ about 128 production and 93 test lines (menu,
  overlay, config, UI, banners, NVN status, transport, diagnostics),
  Gigaflare 88 production (76 `perf_scaler.rs`, 11 `fighter_main.rs`, one
  comment in `diagnostics.rs`) and 37 test lines. Host tests 300 → 274.
- Skyline release package `artifacts/7ea17849ef97f044/` (build
  `0.5.3-49fbfe169695-1792208004`, NRO 1,441,792 B, SHA-256
  `7ea17849ef97f0442ce731df9f4bc25a1a59ea9b75b54251583ec90fd19fdff1`),
  built by the primary after reviewing the commit; 20,480 B smaller than the
  previous package. Not deployed to the card; the first console run is the
  parity check for the re-homed transport (handshake, announce, station
  binding, RTT display) and for the cut-in trigger without Gigaflare.
- Gates on this commit: `cargo clippy --all-targets -- -D warnings`, `cargo
  test` (274 passed), `cargo build --no-default-features`, `cargo fmt --
  --check` (clean on every touched file; `clock_probe.rs`,
  `coverage_profile.rs`, `runtime/clock_probe.rs` and `runtime/timing.rs`
  were already unformatted on main and are left alone). The Skyline build and
  any console or Eden run are the parent's; nothing here is claimed as
  runtime-verified.

### 0.5.3 — 2026-10-17: two-host NSO session — arena, Quickplay, Background and Co-op matches reached; category A is live code

The operator pointed out that Quickplay and arenas need two Eden instances and
asked this thread to run both, so it started the companion itself on the Surface (its
owner's bounded launcher, own unit names, one-hour cap 01:48–02:48Z), ran the
Yoga coverage peer on NSO profile `b` (account 49154, coverage build
292a402a…, LessLag, `platform=console`, player 2 enabled) and drove the Ally's
fixture slot `a` (account 49153, the owner's tested Windows profile) over ssh
through the owner's own harness (scheduled-task launch into the desktop,
named-pipe control client, renderer capture). Two Astra low drivers, one per
peer. Display fix first (same evening): the Qt software-keyboard dialog that
had segfaulted weston's Xwayland three times survives on a **rootful Xwayland
without weston's window manager** (`coverage-nso-1016/display-rootful.ps1`;
offline proof on the Stage Builder name prompt, then live on Create Arena).
Reached on the coverage peer: Quickplay Solo match 1 (dropped for inactivity
while the driver worked the overlay; hooks ran), **public arena COV created
from the Yoga through the Qt dialog, Ally joined, round 1 played to results**,
round 2 cut by the Ally launcher's 30-minute cap, arena closed; **Quickplay
match 2 complete with results**; **Background Matchmaking matched from the
main menu and completed**; **Co-op 2v2 complete** (Dark Samus + Ness vs Shulk
+ Corrin); Debug-view manual snapshot; collect. Result
(`tools/coverage/report.py`, `compare.py` against the 2026-10-15 table): 927
of 1,466 functions executed; **29 functions / 464 lines executed that no
campaign had reached** — exactly category A (arena room init/update, arena
status/parts/text, room-id decode, `banner_peers`, the PIA station board).
They are live; nothing new to remove. 300 functions (2,640 lines) never ran in
either campaign and are all accounted for (removed tiers, FPS++, DRS, plugin
option edits/own pages, the opt-in peer observer, arena-name bookkeeping).
**Finding for review:** the scene-entry hooks `deluxe_rewrite_arena`
(0x22d9fa0) and `deluxe_rewrite_background_matchmaking` (0x22da090) never fired
through a real arena lobby and a real background match, and
`deluxe_rewrite_quickplay` (0x22da160) fired in one process (nso-a3) but zero
times in the two-host process despite four searches; matches were still
classified through `stage_setup` and the character-select classifier
(`mark_match` 58 times), so the product behaved, but the Arena/Quickplay
connection distinction did not come from those hooks here. Harness notes:
same-host two-peer pairing fails (2618-0502); the Ally's run cap is 30 minutes
(relaunch under a new phase takes a minute); "No" on rematch prompts was never
explicitly captured. Evidence: evidence root `coverage-nso-1016/` (REPORT.md
§0, NOTES.md, DRIVE-LOG*.md, `profraw/nso-b2/`, `report-two-host/`,
`compare-two-host/`, screenshots of both peers); both Yoga profiles restored
byte for byte, the Ally's tasks removed, all Edens and displays of this thread
stopped; the companion units expire on their own.

### 0.5.3 — 2026-10-17: NSO coverage campaign on the LAN-NSO Eden (no new removal evidence)

Operator request: finish the code-reduction measurement against the other
agent's "LAN NSO" Eden (commit 289c4684e7, private companion for auth,
matchmaking and NAT check) so the Quickplay, Arena and Doubles paths that no
emulator session had reached get coverage. Coverage build
`artifacts/292a402ab785fec9/` (0.5.3 be82f50, `"variant": "coverage"`),
NSO profile `a` staged with it (LessLag, `platform=console`, player 2 enabled),
Astra low drivers, three runs of peer a and one of peer b (profile `b`, second
address on the same host), companion window 00:29–01:29Z. Reached: Online →
Smash with the window shown and cycled, Quickplay Solo and Co-op waiting rooms
(window shown there too), Background Matchmaking started, kept alive through
Training and cancelled, Spectate ("Failed to play replay"), the Create Arena
settings screen, the public arena list, a Debug-view manual snapshot. Not
reached: an arena lobby (Create Arena → Next opens the game's text-entry
applet and Eden's applet window segfaults the headless Xwayland on the Yoga,
three times, taking Eden down with `std::bad_alloc`) and a connected Quickplay
match (the companion paired the two peers but the connection failed with
2618-0502 "One or more other consoles are not responding"; two addresses on
one host behave like the fixture's unsupported same-IP case). Result
(`tools/coverage/report.py` over four scenario sets, joined by function name
with the 2026-10-15 table): 774 of 1,466 instrumented functions executed; **no
function executed here that the 2026-10-15 campaign had not executed**; 329
functions (3,104 lines) never ran in either campaign, and every one is already
accounted for (Tier 1 and Tier 2 removed on main, FPS++ gate, DRS experiment,
plugin option edits/own pages, and the ~750 lines of category A arena/match
code that needs a lobby or a connected match). Category A stays. Next NSO
attempt needs a display that survives the applet window and a second host as
the peer. Evidence: evidence root `coverage-nso-1016/` (REPORT.md, NOTES.md,
DRIVE-LOG*.md, profiles, reports, receipts; both NSO profiles restored byte for
byte). Tier 2 gates on the merge 55d2c74: `cargo clippy --all-targets -- -D
warnings`, 300 host tests, `cargo build --no-default-features`, Skyline
release package `artifacts/c408b41dc14f6c61/` (build
`0.5.3-55d2c7402673-1792198537`, NRO 1,462,272 B, SHA-256
`c408b41dc14f6c61e8704a70529687b8f77a23997ebe8ceeab9d777794b70a07`); not
deployed to the card.

### 0.5.3 — 2026-10-16: retired five opt-in diagnostics

The coverage campaign's Tier 2 probes are removed. They last shipped in
`be82f506cd33ef8c8dc017e754597d45893c1ecd`. Retired: `DUMP_PRESENTATION_HISTORY`
(the `nn::vi` frame-history dump; Eden returned no usable presented timestamps,
and the CSV captures plus present-at-flush superseded it), `PROBE_FIGHTER_MODULE`
(the sixteen-word event and `fighter-module.txt` dump; the `common` load/unload
observation and logical Main arm stay), `DUMP_RENDER_PROBE` (the 5 October
read-only range dump), `PROBE_CROP` (the code-83 SetCrop log; the crop wrapper
and fullscreen rearm stay), and `dispatcher_backoff.ini` (the override parser;
the shipped default policy stays, with `dispatcher_backoff=false` and
`DISABLE_BACKOFF` as the off switches). Event codes 75, 76, 77 and 83 are no
longer emitted. Flag-absent hooks, defaults and the other event codes are
unchanged. No console run of this commit.

### 0.5.3 — 2026-10-16: `early_present` and `dispatcher_backoff` switches, Eden before/after clips (branch `docs/early-present-backoff-toggles`)

Operator: make the early presentation and the dispatcher backoff switchable
in the INI, record before/after GIFs on the character select and with
effects under LessLag, and explain how the emulator is made to show the drop.
Commit d10fce5: `early_present=true|false` (the flush wrapper skips
`present_at_flush`) and `dispatcher_backoff=true|false` (the installed hook
returns on every idle pass; Backoff row `Off (ini)`), both default true,
read at start, carried through the settings atomics; `DISABLE_BACKOFF`
unchanged. Commits 7635c5c / 6591d6b: README "Before and after, on the
emulator" and `docs/EMULATOR_METHOD.md` (the CPU-quota bias and its
limits). Clips: Eden GIF build on the private display, isolated profile,
build `0.5.3-d10fce5eeb59` (`artifacts/389b94963c3412f3/`), LessLag docked
1080p, driven by Astra low sessions from `toggle-gifs-1016/BRIEF.md` and
`BRIEF-calibration.md`. A first pass at a 120 % CPU quota gave 35 → 58 fps on
the character select and 29 → 52 in Training (73/73/69/72 frames, 0/0/3/1
dropped); the operator asked for a quota tuned to the console, so the quota
was swept with both switches off (175 % → 52 fps, 160 → 46, 150 → 43,
140 → 38, 130 → 35 on the hand sweep) and **175 %** chosen as the value
that reproduces the console's 49.9 fps; with both switches on the same quota
gives 60.0 (console 59.85). Final clips at 175 % (72/73/73/73 frames, none
dropped): character select 50–53 → 60.0 fps,
Training with Blazing End vs Terry 52 → 59 fps, Backoff `Off (ini)` →
2 782 / 1 398 sleeps/s, `early_presents` 0 → one per present. The operator
found the recorder's 640 px GIFs too compressed to read, so the four scenes
were recorded once more at 1280 px (same quota, same build, Pyra vs Terry;
73 frames each, none dropped; `hd-toggles-*.gif` in the kit) and the
published `docs/media/toggles-*.gif` are composed from those by
`compose_gifs.py`: Lanczos downscale to 640 px, the Performance panel pasted
at 2× from the full-resolution frame, one palette per clip. Evidence
root: `toggle-gifs-1016/` (NOTES.md, CALIBRATION.md, q-renderer-*.txt,
q-config-used-*.ini, driver-evidence.jsonl). Host tests 310, clippy `-D warnings`,
`--no-default-features`, Skyline release build. **Emulator only; not on the
card; not merged** — the operator reviews the branch first.

### 0.5.3 — 2026-10-16: clock probe for the ledger, L+R-only window chord, bundle tool (host tests only)

Operator: the LLUltra work must run at stock clocks, and after Horizon OC
2.6.0 went back on the card ("did you add oc detection just in case we
accidentally keep it enabled?") nothing recorded the clock state; also "I
don't want ZL+ZR to activate it, just L+R". Commit 8dde0f2:
`clock_probe` checks the Horizon-OC files at boot (`atmosphere/kips/hoc.kip`,
hoc-clk `boot2.flag`, a `[01006A800016E000]` section and `uncapped_clocks`
in `config/horizon-oc/config.ini`) and times 2^20 dependent adds against the
system tick every 5 s on the diagnostics thread (one cycle per dependent add
on the A57: about 1020 MHz stock, 1785 under the game's loading boost or an
OC profile; a preempted sample reads lower, never higher). Event code 111
(a = MHz, b = HOC bits) at boot, on every stock/OC change and once a minute;
`# clock_mhz` / `# clock` / `# hoc` header lines in both capture CSVs
(`tools/analyze_timing.py` reads `# key=value` anywhere); `clock_mhz` /
`clock` / `hoc` in `renderer-status.txt`; Performance row **Clock**. The
window chord is L+R only: ZL+ZR no longer cycles the window, opens Debug or
cycles the peer. `tools/bundle.py` assembles the two SD-layout release zips
(plugins only; plugins + Horizon OC from the upstream release asset with a
hekate snippet, README and hashed manifest) outside the repository.
Evidence: 309 lib tests (HOC bits/labels, config parse, MHz estimate and
classification at 1020/1785/1224, header and status lines, overlay tests
asserting the triggers do nothing), clippy `-D warnings`,
`--no-default-features`, Skyline release build (the asm loop compiles for
aarch64); package `artifacts/e4718545d048c434/` (build
`0.5.3-8dde0f28167c-1792175817`, NRO 1,486,848 B, SHA-256
`e4718545d048c4347d04288ca80b3f24e52e52ff8c8282d722527c2db353f12e`)
deployed to the card over DBI MTP next to FX Control `0.2.0-0cffbac9d16e`.
Bundles `bundles-1016/ssbu-online-deluxe-0.5.3-8dde0f2-bundle.zip` (4 files)
and `…-bundle-hoc.zip` (33 files, Horizon OC 2.6.0-rel for Atmosphère
1.12.1) in the evidence root, not attached to a release. **Not yet booted;
the 1020/1785 reading is a calculation from the A57 add latency, not a
console measurement until the first boot's event 111 is read.**

### 0.5.3 — 2026-10-16: `hide_pages` + page rows, `perf_view` + PerfView row (host tests only)

Operator requests after the first good 0.5.3 boot (Kirby Hammer cells 1–4
run): FX Control second in the window cycle with the other two pages off
for now, every page except Settings toggleable in-game with Settings
showing which are hidden, and the frame-time graph selectable against a
plain number to trim draw work while measuring. Commits a1e3475 and
71015d5: INI `hide_pages=none|plugins,own_pages,performance` and Settings
rows **PluginsPage / OwnPages / PerfPage** (Shown / Hidden, live, ZR-gated
in a fight like every row); `next_view` takes the `HiddenPages` and drops
the shared Plugins page, every plugin's own page and/or Performance from
the chord walk (Settings is always in it, Debug keeps its chord, a stale
Performance view still hides); INI `perf_view=graph|number` and row
**PerfView**, which replaces the 180-sample `igPlotLines` block at the
bottom of every vertical view with the mean as text (the compact match
strip was text already). Both carried through the runtime's settings
atomics (`HIDE_PAGES` bits, `PERF_NUMBER`); the first Skyline build of
a1e3475 failed on that literal again, 71015d5 fixes it. Evidence: 305 lib
tests (config parse/serialize/reject for both keys, `next_view` with each
flag and all three, menu row order on console and emulator, row toggles,
overlay walk Settings → Hidden with the INI value), clippy `-D warnings`,
`--no-default-features`, Skyline release build; package
`artifacts/2ff8ba576ce7bb78/` (build `0.5.3-71015d54ad09-1792171175`, NRO
1,482,752 B, SHA-256
`2ff8ba576ce7bb784ecc4fd3de2500daddac7900a37ac8b101e387982308cb4b`)
deployed to the card with `hide_pages=plugins,performance` and
`perf_view=graph` in the card INI, next to FX Control `0.2.0-ee00c5c736f4`
(ledger-only page, value rows, EFF siblings off). **Not yet booted.** The
cost of the plot itself is unmeasured; PerfView gives the operator the
switch, not a claim.

### 0.5.3 — 2026-10-16: `ultra_resolution` and the UltraRes row (host tests only)

Operator question: is LLUltra's 576p drop doing anything, and would 720p
hold? From the 2026-10-15 console traces the drop is what lets LLUltra fit
CPU work plus GPU time inside one vblank (GPU ≈ 3.4–5.3 ms at 576p against
≈ 10 ms of idle work; LessLag and Vanilla pipeline the GPU and run 1080p),
while the move drops are CPU work that resolution cannot touch; the GPU
term at 720p was never measured. Commits 7fd057c and e432de1 add the
measurement: INI key `ultra_resolution=576|720` (default 576) and a
console-only Settings row **UltraRes** (last row, ZR + Left/Right, applies
on the next present through `sync_resolution`); `resolution::defaults_for`
and `active_setting` take the Ultra base level, only (Ultra, Console) uses
it, dynamic resolution still steps down from it, and the peer announce
carries the configured level. 1080 was dropped at the operator's request
(it came back in 0.5.6, see that entry).
Evidence: 301 lib tests (config parse/serialize/cycle, menu row visibility
and edit on console vs emulator, policy defaults for 576/720/1080 bases),
clippy `-D warnings`, fmt, Skyline release build; package
`artifacts/48cce4f2b0a5a7d4/` (build `0.5.3-e432de1bf1a5-1792161003`, NRO
1,478,656 B, SHA-256
`48cce4f2b0a5a7d48cc7f7ecb3d25120e7ebc629c56980858daf14eda578f12e`)
deployed to the card. **Not yet booted.** The first Skyline build of
7fd057c failed on the runtime's atomic-backed `settings()` literal (the
host tests do not compile `runtime/`); e432de1 carries the field through
`ULTRA_720`. Capture plan: Training stage, Mario, 10 s idle + 20 s
fireballs at 576p and at 720p, and Kirby Hammer at 720p
(`STAGE-CAPTURES-CHECKLIST.md` in the evidence root, cells 13–15).

### 0.5.3 — 2026-10-16: boot crashes after the plugin API v4 deploy; bootstrap hook chaining and SDK header fix (host tests only)

Operator report: a few crashes at first after the first 0.5.3 + FX Control
0.2.0 deploy; the third boot came up. Two Atmosphère reports
(01792152191, 01792152247) and both plugins' install/event logs for the
crashed and the good boot were pulled to the evidence root
(`switch-mtp-fx02/logs-1016/`, analysis `CRASH-ANALYSIS-1016.md`, read-only
GPT-6 Astra pass with exact-build symbols).

- **Crash 1 (Skyline thread, memcpy in Skyline+0x3500):** Skyline's
  inline-hook relocator decoding another plugin's `ldr x17; br x17; .quad`
  stub at `nvnBootstrapLoader` as instructions: the stack held the stub's
  address word, which decodes to a PC-relative load whose target is the
  faulting address exactly. `overlay_host::hook_bootstrap` recognised the
  far hook (Compatibility 4) but still asked for a trampoline. Commit
  e862970 plans the entry with `hook_chain::plan_far_chain` as the
  fighter-module probe does: Chain keeps the stub's handler as the previous
  callable and installs with a null trampoline; Relocate behaves as before;
  Reject or a failed install refuses the overlay host
  (`overlay_host_entry_malformed`). The hook is now a plain `extern "C"`
  function with its own previous-callable slot, and the embedded imgui host
  receives a chaining original. Entry words before and after and the
  previous handler are written to `install.log`.
- **Crash 2 (MainThread, strlen from cross2+0x37ec290, no plugin frames):**
  game initialisation; none of the return addresses is a Deluxe patch site,
  no host path hands a plugin string to game code (labels are copied at
  registration), and an older report (01792087152, a day earlier, older
  builds) has the same signature. Unattributed, intermittent; no change.
- **Side finding:** FX Control's 39-row page registered as **-3 (invalid)**
  in both boots (`99 0xfffffffffffffffd` in its install.log), so the
  operator never saw the variant rows. Cause: the SDK passed an empty Vec's
  dangling `as_ptr()` as the header rows' choice table where the host
  requires null. `register_with` and `v1_option_descs` now send null for
  empty tables; `tests/plugin_sdk_roundtrip.rs` registers a header + inline
  spec through the real SDK into the host registry.

Evidence: `cargo test --workspace` (300 lib + round trip), SDK 27, clippy
`-D warnings`, fmt, Skyline release build; package
`artifacts/1b361b7fdbb9b00b/` (build `0.5.3-e862970e373d-1792155058`, NRO
1,478,656 B, SHA-256 `1b361b7fdbb9b00be9b79e11febbe6306d4d318c9187bf301d43246d337a1623`)
deployed to the card. **Not yet booted**; the first boot with another
plugin's far hook on the bootstrap entry is the test of the chaining path,
and `install.log` will show `nvn_bootstrap_hook_before … plan=Chain(…)`.

### 0.5.3 — 2026-10-16: ZR hold for window edits while a fight is live (host tests only)

(Since 0.5.6 the hold applies in every scene; see the 0.5.6 entry.)

Operator request: the native character-select edits already need ZL held
(0.4.1); do the same in Training, so a setting is harder to change by
accident. Implemented for every live fight scene (Training, Singles,
Doubles; the runtime sets `Overlay::edit_hold` from the Auto driver's scene
snapshot when it is ready and not Inactive): with the window open, Left and
Right on a Settings row or a plugin row apply only while **ZR** is held
without ZL (ZL+ZR stays the window chord); a plain press is ignored. The
first build (01fd30e) used ZL; the operator reported that ZL + D-pad
Left/Right in Training swaps the controlled fighter, so the hold moved to
ZR before any capture. Up/Down move the selection as before, menus and the
character select keep plain edits, and the native pane gate stays on ZL.
Evidence: overlay test
`edit_hold_needs_zl_for_value_changes_but_not_for_navigation` (Settings
row, plugin row, ZL refused, chord exclusion, flag off); full library suite
green. Not yet run in Eden or on the console; the first run is the FX
Control variant pass on the card, where Left/Right on the variant rows need
ZR.

### 0.5.3 — 2026-10-16: plugin API v4, header and inline rows (host tests only)

Operator request: FX Control's measurement page should show, per fighter, a
non-selectable title row and under it one row per move with all of its
variant choices visible on one line (`[Original]  Variant 1  Variant 2`),
instead of `Label: Value` rows cycled with Left/Right. Commit ab79355 adds
descriptor version 4 to the plugin API without changing any struct layout:
`OPTION_FLAG_HEADER` (label only, `choice_count` 0, null tables, Up/Down
skip it, nothing persisted, callbacks never called) and `OPTION_FLAG_INLINE`
(every choice drawn on the row, the current one bracketed in its tone, the
others dimmed; Left/Right unchanged; compact strip unchanged), option limit
64 for v4 (32 for v3, 8 before). The SDK gains `OptionSpec.layout`
(`RowLayout::Value | Inline | Header`), `OptionSpec::header()`, and retries
4 → 3 → 2 → 1 only while the spec fits the lower revision (a v4 row is never
downgraded). Crates `deluxe-plugin-abi` and `deluxe-plugin-sdk` 4.0.0.
Evidence: `cargo test --workspace` 299 + roundtrip 1 (new: v4 parse,
header validation, per-version option limits, selectable navigation, inline
pieces, compact header), SDK 27 (REJECT_V4 fake host, retry chains,
header/inline flags), ABI 2; `cargo clippy --all-targets -- -D warnings`;
`cargo build --no-default-features`; Skyline release build, package
`artifacts/366490e47a893100/` (build `0.5.3-ab79355250c9-1792148005`, NRO
1,478,656 B, SHA-256 `366490e47a893100c7ab21a6036825492a618fc00a4686bd104058e2af3030ad`).
**Not run in Eden or on the console yet**; the first run is the FX Control
0.2.0 variant-pass deployment on the card (evidence root
`switch-mtp-fx02/`). Hidden-window frame timing is not expected to change
(page drawing only), but the console move captures from 2026-10-15 were on
build 1525e3e0, so any comparison across the two builds says so.

### 0.5.2 — 2026-10-15: response trials after present-at-flush for LLUltra and LLDoubles (Eden, captured images)

Operator request: re-measure the README input-delay table on the 0.5.2
present-at-flush build (commit ce0145a, package `artifacts/1525e3e0392c8403/`,
build `0.5.2-ce0145a2155c-1792070522`, NRO SHA-256
`1525e3e0392c840306073762cb7e78670745fe0e925de013f660c901371cfdff`), because
LLUltra's lead over LessLag might have shrunk. Method as in
LATENCY_METHODOLOGY.md: one uninterrupted CLI-control Eden `10bcd2d849`
(Vulkan/docked/1×/100%, `ui.screenshot_height=360`, 6 GiB cap) on a private copy
of the 0.5.0 profile (Deluxe only plus Training Modpack, gamepad overlay,
Arcropolis, css_preserve, nro_hook, param_hook; no FX Control), Sephiroth versus
Kazuya CPU Stop in Training, `INPUT_RECEIPT` marker, input buffer Auto,
`platform=emulator`, `auto_profile=false`. Four profiles × two injection phases
× three 20-image trials = 24 trials (480 native 640×360 images), normal-phase
order Vanilla, LessLag, LLUltra, LLDoubles, delayed order reversed; the
committed mode and `engine_status=Active` were read from `renderer-status.txt`
before and after every group. Captured-image response `R` (images from the
injection after image 5 to the first crouch image):

| Profile | Normal | Delayed | Saved vs Vanilla, normal / delayed |
| --- | --- | --- | --- |
| Vanilla | 6, 6, 6 | 6, 6, 6 | — |
| LessLag | 3, 3, 3 | 3, 3, 3 | 3 / 3 |
| LLUltra | 2, 2, 2 | 2, 3, 3 | 4 / 4, 3, 3 |
| LLDoubles | 4, 4, 4 | 4, 4, 5 | 2 / 2, 2, 1 |

All 24 critical windows had consecutive present identities (no exclusions, no
replacement trials); automatic and visually reviewed pose indices agreed in all
24 and `R = M + D` held in all 24. Present-at-flush did not change any captured
count versus the b19 and 0.5.0 runs; LLUltra still has one frame fewer than
LessLag in the normal phase (2 against 3, three of three) and is within one
frame of it in the delayed phase (2, 3, 3 against 3, 3, 3). Qualifications:
seven delayed-phase trials (Vanilla 1 and 2, LessLag 1, LLUltra 1, LLDoubles 1
to 3) are not covered by their API trace export (each export holds 1800
identities that start after those trials), so they rest on the marker's image
identities alone; on the 17 API-joined trials the delayed values are Vanilla 6,
LessLag 3, 3, LLUltra 3, 3 and no LLDoubles trial. Five traces carry "Engine
state changed during capture." and the LLUltra normal trace one skipped event;
not investigated, no API-level A/B claim. These are emulator captured-image
counts with synthetic input; they do not measure the console frame-time stall
that present-at-flush targets and do not establish Switch input-to-photon
latency. At the stop the unit recorded SIGBUS at the external
`systemctl --user stop` (after a clean `game.stop` and Eden's own clean
shutdown log), the known shutdown behaviour. Evidence outside Git:
`latency-052-1015/` (NOTES.md, per-trial analysis, 480 images, traces with
partial-copy verification, staging receipt); annotated GIFs
`docs/media/latency-{vanilla,lesslag,llultra}.gif` (normal-phase trial 1 of
each, frame counter from the first image after the injection to the crouch)
and, added 2026-10-16 on the operator's request, `latency-llultra-later.gif`
(LLUltra delayed-phase trial 2, `R` = 3: the case where LLUltra saves 3
frames rather than 4, with a plain-language note in the README).

### 0.5.2 — 2026-10-15: present at the flush for LLUltra and LLDoubles (candidate for the console LLUltra drop)

Operator report: LLUltra drops FPS sharply on the console (scene not
yet specified, no capture yet). Code reading: `render_policy::present_at_flush`
was gated to LessLag, so LLUltra, the other two-image ring, still presented
each frame after the next frame's whole CPU update, exactly the mechanism
the 0.4.0 analysis traced on the console character select (update over
about 13.7 ms → missed vblank → the acquire waits for the following one →
33 ms frames). LLUltra's 576p lightens the GPU and cannot touch that.
Operator decision: apply the fix to LLUltra and Doubles too. Commit ce0145a extends the predicate to every committed, idle render-opts
profile (LessLag, Ultra, Doubles; Vanilla and every swap, drain, forced-wait
or refused state still refused) and the flush hook calls
`nvn::present_at_flush` whenever the policy allows instead of only on the
LessLag field path, so Ultra's three-call flush presents the acquired image
too. Doubles gains the earlier present with its third image as slack. Not
measured anywhere yet; the Ultra early flush and skipped late flush are
unchanged, so visual checks (anything recorded after the early present) are
part of the console test. Gates on ce0145a: host Clippy with warnings
denied, 293 host tests (the present-at-flush test now asserts Ultra and
Doubles), no-default-features build, `cargo fmt`, Skyline Clippy with
warnings denied, `tools/Build.ps1` release. Package
`artifacts/1525e3e0392c8403/`: build `0.5.2-ce0145a2155c-1792070522`, NRO
SHA-256 `1525e3e0392c840306073762cb7e78670745fe0e925de013f660c901371cfdff`
(1,470,464 B, module `6f2750f4…a860`). Card install over the first 0.5.2
build via `switch-mtp-052b/deploy-052b-mtp-1015.py`; `config.ini` on the
card is the LLUltra baseline (manual LLUltra, both DRS experiment flags off)
so the first console run measures this change alone. Acceptance: an LLUltra
Training capture with Input Display off, intervals and the texture-sync
waits compared against the 0.5.1 LessLag Training capture; `early_presents`
≈ `early_present_skips`, zero mismatches (code 105), no visual corruption.

### 0.5.2 — 2026-10-15: per-move and adaptive dynamic-resolution triggers (experiment, on the card)

Operator order after the Training analysis and the console A/B, where Falcon
Punch barely changed and Pyra's side special dropped several frames, mostly on
hit: build the 0.5.2 experiment with both flags, with the trigger threshold
configurable in the INI. Commit 03bd237:
`src/drs_triggers.rs` (catalog of eleven named moves as fighter kind + status
sets from skyline-smash bed642e, `MoveSet` mask parsing, threshold parsing in
hundredths of a millisecond, the `Adaptive` over/under-budget state machine),
`src/perf_scaler.rs` (per-entry move slots and one adaptive slot on the same
report multiset as the cut-in and Gigaflare, `MainTick` input, counters),
`src/runtime/adaptive_drs.rs` (present-thread side: interval from the tracked
present ticks, 192 ticks per hundredth of a millisecond), config keys
`drs_moves`, `drs_adaptive`, `drs_adaptive_threshold_ms`,
`drs_adaptive_trigger_frames`, `drs_adaptive_release_frames` (all default
off/17.5/6/90, round-trip through the INI so menu saves keep them), event
code 108, `renderer-status.txt` settings and counters, Debug-view rows `DRS
moves` and `DRS adaptive`, compact-strip `Adp`. No smashline. Docs:
CONFIGURATION, DEBUGGING, RENDERING. Gates on 03bd237: host Clippy with
warnings denied, 293 host tests (eight new: catalog, mask, threshold, state
machine, move slots, adaptive slot, config round trip), no-default-features
build, `cargo fmt`, Skyline Clippy with warnings denied, `tools/Build.ps1`
release. Package `artifacts/9b9660ead3338d78/` in the tangent evidence: build
`0.5.2-03bd237d7d1c-1792068556`, NRO SHA-256
`9b9660ead3338d782dfcd27f03043efa1e55d351f47daf9d3d602ffc13239183`
(1,470,464 B, module `5a9e2e10…4e42`).

Card install, 2026-10-15 12:50Z: `switch-mtp-052/deploy-052-mtp-1015.py`
preflight and install over DBI USB MTP, `installed_verified` (the 0.5.0
release NRO archived on the card under `…/backups/pre-0.5.2-…/` and locally;
the other ten plugins and the protected files unchanged). The experiment
flags were then written to `config.ini` (previous copy saved as
`switch-mtp-052/config.ini.card-before`): `drs_moves=all`,
`drs_adaptive=true`, threshold 17.5 ms, 6 trigger frames, 90 release frames,
on the operator's manual LessLag. Not released, not tagged, no hardware
result yet. Acceptance per move or setting: a 30 s `CAPTURE_TIMING` on the
console with the trigger on and off, compared on intervals over 20 ms and on
the `Resolution` row; code 108 edges and the status counters show whether and
when each trigger fired.

### 0.5.1 — 2026-10-15: render-opts teardown guard, console test pending

Branch `post-scene-teardown-guard` (candidate `cc7f07f6…`, Eden evidence in
the 0.5.0 entry below: 22 Training → character-select exits without the
post-scene guest abort that hit the release five times in about 22 exits)
merged into main at 8aaef69 (`--no-ff`), version bumped to 0.5.1 at 231f4c8.
Change: `src/teardown_guard.rs` and `src/runtime/render_sequence.rs` skip
the relocated model update and the fighter worker on a frame in which a
fighter module unloaded (code 48 `RenderTeardownGuard`,
`teardown_guard_skips` in `renderer-status.txt`) and fall back to the game's
ordering for 180 scene frames; see `docs/DEBUGGING.md` and
`docs/PERFORMANCE.md`. Gates on 231f4c8: host Clippy with warnings denied,
284 host tests, no-default-features build, Skyline Clippy with warnings
denied, `tools/Build.ps1` release. Package
`artifacts/aee3852d905218e9/` in the tangent evidence: build
`0.5.1-231f4c82ec3a-1792064680`, NRO SHA-256
`aee3852d905218e922886d338e2540cd4dd379a25c23e5f4f32981b2a24642da`
(1,462,272 B, module `d60c4a2e…1fdf`).

Card install, 2026-10-15 11:45Z (the operator asked to test 0.5.1 before
the release): `switch-mtp-051/deploy-051-mtp-1015.py` preflight and install
over DBI USB MTP, `installed_verified`; the 0.5.0 release NRO is archived on
the card under `ultimate/ssbu_online_deluxe_rewrite/backups/pre-0.5.1-
20261015T114532Z/` and locally; the other ten plugins, `config.ini` (manual
LessLag, Auto off) and the protected files are unchanged.

Hardware result, 2026-10-15 ~12:10Z (tested by the operator; 0.5.1 passed):
repeated Training → character-select exits and a LessLag match on this NRO
with the full plugin set, Skyline `9959ecb1…` (latest skyline-dev release),
no crash. This is the first console run of the teardown guard; the guard
counter and code-48 events on the card have not been read back yet. Released
as v0.5.1 (tag at 231f4c8, Forgejo release with the NRO and manifest).
Observed in the same session and NOT addressed by 0.5.1: in Training the
frame rate fell into the 50s and 40s during Kirby's charged hammer, while
the zoom (the cut-in that the scaler already covers) produced no drop. That
is a candidate for the 0.5.2 per-move dynamic-resolution experiment. Planned
next: the same build on upstream Online Deluxe's bundled Skyline `d3630815…`
so neither user group has to change Skyline.

Training frame-rate analysis from the card, 2026-10-15 ~12:30Z (the operator
asked why LessLag ran in the 50s with no effects active). The
`CAPTURE_MATCH` flag on the card recorded the Training session as
`timing-1794782036751.csv` / `api-1794782036751.csv` (label `match_training`,
build 0.5.1, 165 s, 8,609 frames; Kirby vs CPU Corrin per the modpack's
`nro_load` lines). Result: median render-callback interval 17.2 ms, mean
19.2 ms, 23.4 % of intervals over 20 ms, frame rate per 10 s between 45 and
59; the hang-trace heartbeats of the same boot show 47–59 presents/s in
Training. Reference: the previous Training capture on the card,
`timing-958174278187.csv` (build 0.5.0, 2026-10-14, Falco vs Ness, same
plugin set and capture flags): median 16.67 ms, p99 17.1 ms, 0.4 % over
20 ms, 60 fps; the previous boot's heartbeats (0.5.0, earlier the same day)
show Training at 60.0 presents/s. Decomposition from the API trace: kind 3 is
the infinite `nvnSyncWait` on the swapchain texture sync. On 0.5.0 it
returned 1 (waited, about 3 ms) on 99 % of frames, the signature of a frame
finishing early and idling to vblank. On 0.5.1 Training it returned 0
(texture already free, no wait) on 53 % of frames and waited about 6 ms on
the slow frames. Per-frame work excluding that wait: median 16.8 ms, p90
20.0 ms on 0.5.1 Training against 13.8 ms / 14.8 ms on the 0.5.0 capture,
and the 10 s medians swing between 14.6 and 18.6 ms with play. The game's
UI job (kind 77) rose only 7.0 → 7.5 ms and the worker dependency waits
(kind 66) are unchanged per frame. Reading: the scene's frame work exceeded
the 16.7 ms budget by about 3 ms and varied with activity; the two-image
LessLag ring has no slack, so every late frame becomes a missed vblank
(Vanilla's three-image ring would show the same work as 60 fps with one
more frame of latency). Not attributable to the 0.5.1 change from this data:
the guard fired twice in the session (`teardown_guard_skips=2`) and its only
per-frame effect is the 180-frame fallback after a transition (1,040 of
12,512 frames here). Differences from the 60 fps capture that the logs do
show: different fighters (Kirby and Corrin vs Falco and Ness) and the
Training Modpack menu with hitbox visualization, frame advantage, input
display, stage hazards and save-state autoload selected; whether those
toggles were the same on 2026-10-14 is not recorded. Discriminating tests
for the operator: the same Kirby Training scene (a) with hitbox
visualization off, (b) on Vanilla, (c) on the archived 0.5.0 NRO (one
restore command). Evidence copies: session scratchpad `card-051/`.

Resolved by the operator, 2026-10-15 ~12:55Z: turning off Training Modpack's
Input Display, not the hitbox visualization, brought Training back to a locked
60 fps. The Training Modpack's Input Display
overlay was the ~3 ms of extra frame work. With it off, LessLag Training
holds 60 on the console; the operator is taking the cost up with the
modpack's maintainers. Consequence for the fork/upstream comparison above:
the upstream run that "stayed locked" had the modpack removed from the card,
so it never carried that cost, and the earlier "ours in the 50s versus
upstream at 60" impression compared different plugin sets. Deluxe-side
lesson, unchanged: the two-image LessLag ring exposes any per-frame work
over 16.7 ms as a missed vblank; an adaptive resolution trigger (0.5.2
experiment) is the mitigation that does not depend on naming the cause.

### 0.5.0 — 2026-10-14: character FX split into `ssbu-fx-control`; plugin own pages

The operator asked to move the FX mods into their own repository: a plugin
with its own window that turns effects on, off or reduced, with an option per
character plus an All option, leaving Deluxe's menu leaner. Decisions: rows only for covered fighters plus All;
Reduced only where a variant exists; repo `intpa/ssbu-fx-control`; the menu
API stays in Deluxe (imgui-smash stays a renderer).

Deluxe side (commits f3fe436, f571a30, 40d2234, f9c294d, 0213533, b0ca667):
plugin API descriptor version 3 (`PLUGIN_FLAG_OWN_PAGE`, 32 options, export
`ssbu_online_deluxe_online_context_v1`, SDK 3.0.0 with v3→v2→v1 retries);
the scene tracker became the `SCENE` hook family (`DISABLE_SCENE`) instead of
installing from the FX family; timing CSV schema 2 without FX columns; then
the FX feature removed (14 source files, `host/eff-variant`, the family
generator, the FX policy JSON, five FX docs, the FX row, seven Debug rows,
FX event codes; 12,707 lines; code 30 renamed `OnlineContext`,
`auto_driver::context_online`); `effects` / `effects_scope` INI keys are
accepted and ignored. 280 host tests, 25 SDK + 2 ABI tests, 26 analyzer
tests, clippy host + Skyline, no-default build, Skyline release build. The NRO
is 1,462,272 B (0.4.2: 1,531,904 B).

Plugin side (`intpa/ssbu-fx-control` main ad1ac6c): verbatim import of the
FX sources from Deluxe f3fe436, then a per-fighter decision (move → fighter
type → compile-time name-prefix map), own INI with a 2 s quiet-period writer
thread, own events/install logs with Deluxe's code numbers, online scope from
Deluxe's export or the plugin's own latency + stage hooks when Deluxe is
absent (never both; Scope Online never applies when no signal exists). 61 host
tests + 18 EFF-variant tests, all gates. NRO `e5cf868a…` (581,632 B, build
`0.1.0-3e7d0ecb6828-1792010538`).

Eden combined run, 2026-10-14 20:55–21:20Z (GIF build, isolated profile, both
NROs staged, Astra low driver `fx-split-eden-driver-1014`, evidence
`fx-split-1014/`): Deluxe 0.5.0 `8b1c936f…` + FX Control `e5cf868a…`. The
window cycle ran Settings → **FX Control** (own page, titled with the plugin
label; `01-fx-page.png`) → Performance; the page listed Scope, All, the 14
fighters and `EFF variants: Ready`; Deluxe's Settings had no FX row
(`02-deluxe-settings.png`). FX Control's install log: `online_source=Host`
(Deluxe's export), config created with defaults, 22 hooks installed. Scope
set to All through the page and persisted (`fx-config-after-scope.ini`).
Training with Sephiroth, Gigaflare charged and released under On, Off and
Reduced (`gif-fx-sephiroth-gigaflare-{on,off,reduced}.gif`, 73 frames each,
0 dropped): On shows the large white/yellow/purple burst, Off has no burst
but keeps hit sparks, fire and the launch trail, Reduced keeps the purple
crescents and flames with less white washout; damage identical (49.3 %, 8
hits) in all three. Editing one fighter made the All row read `Unavailable`
(by design: mixed rows); All = Off then All = On restored every row
(`fx-config-final.ini`). 59.98–60.01 fps, no crash or dialog.

Released 2026-10-14 21:12Z: Forgejo release v0.5.0 (id 374) with NRO
`d9dd7df281671c12…` (1,462,272 B, build `0.5.0-44830e5d9255-1792012237`,
readback verified) and FX Control v0.1.0 (release id 375, NRO
`505dfecd68330201…`, 581,632 B, build `0.1.0-eb627b89e654-1792012419`, SDK
resolved from the Deluxe v0.5.0 tag). Both NROs are the Eden-tested code
rebuilt from the tagged commits; only the embedded build strings differ.

Card install 21:15Z over DBI USB MTP (`switch-mtp-050/deploy-050-mtp-1014.py`
preflight → install, readback verified): Deluxe `87322eef…` (0.4.2) replaced
by `d9dd7df2…`, `libssbu_fx_control.nro` added; rollback archive under
`ultimate/ssbu_online_deluxe_rewrite/backups/`; `config.ini` untouched
(manual LessLag, input_buffer 3, auto off, streamer off; its `effects` keys are
now ignored). Flags `CAPTURE_CSS` + `CAPTURE_MATCH` remain. Console run of
0.5.0 + FX Control not yet performed.

Eden GIF campaign for the FX Control README, 2026-10-14 21:27Z onward, on
the release NROs (Training, Scope All, 6 s clips per state): two Eden
instances died at the Training → character-select transition. Run 1
(22:49Z, after 48 clips): host SIGBUS with memory pinned at the 8 GB cgroup
cap, no guest panic logged. Run 2 (~23:30Z, after 66 clips): **guest abort**
— `SetTerminateResult 2001-0114` (kernel `InvalidHandle`) from
`nn::os::LockMutex` inside the game's model-update routine (`main+0x3ab5c4`)
called from `deluxe_rewrite_post_scene`, i.e. the LessLag render-opts path
that relocates the model update into the post-scene hook, while fighter NROs
were unloading. Evidence: evidence root `fx-gifs-1014/run-2/GUEST-PANIC.md`
and `guest-panic.txt`. Intermittent (the same transition succeeded many
times earlier that day); not reproduced on purpose; no fix made or tested.
The same render path is on the console in 0.5.0 (unchanged since
2026-10-06). Run 3 (14 GB cap, memory peak 8.19 GB) died 23:46Z with the
identical backtrace at the same transition after 75 clips, so host memory
pressure is not the trigger: three Eden deaths in about twelve Training →
character-select transitions this evening, all with FX Control 0.1.0, the
Training Modpack and the gamepad overlay staged next to Deluxe. Which of the
loaded plugins, or the GIF-build Eden, makes the race observable is not yet
established; a controlled Training → character-select cycling test with and
without FX Control is the next step before any mitigation. Later that night
the host-side Eden deaths (runs 1 and 4, and a latency run) were traced to
the private display service's 1 h runtime limit taking the X server down;
only runs 2 and 3 are guest aborts.

Input-delay re-measurement on the release NRO, 2026-10-15 00:35–01:45Z
(operator request after a PK Thunder recovery stayed at 60 fps under LessLag
on the console): same method and Eden build as the b19 table, 18 trials,
640×360 captures. Included trials: Vanilla R = 6, LessLag R = 3, LLUltra
R = 2 (normal) and 2–3 (delayed) — three and four fewer captured response
frames, unchanged from b19. Four trials excluded for one missing present
identity each; details in `docs/LATENCY_METHODOLOGY.md` ("Re-measurement on
0.5.0"). Emulator captured-image frames, not console input-to-photon.

Card experiment, 2026-10-15 01:53Z (operator order): the rewrite 0.5.0 and
FX Control NROs were removed from the card and the ORIGINAL fork
`libssbu_online_deluxe(nuevo).nro` (`21af1ac0…`, 1,441,792 B) with the
original `libimgui_smash.nro` (`61e19e1b…`, 1,261,568 B) installed over DBI
MTP, readback verified (`switch-mtp-fork-swap/deployment.json`); the three
replaced files are archived on the card under
`ultimate/ssbu_online_deluxe_rewrite/backups/fork-swap-20261015T015254Z/`
and locally in `switch-mtp-fork-swap/before/`; `restore` mode reverses it.
Purpose: compare the original fork's frame drops on PK Thunder recoveries
with the rewrite's.

Outcome, 2026-10-15 10:00–11:45Z (evidence root `switch-mtp-fork-deps/`,
`switch-mtp-fork-solo/`, `switch-mtp-upstream-143/`, crash reports in the
session scratchpad): the fork never reached a match on this console. Smash
first crashed at launch three times (Instruction Abort, PC 0, LR in the fork's
`main+0x838`) because the fork's dynamic imports `ssbu_pia_manager_*` were
unresolved; the swap had installed only the fork and its ImGui host. With
`libssbu_pia_manager.nro`, `libssbusync.nro` 1.3.0, `libnx_over.nro` and
`libsmashline_plugin.nro` added (the set that ran next to the fork until
2026-10-06) and the fork configured for offline LessLag, the game aborted at
the versus splash (User Break in game code, no plugin frame) the moment the
render options engaged; with Training Modpack, gamepad overlay, replay and
Elite removed it failed two boots (an init-time null identical to a
2026-10-13 boot fault seen with the rewrite installed, then a null fiber
stack in a fighter-script coroutine). The UPSTREAM mod,
saad-script/ssbu-online-deluxe v1.4.3 (release zip `5640c63a…`), installed as
its installer script would produce it (latest ARCropolis 4.1.1 = the card's
`92171b2f…`, nro_hook 0.4.0, imgui-smash 1.0.0, pia-interface 1.3.0,
smashline 1.6.8, upstream's own ssbusync 1.3.2, nx_over and Skyline
`d3630815…`; overclock sysmodule omitted, `overclocker = false`), crashed
once at boot while the card still had the older smashline `d472c596…` (null
in a task worker, no plugin frame) and then reached an offline LessLag match
after smashline was swapped to 1.6.8. The operator observed upstream holding
60 fps through special zooms by lowering the internal resolution; no
frame-time capture of that run exists. All swaps were restored in order
(11:41–11:42Z, each `restored_verified`): the card is back on the 0.5.0
plugin set, the rewrite's ImGui host `946de84f…` and Skyline `9959ecb1…`,
which is byte-identical to the latest skyline-dev release ("beta",
2026-04-12). For the record: the rewrite carries the same cut-in and
Gigaflare dynamic-resolution policy (`src/perf_scaler.rs`, "Dynamic
resolution" in `docs/RENDERING.md`), and the two Skyline builds are to be
tested with 0.5.1 so users need not change Skyline.

FX Control performance campaign in Eden, 2026-10-15 02:40–05:45Z (operator
request; results in the FX Control repository, `docs/FX_EMULATOR_RESULTS.md`
"Per-move frame drops under a 120 % CPU quota"): Eden's unit under
`CPUQuota=120%` / 5 ms period, Deluxe LessLag with the Performance view open,
27 moves × On/Off/Reduced, one 30 s `CAPTURE_TIMING` each. Deluxe-relevant
outcomes: (1) the post-scene guest abort recurred twice more on the release
NRO at Training exits (runs 5 and 6; five guest aborts in roughly 22 exits
over the night, all the same backtrace); (2) from 04:55Z the Eden profile
ran the teardown-guard candidate `cc7f07f6…` (branch
`post-scene-teardown-guard`, package `artifacts/6e74426068fed7a2/`): six
Training → character-select exits plus one boot, `teardown_guard_skips` 0 →
11 with a code-48 event each time, no abort and 60 fps between captures.
That is the first evidence for the guard; it is Eden-only, six exits, and
not a verified fix.

Cycling test on the candidate, 2026-10-15 05:50–06:20Z (evidence root
`fx-guard-test-1015/`, no CPU quota, same plugin stack): 16 Training →
character-select exits alternating Sephiroth, Pyra/Mythra, Inkling, Lucas,
Ganondorf, Palutena, Kazuya and Ness, with a main-menu round trip after
every fourth cycle. 16 of 16 exits reached the character select at 60 fps;
`teardown_guard_skips` and the code-48 count rose by exactly one across every
exit (11 → 43, the guard also fires on fighter loads); zero stalls or
aborts. Together with the campaign that is 22 exits without an abort on the
candidate against five aborts in about 22 exits on the release build the
same night, all in Eden. The candidate is not merged, not pushed, not on the
console; the operator decides.

### DBI USB MTP installation of f22d (0.3.0) — 2026-10-12 19:58Z

`switch-mtp-f22d/deploy-f22d-mtp-1012.py` preflight 19:55:47Z (installed
rewrite d773e7bf…2b17 as expected; all plugins and protected files backed
up locally) and install 19:58:08Z `installed_verified`: rollback archive
`/ultimate/ssbu_online_deluxe_rewrite/backups/pre-backoff-f22d94950be1b460.nro`
read back as d773; staged copy read back as f22d; target read back
f22d94950be1b4601ad9137a8d87acf6eccf2ec599eb48812e6f386f8e7403d8
(2465792 B, module cac8cc1b…ca31); other plugins, plugin names, config.ini
and protected files unchanged. Operator authorization recorded in
`operator-approval.json`. Hardware behaviour not yet observed: the Switch
test list is in the standalone repo's `docs/ANALYSIS.md` section 6.

### Pinned release v0.2.1 — 2026-10-12 (deferred persistence on PENDING)

Requested by the Training Modpack side: a `REQUEST_PENDING` answer no longer
queues the INI write. The host persists only when it observes `current`
equal to the pending choice (`Registry::reconcile_pending`, run once per
overlay draw even with the page hidden, and in `value`), or when a later
request on that option returns `REQUEST_APPLIED`; stepping a pending change
back to the live value cancels it and writes nothing. APPLIED behaviour is
unchanged. Three regressions cover pending→applied-by-plugin (one write),
pending→cancel (no write) and pending→applied-on-second-request (one write).
286 host tests plus the SDK round-trip; host and Skyline Clippy with warnings
denied; no-default build; rustfmt; Skyline release. Crate 0.2.1; ABI and SDK
crates unchanged at 2.0.0. Tag `v0.2.1` (commit aa8c33f) clean NRO: build
`0.2.1-aa8c33ff469b-1791823438`, SHA-256
`eb87feb1cbcbc2b2c4f4adbe15008d424def377fe2e1d854eaaf880c7ef69f88`,
2453504 bytes, module `8e8e9491d0eddd5891357113cf68ca83b8f5fe0f1a82f6717eedc875f455977f`,
ELF `d3e08612f7a6017c06d49e08bc7b35c0c938946a35f15bb333ce077d518d0b36`.
Not installed on hardware (the Switch has d773). Package: tangent evidence
`artifacts/eb87feb1cbcbc2b2/`. Forgejo release `v0.2.1` carries the clean NRO
and `manifest.json` as assets; release `v0.2.0` carries its NRO, manifest and
the demo plugin NRO `2f9fcdd730b14ab22fc6c19cfc0a6fe9f61d072379b1cb30a695e43b8f004b60`.
Every future tag ships its NRO the same way.

### Pinned release v0.2.0 — 2026-10-12

Tag `v0.2.0` (commit ae19c58, Forgejo `intpa/ssbu-online-deluxe-rewrite`) is
the consumer reference for the plugin API: `deluxe-plugin-abi` 2.0.0 and
`deluxe-plugin-sdk` 2.0.0. It adds pending-aware stepping on the Plugins page
(a second Left/Right takes a pending change back). Clean NRO from that tree:
build `0.2.0-ae19c58d9df6-1791821467`, SHA-256
`7585760b9bc4c63404ea7ff0a605de8afb642ea3db851f1c4ff2429add52ad78`,
2453504 bytes, module `829fa26224030349a7a84bfe48aab7a058e7c96c78cc2bedb855a5311409fb71`,
ELF `b695068c0d08924d6dc964712e13b9c089f10c1cafde8a2463d047ae7d3a9001`.
283 host tests plus the SDK round-trip; host and Skyline Clippy with warnings
denied; no-default build; rustfmt; Skyline release. Not installed on hardware
(the Switch has d773, which differs only by the pending-aware step and version
strings). Package: tangent evidence `artifacts/7585760b9bc4c634/`.

### DBI USB MTP installation of d773 — 2026-10-12 15:46Z

The operator authorized installing d773 on the Switch; the live DBI MTP mount
implies Smash was closed. Preflight confirmed the installed rewrite was 9d08
and backed up the nine plugins and six protected files. The transaction wrote
`ultimate/ssbu_online_deluxe_rewrite/backups/pre-chord-tones-d773e7bfc5764ab7.nro`
(9d08, readback-verified), staged and copy-replaced the NRO and read every
file back: the rewrite is now
`d773e7bfc5764ab795a1d35839adc2a3595b3d6fd45e815b44cd84567b8b2b17`
(2453504 bytes, build 0.1.0-0922d28fd852-dirty-1791818952, module
`df24e776b86a09a2f342b68e17fc26ea2d9238cd1e68b392ba776e123a2bf1e6`); the INI
(`0617a94705ea8d2da3fff782161b5339f6a3880948a807894a01692bccd9c2d3`, the
operator's latest), eight other plugins, plugin names and protected files are
unchanged. Demo plugin not deployed. Evidence:
`~/.local/share/ssbu-mod-backups/plugin-api-tangent-20261012/switch-mtp-d773/`.
This accepts the transfer; the chord fix and tones are not yet observed on hardware.

### DBI USB MTP installation of 9d08 — 2026-10-12 14:00Z

The operator authorized the install with Smash closed and DBI's USB MTP
responder ready. The single-writer transaction (adapted from the 6c39 script)
backed up all nine plugins and six protected files, confirmed the installed
rewrite was still 6c39 immediately before writing, wrote the rollback archive
`ultimate/ssbu_online_deluxe_rewrite/backups/pre-plugin-api-9d08a711b271e0ac.nro`
with readback, staged the candidate with readback, copy-replaced the NRO and
read every file back: the rewrite is
`9d08a711b271e0acbb0b2be426c6892acf32eefe85e17deaccdd5050547ec40c`
(2449408 bytes, module `5baf03f205f8fb39ce81cffa31459681fa21e0c7dc7b062aadd503eb1aad5f3f`);
the INI (`c1272813e40bd694c7429c8103199141e8017341996f754309e54b2dbaa4158d`),
eight other plugins, plugin names and protected files are unchanged. All
device command processes exited. Only the rewrite NRO was written; the demo
plugin was not deployed, so on hardware the chord cycle stays
settings → performance → hidden until a plugin NRO registers.
Evidence: `~/.local/share/ssbu-mod-backups/plugin-api-tangent-20261012/switch-mtp/`
(`preflight.json`, `deployment.json`, `operator-approval.json`, before/after readbacks).
This accepts the transfer, not runtime behaviour on hardware.
