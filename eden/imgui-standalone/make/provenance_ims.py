#!/usr/bin/env python3
"""provenance.json for eden/imgui-standalone: the sha256 of every file in the run's rows folder (the files committed
here, and the ones it reused from dun-x6's rows folder: steps.py, deluxe_steps.py, the other segments and the refs,
which are game screens and stay out of the repository), the run's pins and the runner it ran on.
Usage: provenance_ims.py <run folder> <out.json>
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
        'run': run.name,
        'runner': config['runner'],
        'rows_folder': {str(p.relative_to(rows)): sha(p) for p in sorted(rows.rglob('*'))
                        if p.is_file() and '__pycache__' not in p.parts},
        'instances': {item['key']: {'app': item['app'], 'plugins': item['plugins'], 'files': item.get('files', []),
                                    'symbols': item.get('trace', {}).get('symbols', [])}
                      for item in config['instances']},
    }
    out.write_text(json.dumps(data, indent=1, ensure_ascii=False) + '\n')
    print(out, len(data['rows_folder']), 'files')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
