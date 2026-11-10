"""Deluxe Unicode opponent label: Eden step hooks (tools/eden-runner StepContext).

Both instances run Online Deluxe and both profiles are named "Æsir テスト", so each label shows the other's name:
a runs the build before the fix (effbaa0), b the fix (0.10.3, 6732f29). The label is the Opp line of Deluxe's match
strip. A guest trace on ImFontBaked::FindGlyph (the call ImFont::RenderText makes for every character it draws) shows,
per character, whether the label drew that character's own glyph or the font's fallback glyph.

Navigation reuses quickplay-cancel-fix's steps.py unchanged (copied from /home/intpa/eden-runs/qcf-srm02/rows, sha256
in rows.json "about"), called through `qcf` with a context whose image compare also tries the half-size compare the
runner uses for "scale": "fit" gates: QCF's references were taken at 0.5x and these boots render at 1x.
"""
import importlib.util
import json
import re
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
MODULE = 'libssbu_online_deluxe_rewrite.nro'
FIND_GLYPH = '_ZN11ImFontBaked9FindGlyphEt'
GLYPHS_DATA, FALLBACK_INDEX, GLYPH_SIZE = 56, 64, 44   # ImFontBaked::Glyphs.Data, FallbackGlyphIndex; sizeof(ImFontGlyph)
LABEL = 'Æsir テスト'
LABEL_CODEPOINTS = [ord(c) for c in LABEL if ord(c) >= 0x80]   # Æ テ ス ト


# --- QCF's navigation through a half-size-aware image compare ---------------------------------------------------------

def _load_qcf():
    spec = importlib.util.spec_from_file_location('qcf_steps', HERE / 'steps.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _half_aware_diff(runner_cls):
    def diff(path_a, path_b, roi, b_is_roi=False):
        from PIL import Image
        full = runner_cls.compare_region(path_a, path_b, roi, b_is_roi)[0]
        with Image.open(path_a) as a, Image.open(path_b) as b:
            region = a.convert('RGB').crop(tuple(roi))
            other = b.convert('RGB') if b_is_roi else b.convert('RGB').crop(tuple(roi))
            if region.size != other.size:
                return full
            small = (max(1, region.size[0] // 2), max(1, region.size[1] // 2))
            half = runner_cls._mean_diff(region.resize(small, Image.Resampling.BILINEAR),
                                         other.resize(small, Image.Resampling.BILINEAR))
        return min(full, half)
    return diff


class _RunnerView:
    def __init__(self, runner):
        self._runner = runner
        self.roi_diff = _half_aware_diff(type(runner))

    def __getattr__(self, name):
        return getattr(self._runner, name)


class _CtxView:
    def __init__(self, ctx):
        self._ctx = ctx
        self.runner = _RunnerView(ctx.runner)

    def __getattr__(self, name):
        return getattr(self._ctx, name)


def qcf(ctx, fn, **args):
    """Call quickplay-cancel-fix's steps.py function `fn` on this instance."""
    return getattr(_load_qcf(), fn)(_CtxView(ctx), **args)


# --- helpers ------------------------------------------------------------------------------------------------------

def _save(ctx, name, data):
    path = Path(ctx.row_dir) / f'{name}.json'
    path.write_text(json.dumps(data, indent=1, ensure_ascii=False) + '\n')
    ctx.log({name: data if len(json.dumps(data)) < 2000 else str(path)})
    return data


def _size(path):
    try:
        return Path(path).stat().st_size
    except OSError:
        return 0


def _records(path, start):
    out = []
    try:
        with open(path, 'rb') as f:
            f.seek(start)
            text = f.read().decode(errors='replace')
    except OSError:
        return out
    for line in text.splitlines():
        try:
            out.append(json.loads(line))
        except ValueError:
            continue
    return out


def _int(value):
    if value is None:
        return None
    if isinstance(value, int):
        return value
    try:
        return int(str(value), 0)
    except ValueError:
        return None


def _capture(record, index):
    caps = record.get('captures') or []
    if index >= len(caps):
        return None
    cap = caps[index]
    return _int(cap.get('value') if cap.get('value') is not None else cap.get('hex'))


# --- rows -----------------------------------------------------------------------------------------------------------

def open_window(ctx, tag='open'):
    """L+R+Down in one frame (segment open-deluxe) opens Deluxe's Player Info window, hidden at boot, on Settings.
    Passes when the frame changed in the left or right window column; keeps both frames."""
    before = ctx.shot(f'{tag}-before')
    ctx.play('open-deluxe')
    time.sleep(2.0)
    after = ctx.shot(f'{tag}-after')
    diff = ctx.runner.roi_diff
    regions = {'left': [0, 0, 420, 400], 'right': [860, 0, 1280, 400]}
    diffs = {name: round(diff(str(after), str(before), roi), 2) for name, roi in regions.items()}
    out = _save(ctx, f'{tag}-{ctx.on}', {'before': str(before), 'after': str(after), 'diffs': diffs})
    if max(diffs.values()) < 3.0:
        raise RuntimeError(f'{ctx.on}: no window appeared after L+R+Down: {diffs}')
    return out


def refocus(ctx, back=False, tag='refocus'):
    """dun-x1: each L+R+Down chord's D-pad Down also moved the Quickplay menu's focus from Solo to Preferred Rules
    (and b's to-css then opened the rules). B leaves the rules editor; Up returns the focus to Solo."""
    if back:
        ctx.press('b', 120)
        time.sleep(2.5)
    ctx.press('up', 120)
    time.sleep(1.0)
    return {'back': back, 'shot': str(ctx.shot(tag))}


def _mean(path, box):
    from PIL import Image, ImageStat
    with Image.open(path) as im:
        return [round(x) for x in ImageStat.Stat(im.convert('RGB').crop(tuple(box))).mean]


def to_solo(ctx, tries=6):
    """dun-x1 m1: a's single Up tap did not move the focus, and to-css opened the Preferred Rules editor. Leaves the
    editor with B while the Quickplay header is not shown, then D-pad Up (segment dpad-up) until Solo has the focus.
    Focus by colour, measured on this run's frames (evidence/dun-x1 mid1-mid3): the Solo tile's top edge
    [228,86,578,94] is green (18,153,30) when focused and grey (166,173,177) otherwise; the Preferred Rules box's top
    edge [330,440,890,448] is light grey (242) unless it is focused (blue mean under 100)."""
    diff = _half_aware_diff(type(ctx.runner))
    looks = []
    for i in range(tries):
        ctx.check_stop()
        shot = ctx.shot(f'solo-look-{i}')
        header = round(diff(str(shot), str(HERE / 'refs' / 'quickplay-menu.png'), [40, 5, 300, 45]), 2)
        solo, rules = _mean(shot, [228, 86, 578, 94]), _mean(shot, [330, 440, 890, 448])
        looks.append({'shot': str(shot), 'header': header, 'solo': solo, 'rules': rules})
        if header > 8:
            ctx.press('b', 150)
            time.sleep(2.5)
            continue
        if solo[1] > 120 and solo[0] < 80:
            return _save(ctx, f'to-solo-{ctx.on}', {'looks': looks})
        ctx.play('dpad-up')
        time.sleep(1.5)
    _save(ctx, f'to-solo-{ctx.on}', {'looks': looks})
    raise RuntimeError(f'{ctx.on}: Solo not focused after {tries} looks: {looks[-1]}')


def hide_window(ctx, chords=3):
    """Settings -> Performance -> Window Options -> hidden (three L+R+Down chords, as d02 did), so the window no
    longer covers the waiting-menu checks. Keeps the frame after."""
    for _ in range(chords):
        ctx.check_stop()
        ctx.play('open-deluxe')
        time.sleep(1.0)
    return {'chords': chords, 'shot': str(ctx.shot('hidden'))}


def arm_label_trace(ctx, max_hits=4000):
    """Arms FindGlyph for non-ASCII codepoints (x1 >= 0x80) with its return. Entry captures Glyphs.Data and
    FallbackGlyphIndex, so each return tells whether the fallback glyph came back."""
    params = {'target': f'{MODULE}!{FIND_GLYPH}', 'registers': ['x0', 'x1'],
              'captures': [{'expression': f'x0+{GLYPHS_DATA}', 'type': 'u64'},
                           {'expression': f'x0+{FALLBACK_INDEX}', 'type': 'i32'}],
              'condition': 'x1 >= 0x80', 'on_return': True, 'rate_limit': 2000, 'max_hits': max_hits}
    trace = ctx.instance.get('trace_jsonl')
    start = _size(trace)
    point = ctx.call('trace.arm', params)
    return _save(ctx, f'arm-{ctx.on}', {'point': point, 'trace': trace, 'start': start, 'params': params})


def _point_id(ctx):
    found = sorted(Path(ctx.row_dir).glob(f'arm-{ctx.on}.json'))
    if not found:
        raise RuntimeError(f'{ctx.on}: arm_label_trace did not run in this row')
    data = json.loads(found[-1].read_text())
    point = data.get('point')
    pid = point.get('id') if isinstance(point, dict) else point
    return str(pid), data['trace'], data['start']


def _classify(records, pid):
    """(codepoint -> {'own': n, 'fallback': n}, samples): entries paired with their returns per thread."""
    open_entry, counts, samples = {}, {}, []
    for r in records:
        if str(r.get('point_id')) != pid:
            continue
        thread = r.get('thread_id')
        if r.get('event') == 'entry':
            regs = r.get('registers') or {}
            code = (_int(regs.get('x1')) or 0) & 0xFFFF
            open_entry[thread] = (code, _capture(r, 0), _capture(r, 1), r.get('frame'))
        elif r.get('event') == 'return' and thread in open_entry:
            code, data, index, frame = open_entry.pop(thread)
            ret = _int((r.get('registers') or {}).get('x0'))
            if code < 0x80 or data is None or index is None or ret is None:
                continue
            fallback = ret == data + index * GLYPH_SIZE
            tally = counts.setdefault(f'U+{code:04X}', {'own': 0, 'fallback': 0})
            tally['fallback' if fallback else 'own'] += 1
            if len(samples) < 24:
                samples.append({'codepoint': f'U+{code:04X}', 'frame': frame, 'glyphs': hex(data),
                                'fallback_index': index, 'returned': hex(ret), 'fallback': fallback})
    return counts, samples


def label_glyphs(ctx, expect, cap_s=90, settle_s=3.0):
    """Waits until the label's non-ASCII characters (Æ テ ス ト) have been drawn, then checks the glyph each one
    drew: expect "fallback" (the build before the fix) or "own" (the fix; the last draws of each character, after
    any first-load growth of the glyph table). Disarms the point and keeps the classification and samples."""
    pid, trace, start = _point_id(ctx)
    wanted = [f'U+{c:04X}' for c in LABEL_CODEPOINTS]
    began = time.monotonic()
    counts, samples = {}, []
    while time.monotonic() - began < cap_s:
        ctx.check_stop()
        counts, samples = _classify(_records(trace, start), pid)
        if all(counts.get(w, {}).get('own', 0) + counts.get(w, {}).get('fallback', 0) >= 3 for w in wanted):
            time.sleep(settle_s)
            counts, samples = _classify(_records(trace, start), pid)
            break
        time.sleep(1.0)
    try:
        ctx.call('trace.disarm', {'id': pid})
    except RuntimeError as error:
        ctx.log({'disarm_error': repr(error)})
    shot = ctx.shot(f'label-{expect}')
    out = _save(ctx, f'label-{ctx.on}', {'expect': expect, 'label': LABEL, 'counts': counts, 'samples': samples,
                                         'seconds': round(time.monotonic() - began, 1), 'shot': str(shot)})
    missing = [w for w in wanted if w not in counts]
    if missing:
        raise RuntimeError(f'{ctx.on}: label characters never drawn: {missing}; counts {counts}')
    if expect == 'fallback':
        bad = {w: counts[w] for w in wanted if counts[w]['own']}
    else:
        bad = {w: counts[w] for w in wanted if counts[w]['own'] == 0}
    if bad:
        raise RuntimeError(f'{ctx.on}: expected {expect}, got {bad}')
    return {'expect': expect, 'counts': {w: counts[w] for w in wanted}}


def hold(ctx, seconds=7.0):
    """Keeps the label on screen for a clip (the fighters stand; under Quickplay's 30 s idle drop)."""
    end = time.monotonic() + seconds
    while time.monotonic() < end:
        ctx.check_stop()
        time.sleep(0.25)
    return {'held_s': seconds}


def font_stats(ctx, expect_shared=None):
    """Deluxe's renderer-status.txt imgui_font_* lines (written every 5 s). The fix reports its merged shared fonts,
    glyphs and atlas bytes; the build before it has no such lines."""
    path = ctx.instance.get('deluxe_status')
    text = Path(path).read_text(errors='replace') if path and Path(path).exists() else ''
    lines = dict(m.groups() for m in re.finditer(r'^(imgui_[a-z_]+)=(.*)$', text, re.M))
    fonts = {k: v for k, v in lines.items() if k.startswith('imgui_font')}
    out = _save(ctx, f'font-stats-{ctx.on}', {'file': path, 'fonts': fonts,
                                              'imgui_stats': lines.get('imgui_stats')})
    if expect_shared is not None:
        shared = int(fonts.get('imgui_font_shared', '0') or 0)
        if expect_shared and shared < 1:
            raise RuntimeError(f'{ctx.on}: no shared fonts merged: {fonts}')
        if not expect_shared and fonts:
            raise RuntimeError(f'{ctx.on}: the build before the fix reported font lines: {fonts}')
    return out
