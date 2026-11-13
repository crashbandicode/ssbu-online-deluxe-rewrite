#!/usr/bin/env python3
"""On the built ELF: no game-thread path waits, and no per-frame hook path allocates or prints.

Ported from TrainingRooms' `tools/game_thread_wait_check.py` (rc.8, worker A's finding A3) and widened for Deluxe
(ssbu-mod-playbook WORKFLOW, "Hooks never wait on a plugin thread"). Every function a hook root reaches by direct
calls (`bl`, and a tail `b` to a function start; machine-outlined bodies are functions too; imports resolve through
their PLT stub to the symbol's name) is checked against two lists:

  waits   (every root): a blocking std lock (`Mutex::lock`, the queue `RwLock`'s contended paths, `Condvar` waits,
          the `Once` waiter, the thread parker), `thread::sleep`, std's channels and their `Backoff` spin, crossbeam,
          and the imports that block: `pthread_mutex_lock`, `pthread_cond_*wait`, `pthread_rwlock_*lock`, `nanosleep`,
          `usleep`, and Horizon's `nn::os` waits (`LockMutex`, `Wait*`, `TimedWait*`, `AcquireSemaphore`,
          `SleepThread`, `svcWaitSynchronization`, `svcArbitrateLock`, `svcSleepThread`).
  alloc   (per-frame roots: every root except `ONE_SHOT`): the allocator (`__rust_alloc*`, `__rust_realloc`,
          `__rust_dealloc`, `malloc`, `calloc`, `realloc`, `free`, `memalign`, `aligned_alloc`) and printing
          (`std::io::stdio::_print`/`_eprint`, `alloc::fmt::format`). The libc allocator takes its own lock, so an
          allocation is a wait too; the per-frame paths format into fixed buffers and post records lock-free.

Roots: every `#[skyline::hook]` function under src/ (exported under its bare name), and every `Hook`/`Export` row of
`FOREIGN` in tests/hook_locks.rs (the C-ABI entries the game or another plugin's hook calls), found by path.

    python3 -I tools/hook_wait_check.py <libssbu_online_deluxe_rewrite.so> [--repo <checkout>] [--paths N]

Exit 0: nothing forbidden reached; 1: each reached function with one call path from its root; 2: the ELF or the
roots could not be read. Indirect calls (`blr`: vtables, `dyn Fn`, function pointers) are not followed;
tests/hook_locks.rs covers those edges in the source.
"""
import os
import re
import subprocess
import sys

OBJDUMP = "/usr/bin/llvm-objdump-21"
READELF = "/usr/bin/llvm-readelf-21"
CRATE = "ssbu_online_deluxe_rewrite"

WAITS = re.compile(
    r"^<?(std::sync::mpmc::|std::sync::mpsc::)"
    r"|Backoff>::spin_(light|heavy)\b"
    r"|crossbeam"
    r"|^<std::sys::pal::switch::sync::mutex::Mutex>::lock\b"
    r"|^<std::sys::sync::rwlock::queue::RwLock>::lock_contended\b"
    r"|::sync::condvar::\w+>::wait"
    r"|^std::sys::sync::once::queue::wait\b"
    r"|::thread_parking::\w+::Parker>::park"
    r"|^std::thread::functions::sleep"
    r"|^(pthread_mutex_lock|pthread_mutex_timedlock|pthread_cond_wait|pthread_cond_timedwait"
    r"|pthread_rwlock_rdlock|pthread_rwlock_wrlock|pthread_rwlock_timedrdlock|pthread_rwlock_timedwrlock"
    r"|pthread_join|nanosleep|usleep|sleep|sched_yield_wait)@plt$"
    r"|nn2os\d+(LockMutex|Wait\w*|TimedWait\w*|AcquireSemaphore|TimedAcquireSemaphore|SleepThread|JoinThread)\w*@plt$"
    r"|^svc(WaitSynchronization|ArbitrateLock|SleepThread|WaitProcessWideKeyAtomic)\w*@plt$"
)
ALLOC = re.compile(
    r"^__rustc::__rust_(alloc|alloc_zeroed|realloc|dealloc)$"
    r"|^(malloc|calloc|realloc|free|memalign|aligned_alloc|posix_memalign)@plt$"
    r"|^std::io::stdio::_e?print$"
    r"|^alloc::fmt::format::format_inner$"
)

# Not followed: the panic machinery. A panic in a hook ends the game (panic = "abort"), so what it reaches (the
# message's formatting, the hook's lock and parker) never runs on a live hook path.
STOP = re.compile(r"^core::panicking::|^core::option::(expect|unwrap)_failed\b|^core::result::unwrap_failed\b"
                  r"|^alloc::alloc::handle_alloc_error\b|^core::slice::index::\w+_fail\b|^core::str::slice_error_fail\b"
                  r"|^core::cell::panic_already_\w+\b")

# A wait a hook makes on purpose, by root and function: (root, wait) -> why.
ALLOWED_WAITS = {
    ("dispatcher_idle_pass", "_ZN2nn2os11SleepThreadENS_8TimeSpanE@plt"):
        "dispatcher backoff: a timed sleep (at most the policy's cap) in the game's idle pass instead of its spin; "
        "it waits on no other thread",
}

# Roots that run once per install, per scene or per connection event, not every frame: (name, why). The wait check
# still covers them; only the allocation check skips them.
ONE_SHOT = {
    "get_proc": "NVN proc lookup: once per proc name at device setup",
    "window_initialize": "NVN window setup",
    "window_finalize": "NVN window teardown",
    "deluxe_rewrite_nvn_bootstrap_hook": "nvnBootstrapLoader: once per proc name",
    "bootstrap_original": "the bootstrap's original",
    "on_context_setup": "ImGui context setup: once",
    "on_backend_ready": "ImGui backend ready: once",
    "accept_external_context": "the standalone's context: once",
    "backend_logger": "ImGui backend log lines: setup and errors",
    "on_load": "module load",
    "on_unload": "module unload",
    "ssbu_online_deluxe_plugin_api_v1": "API table lookup: once per plugin",
    "register": "plugin registration: once per plugin",
    "host_build": "build string: once per caller",
    "deluxe_rewrite_main_menu": "scene entry",
    "deluxe_rewrite_quickplay": "scene entry",
    "deluxe_rewrite_background_matchmaking": "scene entry",
    "deluxe_rewrite_arena": "scene entry",
    "deluxe_rewrite_local_wireless": "scene entry",
    "deluxe_rewrite_stage_setup": "stage load",
    "deluxe_rewrite_arena_room_init": "arena room entry",
    "deluxe_rewrite_observe_station_left": "a peer leaves",
    "deluxe_rewrite_observe_session_end": "session end",
}

FUNC_RE = re.compile(r"^([0-9a-f]+) <(.+)>:$")
INSN_RE = re.compile(r"^\s*([0-9a-f]+):\s+(\S+)\s*(.*)$")
TARGET_RE = re.compile(r"^(?:0x)?([0-9a-f]+)\b")
HOOK_RE = re.compile(r"#\[skyline::hook\b")


def disassemble(elf):
    out = subprocess.run([OBJDUMP, "-d", "--no-show-raw-insn", "--demangle", elf], capture_output=True, text=True,
                         check=True).stdout
    funcs, insns, cur = {}, {}, None
    for line in out.splitlines():
        m = FUNC_RE.match(line)
        if m:
            cur = int(m.group(1), 16)
            funcs[cur] = {"name": re.sub(r" \(\.llvm\.\d+\)$", "", m.group(2)), "body": []}
            continue
        m = INSN_RE.match(line)
        if m and cur is not None:
            insn = (int(m.group(1), 16), m.group(2), m.group(3))
            funcs[cur]["body"].append(insn)
            insns[insn[0]] = insn
    return funcs, insns


def jump_slots(elf):
    """GOT slot address -> imported symbol, from the PLT relocations."""
    out = subprocess.run([READELF, "-r", "-W", elf], capture_output=True, text=True, check=True).stdout
    slots = {}
    for line in out.splitlines():
        parts = line.split()
        if len(parts) >= 5 and parts[2] == "R_AARCH64_JUMP_SLOT":
            slots[int(parts[0], 16)] = parts[4]
    return slots


def plt_name(addr, insns, slots):
    """The import a PLT stub at `addr` jumps through: `adrp x16, page; ldr x17, [x16, #off]`."""
    a, b = insns.get(addr), insns.get(addr + 4)
    if not a or not b or a[1] != "adrp" or b[1] != "ldr" or not a[2].replace(" ", "").startswith("x16,"):
        return None
    page = re.search(r"0x([0-9a-f]+)", a[2])
    off = re.search(r"\[x16(?:,\s*#(0x[0-9a-f]+|\d+))?\]", b[2])
    if not page or not off:
        return None
    slot = int(page.group(1), 16) + (int(off.group(1), 0) if off.group(1) else 0)
    name = slots.get(slot)
    return name + "@plt" if name else None


def graph(elf):
    funcs, insns = disassemble(elf)
    slots = jump_slots(elf)
    nodes = {addr: {"name": f["name"], "calls": set()} for addr, f in funcs.items()}
    for addr, f in funcs.items():
        for _, op, args in f["body"]:
            if op not in ("bl", "b"):
                continue
            m = TARGET_RE.match(args.strip())
            if not m:
                continue
            t = int(m.group(1), 16)
            if t == addr:
                continue
            if t in funcs:
                nodes[addr]["calls"].add(t)
                continue
            name = plt_name(t, insns, slots)
            if name:
                nodes.setdefault(t, {"name": name, "calls": set()})
                nodes[addr]["calls"].add(t)
    return nodes


def strip_cfg_test(text):
    """Drops `#[cfg(test)]` modules and items so test-only functions are never roots."""
    out, i = [], 0
    while True:
        j = text.find("#[cfg(test)]", i)
        if j < 0:
            out.append(text[i:])
            return "".join(out)
        out.append(text[i:j])
        k = text.find("{", j)
        semi = text.find(";", j)
        if k < 0 or (0 <= semi < k):
            i = semi + 1
            continue
        depth, p = 0, k
        while p < len(text):
            if text[p] == "{":
                depth += 1
            elif text[p] == "}":
                depth -= 1
                if depth == 0:
                    break
            p += 1
        i = p + 1


def roots_from_source(repo):
    """[(symbol regex, label)] of every hook root."""
    out = []
    for d, _, files in os.walk(os.path.join(repo, "src")):
        for name in sorted(files):
            if not name.endswith(".rs"):
                continue
            lines = strip_cfg_test(open(os.path.join(d, name), encoding="utf-8").read()).split("\n")
            for i, line in enumerate(lines):
                if HOOK_RE.search(line):
                    for nxt in lines[i + 1:i + 8]:
                        m = re.search(r"\bfn\s+([A-Za-z_]\w*)", nxt)
                        if m:
                            out.append((re.compile("^" + re.escape(m.group(1)) + "$"), m.group(1)))
                            break
    table = open(os.path.join(repo, "tests/hook_locks.rs"), encoding="utf-8").read()
    start = table.index("const FOREIGN:")
    block = table[start:table.index("];", start)]
    for file, fn, cls in re.findall(r'\("([^"]+)",\s*"([^"]+)",\s*Class::(\w+)', block):
        if cls not in ("Hook", "Export"):
            continue
        module = "::".join(file[:-3].split("/"))
        if module.endswith("::mod"):
            module = module[:-5]
        # By path, or under its bare name when it is `#[no_mangle]` (the exports).
        out.append((re.compile("^(" + re.escape(f"{CRATE}::{module}::{fn}") + "|" + re.escape(fn) + ")$"), fn))
    return out



def reach(nodes, root):
    """Breadth first from `root`, not entering the panic machinery: {function: parent}."""
    parent = {root: None}
    queue = [root]
    while queue:
        a = queue.pop(0)
        for c in sorted(nodes[a]["calls"]):
            if c not in parent and not STOP.search(nodes[c]["name"]):
                parent[c] = a
                queue.append(c)
    return parent


def chain(nodes, parent, a):
    out = []
    while a is not None:
        out.append(nodes[a]["name"])
        a = parent[a]
    return list(reversed(out))


def check(kind, nodes, roots, pattern, allowed, paths):
    """Every forbidden function the roots reach, with one call chain each; `allowed(root label, name)` exempts."""
    hits, seen = {}, set()
    for root, label in sorted(roots.items()):
        parent = reach(nodes, root)
        seen.update(parent)
        for a in parent:
            name = nodes[a]["name"]
            if pattern.search(name) and not allowed(label, name) and a not in hits:
                hits[a] = chain(nodes, parent, a)
    for i, (a, path) in enumerate(sorted(hits.items(), key=lambda kv: nodes[kv[0]]["name"])):
        print(f"FORBIDDEN {kind} 0x{a:x} {nodes[a]['name'][:160]}")
        if i < paths:
            for name in path:
                print(f"    {name[:160]}")
    return len(hits), len(seen)


def main():
    args = sys.argv[1:]
    if not args:
        print(__doc__)
        sys.exit(2)
    elf = args[0]
    repo = args[args.index("--repo") + 1] if "--repo" in args else "."
    paths = int(args[args.index("--paths") + 1]) if "--paths" in args else 8
    try:
        nodes = graph(elf)
        wanted = roots_from_source(repo)
    except (OSError, subprocess.CalledProcessError, ValueError) as error:
        print(f"cannot read {elf} or {repo}: {error}")
        sys.exit(2)
    by_name = {}
    for addr, node in nodes.items():
        by_name.setdefault(node["name"], addr)
    roots, frame_roots, missing = {}, {}, []
    for pattern, label in wanted:
        found = [addr for name, addr in by_name.items() if pattern.match(name)]
        if not found:
            missing.append(label)
        for addr in found:
            roots[addr] = label
            if label not in ONE_SHOT:
                frame_roots[addr] = label
    if len(roots) < 40:
        print(f"only {len(roots)} roots found in {elf}; missing {missing}")
        sys.exit(2)
    waits, reached = check("wait", nodes, roots, WAITS, lambda label, name: (label, name) in ALLOWED_WAITS, paths)
    allocs, frame_reached = check("alloc", nodes, frame_roots, ALLOC, lambda label, name: False, paths)
    print(f"{len(roots)} hook roots, {reached} reachable functions, {waits} forbidden waits "
          f"({len(ALLOWED_WAITS)} allowed by name); {len(frame_roots)} per-frame roots, {frame_reached} reachable, "
          f"{allocs} allocation or print")
    if missing:
        print(f"roots not in this ELF (cfg'd out or inlined): {', '.join(sorted(set(missing)))}")
    sys.exit(1 if waits or allocs else 0)


if __name__ == "__main__":
    main()

