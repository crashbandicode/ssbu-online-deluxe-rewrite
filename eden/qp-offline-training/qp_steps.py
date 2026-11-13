"""Deluxe qp-fix Eden hooks (2026-11-13): offline Training must read offline, the Quickplay waiting room quickplay.

The GDB stub is on (gdb_rc11.py, no holding helper for the pair), so Eden's guest tracing is off: every check reads
Deluxe's own files (hang-trace.log, events.log, matches/) and the tile cursor's log, and keeps frames.

- `to_main`: opening movie or attract demo -> main menu (Plus, look again), as quickplay-cancel-fix's to_main.
- `offline_training`: main menu -> Games & More -> Training (library main-stage, stage-css, enter-training; P1 on
  tile 0 as the integration lane's css_to_mario, the CPU coin as the select has it). Passes when the Training stage
  reads offline/training in hang-trace.log and its stage log says connection=offline, with no quickplay scene since
  the row began.
- `qp_waiting`: main menu -> Online -> Quickplay -> character select -> READY TO FIGHT -> Plus: the search starts in
  the waiting room. Passes on quickplay/training in hang-trace.log, event 107 a=4 with an online mode word, and a
  stage log with connection=quickplay.
- `training_after_qp`: from that waiting room, the game's own Leave Battle, B back to the main menu (event 107 a=1:
  Quickplay kept), then Games & More -> Training. Passes on event 107 a=5 (the Training stage made the kept Quickplay
  Offline), offline/training and connection=offline.
- `final_evidence`: Deluxe's files copied into the row folder, events decoded.
"""
import json
import re
import shutil
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
REFS = HERE / 'refs'
FRAME = [1280, 674]
MAIN_ROI = [150, 265, 450, 335]
SIDEBAR_ROI = [1140, 85, 1235, 510]     # the icon column right of the main, Online and Quickplay menus
HEADER_ROI = [40, 5, 300, 45]           # the Online / Quickplay menu header
MENU_BOX = [75, 125, 340, 158]          # the waiting menu's "Training Partners" header (quickplay-cancel-fix)
TAB_BOX = [45, 4, 295, 58]              # the waiting menu's "Leave Battle" tab
EVENT = re.compile(r'^(\d+) (\d+) (\d+) (0x[0-9a-f]+) (0x[0-9a-f]+)\s*$')
STEP_CSS = re.compile(r'step screen=css dir=(\w+) from=(\S+) to=(\S+)')
SETTLED_CSS = re.compile(r'settled screen=css at=(\d+) hovered=(\d+)')
SCENE_CHANGE = re.compile(r'\[HangTrace\] scene (\S+) -> (\S+) t=')
HEARTBEAT_SCENE = re.compile(r'\[Heartbeat\] .*? scene=(\S+) ')
CONN = {0: 'offline', 1: 'local', 2: 'arena', 3: 'quickplay'}
STATUS = {0: 'inactive', 1: 'training', 2: 'singles', 3: 'doubles'}
SOURCES = {0x235AAA0: 'main_menu', 0x22DA160: 'quickplay_menu_22da160', 0x22DA090: 'quickplay_menu_22da090',
           0x22D9FA0: 'arena_22d9fa0', 0x1BD45E0: 'local_1bd45e0', 0x25D9288: 'stage_setup',
           0x16CCAB8: 'latency_hook', 0x0FF11E: 'stale_check'}
KEEP = {30, 73, 107, 110, 116}


# --- Deluxe's files ---------------------------------------------------------------------------------------------------

def _dir(ctx, on=None):
    return Path(ctx.runner.instances[on or ctx.on]['deluxe_events']).parent


def _read(path, start=0):
    try:
        with open(path, 'rb') as f:
            f.seek(start)
            return f.read().decode('utf-8', 'replace')
    except OSError:
        return ''


def _size(path):
    try:
        return path.stat().st_size
    except OSError:
        return 0


def _events(ctx):
    """events.log rows as (sequence, tick, code, a, b); this boot's file only."""
    rows = []
    for line in _read(_dir(ctx) / 'events.log').splitlines():
        m = EVENT.match(line.strip())
        if m:
            rows.append(tuple(int(m.group(i), 0) for i in range(1, 6)))
    return rows


def _decode(row):
    seq, tick, code, a, b = row
    if code == 73:
        text = 'context %s/%s ready=%d src=%s' % (CONN.get(a & 0xff), STATUS.get((a >> 8) & 0xff), a >> 16,
                                                  SOURCES.get(b, hex(b)))
    elif code == 30:
        text = 'online_context online=%d src=%s' % (a, SOURCES.get(b, hex(b)))
    elif code == 107:
        text = 'scene_hold a=%d b=%#x' % (a, b)
    elif code == 110:
        text = 'stage %#x status=%s' % (a, STATUS.get(b, b))
    elif code == 116:
        text = 'scene_words melee=%#04x init=%d armed=%d lobby=%d mode=%#010x' % (
            a & 0xff, (a >> 8) & 1, (a >> 9) & 1, (a >> 10) & 1, b)
    else:
        text = 'code %d a=%#x b=%#x' % (code, a, b)
    return '#%d tick=%d %s' % (seq, tick, text)


def _matches(ctx):
    index = _dir(ctx) / 'matches' / 'index.txt'
    return [n.strip() for n in _read(index).splitlines() if n.strip()]


def _mark(ctx):
    events = _events(ctx)
    return {'hang': _size(_dir(ctx) / 'hang-trace.log'), 'seq': events[-1][0] if events else 0,
            'matches': _matches(ctx), 't': time.time()}


def _scenes(ctx, mark):
    """Scene changes ("from -> to") and heartbeat scenes in hang-trace.log since the mark."""
    text = _read(_dir(ctx) / 'hang-trace.log', mark['hang'])
    changes = ['%s -> %s' % (m.group(1), m.group(2)) for m in SCENE_CHANGE.finditer(text)]
    beats = [m.group(1) for m in HEARTBEAT_SCENE.finditer(text)]
    return changes, beats


def _targets(changes):
    return [c.split(' -> ')[1] for c in changes]


def _wait_scene(ctx, want, mark, cap_s):
    end = time.monotonic() + cap_s
    while time.monotonic() < end:
        ctx.check_stop()
        changes, beats = _scenes(ctx, mark)
        if want in _targets(changes) or want in beats:
            return True
        time.sleep(1.0)
    return False


def _new_stage_logs(ctx, mark):
    out = []
    for name in _matches(ctx):
        if name in mark['matches']:
            continue
        head = _read(_dir(ctx) / 'matches' / name).splitlines()[:3]
        out.append({'name': name, 'head': head})
    return out


def _snapshot(ctx, tag, mark):
    """The row's evidence: Deluxe's scene lines, decoded events and new stage logs since the mark; the files copied."""
    folder = Path(ctx.row_dir) / ('deluxe-%s-%s' % (ctx.on, tag))
    folder.mkdir(parents=True, exist_ok=True)
    for name in ('hang-trace.log', 'events.log', 'install.log'):
        src = _dir(ctx) / name
        if src.is_file():
            shutil.copy2(src, folder / name)
    stage_logs = _new_stage_logs(ctx, mark)
    for item in stage_logs:
        shutil.copy2(_dir(ctx) / 'matches' / item['name'], folder / item['name'])
    changes, beats = _scenes(ctx, mark)
    events = [_decode(e) for e in _events(ctx) if e[0] > mark['seq'] and e[2] in KEEP]
    out = {'instance': ctx.on, 'tag': tag, 'scene_changes': changes, 'heartbeat_scenes': sorted(set(beats)),
           'events': events, 'stage_logs': stage_logs, 'folder': str(folder)}
    (Path(ctx.row_dir) / ('%s-%s.json' % (tag, ctx.on))).write_text(json.dumps(out, indent=1) + '\n')
    ctx.log({'qpt': tag, 'on': ctx.on, 'scene_changes': changes, 'stage_logs': stage_logs})
    return out


def _require(cond, what, out):
    if not cond:
        raise RuntimeError('%s: %s' % (what, json.dumps({k: out[k] for k in ('scene_changes', 'stage_logs', 'events')
                                                         if k in out})[:1500]))


# --- screens ----------------------------------------------------------------------------------------------------------

def _diff(ctx, shot, ref, roi):
    return ctx.runner.compare_region(str(shot), str(REFS / ref), list(roi), True, 'fit', FRAME, 8)[0]


def to_main(ctx, tries=8, wait_s=4.0):
    diffs = []
    for attempt in range(tries + 1):
        shot = ctx.shot('to-main-%d' % attempt)
        diff = _diff(ctx, shot, 'main.png', MAIN_ROI)
        diffs.append(round(diff, 1))
        if diff <= 8:
            return {'attempt': attempt, 'diffs': diffs}
        if attempt < tries:
            ctx.press('plus', 150)
            time.sleep(wait_s)
    raise RuntimeError('main menu not reached after %d presses; diffs %s' % (tries, diffs))


def _css_chord(ctx, buttons, hold_ms=180):
    """L3 first, then the D-pad, held together and released (TILE-CURSOR.md; the integration lane's css_chord)."""
    lease = hold_ms + 500
    for button in buttons:
        ctx.call('input.button', {'button': button, 'state': 'press', 'lease_ms': lease})
    time.sleep(hold_ms / 1000)
    for button in reversed(buttons):
        ctx.call('input.button', {'button': button, 'state': 'release'})
    ctx.call('input.reset')


def _css_one_step(path, offset):
    deadline = time.time() + 4
    step = settled = None
    collected = []
    while time.time() < deadline:
        data = path.read_bytes()
        if len(data) < offset:
            offset = 0
        lines = [line for line in data[offset:].decode('utf-8', 'replace').splitlines() if line.strip()]
        offset = len(data)
        collected.extend(lines)
        for line in lines:
            step = STEP_CSS.search(line) or step
            settled = SETTLED_CSS.search(line) or settled
        if step and (step.group(3) == 'none' or settled):
            break
        time.sleep(0.1)
    return offset, step, settled, collected


def _css_to_tile0(ctx):
    """Up, then left, to the edges: tile 0 (the integration lane's css_to_mario)."""
    path = Path(ctx.instance['tile_log'])
    offset = path.stat().st_size
    trail = []
    for direction in ('up', 'left'):
        for _ in range(16):
            ctx.check_stop()
            _css_chord(ctx, ['lstick', direction], 180)
            offset, step, settled, lines = _css_one_step(path, offset)
            if step is None:
                raise RuntimeError('no css step after %s; lines=%s' % (direction, lines[-4:]))
            trail.append(step.group(0))
            if step.group(3) == 'none':
                break
        else:
            raise RuntimeError('%s did not reach the edge in 16 steps' % direction)
    return trail


def _to_training(ctx, tag):
    """Main menu -> Games & More -> Training, P1 on tile 0, the CPU coin as the select has it."""
    ctx.gate('main')
    ctx.shot('%s-main' % tag)
    ctx.play('main-stage')
    ctx.gate('stage')
    ctx.shot('%s-stage' % tag)
    ctx.play('stage-css')
    ctx.gate('css')
    ctx.shot('%s-css' % tag)
    trail = _css_to_tile0(ctx)
    _css_chord(ctx, ['a'], 150)
    time.sleep(0.8)
    ctx.shot('%s-placed' % tag)
    ctx.play('enter-training')
    return trail


# --- rows -------------------------------------------------------------------------------------------------------------

def offline_training(ctx, tag='ot', cap_s=180):
    mark = _mark(ctx)
    # A retry in the same boot may start on a later screen (qpt-y1: the Training select): back to the main menu.
    back = _back_to_main(ctx, '%s-start' % tag, tries=16)
    trail = _to_training(ctx, tag)
    reached = _wait_scene(ctx, 'offline/training', mark, cap_s)
    time.sleep(6.0)
    ctx.shot('%s-training' % tag)
    out = _snapshot(ctx, tag, mark)
    out.update(css_walk=trail, start_looks=back)
    (Path(ctx.row_dir) / ('%s-%s.json' % (tag, ctx.on))).write_text(json.dumps(out, indent=1) + '\n')
    _require(reached, 'no offline/training scene within %d s' % cap_s, out)
    _require(not any('quickplay/' in c for c in out['scene_changes']), 'a quickplay scene since the row began', out)
    training = [s for s in out['stage_logs'] if '-training-' in s['name']]
    _require(training and all('connection=offline' in ' '.join(s['head']) for s in training),
             'no Training stage log with connection=offline', out)
    return {'scene_changes': out['scene_changes'], 'stage_logs': [s['head'][2] for s in training if len(s['head']) > 2],
            'quickplay_menu_hook': [e for e in out['events'] if 'a=6 ' in e]}


def qp_waiting(ctx, tag='qw', cap_s=150):
    mark = _mark(ctx)
    ctx.gate('main')
    ctx.play('to-online')
    ctx.gate('online-menu')
    ctx.play('to-quickplay')
    ctx.gate('quickplay-menu')
    ctx.shot('%s-quickplay-menu' % tag)
    tile_log = Path(ctx.instance['tile_log'])
    start = _size(tile_log)
    ctx.play('to-css')
    for _ in range(5):
        ctx.check_stop()
        time.sleep(4.0)
        if re.search(r'settled screen=css at=(\d+) hovered=\1\b', _read(tile_log, start)):
            break
        ctx.play('tile-up')
    ctx.gate('css-tile')
    ctx.play('pick')
    looks = []
    for i in range(4):
        try:
            ctx.gate('ready-banner-any')
            break
        except RuntimeError as error:
            looks.append(str(error)[:120])
            ctx.press('a', 120)
            time.sleep(1.5)
    ctx.shot('%s-ready' % tag)
    ctx.press('plus', 120)
    reached = _wait_scene(ctx, 'quickplay/training', mark, cap_s)
    time.sleep(8.0)
    ctx.shot('%s-waiting' % tag)
    out = _snapshot(ctx, tag, mark)
    out['ready_looks'] = looks
    _require(reached, 'no quickplay/training scene within %d s' % cap_s, out)
    _require(any(re.search(r'scene_hold a=4 b=0x70[0-9a-f]{5}', e) for e in out['events']),
             'no event 107 a=4 with an online mode word', out)
    practice = [s for s in out['stage_logs'] if '-training-' in s['name']]
    _require(practice and all('connection=quickplay' in ' '.join(s['head']) for s in practice),
             'no waiting-room stage log with connection=quickplay', out)
    return {'scene_changes': out['scene_changes'], 'promotion': [e for e in out['events'] if 'a=4 ' in e],
            'stage_logs': [s['head'][2] for s in practice if len(s['head']) > 2]}


def _menu_or_tab(ctx, shot):
    return (_diff(ctx, shot, 'waiting-menu.png', MENU_BOX) < 12, _diff(ctx, shot, 'leave-battle-tab.png', TAB_BOX) < 12)


def _dialog_focus(shot):
    """quickplay-cancel-fix's check: the focused (orange) button of a centred two-button dialog."""
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


def _leave_waiting(ctx, tag, mark, cap_s=60):
    """Plus until the waiting menu shows, Up to Leave Battle, A, Yes; done when Deluxe's scene leaves training."""
    began = time.monotonic()
    tries = []
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        changes, _ = _scenes(ctx, mark)
        if any(not target.endswith('/training') for target in _targets(changes)):
            return {'tries': tries, 'changes': changes}
        shot = ctx.shot('%s-look' % tag)
        menu, tab = _menu_or_tab(ctx, shot)
        if menu or tab:
            ctx.press('up', 120)
            time.sleep(0.6)
            ctx.press('a', 120)
            time.sleep(1.2)
            dialog = ctx.shot('%s-dialog' % tag)
            focus = _dialog_focus(dialog)
            if focus == 'no':
                ctx.press('right', 120)
                time.sleep(0.5)
                dialog = ctx.shot('%s-yes' % tag)
                focus = _dialog_focus(dialog)
            if focus == 'yes':
                ctx.press('a', 120)
            tries.append({'menu': menu, 'tab': tab, 'focus': focus, 'dialog': str(dialog)})
            time.sleep(4.0)
        else:
            ctx.press('plus', 100)
            time.sleep(1.2)
    raise RuntimeError('did not leave the waiting room in %d s: %s' % (cap_s, tries[-3:]))


def _back_to_main(ctx, tag, tries=12):
    """B until the main menu, looking before every press (B on the main menu would go to the title). The main menu
    keeps the last panel's focus, and the `main` reference is the focused Smash label, so a frame with the menus'
    icon column (refs/main-sidebar.png) that is neither the Online nor the Quickplay menu gets Left, Up (from any
    panel: Smash) instead of B. Done when the Smash panel has the focus, the state the menu macros start from."""
    seen = []
    for i in range(tries):
        ctx.check_stop()
        shot = ctx.shot('%s-back-%d' % (tag, i))
        main = _diff(ctx, shot, 'main.png', MAIN_ROI)
        side = _diff(ctx, shot, 'main-sidebar.png', SIDEBAR_ROI)
        online = _diff(ctx, shot, 'online-menu-roi.png', HEADER_ROI)
        quick = _diff(ctx, shot, 'quickplay-menu-roi.png', HEADER_ROI)
        games = _diff(ctx, shot, 'gm-header.png', HEADER_ROI)
        seen.append({'main': round(main, 1), 'sidebar': round(side, 1), 'online': round(online, 1),
                     'quickplay': round(quick, 1), 'games_and_more': round(games, 1)})
        if main <= 8:
            return seen
        # A menu header within 30 is that menu (qpt-y1: the Online header read 17-19 on the 2f315fc4 build against
        # the dun runs' reference; the main menu's ticker reads over 100 against every header).
        if side <= 10 and online > 30 and quick > 30 and games > 30:
            ctx.press('left', 120)
            time.sleep(0.6)
            ctx.press('up', 120)
            time.sleep(1.0)
            continue
        # Held: a character select is left only by holding B (qpt-y1, 14:04Z: taps only took the token back).
        ctx.press('b', 1300)
        time.sleep(2.0)
    raise RuntimeError('main menu with Smash focused not reached in %d looks: %s' % (tries, seen[-4:]))


def _scene_now(ctx):
    """The last scene hang-trace.log names (a change or a heartbeat)."""
    text = _read(_dir(ctx) / 'hang-trace.log')
    found = [(m.start(), m.group(2)) for m in SCENE_CHANGE.finditer(text)]
    found += [(m.start(), m.group(1)) for m in HEARTBEAT_SCENE.finditer(text)]
    return max(found)[1] if found else None


def exit_training(ctx, tag='ex'):
    """Leave Training for the main menu: Plus (the pause menu), the quit-focus (Up) and quit-confirm (A) frames of the
    integration lane's quit_to_css, then B back through the character select, the stage select and Games & More."""
    mark = _mark(ctx)
    ctx.press('plus', 150)
    time.sleep(1.4)
    ctx.shot('%s-pause' % tag)
    ctx.play('quit-focus')
    time.sleep(0.8)
    ctx.shot('%s-quit-focus' % tag)
    ctx.play('quit-confirm')
    time.sleep(5.0)
    ctx.shot('%s-after-quit' % tag)
    back = _back_to_main(ctx, tag, tries=16)
    changes, _ = _scenes(ctx, mark)
    out = {'back': back, 'scene_changes': changes}
    (Path(ctx.row_dir) / ('%s-%s.json' % (tag, ctx.on))).write_text(json.dumps(out, indent=1) + '\n')
    return {'looks': len(back), 'scene_changes': changes}


def training_after_qp(ctx, tag='tq', cap_s=180):
    now = _scene_now(ctx)
    if not (now or '').startswith('quickplay/'):
        raise RuntimeError('precondition: no Quickplay connection to keep (hang-trace.log scene %s)' % now)
    mark = _mark(ctx)
    # A retry after the leave (qpt-y1: the first try left the waiting room, then stopped on the Online menu) starts
    # from the menus.
    left = _leave_waiting(ctx, '%s-leave' % tag, mark) if now == 'quickplay/training' else {'skipped': now}
    back = _back_to_main(ctx, tag)
    time.sleep(2.0)
    kept = [_decode(e) for e in _events(ctx) if e[0] > mark['seq'] and e[2] == 107 and e[3] == 1]
    mark2 = _mark(ctx)
    trail = _to_training(ctx, tag)
    reached = _wait_scene(ctx, 'offline/training', mark2, cap_s)
    time.sleep(6.0)
    ctx.shot('%s-training' % tag)
    out = _snapshot(ctx, tag, mark)
    out.update(left=left, back=back, kept=kept, css_walk=trail)
    (Path(ctx.row_dir) / ('%s-%s.json' % (tag, ctx.on))).write_text(json.dumps(out, indent=1) + '\n')
    _require(reached, 'no offline/training scene within %d s' % cap_s, out)
    _require(any(re.search(r'scene_hold a=5 b=0x3$', e) for e in out['events']),
             'no event 107 a=5 b=3 (Training demoting a kept Quickplay)', out)
    training = [s for s in _new_stage_logs(ctx, mark2) if '-training-' in s['name']]
    _require(training and all('connection=offline' in ' '.join(s['head']) for s in training),
             'no Training stage log with connection=offline', out)
    return {'kept': kept, 'demotion': [e for e in out['events'] if 'a=5 ' in e],
            'scene_changes': out['scene_changes'], 'stage_logs': [s['head'][2] for s in training if len(s['head']) > 2]}


def final_evidence(ctx):
    out = _snapshot(ctx, 'final', {'hang': 0, 'seq': 0, 'matches': [], 't': 0})
    folder = Path(out['folder']) / 'matches'
    folder.mkdir(exist_ok=True)
    for name in _matches(ctx) + ['index.txt']:
        src = _dir(ctx) / 'matches' / name
        if src.is_file():
            shutil.copy2(src, folder / name)
    for name in ('renderer-status.txt', 'timing-status.txt', 'config.ini'):
        src = _dir(ctx) / name
        if src.is_file():
            shutil.copy2(src, Path(out['folder']) / name)
    return {'scene_changes': out['scene_changes'], 'stage_logs': len(out['stage_logs'])}
