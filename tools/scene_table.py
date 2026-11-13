"""Deluxe's scene per phase of one LAN Quickplay session, from Deluxe's own files.

usage: scene_table.py <hang-trace.log> <events.log> [<stage log>...] [--json]

Phases: the waiting room (the online practice stage), the battle (the first Singles/Doubles stage with online scene
words after it) at its load, through it (every hang-trace heartbeat and scene change until the next stage load) and
the results (the next stage load). For the battle it also lists the stale-check demotions (event 107 a=2), the
online-context changes (event 30, Deluxe's exported online bit) with seconds after the battle's stage load, and the
`connection=` of the battle's stage log. Event and hang-trace times are aligned on the heartbeats' `tick=`.
"""
import json
import re
import sys

TICKS = 19_200_000
EVENT = re.compile(r'^(\d+) (\d+) (\d+) (0x[0-9a-f]+) (0x[0-9a-f]+)\s*$')
BEAT = re.compile(r'^\[Heartbeat\] t=([\d.]+) tick=(0x[0-9a-f]+) scene=(\S+)')
CHANGE = re.compile(r'^\[HangTrace\] scene (\S+) -> (\S+) t=([\d.]+)')
CONN = {0: 'offline', 1: 'local', 2: 'arena', 3: 'quickplay'}
STATUS = {0: 'inactive', 1: 'training', 2: 'singles', 3: 'doubles'}
SOURCES = {0x235AAA0: 'main_menu', 0x22DA160: 'qp_menu_22da160', 0x22DA090: 'qp_menu_22da090',
           0x22D9FA0: 'arena_22d9fa0', 0x1BD45E0: 'local_1bd45e0', 0x25D9288: 'stage_setup',
           0x16CCAB8: 'latency_hook', 0x0FF11E: 'stale_check'}


def online_words(melee, mode):
    """auto_policy::is_online_stage."""
    return (mode & 0x7FFF0000) == 0x07010000 and melee in (0x0D, 0x0E, 0x10)


def parse_events(text):
    rows = []
    for line in text.splitlines():
        m = EVENT.match(line.strip())
        if m:
            rows.append(tuple(int(m.group(i), 0) for i in range(1, 6)))
    return rows


def parse_hang(text):
    beats, changes = [], []
    for line in text.splitlines():
        m = BEAT.match(line)
        if m:
            beats.append((float(m.group(1)), int(m.group(2), 16), m.group(3)))
            continue
        m = CHANGE.match(line)
        if m:
            changes.append((float(m.group(3)), m.group(1), m.group(2)))
    return beats, changes


def stages(events):
    """Stage loads: (index, tick, stage, status, melee, mode) with the 116 row that follows each 110 row."""
    out = []
    for i, (_, tick, code, a, b) in enumerate(events):
        if code != 110:
            continue
        melee = mode = None
        for _, _, c2, a2, b2 in events[i + 1:i + 4]:
            if c2 == 116:
                melee, mode = a2 & 0xFF, b2
                break
        out.append((i, tick, a, STATUS.get(b, str(b)), melee, mode))
    return out


LEAD = TICKS // 5      # a stage hook writes its connection rows (107 a=4/a=5, 30, 73) just before its 110 row


def window(events, stage, following):
    """Event rows of one stage: from just before its load to just before the next stage's own rows."""
    lo = stage[1] - LEAD
    hi = following[1] - LEAD if following else None
    return [e for e in events if e[1] >= lo and (hi is None or e[1] < hi)]


def scene_at(rows, tick):
    """The last code-73 context logged up to one second after `tick`."""
    found = None
    for _, t, code, a, _ in rows:
        if code == 73 and t <= tick + TICKS:
            found = '%s/%s' % (CONN.get(a & 0xFF), STATUS.get((a >> 8) & 0xFF))
    return found


def analyse(hang_text, events_text, stage_logs=None):
    events = parse_events(events_text)
    beats, changes = parse_hang(hang_text)
    if not beats:
        raise ValueError('no heartbeat in hang-trace.log')
    offset = beats[0][1] / TICKS - beats[0][0]          # hang t = tick / TICKS - offset

    def t_of(tick):
        return round(tick / TICKS - offset, 1)

    def following(stage):
        return next((s for s in loads if s[0] > stage[0]), None)

    def state_before(stage):
        rows = [e for e in events if e[1] < stage[1] - LEAD and e[2] == 73]
        return scene_at(rows, stage[1]) if rows else None

    loads = stages(events)
    waiting = next((s for s in loads if s[3] == 'training' and s[4] == 0x0D and s[5] is not None
                    and online_words(s[4], s[5])), None)
    out = {'waiting': None, 'battle': None, 'battles': [], 'results': None}
    if waiting:
        out['waiting'] = {'t': t_of(waiting[1]), 'stage': hex(waiting[2]), 'before': state_before(waiting),
                          'scene': scene_at(window(events, waiting, following(waiting)), waiting[1]),
                          'mode': hex(waiting[5])}
    for battle in loads:
        if waiting is not None and battle[0] < waiting[0]:
            continue
        if battle[3] == 'inactive' and out['battles']:
            out['results'] = {'t': t_of(battle[1]), 'stage': hex(battle[2]),
                              'scene': scene_at(window(events, battle, following(battle)), battle[1])}
            break
        if battle[3] not in ('singles', 'doubles') or battle[4] is None or not online_words(battle[4], battle[5]):
            continue
        after = following(battle)
        rows = window(events, battle, after)
        start, end = t_of(battle[1]), t_of(after[1]) if after else None
        beats_in = [b[2] for b in beats if b[0] >= start + 1.0 and (end is None or b[0] < end)]
        changes_in = ['%s -> %s @%+.1f' % (c[1], c[2], c[0] - start) for c in changes
                      if c[0] >= start - 0.5 and (end is None or c[0] < end - 0.5)]
        counts = {}
        for scene in beats_in:
            counts[scene] = counts.get(scene, 0) + 1
        holds = ['a=%d @%+.1f' % (e[3], (e[1] - battle[1]) / TICKS) for e in rows if e[2] == 107]
        context = ['online=%d src=%s @%+.1f' % (e[3], SOURCES.get(e[4], hex(e[4])), (e[1] - battle[1]) / TICKS)
                   for e in rows if e[2] == 30]
        out['battles'].append({
            't': start, 'stage': hex(battle[2]), 'status': battle[3], 'melee': hex(battle[4]),
            # No 73 row when the stage changed nothing (a sudden death after a Singles battle): the state carried in.
            'mode': hex(battle[5]), 'before': state_before(battle),
            'scene_at_load': scene_at(rows, battle[1]) or state_before(battle),
            'heartbeats': counts, 'scene_changes': changes_in, 'scene_holds_107': holds,
            'online_context_30': context,
            'demoted': any(h.startswith('a=2 ') for h in holds),
            'context_off': any(c.startswith('online=0') for c in context),
            'seconds': None if end is None else round(end - start, 1),
        })
    out['battle'] = out['battles'][0] if out['battles'] else None
    logs = {}
    for name, head in (stage_logs or {}).items():
        conn = re.search(r'connection=(\S+)', head)
        logs[name] = conn.group(1) if conn else None
    out['stage_logs'] = logs
    return out


def battle_online(result):
    """The fix's acceptance: the battle reads quickplay through its whole run and the online bit never drops."""
    if not result.get('battles'):
        return False, 'no online battle stage in events.log'
    problems = []
    for n, b in enumerate(result['battles']):
        bad = [s for s in b['heartbeats'] if not s.startswith('quickplay/')]
        if not (b['scene_at_load'] or '').startswith('quickplay/'):
            problems.append('battle %d: scene at load %s' % (n, b['scene_at_load']))
        if bad:
            problems.append('battle %d: heartbeats %s' % (n, bad))
        if b['demoted']:
            problems.append('battle %d: stale demotion %s' % (n, b['scene_holds_107']))
        if b['context_off']:
            problems.append('battle %d: online context off %s' % (n, b['online_context_30']))
    singles = [c for n, c in result.get('stage_logs', {}).items() if '-singles-' in n or '-doubles-' in n]
    if singles and any(c != 'quickplay' for c in singles):
        problems.append('stage logs %s' % singles)
    return not problems, '; '.join(problems) or 'quickplay through the battle, online bit on'


def main(argv):
    as_json = '--json' in argv
    argv = [a for a in argv if a != '--json']
    hang, events = open(argv[0], encoding='utf-8', errors='replace').read(), \
        open(argv[1], encoding='utf-8', errors='replace').read()
    logs = {}
    for path in argv[2:]:
        logs[path.rsplit('/', 1)[-1]] = '\n'.join(open(path, encoding='utf-8', errors='replace').read().splitlines()[:3])
    result = analyse(hang, events, logs)
    ok, why = battle_online(result)
    result['battle_online'] = {'ok': ok, 'why': why}
    print(json.dumps(result, indent=1) if as_json else json.dumps(result))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
