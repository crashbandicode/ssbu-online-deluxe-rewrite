# Source reference snapshots

These files are unmodified research inputs, not workspace members or shipped code.
The Quickplay snapshot and exact upstream-to-fork diff preserve the fork's changes
while the replacement is reconstructed in src/. Do not interpret their presence
as implemented feature parity. Their original authorship/comments are retained.

Quickplay: MSFS-RyZe/ssbu-online-deluxe-quickplay-13.0.5, commit
35fdf7cf5fea0f8249ca9fbb848c241b590bb483; upstream baseline
c54c11684004e2d68fc1c519d046bb6793458b4d (saad-script).
Archive: user-supplied ssbu-sync 0.1.2, authors blu-dev and BlankMauser.
Only source/manifests/readme/license files were copied. The archive's nsuite
subproject and build-specific config are not vendored; this is not a build tree.
Per-file SHA-256 values are in provenance.json.

FX selection uses named effects and observations from the user-supplied
2026-05-14 peer sources; the peer plugins and their compiled binaries are not bundled.
C ABI declarations in src/runtime/effects.rs match skyline-smash bed642e
src/cpp.rs. No Nintendo game assets, decompiled game code, or closed host NROs
are included. Do not add a new license to original third-party code by inference.
