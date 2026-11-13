"""Base config for one instance of the scene-online split pair (passed to eden_test.py all --from). Run it on the
executor host: it starts from that host's imgui-parity split base (the same profile, game, inputs and Qt fixture as the
imp-opp pairs) and pins the files it finds there.

usage: make_config_scene.py <xps|yoga> <deluxe NRO> <out.json>

Changes from the imgui-parity base: owner deluxe-scene-online; the Eden 2f315fc4 gdbstub build; no guest trace (the
GDB stub is on, set by the rows before game.load); no standalone imgui-smash; the given Deluxe NRO; Deluxe's
hang-trace.log as a source; this run's display. Kept: profile g, 1x render, volume 0 (lifecycle), the Æsir profiles.dat,
Deluxe's config.ini, the Yoga's Qt fixture (its LAN interface).
"""
import hashlib
import json
import sys
from pathlib import Path

APP = ('/home/intpa/Applications/Eden-GIF/20261113-973abf9c2e3f-unmapped-stop-lan2c-svc-x8-x17-gdbstub-2f315fc4b7d9/'
       'Eden.AppImage')
BASE = {'xps': '/home/intpa/eden-runs/imp-split/base-a-xps.json', 'yoga': '/home/intpa/eden-runs/imp-split/base-b.json'}
DISPLAY = {'xps': ':163', 'yoga': ':164'}
DELUXE = 'user/sdmc/ultimate/ssbu_online_deluxe_rewrite/'


def pin(path, **extra):
    return dict(path=path, sha256=hashlib.sha256(Path(path).read_bytes()).hexdigest(), **extra)


def main():
    host, nro, out = sys.argv[1], sys.argv[2], Path(sys.argv[3])
    config = json.loads(Path(BASE[host]).read_text())
    config.pop('plugin_swaps', None)
    config.update(owner='OpenCode deluxe-scene-online', label='scene-base', runtime=4200, perf_sensitive=False,
                  authorization=('deluxe-scene-online (Deluxe lane coordinator 2026-11-13 16:16Z; no Surface slot): '
                                 'one instance of a LAN Quickplay pair split across the XPS (a) and the Yoga (b); the '
                                 'GDB stub on before game.load (gdb_rc11, no holding helper), volume 0; Deluxe %s, '
                                 'detector + tile cursor. No card; nothing pushed.' % Path(nro).parent.name))
    for item in config['instances']:
        item.pop('trace', None)
        item['display'] = DISPLAY[host]
        item['app'] = pin(APP)
        plugins = [p for p in item['plugins'] if Path(p['path']).name in
                   ('libssbu_desync_detector.nro', 'libssbu_tile_cursor.nro')]
        item['plugins'] = [pin(p['path']) for p in plugins] + [pin(nro)]
        item['files'] = [pin(f['path'], to=f['to']) for f in item['files']]
        item['sources'] = dict(item.get('sources', {}), deluxe_hang=DELUXE + 'hang-trace.log')
        item['eden_settings'] = {'core.resolution_setup': 3}
    out.write_text(json.dumps(config, indent=2, ensure_ascii=False) + '\n')
    print(out, host, [Path(p['path']).name for p in config['instances'][0]['plugins']])


if __name__ == '__main__':
    main()
