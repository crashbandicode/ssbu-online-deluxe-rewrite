"""Yoga Training memory A/B for the fixed build: base configs from hang-card-r5 (the integration lane's Yoga run of
the card set, variant b, whose Training route passed there), with Online Deluxe 2085fd04 (6c05693) and either the
standalone 1.2.0 9c411083 (7d0eff5; variant fc) or no standalone (variant fa). The Surface's route drifted at
load average ~20 (imp-a4, imp-fc1), and timed rows belong on the Yoga, the Ally or the XPS."""
import copy
import hashlib
import json
from pathlib import Path

here = Path('/tmp/opencode/imgui-parity/ytrain')
r5 = json.loads((here / 'r5-config.json').read_text())
builds = Path('/home/intpa/smash-training-lobby/artifacts/imgui-parity/builds')
remote = '/home/intpa/eden-runs/imp-ytrain'
deluxe = builds / 'deluxe-6c05693' / 'libssbu_online_deluxe_rewrite.nro'
imgui = builds / 'imgui-smash-1.2.0-7d0eff5' / 'libimgui_smash.nro'
rows_local = here / 'rows' / 'rows.json'


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


for variant in ('fa', 'fc'):
    config = copy.deepcopy(r5)
    config['owner'] = 'OpenCode deluxe-imgui-parity'
    config['authorization'] = ('imgui-parity worker (Deluxe lane): Training memory A/B of the fixed build on the Yoga, '
                               f'variant {variant}; derived from hang-card-r5. One Yoga slot, Surface untouched.')
    instance = config['instances'][0]
    instance['display'] = ':157'
    plugins = []
    for plugin in instance['plugins']:
        name = plugin.get('name') or plugin['path'].rsplit('/', 1)[1]
        if name == 'libssbu_online_deluxe_rewrite.nro':
            plugin = {'path': f'{remote}/inputs/libssbu_online_deluxe_rewrite.nro', 'sha256': sha(deluxe), 'name': name}
        elif name == 'libimgui_smash.nro':
            if variant == 'fa':
                continue
            plugin = {'path': f'{remote}/inputs/libimgui_smash.nro', 'sha256': sha(imgui), 'name': name}
        plugins.append(plugin)
    instance['plugins'] = plugins
    config['rows'] = {'path': f'{remote}/rows/rows.json', 'sha256': sha(rows_local)}
    # This worker's playbook worktree on the Yoga (13b2e83; runner.py byte-identical to r5's, e008a790).
    config['runner']['path'] = '/home/intpa/pb-imp-13b2e83/tools/eden-runner/runner.py'
    config['playbook'] = {'root': '/home/intpa/pb-imp-13b2e83', 'commit': '13b2e83'}
    # eden_test.py new/all replaces these with the run's own label and folders.
    config['label'] = f'imp-y{variant}0'
    config['work_root'] = f'/home/intpa/eden-runs/imp-y{variant}0/work'
    config['state_dir'] = f'/home/intpa/eden-runs/imp-y{variant}0/state'
    (here / f'base-y{variant}.json').write_text(json.dumps(config, indent=2, ensure_ascii=False) + '\n')
    names = [p['path'].rsplit('/', 1)[1] + ' ' + p['sha256'][:8] for p in plugins]
    print(variant, [n for n in names if 'imgui' in n or 'deluxe' in n], len(plugins), 'plugins')
