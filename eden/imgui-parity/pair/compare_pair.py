"""Side by side: the split pair's Deluxe renderer status (imgui lines), m-label/m-play/n-nav receipts and the desync
detector's verdicts, for instance a (standalone 1.2.0) and b (built-in host)."""
import json
import re
import sys
from pathlib import Path

root = Path('/home/intpa/smash-training-lobby/artifacts/imgui-parity/eden')
suffix = sys.argv[1] if len(sys.argv) > 1 else '9'
runs = {'a': root / f'imp-oppa{suffix}', 'b': root / f'imp-oppb{suffix}'}


def short(value, limit=200):
    text = json.dumps(value, ensure_ascii=False) if not isinstance(value, str) else value
    return text if len(text) <= limit else text[:limit] + '…'


for key, run in runs.items():
    print(f'== {key}: {run.name}')
    status = run / 'post-run' / 'ssbu_online_deluxe_rewrite-renderer-status.txt'
    for line in status.read_text(errors='replace').splitlines():
        if line.startswith(('imgui_', 'host', 'overlay_host')) or 'imgui' in line[:24]:
            print('  status:', line[:240])
    install = run / 'post-run' / 'ssbu_online_deluxe_rewrite-install.log'
    for line in install.read_text(errors='replace').splitlines():
        if re.search(r'overlay_host|imgui|paged|external', line):
            print('  install:', line[:200])
    for name in ('opp-line', 'label', 'font-in-match', 'host-match', 'm1-session', 'open-match'):
        path = run / 'out' / f'm-label-{key}' / f'{name}-{key}.json'
        if path.exists():
            print(f'  {name}:', short(json.loads(path.read_text()), 600))
    for row in ('m-play', 'n-nav', 'f1-fonts', 'z'):
        receipt = run / 'out' / f'{row}-{key}.json'
        if receipt.exists():
            data = json.loads(receipt.read_text())
            print(f'  {row}: status={data.get("status")} checks={short(data.get("checks") or data.get("gates") or "", 400)}')
        folder = run / 'out' / f'{row}-{key}'
        if folder.is_dir():
            print(f'    files: {sorted(p.name for p in folder.iterdir() if p.suffix == ".json")}')
    desync = run / 'post-run' / 'ssbu-desync-detector-events.log'
    lines = [l for l in desync.read_text(errors='replace').splitlines() if re.search(r'desync|verdict|session_end|frame=', l)]
    for line in lines[-4:]:
        print('  desync:', line[:220])
