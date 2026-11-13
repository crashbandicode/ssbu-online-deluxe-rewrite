"""Find P1's CSS hand in a frame by its tag: the red "P1" letters with a white and a black outline next to the
glove. Matching is by pixel class (red / white / black), so shading, banners and 0.5x frames matter little. A coarse
pass over 3-px cells finds candidates; the best few are refined pixel by pixel. Returns the tag's top-left in
1280-wide frame pixels, the fingertip estimate and the match score (share of template pixels of the same class).
Pure Pillow (no numpy on the hosts)."""
import json
import sys
from pathlib import Path

from PIL import Image, ImageChops

HERE = Path(__file__).resolve().parent
TEMPLATE = HERE / 'p1-tag-template.json'
CELL = 3
# The fingertip relative to the tag's top-left (imp-a3 hang-css-steve: tag at (657, 241), tip at about (628, 222)).
TIP = (-29, -19)
CLASSES = ('r', 'w', 'k')


def cls(p):
    r, g, b = p[:3]
    if r >= 170 and g <= 110 and b <= 120 and r - g >= 90:
        return 'r'
    hi, lo = max(r, g, b), min(r, g, b)
    if lo >= 200 and hi - lo <= 45:
        return 'w'
    if hi <= 70:
        return 'k'
    return None


def make_template(frame, box, out=TEMPLATE):
    im = Image.open(frame).convert('RGB')
    x0, y0, x1, y1 = box
    px = im.load()
    pts = []
    for y in range(y0, y1):
        for x in range(x0, x1):
            c = cls(px[x, y])
            if c:
                pts.append([x - x0, y - y0, c])
    Path(out).write_text(json.dumps({'source': str(frame), 'box': box, 'points': pts}))
    return len(pts)


def load(path):
    im = Image.open(path).convert('RGB')
    w, h = im.size
    if w != 1280:
        im = im.resize((1280, round(h * 1280 / w)))
    return im


def _ge(band, v):
    return band.point(lambda x: 255 if x >= v else 0)


def _le(band, v):
    return band.point(lambda x: 255 if x <= v else 0)


def _and(*masks):
    out = masks[0]
    for m in masks[1:]:
        out = ImageChops.darker(out, m)
    return out


def class_masks(im):
    """`cls` for every pixel at once, as three L masks (255 = in the class), with Pillow's band operations."""
    r, g, b = im.split()
    red = _and(_ge(r, 170), _le(g, 110), _le(b, 120), _ge(ImageChops.subtract(r, g), 90))
    hi = ImageChops.lighter(ImageChops.lighter(r, g), b)
    lo = ImageChops.darker(ImageChops.darker(r, g), b)
    white = _and(_ge(lo, 200), _le(ImageChops.subtract(hi, lo), 45), ImageChops.invert(red))
    black = _and(_le(hi, 70), ImageChops.invert(red), ImageChops.invert(white))
    return {'r': red, 'w': white, 'k': black}


def class_grids(im):
    """Per class, the share of each CELLxCELL cell's pixels in that class (0..255)."""
    w, h = im.size
    gw, gh = w // CELL, h // CELL
    return {c: m.resize((gw, gh), Image.BOX).load() for c, m in class_masks(im).items()}, gw, gh


def find(path, top=470, refine=6):
    tpl = json.loads(TEMPLATE.read_text())['points']
    tw = max(dx for dx, _, _ in tpl) + 1
    th = max(dy for _, dy, _ in tpl) + 1
    im = load(path)
    im = im.crop((0, 0, im.size[0], min(top, im.size[1])))
    w, h = im.size
    px = im.load()
    grids, gw, gh = class_grids(im)
    # The template on the same grid: per cell, the dominant class and its share.
    tcells = {}
    for dx, dy, c in tpl:
        key = (dx // CELL, dy // CELL)
        tcells.setdefault(key, {}).setdefault(c, 0)
        tcells[key][c] += 1
    coarse = [(k, max(v, key=v.get), v[max(v, key=v.get)] / CELL ** 2) for k, v in tcells.items()]
    coarse = [(k, c, s) for k, c, s in coarse if s >= 0.5]
    tgw, tgh = tw // CELL + 1, th // CELL + 1
    # Anchor: the template's reddest cell; only grid positions that put red there are scored.
    (ax, ay), _, _ = max((t for t in coarse if t[1] == 'r'), key=lambda t: t[2])
    red = grids['r']
    scored = []
    for gy in range(gh - tgh):
        for gx in range(gw - tgw):
            if red[gx + ax, gy + ay] < 128:
                continue
            s = 0.0
            for (kx, ky), c, _ in coarse:
                s += grids[c][gx + kx, gy + ky]
            scored.append((s, gx, gy))
    scored.sort(reverse=True)
    best = (0.0, None)
    for _, gx, gy in scored[:refine]:
        for oy in range(gy * CELL - CELL, gy * CELL + CELL + 1):
            for ox in range(gx * CELL - CELL, gx * CELL + CELL + 1):
                if ox < 0 or oy < 0 or ox + tw > w or oy + th > h:
                    continue
                s = sum(1 for dx, dy, c in tpl if cls(px[ox + dx, oy + dy]) == c) / len(tpl)
                if s > best[0]:
                    best = (s, (ox, oy))
    if best[1] is None:
        return None
    score, (bx, by) = best
    return {'tag': [bx, by], 'tip': [bx + TIP[0], by + TIP[1]], 'score': round(score, 3)}


if __name__ == '__main__':
    if sys.argv[1] == 'template':
        print(make_template(sys.argv[2], [int(v) for v in sys.argv[3].split(',')]))
    else:
        for p in sys.argv[1:]:
            print(p.rsplit('/', 1)[-1], find(p))
