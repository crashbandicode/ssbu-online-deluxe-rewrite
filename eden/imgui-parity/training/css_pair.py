"""Closed-loop character select for the parity Training route: P1 Game & Watch, the CPU Ganondorf, every step
checked before the next one (2026-11-13).

Why the old route failed (hang.py `_pick_pair`, imp-a1/a4, imq-fc1, imp-fc1, imp-yfc1): two steps acted without
checking their result.
- The CPU coin was reached with a recorded stick push (`lr04-cpu-approach`, 26 frames right, 14 down) from wherever
  the empty hand happened to be after P1's token was placed. The hand glides after a drop (imp-fc1's frame shows it
  still moving), and the push's distance in frames is not the same distance on every boot, so A sometimes landed
  beside the coin (imp-a4, imq-fc1: no `token=p2` step after the pick).
- A single 150 ms A press dropped the coin on Ganondorf; at 4-12 presents/s that press can fall between two polls.
  The drop probe then stepped right from tile 25, the end of its row, where a step never moves (`to=none`), so a
  coin still in the hand passed as dropped. The B that followed then found the hand still holding the CPU coin
  (imp-fc1: `token_after_b p2`, the FAIL frame shows the coin in the hand over Young Link).

This route:
- places P1's token on Game & Watch first (walked with the tile cursor), so no B and no Steve detour is needed;
- moves the empty hand to the CPU coin by steering: a frame, the hand found by its "P1" tag (handfind.py), a short
  stick pulse toward the coin, repeated until the fingertip is on the coin, the pulse length learned from how far the
  last one moved the hand;
- after every A, checks the result with one tile-cursor step in a direction that has a neighbour: a `token=p2` step
  after the pick, no step after a drop (a step means the token is still in the hand: step back, press A again).
"""
import math
import re
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import handfind  # noqa: E402

GW_TILE, GANON_TILE = 29, 25
# The CPU coin starts on the Random tile (tile 86, `from=86 ... token=p2` in every pick so far); its centre in a
# 1280-wide frame (imp-a3/a4/fc1 hang-css-steve frames).
COIN = (998, 385)
TOKEN = re.compile(r'step screen=css .*token=(\w+)')


def _press(ctx, button, ms=250, gap=1.0):
    # Held long enough to span several polls at the slowest CSS rate seen (about 4 presents/s on the Surface).
    ctx.call('input.button', {'button': button, 'state': 'press', 'lease_ms': ms + 500})
    time.sleep(ms / 1000)
    ctx.call('input.button', {'button': button, 'state': 'release'})
    ctx.call('input.reset')
    time.sleep(gap)


def _probe(ctx, hs, direction):
    """One tile-cursor step: the token the hand holds (`p1`, `p2`) or None for an empty hand."""
    path = Path(ctx.instance['tile_log'])
    _, step, _, lines = hs._tile_step(ctx, direction, path.stat().st_size)
    if step is None:
        return None, lines[-3:] if lines else []
    token = next((m.group(1) for m in (TOKEN.search(l) for l in lines or []) if m), '?')
    return token, step.group(0)


def place(ctx, hs, tile, probe, back, trail, key, tries=4):
    """A on `tile` until the hand is empty: the probe step `probe` moves nothing. A probe that moved the token is
    undone with a step `back` before the next A."""
    attempts = []
    for _ in range(tries):
        ctx.check_stop()
        _press(ctx, 'a')
        token, seen = _probe(ctx, hs, probe)
        attempts.append({'token_after_a': token, 'probe': seen})
        if token is None:
            trail[key] = attempts
            return
        back_token, back_seen = _probe(ctx, hs, back)
        attempts[-1]['back'] = back_seen
        if back_token is None:
            raise RuntimeError('%s: the step back after a failed drop moved nothing: %s' % (key, attempts))
    trail[key] = attempts
    raise RuntimeError('%s: the token stayed in the hand after %d presses of A: %s' % (key, tries, attempts))


def _locate(ctx, name, tries=4):
    for k in range(tries):
        shot = ctx.shot('%s-%d' % (name, k))
        found = handfind.find(str(shot))
        if found and found['score'] >= 0.6:
            return found, str(shot)
        time.sleep(0.5)
    return None, None


def steer(ctx, target, trail, key, tolerance=12, rounds=24):
    """Pulses of the left stick until the fingertip is within `tolerance` px of `target`."""
    gain = 0.6  # px per ms of full deflection; learned from each pulse
    log = []
    last = None
    for r in range(rounds):
        ctx.check_stop()
        found, shot = _locate(ctx, '%s-r%02d' % (key, r))
        if found is None:
            log.append({'round': r, 'hand': None})
            raise RuntimeError('%s: the hand was not found in the frame: %s' % (key, log[-4:]))
        tip = found['tip']
        if last is not None:
            moved = math.hypot(tip[0] - last[0][0], tip[1] - last[0][1])
            expected = last[1] * last[2]
            if expected > 0 and moved > 2:
                gain = max(0.1, min(3.0, 0.5 * gain + 0.5 * moved / expected))
        dx, dy = target[0] - tip[0], target[1] - tip[1]
        dist = math.hypot(dx, dy)
        log.append({'round': r, 'tip': tip, 'score': found['score'], 'dist': round(dist, 1), 'gain': round(gain, 3)})
        if dist <= tolerance:
            trail[key] = log
            return tip
        magnitude = 1.0 if dist > 120 else 0.5
        # Aim for 80 % of the distance so the hand never overshoots by much.
        ms = int(max(40, min(700, 0.8 * dist / (gain * magnitude))))
        x, y = dx / dist * magnitude, -dy / dist * magnitude  # the stick's +y is up, the frame's +y is down
        ctx.call('input.stick', {'stick': 'left', 'x': round(x, 3), 'y': round(y, 3), 'lease_ms': ms})
        time.sleep(ms / 1000 + 0.45)
        ctx.call('input.reset')
        last = (tip, ms, magnitude)
    trail[key] = log
    raise RuntimeError('%s: the hand did not reach %s in %d pulses: %s' % (key, target, rounds, log[-4:]))


def pick_pair(ctx, hs, trail):
    """P1 Game & Watch, then the CPU coin from Random to Ganondorf; every step checked."""
    trail['p1'] = hs.css_to_tile(ctx, GW_TILE)
    # Tile 29 has a neighbour to the left (28) and to the right (30).
    place(ctx, hs, GW_TILE, 'left', 'right', trail, 'p1_place')
    ctx.shot('hang-css-gw')
    trail['coin'] = []
    for attempt in range(3):
        steer(ctx, COIN, trail, 'steer%d' % attempt)
        _press(ctx, 'a')
        # A held coin steps 86 -> 85 (Mii Gunner), as the old route's pick macro did.
        token, seen = _probe(ctx, hs, 'left')
        trail['coin'].append({'token': token, 'probe': seen})
        if token == 'p2':
            break
        if token is not None:
            raise RuntimeError('A on the CPU coin picked up %s: %s' % (token, seen))
    else:
        raise RuntimeError('the CPU coin was not picked up in 3 approaches: %s' % trail['coin'])
    trail['cpu'] = hs.css_to_tile(ctx, GANON_TILE)
    # Tile 25 ends its row: the probe goes left (24), the step back right.
    place(ctx, hs, GANON_TILE, 'left', 'right', trail, 'cpu_place')
    trail['shot_pair'] = str(ctx.shot('hang-css-pair'))
