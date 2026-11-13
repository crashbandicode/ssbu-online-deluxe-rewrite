# Eden rows: offline Training reads offline (`1173bea`)

The rows, hooks and generators of Eden run `qpt-y1` (the Yoga, 2026-11-13), recorded in `docs/VALIDATION.md`
("Offline Training is offline"). The runner is the playbook's `tools/eden-runner/runner.py` (sha256 in
`provenance.json`).

- Deluxe `1173bea` (NRO `2278c03d…`), the detector and the tile cursor; the Eden `2f315fc4` gdbstub build with the GDB
  stub on before `game.load`; profile g, 1x render, volume 0; Deluxe's `config.ini` as in the dun runs (Auto on,
  `platform=emulator`, Stealth on).
- `rows-single.json` (`make/make_rows_qpt.py single <out>`, as `qpt-y1` ended): one instance, a chain from a fresh
  boot. Games & More → Training; Plus, Quit, B held back to the main menu; Online → Quickplay → READY → the waiting
  room while the search runs; Leave Battle, back to the main menu, Games & More → Training again.
- `rows-pair.json` (`make/make_rows_qpt.py pair <out>`): the same rows over a LAN Quickplay pair (a: Training; b: the
  waiting room, then Training after it), for a non-Surface host with both slots free. Not run.
- `qp_steps.py` holds the hooks. With the stub on, Eden refuses guest tracing, so every check reads Deluxe's own
  files: the scene changes in `hang-trace.log`, events 30/73/107/110/116 in `events.log` (decoded as
  `docs/DEBUGGING.md` lists them) and the header of each new stage log in `matches/`. Each row keeps frames and a copy
  of those files.
- `_back_to_main` finds the main menu whichever panel has the focus: the main, Online and Quickplay menus and Games &
  More share an icon column, and the menu headers tell them apart. It focuses Smash with Left, Up, the state the menu
  macros start from, and holds B to leave a character select.
- Fixed in the boot (`run --retry`): the library entry's start gate `ready` (the segments are local copies now, without
  the library key); B taps, which only take the token back on a character select; the library's `gm.png` header,
  which didn't match this build's frame (`make/make_header_ref.py` cut `refs/gm-header.png` from a `qpt-y1` frame);
  the Online header's 17–19 difference against the dun runs' reference (menu tolerance 30).
- Reused, pinned by sha256 in `provenance.json`, not copied here: `gdb_rc11.py` and `gdbdump.py` (the integration
  lane's GDB-stub hooks, no holding helper: `gdb-nohold.txt`), the segments `to-online`, `to-quickplay`, `to-css`,
  `tile-up` and `pick` (`eden/unicode-label/segments/`), `quit-focus` and `quit-confirm` (the integration lane), and
  the shared library's `attract-cancel`, `main-stage`, `stage-css` and `enter-training`. The reference shots are game
  screens and stay out of this repository (AGENTS.md); `make/make_sidebar_ref.py` and `make/make_header_ref.py` cut
  the two new ones from frames.
- `make/make_config_qpt.py` writes the base config passed to `eden_test.py new --from` on the executor host (the Yoga
  or the XPS; the Deluxe lane's one Surface slot was the parity worker's).
