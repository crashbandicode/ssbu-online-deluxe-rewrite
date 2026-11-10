# Eden rows: Unicode opponent label (0.10.3)

The rows, hooks, segments and generators of the Eden runs recorded in `docs/VALIDATION.md`. The runner is
`tools/eden-runner/runner.py` of the playbook `87b3de0`.

| Run | Rows | Pair |
| --- | --- | --- |
| `dun-x1` | `rows.json` | a 0.10.2, b the first 0.10.3 (`6732f29`, glyph budget; never released) |
| `dun-x2` | `dun2/rows.json` | a 0.10.2, b 0.10.3 (`bc22c80`, static and dynamic glyph sets) |
| `dun-x3`, `dun-x5` | `dun3/rows.json` | a 0.10.3, b the 0.10.3 diagnostic build with the label feed (`diag-label-feed.txt` as `diag.txt`; 16 s per name in `dun-x3`, 20 s in `dun-x5`) |
| `dun-x6` | `dun4/rows.json` | a 0.10.3 (`bc22c80`), b 0.10.4 (`21c0009`, paged codepoint index) |
| `dun-x7`, `dun-x8` | `dun5/rows.json` | a 0.10.4, b the 0.10.4 diagnostic build with the label feed (20 s per name); the index-bytes trace and the evidence GIF `label-ja-ko` |

- Every pair is LAN Online Quickplay with TLS off, 1x render and volume 0. Both profiles are named `Æsir テスト`
  (`make/nickname.py` rewrites `profiles.dat`), and the detector and the tile cursor run on both.
- `steps.py` is quickplay-cancel-fix's navigation, copied unchanged (sha256 in `provenance.json`).
- `deluxe_steps.py` holds this project's hooks:
  - `qcf`: that navigation, with a half-size image compare, because its references were taken at 0.5x.
  - `open_window`, `to_solo`.
  - `arm_label_trace`, `label_glyphs`: a guest trace on `ImFontBaked::FindGlyph`, giving the glyph each character
    drew.
  - `font_stats`, `font_sample`: the `imgui_font_*` lines and `imgui_font_dynamic_set` of `renderer-status.txt`.
  - `hold_active`: holds with input on both instances, because Quickplay drops a player who is idle for about 30 s.
  - `follow_feed`: for each name the label feed shows (event 121), it checks the dynamic set, the glyphs drawn and
    the rebuild count. Since 0.10.4 it also checks the index mode, records the index-bytes trace (every
    `renderer-status.txt` rewrite), and can stop after a phase (`until_phase`) and resume, so that `feed_wait` and
    `feed_switch` inside a gif step can film one name change.
  - State that spans steps lives on the runner object, because the runner loads this file afresh for every step
    (`dun-x7` failed on that). `make/sim_feed.py` runs the split feed steps against a simulated diagnostic build.
- The file is the latest version; `dun-x1` and `dun-x2` ran earlier versions of the same hooks, kept in their run
  folders.
- `segments/open-deluxe` (L+R+Down in one frame) and `segments/dpad-up` are authored; the other segments come from
  quickplay-cancel-fix.
- `make/` holds the generators:
  - `make_rows.py` and `make_rows_json.py` (`dun-x1`);
  - `make_rows2.py` and `make_config2.py` (`dun-x2` to `dun-x5`);
  - `make_rows3.py` and `make_config3.py` (`dun-x6` to `dun-x8`), `feed_table.py` and `index_trace.py` (the per-name
    table and the index-bytes trace from `feed-b.json`);
  - `config-base-*.json` are the base configs passed to `eden_test.py new --from`.
- The reference screenshots (`refs/`) are game screens and stay out of this repository (AGENTS.md: no Nintendo
  assets). `make/make_rows.py` copies them from quickplay-cancel-fix's rows and crops the `*-roi.png` ones;
  `provenance.json` and each `rows.json` pin them by sha256.
- `rows.json` (`dun-x1`) leaves out row `m1-label`. It ran once in `dun-x1`, and `make/make_rows_json.py` keeps its
  definition and says why.
