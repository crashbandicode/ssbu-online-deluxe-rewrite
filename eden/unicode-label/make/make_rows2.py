#!/usr/bin/env python3
"""Rows for the 0.10.3 static/dynamic glyph-set boots, from eden/rows-dun (hooks, QCF steps.py, segments, refs):

  eden/rows-dun2  label pair: a 0.10.2 (effbaa0), b 0.10.3 (release); before/after label, the dynamic set in the
                  match and after it, one match to sync_end, Rematch No
  eden/rows-dun3  name change: a 0.10.3 (release), b 0.10.3 diag with the label feed; the dynamic set following each
                  fed name (old dropped, new drawn), a's label, one match to sync_end, Rematch No
"""
import hashlib
import json
import shutil
import sys
from pathlib import Path

W = Path('/home/intpa/deluxe-unicode-names-work')
SRC = W / 'eden' / 'rows-dun'
LABEL = 'Æsir テスト'
FEED = ['日本語のなまえ', '강철무사', 'Ølstue Ærø']
Q = 'deluxe_steps.py:qcf'
D = 'deluxe_steps.py:'
both = ('a', 'b')


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def plan_for(out, about, rows, setup_last):
    h = {str(p.relative_to(out)): sha(p) for p in out.rglob('*') if p.is_file() and p.name != 'rows.json'}

    def seg(name, cap):
        path = f'segments/{name}.edenmacro.json'
        return {'file': path, 'sha256': h[path], 'cap': cap}

    def image(ref, roi, tolerance=8, cap=40):
        return {'kind': 'image', 'reference': f'refs/{ref}', 'sha256': h[f'refs/{ref}'], 'roi': roi,
                'tolerance': tolerance, 'cap': cap, 'reference_is_roi': True, 'frame': [1280, 674], 'scale': 'fit'}

    return {
        'game_id': '01006A800016E000',
        'start': 'frontend-ready',
        'about': about + f' Navigation is quickplay-cancel-fix\'s steps.py (sha256 {h["steps.py"]}) through '
                         'deluxe_steps.qcf; the Deluxe window opens only in the match (L+R+Down in menus also moves '
                         'the menu focus).',
        'scope': 'Deluxe 0.10.3 static/dynamic glyph sets: the Opp label, imgui_font_* and the dynamic set, '
                 'one match to sync_end on both, Rematch No.',
        'setup': [
            {'state': 'two Eden frontends up, games not loaded (defer_load)', 'via': 'fixture',
             'ref': 'pair config: profiles.dat "Æsir テスト" and Deluxe config.ini on both (instances[].files)'},
            {'state': 'both at the Quickplay menu', 'via': 'segment',
             'ref': 'segments to-online, to-quickplay after hook steps.py:to_main (qcf-srm02)'},
            {'state': 'b searching alone, a matched with b', 'via': 'hook',
             'ref': 'steps.py reach_css_tile, ready_and_wait_searching, ready_and_wait_session'},
            setup_last,
        ],
        'on_failure': 'stop',
        'segments': {name: seg(name, cap) for name, cap in
                     [('to-online', 60), ('to-quickplay', 40), ('pick', 20), ('tile-up', 20), ('to-css', 60),
                      ('ready', 20), ('open-deluxe', 20), ('dpad-up', 20)]},
        'gates': {
            'loaded': {'library': 'loaded'},
            'installed': {'kind': 'log', 'source': 'detector_log', 'whole_file': True, 'cap': 120,
                          'pattern': 'plugin=ssbu-desync-detector/\\S+ [^\\n]*heap_allocations=0'},
            'deluxe-installed': {'kind': 'log', 'source': 'deluxe_install', 'whole_file': True, 'cap': 120,
                                 'pattern': 'overlay_host_ready'},
            'main': image('main.png', [150, 265, 450, 335], cap=30),
            'online-menu': image('online-menu-roi.png', [40, 5, 300, 45]),
            'quickplay-menu': image('quickplay-menu-roi.png', [40, 5, 300, 45]),
            'css-tile': {'kind': 'log', 'source': 'tile_log', 'pattern': 'settled screen=css at=(\\d+) hovered=\\1\\b',
                         'cap': 40},
            'ready-banner-any': image('ready-banner-roi.png', [200, 110, 600, 240], tolerance=40, cap=30),
        },
        'rows': rows,
    }


def common():
    return [
        {'id': 'q00-load', 'start': 'frontend-ready', 'end': 'loaded', 'budget_min': 4, 'steps': [
            *[{'call': 'game.load', 'on': k, 'params': {'path': '@game'}, 'job_cap': 180} for k in both],
            *[{'gate': 'loaded', 'on': k} for k in both],
            *[{'gate': 'installed', 'on': k} for k in both],
            *[{'gate': 'deluxe-installed', 'on': k} for k in both],
            {'reason': 'other plugins finish booting', 'wait_ms': 15000},
            *[{'shot': 'boot', 'on': k} for k in both]]},
        {'id': 'q01-main', 'start': 'loaded', 'end': 'main', 'budget_min': 4, 'steps': [
            *[{'run': Q, 'on': k, 'check': True, 'args': {'fn': 'to_main'}} for k in both],
            *[{'gate': 'main', 'on': k} for k in both]]},
        {'id': 'q02-online', 'start': 'main', 'end': 'online-menu', 'budget_min': 3, 'steps': [
            step for k in both for step in ({'on': k, 'gate': 'main'}, {'on': k, 'play': 'to-online'},
                                            {'on': k, 'gate': 'online-menu'})]},
        {'id': 'q03-quickplay', 'start': 'online-menu', 'end': 'quickplay-menu', 'budget_min': 3, 'steps': [
            step for k in both for step in ({'on': k, 'gate': 'online-menu'}, {'on': k, 'play': 'to-quickplay'},
                                            {'on': k, 'gate': 'quickplay-menu'})]},
        {'id': 'r01-b-waits', 'start': 'quickplay-menu', 'end': 'b-searching', 'budget_min': 4, 'steps': [
            {'run': Q, 'on': 'b', 'check': True, 'args': {'fn': 'reach_css_tile'}},
            {'gate': 'css-tile', 'on': 'b'},
            {'play': 'pick', 'on': 'b'},
            {'gate': 'ready-banner-any', 'on': 'b'},
            {'run': Q, 'on': 'b', 'check': True, 'args': {'fn': 'ready_and_wait_searching', 'tag': 'w1'}}]},
    ]


def a_to_session(arm):
    return [
        {'run': Q, 'on': 'a', 'check': True, 'args': {'fn': 'reach_css_tile'}},
        {'gate': 'css-tile', 'on': 'a'},
        {'play': 'pick', 'on': 'a'},
        {'gate': 'ready-banner-any', 'on': 'a'},
        *[{'run': D + 'arm_label_trace', 'on': k} for k in arm],
        {'run': Q, 'on': 'a', 'check': True, 'args': {'fn': 'ready_and_wait_session', 'tag': 'm1', 'cap_s': 150}},
        *[{'run': D + 'open_window', 'on': k, 'check': True, 'args': {'tag': 'open-match'}} for k in ('b', 'a')],
    ]


def match_end():
    return [
        {'run': Q, 'on': 'a', 'check': True, 'args': {'fn': 'battle', 'tag': 'm1'}},
        {'run': Q, 'on': 'a', 'check': True, 'args': {'fn': 'rematch_answers', 'tag': 'm1',
                                                       'answers': {'a': 'no', 'b': 'no'}}}]


def zrows():
    return [{'id': f'z-{k}', 'start': 'match-over', 'end': 'match-over', 'budget_min': 1, 'steps': [
        {'run': Q, 'on': k, 'check': True, 'args': {'fn': 'z_instance'}}]} for k in both]


def build(name, about, rows, setup_last):
    out = W / 'eden' / name
    if out.exists():
        shutil.rmtree(out)
    shutil.copytree(SRC, out, ignore=shutil.ignore_patterns('rows.json', '__pycache__'))
    plan = plan_for(out, about, rows, setup_last)
    (out / 'rows.json').write_text(json.dumps(plan, indent=1, ensure_ascii=False) + '\n')
    print(name, [r['id'] for r in plan['rows']])


def main():
    gif = lambda on, name: {'on': on, 'gif': {'name': name, 'preset': 'docs', 'length': 8, 'steps': [
        {'run': D + 'hold_active', 'on': on, 'args': {'seconds': 7}}]}}
    build('rows-dun2',
          'Deluxe 0.10.3 (static/dynamic glyph sets), the label pair: a runs 0.10.2 (effbaa0), b 0.10.3 (bc22c80); '
          'both profiles are named "Æsir テスト"; detector and tile cursor on both.',
          common() + [
              {'id': 'm1-label', 'start': 'b-searching', 'end': 'match-over', 'budget_min': 12,
               'evidence': ['a (0.10.2): Æ テ ス ト draw the fallback glyph',
                            'b (0.10.3): Æ テ ス ト draw their own glyphs; the dynamic set holds テ ス ト',
                            'both detectors: sync_end desync_seen=0'],
               'steps': [
                   *a_to_session(both),
                   gif('b', 'label-after'),
                   gif('a', 'label-before'),
                   {'run': D + 'label_glyphs', 'on': 'b', 'check': True, 'args': {'expect': 'own', 'active': True}},
                   {'run': D + 'label_glyphs', 'on': 'a', 'check': True,
                    'args': {'expect': 'fallback', 'active': True}},
                   {'run': D + 'font_sample', 'on': 'b', 'check': True, 'args': {'tag': 'in-match', 'contains': LABEL}},
                   *match_end()]},
              {'id': 'f1-fonts', 'start': 'match-over', 'end': 'match-over', 'budget_min': 1, 'steps': [
                  {'run': D + 'font_sample', 'on': 'b', 'check': True, 'args': {'tag': 'after-match', 'active': False}},
                  {'run': D + 'font_stats', 'on': 'a', 'check': True, 'args': {'expect_shared': False}}]},
              *zrows()],
          {'state': 'Deluxe windows open in the match', 'via': 'segment', 'ref': 'segment open-deluxe (L+R+Down)'})
    build('rows-dun3',
          'Deluxe 0.10.3 (static/dynamic glyph sets), a name change: a runs 0.10.3 (bc22c80), b the 0.10.3 diag '
          'build (bc22c80 --features diag) whose diag.txt label feed shows ' + ', '.join(FEED) + ' in turn; both '
          'profiles are named "Æsir テスト"; detector and tile cursor on both.',
          common() + [
              {'id': 'm1-feed', 'start': 'b-searching', 'end': 'match-over', 'budget_min': 14,
               'evidence': ['b: for each fed name, its characters draw their own glyphs and the dynamic set is that '
                            'name (the previous name dropped, a rebuild counted)',
                            'a (0.10.3): Æ テ ス ト draw their own glyphs',
                            'both detectors: sync_end desync_seen=0'],
               'steps': [
                   *a_to_session(('a',)),
                   {'run': D + 'follow_feed', 'on': 'b', 'check': True, 'args': {'own': LABEL, 'names': FEED}},
                   {'run': D + 'label_glyphs', 'on': 'a', 'check': True, 'args': {'expect': 'own', 'active': True}},
                   {'run': D + 'font_sample', 'on': 'a', 'check': True, 'args': {'tag': 'in-match', 'contains': LABEL}},
                   *match_end()]},
              {'id': 'f1-fonts', 'start': 'match-over', 'end': 'match-over', 'budget_min': 1, 'steps': [
                  {'run': D + 'font_sample', 'on': k, 'check': True, 'args': {'tag': 'after-match', 'active': False}}
                  for k in both]},
              *zrows()],
          {'state': 'Deluxe windows open in the match; b label feed from diag.txt', 'via': 'fixture',
           'ref': 'segment open-deluxe; instances[b].files diag.txt'})


if __name__ == '__main__':
    sys.exit(main())
