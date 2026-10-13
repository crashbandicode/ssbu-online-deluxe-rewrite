import unittest
from pathlib import Path
from analyze_render_trace import analyze, pointer_from_halves, thread_label

HEADER = """# schema=1
# metric=cpu_api_spans
# tick_hz=19200000
# id=1
# build=test
# label=trial
# engine=LessLag core
# reason=elapsed
# skipped=0
# event_count=3
event,kind,begin_tick,end_tick,index,result
"""

def trace(rows):
    return HEADER + rows + "# complete=1\n"

class RenderTraceTest(unittest.TestCase):
    def test_spans_and_call_intervals_are_distinct(self):
        result = analyze(trace("0,1,100,192100,0,-1\n1,5,50,192200,-1,-1\n2,1,192000100,192192100,1,-1\n"))
        self.assertEqual(result["spans"]["present_call"]["mean_ms"], 10)
        self.assertEqual(result["present_call_intervals"]["mean_ms"], 10000)
        self.assertEqual(result["spans"]["present_call"]["observed_indices"], [0,1])

    def test_partial_and_invalid_spans_are_rejected(self):
        good = trace("0,1,100,200,0,-1\n1,2,201,202,0,-1\n2,1,192000100,192000200,1,-1\n")
        for bad in [good.removesuffix("# complete=1\n"), good.replace("elapsed","capacity"),
                    good.replace("100,200","200,100"), good.replace("# event_count=3","# event_count=4")]:
            with self.assertRaises(ValueError):
                analyze(bad)

    def test_skips_and_mode_changes_are_not_hidden(self):
        data = trace("0,1,100,200,0,-1\n1,2,201,202,0,-1\n2,1,192000100,192000200,1,-1\n")
        data = data.replace("skipped=0","skipped=1").replace("LessLag core","Mixed")
        self.assertEqual(len(analyze(data)["warnings"]),2)

    def test_signed_halves_rebuild_handle_and_buffer(self):
        self.assertEqual(pointer_from_halves(-2147483648, 1), 0x0000000180000000)
        self.assertEqual(pointer_from_halves(0x40, -65536), 0xFFFF000000000040)
        self.assertEqual(pointer_from_halves(-1, -1), 0xFFFFFFFFFFFFFFFF)

    def test_zero_spans_are_counts_and_recording_absence_is_unknown(self):
        rows = "\n".join([
            "0,1,100,200,0,-1",
            "1,18,300,300,0,3",
            "2,27,400,440,-1,1",
            "3,1,192000100,192000200,1,-1",
        ])
        result = analyze(self._trace(rows, 4))
        self.assertNotIn("menu_draw", result["spans"])
        self.assertEqual(result["point_samples"]["menu_draw"]["count"], 1)
        self.assertIn("never GPU time", result["notes"][-1])
        self.assertAlmostEqual(result["spans"]["sync_infinite_other"]["mean_ms"], 40 * 1000 / 19200000)
        self.assertEqual(result["end_recording"]["redirect"], "unknown")
        self.assertIsNone(result["end_recording"]["inside_ui_draw"])
        self.assertIsNone(result["end_recording"]["other_use"])
        self.assertNotIn("unpaired", result["end_recording"])
        self.assertIn("CPU-side guest present cadence", result["summary"])
        self.assertIn("not GPU completion or scanout", result["summary"])

    def test_end_recording_inside_draw_is_separate_from_other_use(self):
        rows = "\n".join([
            "0,1,100,200,0,-1",
            "1,18,500,500,0,3",
            "2,24,600,600,0,3",
            "3,25,600,600,-2147483648,1",
            "4,26,600,600,64,-65536",
            "5,22,700,700,0,25",
            "6,24,900,900,1,3",
            "7,25,900,900,16,0",
            "8,26,900,900,32,0",
            "9,1,192000100,192000200,1,-1",
        ])
        result = analyze(self._trace(rows, 10))
        recording = result["end_recording"]
        self.assertEqual(recording["redirect"], "samples_present")
        self.assertEqual(recording["inside_ui_draw"]["count"], 1)
        self.assertEqual(recording["inside_ui_draw"]["handles"], ["0x0000000180000000"])
        self.assertEqual(recording["inside_ui_draw"]["buffers"], ["0xffff000000000040"])
        self.assertEqual(recording["other_use"]["count"], 1)
        self.assertEqual(recording["other_use"]["handles"], ["0x0000000000000010"])
        self.assertEqual(recording["other_use"]["buffers"], ["0x0000000000000020"])
        self.assertIn("CPU timing boundary", recording["note"])
        self.assertIn("not a GPU command lifetime", recording["note"])
        self.assertNotIn("menu_end_recording", result["spans"])

    def test_incomplete_end_recording_is_not_a_zero_pass(self):
        rows = "\n".join([
            "0,1,100,200,0,-1",
            "1,24,300,300,0,3",
            "2,1,192000100,192000200,1,-1",
        ])
        recording = analyze(self._trace(rows, 3))["end_recording"]
        self.assertEqual(recording["redirect"], "incomplete")
        self.assertIsNone(recording["inside_ui_draw"])
        self.assertIsNone(recording["other_use"])
        self.assertEqual(recording["unpaired"], 1)
        self.assertIn("not a zero EndRecording pass", recording["note"])

    def test_present_intervals_report_spike_runs_and_excess(self):
        # 10, 20, 30, 10, 40 ms between kind 1 begin ticks.
        begins = [1_000_000]
        for delta in (192_000, 384_000, 576_000, 192_000, 768_000):
            begins.append(begins[-1] + delta)
        lines = []
        for n, begin in enumerate(begins):
            lines.append(f"{n},1,{begin},{begin + 100},{n % 2},-1")
        lines.append(f"{len(begins)},5,{begins[0]},{begins[0] + 192000000},-1,-1")
        rows = "\n".join(lines)
        intervals = analyze(self._trace(rows, len(lines)))["present_call_intervals"]
        self.assertEqual(intervals["gt_16_6667_ms"], {"count": 3, "fraction": 0.6})
        self.assertEqual(intervals["gt_20_ms"], {"count": 2, "fraction": 0.4})
        self.assertEqual(intervals["gt_25_ms"], {"count": 2, "fraction": 0.4})
        self.assertEqual(intervals["gt_33_3333_ms"], {"count": 1, "fraction": 0.2})
        self.assertEqual(intervals["longest_run_gt_16_6667"], 2)
        self.assertAlmostEqual(intervals["excess_ms_above_16_6667"], 90 - 3 * 16.6667)
        self.assertIn("p95_ms", intervals)
        self.assertIn("p99_ms", intervals)
        self.assertIn("max_ms", intervals)

    def test_zero_duration_real_wait_stays_in_span_statistics(self):
        rows = "\n".join([
            "0,1,100,200,0,-1",
            "1,3,300,300,-1,0",
            "2,3,300,192300,-1,1",
            "3,5,400,400,-1,0",
            "4,27,500,500,-1,0",
            "5,18,600,600,0,0",
            "6,17,700,700,0,0",
            "7,1,192000100,192000200,1,-1",
        ])
        result = analyze(self._trace(rows, 8))
        wait = result["spans"]["texture_cpu_wait"]
        self.assertEqual(wait["count"], 2)
        self.assertAlmostEqual(wait["mean_ms"], 5)
        self.assertEqual(wait["native_results"], [0, 1])
        self.assertEqual(wait["observed_indices"], [-1])
        self.assertEqual(result["spans"]["dispatch_flush"]["mean_ms"], 0)
        self.assertEqual(result["spans"]["dispatch_flush"]["native_results"], [0])
        self.assertEqual(result["spans"]["sync_infinite_other"]["mean_ms"], 0)
        self.assertEqual(result["spans"]["sync_infinite_other"]["native_results"], [0])
        self.assertNotIn("menu_draw", result["spans"])
        self.assertNotIn("outstanding_after_present", result["spans"])
        draw = result["point_samples"]["menu_draw"]
        self.assertEqual(draw["observed_indices"], [0])
        self.assertEqual(draw["native_results"], [0])
        outstanding = result["point_samples"]["outstanding_after_present"]
        self.assertEqual(outstanding["observed_indices"], [0])
        self.assertEqual(outstanding["native_results"], [0])

    def test_menu_gpu_poll_stays_a_point_sample(self):
        rows = "\n".join([
            "0,1,100,200,0,-1",
            "1,35,300,300,25,1",
            "2,37,400,400,4,2",
            "3,38,400,400,-1,-1",
            "4,1,192000100,192000200,1,-1",
        ])
        result = analyze(self._trace(rows, 5))
        poll = result["point_samples"]["menu_gpu_poll"]
        self.assertEqual(poll["observed_indices"], [4])
        self.assertEqual(poll["native_results"], [2])
        self.assertNotIn("menu_gpu_poll", result["spans"])
        self.assertEqual(result["point_samples"]["menu_gpu_fence"]["native_results"], [-1])
        self.assertIn("timeout-0", " ".join(result["notes"]))
        self.assertIn("submit_incoming", result["point_samples"])

    def test_visible_9e9_capture_parses_without_end_recording(self):
        path = Path("/tmp/delay-audit-01a10c4e-93fe-7db0-9d89-4bcd50bf2340/peer-native-9e9/trace-less-css/api-4185679306935.csv")
        if not path.is_file():
            self.skipTest("visible 9e9 capture is not in this checkout")
        result = analyze(path.read_text())
        self.assertEqual(result["label"], "peer9e9-less-loaded")
        self.assertEqual(result["event_count"], 6720)
        self.assertEqual(result["engine"], "LessLag")
        self.assertEqual(result["warnings"], [])
        self.assertGreater(result["point_samples"]["menu_draw"]["count"], 0)
        self.assertGreater(result["point_samples"]["menu_submit_before"]["count"], 0)
        self.assertNotIn("menu_draw", result["spans"])
        self.assertNotIn("menu_submit_before", result["spans"])
        self.assertIn(0, result["point_samples"]["menu_draw"]["observed_indices"])
        self.assertIn(0, result["point_samples"]["outstanding_after_present"]["observed_indices"])
        for name in result["point_samples"]:
            self.assertNotIn(name, result["spans"])
        recording = result["end_recording"]
        self.assertEqual(recording["redirect"], "unknown")
        self.assertIsNone(recording["inside_ui_draw"])
        self.assertIsNone(recording["other_use"])
        self.assertIn("gt_16_6667_ms", result["present_call_intervals"])
        self.assertIn("longest_run_gt_16_6667", result["present_call_intervals"])
        self.assertIn("excess_ms_above_16_6667", result["present_call_intervals"])
        self.assertIn("scanout", result["not_measured"])

    def _css_rows(self):
        label = thread_label(0x1000)
        counter = 0x5333160
        rows = [
            "1,100,200,0,-1",
            "60,300,300,2,4000",
            "60,400,400,4,6000",
            "61,300,300,3000,1",
            "62,500,500,4096,0",
            "63,500,500,5,3843",
            "62,600,600,0,0",
            "63,600,600,-1,-1",
            "64,700,700,261,%d" % label,
            "64,710,710,261,%d" % label,
            "64,720,720,2,%d" % label,
            "66,800,1600,%d,%d" % (counter, label),
            "66,900,1700,%d,12345" % counter,
            "66,900,1000,-1,12345",
            "77,800,2600,-1,%d" % label,
            "1,192000100,192000200,1,-1",
        ]
        return "\n".join(f"{n},{row}" for n, row in enumerate(rows)), len(rows)

    def test_thread_label_matches_the_formula(self):
        def formula(pointer):
            mask = (1 << 64) - 1
            mixed = ((pointer + 0x9E3779B185EBCA87) & mask) * 0x9E3779B185EBCA87 & mask
            mixed = ((mixed << 27) | (mixed >> 37)) & mask
            return (mixed & 0xFFFFFFFF) & 0x7FFFFFFF
        self.assertEqual(thread_label(0), 137139250)
        self.assertEqual(thread_label(0x1000), 199847157)
        for pointer in (0, 0x1000, 0xFFFFFFFFFFFFFFFF, 0x7100ABCDEF00):
            self.assertEqual(thread_label(pointer), formula(pointer))
            self.assertGreaterEqual(thread_label(pointer), 0)

    def test_css_kinds_classify_as_spans_and_points(self):
        rows, count = self._css_rows()
        result = analyze(self._trace(rows, count))
        for name in ("dependency_wait", "ui_job"):
            self.assertIn(name, result["spans"])
            self.assertNotIn(name, result["point_samples"])
        for name in ("backoff_frame_sleep", "backoff_frame_peak", "thread_setup_pointer",
                     "thread_setup_info", "ui_dispatch"):
            self.assertIn(name, result["point_samples"])
            self.assertNotIn(name, result["spans"])
        self.assertEqual(result["spans"]["dependency_wait"]["count"], 3)
        self.assertEqual(result["spans"]["ui_job"]["count"], 1)
        self.assertEqual(result["point_samples"]["thread_setup_pointer"]["count"], 2)
        self.assertIn("60-64", result["notes"][-1])

    def test_css_capture_section(self):
        rows, count = self._css_rows()
        css = analyze(self._trace(rows, count))["css_capture"]
        label = thread_label(0x1000)
        waits = css["dependency_wait_by_counter"]
        self.assertEqual(sorted(waits), ["-1", "0x5333160"])
        ui_wait = waits["0x5333160"]
        self.assertEqual(ui_wait["count"], 2)
        self.assertEqual(ui_wait["thread_labels"], sorted([label, 12345]))
        self.assertAlmostEqual(ui_wait["max_ms"], 800 * 1000 / 19200000)
        self.assertEqual(waits["-1"]["thread_labels"], [12345])
        self.assertEqual(css["ui_job"]["count"], 1)
        self.assertEqual(css["ui_job"]["thread_labels"], [label])
        self.assertAlmostEqual(css["ui_job"]["mean_ms"], 1800 * 1000 / 19200000)
        self.assertEqual(css["nested_waits"], 1)
        backoff = css["backoff_per_frame"]
        self.assertEqual(backoff["sleeps"]["mean"], 3)
        self.assertEqual(backoff["sleeps"]["max"], 4)
        self.assertEqual(backoff["sleep_us"]["p50"], 5000)
        self.assertEqual(backoff["sleep_us"]["max"], 6000)
        self.assertEqual(backoff["peak_sleep_us_max"], 3000)
        self.assertEqual(css["threads"], [
            {"pointer": "0x0000000000001000", "label": label, "priority": 5,
             "core_info": 3843, "ideal_core": 3, "affinity_mask": 0x0F},
            {"pointer": "0x0000000000000000", "label": thread_label(0), "priority": -1,
             "core_info": None, "ideal_core": None, "affinity_mask": None},
        ])
        self.assertEqual(css["ui_dispatch"]["count"], 3)
        self.assertEqual(css["ui_dispatch"]["worker_priority_pairs"], [(2, 0), (5, 1)])

    def test_css_capture_is_absent_on_an_old_trace(self):
        rows = "0,1,100,200,0,-1\n1,18,300,300,0,3\n2,1,192000100,192000200,1,-1"
        self.assertNotIn("css_capture", analyze(self._trace(rows, 3)))

    def _trace(self, rows, count):
        header = HEADER.replace("# event_count=3", f"# event_count={count}")
        return header + rows + "\n# complete=1\n"

if __name__ == "__main__":
    unittest.main()