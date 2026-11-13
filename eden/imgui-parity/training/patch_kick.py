"""imgui-parity copies of gdb_rc11.py: on the Surface with the GDB stub on, the guest first presented about 235 s after
boot in imp-b2 (Deluxe heartbeat present=192 at t=240.7), after the direct kick's 90 s + 3 x 75 s had run out. The
nohold path now waits up to `draw_cap` (rows.json passes 300) before its retries. Nothing else changes."""
import json
import sys
from pathlib import Path

OLD = "        drawing, st = _drawing(ctx, 90.0)\n"
NEW = "        drawing, st = _drawing(ctx, max(90.0, draw_cap))\n"

for folder in map(Path, sys.argv[1:]):
    path = folder / 'gdb_rc11.py'
    text = path.read_text()
    if NEW not in text:
        if text.count(OLD) != 1:
            raise SystemExit(f'{path}: unexpected gdb_kick')
        path.write_text(text.replace(OLD, NEW))
    plan_path = folder / 'rows.json'
    plan = json.loads(plan_path.read_text())
    for row in plan['rows']:
        for step in row['steps']:
            if step.get('run') == 'gdb_rc11.py:gdb_kick':
                step['args'] = {'draw_cap': 300.0}
    plan_path.write_text(json.dumps(plan, indent=1, ensure_ascii=False) + '\n')
    print(folder, 'patched')
