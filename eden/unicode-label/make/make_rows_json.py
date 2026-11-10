#!/usr/bin/env python3
"""Write eden/rows-dun/rows.json from provenance.json's hashes."""
import json
from pathlib import Path

OUT = Path('/home/intpa/deluxe-unicode-names-work/eden/rows-dun')
prov = json.loads((OUT / 'provenance.json').read_text())
h = {**prov['copied'], **{k: v['sha256'] for k, v in prov['derived'].items()}, **prov['authored']}


def seg(name, cap):
    path = f'segments/{name}.edenmacro.json'
    return {'file': path, 'sha256': h[path], 'cap': cap}


def image(ref, roi, tolerance=8, cap=40):
    return {'kind': 'image', 'reference': f'refs/{ref}', 'sha256': h[f'refs/{ref}'], 'roi': roi,
            'tolerance': tolerance, 'cap': cap, 'reference_is_roi': True, 'frame': [1280, 674], 'scale': 'fit'}


Q = 'deluxe_steps.py:qcf'
both = ('a', 'b')
plan = {
    'game_id': '01006A800016E000',
    'start': 'frontend-ready',
    'about': ('Deluxe Unicode opponent label, one Quickplay pair: a runs Deluxe before the fix (effbaa0), b the fix '
              '(0.10.3, 6732f29); both profiles are named "Æsir テスト"; detector and tile cursor on both. Navigation '
              f'is quickplay-cancel-fix\'s steps.py (sha256 {h["steps.py"]}) through deluxe_steps.qcf.'),
    'scope': 'The Opp line of Deluxe\'s match strip on both consoles, the glyph each label character draws '
             '(FindGlyph trace), font stats, one match to sync_end on both, Rematch No.',
    'setup': [
        {'state': 'two Eden frontends up, games not loaded (defer_load)', 'via': 'fixture',
         'ref': 'pair config: profiles.dat "Æsir テスト" and Deluxe config.ini on both (instances[].files)'},
        {'state': 'both at the Quickplay menu', 'via': 'segment',
         'ref': 'segments to-online, to-quickplay after hook steps.py:to_main (qcf-srm02)'},
        {'state': 'Deluxe window open on Settings', 'via': 'segment', 'ref': 'segment open-deluxe (L+R+Down)'},
        {'state': 'b searching alone, a matched with b', 'via': 'hook',
         'ref': 'steps.py reach_css_tile, ready_and_wait_searching, ready_and_wait_session'},
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
        'main': {**image('main.png', [150, 265, 450, 335], cap=30)},
        'online-menu': image('online-menu-roi.png', [40, 5, 300, 45]),
        'quickplay-menu': image('quickplay-menu-roi.png', [40, 5, 300, 45]),
        'css-tile': {'kind': 'log', 'source': 'tile_log', 'pattern': 'settled screen=css at=(\\d+) hovered=\\1\\b',
                     'cap': 40},
        'ready-banner': image('ready-banner-roi.png', [200, 110, 600, 240], tolerance=8, cap=30),
        'ready-banner-any': image('ready-banner-roi.png', [200, 110, 600, 240], tolerance=40, cap=30),
    },
    'rows': [
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
        {'id': 'd01-window', 'start': 'quickplay-menu', 'end': 'quickplay-menu-window', 'budget_min': 2, 'steps': [
            {'run': 'deluxe_steps.py:open_window', 'on': k, 'check': True} for k in both]},
        # dun-x1 r01: the open window covers the Quickplay header the menu checks read. Hide it again (Settings ->
        # Performance -> Window Options -> hidden) and open it once the match is on.
        {'id': 'd02-hide', 'start': 'quickplay-menu-window', 'end': 'quickplay-menu', 'budget_min': 2, 'steps': [
            *[{'play': 'open-deluxe', 'on': k, 'unchecked': True,
               'reason': 'the window covers the gate region; the gate after the third chord checks the result'}
              for k in both for _ in range(3)],
            *[{'gate': 'quickplay-menu', 'on': k} for k in both]]},
        # dun-x1 r01 again: the chords' D-pad Down moved the menu focus to Preferred Rules; b's to-css opened them.
        {'id': 'd03-refocus', 'start': 'quickplay-menu', 'end': 'quickplay-menu-solo', 'budget_min': 1, 'steps': [
            {'run': 'deluxe_steps.py:refocus', 'on': 'b', 'args': {'back': True}},
            {'run': 'deluxe_steps.py:refocus', 'on': 'a', 'args': {'back': False}},
            *[{'gate': 'quickplay-menu', 'on': k} for k in both]]},
        {'id': 'r01-b-waits', 'start': 'quickplay-menu-solo', 'end': 'b-searching', 'budget_min': 4, 'steps': [
            {'run': Q, 'on': 'b', 'check': True, 'args': {'fn': 'reach_css_tile'}},
            {'gate': 'css-tile', 'on': 'b'},
            {'play': 'pick', 'on': 'b'},
            {'gate': 'ready-banner-any', 'on': 'b'},
            {'run': Q, 'on': 'b', 'check': True, 'args': {'fn': 'ready_and_wait_searching', 'tag': 'w1'}}]},
        {'id': 'm1-label', 'start': 'b-searching', 'end': 'match-over', 'budget_min': 12,
         'evidence': ['a (before the fix): Æ テ ス ト draw the fallback glyph',
                      'b (the fix): Æ テ ス ト draw their own glyphs',
                      'both detectors: sync_end desync_seen=0'],
         'steps': [
            # dun-x1 m1: a's Up tap in d03 did not move the focus; to-css opened Preferred Rules.
            {'run': 'deluxe_steps.py:to_solo', 'on': 'a', 'check': True},
            {'run': Q, 'on': 'a', 'check': True, 'args': {'fn': 'reach_css_tile'}},
            {'gate': 'css-tile', 'on': 'a'},
            {'play': 'pick', 'on': 'a'},
            {'gate': 'ready-banner-any', 'on': 'a'},
            *[{'run': 'deluxe_steps.py:arm_label_trace', 'on': k} for k in both],
            {'run': Q, 'on': 'a', 'check': True, 'args': {'fn': 'ready_and_wait_session', 'tag': 'm1', 'cap_s': 150}},
            *[{'run': 'deluxe_steps.py:open_window', 'on': k, 'check': True, 'args': {'tag': 'open-match'}}
              for k in both],
            {'on': 'b', 'gif': {'name': 'label-after', 'preset': 'docs', 'length': 8,
                                'steps': [{'run': 'deluxe_steps.py:hold', 'on': 'b', 'args': {'seconds': 7}}]}},
            {'on': 'a', 'gif': {'name': 'label-before', 'preset': 'docs', 'length': 8,
                                'steps': [{'run': 'deluxe_steps.py:hold', 'on': 'a', 'args': {'seconds': 7}}]}},
            {'run': 'deluxe_steps.py:label_glyphs', 'on': 'b', 'check': True, 'args': {'expect': 'own'}},
            {'run': 'deluxe_steps.py:label_glyphs', 'on': 'a', 'check': True, 'args': {'expect': 'fallback'}},
            {'run': Q, 'on': 'a', 'check': True, 'args': {'fn': 'battle', 'tag': 'm1'}},
            {'run': Q, 'on': 'a', 'check': True, 'args': {'fn': 'rematch_answers', 'tag': 'm1',
                                                           'answers': {'a': 'no', 'b': 'no'}}}]},
        # dun-x1 m1: both label checks passed (label-a.json, label-b.json), then b was dropped for inactivity during
        # the GIF holds and the match was cancelled, so no sync_end came; stopped with STOP-INPUT 02:22Z. m2 plays one
        # match from the character select with no holds, for sync_end on both.
        # The dry run chains each row's start to the previous row's end; m1-label is left out, so m2 names r01's end.
        # The real start is both at Quickplay's character select after m1's cancelled match.
        {'id': 'm2-match', 'start': 'b-searching', 'end': 'match-over', 'budget_min': 9,
         'evidence': ['both detectors: sync_end desync_seen=0', 'Rematch No on both'],
         'steps': [
            *[{'run': 'deluxe_steps.py:hide_window', 'on': k} for k in both],
            {'run': Q, 'on': 'b', 'check': True, 'args': {'fn': 'reach_css_tile'}},
            {'gate': 'css-tile', 'on': 'b'},
            {'play': 'pick', 'on': 'b'},
            {'gate': 'ready-banner-any', 'on': 'b'},
            {'run': Q, 'on': 'b', 'check': True, 'args': {'fn': 'ready_and_wait_searching', 'tag': 'w2'}},
            {'run': Q, 'on': 'a', 'check': True, 'args': {'fn': 'reach_css_tile'}},
            {'gate': 'css-tile', 'on': 'a'},
            {'play': 'pick', 'on': 'a'},
            {'gate': 'ready-banner-any', 'on': 'a'},
            {'run': Q, 'on': 'a', 'check': True, 'args': {'fn': 'ready_and_wait_session', 'tag': 'm2', 'cap_s': 150}},
            {'run': Q, 'on': 'a', 'check': True, 'args': {'fn': 'battle', 'tag': 'm2'}},
            {'run': Q, 'on': 'a', 'check': True, 'args': {'fn': 'rematch_answers', 'tag': 'm2',
                                                           'answers': {'a': 'no', 'b': 'no'}}}]},
        {'id': 'f1-fonts', 'start': 'match-over', 'end': 'match-over', 'budget_min': 1, 'steps': [
            {'run': 'deluxe_steps.py:font_stats', 'on': 'b', 'check': True, 'args': {'expect_shared': True}},
            {'run': 'deluxe_steps.py:font_stats', 'on': 'a', 'check': True, 'args': {'expect_shared': False}}]},
        *[{'id': f'z-{k}', 'start': 'match-over', 'end': 'match-over', 'budget_min': 1, 'steps': [
            {'run': Q, 'on': k, 'check': True, 'args': {'fn': 'z_instance'}}]} for k in both],
    ],
}
# Rows finished in dun-x1 (evidence kept in out/<id>/); not run again.
DONE = {'m1-label'}
plan['rows'] = [row for row in plan['rows'] if row['id'] not in DONE]
(OUT / 'rows.json').write_text(json.dumps(plan, indent=1, ensure_ascii=False) + '\n')
print('rows', [r['id'] for r in plan['rows']])
