"""eden_test.py base config for one instance of the release-candidate pair, from that host's scene-online base
(eden/scene-online/make/make_config_scene.py: profile g, the "Æsir テスト" profiles.dat, Deluxe's config.ini, the
2f315fc4 gdbstub Eden).

usage: make_config_rc.py <scene base json> <key a|b> <label> <plugin dir> <rows.json> <out json>

The instance's plugins become every *.nro in <plugin dir> (the card's set with Deluxe and imgui-smash 1.2.0 swapped
in, and the tile cursor), each pinned by sha256.
"""
import hashlib
import json
import sys
from pathlib import Path


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    base, key, label, plugins, rows, out = sys.argv[1:7]
    config = json.loads(Path(base).read_text())
    run = '/home/intpa/eden-runs/%s' % label
    config.update({
        'owner': 'OpenCode deluxe-2', 'label': label, 'work_root': run + '/work', 'state_dir': run + '/state',
        'rows': {'path': rows, 'sha256': sha(rows)},
        'authorization': (
            'Release-candidate LAN Quickplay pair (primary 18:31Z to deluxe-2): one instance per host, instance %s '
            'here; Deluxe 1a7235f (7d0c1c68) through imgui-smash 1.2.0 (9c411083) with the card\'s plugin set; '
            'profile "Æsir テスト"; volume 0; the GDB stub on before game.load. Nothing pushed, no card.' % key)})
    (instance,) = config['instances']
    instance['key'] = key
    instance['plugins'] = [{'path': str(p), 'sha256': sha(p), 'name': p.name}
                           for p in sorted(Path(plugins).glob('*.nro'))]
    Path(out).write_text(json.dumps(config, indent=1, ensure_ascii=False) + '\n')
    print(out, len(instance['plugins']), 'plugins', instance['display'], config.get('host'))


if __name__ == '__main__':
    main()
