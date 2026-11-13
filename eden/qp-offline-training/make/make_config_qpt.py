"""Base config for the qp-fix Eden boot (passed to eden_test.py new --from). Run it on the executor host: it pins
the files it finds there.

usage: make_config_qpt.py <yoga|xps> <single|pair> <rows.json> <out.json>

Instances: profile g, the Eden 2f315fc4 gdbstub build, the detector, the tile cursor and Deluxe qp-fix 1173bea
(2278c03d), the dun runs' profiles.dat and Deluxe config.ini, 1x render (core.resolution_setup 3), no trace points
(the GDB stub is on). The Surface is not a target: the Deluxe lane's one Surface slot is the parity worker's
(coordinator, 2026-11-13 13:52Z).
"""
import hashlib
import json
import sys
from pathlib import Path

APP = '/home/intpa/Applications/Eden-GIF/20261113-973abf9c2e3f-unmapped-stop-lan2c-svc-x8-x17-gdbstub-2f315fc4b7d9/Eden.AppImage'
XCI = 'Super Smash Bros. Ultimate [01006A800016E000]+[v13.0.3+ 9DLC][US](nsw2u.com).xci'
GAME = {'yoga': '/home/intpa/eden-assets/games/01006A800016E000/base/' + XCI,
        'xps': '/home/intpa/Games/Roms/Switch/' + XCI}
HOSTNAME = {'yoga': 'yoga-ubuntu', 'xps': 'ubuntu-xps'}
DISPLAYS = {'a': ':153', 'b': ':154'}
INPUTS = '/home/intpa/deluxe-unicode-names-work/qp-fix/eden-inputs'
NRO = '/home/intpa/deluxe-unicode-names-work/qp-fix/builds/1173bea/2278c03db0935d4e/libssbu_online_deluxe_rewrite.nro'
DELUXE = 'user/sdmc/ultimate/ssbu_online_deluxe_rewrite/'


def pin(path, **extra):
    return dict(path=path, sha256=hashlib.sha256(Path(path).read_bytes()).hexdigest(), **extra)


def instance(key, host):
    return {
        'key': key, 'display': DISPLAYS[key], 'source': '/home/intpa/trainingrooms-eden-lead-astra4/profiles/g',
        'app': pin(APP), 'game': GAME[host],
        'plugins': [pin(INPUTS + '/libssbu_desync_detector.nro'), pin(INPUTS + '/libssbu_tile_cursor.nro'), pin(NRO)],
        'files': [pin(INPUTS + '/profiles-aesir.dat', to='user/nand/system/save/8000000000000010/su/avators/profiles.dat'),
                  pin(INPUTS + '/deluxe-config.ini', to=DELUXE + 'config.ini')],
        'sources': {'detector_log': 'user/sdmc/ultimate/ssbu-desync-detector/events.log',
                    'tile_log': 'user/sdmc/ultimate/ssbu-tile-cursor/events.log',
                    'deluxe_install': DELUXE + 'install.log', 'deluxe_status': DELUXE + 'renderer-status.txt',
                    'deluxe_events': DELUXE + 'events.log', 'deluxe_hang': DELUXE + 'hang-trace.log'},
        'eden_settings': {'core.resolution_setup': 3},
    }


def main():
    host, mode, rows, out = sys.argv[1], sys.argv[2], sys.argv[3], Path(sys.argv[4])
    keys = ['a'] if mode == 'single' else ['a', 'b']
    config = {
        'owner': 'OpenCode deluxe-qp-offline-training', 'label': 'qpt-base', 'host': host,
        'hostname': HOSTNAME[host], 'ledger': '/tmp/deluxe-surface-1007/OWNER-COORDINATION.md',
        'display_helper': pin('/home/intpa/eden-nso-autolan/live-tools/display-rootful.ps1'),
        'work_root': '/home/intpa/eden-runs/qpt-base/work', 'state_dir': '/home/intpa/eden-runs/qpt-base/state',
        'pair_authorized': mode != 'single', 'defer_load': True,
        'runner': pin('/home/intpa/ssbu-mod-playbook/tools/eden-runner/runner.py'),
        'runtime': 3600, 'socket_cap': 90, 'load_cap': 180, 'stop_cap': 30, 'force_cap': 30,
        'authorization': 'deluxe-qp-offline-training (Deluxe lane coordinator 2026-11-13; worker of the Deluxe lane; '
                         'no Surface slot per the coordinator 13:52Z): %s, LAN Quickplay, TLS off, the GDB stub on '
                         'before game.load (gdb_rc11, no holding helper), volume 0; Deluxe qp-fix 1173bea (2278c03d), '
                         'detector + tile cursor. No card; nothing pushed.'
                         % ('one instance' if mode == 'single' else 'one pair'),
        'eden_settings': {'core.resolution_setup': 3},
        'perf_sensitive': False,
        'instances': [instance(key, host) for key in keys],
        'rows': pin(rows),
    }
    if host != 'yoga':
        config['ledger_ssh'] = 'memento-claw-yoga-ubuntu'
    out.write_text(json.dumps(config, indent=1) + '\n')
    print(out, host, mode)


if __name__ == '__main__':
    main()
