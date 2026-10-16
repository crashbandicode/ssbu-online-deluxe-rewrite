# How Deluxe LessLag and LLUltra work

LessLag shortens the path between the game's work on a frame and that frame
appearing on screen. It uses fewer active rendering images, changes which
image the renderer selects, and rearranges rendering work. LLUltra takes the
same two-image setup further with an earlier submission schedule.

The aim is more recent visual feedback at the game's usual frame rate. These
profiles do not make the game simulation run faster. This guide describes the
rewrite's implementation; the measured Eden results are recorded separately
below rather than inferred from its buffer counts.

## Less room for frames to accumulate

When you press jump, the CPU updates the game and prepares drawing commands.
The GPU executes those commands into an image held in a buffer. The display
system shows that image at an eligible refresh. Several images can be at
different points along that path at once.

More buffers give the CPU and GPU room to keep working when another image is
still in use. They also leave more room for older visual information to remain
in the pipeline. LessLag reduces the active images from three to two.

![Vanilla, LessLag and LLUltra active buffers and index policies](media/rendering-pipeline-three-profiles.svg)

The boxes show active image capacity, not fixed frame ages, GPU completion, or
an exact number of frames of input delay. Buffer A is an illustrative label,
not an image the game permanently reserves for the oldest frame.

| Profile | Active images | Relative index selection in steady operation | Project policy name |
| --- | --- | --- | --- |
| Vanilla | 3 | `index + 2`, then the game's existing `% 3` | `TwoBehind` |
| LessLag | 2 | `(index + 1) % 2` | `OneBehind` |
| LLUltra | 2 | `index % 2` | `Immediate` |

The additions wrap as unsigned arithmetic in the actual code. These are
cyclic image indices, not displayed frame numbers. `Immediate` is this
project's policy name; it does not mean an image appears instantly or that
the GPU has finished drawing it.

Code: [active image counts](../src/config.rs#L64),
[profile offsets](../src/render_policy.rs#L49),
[`index_value`](../src/render_policy.rs#L348), and
[policy names](../src/resolution.rs#L94). The runtime applies the active-image
count through [the window wrapper](../src/runtime/nvn.rs#L152).

## More than changing the buffer count

The following describes steady operation after the requested profile has been
confirmed. A transition or a safety guard can keep the game's original path.

| Change | What the rewrite actually does | Code |
| --- | --- | --- |
| Fewer active images and a different index | LessLag and LLUltra use the two-image selections above. | [`index_value`](../src/render_policy.rs#L348) |
| Skip one rendering wait | The wait hook returns `0` instead of calling the original function when the policy permits it. | [wait hook](../src/runtime/renderer.rs#L116), [`skip_wait`](../src/render_policy.rs#L362) |
| Move rendering work | Model updates and fighter drawing-command preparation run after the scene update; the deferred rendering queue is then dispatched. | [post-scene work](../src/runtime/render_sequence.rs#L148) |
| Submit earlier in LLUltra | LLUltra flushes at the earlier present-entry hook and suppresses the later flush in steady operation. Its flush plan calls the original function three times. | [early/late hooks](../src/runtime/render_ultra.rs#L27), [`plan_flush`](../src/render_policy.rs#L417) |
| Request presentation at the flush | LessLag does this since 0.4.0; LLUltra since 0.5.2. The later presentation request for that same image is skipped. | [flush hook](../src/runtime/renderer.rs#L219), [presentation wrapper](../src/runtime/nvn.rs#L372) |

![Rendering work placement, flush and presentation for all three profiles](media/rendering-schedule-three-profiles.svg)

This is a sketch of selected rendering steps, not a complete frame timeline.
The sequence hooks use one decision for the frame. They keep the game's own
ordering while the teardown guard is active, rather than relocating work on
objects being unloaded. The hook families are
[installed together after their checks](../src/runtime/renderer.rs#L265).

## Skipping a wait does not remove GPU dependencies

LessLag and LLUltra can skip one particular rendering wait. That lets the CPU
continue at that point, but an image still has to be available before it can
be reused. Acquiring an image can still wait on GPU work. The relocated
fighter work also retains its own worker-completion wait.

With only two active images there is less spare capacity to hide a slow frame.
Waiting at a different point can consume the apparent saving. That is why
buffer selection, work ordering and presentation timing need to be considered
together.

Code: [the guarded wait decision](../src/render_policy.rs#L362),
[the original image-acquire call](../src/runtime/nvn.rs#L480), and
[the relocated worker wait](../src/runtime/render_sequence.rs#L148).

## Present at the flush keeps the submitted image moving

A flush submits GPU commands; it does not establish that the GPU has finished
them. A presentation request queues the image for display. Showing the image
is a later event, once the GPU and display system can make it visible.

The game's original order could submit an image's drawing commands, run the
next frame's CPU update, and only then request presentation of the previous
image. Present-at-flush moves that request immediately after submission.

![The same frame queued before or after the next CPU update](media/present-at-flush-timeline.svg)

Both rows follow frame 99. The CPU does the same work on frame 100 in each.
The times and refresh outcomes are illustrative, assuming GPU completion and
display acceptance in time. Requesting presentation at a refresh does not
guarantee that the image appears at that refresh.

This change addresses a frame-pacing problem exposed by the shorter pipeline.
It does not guarantee that every expensive scene meets its deadline, and it
did not change the captured response counts in the Eden re-measurement below.
The [rendering guide](RENDERING.md#present-at-the-flush-040) records the
separate console result: the LessLag character-select hand sweep improved
from 49.9 to 59.85 fps. That is a console observation, not an Eden result or a
claim about every scene.

## Why LLUltra can go further, with less timing headroom

Both profiles use two active images. LLUltra selects with a +0 offset instead
of +1 and submits the graphics work at the earlier hook. In the recorded
console traces, that schedule left less opportunity to overlap the CPU's work
with GPU work before the next refresh. Expensive scenes therefore exposed
missed deadlines in LLUltra even when LessLag held its frame rate.

That behavior was observed in the
[0.5.2 console move captures](VALIDATION.md). It is not a conclusion
drawn from the word `Immediate`, nor a guarantee that LessLag is always smooth
or LLUltra always stutters.

There is also a resolution difference: the console defaults are 1080p for
LessLag and 576p for LLUltra, with dynamic resolution enabled for both. In Eden
both default to 1080p with dynamic resolution off. Reducing the GPU's pixel
work is a separate setting from shortening the pipeline.
Code: [`defaults_for`](../src/resolution.rs#L115).

## What we actually measured in Eden

These are recorded benchmark results, not estimates from the diagram. The
2026-10-15 run used the 0.5.2 NRO `1525e3e0`, built from `ce0145a`, in Eden
`10bcd2d849`. It captured synthetic stick-down input and the first visible
crouch in Training, using the same plugin set and three trials per cell.
The later phase injected input about 8 ms later.

| Profile | Normal-phase captured response, three trials | Later-phase captured response, three trials | Fewer frames than Vanilla |
| --- | --- | --- | --- |
| Vanilla | 6, 6, 6 | 6, 6, 6 | Baseline |
| LessLag | 3, 3, 3 | 3, 3, 3 | 3 in both phases |
| LLUltra | 2, 2, 2 | 2, 3, 3 | 4 normally; 3–4 in the later phase |

LessLag's three-frame reduction is observed in these trials. Removing one
active image does not, by itself, explain three saved frames: the tested
profile combines the changes above. The experiments did not isolate how much
each change contributed. LLUltra's extra frame of improvement was consistent
in the normal-phase trials; its later-phase advantage varied within the
observed trial spread.

These are consecutive emulator captures with in-game input markers, not
Switch controller-to-screen measurements. A nominal three frames at 60 Hz is
50 ms, but that conversion is not a measured physical-display saving. The
marker observes input in a plugin callback and does not locate the game's
first input read, so a later response does not prove a missed input sample.

The full experiment included LLDoubles as a fourth profile. All 24 trial
windows had consecutive image identities; seven trials lacked matching
API-trace coverage and rely on the image markers. The frame counts were
unchanged from the earlier builds despite the present-at-flush changes.
Full setup, counts and limitations:
[measurement methodology](LATENCY_METHODOLOGY.md#re-measurement-on-052-present-at-flush-2026-10-15),
[performance record](PERFORMANCE.md#response-trials-re-measured-on-052-2026-10-15),
and [validation ledger](VALIDATION.md).

[Back to the README](../README.md#additional-optimizations-not-yet-present-in-deluxe)
