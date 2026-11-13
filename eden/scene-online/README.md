# Eden: Deluxe's scene through a LAN Quickplay session

2026-11-13. The rows, hooks and generators of the deluxe-scene-online runs (`scn-*`), recorded in
`docs/VALIDATION.md`. The runner is the playbook's `tools/eden-runner/runner.py` at `13b2e83`; run outputs are in the
operator's evidence root (`artifacts/deluxe-scene-online/`), not here.

- A LAN Quickplay pair split across hosts (no Surface slot): instance a on the XPS, b on the Yoga, each run by its own
  `eden_test.py all`; `run_pair.ps1` starts the XPS first and the Yoga 75 s later (the imgui-parity route).
  - `prepost`: a = Deluxe `9128cd2` (before `1173bea`), b = `f33cfef` (local `main` HEAD with `1173bea`); the evidence
    row records.
  - `fix`: both instances on the fix; the evidence row needs every battle to read quickplay with the online bit on.
- `make/make_rows_scene.py` builds the rows folders from the imgui-parity split rows (quickplay-cancel-fix's
  `steps.py` through `deluxe_steps.qcf`, `split_steps.search_until_session`, segments and refs) and the
  qp-offline-training rows (`gdb_rc11.py` with no holding helper, `gdbdump.py`, `qp_steps.py`'s readers, the
  `attract-cancel` segment). The reference shots are game screens and stay out of this repository.
- `make/make_config_scene.py` runs on the executor host and starts from that host's imgui-parity base: profile g, 1x,
  the Æsir `profiles.dat`, Deluxe's `config.ini` (Auto on, `platform=emulator`), the Yoga's Qt fixture; the Eden
  `2f315fc4` gdbstub build, the GDB stub on before `game.load`, no guest trace.
- `scene_table.py` (host tests: `python3 -B -I eden/scene-online/test_scene_table.py`) reads `hang-trace.log`,
  `events.log` and the stage logs and gives the scene per phase: the waiting room, each online battle (at its load,
  its heartbeats, events 107 and 30 with seconds after the load) and the results. `phase_rows.py` prints one table
  row per instance; `scene_steps.py` is the row hook that runs it on the run's own files.
