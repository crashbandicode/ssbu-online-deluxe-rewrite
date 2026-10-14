# Implementation and validation status

The rewrite is one Rust/Skyline NRO for Smash 13.0.5. Auto is enabled by
default; it chooses Vanilla in menus, offline play and Training, LessLag for
console online singles, LLUltra for emulator online singles, and LLDoubles for
online matches with more than two fighters. Platform is an explicit INI setting.
No hardware overclocking is implemented.

Publication routing (updated October 14): `origin` is Forgejo
(`git.syntaxbyexample.com/intpa/ssbu-online-deluxe-rewrite`) and is
authoritative; this repository remains private. `github` is a second,
downstream remote that is not pushed automatically. Nothing is pushed without
the operator's explicit permission. On October 9 origin still pointed at GitHub;
that is no longer true.

| Feature | Implementation | Runtime evidence / remaining check |
| --- | --- | --- |
| Vanilla / LessLag / LLUltra | Live guarded transitions, persistent manual choice | Earlier linked Eden matches and transitions passed; historical frame measurements are tied to their exact binary |
| LLDoubles | Three textures, OneBehind, render sequencing, resolution scaling | b19 linked Auto/LLDoubles team match completed through results/CSS with matching observations |
| Auto | Scene, connection and fighter-count policy; preserves saved manual choice | Training/menu Vanilla, b19 linked doubles LLDoubles, d9 linked singles LLUltra and cleanup Vanilla passed |
| Resolution / DRS | Applied internal resolution shown separately from output size | Console-policy Eden Gigaflare tests passed for LessLag, LLUltra and LLDoubles; Vanilla remains off; console hardware untested |
| Fighter Main | Guarded read-only fighter status callback, bounded report tokens | Two count-only Training sessions passed approximate tick-frequency checks; count mode installs no cut-in hook or reports |
| Cut-in scaler | Original cut-in function chained, seven eligible fighter-tick cooldown | Luigi close-range Up-B cut-in passed in Eden: LLDoubles 1080→720→1080; Switch untested |
| Online input buffer | Auto or 0–25, retained null guard and Quickplay path | User menu/persistence checks; actual network buffer changes need live online coverage |
| Stealth | Default On, connection-latched beacon/payload suppression, reconnect notice | d9 fresh-On suppression/zero custom writes, pending-Off and saved Off after restart passed; earlier pending-On passed |
| Peer display | Remote column exists only for a selected connected station; name, RTT/stability colors and announced settings | 9e9 offline/connected/disconnected and two-remote-peer cycling passed in Eden |
| Ethernet / Wi-Fi | Local nifm type and peer announcement, otherwise Unknown | Never inferred from RTT; an emulator may report a virtual interface |
| Streamer / peer cycle | Persistent display privacy, menu and fork chord, identity-consistent peer selection | d9 saved INI/privacy passed; 9e9 cycled two remote peers in a three-station lobby with correct identity/settings. No three-station match completed |
| Global Player Info | Small ImGui window alongside native banners; right in Training and online lobbies, left in ordinary matches | Global window user-confirmed; a9c and 833c Training placement visually confirmed; b99 connected-lobby right placement and ordinary-match left placement passed |
| Native CSS / arena | Fork CSS pane path, room-ID arena hooks, saved-style Streamer restoration | CSS edits, closed-window display and match return verified in Eden; NSO arena and Switch testing pending. [Details](NATIVE_DISPLAY.md) |
| Embedded / standalone ImGui | Export/ABI detection, one active backend, namespaced hooks | Earlier load-order tests passed. The Oct 9 frame-ownership/GPU-storage correction passes embedded and patched standalone Eden checks; intermittent cause and Switch acceptance limits remain in [ImGui dependencies](IMGUI_DEPENDENCY.md) |
| Elite Legal Stages | Separate plugin, standalone ImGui retained | Both menus and settings persist; offline stages remain unrestricted as intended; NSO filtering untested |
| FX On / Off | Named visual requests only, live for newly requested effects | User Palutena/Sephiroth tests and TM save/load passed; final Gigaflare suppression observed; expanded catalog not fully runtime-certified |
| FX scope | Online-only default; all for offline+online tests | c0 Training Gigaflare retains full effects for On/Off/Reduced with online-only scope; df48 Local Wireless exit resets Offline and a targeted Roy effect is restored in subsequent regular VS. Direct NSO routes remain untested |
| Reduced FX | Live visual layer filtering and guarded lighter asset siblings; never silently mapped to Off | Visual request/cleanup checks cover most targeted families on recorded candidates. Corrected small-Flare/smash and full/dry Inkling down-smash requests passed; grounded triggered counter passed on c0; forward-smash trace2 remains open; Off-like frame-time benefit remains unverified. [Acceptance method](FX_PERFORMANCE_METHOD.md) |
| Software FPS boost | Implemented only as an explicitly gated diagnostic; normal menu reports Unsupported | Measured acceleration of the match timer in Eden, so it is withheld from normal play. [Investigation](FPS_BOOST_INVESTIGATION.md) |
| Present-at-flush | Two-image LessLag presents the acquired image at the flush that submits it and skips the game's later present of that index (0.4.0; `render_policy::present_at_flush`) | Console-verified. Counters in renderer-status.txt, mismatches in code 105. [Validation](VALIDATION.md) |
| Match capture | Opt-in `CAPTURE_MATCH` boot flag: streamed timing and API CSVs for a whole match, ended by `scene_end` (0.4.0; `src/runtime/match_capture.rs`) | Shipped in 0.4.0; results in [VALIDATION.md](VALIDATION.md). [Debugging](DEBUGGING.md) |
| Quickplay hold and stale demotion | Main-menu entry keeps a stored Quickplay; a Quickplay with no online-latency hook five seconds into a Singles/Doubles stage is demoted to Offline (0.4.2; `auto_policy::main_menu_connection`, `offline_match_due`) | Host-verified; see VALIDATION.md for console results |
| ZL-gated native edits | Native pane D-pad edits need ZL held (0.4.1; `src/overlay.rs`); added after 23 unintended saves on the console | Host-verified; see VALIDATION.md for console results |

## Current: 0.4.2 (ddea22d)

Releases v0.4.0 to v0.4.2 are on Forgejo. Switch installs and console results
are tracked in [VALIDATION.md](VALIDATION.md), whose latest entries cover
0.4.0 to 0.4.2.

## Historical candidates (October 8–10)

The candidates below were current on the dates shown and are kept as a record.

Latest local screenshot-review candidate: `8328cc88123bd432`, following operator
feedback on f1ce's accepted two-row layout. Compact Settings/Performance backgrounds
now fit their content while preserving the left edge, two inline rows, timing bar,
font guard and alpha0.42. Only runtime/ui.rs changes behavior versus f1ce; menu.rs
changes are comments.265hosttests, host/SkylineClippy, no-default, optimized release
and exact76-file package/source/native checks pass. Parent reviewed both actual1080p
ordinary-match views and exact startup/screenshot provenance under
`~/.local/share/ssbu-mod-backups/hud-content-fit-20261010/`. Eden exited normally0;
the fresh profile journal is RESTORED and all original bytes were independently
rehashed. The operator approved the narrower boxes;8328 was installed over foreground
DBI FTP on October10 at14:09Z after fresh Smash-closed confirmation. Staged and final
SHA readback passed. Latest INI, eight other plugins and five HOC/boot files stayed
byte-identical, plugin membership unchanged; old4c19 is backed up locally/on SD.
Receipt: hud-content-fit-20261010/switch-ftp/deployment.json, installed_verified;
FTP is closed. Hardware HUD/coexistence, peer departure/reconnect and CSV export
tests are next. No connected-peer/720p, performance or hardware acceptance is implied.

Previous Switch candidate `4c1947b84add6114` includes buffered export62a plus a
horizontal/translucent in-match Settings layout, internal-only Platform,
emulator-only FPS++ visibility/navigation, real content-fit height and retirement
of all aliases for a confirmed departing peer. Parent rejected speculative cleanup
for unknown RTT protocols.261hosttests, host/Skyline Clippy, no-default/release
builds and exact packaging pass. Installed October9 at22:32Z through DBI USB MTP
after the operator confirmed Smash closed; full SHA-256 readback passed. Previous5c72
is backed up locally and on the SD card. The latest LessLag/input3/FX Off/all/Stealth
Off INI, eight other plugins and protected HOC/boot files were preserved byte-for-byte.
Visual, declined-rematch and export runtime acceptance remain pending.
See [validation](VALIDATION.md) and the menu-peer-fix-20261009/switch-mtp receipts.

Previous Switch baseline: `5c72a0b54b8c8c95`, with validated far-stub hook chaining
and synchronous pre/post-hook diagnostics. All 253 host tests, host/Skyline
Clippy with warnings denied, no-default build and optimized packaging pass.
It was installed October 9 with exact SHA-256 readback; the latest INI, standalone
ImGui host, HOC/boot files and plugin membership were preserved. The operator
initially reported success, then another crash. Crash-fix acceptance is withdrawn.
Operator clarified successful L-bypassed vanilla gameplay followed by closing
and a failed normal modded relaunch; DBI was opened through Album. A subsequent
before/after capture now also includes a reported L-bypassed failure, successful
retry/play, and failed normal launch. The message is "Unable to start software.
Return to the HOME Menu and try again". Comparison02 temporarily paused SaltyNX
injection: both launches passed per operator, with fresh successful rewrite setup
and runtime evidence of skipped injection. Injection is now restored/readback
verified. Enabled rechallenge03 also passed both launches; its aftermath confirms
fresh expected rewrite startup. The paused/enabled comparison has not isolated
cause. Later bypassed launches failed three times per operator. Aftermath04 has
two new SDK UserBreak reports (result0, UserExceptionHandler, four stock modules)
after SaltyNX core injection completed; rewrite logs are unchanged. There is no
new pre-bootstrap timeout or new fatal/ERPT filename in that interval. The original
exception is unidentified; neither result0 nor injection completion assigns cause.
Exact SDK/report analysis proves the default user-exception callback deliberately
writes zero before a secondary abort. Original context pointers are present but
the pointed-to bytes fall outside every retained memory dump, so those reports
cannot identify the original fault. Fable's completed source audit established no
causal SaltyNX defect. Current Eden cannot reproduce its debugger/sysmodule injection
path. An existing Exosphere first-fault diagnostic was source-traced through Fusée,
but retained Hekate pkg3/secure-monitor-override settings leave the actual boot route
unverified. No diagnostic setting or custom reporter was installed. Details are in
the external hook-chain-retry-20261009/ORIGINAL_CONTEXT_OPTIONS.md evidence record.

The trial03 ten-second recording finished: callback CSV has602 samples and a
completion footer, while the API CSV declares6602 events but stops after6308 full
rows and a partial row. Source audit found unbuffered formatter fragments map to
flush-requesting Switch writes. Local candidate `62a42df8d01d1738` adds64KiB
buffered export and an explicit Exporting status. All257hosttests, host/Skyline
Clippy, no-default/release builds and exact packaging checks pass; it is not installed
or hardware-tested. All76archived source files matched the checkout at packaging;
the later menu candidate has its own source snapshot and gates.
The operator already tried repeated waits; do not request another uninstrumented
wait or treat this export defect as the intermittent launch cause.

The Kirby charged-hammer/Special Zoom dip also reproduces with game mods bypassed
(Kirby vs. Pac-Man, Battlefield). The reported SaltyNX counter is about50FPS without
OC and40-something with OC. Exact clocks/windows were not recorded, so neither
rewrite attribution nor a causal OC regression is established. Preserve zoom,
camera and hitlag; the earlier clean callback trace was not tied to that hit.

The previous 5c72 candidate's compact source-built renderer is unchanged from
`7fdad40f10705afb`, whose
embedded Training/CSS and patched standalone Elite/match/results checks passed
in Eden. See [renderer evidence](IMGUI_DEPENDENCY.md#renderer-correction-and-cleanup-2026-10-09).
Those runtime checks and older Switch/FX evidence remain tied to their recorded
binaries. The new candidate is still under crash investigation; broader menu,
FX, performance and online checks remain separate.

The operator reported a brief successful Switch test on October 9; detailed
screen/match/FX coverage was not specified. Earlier no-mod reopen reports abort
in `nn::apm::SetPerformanceConfiguration` with `2010-0011`, with only four stock
modules loaded. After removing `nst:oc` and background `sys-ftpd`, report
`01791552627_01006a800016e000.log` has a different failure: `2168-0002` Data Abort
in `__memcpy_device`, with Skyline/rewrite loaded and the matching startup log
ending at fighter-module unload-hook installation. Exact-binary and captured-stack
analysis confirms Skyline zero-extends a negative literal displacement, producing
the observed read address 4 GiB too high. The same build completed these hooks
on the previous successful launch. Re-hooking an existing jump stub is the likely
trigger; the failed launch's full pre-hook stub is unavailable. The installed
candidate now chains validated existing far stubs without relocating their
pointer payload and records full pre-hook words. An initially successful retry
was followed by another crash. Preserved logs prove earlier successful hook
installations; later SaltyNX launches lack matching rewrite logs. Operator sequence
correlates PID144 with vanilla bypass and PID146 with the failed modded relaunch,
which has a pre-bootstrap thread-list timeout. DBI title takeover is ruled out;
the timeout's cause remains unknown. The repeat adds PID149/153
timeouts around a successful injected PID151, while rewrite logs remain unchanged
and no new system report names appear. Do not assume modded-only failure or blame
the rewrite/SaltyNX from that symptom. Both the paused and restored-enabled pairs
passed, so injection state alone has not isolated it. Source audit finds FPS graphs sample/draw after startup;
the lower-level NVN hooks overlap, with a return-ABI question requiring separate
actual-chain verification. Neither is proven to cause the recorded early failure.
Evidence and rollback are in docs/VALIDATION.md and the operator's evidence root (the repository no longer carries a handoff file).

System maintenance on October 9 preserved HOC 2.5.1 and its settings, installed
DBI 912 English for foreground transfers, and installed official SaltyNX 2.1.2
for monitoring. All 15 SaltyNX release files passed FTP readback; five protected
HOC/boot hashes remained unchanged. The old SaltySD tree is archived. Firmware
22.5.0 is operator-confirmed and Atmosphere 1.11.2 was byte-identified. The operator
confirmed that SaltyNX is working and the duplicate listing is resolved after
the legacy 1.8.1 module backup was moved outside `atmosphere/contents`.

The Switch test candidate `b99f087cf8ef0a9c` passes 246 host tests,
host/Skyline Clippy with warnings denied, the no-default-features build and
optimized Skyline packaging. It corrects Inkling down-smash filtering to retain
the empty-ink splash and remove an extra full-ink trail. The generator matches
the checked-in policy. It also includes the 1f2d/7c small-Flare and up/down-smash
context fixes. The 7c small-Flare/up/down-smash visual and request checks passed. The b99 full/dry-ink down-smash visual/request checks and RUN/WALK/AIR sibling requests passed with the limits in the runtime matrix; The later c0 grounded triggered-counter check passed with the limits below; forward-smash trace2 remains open.

The 24e family captures cover most targeted families with On/Reduced/On,
recognizable cues, request traces and selected cleanup paths. See the
[per-family matrix](FX_RUNTIME_MATRIX.md) for unobserved branches and limitations.
Training placement is confirmed. The b99 paired run also verified right-side placement in connected CSS/stage selection, left-side placement in an ordinary match, and removal of the opponent column after disconnect.

The later c0 follow-up added 36 measured Palutena hit/whiff trials across
Vanilla/LessLag/LLUltra and On/Off/Reduced, plus 18 excluded warmups. It also
did not establish a consistent On-specific cost or Reduced benefit. The
hitbox CPU explanation remains unmeasured; see the bounded trace limits in
[the FX results](FX_EMULATOR_RESULTS.md).

Thirty measured Palutena windows and six Gigaflare pilots did not establish
a clear On-over-Off frame-time cost. Thirty later blue Special Zoom captures
on 7c, across LessLag/LLUltra and On/Off/Reduced, had no presentation interval
above 25 ms in their approximate impact windows. All modes retained the blue
zoom. These observations do not establish zero GPU cost or a Reduced performance
benefit; see [the measured results](FX_EMULATOR_RESULTS.md).

The 9e9 candidate verified that offline/disconnected views contain no opponent
column, while a connected station still displays its identity even before an
RTT sample arrives. A three-station local-network lobby reached ready/CSS;
cycling selected the correct two remote names, interfaces, profiles and buffers.
Native LDN D-pad editing changed Auto to 0. The bounded runs ended before a
three-station match began. This is lobby evidence, not a desync test.

Concurrent-run embedded-host checks reproduced an invisible Settings window
in LessLag and LLDoubles while Vanilla and LLUltra displayed it. The smaller
Performance window could remain visible. In the later df48 diagnostic,
disabling the LessLag field stores still left Doubles Settings invisible in two
captures: those stores are not necessary for the failure. Capacity-ended traces
contain nonzero game/menu recording handles but do not measure GPU completion
or prove command-memory corruption.

The dbdb diagnostic added a boot-gated, timeout-zero GPU-completion probe;
b111 extends the same probe to a recognized standalone ImGui host. The marker
is absent in normal use. b111 passes 253 host tests, host/Skyline Clippy with
warnings denied, the no-default build and optimized Skyline packaging.
The unchanged render-trace parser passed 11 tests on dbdb.

In b111's standalone CSS captures, the latest placed fence was still pending
in 420/431 and 425/431 polls during two invisible LLDoubles windows. In visible
LLUltra it was pending in 200/513 polls. Pausing the other Eden instance restored
the same LLDoubles window, with only 2/454 polls pending. Three captures ended
at event capacity (7.58–8.45 seconds); Ultra covered 10 seconds. These are
per-submit diagnostic observations, not displayed-frame counts or latency
measurements. They support investigating memory reuse under GPU backlog, but
do not prove which memory is at fault; pending work alone also occurred with
a visible window. Neither diagnostic is a menu fix or Switch candidate.
Their sync-object reuse assumption remains an explicit limitation.

Raw captures, screenshots, configuration and run receipts are preserved in
`~/.local/share/ssbu-mod-backups/menu-gpu-evidence-20261008/evidence.zip`
(SHA-256 `2429473a22353b5425467ae64d4ba5d31a7dae7810dfc5eee977f18a3cb49f5f`).
The paused capture completed before the other Eden instance stopped at its
host-memory floor; that instance reported SIGBUS during shutdown. No cgroup
OOM was recorded. Both diagnostic runs are now stopped and their nested
NRO/marker staging journals restored.

The b99f candidate remains under runtime acceptance. It was installed on the
Switch on October 8 after the user confirmed Smash was closed; FTP readback
matched its SHA-256. The existing INI and other plugins were preserved, and
the G&W plugin remains absent. A later FTP readback on October 8 again matched b99 exactly; no newer release
fix was available, so no replacement was necessary. Hardware testing is in progress.
The isolated four-idle-G&W check is complete on Intel/ANV, with smaller observed
dips when enabled. It did not reproduce the prior RTX slowdown or establish
per-frame 60 FPS; see [the exact results and limits](GNW_FOUR_FIGHTER_CHECK.md).
[Validation](VALIDATION.md) preserves build-specific receipts.

The previous candidate `d9acd03611e1d1d6` passed 176 host tests, host/Skyline checks with
warnings denied, the no-default-features build, and the optimized Skyline build.
The matching NRO, optimized ELF, source, hashes, raw checks and runtime evidence
are preserved outside Git. This candidate passed a G&W LessLag-versus-Auto/Ultra
match through results and CSS, with the fix on only one peer, Elite enabled,
standalone ImGui/TM/SGPO on A and embedded ImGui on B. Both d9 test sessions
exited normally on both peers, without memory-limit or OOM events.

The preceding b19 candidate passed 175 tests. Its renderer is unchanged in d9;
only four display/test files differ. The manual console-LLUltra crop bug is fixed:
an observed game fullscreen crop reset now rearms the smaller crop on the next
tracked present.

Twenty-four b19 response trials passed visual/marker checks; d9 was not remeasured.
Normal-phase response counts were Vanilla 6, LessLag 3, LLUltra 2 and LLDoubles 4
(three each); delayed-phase LLUltra varied 3,2,3 and other modes stayed the same.
See the methodology for the one idle-animation threshold correction, clock
boundaries and the distinction between manual LLDoubles Training and doubles.

The first four-player attempt stopped at character select when the guest's
second controller did not join. A later b19 test used three host controllers
and one guest controller and completed a real four-fighter team match, results
and return to CSS. Auto selected LLDoubles; the G&W fix was present on A only.
Both peers agreed on observed damage, stocks and results.



The later diagnostic-only candidate `c0e9a625eb44a735` passes 247 host tests,
both Clippy checks, the no-default build and optimized packaging. It records
LessLag flush-field values and bounded incoming command-list membership during
finite captures; it does not change the rendering policy or fix the invisible
window. A fresh single-instance LessLag run displays the window, so the earlier
concurrent-run failure remains a separate reproduction target.

A b99 two-station G&W-versus-Sephiroth Final Destination match completed through
results and back to stage selection. Both peers reported Sephiroth first,
two KOs and 47% damage dealt. LessLag and later LLUltra were active in separate
captured segments of that one match. Only A had the G&W fix; A used
Reduced/online-only, B used On/all with standalone ImGui and Elite enabled.
A's request logs show Gigaflare layer filtering during the linked match.
There was no observed disconnect or divergent result. This does not establish
full simulation equality, a three-station match, NSO safety or frame-time benefit.

## Compatibility boundaries

The corrected gamepad overlay passed the user's Kirby and Palutena Switch
matches and returns to character select. New profile/DRS changes remain
console-untested. Earlier asymmetric G&W-fix matches completed in LessLag and
LLUltra with matching visible results and return to CSS. These are bounded
observations, not simulation checksums or universal online-safety proof.

The rewrite installs no global Layout::Draw hook and does not patch SGPO's
`0x374812c` neighborhood. FX hooks chain original EffectModule requests and avoid
gameplay articles, hitboxes, cameras and RNG. Training Modpack's artificial input
delay is separate from Deluxe's online network buffer.

The guarded fighter-Main site must match the supported 13.0.5 code. Another
plugin already owning that site causes refusal; do not assume arbitrary
Smashline stacks are compatible. DRS reports unavailable triggers explicitly.

See [validation](VALIDATION.md) for exact tested artifacts,
[frame methodology](LATENCY_METHODOLOGY.md) for timing boundaries, and
[parity](FORK_PARITY.md) for deliberate differences from the fork.


The c0 grounded Sephiroth counter fixture used TMP Jab/Always on Donkey Kong.
On/Reduced/On preserved the visible shield reaction, counterattack, 20.2% /
four-hit outcome and clean expiry in reviewed first trials of each mode.
Reduced traces culled senkou_start and senkou_hold while keeping senkou_shield.
Success layers are passthrough and are not recorded by the filtering logger;
their visibility was reviewed directly. These are host-timed paused visual
checks, not frame-exact damage parity, aerial/facing coverage or performance
acceptance.
