# G&W four-fighter check, 2026-10-08

The corrected v0.1.1 G&W plugin executed its verified shader and range hooks with four idle stock G&Ws in Training. This Intel/ANV machine was already near 60 FPS without it. The enabled window had fewer low-FPS samples, but the original RTX 3050 Ti 28-to-60 FPS improvement was not reproduced here.

| Plugin | Median FPS | Minimum sampled FPS | Samples below 59.5 FPS |
| --- | ---: | ---: | ---: |
| Absent, first | 59.997 | 58.970 | 7.56% |
| Enabled | 59.998 | 59.918 | 0% |
| Absent, repeat | 59.994 | 57.010 | 5.78% |

Each window followed at least 30 seconds of warmup and sampled for 45 seconds, retaining all 225 regular snapshots. Eden reports correlated, smoothed FPS. These are not individual frame times; zero samples below 59.5 is not proof that every frame completed within 16.67 ms. One enabled window is also insufficient for a general performance guarantee.

The game was Smash 13.0.5, using Vulkan/ANV, accurate GPU, 1x resolution, docked mode, Immediate VSync and 100% speed. Training CPU count was three with Stop behavior; all four fighters remained idle at 0%. Menus were hidden; screenshots were taken outside timing. The rewrite stayed on manual Vanilla, FX On and FPS++ Off. Settings and plugin inventories matched except for the intended G&W addition. The repeat absent run used different stock CPU colors, so exact costume parity is not claimed.

The enabled log verified shader fingerprint `3869d88ad69eee6a` and range restoration. Exact artifacts:

- G&W NRO: `bceef1f09e4339149cdab8e7826d17492680add02307002b0a5d62033d1090c8`, source `a534bb533bc0bd5b2fd0af0c518da55647337c44`.
- Constant rewrite NRO: `a9c38567b4e368d2d7723d760a2b7852503cde3688ba419e221e1bf03a794dcd`.
- Eden source: `10bcd2d849843b146a79c61a21df615e1bdcfcde`.

Evidence, including raw samples, screenshots, inventories, hook excerpts and restoration receipts, is retained locally at:

```text
/home/intpa/.local/share/ssbu-mod-backups/gnw-four-training-1007/measurement-artifacts
```

All three measurement windows completed without an in-match crash and with zero recorded OOM events. The enabled run exited normally. Two absent runs had frontend shutdown failures after explicit game closure, which are not attributed to the plugin. The isolated profile settings, plugins and cosmetic assets were restored and verified; the Switch was untouched.

Earlier attempts with G&W Galaxy cosmetics or incorrectly selected Ice Climbers were excluded. A clean run on the original RTX host remains necessary to check the larger reported slowdown with the latest binary.
