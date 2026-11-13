#!/usr/bin/env python3
"""Base config for the imgui-standalone 1.1.0 boot, from dun-x6's config (the same Surface pair, profiles, files
and trace point): a = Deluxe 0.10.4 release NRO + the standalone 1.1.0 (symbols for both), b = Deluxe 0.10.5
9c994bf (its package ELF). Passed to eden_test.py new --from; that sets the label, work/state paths, the playbook
runner and the rows pin.
Usage: make_config_ims.py <dun-x6 config.json> <eden-inputs dir> <out.json>
"""
import hashlib
import json
import sys
from pathlib import Path

DETECTOR = 'libssbu_desync_detector.nro'
TILE = 'libssbu_tile_cursor.nro'
DELUXE = 'libssbu_online_deluxe_rewrite.nro'
STANDALONE = 'libimgui_smash.nro'


def pin(path):
    path = Path(path).resolve()
    return {'path': str(path), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}


def main(argv):
    config = json.loads(Path(argv[1]).read_text())
    inputs = Path(argv[2]).resolve()
    config['owner'] = 'OpenCode deluxe-imgui-standalone'
    config['label'] = 'ims-base'
    config['runtime'] = 4200
    config['authorization'] = (
        'imgui-standalone-1.1.0 (Deluxe lane coordinator, plan approved 2026-11-13 11:58Z): one LAN Quickplay pair, '
        'TLS off; a Deluxe 0.10.4 release 70a2931e + imgui-smash 1.1.0 standalone 4d8ea566; b Deluxe 0.10.5 '
        're-pin 9c994bf aee7f411 without a standalone; detector + tile cursor on both. Volume 0, no Switch SD.')
    for item in config['instances']:
        keep = [p for p in item['plugins'] if Path(p['path']).name in (DETECTOR, TILE)]
        own = inputs / item['key']
        item['plugins'] = keep + [pin(own / DELUXE)]
        symbols = [{'module': DELUXE, 'elf': str(
            (Path('/home/intpa/deluxe-unicode-names-work/builds/after-21c0009') if item['key'] == 'a' else own)
            / 'libssbu_online_deluxe_rewrite.elf')}]
        if item['key'] == 'a':
            item['plugins'].append(pin(own / STANDALONE))
            symbols.append({'module': STANDALONE, 'elf': str(own / 'libimgui_smash.elf')})
        item['trace']['symbols'] = symbols
    Path(argv[3]).write_text(json.dumps(config, indent=2, ensure_ascii=False) + '\n')
    print(argv[3])
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
