"""Crop the main menu's right-hand icon column (focus-independent) from a full main-menu frame (1280x674) into
refs/main-sidebar.png; qp_steps._back_to_main uses it to see the main menu whichever panel has the focus."""
import sys
from pathlib import Path

from PIL import Image

ROI = (1140, 85, 1235, 510)
src = Path(sys.argv[1])
out = Path(__file__).resolve().parents[1] / 'refs' / 'main-sidebar.png'
with Image.open(src) as im:
    assert im.size == (1280, 674), im.size
    im.convert('RGB').crop(ROI).save(out)
print(out, ROI)
