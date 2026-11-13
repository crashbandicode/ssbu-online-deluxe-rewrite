#!/usr/bin/env python3
"""rows.json for the imgui-standalone 1.1.0 boot, derived from dun-x6's rows (eden/unicode-label/dun4/rows.json).

  a  Deluxe 0.10.4 (release NRO 70a2931e) + the standalone libimgui_smash.nro 1.1.0: Deluxe draws in the
     standalone's ImGui (external host)
  b  Deluxe 0.10.5 9c994bf (the same archive pinned), no standalone: Deluxe's built-in ImGui (embedded host)

Kept from dun-x6: q00-q03 and r01 (load, main menu, Online, Quickplay, b waits), the m1 label row (both Opp lines
must draw Æ テ ス ト with their own glyphs), f1 and z-a/z-b. Added: which host each Deluxe uses (install.log,
events.log code 16, renderer-status.txt), Opp-line crops, b's menu (the L+R+Down cycle, Up/Down, a ZR+Right /
ZR+Left round trip) and L3 + R3 on both (updateGamepad must not run).
Usage: make_rows_ims.py <rows dir>   (reads rows.json there, the dun-x6 copy, and rewrites it)
"""
import hashlib
import json
import sys
from pathlib import Path

D = 'deluxe_steps.py:'
I = 'ims_steps.py:'
LABEL = 'Æsir テスト'
NEW_SEGMENTS = ('zr-right', 'zr-left', 'dpad-down', 'l3-r3')


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main(argv):
    root = Path(argv[1])
    spec = json.loads((root / 'rows.json').read_text())
    rows = {row['id']: row for row in spec['rows']}
    spec['about'] = ('imgui-smash 1.1.0 standalone and Deluxe 0.10.5 re-pin (archive c1e4471b): a runs Deluxe 0.10.4 '
                     '(70a2931e) with the standalone libimgui_smash.nro 1.1.0 (4d8ea566), b runs Deluxe 0.10.5 9c994bf '
                     '(aee7f411) without a standalone; both profiles are named "Æsir テスト"; detector and tile cursor '
                     'on both. Navigation is quickplay-cancel-fix\'s steps.py through deluxe_steps.qcf, as in dun-x6.')
    spec['scope'] = ('Opp label glyphs on the external host (a) and the embedded host (b), which host each uses, b\'s '
                     'menu pages and a settings round trip, L3 + R3 leaving ImGui navigation off on both.')
    spec['setup'] = spec['setup'][:3] + [
        {'state': 'Deluxe windows open in the match', 'via': 'segment', 'ref': 'segment open-deluxe (L+R+Down)'},
        {'state': 'b: Settings open after the match, selection on You', 'via': 'hook',
         'ref': 'ims_steps.py menu_cycle (open-deluxe until Hidden, then once more)'},
        {'state': 'chords ZR+Right, ZR+Left, L3+R3 in one frame', 'via': 'segment',
         'ref': 'segments zr-right, zr-left, dpad-down, l3-r3 (make/make_segments_ims.py)'}]
    for name in NEW_SEGMENTS:
        spec['segments'][name] = {'file': f'segments/{name}.edenmacro.json',
                                  'sha256': sha(root / 'segments' / f'{name}.edenmacro.json'), 'cap': 20}
    gates = spec['gates']
    base = gates.pop('deluxe-installed')
    gates['deluxe-external'] = {**base, 'pattern': 'overlay_host_external_ready'}
    gates['deluxe-embedded'] = {**base, 'pattern': 'overlay_host_ready'}
    gates['external-accepted'] = {'kind': 'log', 'source': 'deluxe_events', 'whole_file': True, 'cap': 120,
                                  'pattern': r'(?m)^\d+ \d+ 16 0x5 0x4b3f\s*$'}

    load = rows['q00-load']['steps']
    at = next(i for i, s in enumerate(load) if s.get('gate') == 'deluxe-installed')
    load[at:at + 2] = [{'gate': 'deluxe-external', 'on': 'a'}, {'gate': 'external-accepted', 'on': 'a'},
                       {'gate': 'deluxe-embedded', 'on': 'b'}]

    m1 = rows['m1-label']
    m1['evidence'] = ['a (0.10.4 on the standalone 1.1.0) and b (0.10.5 built-in): Æ テ ス ト draw their own glyphs',
                      'a: imgui_stats=not_embedded, code 16 a=5 b=19263; b: imgui_stats=embedded, index paged, '
                      'the dynamic set テ ス ト',
                      'Opp-line crops on both; both detectors: sync_end desync_seen=0']
    steps = m1['steps']
    at = next(i for i, s in enumerate(steps) if s.get('run') == D + 'font_sample' and s['on'] == 'b')
    steps[at:at + 2] = [
        {'run': I + 'crop_strip', 'on': 'b', 'check': True, 'args': {'tag': 'opp-line'}},
        {'run': I + 'crop_strip', 'on': 'a', 'check': True, 'args': {'tag': 'opp-line'}},
        {'run': D + 'font_sample', 'on': 'b', 'check': True,
         'args': {'tag': 'in-match', 'contains': LABEL, 'index_mode': 'paged'}},
        {'run': I + 'host_status', 'on': 'b', 'check': True, 'args': {'expect': 'embedded', 'tag': 'host-match'}},
        {'run': I + 'host_status', 'on': 'a', 'check': True, 'args': {'expect': 'not_embedded', 'tag': 'host-match'}}]

    rows['f1-fonts']['steps'] = [s for s in rows['f1-fonts']['steps'] if s['on'] == 'b']

    menu = {'id': 'n1-menu-b', 'start': 'match-over', 'end': 'match-over', 'budget_min': 4,
            'evidence': ['b: L+R+Down cycles Settings, Performance, Window Options, Hidden (code 23 each), '
                         'then Settings again', 'b: Down moves the selection; ZR+Right on NetLatency logs code 8 '
                         'b=0, ZR+Left b=Auto', 'b: after L3 + R3, updateGamepad never runs'],
            'steps': [{'run': I + 'menu_cycle', 'on': 'b', 'check': True},
                      {'run': I + 'settings_edit', 'on': 'b', 'check': True},
                      {'run': I + 'nav_chord', 'on': 'b', 'check': True,
                       'args': {'module': 'libssbu_online_deluxe_rewrite.nro'}}]}
    nav_a = {'id': 'n2-nav-a', 'start': 'match-over', 'end': 'match-over', 'budget_min': 2,
             'evidence': ['a: after L3 + R3, the standalone\'s updateGamepad never runs (no config.ini)'],
             'steps': [{'run': I + 'nav_chord', 'on': 'a', 'check': True, 'args': {'module': 'libimgui_smash.nro'}},
                       {'run': I + 'host_status', 'on': 'a', 'check': True,
                        'args': {'expect': 'not_embedded', 'tag': 'host-end'}}]}
    order = [r['id'] for r in spec['rows']]
    at = order.index('z-a')
    spec['rows'] = [rows[i] for i in order[:at]] + [menu, nav_a] + [rows[i] for i in order[at:]]
    (root / 'rows.json').write_text(json.dumps(spec, indent=1, ensure_ascii=False) + '\n')
    print(root / 'rows.json', sha(root / 'rows.json'))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
