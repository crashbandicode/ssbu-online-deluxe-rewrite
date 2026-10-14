# Character effects

FX On forwards original requests. FX Off suppresses the named character effects
below when the configured scope allows it. Reduced retains selected move cues and
removes surrounding decoration. Its goal is to approach Off's spike stability in
LessLag and LLUltra; a visible change alone does not establish that benefit.

## INI scope

In `sd:/ultimate/ssbu_online_deluxe_rewrite/config.ini`:

```ini
version=1
effects=on
effects_scope=online
input_buffer=auto
```

`effects_scope=online` is the default, including for existing files without this key.
`effects_scope=all` enables the same policy offline and online. This restriction
applies to both Off and Reduced. FX On always preserves visuals.
INI edits take effect on game restart. Menu saves retain the scope; debug view shows it.

The ready-to-copy [Eden test config](../config/eden-test.ini) sets scope to all.
Install or merge it while Eden is closed. Preserve other existing INI settings;
restore the default online-only scope after offline testing.

Online context uses the preserved 13.0.5 fork's Quickplay, background matchmaking,
arena and local-wireless scene entries. Main-menu entry resets the connection to
Offline unless Quickplay is live, because background matchmaking passes through
the main menu. Since 0.4.2 a Quickplay with no online-latency hook five seconds
into a Singles/Doubles stage is demoted to Offline
(`auto_policy::main_menu_connection`, `offline_match_due`). `fx_online()` also
counts the latency-hook mark, so a stage that hook reached is online for FX.
Offline Training is excluded. The online input-delay setup hook positively refreshes
match context after transient matchmaking menus, independently of buffer editing.
All scene and match-marker preflights must succeed or online-only
filtering stays inactive and the window reports tracking unavailable. This does not measure Internet connectivity; live
Quickplay/arena/wireless and return-to-offline validation is pending.

## Coverage

The expanded candidate includes 314 distinct names from the peer's unconditional
legacy tables, three verified Inkling roller splash parents, and 19 qualified move groups. These overlap the initial 123-name
policy. Aliases and emitter-child names are source candidates, not proof that
every name reaches a request hook.

| Character | Named effects targeted by Off |
|---|---|
| Pyra / Mythra | Peer legacy sword, charge, slash and special-move layers |
| Inkling / Ganondorf / Byleth | Peer legacy ink, sword, axe and charge layers |
| Captain Falcon | Falcon Punch decorations |
| Ness / Lucas | PK Thunder hold, self-launch aura/burst and peer named layers, qualified by actor/move or PK Thunder weapon |
| Ike / Chrom / Roy | Neutral-B charge, eruption/volcano and surrounding named layers |
| Kazuya | Forward-smash visual layers, qualified by move |
| Palutena | Rapid jab, neutral-B staff glow, teleport, Explosive Flame, tilts, wing smashes, up-smash and up-air glow |
| Sephiroth | Neutral-B burst/smoke/rings, Shadow Flare, counter star, dash attack and forward smash |

The code uses ordinary EffectModule request wrappers plus ten ACMD creation variants:
EFFECT, EFFECT_FOLLOW, EFFECT_FOLLOW_ALPHA, EFFECT_FOLLOW_NO_STOP, EFFECT_FLW_POS,
EFFECT_FLW_POS_NO_STOP, LANDING_EFFECT, EFFECT_FOLLOW_FLIP, EFFECT_FOLLOW_FLIP_ALPHA,
and EFFECT_FOLLOW_NO_STOP_FLIP. Flip variants independently filter their first two
effect names; joint, transform, scale and other arguments remain intact.

On is immediate passthrough. Off substitutes the named null resource and calls the
original so downstream bookkeeping still runs. Only edge_fire3_screen2's screen
request is skipped. Newly created effects follow the setting during the same match;
previously created effects expire naturally. There are no per-frame deletion sweeps.

## Differences from the peer

Off removes included layers even where the peer merely shrank them. Reduced
retains selected cues and lighter layers. The policy is not an exact copy of the peer.

Shared effect names require the correct fighter/weapon category, kind and move.
There is no owner traversal or global victim timer. Forty-one named projectile/position, reflector/counter and hitbox cues are preserved,
including Palutena's up-air marker. They are listed under excluded_cues in the policy. General hit/shield/KO feedback remains outside the filter.
Article/model/posture changes, camera/LOD changes, color/flash/afterimage command
hooks and unidentified raw hashes are not ported. Generic sys_attack_arc_d and unobserved creation variants (NO_SCALE, COLOR,
UNSYNC_VIS, RND, VARIATION) are also outside current coverage. These omissions are explicit:
in particular, a separate article or scripted tint can remain visible in Off.

[Initial targets](../reference/fx-policy.json) and
[expanded targets and qualifications](../reference/fx-expanded-policy.json) record
the policy; raw source presence alone does not establish runtime coverage.

## Evidence and provenance

User archive: effect disable-reduce source.zip, May 14 master/screen sources.
The source SHA and per-profile source groups are in the expanded policy.
Vanilla dumped scripts independently identify the actual PK Thunder names:
[Ness](https://github.com/WuBoytH/SSBU-Dumped-Scripts/blob/59004685238a3f6f93bae905d3a4e079701c40b4/smashline/lua2cpp_ness/ness/SpecialAirHi.txt) and
[Lucas](https://github.com/WuBoytH/SSBU-Dumped-Scripts/blob/59004685238a3f6f93bae905d3a4e079701c40b4/smashline/lua2cpp_lucas/lucas/SpecialAirHi.txt).

Palutena/Sephiroth creation names were corroborated with vanilla scripts and local
EFF resource names extracted read-only from data.arc. Toolbox-Cli's Rust commands
handle layouts/BNTX/SARC, so smash-arc 0.6.0 was used to read those EFF containers.
No Nintendo asset or compiled peer plugin is included.

The bounded primitive Lua bridge follows skyline-smash bed642e: L2CAgent 0x48 bytes,
24-byte L2CValue return forces AArch64 x8, with unused MaybeUninit padding.
At most 32 primitive arguments are retained; unknown/oversized commands pass through.

On installed c1706bbe9589de97, the user confirmed Palutena/Sephiroth Off/On, the
global window, SGPO, Training Modpack menu and save/load coexistence. Logs corroborate
Palutena bomb and Sephiroth Shadow Flare charge/beam and Gigaflare interceptions.
On final bundle 5d47c70452ce73ba, the custom CLI Eden build also verified live
Sephiroth neutral-B suppression with scope all in Vanilla and LessLag core.
The same named ACMD interceptions appear in both Off traces. With effects=off
and effects_scope=online, offline Training retained its visuals; capture
940587468188 reported effective filtering false throughout, and no suppression
events occurred within that capture. The online tracker installed successfully.
Ordinary offline Smash, live online transitions and the rest of the expanded
move catalog still need runtime tests.
There is no console/performance acceptance yet.

## Reduced implementation and acceptance

The request policy keeps identified move bodies, projectiles and other cues while
suppressing selected surrounding layers. Palutena's up-air resource has a separate
lighter copy so its visible cue remains; the stock resource is retained for On.
The shipped Off policy deliberately protects that up-air parent, so Off is not
a zero-cost reference for that particular parent.

Candidate `833cc40bb7db2721` extends resource copies to Shadow Flare charge/projectile
layers, Inkling roller splashes and Chrom's first side-B trail. Its visual checks
confirm recognizable cues and normal expiry, including Shadow Flare and roller
cleanup after switching back to On. These changes are not part of the existing
Switch prerelease. A failed resource callback leaves that cue original. Stock
game assets are read at runtime and are not distributed in the NRO or repository.

The roller runtime check exposed an Off-table alias gap: the peer names
`inkling_roller`, while the game requests `inkling_roller_splash_walk`,
`inkling_roller_splash_run` and `inkling_roller_splash_air`. Candidate `24e714e77b3eabf0` adds those three unique parents to Off.
Its ground-roller test confirms full splash On, splash absent Off and a smaller
splash Reduced, with the same observed 12.1%/one hit against Bowser Jr. Switching
Reduced to On during the roller and cancelling leaves no persistent splash. Ground ink and gameplay articles remain
outside the filter. Runtime acceptance of that correction is pending.

The mode applies to new requests during the match. Changing it does not erase an
already active effect; normal script cleanup and finite emitter lifetimes still
apply. A replacement resource must also respond to the original's stop/detach
commands, including after Reduced is switched back to On.

Follow the [performance method](FX_PERFORMANCE_METHOD.md): reproduce the dip,
compare matched On/Off/Reduced trials under both LessLag and LLUltra, and then
verify on Switch. Selective layers, fewer particles or shorter lifetimes can
preserve recognizable feedback. Smaller scale alone may reduce overdraw while
leaving CPU/update costs unchanged. See [validation](VALIDATION.md) for exact
builds and observations; implementation and visual checks are not a demonstrated
performance improvement.

Move-level runtime checks and known unchanged paths are tracked in the
[FX runtime matrix](FX_RUNTIME_MATRIX.md).
