"""imgui_* lines of a and b's final renderer-status that differ (frame/draw counts excluded: they count frames)."""
from pathlib import Path

root = Path('/home/intpa/smash-training-lobby/artifacts/imgui-parity/eden')
COUNTS = ('imgui_frame_accepted', 'imgui_frame_busy', 'imgui_draw_submitted', 'imgui_draw_reused', 'imgui_stats',
          'imgui_host_clients')
for suffix in ('9', '10'):
    lines = {}
    for key in ('a', 'b'):
        path = root / f'imp-opp{key}{suffix}' / 'post-run' / 'ssbu_online_deluxe_rewrite-renderer-status.txt'
        lines[key] = dict(l.split('=', 1) for l in path.read_text().splitlines() if l.startswith('imgui_') and '=' in l)
    keys = sorted(set(lines['a']) | set(lines['b']))
    differ = [(k, lines['a'].get(k), lines['b'].get(k)) for k in keys if k not in COUNTS and lines['a'].get(k) != lines['b'].get(k)]
    same = [k for k in keys if k not in COUNTS and lines['a'].get(k) == lines['b'].get(k)]
    print(f'pair {suffix}: {len(same)} equal lines; differ: {differ}')
    print('  counts a/b:', [(k, lines['a'].get(k), lines['b'].get(k)) for k in COUNTS])
