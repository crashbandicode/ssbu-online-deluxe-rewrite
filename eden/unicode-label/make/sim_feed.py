#!/usr/bin/env python3
"""Offline check of deluxe_steps.py's split feed steps (follow_feed until_phase=1, feed_wait, feed_switch inside a
"gif", follow_feed to the end) against a simulated Deluxe diag build: events.log rows (code 121) every `step` s,
renderer-status.txt rewritten every 5 s. Each step loads the hook file afresh, as runner.py does.
Usage: sim_feed.py <deluxe_steps.py> [step seconds]"""
import importlib.util
import json
import sys
import tempfile
import threading
import time
from pathlib import Path

HOOKS = Path(sys.argv[1]).resolve()
STEP = float(sys.argv[2]) if len(sys.argv) > 2 else 20.0
OWN, NAMES = 'Æsir テスト', ['日本語のなまえ', '강철무사', 'Ølstue Ærø']


def dynamic_of(text):
    return sorted({ord(c) for c in text if not (ord(c) <= 0x7E or 0xA0 <= ord(c) <= 0x17F)})


class Runner:
    def __init__(self, tmp):
        self.instances = {k: {'deluxe_events': str(tmp / f'{k}-events.log'),
                              'deluxe_status': str(tmp / f'{k}-status.txt'),
                              'trace_jsonl': str(tmp / f'{k}.trace.jsonl')} for k in ('a', 'b')}
        self.row_dir = tmp
        self.calls = []

    def call(self, on, method, params=None, job_cap=15):
        self.calls.append((time.monotonic(), on, method))
        return {'id': '9'} if method == 'trace.arm' else {}

    def shot(self, on, name):
        return str(self.row_dir / f'{on}-{name}.png')

    def check_stop(self):
        pass

    def log(self, record):
        pass


class Ctx:
    def __init__(self, runner, on):
        self.runner, self.on, self.instance, self.row_dir = runner, on, runner.instances[on], runner.row_dir

    def call(self, method, params=None):
        return self.runner.call(self.on, method, params)

    def shot(self, name):
        return self.runner.shot(self.on, name)

    def log(self, record):
        pass

    def check_stop(self):
        pass


def load():
    spec = importlib.util.spec_from_file_location('step_hook_deluxe_steps', HOOKS)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module._keep_active = lambda ctx, every_s=0.8: None
    shown = {}

    def classify(records, pid):
        name = shown.get('name', '')
        return {f'U+{ord(c):04X}': {'own': 5, 'fallback': 0} for c in name if ord(c) >= 0x80}, []
    module._classify = classify
    module._records = lambda path, start: []
    return module, shown


def deluxe(runner, began, stop):
    """Feed: own name, the three names, own name again; status every 5 s; the dynamic set 1 s after a change."""
    seq = 1000
    events = Path(runner.instances['b']['deluxe_events'])
    status = Path(runner.instances['b']['deluxe_status'])
    sequence = [OWN] + NAMES + [OWN]
    written, last_status, rebuilds = 0, -10.0, 0
    while not stop.is_set():
        t = time.monotonic() - began
        phase = min(int(t // STEP), len(sequence) - 1)
        while written <= phase:
            with events.open('a') as f:
                f.write(f'{seq} {seq * 1000} 121 0x{[0, 1, 2, 3, 0][written]:x} '
                        f'0x{len(dynamic_of(sequence[written])):x}\n')
            seq += 1
            written += 1
            if written > 1:
                rebuilds += 1
        if t - last_status >= 5.0:
            last_status = t
            current = sequence[phase] if t - phase * STEP > 1.0 or phase == 0 else sequence[phase - 1]
            dyn = dynamic_of(current)
            status.write_text('imgui_font_static_glyphs=321\n'
                              f'imgui_font_dynamic_glyphs={len(dyn)}\n'
                              f'imgui_font_rebuilds={rebuilds}\nimgui_font_misplaced=0\n'
                              'imgui_font_texture_bytes=262144\n'
                              f'imgui_font_index_bytes={20000 + 1536 * len({c >> 8 for c in dyn})}\n'
                              f'imgui_font_index=paged pages={2 + len({c >> 8 for c in dyn})}\n'
                              'imgui_font_dynamic_set=' + (' '.join(f'U+{c:04X}' for c in dyn) or 'none') + '\n')
        time.sleep(0.05)


def main():
    tmp = Path(tempfile.mkdtemp(prefix='sim-feed-'))
    runner = Runner(tmp)
    began = time.monotonic()
    stop = threading.Event()
    sim = threading.Thread(target=deluxe, args=(runner, began, stop), daemon=True)
    sim.start()
    args = {'own': OWN, 'names': NAMES, 'index_mode': 'paged'}
    out = []

    def step(fn, **kw):
        module, shown = load()
        sequence = [OWN] + NAMES + [OWN]
        orig = module._fresh_status

        def fresh(ctx, after, cap_s=12.0, active=True):
            t = time.monotonic() - began
            shown['name'] = sequence[min(int(t // STEP), 4)]
            return orig(ctx, after, cap_s, active)
        module._fresh_status = fresh
        t0 = time.monotonic() - began
        try:
            value = getattr(module, fn)(Ctx(runner, 'b'), **kw)
        except RuntimeError as error:
            value = f'RAISED {error}'
        out.append((fn, round(t0, 1), round(time.monotonic() - began, 1), value))
        print(out[-1], flush=True)
        return value

    step('follow_feed', **args, until_phase=1)
    step('feed_wait', phase=1, after_s=STEP * 0.775)
    g0 = time.monotonic() - began
    step('feed_switch', index=2, hold_s=4.5)
    g1 = time.monotonic() - began
    print(f'gif window {g0:.1f}-{g1:.1f} s; the Korean name from {2 * STEP:.1f} s', flush=True)
    step('follow_feed', **args)
    stop.set()
    feed = json.loads((tmp / 'feed-b.json').read_text())
    for p in feed['phases']:
        print(p['phase'], p['name'], 'late' if p.get('late') else 'on time', p.get('dynamic'), p.get('index_mode'))
    print('problems', feed['problems'])
    print('index trace samples', len(feed['index_trace']), feed['index_trace'][:2])


if __name__ == '__main__':
    main()
