import unittest

from heartbeat_memory import MIB, compare, phases, scenes, scenes_table, table


def beat(t, scene, used_mib, css=0):
    used = int(used_mib * MIB)
    return (f"[Heartbeat] t={t:.3f} tick=0x1 scene={scene} ready=1 css={css} trace=0 mem_used={used} "
            f"mem_total={3285 * MIB} mem_free={3285 * MIB - used} heap_region=8589934592 present=1")


def log(lines):
    return "build=test\nboot_tick=0x1 tick_hz=19200000\n" + "\n".join(lines) + "\n"


SAMPLE = log([
    beat(5.0, "offline/inactive", 220.0),
    beat(100.0, "offline/inactive", 2100.0),
    beat(110.0, "offline/inactive", 2110.0),
    beat(120.0, "offline/inactive", 2120.0),
    "[HangTrace] scene offline/inactive -> quickplay/inactive t=125.000 tick=0x2",
    beat(126.0, "quickplay/inactive", 2050.0),
    beat(131.0, "quickplay/inactive", 2380.0, css=1),
    beat(136.0, "quickplay/inactive", 2390.0, css=1),
    beat(138.0, "quickplay/inactive", 2450.0),
    "[HangTrace] scene quickplay/inactive -> quickplay/training t=140.000 tick=0x3",
    beat(142.0, "quickplay/training", 2600.0),
    beat(150.0, "quickplay/training", 2880.0),
    beat(160.0, "quickplay/training", 2882.0),
    beat(170.0, "quickplay/training", 2884.0),
    "[HangTrace] scene quickplay/training -> quickplay/inactive t=175.000 tick=0x4",
    beat(176.0, "quickplay/inactive", 2400.0, css=1),
])


class HeartbeatMemoryTest(unittest.TestCase):
    def test_phases_follow_the_files_own_scene_lines(self):
        result = phases(SAMPLE)
        # Boot's 220 MiB is outside the 30 s before the first change.
        self.assertEqual(result["menu"], {"n": 3, "min": 2100.0, "median": 2110.0, "max": 2120.0})
        self.assertEqual(result["css"], {"n": 2, "min": 2380.0, "median": 2385.0, "max": 2390.0})
        # Load: after the CSS (138, before the scene says training) and the first 15 s after the change (142, 150).
        self.assertEqual(result["load"], {"n": 3, "min": 2450.0, "median": 2600.0, "max": 2880.0})
        # Settled Training: 15 s after the change, up to the next change (the CSS beat after it is not Training).
        self.assertEqual(result["training"], {"n": 2, "min": 2882.0, "median": 2883.0, "max": 2884.0})
        self.assertEqual(result["peak"], 2884.0)
        self.assertEqual(result["training_entered_at"], 140.0)

    def test_a_planted_difference_shows_in_every_phase_and_none_without_it(self):
        heavier = "\n".join(line.replace(f"mem_used={int(m * MIB)}", f"mem_used={int((m + 3) * MIB)}")
                            for line in SAMPLE.splitlines()
                            for m in [_used_mib(line)])
        results = {"a": phases(SAMPLE), "same": phases(SAMPLE), "plus3": phases(heavier)}
        deltas = compare(results, "a")
        self.assertEqual(deltas["same"], {"menu": 0.0, "css": 0.0, "load": 0.0, "training": 0.0, "peak": 0.0})
        self.assertEqual(deltas["plus3"], {"menu": 3.0, "css": 3.0, "load": 3.0, "training": 3.0, "peak": 3.0})
        text = table(results, deltas, "a")
        self.assertIn("| a (ref) |", text)
        self.assertIn("2,886.0 (2,885.0-2,887.0, n=2) +3.0", text)

    def test_a_boot_that_never_reaches_training_has_no_training_phase(self):
        cut = "\n".join(SAMPLE.splitlines()[:10]) + "\n"  # up to the second CSS heartbeat
        result = phases(cut)
        self.assertIsNone(result["training"])
        self.assertIsNone(result["load"])
        self.assertIsNone(result["training_entered_at"])
        self.assertEqual(result["css"]["n"], 2)
        deltas = compare({"a": phases(SAMPLE), "hang": result}, "a")
        self.assertIsNone(deltas["hang"]["training"])

    def test_a_file_without_heartbeats_is_refused(self):
        with self.assertRaises(ValueError):
            phases("build=test\n")
        with self.assertRaises(ValueError):
            scenes("build=test\n")

    def test_scene_runs_split_on_scene_and_css_in_file_order(self):
        runs = scenes(SAMPLE)
        self.assertEqual([(r["scene"], r["css"], r["n"]) for r in runs], [
            ("offline/inactive", 0, 4),
            ("quickplay/inactive", 0, 1),
            ("quickplay/inactive", 1, 2),
            ("quickplay/inactive", 0, 1),
            ("quickplay/training", 0, 4),
            ("quickplay/inactive", 1, 1),
        ])
        self.assertEqual(runs[2], {"scene": "quickplay/inactive", "css": 1, "t_start": 131.0, "t_end": 136.0,
                                   "n": 2, "min": 2380.0, "median": 2385.0, "max": 2390.0})
        self.assertEqual(runs[4]["median"], 2881.0)
        self.assertIn("| quickplay/training | 0 | 142.0-170.0 | 2,881.0 | 2,600.0-2,884.0 | 4 |",
                      scenes_table("a", runs))


def _used_mib(line):
    marker = "mem_used="
    if marker not in line:
        return 0
    return int(line.split(marker, 1)[1].split()[0]) / MIB


if __name__ == "__main__":
    unittest.main()
