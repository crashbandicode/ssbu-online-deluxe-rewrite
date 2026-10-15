import tempfile
import unittest
from pathlib import Path

import frame_windows as fw

HZ = 19_200_000
FRAME = HZ // 60


def capture(seconds=5, slow_second=None):
    """Synthetic schema-2 capture: 60 callbacks per second, optionally one
    second at 30 Hz."""
    lines = ["# schema=2", "# metric=render_callback", f"# tick_hz={HZ}", "# id=7",
             "# build=test", "# label=match_training", "sample,callback,tick"]
    tick, n = 1_000_000, 0
    for s in range(seconds):
        step = FRAME * 2 if s == slow_second else FRAME
        while tick < 1_000_000 + (s + 1) * HZ:
            lines.append(f"{n},{n},{tick}")
            tick += step
            n += 1
    return "\n".join(lines) + "\n"


class FrameWindows(unittest.TestCase):
    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.path = Path(self.dir.name) / "timing-7.csv"

    def tearDown(self):
        self.dir.cleanup()

    def test_steady_sixty_passes(self):
        self.path.write_text(capture())
        header, ticks = fw.read_capture(self.path)
        self.assertEqual(header["label"], "match_training")
        counts, _ = fw.per_second(ticks, HZ)
        self.assertEqual(counts[:5], [60, 60, 60, 60, 60])
        self.assertEqual(fw.worst_sliding_window(ticks, HZ)["count"], 60)

    def test_half_rate_second_is_found_by_count_and_window(self):
        self.path.write_text(capture(slow_second=2))
        _, ticks = fw.read_capture(self.path)
        counts, _ = fw.per_second(ticks, HZ)
        self.assertEqual(counts[2], 30)
        worst = fw.worst_sliding_window(ticks, HZ)
        self.assertEqual(worst["count"], 30)
        self.assertAlmostEqual(worst["offset_s"], 2.0, places=1)

    def test_partial_row_is_counted_not_parsed(self):
        self.path.write_text(capture() + "300,30")  # last row cut off mid-write
        header, ticks = fw.read_capture(self.path)
        self.assertEqual(header["malformed_rows"], 1)
        self.assertEqual(len(ticks), 300)

    def test_resolution_edges_inside_span_only(self):
        self.path.write_text(capture())
        _, ticks = fw.read_capture(self.path)
        events = Path(self.dir.name) / "events.log"
        inside = ticks[0] + 2 * HZ + FRAME
        events.write_text("\n".join([
            f"1 {ticks[0] - HZ} 82 10100 0",      # before the capture
            f"2 {inside} 82 10100 0",             # push inside second 2
            f"3 {inside + FRAME * 20} 108 1 1",   # pop inside second 2
            f"4 {ticks[-1] + HZ} 82 10200 0",     # after the capture
            "not a row",
        ]) + "\n")
        edges = fw.read_drs_edges(events, ticks[0], ticks[-1])
        self.assertEqual([e["code"] for e in edges], [82, 108])
        self.assertEqual({(e["tick"] - ticks[0]) // HZ for e in edges}, {2})


if __name__ == "__main__":
    unittest.main()
