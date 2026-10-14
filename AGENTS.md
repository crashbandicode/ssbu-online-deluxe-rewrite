# Project instructions

Use Rust for the in-game Skyline plugin. Keep the product one NRO.
Native Ubuntu commands use /usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -Command.
Global routing authority: /home/intpa/.codex/DELEGATION_ROUTING.md.
Do not push without the operator's explicit permission.
Do not add a handoff file to this repository; the restart-safe handoff lives in the
operator's evidence root outside it. Preserve exact fork provenance and a behavior
parity ledger.
Never claim an inferred crash fix or untested game compatibility as verified.
No global Layout::Draw hook. Do not patch the overlay's 0x374812c neighborhood.
FX lives in ssbu-fx-control; Deluxe carries no effect hooks. Do not add
character-effect, gameplay-article, hitbox, camera-request, or RNG/simulation
changes here.
Before completing Cargo work: cargo clippy --all-targets -- -D warnings,
cargo build --no-default-features, meaningful host tests, actual Skyline build.
Keep optimized debug symbols and SHA-256 manifests outside the repository.
Do not commit Nintendo assets, third-party compiled NROs, credentials, or build caches.
