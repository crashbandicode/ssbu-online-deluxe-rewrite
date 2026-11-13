"""Rows folders for the scene-online split pair: instance a on the XPS, b on the Yoga, each run by its own
`eden_test.py all`; they meet through LAN Online Quickplay.

usage: make_rows_scene.py <imp-split rows-b dir> <qpt-y1 rows dir> <pair: prepost|fix> <out dir>

Writes <out>/rows-<pair>-a and <out>/rows-<pair>-b. Sources, copied unchanged: the imgui-parity split rows
(quickplay-cancel-fix's steps.py through deluxe_steps.qcf, split_steps.search_until_session, its segments and refs)
and the qp-offline-training rows (gdb_rc11.py, gdbdump.py, gdb-nohold.txt, qp_steps.py's file readers, the
attract-cancel segment). Added: scene_steps.py and scene_table.py from this folder, gdb-port-<key>.txt.
prepost: a = Deluxe 9128cd2 (before 1173bea), b = f33cfef (local main HEAD); the evidence row records.
fix: a and b = the fix; the evidence row needs every battle to read quickplay with the online bit on.
"""
import copy
import hashlib
import json
import shutil
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
BUILDS = {'prepost': {'a': '9128cd2', 'b': 'f33cfef'}, 'fix': {'a': '4573890', 'b': '4573890'}}
PORTS = {'a': 6591, 'b': 6592}
FROM_IMP = ['steps.py', 'deluxe_steps.py', 'split_steps.py', 'segments/to-online.edenmacro.json',
            'segments/to-quickplay.edenmacro.json', 'segments/to-css.edenmacro.json', 'segments/tile-up.edenmacro.json',
            'segments/pick.edenmacro.json', 'refs/main.png', 'refs/online-menu-roi.png', 'refs/quickplay-menu-roi.png',
            'refs/quickplay-menu.png', 'refs/ready-banner-roi.png', 'refs/ready-banner.png', 'refs/rematch-dialog.png',
            'refs/leave-battle-tab.png', 'refs/waiting-menu.png', 'refs/online-menu.png', 'refs/css-quickplay.png']
FROM_QPT = ['gdb_rc11.py', 'gdbdump.py', 'gdb-nohold.txt', 'qp_steps.py', 'segments/attract-cancel.edenmacro.json']
OURS = ['scene_steps.py', 'scene_table.py']


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def seg(name, cap):
    path = 'segments/%s.edenmacro.json' % name
    return {'file': path, 'cap': cap}


def gates(build):
    roi = {'reference_is_roi': True, 'frame': [1280, 674], 'scale': 'fit'}
    return {
        'loaded': {'library': 'loaded'},
        'installed': {'kind': 'log', 'source': 'detector_log', 'whole_file': True, 'cap': 120,
                      'pattern': 'plugin=ssbu-desync-detector/\\S+ [^\\n]*heap_allocations=0'},
        'deluxe-embedded': {'kind': 'log', 'source': 'deluxe_install', 'whole_file': True, 'cap': 120,
                            'pattern': 'overlay_host_ready'},
        'deluxe-build': {'kind': 'log', 'source': 'deluxe_events', 'whole_file': True, 'cap': 60,
                         'pattern': 'Deluxe rewrite 0\\.10\\.5-%s' % build},
        'main': dict(kind='image', reference='refs/main.png', roi=[150, 265, 450, 335], tolerance=8, cap=45,
                     recover='attract-cancel', recover_times=4, **roi),
        'online-menu': dict(kind='image', reference='refs/online-menu-roi.png', roi=[40, 5, 300, 45], tolerance=8,
                            cap=40, **roi),
        'quickplay-menu': dict(kind='image', reference='refs/quickplay-menu-roi.png', roi=[40, 5, 300, 45],
                               tolerance=8, cap=40, **roi),
        'css-tile': {'kind': 'log', 'source': 'tile_log', 'pattern': 'settled screen=css at=(\\d+) hovered=\\1\\b',
                     'cap': 40},
        'ready-banner-any': dict(kind='image', reference='refs/ready-banner-roi.png', roi=[200, 110, 600, 240],
                                 tolerance=40, cap=30, **roi),
    }


def rows(key, expect):
    def on(step):
        step['on'] = key
        return step

    def run(hook, **args):
        return on({'run': hook, 'check': True, **({'args': args} if args else {})})

    return [
        {'id': 'q00-load-%s' % key, 'start': 'frontend-ready', 'end': 'loaded', 'budget_min': 6, 'steps': [
            run('gdb_rc11.py:gdb_on'), on({'call': 'game.load', 'params': {'path': '@game'}, 'job_cap': 240}),
            run('gdb_rc11.py:gdb_kick'), on({'gate': 'loaded'}), on({'gate': 'installed'}),
            on({'gate': 'deluxe-embedded'}), on({'gate': 'deluxe-build'}),
            {'reason': 'other plugins finish booting', 'wait_ms': 45000}, on({'shot': 'boot'})]},
        {'id': 'q01-main-%s' % key, 'start': 'loaded', 'end': 'main', 'budget_min': 3, 'steps': [
            run('deluxe_steps.py:qcf', fn='to_main'), on({'gate': 'main'})]},
        {'id': 'q02-online-%s' % key, 'start': 'main', 'end': 'online-menu', 'budget_min': 3, 'steps': [
            on({'gate': 'main'}), on({'play': 'to-online'}), on({'gate': 'online-menu'})]},
        {'id': 'q03-quickplay-%s' % key, 'start': 'online-menu', 'end': 'quickplay-menu', 'budget_min': 3, 'steps': [
            on({'gate': 'online-menu'}), on({'play': 'to-quickplay'}), on({'gate': 'quickplay-menu'})]},
        {'id': 'm-css-%s' % key, 'start': 'quickplay-menu', 'end': 'css-ready', 'budget_min': 3, 'steps': [
            run('deluxe_steps.py:qcf', fn='reach_css_tile'), on({'gate': 'css-tile'}), on({'play': 'pick'}),
            on({'gate': 'ready-banner-any'})],
         'expect': 'The character select with a fighter picked and READY TO FIGHT up.'},
        {'id': 'm-match-%s' % key, 'start': 'css-ready', 'end': 'after-match', 'budget_min': 16, 'steps': [
            run('scene_steps.py:mark'),
            run('split_steps.py:search_until_session', tag='m1'),
            run('deluxe_steps.py:qcf', fn='battle_self', tag='m1'),
            run('deluxe_steps.py:qcf', fn='rematch_self', answer='no', tag='m1'),
            run('scene_steps.py:evidence', tag='m1', expect=expect)],
         'expect': ('Waiting room, matched battle and results played; Deluxe\'s scene per phase recorded.'
                    if expect == 'record' else
                    'Every battle reads quickplay/singles from its load to the results; no stale demotion (107 a=2); '
                    'the online bit (event 30) never goes off; the battle stage log says connection=quickplay.')},
        {'id': 'z-%s' % key, 'start': 'after-match', 'end': 'after-match', 'budget_min': 1, 'steps': [
            run('gdb_rc11.py:z_stub'), run('qp_steps.py:final_evidence')],
         'expect': 'No panic/abort/crash/Unmapped in the console; Deluxe\'s files kept.'},
    ]


def main():
    imp, qpt, pair, out = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3], Path(sys.argv[4])
    expect = 'record' if pair == 'prepost' else 'online'
    for key in ('a', 'b'):
        dest = out / ('rows-%s-%s' % (pair, key))
        if dest.exists():
            shutil.rmtree(dest)
        for src, names in ((imp, FROM_IMP), (qpt, FROM_QPT), (HERE, OURS)):
            for name in names:
                (dest / name).parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(src / name, dest / name)
        (dest / ('gdb-port-%s.txt' % key)).write_text('%d\n' % PORTS[key])
        build = BUILDS[pair][key]
        segments = {name: dict(seg(name, cap), sha256=sha(dest / ('segments/%s.edenmacro.json' % name)))
                    for name, cap in (('to-online', 60), ('to-quickplay', 40), ('to-css', 60), ('tile-up', 20),
                                      ('pick', 20), ('attract-cancel', 30))}
        g = gates(build)
        for gate in g.values():
            if 'reference' in gate:
                gate['sha256'] = sha(dest / gate['reference'])
        plan = {
            'game_id': '01006A800016E000', 'start': 'frontend-ready', 'on_failure': 'stop',
            'about': ('deluxe-scene-online %s pair, instance %s (%s): Deluxe %s, the detector and the tile cursor, both '
                      'profiles "Æsir テスト", the GDB stub on before game.load (no holding helper), LAN Quickplay '
                      'against the other host.') % (pair, key, 'XPS' if key == 'a' else 'Yoga', build),
            'scope': 'Deluxe\'s scene (hang-trace.log, events 30/73/107/110/116, matches/ connection=) through the '
                     'waiting room, the battle and the results.',
            'setup': [
                {'state': 'the Eden frontend up, game not loaded, the GDB stub on', 'via': 'hook',
                 'ref': 'gdb_rc11.py gdb_on with gdb-port-%s.txt and gdb-nohold.txt' % key},
                {'state': 'the main menu', 'via': 'hook', 'ref': 'steps.py to_main through deluxe_steps.qcf; main gate '
                                                                 'recover attract-cancel'},
                {'state': 'the Quickplay character select, READY TO FIGHT', 'via': 'segment',
                 'ref': 'segments to-online, to-quickplay, to-css, tile-up, pick'},
                {'state': 'matched with the other host, battle, results', 'via': 'hook',
                 'ref': 'split_steps.search_until_session, steps.py battle_self, rematch_self'},
            ],
            'segments': segments, 'gates': g, 'rows': rows(key, expect),
        }
        (dest / 'rows.json').write_text(json.dumps(plan, indent=1, ensure_ascii=False) + '\n')
        print(dest, build, [r['id'] for r in plan['rows']])


if __name__ == '__main__':
    main()
