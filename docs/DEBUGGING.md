# Diagnosing crashes

Keep the exact dist/<NRO-hash-prefix>/ bundle for every installed build.
Packaging checks matching NRO/ELF executable bytes, saves both SHA-256 hashes,
the NRO module ID and compiler identity. The optimized ELF retains DWARF and symbols.

Under sd:/ultimate/ssbu_online_deluxe_rewrite/:

- install.log: identity/base addresses and synchronous before/after install checkpoints.
  Previous boot: install.previous.log.
- events.log: background snapshots flushed every 500 ms. One previous log is retained;
  each is capped at 4 MiB.
- panic.log: best-effort Rust panic message/location and recent events. Retained until another panic.
- config.ini: saved settings; invalid input is reported and preserved.

128 fixed-size events use a nonblocking try-lock. Game hooks do no file I/O; contention
increments a dropped counter. Startup install checkpoints are synchronous. The sink reports
overwritten sequences/drops. File/thread failures are reported to Skyline.
Up to the latest 500 ms may be absent after an abrupt crash; this is not a crash dump.

The panic hook chains the previous mod's hook. panic=abort cannot recover through catch_unwind.
Hardware exceptions, GPU faults and hangs do not invoke it. Collect Atmosphere/emulator crash
logs too: module list, PC/LR/backtrace, game version, loaded NROs and reproduction steps.

    /usr/bin/python3 tools/symbolicate.py dist/<hash> 0x<plugin-relative-PC>
    /usr/bin/python3 tools/symbolicate.py dist/<hash> 0x<absolute-PC> --base 0x<plugin-base> --module-id <crash-module-id>

The script verifies bundle hashes and optional module ID, then uses LLVM/binutils addr2line.
Use the faulting plugin's base/ELF, not the game's main-module base.

For controlled bisection, create empty DISABLE_MENU, DISABLE_FX or DISABLE_BUFFER files
in the data directory and restart. They skip the corresponding hook family.
All three produce observer-only behavior. Remove to re-enable.
These differ from FX On, which keeps passthrough hooks installed.

| Code | Meaning |
|---|---|
| 1 / 2 | Boot / unsupported game version |
| 3 / 4 / 5 | Hook installing / installed / rejected |
| 6 / 7 | Config loaded / rejected |
| 8 | Setting changed |
| 9 / 10 | Network buffer applied / null pointer skipped |
| 11 | FX suppression count: first then every 128 |
| 12 / 13 | Sampled CSS entry / exit |
| 14 / 15 / 16 | Panic / heartbeat / compatibility or diagnostic flag |
| 17 | Shared function hook entry instruction before chaining |
| 18 / 19 | Accepted title pane / unsupported CSS pane or mapping |

Fixed offsets have source/version, text-bound and common-trampoline checks.
Original instruction signatures have not been captured from the game: this is not a complete
validator. Shared EffectModule function hooks deliberately chain. Live load-order validation
is separate from successful compilation.

CSS pointers now use Horizon QueryMemory permission/range checks before each read,
and writes require a title TextBox name verified from the user's local CSS asset.
The chain stops at its second value (the pane), not a third dereference (the vtable).
This does not prove object lifetime; access stays inside the owning CSS callback before
calling its original, with no pane pointer retained for later dereference.
QueryMemory ABI reference: https://github.com/switchbrew/libnx/blob/master/nx/source/kernel/svc.s
