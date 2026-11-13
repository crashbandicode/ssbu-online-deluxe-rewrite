"""Release-candidate pair: Eden step hooks (tools/eden-runner StepContext) beside deluxe_steps.py and ims_steps.py.

The GDB stub is on from before game.load, so Eden's trace points (they need the stub off) are not armed: the
FindGlyph and pad-poll traces of the imgui-parity rows are replaced by the font lines and the frames below.
"""
import json
import re
import time
from pathlib import Path

ROW = re.compile(r'^(\d+) (\d+) (\d+) 0x([0-9a-f]+) 0x([0-9a-f]+)\s*$', re.M)


def _save(ctx, name, data):
    path = Path(ctx.row_dir) / f'{name}.json'
    path.write_text(json.dumps(data, indent=1, ensure_ascii=False) + '\n')
    ctx.log({name: data if len(json.dumps(data)) < 2000 else str(path)})
    return data


def _fresh(ctx, cap_s=15.0):
    path = Path(ctx.instance['deluxe_status'])
    start, began = time.time(), time.monotonic()
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        if path.exists() and path.stat().st_mtime > start:
            time.sleep(0.3)
            return path.read_text(errors='replace'), True
        time.sleep(0.5)
    return (path.read_text(errors='replace') if path.exists() else ''), False


def status(ctx, tag='status', plugins=21, host='external'):
    """A fresh renderer-status.txt: `hook_lock_skipped=<n>` present (release builds write it), and Deluxe drawing
    through the standalone (`host='external'`: `imgui_stats=external`, the host-clients line with index=paged and
    the plugin count) or through its built-in ImGui (`host='embedded'`, as on the card since 0.10.6)."""
    text, fresh = _fresh(ctx)
    Path(ctx.row_dir, f'renderer-status-{tag}-{ctx.on}.txt').write_text(text)
    skipped = re.search(r'^hook_lock_skipped=(\d+)\s*$', text, re.M)
    stats = re.search(r'^imgui_stats=(\S+)\s*$', text, re.M)
    clients = re.search(r'^imgui_host_clients=\d+ declared=\d+ declarations=\d+ plugins=(\d+) index=(\w+)\s*$',
                        text, re.M)
    out = _save(ctx, f'{tag}-{ctx.on}', {
        'fresh': fresh, 'hook_lock_skipped': int(skipped.group(1)) if skipped else None,
        'imgui_stats': stats.group(1) if stats else None, 'host_clients': clients.group(0) if clients else None})
    problems = []
    if not fresh:
        problems.append('renderer-status.txt not rewritten within 15 s')
    if skipped is None:
        problems.append('no hook_lock_skipped= line')
    if out['imgui_stats'] != host:
        problems.append(f'imgui_stats={out["imgui_stats"]}')
    if host == 'external' and (not clients or clients.group(2) != 'paged' or int(clients.group(1)) != plugins):
        problems.append(f'host clients line: {out["host_clients"]}')
    if problems:
        raise RuntimeError(f'{ctx.on}: {problems}')
    return out


def nav_frames(ctx, settle_s=2.0):
    """L3 + R3 in one frame, then D-pad Down and Up, with frames before and after and Deluxe's view-change rows
    (code 23) across it. No trace (the GDB stub is on): records the frames and fails only on a view change, which
    would mean the chord reached Deluxe's window."""
    events = Path(ctx.instance['deluxe_events'])

    def rows():
        text = events.read_text(errors='replace') if events.exists() else ''
        return [(int(m[1]), int(m[3])) for m in ROW.finditer(text)]

    after = max((r[0] for r in rows()), default=-1)
    before = ctx.shot('nav-before')
    ctx.play('l3-r3')
    time.sleep(1.0)
    ctx.play('dpad-down')
    time.sleep(0.6)
    ctx.play('dpad-up')
    time.sleep(settle_s)
    shot = ctx.shot('nav-after')
    views = [r for r in rows() if r[0] > after and r[1] == 23]
    out = _save(ctx, f'nav-{ctx.on}', {'before': str(before), 'after': str(shot), 'view_changes': views})
    if views:
        raise RuntimeError(f'{ctx.on}: L3 + R3 / D-pad changed Deluxe\'s view: {views}')
    return out
