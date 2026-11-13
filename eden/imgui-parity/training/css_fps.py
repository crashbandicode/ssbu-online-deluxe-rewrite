"""Presents per second over the CSS heartbeats (css=1) of each run's Deluxe hang-trace.log."""
import re
import sys
from pathlib import Path

BEAT = re.compile(r'^\[Heartbeat\] t=([\d.]+) .*?\bcss=(\d) .*?\bpresent=(\d+)')
runs = {
    'imp-a3': '/home/intpa/eden-runs/imp-a3', 'imp-a1': '/home/intpa/eden-runs/imp-a1',
    'imp-a4': '/home/intpa/eden-runs/imp-a4', 'imp-fa1': '/home/intpa/eden-runs/imp-fa1',
    'imp-b3': '/home/intpa/eden-runs/imp-b3', 'imp-c1': '/home/intpa/eden-runs/imp-c1',
    'imq-c3': '/home/intpa/eden-runs/imq-c3', 'imq-fc1': '/home/intpa/eden-runs/imq-fc1',
    'imp-fc1': '/home/intpa/eden-runs/imp-fc1',
}
sub = 'out/post-run-diag/user/sdmc/ultimate/ssbu_online_deluxe_rewrite/hang-trace.log'
paths = {k: Path(v) / sub for k, v in runs.items()}
art = Path('/home/intpa/smash-training-lobby/artifacts/imgui-parity/eden')
paths['hang-card-r5 (Yoga)'] = art / 'hang-card-r5/post-run/ssbu_online_deluxe_rewrite-hang-trace.log'
paths['imp-yfc1 (Yoga)'] = art / 'imp-yfc1/post-run/ssbu_online_deluxe_rewrite-hang-trace.log'
for name, path in paths.items():
    if not path.exists():
        print(f'{name:20s} no log')
        continue
    beats = [(float(m[1]), int(m[2]), int(m[3])) for m in map(BEAT.search, path.read_text(errors='replace').splitlines()) if m]
    css = [(t, p) for t, c, p in beats if c == 1]
    if len(css) < 2:
        print(f'{name:20s} css beats {len(css)}')
        continue
    rate = (css[-1][1] - css[0][1]) / (css[-1][0] - css[0][0])
    print(f'{name:20s} css beats {len(css):3d}  presents/s {rate:5.1f}')
