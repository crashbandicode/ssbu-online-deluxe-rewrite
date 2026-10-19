# Native display and global window

The rewrite retains the small global Player Info window and the fork's native
character-select display together. Change NetLatency or NetProfile in the
window, close it, and the selected values remain at the top of character select.
The selected opponent's available name, settings and RTT appear on the second
line. RTT supplies the same stability color as the window; connection type
remains available in the window's You and Opp lines (the Interface row before
0.6.0).

Streamer restores the original title, font, colors and background. Turning it
off restores the status display. Font sizes are absolute, not multiplied by the
current size, so repeated edits do not enlarge the text. Since 2026-10-17 the
restore is a one-shot: it fires while the pane still shows this mod's text (or
right after the game refreshed the words under this mod's style) and no longer
re-fires when only the live font drifted from the saved one.

![Training character select with the native status line](media/training-css.png)

The native status line on the Training character select, Eden 2026-10-14 with
0.4.2: `You: LessLag | Auto` is the selected profile and input buffer. The
opponent line appears only with a connected station (since 0.5.5, one that has
produced an RTT sample within 10 s). The window itself and
Streamer on are shown in [Screenshots](SCREENSHOTS.md).

## Reused fork code paths

Source reference: `MSFS-RyZe/ssbu-online-deluxe-quickplay-13.0.5`, commit
`35fdf7cf5fea0f8249ca9fbb848c241b590bb483`, preserved under:

```text
reference/quickplay/src/net/mod.rs
reference/quickplay/src/ui/native/mod.rs
reference/quickplay/src/ui/native/utils.rs
```

The integration uses the fork's CSS hook (`0x1a12f60`), two-load pane lookup,
parent/previous-pane walk, text setter (`0x37a28a0`), title dimensions, spacing,
alignment and material colors. The arena hooks are `0x1887b1c` and `0x18881f0`;
the room-ID pane is `set_txt_roomid`. Arena text retains its original font and
uses the fork's vertical-position formula for the displayed lines.

Necessary integration changes are version/instruction checks, bounded text
decoding, mapped-pointer and known-pane checks, scene lifetime tracking, and
per-pane capture of original styles. Valid virtual tables can reside in main's
data segment. A standard existing Skyline text-setter hook is called through
after checking its untouched instruction tail; its owner is not assumed.
There is no global `Layout::Draw` hook or overlay HUD-offset patch.

The display reads the rewrite's existing settings and peer snapshot. It does
not change simulation, networking, rendering policy or input handling.

## Differences and limits

- The window remains the settings editor. The fork's native D-pad edits are
  ported (`overlay.rs` `poll_with`, `menu.rs` `apply_native_edges`) and have
  required ZL held since 0.4.1, because the D-pad also moves the game's own
  cursor.
- The local-wireless lobby pane (`set_text_dev_count`, captured at the scene
  entry `0x1bd45e0` and written at the game's count update `0x1bd7a80`) shows
  the input buffer behind the same marker as the titles. Since 2026-10-17 it
  goes through the same per-pane capture as the CSS and arena panes: Streamer
  restores the game's own captured count text once and leaves the pane to the
  game, whose count refresh lands after this mod's write in the call that
  refreshes it and is captured again as the pane's game text. The fork's
  `nn::ldn::Initialize` / `Scan` hooks and its bare scan-count number are not
  reproduced. The 13.0.5 image shows the hook sits directly after the dirty
  query at the site entry (`blr x8; tbz w0, #0` on the `+0x180` sub-object),
  and the latency text observed on Eden lobbies shows the game's own write is
  not unconditional per call.
- CSS status is available offline too. Offline screens omit the opponent line.
- Arena lists the known mapped remote stations (up to eight lines, local and
  unmapped stations omitted; `runtime/native_banner.rs` `arena_remotes`), each
  with name, profile, input buffer, RTT and the announced interface suffix when
  known. Its room-ID path, style and positioning are implemented but require an
  actual NSO arena test on Switch.
- One text builder (`native_banner::status_text`) formats the CSS line and the
  arena roster, so the CSS fields take the arena byte bounds (name ≤ 40 bytes,
  profile ≤ 9, latency ≤ 4, ping ≤ 17); one style decision
  (`native_banner::paint`) chooses between the Streamer restore and the
  telemetry write for every pane kind (2026-10-17).
- Original styles are captured and restored instead of assuming the fork's
  hard-coded default title styles. The CSS original function runs before the
  display update; this ordering is covered by the emulator tests below.
- Unknown panes or incompatible hook entries are skipped and logged. This is
  specific to Smash 13.0.5.

## Validation

The offline test used candidate `84d48448ac91e65d`, isolated Eden run
`20261007-101737-33eb02`, with Training Modpack, corrected gamepad overlay,
ARCropolis, Elite Legal Stages and standalone ImGui loaded.

Observed screenshots and persisted INI confirm:

1. The small window and native title appear together.
2. Selecting LessLag and buffer 0 updates the title to `You: LessLag | 0`.
3. The title remains after cycling the window to hidden.
4. Streamer restores `Solo Battle` and its original yellow background; disabling
   Streamer restores the status at the same size.
5. Leaving/re-entering CSS and completing Olimar versus Fox through results and
   back to CSS retain a correctly sized title.

The instance exited normally, peaked at 6,148,984,832 bytes with no OOM or limit
events, and the staging journal restored original file hashes. Evidence stays
outside Git under the corresponding `dist` bundle and local backup journal.

Candidate `4a927dc20904722d` adds the arena-specific font/position correction.
It passes 192 host tests, host and Skyline Clippy with warnings denied,
`cargo build --no-default-features`, and the optimized Skyline build. Its linked
runtime result is recorded in [VALIDATION.md](VALIDATION.md).

Final-build observations:

- Solo run `20261007-105849-510a19` showed Player Info, Elite's window and the
  native title together at approximately 60 FPS. It exited normally, with
  6,733,324,288 bytes peak memory and no OOM.
- Solo run `20261007-110542-4550ab` added the G&W NRO to the same stack and also
  showed both windows and the title. Elite remained Off in these screenshots;
  a D-pad edit changed its pool, not its enable switch. The run reached its
  configured 600-second deadline, with 7,787,769,856 bytes peak memory and no
  OOM. The subsequent stop request found the socket already closed. This was
  a watchdog stop, not a normal exit or observed match crash.
- Linked runs `20261007-103320-77c12b` and `20261007-103321-17d19e` displayed
  names, LessLag/LLUltra selections and green RTT of roughly 19–21 ms in both
  native headers. The embedded-host peer also showed its window. The
  standalone-host peer's Deluxe and Elite windows were invisible, despite
  registered callbacks and working edits. No match was started.
- That paired run was below full speed before CSS and did not OOM. One
  frontend required an explicit owned-service stop after a game-stop modal
  timeout. Neither observation establishes a mod root cause. Solo success
  with and without G&W rules out G&W's mere presence as a sufficient
  explanation; the concurrent-window anomaly remains open.

All four journals were restored with original file hashes checked. No normal
Eden profile or Switch files were changed. The final bundle retains raw checks,
logs, screenshots, exact plugin hashes and a separate scoped acceptance receipt;
the original build manifest is unchanged.

For a future controlled comparison, the existing diagnostic marker
`sd:/ultimate/ssbu_online_deluxe_rewrite/DISABLE_NATIVE` skips these native hooks
on restart while retaining the global window. Compare otherwise identical
paired sessions with and without the marker before attributing the missing
external windows to this change. Remove it and restart to restore the native
display.

No new latency measurements are claimed for these UI changes. The measurements
in [LATENCY_METHODOLOGY.md](LATENCY_METHODOLOGY.md) remain tied to their original
builds. No Switch or NSO test was performed during this change.
