"""imgui-parity copy of the integration lane's hang.py: after the CPU coin is dropped on Ganondorf, move the empty hand
down, well away from every coin, before B (imp-a1 on the Surface: the 260 ms left push stayed over the coin and B
picked the CPU coin up again). Nothing else changes."""
import sys
from pathlib import Path

OLD = "ctx.call('input.stick', {'stick': 'left', 'x': -1.0, 'y': 0, 'lease_ms': 260})"
NEW = "ctx.call('input.stick', {'stick': 'left', 'x': -0.3, 'y': -1.0, 'lease_ms': 700})"

for folder in sys.argv[1:]:
    path = Path(folder) / 'hang.py'
    text = path.read_text()
    if NEW in text:
        print(path, 'already patched')
        continue
    if text.count(OLD) != 1:
        raise SystemExit(f'{path}: expected one stick-left push')
    path.write_text(text.replace(OLD, NEW))
    print(path, 'patched')
