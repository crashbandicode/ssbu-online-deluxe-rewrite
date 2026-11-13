"""Instance a of the split pair on the XPS instead of the Surface: the Surface ran at about 5 frames/s at 14:5xZ
(load average 15-20 beside other lanes' Edens and compiles), and its one Deluxe-lane slot keeps the Training memory
runs, which must stay on one host to compare."""
import json
from pathlib import Path

src = Path('/tmp/opencode/imgui-parity/split/base-a.json')
config = json.loads(src.read_text())
config.update(host='xps', hostname='ubuntu-xps', ledger_ssh='memento-claw-yoga-ubuntu')
config['instances'][0]['display'] = ':158'
config['authorization'] = config['authorization'].replace('on the Surface', 'on the XPS (moved off the Surface: '
                                                          'about 5 frames/s there at 14:5xZ)')
Path('/tmp/opencode/imgui-parity/split/base-a-xps.json').write_text(json.dumps(config, indent=2, ensure_ascii=False) + '\n')
print(config['host'], config['instances'][0]['display'], [p['path'].rsplit('/', 1)[1] for p in config['instances'][0]['plugins']])
