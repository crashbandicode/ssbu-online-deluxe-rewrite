"""Desync verdicts (desync_seen per session), nav-chord and battle receipts of the split pair."""
import json
import re
import sys
from pathlib import Path

root = Path('/home/intpa/smash-training-lobby/artifacts/imgui-parity/eden')
suffix = sys.argv[1] if len(sys.argv) > 1 else '9'
for key, label in (('a', f'imp-oppa{suffix}'), ('b', f'imp-oppb{suffix}')):
    run = root / label
    print(f'== {key}: {label}')
    log = (run / 'post-run' / 'ssbu-desync-detector-events.log').read_text(errors='replace').splitlines()
    for line in log:
        if re.search(r'desync_seen|session_end|session_start|match_end|summary|mismatch', line) and 'telemetry' not in line:
            print('  detector:', line[:260])
    for name in ('m-play-{k}/m1-battle-{k}.json', 'm-play-{k}/m1-rematch-{k}.json', 'n-nav-{k}/nav-chord-{k}.json',
                 'n-nav-{k}/host-end-{k}.json', 'z-{k}/z-{k}.json'):
        path = run / 'out' / name.format(k=key)
        if path.exists():
            text = json.dumps(json.loads(path.read_text()), ensure_ascii=False)
            print(f'  {path.name}:', text[:700])
