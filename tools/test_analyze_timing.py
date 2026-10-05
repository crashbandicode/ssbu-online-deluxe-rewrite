import unittest
from analyze_timing import analyze


def trace(extra_at=None, gap_at=None, mixed=False):
    lines = ["# schema=1", "# metric=render_callback", "# engine=Vanilla", "# tick_hz=19200000",
             "# duration_ticks=192000000", "# id=1", "# build=test", "# label=synthetic",
             "# reason=elapsed", "# skipped=0", "# sample_count=601",
             "sample,callback,tick,buttons,fx_mode,fx_scope,fx_effective_off,overlay_view"]
    for n in range(601):
        tick = 100 + n * 320000 + (320000 if extra_at is not None and n >= extra_at else 0)
        callback = n + (1 if gap_at is not None and n >= gap_at else 0)
        mode = int(mixed and n > 300)
        lines.append(f"{n},{callback},{tick},0,{mode},1,{mode},0")
    return "\n".join(lines + ["# complete=1", ""])


class TimingAnalysis(unittest.TestCase):
    def test_normal_sixty_hz_and_single_long_interval(self):
        normal = analyze(trace())
        self.assertEqual(normal["samples"], 601)
        self.assertAlmostEqual(normal["p50_ms"], 1000 / 60)
        self.assertEqual(normal["callback_gaps_over_ms"]["25"], 0)
        delayed = analyze(trace(extra_at=200))
        self.assertAlmostEqual(delayed["max_ms"], 1000 / 30)
        self.assertEqual(delayed["callback_gaps_over_ms"]["25"], 1)

    def test_missing_callbacks_are_excluded_not_reported_as_slow_frames(self):
        report = analyze(trace(extra_at=200, gap_at=200))
        self.assertEqual(report["missing_callbacks"], 1)
        self.assertEqual(report["excluded_intervals"], 1)
        self.assertEqual(report["callback_gaps_over_ms"]["25"], 0)
        self.assertTrue(report["warnings"])

    def test_mixed_conditions_are_flagged(self):
        self.assertEqual(len(analyze(trace(mixed=True))["conditions"]), 2)
        self.assertTrue(analyze(trace(mixed=True))["warnings"])

    def test_truncation_invalid_clock_and_short_capture_are_rejected(self):
        for text in (trace().replace("# complete=1\n", ""),
                     trace().replace("1,1,320100,", "1,1,100,"),
                     trace().replace("# sample_count=601", "# sample_count=600"),
                     trace().replace("# duration_ticks=192000000", "# duration_ticks=384000000"),
                     trace().replace("# reason=elapsed", "# reason=aborted")):
            with self.assertRaises(ValueError):
                analyze(text)


if __name__ == "__main__":
    unittest.main()
