"""imgui-parity: split the imp-opp1 pair (ims-x1 rows, fixed builds) into one rows folder and one base config per host.

usage: make_split.py <rows-opp dir> <imp-opp1 config.json> <out dir>
writes <out>/rows-a, <out>/rows-b (each with split_steps.py) and <out>/base-a.json (Surface), <out>/base-b.json (Yoga).
"""
import copy
import json
import shutil
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
src_rows, src_config, out = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
NAME = 'Æsir テスト'
MODULE = {'a': 'libimgui_smash.nro', 'b': 'libssbu_online_deluxe_rewrite.nro'}
HOST = {'a': 'external', 'b': 'embedded'}


def own(steps, key):
    return [copy.deepcopy(s) for s in steps if s.get('on') in (None, key)]


def rows_for(plan, key):
    by_id = {r['id']: r for r in plan['rows']}
    rows = []
    for rid in ('q00-load', 'q01-main', 'q02-online', 'q03-quickplay'):
        row = copy.deepcopy(by_id[rid])
        row['steps'] = own(row['steps'], key)
        for step in row['steps']:
            # imp-oppa3: the frame was still black 24 s after the 15 s settle (a compile was running beside it).
            if 'wait_ms' in step:
                step['wait_ms'] = max(step['wait_ms'], 45000)
        row['id'] = f'{rid}-{key}'
        rows.append(row)
    def run(hook, **args):
        return {'run': hook, 'on': key, 'check': True, **({'args': args} if args else {})}
    rows.append({'id': f'm-css-{key}', 'start': 'quickplay-menu', 'end': 'css-ready', 'budget_min': 3,
                 'steps': [run('deluxe_steps.py:qcf', fn='reach_css_tile'), {'gate': 'css-tile', 'on': key},
                           {'play': 'pick', 'on': key}, {'gate': 'ready-banner-any', 'on': key}],
                 'expect': 'The character select with a fighter picked and READY TO FIGHT up.'})
    rows.append({'id': f'm-label-{key}', 'start': 'css-ready', 'end': 'match', 'budget_min': 12,
                 'steps': [{'run': 'deluxe_steps.py:arm_label_trace', 'on': key},
                           run('split_steps.py:search_until_session', tag='m1'),
                           run('deluxe_steps.py:open_window', tag='open-match'),
                           run('deluxe_steps.py:label_glyphs', expect='own', active=True),
                           run('ims_steps.py:crop_strip', tag='opp-line'),
                           run('deluxe_steps.py:font_sample', tag='in-match', contains=NAME, index_mode='paged'),
                           run('ims_steps.py:host_status', expect=HOST[key], tag='host-match')],
                 'expect': f'Matched with the other host; the Opp line reads {NAME} with each character\'s own glyph; '
                           f'the fonts index in pages; Deluxe draws through the {HOST[key]} host.'})
    rows.append({'id': f'm-play-{key}', 'start': 'match', 'end': 'after-match', 'budget_min': 8,
                 'steps': [run('deluxe_steps.py:qcf', fn='battle_self', tag='m1'),
                           run('deluxe_steps.py:qcf', fn='rematch_self', answer='no', tag='m1')],
                 'expect': 'The match played out on this instance; Rematch? answered No.'})
    rows.append({'id': f'f1-fonts-{key}', 'start': 'after-match', 'end': 'after-match', 'budget_min': 1,
                 'steps': [run('deluxe_steps.py:font_sample', tag='after-match', active=False, index_mode='paged')],
                 'expect': 'After the match: the fonts still index in pages; the dynamic set holds what is on screen.'})
    rows.append({'id': f'n-nav-{key}', 'start': 'after-match', 'end': 'after-match', 'budget_min': 2,
                 'steps': [run('ims_steps.py:nav_chord', module=MODULE[key]),
                           run('ims_steps.py:host_status', expect=HOST[key], tag='host-end')],
                 'expect': 'L3 + R3 does not turn ImGui gamepad navigation on (updateGamepad never runs).'})
    rows.append({'id': f'z-{key}', 'start': 'after-match', 'end': 'after-match', 'budget_min': 1,
                 'steps': [run('deluxe_steps.py:qcf', fn='z_instance')],
                 'expect': 'trace.unmapped_access total 0; no panic/abort/crash/Unmapped in the console.'})
    return rows


plan = json.loads((src_rows / 'rows.json').read_text())
base_config = json.loads(src_config.read_text())
out.mkdir(parents=True, exist_ok=True)
for key in ('a', 'b'):
    dest = out / f'rows-{key}'
    if dest.exists():
        shutil.rmtree(dest)
    shutil.copytree(src_rows, dest)
    shutil.copy2(HERE / 'split_steps.py', dest / 'split_steps.py')
    mine = copy.deepcopy(plan)
    mine['rows'] = rows_for(plan, key)
    mine['scope'] = (f'imgui-parity split pair, instance {key} only ({"Surface" if key == "a" else "Yoga"}): '
                     + plan.get('scope', ''))
    (dest / 'rows.json').write_text(json.dumps(mine, indent=1, ensure_ascii=False) + '\n')
    config = copy.deepcopy(base_config)
    config['instances'] = [i for i in config['instances'] if i['key'] == key]
    config.pop('pair_authorized', None)
    config['authorization'] = (
        'imgui-parity split pair (Deluxe lane; primary 13:52Z: one Surface slot, pairs split across hosts): instance '
        f'{key} only on the {"Surface" if key == "a" else "Yoga"}; LAN Online Quickplay against the other host\'s '
        'instance; both profiles "Æsir テスト"; a = Deluxe 6c05693 through imgui-smash 1.2.0, b = Deluxe 6c05693 '
        'built-in; volume 0; tracing on (GDB stub off: trace points). No card; nothing pushed.')
    if key == 'b':
        config.update(host='yoga', hostname='yoga-ubuntu')
        config.pop('ledger_ssh', None)
        config['instances'][0]['display'] = ':157'
    else:
        config['instances'][0]['display'] = ':141'
    (out / f'base-{key}.json').write_text(json.dumps(config, indent=2, ensure_ascii=False) + '\n')
    print(key, [r['id'] for r in mine['rows']])
