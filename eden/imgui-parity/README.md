# Eden: drawing through the standalone imgui-smash 1.2.0 next to the built-in host

2026-11-13. Runner inputs for the imgui-parity runs: Online Deluxe 2085fd04 (6c05693) drawing through
`libimgui_smash.nro` 1.2.0, against the same Deluxe drawing with its built-in ImGui. Eden 2f315fc4 (gdbstub build),
playbook 13b2e83. Paths are this machine's; run outputs are in the operator's evidence root
(`artifacts/imgui-parity/`), not here.

## pair/ — one online match, standalone host against built-in host

- `make_split.py` turns the ims-x1 two-instance rows (`eden/imgui-standalone`, re-made with expect=external) into
  one row list per instance, `rows-a.json` and `rows-b.json`, each run by its own `eden_test.py all` on its own
  host: instance a with the standalone (`config-base-a-xps.json`, XPS, display :158), instance b without it
  (`config-base-b-yoga.json`, Yoga, display :157). `split_steps.py` holds the per-instance steps; the rows also run
  `deluxe_steps.py` and `ims_steps.py` as copied here, and `segments/`.
- `wait_pair.ps1` polls a read-only `lifecycle.py preflight` on both hosts, then `run_pair_xy.ps1` starts the XPS
  first and the Yoga 75 s later: the first LAN broker leads the coordinator, and inbound connections to the Yoga
  timed out before (eden-pair e01).
- The Yoga's Eden profile chose its VPN interface for the LAN broker (`[auto] broker started ... lan=10.125.210.2`).
  Its qt-config fixture sets `network_interface=enxf8e43bdc2cb5` with `network_interface\default=false`
  (`fix_qt.py`); with `\default=true` Eden ignores the value and writes it back empty. The fixture itself is not
  committed: it carries the profile's web token.
- Evidence: `collect_pair.ps1` copies receipts, row screenshots and Deluxe/detector files; `compare_pair.py` and
  `pair_detail.py` print them side by side; `tools/heartbeat_memory.py --scenes a=<log> b=<log>` gives memory per
  scene.

## training/ — memory in Training with the card's plugins

- Surface (lane 1, display :141): `queue.ps1` runs `run_variant.ps1` per variant over the integration lane's
  `make_hang.py` rows with two local fixes (`patch_hang.py`: hand down before B after the CPU drop; `patch_kick.py`:
  gdb_kick waits up to 300 s). `compare.ps1` runs `tools/heartbeat_memory.py` over the runs' hang-trace logs.
- Yoga (`yoga/`): `make_ytrain.py` derives the fa/fc configs from hang-card-r5 (the card set on the Yoga, whose
  Training route passed there); `run_ytrain.ps1` runs them one at a time with a gap after each release.
- Variants: fa = Deluxe 2085fd04, no standalone; fc = Deluxe 2085fd04 + standalone 1.2.0.
