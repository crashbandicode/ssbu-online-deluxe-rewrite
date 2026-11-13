"""One Markdown table row per run instance, from scene_table.analyse: the scene per phase.

usage: phase_rows.py <label>=<deluxe dir> ...   (each dir holds hang-trace.log, events.log and optionally matches/)

Columns: run, waiting room, battle at load, battle (heartbeats), stale demotion (107 a=2), online bit (event 30),
stage log connection=, results.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import scene_table  # noqa: E402


def read(path):
    try:
        return open(path, encoding='utf-8', errors='replace').read()
    except OSError:
        return ''


def first(path, names):
    for name in names:
        if os.path.isfile(os.path.join(path, name)):
            return os.path.join(path, name)
    return os.path.join(path, names[0])


def row(label, folder):
    hang = read(first(folder, ['hang-trace.log', 'ssbu_online_deluxe_rewrite-hang-trace.log']))
    events = read(first(folder, ['events.log', 'ssbu_online_deluxe_rewrite-events.log']))
    logs = {}
    matches = os.path.join(folder, 'matches')
    if os.path.isdir(matches):
        for name in sorted(os.listdir(matches)):
            if name.endswith('.log'):
                logs[name] = '\n'.join(read(os.path.join(matches, name)).splitlines()[:3])
    r = scene_table.analyse(hang, events, logs)
    build = (events.splitlines() or [''])[0].replace('Deluxe rewrite ', '')
    w = r['waiting'] or {}
    cells = [label, build, w.get('scene') or '-']
    for b in r['battles']:
        beats = ', '.join('%s x%d' % kv for kv in b['heartbeats'].items())
        cells += ['%s %s' % (b['stage'], b['scene_at_load']), beats, ' '.join(h for h in b['scene_holds_107']
                                                                              if h.startswith('a=2')) or 'none',
                  '; '.join(b['online_context_30']) or 'no change']
        break
    else:
        cells += ['-'] * 4
    stage = [c for n, c in r.get('stage_logs', {}).items() if '-singles-' in n or '-doubles-' in n]
    cells.append(', '.join(c or '?' for c in stage) or '-')
    cells.append((r.get('results') or {}).get('scene') or '-')
    extra = len(r['battles']) - 1
    if extra:
        cells[0] += ' (+%d battle%s)' % (extra, 's' if extra > 1 else '')
    ok, why = scene_table.battle_online(r)
    cells.append('yes' if ok else 'no')
    return '| ' + ' | '.join(cells) + ' |'


def main(argv):
    print('| Run | Build | Waiting room | Battle at load | Battle heartbeats | Stale demotion | Online bit (30) '
          '| Stage log | Results | Battle online |')
    print('| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |')
    for arg in argv:
        label, folder = arg.split('=', 1)
        print(row(label, folder))


if __name__ == '__main__':
    main(sys.argv[1:])
