#!/usr/bin/env python3
"""Base configs for the 0.10.4 boots, with make_config2's helpers: eden/config-dun4-<host>.json (a 0.10.3 bc22c80,
b 0.10.4 21c0009) and eden/config-dun5-<host>.json (a 0.10.4, b 0.10.4 diag + label feed).
Usage: make_config3.py <xps|surface>"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import make_config2 as base  # noqa: E402

W = base.W
base.BUILDS = {'card': W / 'builds' / 'after-bc22c80', 'after': W / 'builds' / 'after-21c0009',
               'diag': W / 'builds' / 'diag-21c0009'}


def main(argv):
    host = argv[1]
    for name, rows, a, b, b_diag, note in (
            ('dun4', 'rows-dun4', 'card', 'after', False, 'a Deluxe 0.10.3 bc22c80, b Deluxe 0.10.4 21c0009'),
            ('dun5', 'rows-dun5', 'after', 'diag', True,
             'a Deluxe 0.10.4 21c0009, b Deluxe 0.10.4 21c0009 diag build with the label feed')):
        cfg = base.config(host, f'{name}-base', a, b, b_diag, note)
        cfg['authorization'] = cfg['authorization'].replace(
            '0.10.3 rework: operator via primary 11:50Z/11:58Z 11-10',
            '0.10.4 paged codepoint index: operator via primary 17:51Z 11-10')
        cfg['rows'] = base.pin(W / 'eden' / rows / 'rows.json')
        out = W / 'eden' / f'config-{name}-{host}.json'
        out.write_text(json.dumps(cfg, indent=2, ensure_ascii=False) + '\n')
        print(out)


if __name__ == '__main__':
    sys.exit(main(sys.argv))
