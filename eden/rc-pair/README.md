# Eden: the release-candidate LAN Quickplay pair

2026-11-13. Deluxe through the imgui-smash 1.2.0 standalone with the card's plugin set, one instance per host,
recorded in `docs/VALIDATION.md`. Run outputs are in the operator's evidence root, not here.

- `make_rows_rc.py` builds `rows-rc-a` and `rows-rc-b` from the scene-online fix rows (`eden/scene-online`) and the
  imgui-parity split rows (`eden/imgui-parity/pair`): the scene evidence with `expect=online`, `battle_self` (the
  attack rule) to `sync_end`, the external host, `index=paged`, the "Æsir テスト" glyphs in the font lines, the Opp
  line crop, and `rc_steps.py` (`hook_lock_skipped=`, the plugin count, L3 + R3 frames).
- `make_config_rc.py` runs on each executor host and starts from that host's scene-online base; the plugins are
  every NRO in one folder, pinned by sha256.
- `run_rc_pair.ps1` starts host A, then host B 75 s later, each `eden_test.py up`, `run`, then `down`.
- The GDB stub is on before `game.load`, so Eden's trace points (they need it off) are not armed: the imgui-parity
  rows' FindGlyph and pad-poll traces are not part of these rows.
