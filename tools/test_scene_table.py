"""Host tests for scene_table.py: python3 -B -I eden/scene-online/test_scene_table.py"""
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import scene_table  # noqa: E402

T = scene_table.TICKS
BASE = 1000 * T          # the first heartbeat's tick, at hang t=5.0


def ev(seq, t, code, a, b):
    return '%d %d %d %#x %#x' % (seq, BASE + int((t - 5.0) * T), code, a, b)


def beat(t, scene):
    return '[Heartbeat] t=%.3f tick=%#x scene=%s ready=1' % (t, BASE + int((t - 5.0) * T), scene)


def ctx(conn, status):
    return conn | status << 8 | 1 << 16


STAGE = 0x25D9288
# imp-oppb10's rows, re-timed: the waiting room at 123 s, the battle at 174.3 s, the stale demotion 5.3 s in, the
# latency mark 14.1 s in, results at 383.5 s.
BEFORE_EVENTS = '\n'.join([
    'Deluxe rewrite 0.10.5-6c05693130a4-1794576222',
    ev(1, 123.0, 107, 4, 0x07010101), ev(2, 123.0, 30, 1, STAGE), ev(3, 123.0, 73, ctx(3, 0), STAGE),
    ev(4, 123.0, 110, 0x137, 1), ev(5, 123.0, 116, 0x30D, 0x07010101), ev(6, 123.0, 73, ctx(3, 1), STAGE),
    ev(7, 174.3, 110, 0x15B, 2), ev(8, 174.3, 116, 0x30E, 0x07010101), ev(9, 174.3, 73, ctx(3, 2), STAGE),
    ev(10, 179.6, 107, 2, 0x5FF28BB), ev(11, 179.6, 30, 0, 0x0FF11E), ev(12, 179.6, 73, ctx(0, 2), 0x0FF11E),
    ev(13, 188.4, 30, 1, 0x16CCAB8),
    ev(14, 383.5, 110, 0x162, 0), ev(15, 383.5, 116, 0x30E, 0x07010101), ev(16, 383.5, 73, ctx(3, 0), STAGE),
])
BEFORE_HANG = '\n'.join([
    beat(5.0, 'offline/inactive'),
    '[HangTrace] scene offline/inactive -> quickplay/training t=123.170 tick=0x0',
    beat(170.9, 'quickplay/training'),
    '[HangTrace] scene quickplay/training -> quickplay/singles t=174.387 tick=0x0',
    beat(176.6, 'quickplay/singles'),
    '[HangTrace] scene quickplay/singles -> offline/singles t=179.523 tick=0x0',
    beat(182.3, 'offline/singles'), beat(250.0, 'offline/singles'), beat(380.0, 'offline/singles'),
    '[HangTrace] scene offline/singles -> quickplay/inactive t=383.531 tick=0x0',
    beat(385.0, 'quickplay/inactive'),
])
# The fix: no demotion, no context change; the mark's code-30 row is not logged under an online connection.
AFTER_EVENTS = '\n'.join(l for l in BEFORE_EVENTS.splitlines()
                         if not any(l.startswith('%d ' % s) for s in (10, 11, 12, 13)))
AFTER_HANG = '\n'.join(l.replace('offline/singles', 'quickplay/singles') for l in BEFORE_HANG.splitlines()
                       if 'quickplay/singles -> offline/singles' not in l)


class SceneTable(unittest.TestCase):
    def test_before_the_fix_the_battle_is_demoted(self):
        r = scene_table.analyse(BEFORE_HANG, BEFORE_EVENTS, {'000006-singles-15b.log': 'x\nconnection=offline'})
        self.assertEqual(r['waiting']['scene'], 'quickplay/training')
        b = r['battle']
        self.assertEqual(b['stage'], '0x15b')
        self.assertEqual(b['before'], 'quickplay/training')
        self.assertEqual(b['scene_at_load'], 'quickplay/singles')
        self.assertTrue(b['demoted'])
        self.assertTrue(b['context_off'])
        self.assertEqual(b['scene_holds_107'], ['a=2 @+5.3'])
        self.assertEqual(b['online_context_30'], ['online=0 src=stale_check @+5.3', 'online=1 src=latency_hook @+14.1'])
        self.assertEqual(b['heartbeats'], {'quickplay/singles': 1, 'offline/singles': 3})
        self.assertEqual(len(r['battles']), 1)
        self.assertIn('quickplay/singles -> offline/singles @+5.2', b['scene_changes'])
        self.assertEqual(r['results']['scene'], 'quickplay/inactive')
        ok, why = scene_table.battle_online(r)
        self.assertFalse(ok)
        self.assertIn('stale demotion', why)
        self.assertIn('stage logs', why)

    def test_after_the_fix_the_battle_reads_quickplay(self):
        r = scene_table.analyse(AFTER_HANG, AFTER_EVENTS, {'000006-singles-15b.log': 'x\nconnection=quickplay'})
        b = r['battle']
        self.assertEqual(b['scene_at_load'], 'quickplay/singles')
        self.assertFalse(b['demoted'])
        self.assertFalse(b['context_off'])
        self.assertEqual(b['heartbeats'], {'quickplay/singles': 4})
        self.assertEqual(scene_table.battle_online(r), (True, 'quickplay through the battle, online bit on'))

    def test_a_sudden_death_is_a_second_battle_and_the_next_stage_rows_stay_out(self):
        # imp-oppb10: a second battle at 347.1 s, promoted again from the demoted Offline (107 a=4 with its own
        # 30/73 rows logged just before its 110 row), then the results stage at 383.5 s.
        lines = BEFORE_EVENTS.splitlines()
        second = [ev(20, 347.1, 107, 4, 0x07010101), ev(21, 347.1, 30, 1, STAGE), ev(22, 347.1, 73, ctx(3, 2), STAGE),
                  ev(23, 347.1, 110, 0x15B, 2), ev(24, 347.1, 116, 0x30E, 0x07010101)]
        events = '\n'.join(lines[:14] + second + lines[14:])
        r = scene_table.analyse(BEFORE_HANG, events)
        self.assertEqual(len(r['battles']), 2)
        first, again = r['battles']
        self.assertEqual(first['scene_holds_107'], ['a=2 @+5.3'])
        self.assertEqual(first['seconds'], 172.8)
        self.assertEqual(again['before'], 'offline/singles')
        self.assertEqual(again['scene_holds_107'], ['a=4 @+0.0'])
        self.assertEqual(again['scene_at_load'], 'quickplay/singles')
        self.assertEqual(r['results'], {'t': 383.5, 'stage': '0x162', 'scene': 'quickplay/inactive'})

    def test_no_online_battle(self):
        events = '\n'.join(BEFORE_EVENTS.splitlines()[:7])
        r = scene_table.analyse(BEFORE_HANG, events)
        self.assertIsNone(r['battle'])
        self.assertFalse(scene_table.battle_online(r)[0])

    def test_offline_words_are_not_an_online_battle(self):
        self.assertTrue(scene_table.online_words(0x0E, 0x07010101))
        self.assertTrue(scene_table.online_words(0x10, 0x87010400))
        for melee, mode in ((0x00, 0x07010101), (0x20, 0x04040000), (0x0E, 0x02010000)):
            self.assertFalse(scene_table.online_words(melee, mode))


if __name__ == '__main__':
    unittest.main()
