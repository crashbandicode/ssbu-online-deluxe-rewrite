#!/usr/bin/env python3
"""provenance.json for eden/qp-offline-training: the sha256 of every file in a run's rows folder (the files committed
here and the reused ones: gdb_rc11.py, gdbdump.py, segments and refs, which are game screens and stay out of the
repository), the run's pins and the runner it ran on.
Usage: provenance_qpt.py <run folder with config.json and rows/> <out.json>
"""
import hashlib
import json
import sys
from pathlib import Path


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main(argv):
    run, out = Path(argv[1]), Path(argv[2])
    rows = run / 'rows'
    config = json.loads((run / 'config.json').read_text())
    data = {
        'run': config['label'],
        'host': config['host'],
        'runner': config['runner'],
        'rows_folder': {str(p.relative_to(rows)): sha(p) for p in sorted(rows.rglob('*'))
                        if p.is_file() and '__pycache__' not in p.parts},
        'reused_from': {
            'gdb_rc11.py, gdbdump.py': 'the integration lane rows (/tmp/opencode/imgui-parity/stage-a/rows-hang, '
                                       'staged by make_hang.py)',
            'segments/to-online, to-quickplay, to-css, tile-up, pick': 'eden/unicode-label/segments (dun-x6, ims-x1)',
            'segments/quit-focus, quit-confirm': 'the integration lane rows',
            'segments/attract-cancel, main-stage, stage-css, enter-training; refs/stage.png, css.png':
                'the playbook library (tools/eden-runner/library)',
            'refs/main.png, online-menu-roi.png, quickplay-menu-roi.png, ready-banner-roi.png, waiting-menu.png, '
            'leave-battle-tab.png': 'ims-x1 rows (quickplay-cancel-fix)',
            'refs/main-sidebar.png': 'make/make_sidebar_ref.py from an ims-x1 main-menu frame',
            'refs/gm-header.png': 'make/make_header_ref.py from a qpt-y1 Games & More frame',
        },
        'instances': {item['key']: {'app': item['app'], 'plugins': item['plugins'], 'files': item.get('files', [])}
                      for item in config['instances']},
    }
    out.write_text(json.dumps(data, indent=1, ensure_ascii=False) + '\n')
    print(out, len(data['rows_folder']), 'files')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
