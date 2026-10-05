import unittest
from analyze_render_trace import analyze

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

if __name__ == "__main__":
    unittest.main()