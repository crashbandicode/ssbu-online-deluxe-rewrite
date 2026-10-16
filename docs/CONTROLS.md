# Controls and settings

Hold **L+R** or **ZL+ZR**, then tap **D-pad Down** to cycle
**settings → performance → hidden**. When a separately installed plugin NRO has
registered options, a **plugins** page sits between settings and performance,
and a plugin with its own page (since 0.5.0) follows it (see [Plugins](PLUGINS.md)).
With the separate [FX Control](https://git.syntaxbyexample.com/intpa/ssbu-fx-control)
plugin installed, its **FX Control** page appears in that cycle; character FX
is no longer a Deluxe row. The small Player Info window works in menus,
character select and matches. It moves to the right in Training and connected
lobbies, keeping the pause controls visible, and stays left in regular matches.

Character select also keeps the fork-style native status at the top after the
window closes. It shows your selected profile and input buffer, plus the selected
opponent's available settings and colored RTT. Edit values in the window;
Streamer restores the game's original title and background. Arena text uses the
fork's room-ID path, but still needs an NSO test. See [native display](NATIVE_DISPLAY.md).

| Control | Action |
| --- | --- |
| L+R (or ZL+ZR) + Down | Cycle settings → (plugins) → (own plugin pages) → performance → hidden |
| Up / Down | Select Peer, NetLatency, NetProfile, Stealth, Streamer, FPS++ (emulator) or UltraRes (console, 0.5.3: LLUltra's internal resolution, 576p or 720p) (or a plugin row on a plugin page) |
| Left / Right | Change the selected value (in menus and on the character select) |
| ZR + Left / Right | Change the selected value while a fight is live (Training, Singles, Doubles; since 0.5.3). A plain Left/Right is ignored there, so a taunt or a stray press cannot change a setting mid-game; Up/Down still move the selection. ZR rather than ZL because ZL + D-pad Left/Right in Training swaps the controlled fighter |
| Shoulder chord + Left / Right outside settings | Cycle the selected remote peer |
| ZL + Left / Right or Up / Down on an online character select or arena pane, window closed | Edit the input buffer or the profile on the native status (since 0.4.1 the D-pad alone only moves the hand) |
| L+R + either ZL or ZR + Y | Toggle Streamer |
| L+R (or ZL+ZR) + Plus or Minus | Open the debug view; again to hide it |
| L+R + Up in the Debug view | Start a 30 s timing capture (see [Captures](CAPTURES.md)) |
| L+R + Left in the Debug view | Coverage build only: write a `manual-NNN.profraw` snapshot (see [Coverage build](DEBUGGING.md#coverage-build-052)) |

The window commands fire when the whole combination is held, in any press
order, and once per press: L+R+Down works whether the bumpers or the D-pad
land first. Holding Plus or Minus while tapping Down with the chord opens Debug
from Settings or Plugins and keeps Debug open; from hidden the chord still
opens Settings and from Performance it still hides the window.

The first controller operates the window. Input also reaches the game. If
Elite's menu is open too, D-pad edits can affect both menus; close one before
changing settings. With the current Eden keyboard mapping, **Q+E+Down Arrow**
is the window chord.

## Rows

**NetLatency** is the game's online input buffer, **Auto or 0–25**. It is
separate from NetProfile's rendering policy. A menu change takes effect at once
and is written to `config.ini` by the background thread within about half a
second; the game's next relevant buffer hook applies the selected value.

**NetProfile** selects the rendering profile or Auto; see
[Rendering profiles](RENDERING.md). A manual selection that the post-match
cleanup hold is running as Vanilla reads `Vanilla*`.

**Stealth** defaults to **On** and suppresses this mod's recognition beacon and
custom settings payload. Its value is latched for the connection. A change
during a connection shows a reconnect notice and takes effect on the next one.
It does not promise anonymity or detection avoidance by Nintendo.

**Streamer** hides remote names, connection types, settings and ping; it does
not change network traffic. Peer RTT uses the game's existing measurement and
the fork's stability colors. The opponent column appears only for a connected,
identified station; a connected peer with unknown RTT still appears with `---`.
Interface shows local Wi-Fi/Ethernet/Unknown and the selected peer's announced
type, if available. An emulator's reported interface can be virtual; it does
not identify the host computer's physical network connection.

**FPS++** is a diagnostic experiment and reports **Unsupported** in the normal
build; see [Rendering profiles](RENDERING.md#fps).
