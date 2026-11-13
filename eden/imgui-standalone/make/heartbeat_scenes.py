#!/usr/bin/env python3
"""Deluxe hang-trace.log heartbeats per scene, verbatim: for each run of consecutive heartbeats with the same scene=
value, the first and last heartbeat's t, scene, mem_used and mem_free, plus the count. No arithmetic beyond
grouping; the values are the log's. Usage: heartbeat_scenes.py <hang-trace.log> [...]"""
import re
import sys

BEAT = re.compile(r'^\[Heartbeat\] t=([\d.]+) .*?scene=(\S+) ready=(\d) css=(\d) .*?mem_used=(\d+) mem_total=(\d+) '
                  r'mem_free=(\d+)')


def runs(path):
    out, current = [], None
    for line in open(path, encoding='utf-8', errors='replace'):
        m = BEAT.match(line)
        if not m:
            continue
        t, scene, ready, css, used, total, free = m.groups()
        key = (scene, css)
        if current is None or current['key'] != key:
            current = {'key': key, 'first': (t, used, free), 'last': (t, used, free), 'beats': 0}
            out.append(current)
        current['last'] = (t, used, free)
        current['beats'] += 1
    return out


def main(argv):
    for path in argv[1:]:
        print(f'== {path}')
        build = next((l.strip() for l in open(path, encoding='utf-8', errors='replace') if l.startswith('build=')), '')
        print(build)
        for r in runs(path):
            (scene, css), (t0, u0, f0), (t1, u1, f1) = r['key'], r['first'], r['last']
            print(f'scene={scene} css={css} beats={r["beats"]} first t={t0} mem_used={u0} mem_free={f0} '
                  f'last t={t1} mem_used={u1} mem_free={f1}')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
