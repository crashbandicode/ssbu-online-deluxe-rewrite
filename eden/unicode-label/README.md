# Eden rows: Unicode opponent label (0.10.3)

The rows, hooks, segments and generators of the Eden runs recorded in `docs/VALIDATION.md`. The runner is
`tools/eden-runner/runner.py` of the playbook `87b3de0`.

| Run | Rows | Pair |
| --- | --- | --- |
| `dun-x1` | `rows.json` | a 0.10.2, b the first 0.10.3 (`6732f29`, glyph budget; never released) |
| `dun-x2` | `dun2/rows.json` | a 0.10.2, b 0.10.3 (`bc22c80`, static and dynamic glyph sets) |
| `dun-x3`, `dun-x4` | `dun3/rows.json` | a 0.10.3, b the 0.10.3 diagnostic build with the label feed (`diag-label-feed.txt` as `diag.txt`) |

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
    the rebuild count.
- The file is the latest version; `dun-x1` and `dun-x2` ran earlier versions of the same hooks, kept in their run
  folders.
- `segments/open-deluxe` (L+R+Down in one frame) and `segments/dpad-up` are authored; the other segments come from
  quickplay-cancel-fix.
- `make/` holds the generators:
  - `make_rows.py` and `make_rows_json.py` (`dun-x1`);
  - `make_rows2.py` and `make_config2.py` (`dun-x2` to `dun-x4`);
  - `config-base-*.json` are the base configs passed to `eden_test.py new --from`.
- The reference screenshots (`refs/`) are game screens and stay out of this repository (AGENTS.md: no Nintendo
  assets). `make/make_rows.py` copies them from quickplay-cancel-fix's rows and crops the `*-roi.png` ones;
  `provenance.json` and each `rows.json` pin them by sha256.
- `rows.json` (`dun-x1`) leaves out row `m1-label`. It ran once in `dun-x1`, and `make/make_rows_json.py` keeps its
  definition and says why.
