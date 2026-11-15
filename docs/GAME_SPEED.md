# Game speed (0.10.8 candidate)

The Settings strip in a networked battle shows “Game 100%”, or a lower
percentage when native elapsed match time advances more slowly than wall
time. A short hint describes sustained slowdown (“delay too low”) or abrupt
dips (“lag spikes”). Host or emulator slowdown can produce the same readings;
the hints do not identify a faulty peer or diagnose the network.

The readout covers Quickplay, Elite Smash, Arena and local-wireless battles.
It is absent in Training, waiting-room practice, offline play, character select,
results and menus. It starts as “Game --” while collecting the first second.
Native character-select and lobby banners are unchanged.

The current smoothing and cause values are **provisional**, pending paired
calibration and the primary's ruling:

- A one-second trailing window, updated at most 10 times per second.
- Below 97% continuously for two seconds selects “delay too low”.
  At least 99% for one second clears that hint.
- A 100 ms interval without advancement, or a fast window of at least 250 ms
  below 80%, selects “lag spikes” for two seconds, ahead of the sustained hint.

Discrete counter quantization can classify steady 80% as a spike with these
provisional values. Final values will be recorded with measurements. A frozen
render cannot update the text until rendering resumes; the next sample retains
the full elapsed interval.

## Native source and lifecycle

The source is 13.0.5 main+0x52e8b44, a four-byte mirror of the current match
object's native elapsed time at +0x3d8, expressed in 60 Hz units. Owning update
main+0x1572540 accumulates the native fractional rate and obeys native pause
state; main+0x157d738 publishes the mirror. This is native elapsed match time,
not an unconditional engine-tick serial or a presentation count.

The constructor resets it to zero. Its signed, nonnegative 32-bit range
saturates at 21,599,999 (100 hours minus one frame). Decrease, saturation,
missing eligibility or a new stage restarts or invalidates the estimate.

The native owner, copy and ReadyGo getter have exact 64-byte identity checks.
ReadyGo is read through its native getter's checked indirections on each draw,
without retaining the heap match object. Every stage attempt starts a new
generation before validation. Rejected stages, same-scene rematches, menu
entries and CSS cannot reuse the previous stage's offer. The current native
mode/melee words must also prove a networked battle; stored connection and
latency-hook marks do not supply that proof. The CSS invalidation hook must
actually have installed before sampling is armed.

Sampling occurs inside the existing ui.rs::draw and UI try-lock, including
while the overlay is hidden. It uses GetSystemTick at 19,200,000 ticks/second
and a fixed 16-point ring. Percentage is native elapsed-frame delta divided
by actual wall time and 60, rounded and bounded to 0–100 for display; raw
rates drive the hints. There is no new hook, heap allocation, blocking lock,
file I/O, game-memory write or thread on this path. The bounded fact joins the
existing timing segment; the three-fact capacity is unchanged.

## Counter evidence, 2026-11-15

The native source NSO SHA-256 is
8b3dc2eb011f26dda1f87947e4f62665e8d8fe3d84397e85ecf9b3cd9d0137ce,
build ID 21450c647b8c5940ed5b68218ea128fd3c5d597d000000000000000000000000.
Focused REA established the owner, reset, native rate and bounds. Actual
64-byte instruction windows were independently checked against that source.

On released 0.10.7 in Eden, native hitlag stayed active across 13 samples while
its countdown fell 17→1 and this counter advanced 3158→3174; all 12 adjacent
deltas matched. Two earlier positive intervals also advanced, with one-frame
endpoint differences retained in the research evidence. Running reads are
not atomic.

In the real LAN Online pair, the running peer's source and mirror stayed flat
at 4582 for at least 0.406626311 seconds and 5017 for at least 1.429811239 seconds
during the other peer's 0.5/1.5 second process pauses, then resumed. All 2,557
pre-end paired samples had native ReadyGo=1, enabled=1 and native rate=1.0.
This is counter evidence; 0.10.8 UI, performance and hardware acceptance remain
separate checks.

The private evidence ledger is artifacts/deluxe-gamespeed/counter/REVIEW.md
in the operator's Training lobby workspace. Native binaries stay outside this
repository. Candidate host gates, paired 92%/80%/pause calibration, UI negatives,
media and final NRO/resident sizes will be recorded in
[VALIDATION.md](VALIDATION.md). Hardware is untested.

## Diagnostic trace

The diag variant alone records code 122 at most 10 times per second through
the existing nonblocking recorder. Event.tick is the sampled SystemTick.
The fields are:

- a bits 0–31: native elapsed frames (u32::MAX when ineligible).
- a bits 32–39: displayed percentage (0xff during warmup or absence).
- a bits 40–41: cause (0=none, 1=delay, 2=spike).
- b: current stage token.

The product emits no such sample stream. These records contain actual sampled
display facts; they do not recreate every intervening UI draw.
