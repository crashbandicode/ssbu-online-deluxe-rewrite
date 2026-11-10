#!/usr/bin/env python3
"""Write eden/config-base-xps.json for `eden_test.py new --from` (paths are XPS paths; the work folder is mirrored
there at the same path)."""
import hashlib
import json
from pathlib import Path

W = Path('/home/intpa/deluxe-unicode-names-work')
IN = W / 'eden' / 'inputs'
APP = '/home/intpa/Applications/Eden-GIF/20261104-1961a200485e-unmapped-stop-0c7f89f711d6/Eden.AppImage'
APP_SHA = '0c7f89f711d6df37df2243928a92e789aaee319c81d0647a0b8e5b2df9975476'
GAME = '/home/intpa/Games/Roms/Switch/Super Smash Bros. Ultimate [01006A800016E000]+[v13.0.3+ 9DLC][US](nsw2u.com).xci'
SOURCE = '/home/intpa/trainingrooms-eden-lead-astra4/profiles/y'
DATA = 'user/sdmc/ultimate/ssbu_online_deluxe_rewrite'
BUILDS = {'a': W / 'builds' / 'before-effbaa0', 'b': W / 'builds' / 'after-6732f29'}
NRO = 'libssbu_online_deluxe_rewrite.nro'


def pin(path):
    return {'path': str(path), 'sha256': hashlib.sha256(Path(path).read_bytes()).hexdigest()}


def instance(key, display):
    build = BUILDS[key]
    return {
        'key': key, 'display': display, 'source': SOURCE,
        'app': {'path': APP, 'sha256': APP_SHA}, 'game': GAME,
        'plugins': [pin(IN / 'libssbu_desync_detector.nro'), pin(IN / 'libssbu_tile_cursor.nro'), pin(build / NRO)],
        'files': [
            {**pin(IN / 'profiles-aesir.dat'), 'to': 'user/nand/system/save/8000000000000010/su/avators/profiles.dat'},
            {**pin(IN / 'deluxe-config.ini'), 'to': f'{DATA}/config.ini'},
        ],
        'sources': {
            'detector_log': 'user/sdmc/ultimate/ssbu-desync-detector/events.log',
            'tile_log': 'user/sdmc/ultimate/ssbu-tile-cursor/events.log',
            'deluxe_install': f'{DATA}/install.log',
            'deluxe_status': f'{DATA}/renderer-status.txt',
        },
        'trace': {
            'unmapped_access': 'log',
            'symbols': [{'module': NRO, 'elf': str(build / 'libssbu_online_deluxe_rewrite.elf')}],
            'points': [
                # quickplay-cancel-fix's IngameCallSequenceScene state store (steps.py ready_and_wait_searching).
                {'target': 'main+0x23191bc', 'registers': ['x8', 'x9'],
                 'captures': [{'expression': 'x20+0x174', 'type': 'u32'}]},
                {'target': f'{NRO}!imgui_smash_draw_stats', 'mode': 'counters'},
            ],
        },
        'eden_settings': {'core.resolution_setup': 3},
    }


config = {
    'owner': 'OpenCode deluxe-unicode-names',
    'label': 'dun-base',
    'host': 'xps',
    'hostname': 'ubuntu-xps',
    'ledger': '/tmp/deluxe-surface-1007/OWNER-COORDINATION.md',
    'ledger_ssh': 'memento-claw-yoga-ubuntu',
    'display_helper': {'path': '/home/intpa/eden-nso-autolan/live-tools/display-rootful.ps1',
                       'sha256': 'a930b7a6dcb6c7401afb8bbbb2c918ee9b09219a1cd64b4af862938c094e2cff'},
    'work_root': '/home/intpa/eden-runs/dun-base/work',
    'state_dir': '/home/intpa/eden-runs/dun-base/state',
    'pair_authorized': True,
    'defer_load': True,
    'runner': pin(W / 'playbook' / 'tools' / 'eden-runner' / 'runner.py'),
    'runtime': 2700, 'socket_cap': 90, 'load_cap': 180, 'stop_cap': 30, 'force_cap': 30,
    'authorization': ('deluxe-unicode-names (Deluxe opponent label Unicode; TASK.md, primary 22:36Z 11-09): one '
                      'Quickplay pair on the XPS slot integration named at 01:45Z 11-10 (after rc6-bg-qp2, Down by '
                      '02:45Z), TLS off; a Deluxe before the fix, b Deluxe 0.10.3; detector + tile cursor on both. '
                      'Volume 0, no Switch SD.'),
    'instances': [instance('a', ':131'), instance('b', ':132')],
    'rows': pin(W / 'eden' / 'rows-dun' / 'rows.json'),
}
out = W / 'eden' / 'config-base-xps.json'
out.write_text(json.dumps(config, indent=2, ensure_ascii=False) + '\n')
print(out)
