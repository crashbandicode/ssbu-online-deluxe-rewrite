"""Decode the opt-in in-game HID/present marker; reject ambiguous captures."""
from pathlib import Path
from PIL import Image


def decode_marker(path):
    with Image.open(path) as source:
        image = source.convert("RGB")
        width, height = image.size
        cells = [
            image.getpixel((int((1112 + 24 * i) * width / 1920),
                            int(1026 * height / 1080)))
            for i in range(28)
        ]
    magenta, cyan = cells[:2]
    if not (magenta[0] > 220 and magenta[1] < 35 and magenta[2] > 220
            and cyan[0] < 35 and cyan[1] > 220 and cyan[2] > 220):
        raise ValueError(f"{path}: marker anchors absent")
    bits = []
    for value in cells[2:]:
        if min(value) > 220:
            bits.append(1)
        elif max(value) < 35:
            bits.append(0)
        else:
            raise ValueError(f"{path}: ambiguous marker cell {value}")
    if sum(bits[:-1]) % 2 != bits[-1]:
        raise ValueError(f"{path}: marker parity failure")
    payload = sum(bit << n for n, bit in enumerate(bits[:-1]))
    return {"serial": payload & 0xffffff, "down": bool(payload & (1 << 24))}


def check_sequence(markers):
    """One present identity per PNG is required for an exact frame-count trial."""
    if len(markers) < 2:
        raise ValueError("at least two marked images required")
    for a, b in zip(markers, markers[1:]):
        delta = (b["serial"] - a["serial"]) & 0xffffff
        if delta != 1:
            raise ValueError(f"nonconsecutive present identity: {a['serial']} -> {b['serial']} ({delta})")


if __name__ == "__main__":
    import argparse
    import json
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    frames = sorted(args.directory.glob("[0-9][0-9][0-9].png"))
    markers = [dict(png=p.name, **decode_marker(p)) for p in frames]
    check_sequence(markers)
    print(json.dumps({"metric": "captured_present_identity_and_plugin_hid_observation",
                      "frames": markers, "consecutive": True}, indent=2))