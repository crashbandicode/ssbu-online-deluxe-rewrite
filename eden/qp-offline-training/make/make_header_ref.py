"""Crop a menu header (ROI 40,5,300,45 of a 1280x674 frame) into refs/<name>; qp_steps._back_to_main uses
refs/gm-header.png to tell the Games & More menu from the main menu (the shared library's gm.png did not match the
2f315fc4 build's frame: diff 56 in qpt-y1)."""
import sys
from pathlib import Path

from PIL import Image

ROI = (40, 5, 300, 45)
src, name = Path(sys.argv[1]), sys.argv[2]
out = Path(__file__).resolve().parents[1] / 'refs' / name
with Image.open(src) as im:
    assert im.size == (1280, 674), im.size
    im.convert('RGB').crop(ROI).save(out)
print(out, ROI)
