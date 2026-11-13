"""Rows folders for the release-candidate LAN Quickplay pair (instance a on one host, b on another, each run by its
own `eden_test.py all`).

usage: make_rows_rc.py <scene rows-fix-a dir> <imgui-parity rows-a dir> <build id> <out dir>

Starts from the scene-online fix rows (eden/scene-online/make/make_rows_scene.py: GDB stub on before game.load, the
scene evidence with expect=online, battle_self's attack rule, rematch_self) and adds the imgui-parity rows' checks
that need no trace point: the external host (ims_steps.host_status), the Opp line crop, the font lines with the
"Æsir テスト" glyphs and index=paged (deluxe_steps.font_sample), plus rc_steps.py (hook_lock_skipped=, plugins,
L3 + R3 frames). Both instances draw through the standalone.
"""
import copy
import hashlib
import json
import shutil
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
PORTS = {'a': 6593, 'b': 6594}
FROM_IMP = ['ims_steps.py', 'segments/open-deluxe.edenmacro.json', 'segments/l3-r3.edenmacro.json',
            'segments/dpad-down.edenmacro.json', 'segments/dpad-up.edenmacro.json']
NAME = 'Æsir テスト'


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    scene, imp, build, out = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3], Path(sys.argv[4])
    template = json.loads((scene / 'rows.json').read_text())
    imp_plan = json.loads((imp / 'rows.json').read_text())
    for key in ('a', 'b'):
        dest = out / ('rows-rc-%s' % key)
        if dest.exists():
            shutil.rmtree(dest)
        shutil.copytree(scene, dest, ignore=shutil.ignore_patterns('__pycache__', 'gdb-port-*.txt', 'rows.json'))
        for name in FROM_IMP:
            (dest / name).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(imp / name, dest / name)
        shutil.copy2(HERE / 'rc_steps.py', dest / 'rc_steps.py')
        (dest / ('gdb-port-%s.txt' % key)).write_text('%d\n' % PORTS[key])
        plan = json.loads(json.dumps(template).replace('-a"', '-%s"' % key).replace('"on": "a"', '"on": "%s"' % key))
        plan['segments'].update({
            name: dict(copy.deepcopy(imp_plan['segments'][name]),
                       sha256=sha(dest / ('segments/%s.edenmacro.json' % name)))
            for name in ('open-deluxe', 'l3-r3', 'dpad-down', 'dpad-up')})
        gates = plan['gates']
        gates.pop('deluxe-embedded')
        gates['deluxe-external'] = copy.deepcopy(imp_plan['gates']['deluxe-external'])
        gates['external-accepted'] = copy.deepcopy(imp_plan['gates']['external-accepted'])
        gates['deluxe-build']['pattern'] = 'Deluxe rewrite 0\\.10\\.5-%s' % build

        def on(step):
            step['on'] = key
            return step

        def run(hook, **args):
            return on({'run': hook, 'check': True, **({'args': args} if args else {})})

        rows = plan['rows']
        load = rows[0]
        load['steps'] = [s for s in load['steps'] if s.get('gate') != 'deluxe-embedded']
        at = next(i for i, s in enumerate(load['steps']) if s.get('gate') == 'deluxe-build')
        load['steps'][at:at] = [on({'gate': 'deluxe-external'}), on({'gate': 'external-accepted'})]
        match = next(r for r in rows if r['id'].startswith('m-match-'))
        at = next(i for i, s in enumerate(match['steps']) if 'battle_self' in json.dumps(s))
        match['steps'][at:at] = [
            run('deluxe_steps.py:open_window', tag='open-match'),
            run('ims_steps.py:crop_strip', tag='opp-line'),
            run('deluxe_steps.py:font_sample', tag='in-match', contains=NAME, index_mode='paged'),
            run('ims_steps.py:host_status', expect='external', tag='host-match'),
            run('rc_steps.py:status', tag='status-match')]
        match['expect'] += (' Deluxe draws through the standalone (external, index=paged, 21 plugins), the Opp line '
                            'holds "%s" glyphs, hook_lock_skipped= is written; both fighters act until sync_end '
                            'with desync_seen=0.' % NAME)
        z = next(i for i, r in enumerate(rows) if r['id'].startswith('z-'))
        rows.insert(z, {
            'id': 'f-after-%s' % key, 'start': 'after-match', 'end': 'after-match', 'budget_min': 3, 'steps': [
                run('deluxe_steps.py:font_sample', tag='after-match', active=False, index_mode='paged'),
                run('rc_steps.py:nav_frames'),
                run('ims_steps.py:host_status', expect='external', tag='host-end'),
                run('rc_steps.py:status', tag='status-end')],
            'expect': 'Fonts still paged after the match; L3 + R3 changes no Deluxe view; the host still external; '
                      'hook_lock_skipped= present.'})
        plan['about'] = ('Release-candidate LAN Quickplay pair, instance %s: Deluxe %s through imgui-smash 1.2.0 '
                         '(standalone), the card\'s plugin set and the tile cursor (21), profile "%s", the GDB stub on '
                         'before game.load (port %d), muted.' % (key, build, NAME, PORTS[key]))
        plan['gdb_note'] = 'Trace points need the GDB stub off; the FindGlyph and pad-poll traces are not armed.'
        for gate in gates.values():
            if 'reference' in gate:
                gate['sha256'] = sha(dest / gate['reference'])
        (dest / 'rows.json').write_text(json.dumps(plan, indent=1, ensure_ascii=False) + '\n')
        print(dest, [r['id'] for r in rows])


if __name__ == '__main__':
    main()
