"""imgui_* lines of two final renderer-status.txt files that differ, frame/draw counters excluded (they count
frames). usage: status_diff.py <a renderer-status.txt> <b renderer-status.txt>"""
import sys
from pathlib import Path

COUNTS = ('imgui_frame_accepted', 'imgui_frame_busy', 'imgui_draw_submitted', 'imgui_draw_reused')


def lines(path):
    return dict(l.split('=', 1) for l in Path(path).read_text().splitlines() if l.startswith('imgui_') and '=' in l)


a, b = lines(sys.argv[1]), lines(sys.argv[2])
keys = sorted(set(a) | set(b))
differ = [(k, a.get(k), b.get(k)) for k in keys if k not in COUNTS and a.get(k) != b.get(k)]
same = [k for k in keys if k not in COUNTS and a.get(k) == b.get(k)]
print(f'{len(same)} equal lines; differ: {differ}')
sys.exit(1 if differ else 0)
