"""quickplay-cancel-fix Eden step hooks (tools/eden-runner StepContext).

Two vanilla-or-plugin instances on the XPS Quickplay route (a and b pair only with each other: rating runtime, TLS).
b searches first and waits in Quickplay's waiting area; a readies and is matched at once. `ready_and_try_leave` is the
row under test: a readies, then tries the game's own cancel (Plus opens the waiting menu, Up to "Leave Battle", A,
Yes) until a match session starts or the leave goes through. Every look is kept as a frame; the trace stream of the
Quickplay state machines (trace points in config.json) is copied into the row folder.

Route pieces come from rc6-bg-qp6 (worker B's XPS Quickplay rows): to_main (online_steps.py), reach_css_tile
(xps_steps.py), the READY TO FIGHT look (qp.py ready_plus), _vary_still and keepalive (of_steps.py).
"""
import json
import os
import re
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
REFS = HERE / 'refs'
MAIN_REF = REFS / 'main.png'
BANNER_ROI = [200, 110, 600, 240]       # READY TO FIGHT banner (ready-banner gate)
TAB_BOX = [45, 4, 295, 58]              # the waiting menu's "Leave Battle" tab, 1280x674 frame
MENU_BOX = [75, 125, 340, 158]          # the waiting menu's "Training Partners" header (Plus opens it there)
DIALOG_ROI = [440, 285, 840, 325]       # a centered two-button dialog's text line (Rematch?, Leave?)
SESSION = re.compile(r'session slot=\d+ kind=online')
SYNC_END = re.compile(r'sync_end trigger=\w+[^\n]*')
# config.json trace points (both instances): the Quickplay parent's and IngameCallSequenceScene's state stores, the
# PracticeScene result read and the scene request.
POINTS = {'main+0x236b4ac': 'parent', 'main+0x23191bc': 'child', 'main+0x23105bc': 'practice_result',
          'main+0x37266d0': 'scene', 'main+0x18514bc': 'practice_state', 'main+0x185129c': 'match_received',
          'main+0x185113c': 'practice_msg67', 'main+0x18518b0': 'practice_leave', 'main+0x1851310': 'quit_request',
          'main+0x16a1df0': 'round_start', 'main+0x16b9830': 'round_ensure', 'main+0x16a2120': 'mask_abort',
          'main+0x16a21e8': 'mask_notready', 'main+0x168b500': 'mask_reset', 'main+0x16b2d6c': 'mask_store_b2d6c',
          'main+0x169eadc': 'mask_store_9eadc', 'main+0x1892048': 'mask_store_92048', 'main+0x1685cb0': 'send',
          'main+0x1685ac0': 'send_to', 'main+0x1566af4': 'host_round_start', 'main+0x15669c4': 'host_ready_check',
          'main+0x1564088': 'countdown_read'}


def _size(path):
    try:
        return os.path.getsize(path)
    except OSError:
        return 0


def _tail(path, start):
    try:
        with open(path, 'rb') as f:
            f.seek(start)
            return f.read().decode(errors='replace')
    except OSError:
        return ''


def _trace_path(ctx, on=None):
    return ctx.runner.instances[on or ctx.on].get('trace_jsonl')


def _save(ctx, name, data):
    path = Path(ctx.row_dir) / f'{name}.json'
    path.write_text(json.dumps(data, indent=1) + '\n')
    ctx.log({name: data if len(json.dumps(data)) < 2000 else str(path)})
    return data


def _diff(ctx, shot, ref, roi):
    return ctx.runner.roi_diff(str(shot), str(ref), list(roi))


def _trace_records(path, start):
    out = []
    for line in _tail(path, start).splitlines():
        try:
            out.append(json.loads(line))
        except ValueError:
            continue
    return out


def _kind(record):
    for loc in (record.get('locations') or {}).values():
        if loc in POINTS:
            return POINTS[loc]
    return None


def _states(records):
    """The QCF trace points' records, compact: kind, frame, x8 (new state / result), x9 (old state), captures."""
    rows = []
    for r in records:
        kind = _kind(r)
        if kind is None or r.get('event') != 'entry':
            continue
        regs = r.get('registers') or {}
        caps = r.get('captures') or []
        rows.append({'kind': kind, 'frame': r.get('frame'), 'x8': regs.get('x8'), 'x9': regs.get('x9'),
                     'regs': regs, 'caps': [c.get('value') if c.get('type') == 'c-string' else
                                            (c.get('hex') or c.get('value')) for c in caps]})
    return rows


# --- navigation -------------------------------------------------------------------------------------------------------

def to_main(ctx, roi=(150, 265, 450, 335), tolerance=8, tries=6, wait_s=4.0):
    """Opening movie, attract demo or title -> the main menu: look; if it isn't the main menu, Plus, look again."""
    diffs = []
    for attempt in range(tries + 1):
        path = ctx.shot(f'to-main-{attempt}')
        diff = ctx.runner.roi_diff(path, str(MAIN_REF), list(roi), True)
        diffs.append(round(diff, 1))
        if diff <= tolerance:
            return _save(ctx, f'to-main-{ctx.on}', {'attempt': attempt, 'diffs': diffs})
        if attempt < tries:
            ctx.press('plus', 150)
            time.sleep(wait_s)
    raise RuntimeError(f'main menu not reached after {tries} presses; diffs {diffs}')


def reach_css_tile(ctx, tries=5, wait_s=4.0):
    """Quickplay menu or its character select -> a token on a CSS tile (`settled screen=css at=N hovered=N`)."""
    runner, on = ctx.runner, ctx.on
    tile_log = runner.instances[on]['tile_log']
    start = _size(tile_log)
    shot = runner.shot(on, 'css-look')
    if _diff(ctx, shot, REFS / 'quickplay-menu.png', [40, 5, 300, 45]) <= 8:
        ctx.play('to-css')
    for i in range(tries):
        ctx.check_stop()
        time.sleep(wait_s)
        if re.search(r'settled screen=css at=(\d+) hovered=\1\b', _tail(tile_log, start)):
            return {'tile_steps': i}
        ctx.play('tile-up')
    raise RuntimeError(f'no settled css tile on {on} after {tries} tile steps')


def css_to_banner(ctx, cap_s=20):
    """Character select reached after a match or a leave: the READY TO FIGHT banner up (token placed), or a tile step
    and A to place the token. Looks first."""
    shot = ctx.shot('css-look')
    diff = _diff(ctx, shot, REFS / 'ready-banner.png', BANNER_ROI)
    if diff <= 40:
        return {'banner': 'already up', 'diff': round(diff, 2)}
    ctx.play('tile-up')
    ctx.gate('css-tile')
    ctx.play('pick')
    ctx.gate('ready-banner-any')
    return {'banner': 'stepped and placed', 'diff_before': round(diff, 2)}


def _ready(ctx, tag, tries=4):
    """READY TO FIGHT up -> Plus starts the search (qp.py ready_plus); no banner -> A places the token, look again."""
    looks = []
    for i in range(tries):
        shot = ctx.shot(f'{tag}-ready-look-{i}')
        diff = _diff(ctx, shot, REFS / 'ready-banner.png', BANNER_ROI)
        looks.append(round(diff, 2))
        if diff <= 40:
            ctx.press('plus', 120)
            return {'looks': looks, 'ready_at': time.time()}
        ctx.press('a', 120)
        time.sleep(1.5)
    raise RuntimeError(f'no READY TO FIGHT banner after {tries} looks: {looks}')


def ready_and_wait_searching(ctx, tag='w', cap_s=90):
    """b readies alone: its IngameCallSequenceScene reaches state 3 (PracticeScene, the waiting area), read from the
    child state-change point (x8 == 3) in b's trace. A frame of the waiting area."""
    path = _trace_path(ctx)
    start = _size(path)
    ready = _ready(ctx, tag)
    began = time.monotonic()
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        states = _states(_trace_records(path, start))
        if any(s['kind'] == 'child' and s['x8'] == '0x3' for s in states):
            time.sleep(4.0)
            shot = ctx.shot(f'{tag}-searching')
            return _save(ctx, f'{tag}-searching-{ctx.on}', {'seconds': round(time.monotonic() - began, 1),
                                                           'ready': ready, 'shot': str(shot), 'states': states})
        time.sleep(1.0)
    shot = ctx.shot(f'{tag}-searching-timeout')
    _save(ctx, f'{tag}-searching-{ctx.on}', {'timeout': cap_s, 'states': _states(_trace_records(path, start))})
    raise RuntimeError(f'{ctx.on}: no PracticeScene (child state 3) within {cap_s} s: {shot}')


# --- the row under test -----------------------------------------------------------------------------------------------

def _tab_shown(ctx, shot):
    # The reference is the ROI itself (compared with b_is_roi; a full-frame compare pads it with black).
    return ctx.runner.roi_diff(str(shot), str(REFS / 'leave-battle-tab.png'), list(TAB_BOX), True) < 12


def _menu_shown(ctx, shot):
    """Plus opened the waiting menu: its "Training Partners" header (focus on it). ROI-sized reference."""
    return ctx.runner.roi_diff(str(shot), str(REFS / 'waiting-menu.png'), list(MENU_BOX), True) < 12


def _left_in_trace(ctx, start):
    """PracticeScene entered its leave state (4) since `start` in this instance's trace."""
    path = ctx.runner.instances[ctx.on].get('trace_jsonl') or ''
    # Quickplay's "leave the game" ends PracticeScene with result 4 (yrm01 m2b: exit, result 4, child 1); Local
    # Wireless rooms go through PracticeScene state 4.
    return any((s['kind'] == 'practice_state' and s['x8'] == '0x4')
               or (s['kind'] == 'practice_result' and s['x8'] in ('0x4', '0x5'))
               or s['kind'] in ('practice_leave', 'quit_request')
               for s in _states(_trace_records(path, start)))


def _leave_from_menu(ctx, tag, trace_start, menu_shot):
    """The waiting menu is open (menu_shot): the game's own Leave Battle (_explore_menu)."""
    return _explore_menu(ctx, tag, trace_start, menu_shot)


def _dialog_yes(ctx, tag):
    """A centered two-button dialog is up: Yes is the right-hand button in this game's dialogs (Rematch?, Exit
    without saving?). Right, then A. Returns the frames."""
    shots = [str(ctx.shot(f'{tag}-dialog'))]
    ctx.press('right', 120)
    time.sleep(0.5)
    shots.append(str(ctx.shot(f'{tag}-dialog-right')))
    ctx.press('a', 120)
    time.sleep(1.0)
    shots.append(str(ctx.shot(f'{tag}-dialog-a')))
    return shots


def _last_waiting_entry_ns(ctx, inst):
    """Wall time (ns) of `inst`'s latest IngameCallSequenceScene change to state 3 (its PracticeScene began)."""
    last = None
    for r in _trace_records(ctx.runner.instances[inst].get('trace_jsonl') or '', 0):
        if _kind(r) == 'child' and (r.get('registers') or {}).get('x8') == '0x3':
            last = int(r.get('wall_time_ns') or 0)
    return last


def ready_and_try_leave(ctx, tag='m', expect=None, cap_s=45, settle_s=20, host_wait_s=None, try_leave=True):
    """a readies; then, until a match session starts on a (detector) or `cap_s`: look every 0.5 s; when the game's
    waiting menu is open, Up to "Leave Battle", A, Yes; when it is not, Plus (opens it) at most once a second.
    Outcome `left` (a left the search before any session started, and none started within `settle_s`) or `matched`
    (a session started). `expect` ('left' or 'matched') fails the row on the other outcome; evidence either way.
    `host_wait_s`: Ready only once the peer's waiting area has run that long (the peer's own countdown); the peer's
    actual wait at Ready is recorded either way."""
    runner, on = ctx.runner, ctx.on
    det = runner.instances[on]['detector_log']
    det_start = _size(det)
    peer = 'b' if on == 'a' else 'a'
    peer_det = runner.instances[peer]['detector_log']
    peer_start = _size(peer_det)
    traces = {k: _size(runner.instances[k].get('trace_jsonl') or '') for k in (on, peer)}
    out = {'tag': tag, 'expect': expect, 'events': [], 'host_wait_s': host_wait_s}
    entered = _last_waiting_entry_ns(ctx, peer)
    if host_wait_s and entered:
        remaining = host_wait_s - (time.time_ns() - entered) / 1e9
        if remaining > 0:
            time.sleep(remaining)
    out['peer_waiting_s_at_ready'] = round((time.time_ns() - entered) / 1e9, 1) if entered else None
    out['ready'] = _ready(ctx, tag)
    began = time.monotonic()
    last_plus = 0.0
    left_at = None
    i = 0
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        if SESSION.search(_tail(det, det_start)):
            out['events'].append({'t': round(time.monotonic() - began, 2), 'what': 'session'})
            break
        if _match_in_trace(runner, on, traces[on]):
            out['events'].append({'t': round(time.monotonic() - began, 2), 'what': 'match-found (trace)'})
            break
        if not try_leave:
            time.sleep(0.5)   # the same match, without touching the waiting menu
            continue
        shot = ctx.shot(f'{tag}-look-{i:03d}')
        i += 1
        t = round(time.monotonic() - began, 2)
        if _menu_shown(ctx, shot) or _tab_shown(ctx, shot):
            out['events'].append({'t': t, 'what': 'menu', 'shot': str(shot)})
            attempt = _leave_from_menu(ctx, tag, traces[on], shot)
            out['events'].append(dict(attempt, t=round(time.monotonic() - began, 2), what='leave-attempt'))
            if attempt['left']:
                left_at = time.monotonic()
                break
            last_plus = time.monotonic()   # the menu was closed again with B
            continue
        if time.monotonic() - last_plus >= 1.0:
            ctx.press('plus', 100)
            last_plus = time.monotonic()
            out['events'].append({'t': t, 'what': 'plus'})
        time.sleep(0.35)
    if left_at is not None:
        end = left_at + settle_s
        while time.monotonic() < end and not SESSION.search(_tail(det, det_start)):
            ctx.check_stop()
            time.sleep(1.0)
    if left_at is None and _match_in_trace(runner, on, traces[on]):
        # The match was found and is loading (slow hosts): wait for the battle's session line.
        end = time.monotonic() + 120
        while time.monotonic() < end and not SESSION.search(_tail(det, det_start)):
            ctx.check_stop()
            time.sleep(1.0)
        out['match_in_trace'] = True
    session = SESSION.search(_tail(det, det_start))
    out['outcome'] = 'matched' if session else ('left' if left_at is not None else 'none')
    out['frames_taken'] = i
    out['after'] = str(ctx.shot(f'{tag}-after'))
    out['peer_after'] = str(runner.shot(peer, f'{tag}-peer-after'))
    out['detector'] = {on: _tail(det, det_start).splitlines()[-6:], peer: _tail(peer_det, peer_start).splitlines()[-6:]}
    out['trace'] = {}
    for k in (on, peer):
        path = runner.instances[k].get('trace_jsonl') or ''
        (Path(ctx.row_dir) / f'{tag}-trace-{k}.jsonl').write_text(_tail(path, traces[k]))
        out['trace'][k] = _states(_trace_records(path, traces[k]))[-300:]
    _save(ctx, f'{tag}-try-leave', out)
    if expect and out['outcome'] != expect:
        raise RuntimeError(f'{tag}: outcome {out["outcome"]}, expected {expect}')
    return {'outcome': out['outcome'], 'events': len(out['events'])}


def _match_in_trace(runner, key, start):
    """The instance's trace since `start` shows a found match: PracticeScene result 3 or MeleeScene requested."""
    path = runner.instances[key].get('trace_jsonl') or ''
    for s in _states(_trace_records(path, start)):
        if s['kind'] == 'practice_result' and s['x8'] == '0x3':
            return True
        if s['kind'] == 'child' and s['x8'] == '0x4':
            return True
    return False


def leave_waiting(ctx, tag='lw', cap_s=45):
    """This instance leaves its waiting area the game's way (Plus, the waiting menu, Leave Battle), so that its
    next search starts a fresh room. Verified by PracticeScene's leave state in its trace; frames."""
    path = _trace_path(ctx) or ''
    start = _size(path)
    began = time.monotonic()
    attempts = []
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        shot = ctx.shot(f'{tag}-look')
        if _menu_shown(ctx, shot) or _tab_shown(ctx, shot):
            attempt = _leave_from_menu(ctx, tag, start, shot)
            attempts.append(attempt)
            if attempt['left']:
                time.sleep(8.0)
                return _save(ctx, f'{tag}-left-{ctx.on}', {'attempts': attempts,
                                                           'seconds': round(time.monotonic() - began, 1)})
        else:
            ctx.press('plus', 100)
        time.sleep(1.2)
    raise RuntimeError(f'{ctx.on}: did not leave the waiting area in {cap_s} s: {attempts}')


def _explore_menu(ctx, tag, start, menu_shot):
    """The waiting menu is open. When its "Leave Battle" tab shows (the reference is the tab unfocused; the focus
    starts on Training Partners, and Up focuses the tab, e1 20:46Z): Up, A, then Yes (Right+A, then Left+A),
    verified by PracticeScene's leave state. Without the tab: Up x3, L, R, ZL, ZR with frames (where else could it
    be). B closes the menu."""
    steps = []
    out = {'tab': _tab_shown(ctx, menu_shot), 'left': False, 'steps': steps}
    if out['tab']:
        ctx.press('up', 120)
        time.sleep(0.6)
        shot = ctx.shot(f'{tag}-x-tab-focus')
        steps.append({'button': 'up', 'tab_unfocused': _tab_shown(ctx, shot), 'shot': str(shot)})
        ctx.press('a', 120)
        time.sleep(1.2)
        dialog = ctx.shot(f'{tag}-x-after-a')
        out['after_a'] = str(dialog)
        # "Would you like to leave the game?" opens on No (yrm01 m2b): move to Yes only when the frame shows No.
        focus = _dialog_focus(dialog)
        out['dialog_focus'] = focus
        if focus == 'no':
            ctx.press('right', 120)
            time.sleep(0.5)
            dialog = ctx.shot(f'{tag}-x-yes-focus')
            out['yes_focus'] = str(dialog)
        if _dialog_focus(dialog) == 'yes':
            ctx.press('a', 120)
            time.sleep(2.0)
            out['confirm'] = str(ctx.shot(f'{tag}-x-confirmed'))
        out['left'] = _left_in_trace(ctx, start)
    else:
        for button in ('up', 'up', 'up', 'l', 'r', 'zl', 'zr'):
            ctx.press(button, 120)
            time.sleep(0.6)
            shot = ctx.shot(f'{tag}-x-{len(steps)}-{button}')
            steps.append({'button': button, 'tab': _tab_shown(ctx, shot), 'shot': str(shot)})
    if not out['left']:
        for _ in range(3):
            ctx.press('b', 120)
            time.sleep(0.4)
        out['closed'] = str(ctx.shot(f'{tag}-x-closed'))
    return out


def explore_waiting(ctx, tag='ex', cap_s=60, ready=False):
    """Exploration (never fails on the game's answer): optionally Ready first; then Plus until the waiting menu
    opens, and `_explore_menu` (where is Leave Battle, and does it leave). Stops when a match is found (trace) or
    it left. Saves `{tag}-try-leave.json` (outcome left / matched / none) so `after_soft` can follow."""
    runner, on = ctx.runner, ctx.on
    path = _trace_path(ctx) or ''
    start = _size(path)
    out = {'tag': tag, 'events': []}
    if ready:
        out['ready'] = _ready(ctx, tag)
    began = time.monotonic()
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        t = round(time.monotonic() - began, 2)
        if _match_in_trace(runner, on, start):
            out['events'].append({'t': t, 'what': 'match-found (trace)'})
            break
        shot = ctx.shot(f'{tag}-look')
        if _menu_shown(ctx, shot) or _tab_shown(ctx, shot):
            out['events'].append({'t': t, 'what': 'menu', 'shot': str(shot), 'tab': _tab_shown(ctx, shot)})
            got = _explore_menu(ctx, tag, start, shot)
            out['events'].append(dict(got, t=round(time.monotonic() - began, 2), what='explore'))
            if got['left']:
                break
            continue
        ctx.press('plus', 100)
        out['events'].append({'t': t, 'what': 'plus'})
        time.sleep(1.0)
    time.sleep(3.0)
    out['states'] = [s for s in _states(_trace_records(path, start))
                     if s['kind'] in ('child', 'parent', 'practice_state', 'practice_result', 'local_state',
                                      'leave_confirmed', 'quit_request', 'practice_leave')]
    out['outcome'] = ('left' if _left_in_trace(ctx, start) else
                      'matched' if _match_in_trace(runner, on, start) else 'none')
    out['frames'] = {k: str(runner.shot(k, f'{tag}-end')) for k in ctx.runner.instances}
    _save(ctx, f'{tag}-try-leave', out)
    return {'outcome': out['outcome'], 'events': len(out['events'])}


def _match_marks(runner, starts):
    """Per instance since `starts`: (session and sync_end lines seen, a session is open: the last session line comes
    after the last sync_end)."""
    out = {}
    for k, start in starts.items():
        text = _tail(runner.instances[k]['detector_log'], start)
        sessions = [m.start() for m in SESSION.finditer(text)]
        ends = [m.start() for m in SYNC_END.finditer(text)]
        out[k] = (len(sessions) + len(ends), bool(sessions) and (not ends or sessions[-1] > ends[-1]))
    return out


def after_soft(ctx, tag='m', settle_s=60.0, cap_s=240.0):
    """after_try that records instead of failing: a match that errors is waited out (both back at the character
    select) and noted. A sudden death after a time-out (a second detector session) is waited out too, then the
    Rematch? countdown."""
    runner = ctx.runner
    starts = {k: _size(runner.instances[k]['detector_log']) for k in ctx.runner.instances}
    out = {}
    try:
        out['after'] = after_try(ctx, tag)
    except RuntimeError as e:
        out['error'] = str(e)
        time.sleep(settle_s)
    began = time.monotonic()
    marks, quiet_since, extra = None, time.monotonic(), False
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        now = _match_marks(runner, starts)
        if now != marks:
            marks, quiet_since = now, time.monotonic()
        if any(open_ for _, open_ in now.values()):
            extra = True
        elif time.monotonic() - quiet_since >= 20.0:
            break
        time.sleep(3.0)
    if extra:
        time.sleep(35.0)   # results, then the Rematch? countdown
    out['marks'] = marks
    out['waited_extra_s'] = round(time.monotonic() - began, 1)
    out['frames'] = {k: str(runner.shot(k, f'{tag}-soft')) for k in ctx.runner.instances}
    return _save(ctx, f'{tag}-after-soft', out)


def ready_only(ctx, tag='r'):
    """READY TO FIGHT -> Plus (the search starts); returns at once."""
    return _ready(ctx, tag)


def leave_after_peer_joins(ctx, tag='hl', delay_s=4.0, join_cap_s=60, cap_s=60):
    """This instance waits alone in its room; once the peer's trace shows it entering the match flow
    (IngameCallSequenceScene, i.e. it joined this room and is loading), wait `delay_s`, then the waiting menu and the
    game's own Leave Battle (`explore_waiting`). The native leave of the console that was waiting, while the other
    one loads."""
    peer = 'a' if ctx.on == 'b' else 'b'
    ppath = _trace_path(ctx, peer) or ''
    pstart = _size(ppath)
    began = time.monotonic()
    joined = None
    while time.monotonic() - began < join_cap_s:
        ctx.check_stop()
        if any(s['kind'] == 'parent' and s['x8'] == '0x8' for s in _states(_trace_records(ppath, pstart))):
            joined = round(time.monotonic() - began, 1)
            break
        time.sleep(0.5)
    _save(ctx, f'{tag}-peer-joined', {'peer': peer, 'joined_after_s': joined,
                                      'frames': {k: str(ctx.runner.shot(k, f'{tag}-joined')) for k in ctx.runner.instances}})
    if joined is None:
        raise RuntimeError(f'{peer} did not join within {join_cap_s} s')
    time.sleep(delay_s)
    return explore_waiting(ctx, tag=tag, cap_s=cap_s)


def _tap(runner, on, button):
    runner.call(on, 'input.button', {'button': button, 'state': 'tap', 'duration_ms': 100, 'lease_ms': 500})


def _where(runner, on):
    """This instance's last Quickplay parent and IngameCallSequenceScene states from its trace."""
    states = [s for s in _states(_trace_records(runner.instances[on].get('trace_jsonl') or '', 0))
              if s['kind'] in ('parent', 'child')]
    parent = next((s['x8'] for s in reversed(states) if s['kind'] == 'parent'), None)
    child = next((s['x8'] for s in reversed(states) if s['kind'] == 'child'), None)
    return {'parent': parent, 'child': child, 'in_lobby': parent == '0x8' and child == '0x3',
            'at_css': parent == '0x4'}


def _dialog_focus(shot):
    """Which button of the centered two-button dialog is focused (orange): 'no' (left), 'yes' (right) or None."""
    from PIL import Image
    with Image.open(shot) as im:
        rgb = im.convert('RGB')
        w, h = rgb.size
        sx, sy = w / 1280.0, h / 674.0

        def orange(x, y):
            r, g, b = rgb.getpixel((int(x * sx), int(y * sy)))
            return r > 180 and 60 < g < 170 and b < 90
        if orange(300, 500):
            return 'no'
        if orange(700, 500):
            return 'yes'
    return None


def rematch_answers(ctx, answers=None, tag='rm', cap_s=180, tolerance=15.0):
    """After a match: wait for each console's "Rematch?" dialog and answer it ('yes': Right, A; 'no': Left, A; Yes is
    the right-hand button). Both idle meanwhile (a time-out's sudden death ends by itself). Then where each went
    (lobby or character select) and frames."""
    runner = ctx.runner
    answers = answers or {'a': 'yes', 'b': 'no'}
    out = {'answers': answers, 'seen': {}, 'diffs': {k: [] for k in answers}}
    began = time.monotonic()
    while time.monotonic() - began < cap_s and len(out['seen']) < len(answers):
        ctx.check_stop()
        for k, answer in answers.items():
            if k in out['seen']:
                continue
            shot = runner.shot(k, f'{tag}-look')
            diff = runner.roi_diff(str(shot), str(REFS / 'rematch-dialog.png'), list(DIALOG_ROI))
            out['diffs'][k] = (out['diffs'][k] + [round(diff, 1)])[-6:]
            if diff < tolerance:
                # The focus starts on Yes and Left/Right wrap (yrm01 m1: Right from Yes went to No): move only when
                # the frame shows the other button focused, then A on the checked focus.
                focus = _dialog_focus(shot)
                chosen = shot
                if focus != answer:
                    _tap(runner, k, 'right' if answer == 'yes' else 'left')
                    time.sleep(0.4)
                    chosen = runner.shot(k, f'{tag}-{answer}-focus')
                    if _dialog_focus(chosen) != answer:
                        continue
                _tap(runner, k, 'a')
                out['seen'][k] = {'t': round(time.monotonic() - began, 1), 'dialog': str(shot),
                                  'focus_before': focus, 'focus': str(chosen)}
        time.sleep(0.5)
    if len(out['seen']) < len(answers):
        _save(ctx, f'{tag}-rematch', out)
        raise RuntimeError(f'no Rematch? dialog on {set(answers) - set(out["seen"])} within {cap_s} s: {out["diffs"]}')
    time.sleep(12.0)
    out['after'] = {k: str(runner.shot(k, f'{tag}-after')) for k in answers}
    out['where'] = {k: _where(runner, k) for k in answers}
    return _save(ctx, f'{tag}-rematch', out)


def where_are(ctx, tag='where', wait_s=0.0):
    """Frames and the last parent/child states of both instances."""
    time.sleep(wait_s)
    out = {k: dict(_where(ctx.runner, k), shot=str(ctx.runner.shot(k, tag))) for k in ctx.runner.instances}
    return _save(ctx, f'{tag}-where', out)


def play_out(ctx, tag='m', cap_s=240.0):
    """A match is running: both act until it ends (`battle`); an error state or a sudden death is waited out, then the
    Rematch? countdown (no answer: both to the character select). Records instead of failing."""
    runner = ctx.runner
    starts = {k: _size(runner.instances[k]['detector_log']) for k in ctx.runner.instances}
    out = {}
    try:
        out['battle'] = battle(ctx, tag)
    except RuntimeError as e:
        out['error'] = str(e)
        time.sleep(60.0)
    began = time.monotonic()
    marks, quiet_since = None, time.monotonic()
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        now = _match_marks(runner, starts)
        if now != marks:
            marks, quiet_since = now, time.monotonic()
        if not any(open_ for _, open_ in now.values()) and time.monotonic() - quiet_since >= 20.0:
            break
        time.sleep(3.0)
    time.sleep(35.0)
    out['where'] = {k: _where(runner, k) for k in ctx.runner.instances}
    out['frames'] = {k: str(runner.shot(k, f'{tag}-played')) for k in ctx.runner.instances}
    return _save(ctx, f'{tag}-play-out', out)


def leave_if_searching(ctx, tag='lis'):
    """If this instance waits in its lobby, leave it the game's way (alone: the Leave Battle tab); else nothing."""
    where = _where(ctx.runner, ctx.on)
    if not where['in_lobby']:
        return _save(ctx, f'{tag}-{ctx.on}', {'where': where, 'left': None})
    return _save(ctx, f'{tag}-{ctx.on}', {'where': where, 'left': leave_waiting(ctx, tag=tag)})


def ready_until_searching(ctx, tag='rs', tries=3, wait_s=25.0):
    """From the character select: READY, Plus, and check this instance's own trace for its search start (parent 5);
    again when it doesn't come (yrm01 m2b: a Ready right after a declined rematch didn't register)."""
    path = _trace_path(ctx) or ''
    attempts = []
    for i in range(tries):
        start = _size(path)
        attempts.append({'css': css_to_banner(ctx), 'ready': _ready(ctx, f'{tag}-{i}')})
        began = time.monotonic()
        while time.monotonic() - began < wait_s:
            ctx.check_stop()
            if any(s['kind'] == 'parent' and s['x8'] == '0x5' for s in _states(_trace_records(path, start))):
                return _save(ctx, f'{tag}-searching-{ctx.on}', {'attempts': attempts, 'try': i})
            time.sleep(0.5)
        attempts[-1]['shot'] = str(ctx.shot(f'{tag}-no-search-{i}'))
    _save(ctx, f'{tag}-searching-{ctx.on}', {'attempts': attempts, 'try': None})
    raise RuntimeError(f'{ctx.on}: no search start after {tries} Readys')


def pause_during_join(ctx, tag='pj', join_cap_s=90, cap_s=90, leave=True, peer_host=None, peer_trace=None):
    """This instance waits in its lobby. From the moment the peer's trace shows it in a room (parent 8), press Plus
    until the waiting menu is open and take a frame every second: is "Leave Battle" there while the peer loads? With
    `leave`, take it (Up, A, Yes on "leave the game?"). Ends when this instance left, its match was found, or `cap_s`.
    Saves `{tag}-try-leave.json` (outcome left / matched / none) for after_soft."""
    runner, on = ctx.runner, ctx.on
    peer = 'a' if on == 'b' else 'b'
    path = _trace_path(ctx) or ''
    ppath = peer_trace if peer_host else (_trace_path(ctx, peer) or '')
    # The peer may have reached its room just before this step: read its latest parent state from a window back.
    start, pstart = _size(path), max(0, _size(ppath) - 262144)
    out = {'tag': tag, 'peer': peer, 'looks': []}
    began = time.monotonic()
    while time.monotonic() - began < join_cap_s:
        ctx.check_stop()
        parents = [s['x8'] for s in _peer_list(ppath, pstart, peer_host) if s['kind'] == 'parent']
        if parents and parents[-1] == '0x8':
            out['peer_in_room_s'] = round(time.monotonic() - began, 1)
            break
        time.sleep(0.25)
    else:
        out['outcome'] = 'none'
        _save(ctx, f'{tag}-try-leave', out)
        raise RuntimeError(f'{peer} did not get a room within {join_cap_s} s')
    began = time.monotonic()
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        t = round(time.monotonic() - began, 2)
        if _match_in_trace(runner, on, start):
            out['looks'].append({'t': t, 'what': 'match-found (trace)'})
            break
        if _left_in_trace(ctx, start):
            out['looks'].append({'t': t, 'what': 'left (trace)'})
            break
        shot = ctx.shot(f'{tag}-look')
        menu, tab = _menu_shown(ctx, shot), _tab_shown(ctx, shot)
        peer_child = next((s['x8'] for s in reversed(_peer_list(ppath, pstart, peer_host)) if s['kind'] == 'child'),
                          None)
        out['looks'].append({'t': t, 'menu': menu, 'tab': tab, 'peer_child': peer_child, 'shot': str(shot)})
        if menu or tab:
            if tab and leave:
                got = _explore_menu(ctx, tag, start, shot)
                out['looks'].append(dict(got, t=round(time.monotonic() - began, 2), what='leave'))
                if got['left']:
                    break
        else:
            ctx.press('plus', 100)
        time.sleep(0.8)
    time.sleep(3.0)
    out['outcome'] = ('left' if _left_in_trace(ctx, start) else
                      'matched' if _match_in_trace(runner, on, start) else 'none')
    out['states'] = [s for s in _states(_trace_records(path, start))
                     if s['kind'] in ('child', 'parent', 'practice_state', 'practice_result')]
    out['peer_states'] = [s for s in _peer_list(ppath, pstart, peer_host) if s['kind'] in ('child', 'parent')]
    out['frames'] = {k: str(runner.shot(k, f'{tag}-end')) for k in ctx.runner.instances}
    _save(ctx, f'{tag}-try-leave', out)
    return {'outcome': out['outcome'], 'looks': len(out['looks']), 'peer_in_room_s': out.get('peer_in_room_s')}


# --- cross-host pair (primary 2026-11-10 00:05Z: one instance per PC, one driver per host) ---------------------------
# The peer's state comes from its own trace on its host, read over ssh: only its parent/child state records (grep on
# the remote side), so a 4 Hz sampler in the trace doesn't make every poll read megabytes.

STATE_RE = r'main\+0x236b4ac|main\+0x23191bc'


def _peer_states(host, trace):
    """The peer's parent/child state changes, oldest first: [{'kind', 'x8', 'x9', 'wall_ns'}]."""
    import subprocess
    cmd = ['ssh', '-o', 'BatchMode=yes', '-o', 'ConnectTimeout=8', host,
           f"grep -E '{STATE_RE}' '{trace}' | tail -n 40"]
    try:
        text = subprocess.run(cmd, capture_output=True, text=True, timeout=20).stdout
    except (subprocess.SubprocessError, OSError):
        return None
    out = []
    for line in text.splitlines():
        try:
            r = json.loads(line)
        except ValueError:
            continue
        kind = _kind(r)
        if kind in ('parent', 'child') and r.get('event') == 'entry':
            regs = r.get('registers') or {}
            out.append({'kind': kind, 'x8': regs.get('x8'), 'x9': regs.get('x9'),
                        'wall_ns': int(r.get('wall_time_ns') or 0)})
    return out


def _peer_is(states, condition, since_ns=0):
    """'searching' (in a lobby: parent 8, PracticeScene child 3), 'at_css' (parent 4), 'in_room' (parent 8),
    'rematch_lobby' (a child 5 -> 3 since `since_ns`, still there)."""
    if not states:
        return False
    parent = next((s['x8'] for s in reversed(states) if s['kind'] == 'parent'), None)
    child = next((s['x8'] for s in reversed(states) if s['kind'] == 'child'), None)
    if condition == 'searching':
        return parent == '0x8' and child == '0x3'
    if condition == 'at_css':
        return parent == '0x4'
    if condition == 'in_room':
        return parent == '0x8'
    if condition == 'rematch_lobby':
        return child == '0x3' and parent == '0x8' and any(
            s['kind'] == 'child' and s['x9'] == '0x5' and s['x8'] == '0x3' and s['wall_ns'] >= since_ns
            for s in states)
    raise ValueError(condition)


def _peer_list(path, start, host=None):
    """The peer's parent/child records: from its local trace after `start`, or (host given) its remote trace."""
    if host:
        return _peer_states(host, path) or []
    return [s for s in _states(_trace_records(path, start)) if s['kind'] in ('parent', 'child')]


def wait_peer(ctx, host, trace, condition, cap_s=300, tag='wp', since_s=600):
    """Wait until the peer (other host) is in `condition` by its own trace. `since_s`: how far back a rematch
    re-entry may lie."""
    began = time.monotonic()
    since_ns = time.time_ns() - int(since_s * 1e9)
    last = None
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        states = _peer_states(host, trace)
        if states is not None:
            last = states[-3:]
            if _peer_is(states, condition, since_ns):
                return _save(ctx, f'{tag}-{condition}', {'seconds': round(time.monotonic() - began, 1),
                                                         'last': last})
        time.sleep(1.0)
    _save(ctx, f'{tag}-{condition}', {'timeout_s': cap_s, 'last': last})
    raise RuntimeError(f'peer not {condition} within {cap_s} s: {last}')


def _own_det(ctx):
    return ctx.runner.instances[ctx.on]['detector_log']


def battle_self(ctx, tag='m', cap_s=330, shot_every_s=20.0):
    """This instance acts (no idle drop) until its own detector logs sync_end; then a sudden death (a second session)
    is played out too. Returns the sync_end lines; records instead of failing."""
    runner, on = ctx.runner, ctx.on
    det = _own_det(ctx)
    start = max(0, _size(det) - 4096)
    began, tick, next_shot = time.monotonic(), 0, time.monotonic()
    out = {'ends': []}
    try:
        while time.monotonic() - began < cap_s:
            ctx.check_stop()
            text = _tail(det, start)
            sessions = [m.start() for m in SESSION.finditer(text)]
            ends = list(SYNC_END.finditer(text))
            if ends and (not sessions or ends[-1].start() > sessions[-1]):
                out['ends'] = [m.group(0) for m in ends]
                # a sudden death starts a new session within a few seconds of a time-out
                quiet = time.monotonic()
                while time.monotonic() - quiet < 12.0:
                    time.sleep(1.0)
                    text = _tail(det, start)
                    sessions = [m.start() for m in SESSION.finditer(text)]
                    if sessions and sessions[-1] > ends[-1].start():
                        break
                else:
                    break
                continue
            _vary_still(runner, on, tick)
            tick += 1
            if time.monotonic() >= next_shot:
                runner.shot(on, f'{tag}-battle-{int(time.monotonic() - began):03d}s')
                next_shot += shot_every_s
            time.sleep(0.5)
    finally:
        runner.call(on, 'input.reset')
    out['seconds'] = round(time.monotonic() - began, 1)
    out['desync_free'] = bool(out['ends']) and all('desync_seen=0' in e for e in out['ends'])
    return _save(ctx, f'{tag}-battle-{on}', out)


def rematch_self(ctx, answer='no', tag='rm', cap_s=150, tolerance=15.0):
    """This instance's "Rematch?" dialog: answer it ('yes'/'no'; the focus is read from the frame, Left/Right wrap),
    then where it went. Records a missing dialog instead of failing."""
    runner, on = ctx.runner, ctx.on
    out = {'answer': answer, 'diffs': []}
    began = time.monotonic()
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        shot = runner.shot(on, f'{tag}-look')
        diff = runner.roi_diff(str(shot), str(REFS / 'rematch-dialog.png'), list(DIALOG_ROI))
        out['diffs'] = (out['diffs'] + [round(diff, 1)])[-6:]
        if diff < tolerance:
            focus = _dialog_focus(shot)
            chosen = shot
            if focus != answer:
                _tap(runner, on, 'right' if answer == 'yes' else 'left')
                time.sleep(0.4)
                chosen = runner.shot(on, f'{tag}-{answer}-focus')
                if _dialog_focus(chosen) != answer:
                    continue
            _tap(runner, on, 'a')
            out.update(seen_s=round(time.monotonic() - began, 1), dialog=str(shot), focus_before=focus,
                       focus=str(chosen))
            break
        time.sleep(0.5)
    time.sleep(12.0)
    out['after'] = str(runner.shot(on, f'{tag}-after'))
    out['where'] = _where(runner, on)
    return _save(ctx, f'{tag}-rematch-{on}', out)


def play_if_matched(ctx, tag='m', wait_s=120, answer=None):
    """Wait up to `wait_s` for this instance's own match session; if one starts: play it out (battle_self), then
    answer the Rematch? (`answer`) or let its countdown run out. Records where this instance ends up."""
    det = _own_det(ctx)
    start = _size(det)
    began = time.monotonic()
    out = {'matched': False}
    while time.monotonic() - began < wait_s:
        ctx.check_stop()
        if SESSION.search(_tail(det, start)):
            out['matched'] = True
            out['session_after_s'] = round(time.monotonic() - began, 1)
            break
        time.sleep(1.0)
    if out['matched']:
        out['battle'] = battle_self(ctx, tag)
        if answer:
            out['rematch'] = rematch_self(ctx, answer, tag)
        else:
            time.sleep(40.0)   # results, then the Rematch? countdown runs out
    else:
        time.sleep(5.0)
    out['where'] = _where(ctx.runner, ctx.on)
    out['frame'] = str(ctx.runner.shot(ctx.on, f'{tag}-played'))
    return _save(ctx, f'{tag}-play-{ctx.on}', out)


def settle(ctx, wait_s=20.0):
    """Wait, then a frame of both (where each is before the next row's steps)."""
    time.sleep(wait_s)
    return {k: str(ctx.runner.shot(k, 'settle')) for k in ctx.runner.instances}


def noop(ctx, why=''):
    """A row kept for the chain after a same-boot fix."""
    return {'why': why}


def tap(ctx, button='a', wait_s=3.0):
    """One button tap, then a fixed wait (the next step is a gate)."""
    ctx.press(button, 120)
    time.sleep(wait_s)
    return {'tap': button}


def ready_now(ctx, tag='r'):
    """Ready at once (Plus on the READY TO FIGHT banner), without waiting for the search to show."""
    return _save(ctx, f'{tag}-ready-{ctx.on}', {'ready': _ready(ctx, tag)})


def ensure_searching(ctx, tag='w'):
    """The peer after a's leave: still searching (its last IngameCallSequenceScene state is 3, PracticeScene) or back
    at the character select (then it readies again). Records which, with a frame."""
    path = _trace_path(ctx)
    states = [s for s in _states(_trace_records(path or '', 0)) if s['kind'] in ('child', 'parent')]
    last_child = next((s for s in reversed(states) if s['kind'] == 'child'), None)
    last_parent = next((s for s in reversed(states) if s['kind'] == 'parent'), None)
    shot = ctx.shot(f'{tag}-peer-look')
    searching = bool(last_child and last_child['x8'] == '0x3' and last_parent and last_parent['x8'] == '0x8')
    out = {'searching': searching, 'last_child': last_child, 'last_parent': last_parent, 'shot': str(shot)}
    if not searching:
        out['css'] = css_to_banner(ctx)
        out['again'] = ready_and_wait_searching(ctx, tag)
    return _save(ctx, f'{tag}-ensure-{ctx.on}', out)


def ready_and_wait_session(ctx, tag='m', cap_s=120):
    """Ready, no leave attempt: wait for a match session on this instance's detector (searching again works)."""
    det = ctx.runner.instances[ctx.on]['detector_log']
    start = _size(det)
    trace = _trace_path(ctx)
    tstart = _size(trace or '')
    ready = _ready(ctx, tag)
    began = time.monotonic()
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        m = SESSION.search(_tail(det, start))
        if m:
            return _save(ctx, f'{tag}-session-{ctx.on}', {'seconds': round(time.monotonic() - began, 1), 'line': m.group(0),
                                                         'ready': ready,
                                                         'states': _states(_trace_records(trace or '', tstart))})
        time.sleep(1.0)
    raise RuntimeError(f'{ctx.on}: no match session within {cap_s} s of Ready')


# --- matches ----------------------------------------------------------------------------------------------------------

def _vary_still(runner, on, tick):
    """Varied input that does not drift (of_steps.py): half walk left, jab, half walk right, shield."""
    kind = tick % 4
    if kind == 0:
        runner.call(on, 'input.stick', {'stick': 'left', 'x': -0.5, 'y': 0, 'lease_ms': 250})
    elif kind == 2:
        runner.call(on, 'input.stick', {'stick': 'left', 'x': 0.5, 'y': 0, 'lease_ms': 250})
    else:
        runner.call(on, 'input.button', {'button': 'a' if kind == 1 else 'zl', 'state': 'tap',
                                         'duration_ms': 80, 'lease_ms': 500})


def battle(ctx, tag='m', cap_s=300, shot_every_s=20.0):
    """Both fighters act (no idle drop) until both detectors log sync_end. Returns the lines; fails on a desync."""
    runner = ctx.runner
    logs = {k: runner.instances[k]['detector_log'] for k in ctx.runner.instances}
    starts = {k: max(0, _size(p) - 4096) for k, p in logs.items()}
    ended, tick = {}, 0
    began = time.monotonic()
    next_shot = began
    try:
        while time.monotonic() - began < cap_s and len(ended) < 2:
            ctx.check_stop()
            for k in logs:
                if k not in ended:
                    _vary_still(runner, k, tick + (1 if k == 'b' else 0))
            tick += 1
            if time.monotonic() >= next_shot:
                for k in logs:
                    runner.shot(k, f'{tag}-battle-{int(time.monotonic() - began):03d}s')
                next_shot += shot_every_s
            time.sleep(0.5)
            for k, p in logs.items():
                m = SYNC_END.search(_tail(p, starts[k]))
                if m:
                    ended[k] = m.group(0)
    finally:
        for k in logs:
            runner.call(k, 'input.reset')
    out = _save(ctx, f'{tag}-battle', {'sync_end': ended, 'seconds': round(time.monotonic() - began, 1)})
    if len(ended) < 2:
        raise RuntimeError(f'no sync_end on both detectors within {cap_s} s: {ended}')
    bad = {k: v for k, v in ended.items() if 'desync_seen=0' not in v}
    if bad:
        raise RuntimeError(f'desync: {bad}')
    return out


def after_try(ctx, tag='m', wait_s=30.0):
    """After a try row: when that try ended in a match, both play it out (`battle`) and decline the Rematch? (both back
    to the character select); when a left, nothing (a is at the character select, b still searching or not:
    the next row's `ensure_searching` reads it)."""
    path = Path(ctx.row_dir).parent
    found = sorted(path.glob(f'*/{tag}-try-leave.json'))
    if not found:
        raise RuntimeError(f'no {tag}-try-leave.json in {path}')
    outcome = json.loads(found[-1].read_text()).get('outcome')
    out = {'tag': tag, 'outcome': outcome}
    if outcome == 'matched':
        out['battle'] = battle(ctx, tag)
        out['decline'] = decline_to_css(ctx, wait_s)
    else:
        out['frames'] = {k: str(ctx.runner.shot(k, f'{tag}-after-leave')) for k in ctx.runner.instances}
    return _save(ctx, f'{tag}-after', out)


def decline_to_css(ctx, wait_s=30.0):
    """Neither answers 'Rematch?': both countdowns run out and both return to the character select. Frames."""
    time.sleep(wait_s)
    return {k: str(ctx.runner.shot(k, 'after-decline')) for k in ctx.runner.instances}


def leave_css_to_quickplay(ctx, tries=4):
    """From the character select back to the Quickplay menu (B), looking after each press."""
    for i in range(tries):
        ctx.press('b', 120)
        time.sleep(2.5)
        shot = ctx.shot(f'leave-css-{i}')
        if _diff(ctx, shot, REFS / 'quickplay-menu.png', [40, 5, 300, 45]) <= 8:
            return {'presses': i + 1}
    raise RuntimeError(f'{ctx.on}: not on the Quickplay menu after {tries} B presses')


def z_instance(ctx):
    """End row per instance: trace.unmapped_access total 0 and no panic/abort/crash/Unmapped in the console."""
    got = ctx.call('trace.unmapped_access', {})
    text = Path(ctx.instance['console_log']).read_text(errors='replace') if ctx.instance.get('console_log') else ''
    bad = {k: len(re.findall(k, text)) for k in ('panic', 'abort', 'crash', 'Unmapped')}
    events = Path(ctx.instance['qcf_log']).read_text(errors='replace') if ctx.instance.get('qcf_log') and \
        os.path.exists(ctx.instance['qcf_log']) else ''
    out = _save(ctx, f'z-{ctx.on}', {'unmapped': got, 'console': bad, 'qcf_events': events.splitlines()[-20:]})
    if int((got or {}).get('total', -1)) != 0 or any(bad.values()):
        raise RuntimeError(f'{ctx.on}: {out}')
    return out
