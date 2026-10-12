# FX runtime acceptance matrix

Implementation coverage and runtime acceptance are separate. Reduced changes
specific visual layers, not every move a character can perform. All performance
benefits remain subject to the [measurement method](FX_PERFORMANCE_METHOD.md).
The tested artifact and scene matter; later builds are not automatically retested.

| Family | Reduced change | Visual evidence | Remaining check |
| --- | --- | --- | --- |
| Palutena Explosive Flame | Keep flame body/finish, remove surrounding layers | 833c On/Off/Reduced and expiry; 24e 2x On fixture | No clear performance gap in 1x repeated or 2x pilot trials |
| Palutena up-air | Lighter sibling retaining blue wing cue | 833c On/Reduced/On, cleanup | Switch performance |
| Palutena other targeted moves | Selected wand/backlight layers | 24e jab, neutral/up-B, tilts, three smashes On/Reduced/On and expiry; sampled cull/keep traces | Tilt trace08 and teleport_start not observed; Switch performance |
| Sephiroth neutral-B | Keep burst body, remove secondary body/screen wash | 833c full Gigaflare, 49.3%/8 hits; 24e Megaflare and 7c small Flare On/Reduced/On, expiry and cull/keep traces | Switch performance |
| Sephiroth side-B | Lighter charge/projectile/orb siblings | 833c compact orbs and Reduced-to-On cleanup | Switch performance |
| Sephiroth other targeted moves | Extra smash/dash/counter layers | 24e forward smash/dash/untriggered counter; 7c up/down-smash On/Reduced/On; c0 grounded triggered-counter On/Reduced/On retains reactive shield and attack, with clean expiry and cull traces | Down-smash retained names and counter hold_attack not observed; airborne/opposite-facing counter not separately checked; Switch performance |
| Inkling roller | Keep Splash emitter, remove extra emitters | 833c/24e run and cleanup; b99 RUN/WALK/AIR sibling requests observed, On/Reduced/On recognizable roller cue and expiry | Airborne scene not separately frame-matched; Switch performance |
| Inkling smashes | Keep splash body, remove selected secondary layers | b99 full-ink down-smash culls slosher_trace and retains splash; verified dry down-smash retains sole splash2; On/Reduced/On and expiry; c0 with shared26w hot-swap gives a recognizable forward-smash splash in On/Reduced, both one hit/23.8%, with clean expiry and splash keep trace | Forward-smash trace2 cull remains unobserved; Switch performance |
| Chrom first side-B | Trail lifetime 3 to 2 in guarded sibling | 833c recognizable trail and expiry | Other side-B branches remain original |
| Chrom neutral-B | Keep charge-level body, remove extra hold/spark | 24e short/full On/Reduced/On, clean expiry; full 12% self/63% target/one hit; cull/keep trace | Switch performance |
| Ike neutral-B | Keep volcano bodies, remove additional layers | 833c full charge, 10% self/39% target, cull/keep trace | Switch performance |
| Roy neutral-B | Keep eruption body, remove extra start layer | 24e short/full On/Reduced/On, clean expiry; full 12% self/45.3% target/one hit; cull/keep trace | Switch performance |
| Pyra sword | Keep sword/open cues, remove firetrail pair | 24e tilt On/Reduced/On and expiry, 14.4%/one hit; cull/keep trace; neutral-B body remains | Broad main arc remains, including sampled Off tilt; major specials remain original; Switch performance |
| Mythra sword | Keep sword/open cues, remove light layers | 24e tilt On/Reduced/On and expiry, cull/keep trace; neutral-B 41.5%/five hits all modes tested | light_few branch not observed; beams/buster/swap remain original; Switch performance |
| Byleth axe | Keep axe flare/slash, remove extra flare/hold | 24e On/Off/Reduced, On restoration and expiry; 37.8%/one hit vs DK; cull/keep trace | Switch performance; bow/spear remain original |
| Ganondorf | Keep punch/sword cues, remove flash/trace | 24e punch and charged sword On/Reduced, expiry, cull/keep trace | Charged damage comparison was not frame-matched; Switch performance |
| Captain Falcon | Keep punch body/hold, remove named wing auxiliary layers | 24e On/Reduced/On and expiry; cull/keep trace confirms both wing requests filtered | Main bird silhouette remains; Switch performance; Falcon Kick remains original |
| Ness | Keep steer/projectile and self-launch body, remove bomb layer | 24e On/Reduced/On actual PK Thunder self-hit, recognizable launch, 31.5%/one hit vs Hero; trace85 attack kept/84 bomb culled, clean expiry | Switch performance |
| Lucas | Keep steer/projectile and self-launch body, remove bomb layer | 24e actual PK Thunder self-hit On/Reduced/On and expiry; trace85 attack kept/84 bomb culled | Host-time paths not frame matched; Switch performance |
| Kazuya forward smash | Keep line/aura/start, remove electrical/sign flashes | 24e charged forward smash On/Reduced/On; trace5077871101534 keeps aura/start/line and culls electric/sign layers; clean expiry | Other smash profiles remain original; Switch performance |

Pausing, slow motion and screenshots verify cues and cleanup, not frame-time
benefits. A listed retained cue must remain identifiable during play, and a
successful menu change is not proof that the intended effect hook ran.

The 24e family captures and request traces are archived with a SHA-256 manifest at
`~/.local/share/ssbu-mod-backups/fx-families-24e-20261008/`. Concurrent-instance
captures are visual checks only; paused images do not establish frame-time gains.
