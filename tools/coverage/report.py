#!/usr/bin/env python3
"""Coverage analysis for the coverage build (docs/DEBUGGING.md "Coverage build").

Inputs are the coverage build's ELF (the `.elf` in its package) and raw
profiles the plugin wrote to `coverage/` on the card, grouped into named
scenarios. For every scenario and for all of them together the script runs
`llvm-profdata merge` and `llvm-cov`, then derives from the exports:

* per-file line and function coverage (`summary.md`);
* functions never executed in any scenario, with their line span, physical
  lines and whether anything outside test code references them
  (`never-functions.md`, `functions.csv`);
* lines never executed inside functions that did run (`uncovered-lines.md`);
* ride-along classes: functions whose execution-count vector across the
  scenarios is identical, so they only ever run together (`ride-along.md`).

Nothing here decides what to remove. The numbers are the evidence a reviewer
reads next to the source; `Estimated reduction` is the physical-line union of
functions that never ran and are referenced from nothing but other such
functions or test code.

Usage:
  report.py --elf <libssbu_online_deluxe_rewrite.elf> --out <dir>
            [--llvm-bin <dir>] [--source-root <repo>] [--demangler c++filt]
            <scenario>=<profraw file or directory> ...
"""
import argparse
import csv
import json
import re
import shutil
import subprocess
from collections import defaultdict
from pathlib import Path

DEFAULT_LLVM_BIN = Path.home() / (
    ".rustup/toolchains/nightly-2026-02-14-x86_64-unknown-linux-gnu/lib/rustlib/"
    "x86_64-unknown-linux-gnu/bin")


def run(command, **kwargs):
    return subprocess.run(command, check=True, text=True, capture_output=True, **kwargs)


def profraw_files(spec):
    path = Path(spec)
    if path.is_dir():
        files = sorted(p for p in path.glob("*.profraw") if p.name != "writing.profraw")
    else:
        files = [path]
    if not files:
        raise SystemExit(f"no .profraw files under {spec}")
    return files


def demangle(names, demangler):
    if not names or not shutil.which(demangler):
        return {name: name for name in names}
    result = subprocess.run([demangler], input="\n".join(names) + "\n", text=True,
                            capture_output=True, check=True)
    out = {}
    for mangled, line in zip(names, result.stdout.splitlines()):
        # c++filt prints `crate[hash]::path`; drop the crate hash.
        out[mangled] = re.sub(r"\[[0-9a-f]{16}\]", "", line.strip()) or mangled
    return out


def relative(filename, source_root):
    try:
        return str(Path(filename).resolve().relative_to(source_root))
    except ValueError:
        return None


def export_json(llvm_cov, elf, profdata):
    text = run([str(llvm_cov), "export", str(elf), f"-instr-profile={profdata}",
                "-format=text", "-skip-expansions"]).stdout
    return json.loads(text)["data"][0]


def export_lcov_lines(llvm_cov, elf, profdata, source_root):
    """{relative file: {line: count}} from the lcov export."""
    text = run([str(llvm_cov), "export", str(elf), f"-instr-profile={profdata}",
                "-format=lcov"]).stdout
    lines = defaultdict(dict)
    current = None
    for row in text.splitlines():
        if row.startswith("SF:"):
            current = relative(row[3:], source_root)
        elif row.startswith("DA:") and current:
            number, count = row[3:].split(",")[:2]
            lines[current][int(number)] = int(count)
    return lines


def union_lines(spans):
    """Physical lines covered by the union of (start, end) spans."""
    total = 0
    for start, end in sorted(spans):
        if end < start:
            continue
        if total and start <= last_end:  # noqa: F821 - set on first iteration
            if end > last_end:
                total += end - last_end
                last_end = end
        else:
            total += end - start + 1
            last_end = end
    return total


def test_boundary(path):
    """First line of a file's test module (`mod tests` and its `#[cfg(test)]`).

    Inline `#[cfg(test)]` helpers earlier in the file are not the boundary;
    they are small, and treating them as it would call live code test-only.
    """
    try:
        text = path.read_text().splitlines()
    except OSError:
        return None
    for index, line in enumerate(text, 1):
        if re.match(r"\s*(pub(\(crate\))?\s+)?mod tests\b", line):
            if index > 1 and "#[cfg(test)]" in text[index - 2]:
                return index - 1
            return index
    return len(text) + 1


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--elf", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--llvm-bin", type=Path, default=DEFAULT_LLVM_BIN)
    parser.add_argument("--source-root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--demangler", default="c++filt")
    parser.add_argument("scenarios", nargs="+", help="name=profraw-file-or-directory")
    args = parser.parse_args()

    llvm_profdata = args.llvm_bin / "llvm-profdata"
    llvm_cov = args.llvm_bin / "llvm-cov"
    source_root = args.source_root.resolve()
    args.out.mkdir(parents=True, exist_ok=True)

    scenarios = []
    for spec in args.scenarios:
        name, _, location = spec.partition("=")
        if not location:
            raise SystemExit(f"scenario must be name=path: {spec}")
        scenarios.append((name, profraw_files(location)))
    names = [name for name, _ in scenarios]

    # Per-scenario and merged profiles.
    profdata = {}
    all_files = []
    for name, files in scenarios:
        target = args.out / f"{name}.profdata"
        run([str(llvm_profdata), "merge", "-sparse", "-o", str(target), *map(str, files)])
        profdata[name] = target
        all_files += files
    merged = args.out / "all.profdata"
    run([str(llvm_profdata), "merge", "-sparse", "-o", str(merged), *map(str, all_files)])
    (args.out / "all-report.txt").write_text(
        run([str(llvm_cov), "report", str(elf := args.elf), f"-instr-profile={merged}"]).stdout)
    run([str(llvm_cov), "show", str(elf), f"-instr-profile={merged}", "-format=html",
         f"-output-dir={args.out / 'html'}", "-show-branches=count",
         f"-Xdemangler={args.demangler}"] if shutil.which(args.demangler) else
        [str(llvm_cov), "show", str(elf), f"-instr-profile={merged}", "-format=html",
         f"-output-dir={args.out / 'html'}", "-show-branches=count"])

    # Function table: entry count per scenario, span, file.
    functions = {}
    per_scenario_lines = {}
    for name in names:
        data = export_json(llvm_cov, elf, profdata[name])
        per_scenario_lines[name] = export_lcov_lines(llvm_cov, elf, profdata[name], source_root)
        for function in data["functions"]:
            rel = relative(function["filenames"][0], source_root) if function["filenames"] else None
            if rel is None or not rel.startswith("src/"):
                continue
            regions = [r for r in function["regions"] if r[5] == 0 and r[7] == 0]
            if not regions:
                continue
            start = min(r[0] for r in regions)
            end = max(r[2] for r in regions)
            entry = functions.setdefault(function["name"], {
                "file": rel, "start": start, "end": end, "counts": {},
                "zero_regions": {}, "regions": len(regions)})
            entry["counts"][name] = function["count"]
            entry["zero_regions"][name] = sum(1 for r in regions if r[4] == 0)
    demangled = demangle(sorted(functions), args.demangler)

    never, executed = [], []
    for mangled, entry in functions.items():
        vector = tuple(entry["counts"].get(name, 0) for name in names)
        entry["vector"] = vector
        entry["name"] = demangled[mangled]
        entry["lines"] = entry["end"] - entry["start"] + 1
        (executed if any(vector) else never).append(entry)

    # References to never-executed functions from non-test code.
    sources = {path: path.read_text().splitlines() for path in (source_root / "src").rglob("*.rs")}
    boundaries = {path: test_boundary(path) for path in sources}
    never_spans = defaultdict(list)
    for entry in never:
        never_spans[entry["file"]].append((entry["start"], entry["end"]))

    def in_never_span(rel, line):
        return any(start <= line <= end for start, end in never_spans.get(rel, []))

    for entry in never:
        short = entry["name"].rsplit("::", 1)[-1].split("<", 1)[0]
        if short in {"{{closure}}", "new", "default", "fmt", "drop", "clone", "eq"}:
            entry["refs"] = None
            continue
        pattern = re.compile(rf"\b{re.escape(short)}\b")
        live, test, dead = 0, 0, 0
        for path, lines in sources.items():
            rel = str(path.relative_to(source_root))
            for number, text in enumerate(lines, 1):
                if not pattern.search(text):
                    continue
                if rel == entry["file"] and entry["start"] <= number <= entry["end"]:
                    continue
                if number >= boundaries[path] or rel.startswith("tests/"):
                    test += 1
                elif in_never_span(rel, number):
                    dead += 1
                else:
                    live += 1
        entry["refs"] = (live, dead, test)

    # Lines never executed inside functions that ran.
    file_lines = defaultdict(lambda: defaultdict(list))
    for name in names:
        for rel, counts in per_scenario_lines[name].items():
            for line, count in counts.items():
                file_lines[rel][line].append(count)
    uncovered = defaultdict(list)
    instrumented_lines = 0
    covered_lines = 0
    for rel, lines in file_lines.items():
        if not rel.startswith("src/"):
            continue
        for line, counts in lines.items():
            instrumented_lines += 1
            if any(counts):
                covered_lines += 1
            elif not in_never_span(rel, line):
                uncovered[rel].append(line)

    # Ride-along classes among executed functions.
    classes = defaultdict(list)
    for entry in executed:
        classes[entry["vector"]].append(entry)
    init_once = classes.get(tuple(1 for _ in names), [])
    interesting = sorted(
        ((vector, members) for vector, members in classes.items()
         if len(members) >= 2 and all(vector) and max(vector) > 1),
        key=lambda item: -len(item[1]))

    # Candidate reduction: never functions with no live references. Spans are
    # unioned per file so nested closures are not counted twice.
    def lines_per_file(entries):
        spans = defaultdict(list)
        for entry in entries:
            spans[entry["file"]].append((entry["start"], entry["end"]))
        return sum(union_lines(group) for group in spans.values())

    # Constructors and trait impls (`refs` None) are skipped: `const fn new`
    # bodies run at compile time for statics and never count at runtime.
    candidates = [e for e in never if e["refs"] is not None and e["refs"][0] == 0]
    constructors = [e for e in never if e["refs"] is None]
    closures = [e for e in never if "{closure" in e["name"]]
    candidate_lines = lines_per_file(candidates)
    never_lines = lines_per_file(never)
    uncovered_count = sum(len(v) for v in uncovered.values())

    with (args.out / "functions.csv").open("w", newline="") as handle:
        writer = csv.writer(handle)
        writer.writerow(["file", "start", "end", "lines", *names, "live_refs", "dead_refs", "test_refs", "function"])
        for entry in sorted(functions.values(), key=lambda e: (e["file"], e["start"])):
            refs = entry.get("refs") or ("", "", "")
            writer.writerow([entry["file"], entry["start"], entry["end"], entry["lines"],
                             *entry["vector"], *refs, entry["name"]])

    by_file = defaultdict(lambda: {"never": 0, "never_lines": [], "functions": 0})
    for entry in functions.values():
        by_file[entry["file"]]["functions"] += 1
        if not any(entry["vector"]):
            by_file[entry["file"]]["never"] += 1
            by_file[entry["file"]]["never_lines"].append((entry["start"], entry["end"]))

    summary = [
        "# Coverage summary", "",
        f"ELF: `{args.elf}`", f"Scenarios: {', '.join(names)}", "",
        "| Metric | Value |", "| --- | --- |",
        f"| Instrumented functions (src/) | {len(functions)} |",
        f"| Executed in at least one scenario | {len(executed)} |",
        f"| Never executed | {len(never)} (of which {len(closures)} closures, {len(constructors)} constructors/trait impls) |",
        f"| Physical lines of never-executed functions | {never_lines} |",
        f"| Never-executed functions with no live reference | {len(candidates)} |",
        f"| Estimated reduction (lines, those candidates) | {candidate_lines} |",
        f"| Instrumented lines (lcov) | {instrumented_lines} |",
        f"| Covered lines | {covered_lines} |",
        f"| Uncovered lines inside executed functions | {uncovered_count} |",
        f"| Init-once functions (count 1 in every scenario) | {len(init_once)} |",
        "", "## Per file", "",
        "| File | Functions | Never | Never lines | Uncovered lines in executed fns |",
        "| --- | --- | --- | --- | --- |",
    ]
    for rel in sorted(by_file, key=lambda r: -union_lines(by_file[r]["never_lines"])):
        info = by_file[rel]
        summary.append(f"| {rel} | {info['functions']} | {info['never']} | "
                       f"{union_lines(info['never_lines'])} | {len(uncovered.get(rel, []))} |")
    (args.out / "summary.md").write_text("\n".join(summary) + "\n")

    rows = ["# Functions never executed", "",
            "live = references from non-test code outside never-executed functions; "
            "dead = references from other never-executed functions; test = references from test code.", "",
            "| File | Lines | LOC | live | dead | test | Function |", "| --- | --- | --- | --- | --- | --- | --- |"]
    for entry in sorted(never, key=lambda e: (e["file"], e["start"])):
        refs = entry["refs"] or ("-", "-", "-")
        rows.append(f"| {entry['file']} | {entry['start']}-{entry['end']} | {entry['lines']} | "
                    f"{refs[0]} | {refs[1]} | {refs[2]} | `{entry['name']}` |")
    (args.out / "never-functions.md").write_text("\n".join(rows) + "\n")

    rows = ["# Lines never executed inside functions that ran", "",
            "Line numbers from the lcov export, grouped into runs. Each run is a branch arm, "
            "early return or match arm that no scenario reached.", ""]
    for rel in sorted(uncovered):
        lines = sorted(uncovered[rel])
        runs, start, previous = [], lines[0], lines[0]
        for line in lines[1:]:
            if line != previous + 1:
                runs.append((start, previous))
                start = line
            previous = line
        runs.append((start, previous))
        rows.append(f"## {rel} ({len(lines)} lines)")
        rows.append(", ".join(f"{a}" if a == b else f"{a}-{b}" for a, b in runs))
        rows.append("")
    (args.out / "uncovered-lines.md").write_text("\n".join(rows) + "\n")

    rows = ["# Ride-along classes", "",
            "Functions with an identical execution-count vector across every scenario only ever ran "
            "together; each class lists them. Classes of all-ones (init-once) are counted, not listed.", "",
            f"Init-once functions: {len(init_once)}", ""]
    for vector, members in interesting[:60]:
        rows.append(f"## counts {list(vector)} ({len(members)} functions)")
        for entry in sorted(members, key=lambda e: (e["file"], e["start"])):
            rows.append(f"- {entry['file']}:{entry['start']} `{entry['name']}`")
        rows.append("")
    (args.out / "ride-along.md").write_text("\n".join(rows) + "\n")
    print((args.out / "summary.md").read_text())


if __name__ == "__main__":
    main()
