import hashlib
import json
from pathlib import Path

root = Path('/tmp/opencode/imgui-parity/split')
for name, key in (('base-a-xps.json', 'a'), ('base-b.json', 'b')):
    path = root / name
    config = json.loads(path.read_text())
    local = root / f'rows-{key}' / 'rows.json'
    config['rows'] = {'path': f'/home/intpa/eden-runs/imp-split/rows-{key}/rows.json',
                      'sha256': hashlib.sha256(local.read_bytes()).hexdigest()}
    path.write_text(json.dumps(config, indent=2, ensure_ascii=False) + '\n')
    print(name, config['rows'])
