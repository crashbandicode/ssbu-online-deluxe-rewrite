# Resume and test plan

Read [STATUS.md](STATUS.md) for current implementation and
[VALIDATION.md](VALIDATION.md) for artifact-specific evidence. The cumulative
root [HANDOFF.md](../HANDOFF.md) records exact live processes, staging journals,
worker threads and next commands. Its latest entry takes precedence over older
operational notes.

## Environment and safety

Use the custom CLI Eden delivery and its handoff:

```text
/home/intpa/eden-cli-handoff/AGENT_HANDOFF.md
/home/intpa/eden-cli-handoff/source/docs/user/CLIControl.md
```

Always use an isolated profile for autonomous tests:

```powershell
/usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -Command '& /home/intpa/eden-cli-handoff/launch-eden.ps1 -IsolatedProfile -MemoryGiB 8 -Runtime 900'
```

Attach to the exact socket printed by that run. Two-console linked tests need
separate profiles and finite runtime/memory limits; the tested pair used 8/7 GiB
with zero swap. Latency measurements use one emulator. Do not run heavy builds
alongside games. Do not change the user's Switch or normal Eden profile.
Restore every staging journal after the corresponding frontends have exited.
Preserve the NRO, optimized symbols, source, plugin hashes, raw logs and captures.

## Native display follow-up

Current candidate `4a927dc20904722d` adds the fork's CSS and arena display path
to the existing global window. It passes 192 host tests and required builds.
CSS editing, hidden-window status, stable sizing, Streamer restoration and
linked peer text passed. Final solo runs showed standalone Deluxe and Elite
windows together, both with and without G&W. The arena path needs a Switch
NSO test.

One concurrent linked run's full-stack external windows were invisible while
the native header and the other peer's embedded window worked. Reproduce under
the same conditions, then compare the `DISABLE_NATIVE` marker on/off with the
same binary and plugin hashes. Do not blame G&W or native text from the
existing evidence. See [NATIVE_DISPLAY.md](NATIVE_DISPLAY.md) for exact runs,
including bounded frontend-stop failures.

FPS++ is now traced to native-window swap interval / emulator composition
scheduling; the older simulation-cap hypothesis is superseded. It remains
unimplemented and unmeasured. Read [FPS_BOOST_INVESTIGATION.md](FPS_BOOST_INVESTIGATION.md)
before planning any controlled implementation or A/B experiment. Hardware
overclocking remains excluded.

Current matching bundle:

```text
/home/intpa/ssbu-online-deluxe-rewrite/dist/4a927dc20904722d/
```

## Previous emulator milestone

Final candidate `d9acd03611e1d1d6` passes 176 host tests and the required
checks/build. Fresh Stealth On suppression, Off after reconnect, exact INI
reload, Streamer privacy, peer names/buffers/profiles/colored RTT, standalone
and embedded ImGui, enabled Elite, and a one-sided G&W LessLag match through
results/CSS passed. Auto selected Ultra for linked singles and Vanilla on cleanup.

Preceding candidate `b19b7d94ae136e55` supplies the 24 response trials,
four-fighter linked Auto/LLDoubles match, G&W Ultra match, cut-in/Gigaflare
scaling and corrected title crop test. D9 changes four display/test files only;
do not relabel b19's timing measurements as a new d9 experiment.

Preserved candidate and symbols:

```text
/home/intpa/ssbu-online-deluxe-rewrite/dist/d9acd03611e1d1d6/
/home/intpa/ssbu-online-deluxe-rewrite/dist/b19b7d94ae136e55/
```

All test frontends are closed and the staging journals restored. The user's
Switch and normal Eden profile were not changed during this final test cycle.
Use the current NRO and matching ELF for the next console test, retaining backups.

High-RTT/VPN matches and more than two remote network stations remain additional
coverage. Two local controllers on each of two consoles do not produce four
network peers. Do not describe localhost or a delay relay as an actual VPN test.

## Switch follow-up

The user accepted two corrected-overlay matches (Kirby and Palutena), including
return from results to CSS. There is no special requirement for five runs.
New Auto/DRS/render transitions are not yet console-tested. Use the existing
backups and the exact tested candidate when the user is ready to install.
Capture Atmosphère reports and matching optimized symbols for any recurrence.
A separate Eden unbound GPU-subchannel crash is unresolved; do not attribute
it to the rewrite or G&W without evidence.

## Reduced FX

Reduced remains unavailable. First establish the exact Switch scene and effect
that causes a repeatable dip, then reproduce comparable load in Eden. Record
mode, resolution, console/host settings, stage, fighters/costumes, damage,
camera, move/charge and shader warmup. A forced FPS cap is not a reproduction.
Implement one reduced effect while preserving charge/trajectory/hit cues, test
On/Off/Reduced, and return to Switch before iterating the catalog.

Full Gigaflare needs approximately a 4.5-second B hold in the current tests.
Keep FX On and rendering mode fixed when isolating effect cost. Avoid the peer
mod's broad generic deletion, article scaling, global timers or color-reset
suppression. Test Training save/load, normal match cleanup and online-only FX
scope separately. Elite Legal Stages should restrict NSO only, never offline.

No hardware overclocking is planned. FPS++ needs a separately reviewed implementation and controlled measurements.
