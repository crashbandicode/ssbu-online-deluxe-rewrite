"""Deluxe scene-online Eden hooks (2026-11-13): Deluxe's scene through one LAN Quickplay session.

The GDB stub is on (gdb_rc11.py, no holding helper), so Eden's guest tracing is off: the checks read Deluxe's own
files (hang-trace.log, events.log, matches/) through qp_steps.py's readers and scene_table.py.

- `mark`: before the search, the sizes and stage-log list the evidence row diffs against (scene-mark-<key>.json).
- `evidence`: after the battle and the results, Deluxe's files copied into the row folder and the scene per phase
  (scene_table.analyse) saved as scene-<tag>-<key>.json. `expect='record'` passes when an online battle was seen;
  `expect='online'` also needs every battle to read quickplay through its run with the online bit never off
  (scene_table.battle_online).
"""
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import qp_steps  # noqa: E402
import scene_table  # noqa: E402


def mark(ctx):
    out = qp_steps._mark(ctx)
    (Path(ctx.row_dir) / ('scene-mark-%s.json' % ctx.on)).write_text(json.dumps(out, indent=1) + '\n')
    return {'hang': out['hang'], 'seq': out['seq'], 'stage_logs': len(out['matches'])}


def _find_mark(ctx):
    for path in sorted(Path(ctx.row_dir).parent.glob('*/scene-mark-%s.json' % ctx.on)):
        return json.loads(path.read_text())
    return {'hang': 0, 'seq': 0, 'matches': [], 't': 0}


def evidence(ctx, tag='m1', expect='record'):
    snap = qp_steps._snapshot(ctx, tag, _find_mark(ctx))
    folder = Path(snap['folder'])
    logs = {s['name']: '\n'.join(s['head']) for s in snap['stage_logs']}
    result = scene_table.analyse((folder / 'hang-trace.log').read_text(errors='replace'),
                                 (folder / 'events.log').read_text(errors='replace'), logs)
    ok, why = scene_table.battle_online(result)
    result['battle_online'] = {'ok': ok, 'why': why}
    (Path(ctx.row_dir) / ('scene-%s-%s.json' % (tag, ctx.on))).write_text(json.dumps(result, indent=1) + '\n')
    ctx.log({'scene': tag, 'on': ctx.on, 'battle_online': result['battle_online'],
             'battles': len(result['battles'])})
    if not result['battles']:
        raise RuntimeError('%s: no online battle stage in events.log' % ctx.on)
    if expect == 'online' and not ok:
        raise RuntimeError('%s: %s' % (ctx.on, why))
    return {'battles': len(result['battles']), 'battle_online': ok, 'why': why}
