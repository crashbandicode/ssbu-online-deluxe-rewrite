# Online Deluxe rewrite

Private Rust/Skyline reconstruction for Smash Ultimate **13.0.5**, delivered as **one NRO**.
This is a development build. It is not yet a complete replacement for the fork's
runtime profile system. The latest live-profile build has been tested in Eden;
its new profile transitions still need Switch validation.

Implemented:
- Global top-left Player Info window with the fork's table/navigation UX: character FX **On / Off**, default **On**. Reduced is unavailable.
- Online input buffer **Auto / 0–25**, retaining the fork's Quickplay null-pointer guard.
- Opponent RTT with the fork's stability colors, from the game's existing Pia measurements. Names, interface suffixes and peer settings remain pending.
- Compatible standalone ImGui host reuse, with the embedded backend as a fallback.
- In-menu Vanilla / LessLag / LLUltra selection, applied live and saved to INI.
- Bounded events, persistent install checkpoints, panic diagnostics, build identity and matching debug symbols.
- Expanded named character effects from the supplied peer mod: [exact scope](docs/FX.md).

**Rendering defaults to Vanilla.** `render_mode` selects the initial policy:

| `render_mode` | Behavior |
| --- | --- |
| `vanilla` (default) | The game's three-texture TwoBehind behavior through the installed hook set |
| `lesslag` | Full modern LessLag rendering: two active textures, OneBehind index/flush, render-finished poll bypass and render_opts fighter model/recording/dispatch sequencing |
| `llultra` | LessLag sequencing plus Immediate index, a guarded pre-present flush, triple original flush and suppression of the later flush |
| `lesslag_core` | Component experiment: LessLag without render_opts sequencing |
| `doubles_core` | Component experiment: three textures with OneBehind; not full LLDoubles |

Vanilla, LessLag and LLUltra can be changed from NetProfile during menus or a
match. The renderer drains pending work and confirms the native buffer count
before reporting Active. Core experiments still require a restart. Automatic
online selection, dynamic resolution and clock changes remain pending.
In custom Eden, captured-image evidence at the normal injection phase shows the crouch response
three frames earlier with `lesslag` and four with `llultra` than with `vanilla`;
delayed-input trials vary by one frame while preserving the ordering.
See [performance](docs/PERFORMANCE.md) for the method and limits. That is not a
console, display or end-to-end latency result. Full peer metadata and Doubles
validation remain pending.
The fork's original source and exact changes are preserved under [reference](reference/README.md).
See the [fork parity ledger](docs/FORK_PARITY.md) and [validation evidence](docs/VALIDATION.md).

The ImGui/NVN backend is embedded in this NRO; no separate ImGui plugin is needed
for Deluxe. If another mod requires the compatible standalone host, Deluxe reuses
it. Unknown host ABIs are refused; see [dependency details](docs/IMGUI_DEPENDENCY.md).
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

- **Up/Down** selects NetLatency, NetProfile, FX or Stealth.
- **Left/Right** changes the value; shoulders are no longer required.
- Holding **Plus/Minus** with the toggle chord opens debug view from settings.

Current Eden keyboard mapping: **Q/E = L/R**, arrow keys = D-pad.
The window retains the fork's Name/Ping/NetLatency/NetProfile table and frame graph,
with added FX and Stealth rows. One fresh opponent's averaged RTT appears with stability
color; absent, stale or ambiguous peers show uncolored `---`. NetProfile reports
the saved selection and its live status; `*`/`Swap` marks a pending transition.
Stealth currently displays **Unavailable**: its preference is saved, but its
network behavior is not implemented. Frame statistics measure intervals between
overlay render callbacks. Main-menu/CSS/match appearance, SGPO and Training Modpack
menu/save-load coexistence are confirmed on the prior Eden candidate. Expanded
effect coverage, online scope transitions and console behavior await validation.

The first controller (or handheld fallback) operates it. Ordinary game input is not
suppressed. No game-owned text pane or Layout::Draw hook is used, and no mode patches
SGPO's 0x374812c neighborhood.
Settings save to sd:/ultimate/ssbu_online_deluxe_rewrite/config.ini; config.previous.ini
retains the last good version for recovery across interrupted replacement.
On/Off changes future requests; existing effects expire normally.
effects_scope=online is the default. Use effects_scope=all for offline + online testing;
External INI edits require restarting the game; supported menu changes apply live
and persist automatically. `stealth=false` is the default saved preference.
See [performance measurements](docs/PERFORMANCE.md), [default config](config/default.ini) and
[Eden test override](config/eden-test.ini). Install the test config with the matching
new NRO; older builds may reject configuration keys they do not recognize.

Build/tests do not establish runtime compatibility or lag reduction.
See [debugging](docs/DEBUGGING.md), [validation](docs/VALIDATION.md) and [status](docs/STATUS.md).
