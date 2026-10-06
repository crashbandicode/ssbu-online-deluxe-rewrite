"""Capture validity checks: unreadable pixels, corruption, gaps and wraparound."""
import tempfile
import unittest
from pathlib import Path

from PIL import Image, ImageDraw
from response_marker import check_sequence, decode_marker


class MarkerValidity(unittest.TestCase):
    def fixture(self, path, flip_parity=False, ambiguous=False):
        # Fixed independent wire fixture: serial0xA50003, stick down.
        bits = [1, 1] + [0] * 14 + [1, 0, 1, 0, 0, 1, 0, 1] + [1]
        parity = 1  # serial has6 set bits, input adds1
        image = Image.new("RGB", (640, 360), (90, 90, 90))
        draw = ImageDraw.Draw(image)
        colors = [(255, 0, 255), (0, 255, 255)]
        colors += [(255, 255, 255) if b else (0, 0, 0) for b in bits]
        colors += [(255, 255, 255) if parity ^ flip_parity else (0, 0, 0)]
        if ambiguous:
            colors[10] = (130, 130, 130)
        for i, color in enumerate(colors):
            x = int((1112 + 24 * i) / 3)
            draw.rectangle((x - 2, 340, x + 2, 344), fill=color)
        image.save(path)

    def test_known_png_payload(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp) / "fixture.png"
            self.fixture(p)
            self.assertEqual(decode_marker(p), {"serial": 0xA50003, "down": True})

    def test_corruption_and_ambiguous_pixels_are_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp) / "fixture.png"
            for options in ({"flip_parity": True}, {"ambiguous": True}):
                self.fixture(p, **options)
                with self.assertRaises(ValueError):
                    decode_marker(p)

    def test_no_marker_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp) / "blank.png"
            Image.new("RGB", (640, 360), "black").save(p)
            with self.assertRaisesRegex(ValueError, "anchors absent"):
                decode_marker(p)

    def test_gaps_and_duplicates_are_rejected_and_wrap_is_supported(self):
        check_sequence([{"serial": 0xfffffe}, {"serial": 0xffffff}, {"serial": 0}])
        for serials in ([1, 1], [1, 3], [9, 8]):
            with self.assertRaises(ValueError):
                check_sequence([{"serial": n} for n in serials])


if __name__ == "__main__":
    unittest.main()