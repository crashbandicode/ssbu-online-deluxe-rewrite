#!/usr/bin/env python3
"""Host tests for tools/hook_wait_check.py on hand-made graphs and a small source tree."""
import contextlib
import io
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import hook_wait_check as h  # noqa: E402


def nodes(edges, names):
    out = {addr: {"name": name, "calls": set()} for addr, name in names.items()}
    for a, b in edges:
        out[a]["calls"].add(b)
    return out


def run_check(*args):
    with contextlib.redirect_stdout(io.StringIO()) as out:
        result = h.check(*args)
    return result, out.getvalue()


class HookWaitCheck(unittest.TestCase):
    def test_a_lock_behind_a_helper_is_found_with_its_chain(self):
        g = nodes([(1, 2), (2, 3)], {1: "on_fighter_main", 2: "helper", 3: "pthread_mutex_lock@plt"})
        (hits, reached), text = run_check("wait", g, {1: "on_fighter_main"}, h.WAITS, lambda r, n: False, 4)
        self.assertEqual((hits, reached), (1, 3))
        self.assertIn("FORBIDDEN wait 0x3 pthread_mutex_lock@plt", text)
        self.assertIn("    helper", text)

    def test_the_panic_machinery_is_not_followed(self):
        g = nodes([(1, 2), (2, 3)], {1: "queue_present", 2: "core::panicking::panic_fmt", 3: "pthread_cond_wait@plt"})
        (hits, _), _ = run_check("wait", g, {1: "queue_present"}, h.WAITS, lambda r, n: False, 4)
        self.assertEqual(hits, 0)

    def test_an_allowed_wait_is_allowed_from_its_root_only(self):
        sleep = "_ZN2nn2os11SleepThreadENS_8TimeSpanE@plt"
        g = nodes([(1, 3), (2, 3)], {1: "dispatcher_idle_pass", 2: "queue_present", 3: sleep})
        allowed = lambda r, n: (r, n) in h.ALLOWED_WAITS  # noqa: E731
        (hits, _), _ = run_check("wait", g, {1: "dispatcher_idle_pass"}, h.WAITS, allowed, 4)
        self.assertEqual(hits, 0)
        (hits, _), _ = run_check("wait", g, {2: "queue_present"}, h.WAITS, allowed, 4)
        self.assertEqual(hits, 1)

    def test_allocation_and_formatting_are_caught_and_try_lock_is_not_a_wait(self):
        g = nodes([(1, 2), (1, 3), (1, 4)], {1: "window_acquire", 2: "malloc@plt", 3: "alloc::fmt::format::format_inner",
                                              4: "pthread_mutex_trylock@plt"})
        (hits, _), _ = run_check("alloc", g, {1: "window_acquire"}, h.ALLOC, lambda r, n: False, 0)
        self.assertEqual(hits, 2)
        (hits, _), _ = run_check("wait", g, {1: "window_acquire"}, h.WAITS, lambda r, n: False, 0)
        self.assertEqual(hits, 0)

    def test_a_plt_stub_resolves_to_its_import(self):
        insns = {0x100: (0x100, "adrp", "x16, 0x122000"), 0x104: (0x104, "ldr", "x17, [x16, #0x7a0]")}
        self.assertEqual(h.plt_name(0x100, insns, {0x1227a0: "free"}), "free@plt")
        self.assertIsNone(h.plt_name(0x104, insns, {0x1227a0: "free"}))

    def test_roots_come_from_hooks_and_the_foreign_table_but_not_from_tests(self):
        with tempfile.TemporaryDirectory() as repo:
            os.makedirs(os.path.join(repo, "src/runtime"))
            os.makedirs(os.path.join(repo, "tests"))
            with open(os.path.join(repo, "src/runtime/nvn.rs"), "w", encoding="utf-8") as f:
                f.write("#[skyline::hook(offset = 1)]\nunsafe fn real_hook() {}\n"
                        "#[cfg(test)]\nmod tests {\n    #[skyline::hook(offset = 2)]\n    fn test_hook() {}\n}\n")
            with open(os.path.join(repo, "tests/hook_locks.rs"), "w", encoding="utf-8") as f:
                f.write('const FOREIGN: &[(&str, &str, Class, &str)] = &[\n'
                        '    ("runtime/nvn.rs", "queue_present", Class::Hook, "x"),\n'
                        '    ("runtime/plugin_export.rs", "api_v1", Class::Export, "x"),\n'
                        '    ("runtime/logging.rs", "thread_main", Class::NotHook, "x"),\n];\n')
            labels = [label for _, label in h.roots_from_source(repo)]
            self.assertEqual(sorted(labels), ["api_v1", "queue_present", "real_hook"])
            patterns = dict((label, p) for p, label in h.roots_from_source(repo))
            self.assertTrue(patterns["queue_present"].match("ssbu_online_deluxe_rewrite::runtime::nvn::queue_present"))
            self.assertTrue(patterns["api_v1"].match("api_v1"))


if __name__ == "__main__":
    unittest.main()
