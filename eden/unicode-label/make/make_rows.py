#!/usr/bin/env python3
"""Assemble eden/rows-dun: QCF's steps.py, segments and references (copied unchanged, with sha256), ROI-only
references cropped from QCF's full frames (for the runner's scale "fit" half-size compare), and the authored
open-deluxe chord segment. Writes eden/rows-dun/provenance.json."""
import hashlib
import json
import shutil
from pathlib import Path

from PIL import Image

QCF = Path('/home/intpa/eden-runs/qcf-srm02/rows')
OUT = Path('/home/intpa/deluxe-unicode-names-work/eden/rows-dun')
SEGMENTS = ['to-online', 'to-quickplay', 'pick', 'tile-up', 'to-css', 'ready']
REFS = ['main.png', 'quickplay-menu.png', 'ready-banner.png', 'rematch-dialog.png', 'leave-battle-tab.png',
        'waiting-menu.png', 'online-menu.png', 'css-quickplay.png']
ROI_REFS = {'online-menu-roi.png': ('online-menu.png', [40, 5, 300, 45]),
            'quickplay-menu-roi.png': ('quickplay-menu.png', [40, 5, 300, 45]),
            'ready-banner-roi.png': ('ready-banner.png', [200, 110, 600, 240])}
MOTION = [{'accel': [0, 0, -1], 'delta_us': 1000, 'gyro': [0, 0, 0]}] * 2


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    (OUT / 'segments').mkdir(parents=True, exist_ok=True)
    (OUT / 'refs').mkdir(parents=True, exist_ok=True)
    prov = {'source': str(QCF), 'copied': {}, 'derived': {}, 'authored': {}}
    shutil.copyfile(QCF / 'steps.py', OUT / 'steps.py')
    prov['copied']['steps.py'] = sha(OUT / 'steps.py')
    for name in SEGMENTS:
        shutil.copyfile(QCF / 'segments' / f'{name}.edenmacro.json', OUT / 'segments' / f'{name}.edenmacro.json')
        prov['copied'][f'segments/{name}.edenmacro.json'] = sha(OUT / 'segments' / f'{name}.edenmacro.json')
    for name in REFS:
        shutil.copyfile(QCF / 'refs' / name, OUT / 'refs' / name)
        prov['copied'][f'refs/{name}'] = sha(OUT / 'refs' / name)
    for name, (source, roi) in ROI_REFS.items():
        with Image.open(QCF / 'refs' / source) as im:
            im.convert('RGB').crop(tuple(roi)).save(OUT / 'refs' / name)
        prov['derived'][f'refs/{name}'] = {'from': f'refs/{source}', 'roi': roi, 'sha256': sha(OUT / 'refs' / name)}
    # L (bit 6) + R (bit 7) + D-pad Down (bit 15), NpadButton bits, in one frame, held 12 frames.
    chord = {'bindings': {'gamepad': '', 'keyboard': ''}, 'duration_frames': 40,
             'events': [{'buttons': (1 << 6) | (1 << 7) | (1 << 15), 'frame': 0, 'motion': MOTION,
                         'sticks': [0, 0, 0, 0], 'triggers': [0, 0]},
                        {'buttons': 0, 'frame': 12, 'motion': MOTION, 'sticks': [0, 0, 0, 0], 'triggers': [0, 0]}],
             'format': 'eden-input-macro', 'game_id': '01006A800016E000', 'markers': [], 'name': 'open-deluxe',
             'parent': {'marker': '', 'name': ''}, 'player': 0, 'version': 1}
    path = OUT / 'segments' / 'open-deluxe.edenmacro.json'
    path.write_text(json.dumps(chord, separators=(',', ':')))
    prov['authored']['segments/open-deluxe.edenmacro.json'] = sha(path)
    # D-pad Up (NpadButton bit 13), held 8 frames.
    up = dict(chord, name='dpad-up', duration_frames=30,
              events=[dict(chord['events'][0], buttons=1 << 13), dict(chord['events'][1], frame=8)])
    path = OUT / 'segments' / 'dpad-up.edenmacro.json'
    path.write_text(json.dumps(up, separators=(',', ':')))
    prov['authored']['segments/dpad-up.edenmacro.json'] = sha(path)
    (OUT / 'provenance.json').write_text(json.dumps(prov, indent=1) + '\n')
    print(json.dumps(prov, indent=1))


if __name__ == '__main__':
    main()
