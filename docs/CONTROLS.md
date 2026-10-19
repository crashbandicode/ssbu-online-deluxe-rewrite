# Controls and settings

Hold **L+R**, then tap **D-pad Down** to cycle
**settings → performance → window → hidden**. When a separately installed plugin NRO has
registered options, a **plugins** page sits between settings and performance,
and a plugin with its own page (since 0.5.0) follows it (see [Plugins](PLUGINS.md)).
With the separate [FX Control](https://git.syntaxbyexample.com/intpa/ssbu-fx-control)
plugin installed, its **FX Control** page appears in that cycle; character FX
is no longer a Deluxe row. The **Window** page (0.6.0) is the last page of the
cycle and is always in it; it holds the page toggles and the other settings of
the window itself. The small Player Info window works in menus,
character select and matches. It moves to the right in Training and connected
lobbies, keeping the pause controls visible, and stays left in regular matches.

Character select also keeps the fork-style native status at the top after the
window closes. It shows your selected profile and input buffer, plus the selected
opponent's available settings and colored RTT. Edit values in the window;
Streamer restores the game's original title and background. Arena text uses the
fork's room-ID path, but still needs an NSO test. See [native display](NATIVE_DISPLAY.md).

| Control | Action |
| --- | --- |
| L+R + Down | Cycle settings → (plugins) → (own plugin pages) → performance → window → hidden. The Window page rows PluginsPage, OwnPages and PerfPage (or `hide_pages` in the INI, 0.5.3; the rows moved from Settings to the Window page in 0.6.0) take pages out of the cycle, e.g. settings → FX Control → window → hidden; Settings and Window are always in it. See [Configuration](CONFIGURATION.md) |
| Up / Down | Move the selection; it wraps within the page. **Settings** (0.6.0): the Opp line (only while the peer shows), NetLatency, NetProfile, Stealth, Streamer, UltraRes (console only, 0.5.3: LLUltra's internal resolution, 576p, 720p or, since 0.5.6, 1080p; Right walks 576p → 720p → 1080p → 576p, Left walks back); the You line is read-only. **Window** (0.6.0): PluginsPage / OwnPages / PerfPage (Shown or Hidden), PerfView (Graph or Number, 0.5.3) and Hints (Shown or Hidden). On a plugin page, the plugin's rows |
| Left / Right (window open) | Does not change a value since 0.5.6; the window ignores a plain press in every scene (until 0.5.5 it changed values in menus and on the character select). Use ZR + Left / Right below |
| ZR + Left / Right | Change the selected value on any Settings row, Window row or plugin row; on the Opp line (Settings) it cycles the selected remote station, what the Peer row did before 0.6.0. The hold is required in **every scene since 0.5.6** (menus, character select, Training and matches); in 0.5.3–0.5.5 it applied only while a fight was live (Training, Singles, Doubles). A plain Left/Right is ignored, so a menu D-pad press, a taunt or a stray press cannot change a setting (operator rule, 2026-10-18); Up/Down still move the selection. Since 0.6.0 the selected row's value reads `< value >` (for example `< Auto >`) while ZR is held and is plain text without it, on Settings, Window and in the match strip. ZR rather than ZL because ZL + D-pad Left/Right in Training swaps the controlled fighter. Window closed, the ZL-gated native pane edits below are unchanged |
| Shoulder chord + Left / Right outside Settings and Window | Cycle the selected remote peer |
| ZL + Left / Right or Up / Down on an online character select or arena pane, window closed | Edit the input buffer or the profile on the native status (since 0.4.1 the D-pad alone only moves the hand) |
| L+R + ZR + Y | Toggle Streamer. ZR is the same hold every window edit needs; since 0.5.7 the ZL variant does nothing (it flipped Streamer on unnoticed during a console session, and ZL is Training's controlled-fighter swap modifier) |
| L+R + X (neither ZL nor ZR held) | Toggle Vanilla+ (0.5.6): the Vanilla profile presents at the flush when on (the default). Applies live, saves `vanilla_plus` in the INI (event code 24) and flips the NetProfile row text between `Vanilla+` and `Vanilla`. See [Rendering profiles](RENDERING.md#vanilla-056) |
| L+R + Plus or Minus | Open the debug view; again to hide it. ZL+ZR stopped being a window chord in 0.5.3, so a trigger chord during play cannot open the window |
| L+R + Up in the Debug view | Start a 30 s timing capture (see [Captures](CAPTURES.md)) |
| L+R + Left in the Debug view | Coverage build only: write a `manual-NNN.profraw` snapshot (see [Coverage build](DEBUGGING.md#coverage-build-052)) |

The window commands fire when the whole combination is held, in any press
order, and once per press: L+R+Down works whether the bumpers or the D-pad
land first. Holding Plus or Minus while tapping Down with the chord opens Debug
from Settings, Plugins, an own plugin page or Window and keeps Debug open; a
plain chord then continues from Debug to Performance (to Window when
Performance is hidden). From hidden the chord still opens Settings, and from
Window (the last page, since 0.6.0) it hides the window.

The first controller operates the window. Input also reaches the game. If
Elite's menu is open too, D-pad edits can affect both menus; close one before
changing settings. On Eden the chord is the two keys mapped to L and R plus
Down Arrow (the ZL/ZR keys no longer work since 0.5.3).

## Settings page (0.6.0)

Until 0.5.7 the first page was one table of up to 14 rows in three columns
(label, You, Opponent) with the 11-row Performance block and the frame-time plot
under it. Since 0.6.0 it has two columns (a label and your value, no opponent
column) and reads top to bottom as follows. The window title is still Player
Info.

**You** is a read-only summary line, `<profile> | <buffer> | <interface>`, for
example `Vanilla+ | Auto | Wi-Fi`. The ` | ` separator is the one the native
character-select banner already uses (ASCII on purpose). Interface shows the
local Wi-Fi/Ethernet/Unknown type; an emulator's reported interface can be
virtual and does not identify the host computer's physical network connection.

**Opp** appears only while the peer shows (Streamer off, and a connected, live
station): `<name> | <announced profile> | <announced buffer> | <ping> |
<interface>`, for example `Mario | LessLag | 3 | 23ms | Ethernet`. A fact the
peer has not announced, or an RTT that is not known yet, is left out of the line
(no `---` placeholders), and the interface is the peer's announced type, if
available. The whole line is painted in the ping's stability colour (the fork's
green, yellow or red). It is a selectable line: with ZR held, Left/Right cycles
the selected remote station, which is what the Peer row did. It replaces the
Name, Interface, Peer and Ping rows and the opponent column.

Under them the table has up to five rows: **NetLatency**, **NetProfile**,
**Stealth**, **Streamer** and **UltraRes** (console only). The old Status row is
folded into NetProfile's value, and the PluginsPage, OwnPages, PerfPage and
PerfView rows moved to the Window page.

A one-line performance footer sits under the table in the frame-time colour:
`60.0 fps | 16.67 ms | 1080p | DRS on` (fps, mean frame time, the applied
resolution, DRS state), or `--- fps | --- ms | ...` until a frame time exists.
The full Performance block and the frame-time plot are on the Performance page
(and in Debug) only.

A dim grey hint line under the footer, wrapping to the window width, lists the
chords: `ZR+Left/Right edits   L+R+Down next page   L+R+X Vanilla+   L+R+ZR+Y
Streamer`. It is on by default; `hints=false` in the INI or the Window page's
**Hints** row (Shown / Hidden) turns it off ([Configuration](CONFIGURATION.md)).

## Window page (0.6.0)

The last page of the L+R+Down cycle, titled **Window** and always in the cycle.
Its rows are **PluginsPage**, **OwnPages** and **PerfPage** (Shown / Hidden;
they toggle `hide_pages` live, as the same rows did on Settings before 0.6.0),
**PerfView** (Graph / Number; since 0.6.0 it affects the Performance and Debug
pages only, because Settings has the one-line footer) and **Hints** (Shown /
Hidden, the `hints` key). The same hint line is under the table (no performance
footer). ZR + Left/Right edits, Up/Down move the selection and it wraps within
the page.

## In-match strip (0.6.0)

In a ready Singles or Doubles match the window is the compact two-line strip.
Line 1 of the Settings strip is `Net`, `Profile`, `Stealth`, `Stream` and
`UltraRes`; the `PlugPg`, `OwnPg`, `PerfPg` and `PerfView` fields moved to the
Window page's strip, which shows them plus `Hints`. Line 2 is
`>Opp:<name> | <profile> | <buffer> | <ping> | <interface>` (when the peer
shows) followed by the timing line `FPS:.. FT:.. AVG:.. [####------]`. The old
`Peer:`, `IF:`, `Ping:`, `PeerNet:` and `PeerProfile:` fields are gone. The
`< value >` marker applies to the strip's selected field too.

## Rows

**NetLatency** is the game's online input buffer, **Auto or 0–25**. It is
separate from NetProfile's rendering policy. A menu change takes effect at once
and is written to `config.ini` by the background thread within about half a
second; the game's next relevant buffer hook applies the selected value.

**NetProfile** selects the rendering profile or Auto; see
[Rendering profiles](RENDERING.md). Since 0.6.0 the value carries the status
the Status row used to show: `LessLag | Swap`, `Auto:LLDbl | Active`; an empty
status, or the uninformative `Vanilla` status, shows the profile alone. Since
0.5.6 the Vanilla entry reads `Vanilla+` while Vanilla+ is on (the default; `Auto:Van+` under Auto), and plain
`Vanilla` (`Auto:Van`) when it is off; the in-match strip's `Profile:` field
and the native CSS banner's `You:` field use the same text. There is no separate
Vanilla+ entry in the NetProfile cycle: L+R+X flips the flag. A manual
selection that the post-match cleanup hold is running as Vanilla reads
`Vanilla+*` (`Vanilla*` with the flag off).

**Stealth** defaults to **On** and suppresses this mod's recognition beacon and
custom settings payload. Its value is latched for the connection. A change
during a connection shows a reconnect notice and takes effect on the next one.
It does not promise anonymity or detection avoidance by Nintendo.

**Streamer** hides remote names, connection types, settings and ping; it does
not change network traffic. Peer RTT uses the game's existing measurement and
the fork's stability colors. The Opp line (before 0.6.0 the opponent column)
appears only for a connected, identified station; a connected peer with unknown
RTT still appears, with the unknown facts left out of the line.
Since 0.5.5 the station must also be live, meaning a measured RTT sample within
the last 10 s (a new binding gets a 10 s grace); a bound station that goes
silent, as after an opponent declines a rematch, is hidden (the Opp line, the
match strip's `>Opp:` line and the CSS line included) and logs event code 112.
Console behaviour of this rule is untested. With Streamer On the match strip's
`Stream` field already reads On, so there is no separate `Peer:Hidden` field
since 0.6.0.

The fork's **FPS++** row was removed on 2026-10-17, and its L+R+ZL/ZR+X chord
with it. Since 0.5.6 plain **L+R+X** (neither ZL nor ZR held) toggles Vanilla+
instead; the trigger variants L+R+ZL+X and L+R+ZR+X still do nothing. See [Rendering profiles](RENDERING.md#fps)
and [Vanilla+](RENDERING.md#vanilla-056).
