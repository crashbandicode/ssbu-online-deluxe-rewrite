# Character effects

Off covers **111 exact names** plus Sephiroth's edge_fire3_screen2 screen request.
See src/effects.rs and reference/fx-policy.json. These include aliases, not 111 confirmed emitters.

| Group | Initial scope |
|---|---|
| Byleth | Axe aura, hold, flare and slash decoration |
| Ike | Named volcano/neutral-B effects |
| Roy / Chrom | Named eruption/volcano effects |
| Captain Falcon | Named Falcon Punch effects; separate article/model remains |
| Sephiroth | Named fire burst, smoke and core rings; one screen overlay |

On forwards original arguments. Off uses the peer's named null resource for ordinary requests
and skips only the listed screen request. It does not delete all effects, hide articles, alter posture,
suppress camera requests or rewrite Lua stacks. Newly requested effects follow the setting;
existing effects expire normally.

This covers less than the full peer. Ness/Lucas self-hit, Palutena, Aegis and context-sensitive
raw lists need owner/move qualification. The peer changes articles and uses global per-fighter
callback counters; adding an On guard would not restore persistent mutations.
General hit, shield, smoke, KO and status feedback are not in this filter.

## Reduced next

The peer already partly reduces: Byleth scales around 0.4–0.65, Palutena up-air marker at 0.85,
and preserved Sephiroth Burst2 rings. Its raw-heavy list overlaps those ring hashes in another
context; blindly merging it would remove the cue we want to keep.

| Approach | Retained feedback | Tradeoff to measure |
|---|---|---|
| Selective layers | Main explosion ring, charge-ready cue, hit spark; suppress secondary smoke/glows | Avoids omitted creation; strongest first candidate with current hooks |
| Smaller decoration | Recognizable flare at moderate scale | May reduce overdraw; update/draw setup costs can remain |
| Fewer particles / shorter decorative lifetimes | Similar silhouette | Needs verified emitter parameters or edited assets; not implemented |
| Cheaper authored substitute | Small flash/ring | Needs resource authoring and attachment/handle testing |

Start with Sephiroth's ring and fewer surrounding layers, then Byleth axe cues.
Measure request counts and frame times for the same move/scene on emulator and console separately.
Smaller scale alone does not establish lower CPU cost or input latency.
Rendering-transition crashes are a separate backend problem.

Reduced is unavailable in M0. Configuring it reports an error, uses default On/Auto and preserves
the invalid file; it never silently maps to Off.

## Provenance

User archive: effect disable-reduce source.zip; May 14 tags
ssbu_master_charge_fx_lite 2026-05-14-edge-fire-lite-j and
ssbu_screen_fx_off 2026-05-14-edge-fire-screen.
No compiled peer binary or Nintendo asset is bundled. Full audit: FX_SOURCE_AUDIT.md in the
original delay-mod workspace.
