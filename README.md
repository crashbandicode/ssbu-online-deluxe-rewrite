# Online Deluxe rewrite

Private Rust/Skyline reconstruction for Smash Ultimate **13.0.5**, delivered as **one NRO**.
This is an initial development build, not a completed LessLag replacement.

Implemented:
- Native character-select menu: character FX **On / Off**, default **On**. Reduced is unavailable.
- Online input buffer **Auto / 0–25**, retaining the fork's Quickplay null-pointer guard.
- Bounded events, persistent install checkpoints, panic diagnostics, build identity and matching debug symbols.
- Targeted effects for Byleth, Ike, Roy, Chrom, Falcon and Sephiroth: [exact scope](docs/FX.md).

**Rendering remains Vanilla.** LessLag/Ultra/Doubles reconstruction, automatic profile application,
arena/local-room UI, overclocking and performance scaling are pending.
The fork's original source and exact changes are preserved under [reference](reference/README.md).
See the [parity ledger](docs/STATUS.md).

No ImGui, external SsbuSync backend, PIA manager, smashline, ARCropolis assets or extra FX NRO
is required by this build. Skyline itself is required. Training Modpack and SGPO remain their own mods.

## Build

Host Rust is pinned in rust-toolchain.toml. Switch builds use installed cargo-skyline 3.5.0 /
skyline-v3; each bundle records the actual compiler. On native Ubuntu:

    /usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -File ./tools/Build.ps1 -Package

Run host checks as separate commands:

    cargo clippy --all-targets -- -D warnings
    cargo build --no-default-features
    cargo test

Output: target/aarch64-skyline-switch/release/libssbu_online_deluxe_rewrite.nro.
dist/<NRO-hash-prefix>/ retains that NRO, its ELF, Cargo.lock and manifest. Keep each tested bundle.
Build references: [Skyline](https://github.com/ultimate-research/skyline-rs),
[cargo-skyline](https://github.com/jam1garner/cargo-skyline).

## Development use

Copy only the NRO to atmosphere/contents/01006A800016E000/romfs/skyline/plugins/.
Replace the old Deluxe/SsbuSync stack and overlapping standalone FX/latency plugins for this test.
The rewrite skips hooks when a loaded SsbuSync export is detected; that is a best-effort check,
not a load-order guarantee.

At character select, hold **L+R** or **ZL+ZR**:

- **Down** opens/closes the menu.
- **Up** selects the other row.
- **Left/Right** changes the selected value.

This edits an existing CSS banner. It does not capture or suppress ordinary game input.
The first controller (or handheld fallback) operates it. Layout/hotkeys need live verification.
Settings save to sd:/ultimate/ssbu_online_deluxe_rewrite/config.ini.
On/Off changes future requests; existing effects expire normally.

Build/tests do not establish runtime compatibility or lag reduction.
See [debugging](docs/DEBUGGING.md), [validation](docs/VALIDATION.md) and [status](docs/STATUS.md).
