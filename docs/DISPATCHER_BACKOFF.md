# Dispatcher backoff

The game's task dispatcher polls three local and three shared job queues in a
loop while it waits for a dependency. When every queue is empty and the awaited
job is still running, it does not wait on its event; it rescans immediately.
On the character select the main thread spends about 5 ms of every frame in
that loop while one UI layout job runs on a worker, and under CPU pressure the
two-image LessLag ring cannot hide the resulting variance (Switch: 48–50 fps on
the CSS, 60 in matches, before 0.4.0's present-at-flush removed the stall; see
[Rendering profiles](RENDERING.md)).

This build adds one inline hook at the point where all six probes have failed.
When the dispatcher is the main thread, every pending record is still blocked,
the queues are empty and no fiber is ready to resume, the thread sleeps briefly
before the game's own rescan. No job, counter, lock or ordering changes; the
hook only decides whether to wait before repeating. It is installed for every
rendering profile and scene because the waste is in game code and exists in
Vanilla too.

Eden A/B/A with a 175% CPU quota on the CSS: LessLag 52 → 59 fps and 18% less
CPU per frame; Vanilla and LLDoubles kept 60 with the same CPU saving; a live
match lost no frames. Those are emulator results; the hardware benefit is
measured separately (see [validation](VALIDATION.md)). The Performance
view shows a **Backoff** row: sleeps per second and the last sleep length,
`Idle` when nothing is waiting, `External` when another plugin already owns
the hook site, `Unavailable` when the game image does not match 13.0.5, and
`Off` when disabled.

## Tuning

Tuning is optional. Create `sd:/ultimate/ssbu_online_deluxe_rewrite/dispatcher_backoff.ini`:

```ini
# idle passes to spin before the first sleep (default 16)
spin_passes=16
# sleep on every Nth idle pass after that (default 1)
every_passes=1
# first sleep and the cap of the doubling ramp, nanoseconds (default 5000 / 50000)
base_ns=5000
max_ns=50000
# minimum gap between sleeps in microseconds (default 0 = none)
cooldown_us=0
# 1 lets worker dispatchers sleep too (default 0: MainThread only)
all_threads=0
# preset=measured reproduces the Eden diagnostic cadence (1 us once per 171 passes)
```

`dispatcher_backoff=false` in `config.ini` (0.5.3) keeps the hook installed
but never sleeps, so an A/B needs only an INI edit and a restart; the Backoff
row then reads `Off (ini)`. `DISABLE_BACKOFF` in the data directory skips the
install entirely. The same fix ships alone as the
[ssbu-dispatcher-backoff](https://git.syntaxbyexample.com/intpa/ssbu-dispatcher-backoff)
plugin; exactly one of the two installs the hook, whichever loads first, and
the other reports `External`.

## Eligibility gate

Since 0.3.1 the hook also has an eligibility gate: it never sleeps before one
of the scene hooks has fired (main menu, Quickplay, Arena, local wireless,
stage setup or the character select), and never while the tracked window has
not presented for 250 ms. Boot, the logo/movie/title sequence and a stalled
renderer therefore run the game's original loop. The Backoff row reads
`Armed` until the gate first passes; event log code 102 with `a=5` marks
that moment. If the scene hooks fail their own checks the gate stays closed.
