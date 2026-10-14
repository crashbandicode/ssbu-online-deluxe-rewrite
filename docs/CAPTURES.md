# Timing captures

Every capture writes a pair of CSVs to `sd:/ultimate/ssbu_online_deluxe_rewrite/`:
`timing-<tick>.csv` (one row per render callback: tick, buttons, FX state,
overlay view) and `api-<tick>.csv` (CPU-side spans around the render API
calls and the plugin's own markers). Both stream from the diagnostics thread
while the capture runs; no game hook does file I/O. `timing-status.txt` reads
`capturing …` and then `saved …`. Analyze with `tools/analyze_timing.py` and
`tools/analyze_render_trace.py`; the column and footer formats are in
[Performance](PERFORMANCE.md) and the event kinds in [Debugging](DEBUGGING.md).

## Manual capture

Hold L+R and tap Up in the Debug view, or write a label (letters, digits, `_`,
`-`, up to 48 characters) to `sd:/ultimate/ssbu_online_deluxe_rewrite/CAPTURE_TIMING`.
The capture lasts 30 s. A request during a running capture or export is ignored.

## Character-select capture (opt-in)

To locate the remaining hardware character-select cost, create an empty file
`sd:/ultimate/ssbu_online_deluxe_rewrite/CAPTURE_CSS` before boot. The build
then installs three extra read-only hooks (dispatcher job registration,
dependency wait, the UI layout job) and arms a 30 s timing capture
automatically each time the character select is entered: leave the hand still
for the first ten seconds, then move it continuously. The `api-<tick>.csv`
gains per-frame backoff sleep totals (kinds 60/61), one read-only priority and
core-mask row per thread (62/63), the UI job's dispatch (64), every dependency
wait with its counter and thread (66) and the UI job span (77). Nothing is
changed: no priority, affinity, clock or task order. Without the flag none of
these hooks exist. The Debug view's `CSS capture` row shows the state, and
`tools/analyze_render_trace.py` prints a `css_capture` section.

## Match capture (opt-in)

To see what a whole match does, create an empty file
`sd:/ultimate/ssbu_online_deluxe_rewrite/CAPTURE_MATCH` before boot. The same
capture pair is then armed when a stage loads (Singles, Doubles or Training;
label `match_singles`, `match_doubles` or `match_training`) and ends when the
game leaves the stage (results screen or character select; `# reason=scene_end`),
or after ten minutes. Both files stream to the card while the match runs, about
4 MB per minute. The flag also installs the three read-only `CAPTURE_CSS` hooks
so dependency waits and the UI job appear in the trace; with both flags on the
card the character-select capture runs as well. The Debug view's
`Match capture` row counts the captures armed and ended; event log code 106
records each step.

Long match captures can lose a small share of API rows at the stage-load burst
(reported in the CSV footer as `skipped`); frame statistics are unaffected.
