"""imgui-smash 1.1.0 standalone and Deluxe 0.10.5's re-pinned built-in ImGui: Eden step hooks (tools/eden-runner
StepContext). The label rows reuse deluxe_steps.py (its FindGlyph trace and font lines) unchanged.

a runs Deluxe 0.10.4 (release NRO 70a2931e) with the standalone libimgui_smash.nro 1.1.0, so Deluxe draws in the
standalone's ImGui (external host). b runs the Deluxe build that pins the same archive, without a standalone
(embedded host). Deluxe's events.log rows are `<sequence> <tick> <code> 0x<a> 0x<b>`: code 16 a=5 b=19263 is the
external host accepted (src/runtime/overlay_host.rs), 23 a=previous view b=new view (src/overlay.rs View: 0 Hidden,
1 Settings, 2 Performance, 3 Debug, 4 Plugins, 5 a plugin's own page, 6 Window Options), 8 a setting changed with
b = the input buffer (Auto is -1).
"""
import json
import re
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROW = re.compile(r'^(\d+) (\d+) (\d+) 0x([0-9a-f]+) 0x([0-9a-f]+)\s*$', re.M)
VIEWS = {0: 'Hidden', 1: 'Settings', 2: 'Performance', 3: 'Debug', 4: 'Plugins', 5: 'PluginPage', 6: 'Window'}
AUTO = 0xFFFFFFFFFFFFFFFF
# The Opp line sits in the window's top-left rows (dun-x6 frames, 1280x674): crop it and scale it up 3x.
STRIP = (60, 0, 470, 28)
UPDATE_PAD = '_ZN11InputHelper14updatePadStateEv'
UPDATE_GAMEPAD = '_ZN15ImguiNvnBackend13updateGamepadER7ImGuiIO'


def _save(ctx, name, data):
    path = Path(ctx.row_dir) / f'{name}.json'
    path.write_text(json.dumps(data, indent=1, ensure_ascii=False) + '\n')
    ctx.log({name: data if len(json.dumps(data)) < 2000 else str(path)})
    return data


def _rows(ctx, code=None):
    path = ctx.instance.get('deluxe_events')
    text = Path(path).read_text(errors='replace') if path and Path(path).exists() else ''
    out = []
    for m in ROW.finditer(text):
        row = (int(m[1]), int(m[3]), int(m[4], 16), int(m[5], 16))
        if code is None or row[1] == code:
            out.append(row)
    return out


def _last_sequence(ctx):
    return max((row[0] for row in _rows(ctx)), default=-1)


def _wait_rows(ctx, code, after, cap_s=8.0):
    began = time.monotonic()
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        new = [row for row in _rows(ctx, code) if row[0] > after]
        if new:
            return new
        time.sleep(0.25)
    return []


def _fresh_status(ctx, cap_s=15.0):
    path = Path(ctx.instance['deluxe_status'])
    start, began = time.time(), time.monotonic()
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        if path.exists() and path.stat().st_mtime > start:
            time.sleep(0.3)
            return path, path.read_text(errors='replace'), True
        time.sleep(0.5)
    return path, path.read_text(errors='replace') if path.exists() else '', False


def host_status(ctx, expect, tag='host'):
    """Which ImGui Deluxe uses: renderer-status.txt `imgui_stats=` (rewritten every 5 s) and the code 16 a=5
    rows. `not_embedded`: the external host, accepted with b=19263, no imgui_font_* lines (they describe only the
    embedded host). `embedded`: no external acceptance, and the imgui_font_* lines are there."""
    path, text, fresh = _fresh_status(ctx)
    stats = re.search(r'^imgui_stats=(\S+)\s*$', text, re.M)
    fonts = [line for line in text.splitlines() if line.startswith('imgui_font_')]
    external = [row for row in _rows(ctx, 16) if row[2] == 5]
    install = Path(ctx.instance['deluxe_install']).read_text(errors='replace')
    phases = re.findall(r'phase=(overlay_host_\w+)', install)
    out = _save(ctx, f'{tag}-{ctx.on}', {
        'file': str(path), 'fresh': fresh, 'imgui_stats': stats.group(1) if stats else None,
        'font_lines': fonts, 'code16_a5': [list(row) for row in external], 'install_phases': phases})
    problems = []
    if out['imgui_stats'] != expect:
        problems.append(f'imgui_stats={out["imgui_stats"]}, not {expect}')
    if expect == 'external':
        clients = re.search(r'^imgui_host_clients=(\d+) declared=(\d+) .*index=(\w+)\s*$', text, re.M)
        out['host_clients'] = clients.group(0) if clients else None
        _save(ctx, f'{tag}-{ctx.on}', out)
        if not any(row[3] == 19263 for row in external):
            problems.append('no code 16 a=5 b=19263')
        if 'overlay_host_external_ready' not in phases:
            problems.append('install.log has no overlay_host_external_ready')
        if not any(line.startswith('imgui_font_static_glyphs=') for line in fonts):
            problems.append('no imgui_font_* lines from the external host')
        if not clients or clients.group(3) != 'paged':
            problems.append(f'host clients line: {out["host_clients"]}')
    elif expect == 'not_embedded':
        if not any(row[3] == 19263 for row in external):
            problems.append('no code 16 a=5 b=19263')
        if 'overlay_host_external_ready' not in phases:
            problems.append('install.log has no overlay_host_external_ready')
        if fonts:
            problems.append('font lines on the external host')
    else:
        if external:
            problems.append('code 16 a=5 on the embedded host')
        if 'overlay_host_ready' not in phases:
            problems.append('install.log has no overlay_host_ready')
        if not any(line.startswith('imgui_font_static_glyphs=') for line in fonts):
            problems.append('no imgui_font_* lines')
    if problems:
        raise RuntimeError(f'{ctx.on}: {problems}')
    return {'imgui_stats': out['imgui_stats'], 'code16_a5': out['code16_a5'][:2], 'phases': phases[-2:]}


def crop_strip(ctx, tag='opp-line'):
    """A frame and its top-left window rows (the Opp line) cropped and scaled 3x, for the evidence."""
    from PIL import Image
    shot = ctx.shot(tag)
    with Image.open(shot) as image:
        frame = image.convert('RGB')
        scale_x, scale_y = frame.size[0] / 1280, frame.size[1] / 674
        box = (round(STRIP[0] * scale_x), round(STRIP[1] * scale_y), round(STRIP[2] * scale_x), round(STRIP[3] * scale_y))
        crop = frame.crop(box)
        crop = crop.resize((crop.size[0] * 3, crop.size[1] * 3), Image.Resampling.LANCZOS)
    out = Path(ctx.row_dir) / f'{ctx.on}-{tag}-crop.png'
    crop.save(out)
    return _save(ctx, f'{tag}-{ctx.on}', {'frame': str(shot), 'crop': str(out), 'box': box, 'frame_size': list(frame.size)})


def menu_cycle(ctx, chords=8, cap_s=8.0):
    """L+R+Down (segment open-deluxe) until the window hides, then once more so Settings is open: every chord must
    log one view change (code 23). Passes when the cycle showed Settings, Performance and Window Options and ended
    Hidden -> Settings; a frame after each chord."""
    steps = []
    reopened = False
    for i in range(chords + 1):
        ctx.check_stop()
        after = _last_sequence(ctx)
        ctx.play('open-deluxe')
        new = _wait_rows(ctx, 23, after, cap_s)
        time.sleep(0.8)
        shot = ctx.shot(f'cycle-{i}')
        if not new:
            _save(ctx, f'cycle-{ctx.on}', {'steps': steps})
            raise RuntimeError(f'{ctx.on}: chord {i}: no view change logged within {cap_s} s')
        previous, view = new[-1][2], new[-1][3]
        steps.append({'chord': i, 'from': VIEWS.get(previous, previous), 'to': VIEWS.get(view, view), 'shot': str(shot)})
        if reopened:
            break
        if view == 0:
            reopened = True
    seen = {step['to'] for step in steps}
    out = _save(ctx, f'cycle-{ctx.on}', {'steps': steps, 'seen': sorted(seen)})
    missing = {'Settings', 'Performance', 'Window', 'Hidden'} - seen
    if missing or steps[-1]['to'] != 'Settings' or steps[-1]['from'] != 'Hidden':
        raise RuntimeError(f'{ctx.on}: cycle {[(s["from"], s["to"]) for s in steps]}; missing {sorted(missing)}')
    return {'cycle': [f'{s["from"]}->{s["to"]}' for s in out['steps']]}


# Where the window's rows are: top left in matches and most menus, top right on the character select (ims-x1
# n1-menu-b, first try: the left box changed by 0.64 on a Down, the right one by 1.5; 0.07 with no input).
WINDOW_BOXES = {'left': [0, 0, 480, 140], 'right': [980, 0, 1240, 120]}


def _window_diff(ctx, a, b):
    return max(round(ctx.runner.roi_diff(str(a), str(b), box), 2) for box in WINDOW_BOXES.values())


def settings_edit(ctx, cap_s=6.0):
    """On Settings, freshly opened (selection on You): Down moves the selection (frames differ in the window
    rows), then ZR+Right on NetLatency logs code 8 with b=0 (Auto -> 0) and ZR+Left logs b=Auto again. When the
    first Down lands on Opp (a peer still listed), ZR+Right changes nothing and a second Down reaches NetLatency.
    Up moves the selection back."""
    out = {'downs': []}
    before = ctx.shot('settings-before')
    for attempt in range(2):
        ctx.play('dpad-down')
        time.sleep(0.8)
        moved = ctx.shot(f'settings-down-{attempt}')
        out['downs'].append({'shot': str(moved), 'diff': _window_diff(ctx, moved, before)})
        before = moved
        after = _last_sequence(ctx)
        ctx.play('zr-right')
        right = _wait_rows(ctx, 8, after, cap_s)
        if right:
            break
    else:
        _save(ctx, f'settings-{ctx.on}', out)
        raise RuntimeError(f'{ctx.on}: ZR+Right changed no setting on two rows')
    time.sleep(0.8)
    out['right'] = {'rows': [list(r) for r in right], 'shot': str(ctx.shot('settings-zr-right'))}
    after = _last_sequence(ctx)
    ctx.play('zr-left')
    left = _wait_rows(ctx, 8, after, cap_s)
    time.sleep(0.8)
    out['left'] = {'rows': [list(r) for r in left], 'shot': str(ctx.shot('settings-zr-left'))}
    ctx.play('dpad-up')
    time.sleep(0.8)
    up = ctx.shot('settings-up')
    out['up'] = {'shot': str(up), 'diff': _window_diff(ctx, up, before)}
    _save(ctx, f'settings-{ctx.on}', out)
    problems = []
    if not right or right[-1][3] != 0:
        problems.append(f'ZR+Right: {right}')
    if not left or left[-1][3] != AUTO:
        problems.append(f'ZR+Left did not restore Auto: {left}')
    if out['downs'][0]['diff'] < 1.0 or out['up']['diff'] < 1.0:
        problems.append(f'selection frames did not change: downs {out["downs"]}, up {out["up"]["diff"]}')
    if problems:
        raise RuntimeError(f'{ctx.on}: {problems}')
    return {'right_b': hex(right[-1][3]), 'left_b': hex(left[-1][3]), 'downs': len(out['downs'])}


def _records(path, start):
    out = []
    try:
        with open(path, 'rb') as f:
            f.seek(start)
            text = f.read().decode(errors='replace')
    except OSError:
        return out
    for line in text.splitlines():
        try:
            out.append(json.loads(line))
        except ValueError:
            continue
    return out


def nav_chord(ctx, module, settle_s=3.0):
    """L3 + R3 in one frame, then D-pad Down and Up. Two trace points on `module`: InputHelper::updatePadState
    (the backend's pad poll, every frame; up to 5 entries) and ImguiNvnBackend::updateGamepad (called only while
    gamepad navigation is on; up to 5 entries). Passes when the poll ran and updateGamepad never did: the chord left
    navigation off."""
    trace = ctx.instance.get('trace_jsonl')
    start = Path(trace).stat().st_size if trace and Path(trace).exists() else 0
    points = {}
    for name in (UPDATE_PAD, UPDATE_GAMEPAD):
        got = ctx.call('trace.arm', {'target': f'{module}!{name}', 'registers': ['x0'], 'max_hits': 5,
                                     'rate_limit': 100})
        points[name] = str(got.get('id') if isinstance(got, dict) else got)
    time.sleep(1.0)
    ctx.play('l3-r3')
    time.sleep(1.0)
    ctx.play('dpad-down')
    time.sleep(0.6)
    ctx.play('dpad-up')
    time.sleep(settle_s)
    shot = ctx.shot('nav-chord-after')
    listed = ctx.call('trace.list', {})
    for pid in points.values():
        try:
            ctx.call('trace.disarm', {'id': pid})
        except RuntimeError as error:
            ctx.log({'disarm_error': repr(error)})
    records = _records(trace, start)
    hits = {name: sum(1 for r in records if str(r.get('point_id')) == pid and r.get('event') == 'entry')
            for name, pid in points.items()}
    out = _save(ctx, f'nav-chord-{ctx.on}', {'module': module, 'points': points, 'entries': hits, 'shot': str(shot),
                                              'trace_list': listed})
    if hits[UPDATE_PAD] < 1:
        raise RuntimeError(f'{ctx.on}: the pad poll was never traced: {out}')
    if hits[UPDATE_GAMEPAD]:
        raise RuntimeError(f'{ctx.on}: updateGamepad ran after L3 + R3: navigation turned on')
    return {'entries': hits}
