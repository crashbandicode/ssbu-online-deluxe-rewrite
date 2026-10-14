# FX expansion review disposition — 2026-10-05

Fable 5.1 xhigh independently checked the current expansion against the supplied
May-14 source and skyline-smash bed642e. This is source review, not runtime evidence.

| Finding | Primary resolution |
|---|---|
| Peer scale tables include projectile/position, reflector/counter and hitbox cues | Exclude 41 exact names from global and scoped Off; regression test and JSON ledger |
| Native effect installation dereferenced target before checking mapping | QueryMemory guard and HookRejected; skip that hook |
| Flip commands have two effect names before joint | Independently filter slots 1/2, retain joint/rest; host regression |
| Lua Int/Hash type normalization unnecessary | Preserve incoming kind and metadata |
| (2026-10-05; superseded by the 0.4.2 Quickplay hold) Quickplay can temporarily pass main-menu init | Clear there; re-arm from the already-known online delay-setup site |
| DISABLE_BUFFER must not disable the FX online marker | Shared hook installed once for either purpose; separate buffer-write flag |
| Tracker unavailable could look like a broken toggle | Visible status plus diagnostic Compatibility record |

The review verified primitive-stack bounds, 24-byte return ABI, actor category/kind,
motion/status getters, constants and weapon qualification. No owner traversal or
global victim counters are used. It found no accidental omissions in the promoted
peer groups apart from explicitly deferred generic sys_attack_arc_d.

Off intentionally removes some named layers that the peer only shrank; cue exclusions
take priority. This is documented policy, not exact peer parity. Reduced is pending.
The full review is retained locally with the audit artifacts; no peer code is added
as a runtime dependency by this review.

Online context has source evidence but needs the actual transition matrix. The old c1706bbe runtime acceptance does not validate new hooks.
