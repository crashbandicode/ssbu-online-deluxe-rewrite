#!/usr/bin/env python3
"""Authored segments for the imgui-standalone 1.1.0 boot (same format as segments/open-deluxe and dpad-up).

Buttons are Eden's macro mask, the NpadButton bits: L3 0x10, R3 0x20, ZR 0x200, Left 0x1000, Right 0x4000,
Down 0x8000 (open-deluxe is L+R+Down = 0x80c0, dpad-up is Up = 0x2000). Chords go down in one frame (MACROS rule 14).
  zr-right / zr-left  ZR held, then Left/Right pressed under it: a Deluxe Settings edit
  dpad-down           one Down tap: the selection moves one row
  l3-r3               both stick clicks in one frame: imgui-smash's gamepad-navigation chord
Usage: make_segments_ims.py <segments dir>; prints each file's sha256.
"""
import hashlib
import json
import sys
from pathlib import Path

MOTION = [{"accel": [0, 0, -1], "delta_us": 1000, "gyro": [0, 0, 0]}] * 2


def event(frame, buttons):
    return {"buttons": buttons, "frame": frame, "motion": MOTION, "sticks": [0, 0, 0, 0], "triggers": [0, 0]}


def macro(name, events, duration):
    return {"bindings": {"gamepad": "", "keyboard": ""}, "duration_frames": duration, "events": events,
            "format": "eden-input-macro", "game_id": "01006A800016E000", "markers": [], "name": name,
            "parent": {"marker": "", "name": ""}, "player": 0, "version": 1}


SEGMENTS = {
    "zr-right": macro("zr-right", [event(0, 0x200), event(6, 0x200 | 0x4000), event(14, 0x200), event(20, 0)], 40),
    "zr-left": macro("zr-left", [event(0, 0x200), event(6, 0x200 | 0x1000), event(14, 0x200), event(20, 0)], 40),
    "dpad-down": macro("dpad-down", [event(0, 0x8000), event(8, 0)], 30),
    "l3-r3": macro("l3-r3", [event(0, 0x10 | 0x20), event(10, 0)], 30),
}


def main(argv):
    out = Path(argv[1])
    out.mkdir(parents=True, exist_ok=True)
    for name, data in SEGMENTS.items():
        path = out / f"{name}.edenmacro.json"
        path.write_text(json.dumps(data, separators=(",", ":"), sort_keys=True))
        print(name, hashlib.sha256(path.read_bytes()).hexdigest())
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
