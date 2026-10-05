# Online Deluxe rewrite

Private Rust/Skyline reconstruction for Smash Ultimate **13.0.5**, delivered as **one NRO**.
This is an initial development build, not a completed LessLag replacement.

Implemented:
- Global top-left Player Info window with the fork's table/navigation UX: character FX **On / Off**, default **On**. Reduced is unavailable.
- Online input buffer **Auto / 0–25**, retaining the fork's Quickplay null-pointer guard.
- Bounded events, persistent install checkpoints, panic diagnostics, build identity and matching debug symbols.
- Expanded named character effects from the supplied peer mod: [exact scope](docs/FX.md).

**Rendering remains Vanilla.** LessLag/Ultra/Doubles reconstruction, automatic profile application,
network-player telemetry, overclocking and performance scaling are pending.
The fork's original source and exact changes are preserved under [reference](reference/README.md).
See the [parity ledger](docs/STATUS.md).

The ImGui/NVN backend is embedded in this NRO; no separate ImGui plugin is needed.
No external SsbuSync backend, PIA manager, smashline, ARCropolis assets or extra FX NRO
is required by this build. Skyline itself is required. Training Modpack and SGPO remain their own mods.

## Build

Host Rust is pinned in rust-toolchain.toml. Switch builds use installed cargo-skyline 3.5.0 /
skyline-v3; each bundle records the actual compiler. Prepare the pinned native library
(see [provenance](docs/IMGUI_DEPENDENCY.md)) before building:

    /usr/bin/python3 tools/prepare_imgui.py

On native Ubuntu:

    /usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -File ./tools/Build.ps1 -Package

Run host checks as separate commands:

    cargo clippy --all-targets -- -D warnings
    cargo build --no-default-features
    cargo test

Output: target/aarch64-skyline-switch/release/libssbu_online_deluxe_rewrite.nro.
dist/<NRO-hash-prefix>/ retains that NRO, its ELF, Cargo.lock, full source archive,
native dependency provenance and manifest. Keep each tested bundle.
Build references: [Skyline](https://github.com/ultimate-research/skyline-rs),
[cargo-skyline](https://github.com/jam1garner/cargo-skyline).

## Development use

Copy only the NRO to atmosphere/contents/01006A800016E000/romfs/skyline/plugins/.
Replace the old Deluxe/SsbuSync stack and overlapping standalone FX/latency plugins for this test.
The rewrite skips hooks when a loaded SsbuSync export is detected; that is a best-effort check,
not a load-order guarantee.

The menu draws independently of the game UI. Hold **L+R** or **ZL+ZR**, then tap
**Down** to cycle **settings → performance → hidden**. In the settings window:

- **Up/Down** selects FX or NetLatency.
- **Left/Right** changes the value; shoulders are no longer required.
- Holding **Plus/Minus** with the toggle chord opens debug view from settings.

Current Eden keyboard mapping: **Q/E = L/R**, arrow keys = D-pad.
The window retains the fork's Name/Ping/NetLatency/NetProfile table and frame graph,
with an added FX row. Ping telemetry is pending (---); NetProfile stays Vanilla
until the render engine is implemented. Frame statistics measure intervals between
overlay render callbacks. Main-menu/CSS/match appearance, SGPO and Training Modpack
menu/save-load coexistence are confirmed on the prior Eden candidate. Expanded
effect coverage, online scope transitions and console behavior await validation.

The first controller (or handheld fallback) operates it. Ordinary game input is not
suppressed. No game-owned text pane or Layout::Draw hook is used.
Settings save to sd:/ultimate/ssbu_online_deluxe_rewrite/config.ini; config.previous.ini
retains the last good version for recovery across interrupted replacement.
On/Off changes future requests; existing effects expire normally.
effects_scope=online is the default. Use effects_scope=all for offline + online testing;
INI changes require restarting the game. See [default config](config/default.ini) and
[Eden test override](config/eden-test.ini). Install the test config with the matching
new NRO; the older live test build cannot parse this new key.

Build/tests do not establish runtime compatibility or lag reduction.
See [debugging](docs/DEBUGGING.md), [validation](docs/VALIDATION.md) and [status](docs/STATUS.md).
