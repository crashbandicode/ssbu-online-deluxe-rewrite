"""imgui-parity: the ims-x1 Opp-line pair split across two hosts (primary's order 13:52Z: the Deluxe lane holds one
Surface slot; pairs are split across hosts). Each host runs its own runner over its own instance; the two meet through
LAN Online matchmaking. Instance a (Surface) runs Deluxe 6c05693 through the standalone imgui-smash 1.2.0; instance b
(Yoga) runs the same Deluxe on its built-in ImGui. Both profiles are named "Æsir テスト".

`search_until_session`: READY TO FIGHT -> Plus (steps.ready_only), then this instance's own detector logs
`session slot=<n> kind=online` (the match started). Frames and the wait are saved."""
import json
import re
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import deluxe_steps  # noqa: E402

SESSION = re.compile(r'session slot=\d+ kind=online[^\n]*')


def _size(path):
    try:
        return Path(path).stat().st_size
    except OSError:
        return 0


def _tail(path, start):
    try:
        with open(path, 'rb') as handle:
            handle.seek(start)
            return handle.read().decode(errors='replace')
    except OSError:
        return ''


def search_until_session(ctx, tag='m', cap_s=600.0, shot_every_s=30.0):
    det = ctx.instance['detector_log']
    start = _size(det)
    ready = deluxe_steps.qcf(ctx, 'ready_only', tag=tag)
    began, next_shot, shots = time.monotonic(), time.monotonic(), []
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        found = SESSION.search(_tail(det, start))
        if found:
            out = {'session': found.group(0), 'waited_s': round(time.monotonic() - began, 1), 'ready': ready,
                   'shots': shots, 'at_utc': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())}
            (Path(ctx.row_dir) / f'{tag}-session-{ctx.on}.json').write_text(json.dumps(out, indent=2) + '\n')
            return out
        if time.monotonic() >= next_shot:
            shots.append(str(ctx.shot(f'{tag}-searching-{int(time.monotonic() - began):03d}s')))
            next_shot += shot_every_s
        time.sleep(1.0)
    raise RuntimeError(f'{ctx.on}: no session within {cap_s} s (is the other host searching?)')
