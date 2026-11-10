# Eden rows: Unicode opponent label (0.10.3)

The rows, hooks, segments and references of Eden run `dun-x1` (XPS, 2026-11-10), as the runner
(`tools/eden-runner/runner.py` of the playbook `87b3de0`) played them; the record is in
`docs/VALIDATION.md` ("0.10.3 — Unicode opponent names — Eden checks").

- One Quickplay pair, LAN Online with TLS off, 1x render, volume 0. Instance a runs 0.10.2 (`effbaa0`), instance b
  0.10.3; both have ssbu-desync-detector and the tile cursor. Both profiles are named `Æsir テスト`
  (`make/nickname.py` rewrites `profiles.dat`), so each Opp label shows that name.
- `steps.py` is quickplay-cancel-fix's navigation, copied unchanged (sha256 in `provenance.json`).
  `deluxe_steps.py` holds this run's hooks: `qcf` (that navigation with a half-size image compare, because its
  references were taken at 0.5x), `open_window`, `hide_window`, `to_solo`, `arm_label_trace` and `label_glyphs`
  (a guest trace on `ImFontBaked::FindGlyph`: for each of Æ テ ス ト, did the draw return the fallback glyph),
  and `font_stats` (the `imgui_font_*` lines of `renderer-status.txt`).
- `segments/open-deluxe` (L+R+Down in one frame) and `segments/dpad-up` are authored; the other segments come from
  quickplay-cancel-fix. `make/` holds the generators (`make_rows.py`, then `make_rows_json.py`; `make_config.py`
  writes the base config for `eden_test.py new --from`).
- The reference screenshots (`refs/`) are game screens and stay out of this repository (AGENTS.md: no Nintendo
  assets). `make/make_rows.py` copies them from quickplay-cancel-fix's rows and crops the `*-roi.png` ones;
  `provenance.json` and `rows.json` pin each by sha256.
- `rows.json` is the final one. Row `m1-label` (the label checks, the GIFs and its match) ran once in `dun-x1` and is
  left out of it; `make/make_rows_json.py` keeps its definition and says why.
