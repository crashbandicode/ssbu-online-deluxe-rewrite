# Build and release

Host checks use the Rust toolchain pinned in `rust-toolchain.toml`. The Switch
build uses cargo-skyline and its `skyline-v3` toolchain, which the package
manifest records as `compiler`. Native ImGui preparation and provenance are
documented in [IMGUI_DEPENDENCY.md](IMGUI_DEPENDENCY.md).

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
It is a measurement build, so far run only in Eden, and never a release asset. How it works and how to read its profiles:
[Coverage build](DEBUGGING.md#coverage-build-052).

## Release

A release is a tag `vX.Y.Z` on the commit that bumps `Cargo.toml`, built from
that commit so the build id (`X.Y.Z-<commit>-<epoch>`) names it, plus a Forgejo
release carrying `libssbu_online_deluxe_rewrite.nro`, `manifest.json` and,
since 0.10.0, `SHA256SUMS.txt`. The build id's epoch comes from
`SOURCE_DATE_EPOCH` when it is set (`build.rs`); the v0.10.4 rebuild from its
tag with that epoch gave the same NRO bytes. The manifest records the NRO and
ELF SHA-256, module id, compiler, native dependency provenance and
`runtime_tested`: `tools/package_debug.py` writes `false`, and v0.10.4's
manifest is the first with `true` and a `runtime_evidence` record of its
console run. Record each console install in [validation](VALIDATION.md)
with the replaced NRO's hash and the rollback path.
