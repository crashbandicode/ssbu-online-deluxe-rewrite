# hoc-standin (test only)

A Skyline plugin that stands in for the game-aware hoc-clk fork
([intpa/horizon-oc-game-aware](https://git.syntaxbyexample.com/intpa/horizon-oc-game-aware),
protocol v1) inside the Smash process, so Deluxe's Horizon-OC client, gate,
ZR+L3 toggle and clock mark run end to end in Eden. Eden cannot run
Horizon-OC. **Never install it on a console and never ship it with Deluxe.**

What it does:

- Registers `hoc:clk` with Eden's sm and serves commands 0 (`GetApiVersion` =
  2) and 100-103 (`GameAwareGetInfo`, `Arm`, `SetApply`, `Disarm`) with the
  fork's arm/lease semantics (`src/protocol.rs` mirrors the fork's
  `mgr/game_aware_policy.hpp`). The server loop follows hoc-clk's
  `ipc/ipc_server.c`.
- Changes no clock. "Applying" only means a lease is active in its replies.
- Logs integration evidence to `sd:/ultimate/hoc_standin/standin.log`:
  registration, sm presence probes, accepted sessions, the first use of each
  command, and arm/apply/disarm edges. Deluxe records its side as events 114
  and 115.

What it found on its first run (Eden GIF build `9a5d704c9c`): Eden's TIPC
`AtmosphereHasService` reply carries `false` for every service, built-in
`fsp-srv` included, while its CMIF reply is correct. Deluxe therefore asks over
CMIF when `platform=emulator` (`HasServiceFraming` in `src/hoc_game_aware.rs`);
consoles keep TIPC, the only framing Atmosphère accepts for that command.

Build and check from this directory:

```powershell
cargo test
cargo clippy --all-targets -- -D warnings
cargo skyline clippy -- -- -D warnings
cargo skyline build --release
```

Install `target/aarch64-skyline-switch/release/libhoc_standin.nro` next to
`libssbu_online_deluxe_rewrite.nro` in an **isolated Eden profile** only, with
`platform=emulator` and `hoc_oc=online` in Deluxe's `config.ini`.
