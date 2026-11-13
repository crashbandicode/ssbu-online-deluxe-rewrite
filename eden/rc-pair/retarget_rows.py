#!/usr/bin/env python3
"""A release-candidate rows.json (make_rows_rc.py) for another Deluxe build and, optionally, the built-in ImGui.

usage: retarget_rows.py <rows.json> <out rows.json> <build id prefix> <external|embedded> <about>

The build gate matches `Deluxe rewrite <build id prefix>`. With `embedded` (the card's set since 0.10.6: no
standalone libimgui_smash.nro), the external-host gates go, and the host checks expect `imgui_stats=embedded`
(ims_steps.host_status, rc_steps.status). The rest of the rows is unchanged.
"""
import json
import re
import sys
from pathlib import Path

EXTERNAL_GATES = ('deluxe-external', 'external-accepted')


def main():
    src, out, build, host, about = sys.argv[1:6]
    if host not in ('external', 'embedded'):
        raise SystemExit('host is external or embedded')
    spec = json.loads(Path(src).read_text())
    spec['gates']['deluxe-build']['pattern'] = 'Deluxe rewrite ' + re.escape(build)
    spec['about'] = about
    changed = 0
    for row in spec['rows']:
        steps = []
        for step in row['steps']:
            if host == 'embedded':
                if step.get('gate') in EXTERNAL_GATES:
                    changed += 1
                    continue
                if step.get('run') == 'ims_steps.py:host_status':
                    step.setdefault('args', {})['expect'] = 'embedded'
                    changed += 1
                if step.get('run') == 'rc_steps.py:status':
                    step.setdefault('args', {})['host'] = 'embedded'
                    changed += 1
            steps.append(step)
        row['steps'] = steps
        if host == 'embedded' and 'expect' in row:
            row['expect'] = row['expect'].replace('through the standalone (external', 'through its built-in ImGui (embedded')
    if host == 'embedded':
        for gate in EXTERNAL_GATES:
            spec['gates'].pop(gate, None)
    Path(out).write_text(json.dumps(spec, indent=1, ensure_ascii=False) + '\n')
    print(out, 'steps changed', changed)


if __name__ == '__main__':
    main()
