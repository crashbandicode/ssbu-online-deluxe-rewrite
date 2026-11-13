"""imp-oppb8: the Yoga broker still bound the VPN (lan=10.125.210.2). qt-yoga-b.ini set network_interface but kept
network_interface\\default=true, so Eden took the default (empty) and rewrote the value. qt-yoga-b2.ini marks it
non-default; base-b.json pins the new file."""
import hashlib
import json
from pathlib import Path

root = Path('/tmp/opencode/imgui-parity/split')
src = (root / 'qt-yoga-b.ini').read_text()
assert src.count('network_interface\\default=true\nnetwork_interface=enxf8e43bdc2cb5\n') == 1
out = src.replace('network_interface\\default=true\nnetwork_interface=enxf8e43bdc2cb5\n',
                  'network_interface\\default=false\nnetwork_interface=enxf8e43bdc2cb5\n')
(root / 'qt-yoga-b2.ini').write_text(out)
sha = hashlib.sha256(out.encode()).hexdigest()
path = root / 'base-b.json'
config = json.loads(path.read_text())
files = config['instances'][0]['files']
files[:] = [f for f in files if f.get('to') != 'user/config/qt-config.ini']
files.append({'path': '/home/intpa/eden-runs/imp-split/qt-yoga-b2.ini', 'sha256': sha, 'to': 'user/config/qt-config.ini'})
path.write_text(json.dumps(config, indent=2, ensure_ascii=False) + '\n')
print('qt-yoga-b2.ini', sha)
