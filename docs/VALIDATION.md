# Validation record

Development milestone, 2026-10-05:

- Optimized AArch64 Skyline NRO built.
- Host cargo clippy --all-targets -- -D warnings passed with zero warnings.
- Host cargo build --no-default-features passed.
- Host cargo test: 18 passed after review fixes, including pointer-walk regression coverage.
- Debug ELF has .debug_info, .debug_line and .symtab.
- Packager verified matching executable bytes.
- Known buffer/CSS hook addresses resolved to Rust source file/line.

Switch-target Clippy is unavailable in installed skyline-v3:
error: 'cargo-clippy' is not installed for the custom toolchain 'skyline-v3'.
Host Clippy excludes Switch-only code; the actual Skyline compiler checks those bodies.
No warning was emitted by that build.

No NRO was installed and no game process was run.

## First live matrix

Start with this mod alone, then Training Modpack plus SGPO; test both load orders.
Keep the exact build bundle.

1. Observer-only diagnostic flags: startup identity and log persistence.
2. On + Auto: baseline visuals and CSS/match/results/Training transitions.
3. Menu: each controller family, both rows, saving and restart.
4. Input buffer: Auto and explicit values in Quickplay/arena/local.
5. Each target move: Off suppression and newly spawned On restoration.
6. Shield/parry, smoke, KO, colors, projectiles and LAST_EFFECT consumers.
7. Training save-state/reset/menu and SGPO HUD in the same session.
8. Multi-fighter matches and repeated loads, separately console/emulator.
9. Compare frame times before claiming performance gains.

Diagnose failures from logs and addresses; do not add broad deletion or catch_unwind as a substitute.
