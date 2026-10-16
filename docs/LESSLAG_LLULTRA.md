# How Deluxe LessLag and LLUltra work

When you press a button, Smash processes the input long before you see the result. The CPU must update the game, the GPU must render the image into a buffer, and the display system must show it.

![Input, CPU update, GPU draw, presentation request and screen refresh for overlapping frames](media/rendering-pipeline-overlap.svg)

Illustrative, not a measured trace or a profile-specific schedule. Follow frame N: its presentation request can be queued while the GPU is still drawing it. The CPU can prepare N+1 at the same time. The display shows N at a later eligible refresh, once it is ready.

## Vanilla: headroom first

**Vanilla Smash** uses three buffers and a longer rendering pipeline. This gives the CPU and GPU more breathing room to prepare frames, but increases the delay before your inputs become visible.

![Vanilla has three reusable image buffers; LessLag and LLUltra each have two](media/framebuffers-three-vs-two.svg)

A framebuffer is reusable memory; a frame is the picture drawn into it. The presentation queue tracks requests to display those pictures. More buffers allow more frames in circulation. Removing one buffer does not itself explain three saved frames: the measured reduction belongs to the combined buffer selection, rendering schedule and waiting changes.

## LessLag: a shorter pipeline

**LessLag** reduces this to two buffers, changes which frame gets rendered, and eliminates unnecessary waiting. Together, these changes shorten the pipeline, reducing input delay by approximately three frames.

## LLUltra: shorter still, less headroom

**LLUltra** pushes the pipeline further, potentially saving a fourth frame. However, this leaves less time for the CPU and GPU to finish their work. If they miss the display's deadline, FPS can drop and the additional latency advantage disappears.

![Captured frames from the press to the first visible response: Vanilla 6, LessLag 3, LLUltra 2 to 3](media/latency-captured-frames.svg)

Measured in Eden on 0.5.2 as captured images from an injected input to the first image showing the response. They are captured-response frames, not controller-to-photon timings. Full tables and clips: [Measured input delay](../README.md#measured-input-delay). Method: [Measuring frames removed by LessLag and LLUltra](LATENCY_METHODOLOGY.md).

![At CPU simulation N, Vanilla conceptually renders N−2, LessLag N−1 and LLUltra N; heavy GPU work can make LLUltra miss its next refresh](media/llultra-deadline-tradeoff.svg)

At the same CPU simulation point N, the image labels N−2, N−1 and N illustrate the relative depth of TwoBehind, OneBehind and Immediate. These are conceptual content ages, not measured frame ages or fixed buffer identities. The implementation uses cyclic image selection; its offsets alone do not establish those ages.

Illustrative timings. In console Training captures on 0.5.2, expensive moves caused FPS drops in LLUltra while Vanilla and LessLag held their frame rate in those sessions. This is a scoped observation, not a guarantee for every scene or device. See [Implementation and timing tradeoffs](RENDERING.md#implementation-and-timing-tradeoffs).

### Following the same input

![Detailed CPU, GPU and presentation timelines with six captured-frame guides and separate occupancy insets: three reusable images in Vanilla, two each in LessLag and LLUltra](media/same-input-earlier-response.svg)

Follow N, the picture carrying the same press. The six guides and response dots show the Eden counts: Vanilla 6, LessLag 3, LLUltra 2–3. LLUltra's outlined marker is the alternative later-input result. Left insets show three reusable images versus two; their shown, queued and rendering roles can move between slots. CPU/GPU work and request positions are illustrative. Conceptual buffer occupancy, not an exact trace.

## Recovering smoothness

Finally, **dispatcher backoff and present-at-flush** reduce wasted CPU work and prevent late presentation requests, helping recover FPS consistency. Faster emulator hardware generally has more processing headroom, making these tighter deadlines easier to meet.

Dispatcher backoff briefly sleeps instead of repeatedly checking empty work queues; it runs in every profile, Vanilla included. Present-at-flush queues an image as soon as its final drawing commands are submitted, before the next CPU update. Submission does not mean the GPU has finished or the image is on screen. These additions support smoothness rather than independently explaining the original latency reduction; the captured-response counts were unchanged after present-at-flush.

See the diagrams under [Additional optimizations not yet present in Deluxe](../README.md#additional-optimizations-not-yet-present-in-deluxe), [Dispatcher backoff](DISPATCHER_BACKOFF.md) and [Present at the flush](RENDERING.md#present-at-the-flush-040). Full profile behavior is in [Rendering profiles](RENDERING.md).
