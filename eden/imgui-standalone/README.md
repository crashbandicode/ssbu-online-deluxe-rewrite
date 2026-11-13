# Eden rows: the imgui-smash 1.1.0 standalone and the 0.10.5 re-pin (`ims-x1`)

The rows, hooks and new segments of Eden run `ims-x1` (2026-11-13), recorded in `docs/VALIDATION.md`. The runner is
`tools/eden-runner/runner.py` of the playbook (`provenance.json` pins its sha256).

| Instance | Plugins |
| --- | --- |
| a | Deluxe 0.10.4 release NRO (`70a2931e`) and the standalone `libimgui_smash.nro` 1.1.0 (`4d8ea566`): Deluxe draws in the standalone's ImGui |
| b | Deluxe 0.10.5 `9c994bf` (`aee7f411`), which links the same archive (`c1e4471b`), no standalone: Deluxe's built-in ImGui |

- Both run the detector and the tile cursor. It is a LAN Online Quickplay pair with TLS off, 1x render and volume 0,
  both profiles named `Æsir テスト`, as in `dun-x6` (`eden/unicode-label/`).
- `rows.json` is `make/make_rows_ims.py` applied to `dun-x6`'s rows. The run's rows folder also held `dun-x6`'s
  `steps.py`, `deluxe_steps.py`, segments and refs; `provenance.json` gives every file's sha256. The refs are game
  screens and stay out of this repository.
- `ims_steps.py` holds the new hooks:
  - `host_status`: which ImGui Deluxe uses, from `renderer-status.txt`, the code 16 rows and `install.log`.
  - `crop_strip`: the Opp line cropped from a frame and scaled 3x.
  - `menu_cycle`: L+R+Down until the window hides, then once more; one code 23 row per chord.
  - `settings_edit`: Down, ZR+Right and ZR+Left on NetLatency (code 8, b = 0, then Auto), Up.
  - `nav_chord`: L3 + R3, then D-pad Down and Up, with trace points on `InputHelper::updatePadState` and
    `ImguiNvnBackend::updateGamepad` in the module that owns the active ImGui.
- `segments/zr-right`, `zr-left`, `dpad-down` and `l3-r3` are authored (`make/make_segments_ims.py`).
- `config-base-surface.json` (`make/make_config_ims.py`) is the base passed to `eden_test.py new --from`.
- `make/heartbeat_scenes.py` lists `hang-trace.log`'s heartbeats per scene (first and last `mem_used`, as logged);
  `make/provenance_ims.py` wrote `provenance.json`.
- `n1-menu-b` failed once on the hook's own frame check, which compared the top-left box while the window sat top
  right on the character select. The hook now takes the larger of both boxes, and `run --retry` passed it in the
  same boot.
