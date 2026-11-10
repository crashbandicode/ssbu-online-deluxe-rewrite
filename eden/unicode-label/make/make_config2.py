#!/usr/bin/env python3
"""Base configs for the 0.10.3 static/dynamic boots: eden/config-dun2-<host>.json (a 0.10.2, b 0.10.3) and
eden/config-dun3-<host>.json (a 0.10.3, b 0.10.3 diag + label feed). Usage: make_config2.py <xps|surface>
(paths are that host's; the work folder is mirrored there at the same path)."""
import hashlib
import json
import sys
from pathlib import Path

W = Path('/home/intpa/deluxe-unicode-names-work')
IN = W / 'eden' / 'inputs'
APP = '/home/intpa/Applications/Eden-GIF/20261104-1961a200485e-unmapped-stop-0c7f89f711d6/Eden.AppImage'
APP_SHA = '0c7f89f711d6df37df2243928a92e789aaee319c81d0647a0b8e5b2df9975476'
HOSTS = {
    'xps': {'hostname': 'ubuntu-xps', 'source': '/home/intpa/trainingrooms-eden-lead-astra4/profiles/y',
            'game': '/home/intpa/Games/Roms/Switch/Super Smash Bros. Ultimate [01006A800016E000]+[v13.0.3+ 9DLC][US]'
                    '(nsw2u.com).xci', 'displays': (':131', ':132')},
    'surface': {'hostname': 'ubuntu-surface', 'source': '/home/intpa/trainingrooms-eden-lead-astra4/profiles/g',
                'game': '/home/intpa/eden-assets/games/01006A800016E000/base/Super Smash Bros. Ultimate '
                        '[01006A800016E000]+[v13.0.3+ 9DLC][US](nsw2u.com).xci', 'displays': (':131', ':132')},
}
DATA = 'user/sdmc/ultimate/ssbu_online_deluxe_rewrite'
NRO = 'libssbu_online_deluxe_rewrite.nro'
BUILDS = {'before': W / 'builds' / 'before-effbaa0', 'after': W / 'builds' / 'after-bc22c80',
          'diag': W / 'builds' / 'diag-bc22c80'}


def pin(path):
    return {'path': str(path), 'sha256': hashlib.sha256(Path(path).read_bytes()).hexdigest()}


def instance(host, key, display, build, diag=False):
    build_dir = BUILDS[build]
    files = [{**pin(IN / 'profiles-aesir.dat'), 'to': 'user/nand/system/save/8000000000000010/su/avators/profiles.dat'},
             {**pin(IN / 'deluxe-config.ini'), 'to': f'{DATA}/config.ini'}]
    if diag:
        files.append({**pin(IN / 'diag-label-feed.txt'), 'to': f'{DATA}/diag.txt'})
    return {
        'key': key, 'display': display, 'source': HOSTS[host]['source'],
        'app': {'path': APP, 'sha256': APP_SHA}, 'game': HOSTS[host]['game'],
        'plugins': [pin(IN / 'libssbu_desync_detector.nro'), pin(IN / 'libssbu_tile_cursor.nro'),
                    pin(build_dir / NRO)],
        'files': files,
        'sources': {
            'detector_log': 'user/sdmc/ultimate/ssbu-desync-detector/events.log',
            'tile_log': 'user/sdmc/ultimate/ssbu-tile-cursor/events.log',
            'deluxe_install': f'{DATA}/install.log',
            'deluxe_status': f'{DATA}/renderer-status.txt',
            'deluxe_events': f'{DATA}/events.log',
        },
        'trace': {
            'unmapped_access': 'log',
            'symbols': [{'module': NRO, 'elf': str(build_dir / 'libssbu_online_deluxe_rewrite.elf')}],
            'points': [
                # quickplay-cancel-fix's IngameCallSequenceScene state store (steps.py ready_and_wait_searching).
                {'target': 'main+0x23191bc', 'registers': ['x8', 'x9'],
                 'captures': [{'expression': 'x20+0x174', 'type': 'u32'}]},
            ],
        },
        'eden_settings': {'core.resolution_setup': 3},
    }


def config(host, label, a, b, b_diag, note):
    h = HOSTS[host]
    return {
        'owner': 'OpenCode deluxe-unicode-names',
        'label': label,
        'host': host,
        'hostname': h['hostname'],
        'ledger': '/tmp/deluxe-surface-1007/OWNER-COORDINATION.md',
        'ledger_ssh': 'memento-claw-yoga-ubuntu',
        'display_helper': {'path': '/home/intpa/eden-nso-autolan/live-tools/display-rootful.ps1',
                           'sha256': 'a930b7a6dcb6c7401afb8bbbb2c918ee9b09219a1cd64b4af862938c094e2cff'},
        'work_root': f'/home/intpa/eden-runs/{label}/work',
        'state_dir': f'/home/intpa/eden-runs/{label}/state',
        'pair_authorized': True,
        'defer_load': True,
        'runner': pin(W / 'playbook' / 'tools' / 'eden-runner' / 'runner.py'),
        'runtime': 2700, 'socket_cap': 90, 'load_cap': 180, 'stop_cap': 30, 'force_cap': 30,
        'authorization': ('deluxe-unicode-names (Deluxe opponent label Unicode, 0.10.3 rework: operator via primary '
                          '11:50Z/11:58Z 11-10): one Quickplay pair, TLS off, in a slot integration names; ' + note +
                          '; detector + tile cursor on both. Volume 0, no Switch SD.'),
        'instances': [instance(host, 'a', h['displays'][0], a),
                      instance(host, 'b', h['displays'][1], b, diag=b_diag)],
    }


def main(argv):
    host = argv[1]
    for name, rows, a, b, b_diag, note in (
            ('dun2', 'rows-dun2', 'before', 'after', False, 'a Deluxe 0.10.2 effbaa0, b Deluxe 0.10.3 bc22c80'),
            ('dun3', 'rows-dun3', 'after', 'diag', True,
             'a Deluxe 0.10.3 bc22c80, b Deluxe 0.10.3 bc22c80 diag build with the label feed')):
        cfg = config(host, f'{name}-base', a, b, b_diag, note)
        cfg['rows'] = pin(W / 'eden' / rows / 'rows.json')
        out = W / 'eden' / f'config-{name}-{host}.json'
        out.write_text(json.dumps(cfg, indent=2, ensure_ascii=False) + '\n')
        print(out)


if __name__ == '__main__':
    sys.exit(main(sys.argv))
