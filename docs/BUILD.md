# Build and release

Use the pinned Rust toolchain in `rust-toolchain.toml`, installed cargo-skyline
and the Skyline target. Native ImGui preparation and provenance are documented
in [IMGUI_DEPENDENCY.md](IMGUI_DEPENDENCY.md).

Run each host check separately from the repository root:

```powershell
cargo clippy --all-targets -- -D warnings
cargo build --no-default-features
cargo test
```

The installed cargo-skyline 3.5.0 check wrapper needs two separators to forward
warning flags:

```powershell
cargo skyline clippy -- -- -D warnings
```

With Docker and Git available, build the native dependency from source, then
build and package the Switch plugin:

```powershell
/usr/bin/python3 tools/prepare_imgui.py
/usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -Command '& ./tools/Build.ps1 -Package'
```

Outputs:

```text
target/aarch64-skyline-switch/release/libssbu_online_deluxe_rewrite.nro
dist/<NRO-hash-prefix>/
```

The ignored bundle contains the NRO, optimized ELF, source archive, dependency
provenance and hashes. Preserve each tested bundle. No Nintendo assets or
third-party compiled plugins are committed. Build checks alone do not establish
runtime compatibility or reduced latency.

`Build.ps1 -Coverage` builds the instrumented coverage variant into
`target/coverage` (feature `coverage`, `tools/coverage/config.toml`); with
`-Package -Output <dir>` the bundle's manifest records `"variant": "coverage"`.
It is a measurement build for emulator and console sessions, never a release
asset. How it works and how to read its profiles:
[Coverage build](DEBUGGING.md#coverage-build-052).

## Release

A release is a tag `vX.Y.Z` on the commit that bumps `Cargo.toml`, built from
that commit so the build id (`X.Y.Z-<commit>-<epoch>`) names it, plus a Forgejo
release carrying `libssbu_online_deluxe_rewrite.nro` and `manifest.json`. The
manifest records the NRO and ELF SHA-256, module id, compiler and native
dependency provenance. Every console install is recorded in
[validation](VALIDATION.md) with the replaced NRO's hash and the rollback path.
