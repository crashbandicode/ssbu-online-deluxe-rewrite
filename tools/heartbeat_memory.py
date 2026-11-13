"""Process memory per phase from Deluxe's hang-trace.log heartbeats (mem_used, svcGetInfo), in MiB (2**20).

Phases, from each file's own scene lines:
- menu: offline/inactive heartbeats (css=0) in the `--menu-window` seconds before the first scene change;
- css: heartbeats with css=1 before the first */training heartbeat;
- load: heartbeats after the last CSS heartbeat (the game loads the stage and fighters before the packed scene says
  training) and */training heartbeats less than `--settle` seconds after the scene changed to */training;
- training: */training heartbeats from `--settle` seconds on (one Training entry: up to the next scene change).
`peak` is the largest mem_used of the file.

    python3 -I tools/heartbeat_memory.py a=<hang-trace.log> b=<hang-trace.log> ... [--ref a] [--json]

With several labelled files, each phase's median is also given against the reference label (default: the first).

`--scenes` lists instead each run of consecutive heartbeats with one scene and css flag (for an online match: the
Quickplay CSS, the match, the CSS after it), with the start and end times and min/median/max.
"""
import argparse
import json
import re
import statistics
import sys

BEAT = re.compile(r"^\[Heartbeat\] t=([\d.]+) .*?\bscene=(\S+) .*?\bcss=(\d) .*?\bmem_used=(\d+)")
CHANGE = re.compile(r"^\[HangTrace\] scene (\S+) -> (\S+) t=([\d.]+)")
PHASES = ("menu", "css", "load", "training")
MIB = 2 ** 20


def parse(text):
    """Heartbeats (t, scene, css, used bytes) and scene changes (t, from, to), in file order."""
    beats, changes = [], []
    for line in text.splitlines():
        match = BEAT.search(line)
        if match:
            beats.append((float(match.group(1)), match.group(2), int(match.group(3)), int(match.group(4))))
            continue
        match = CHANGE.search(line)
        if match:
            changes.append((float(match.group(3)), match.group(1), match.group(2)))
    return beats, changes


def summary(values):
    if not values:
        return None
    mib = [v / MIB for v in values]
    return {"n": len(mib), "min": round(min(mib), 1), "median": round(statistics.median(mib), 1),
            "max": round(max(mib), 1)}


def phases(text, menu_window=30.0, settle=15.0):
    beats, changes = parse(text)
    if not beats:
        raise ValueError("no heartbeat lines")
    first_change = changes[0][0] if changes else float("inf")
    training_at = next((t for t, _, to in changes if to.endswith("/training")), None)
    after = [t for t, _, _ in changes if training_at is not None and t > training_at]
    training_end = after[0] if after else float("inf")
    css_end = max((t for t, _, css, _ in beats if css == 1 and (training_at is None or t < training_at)),
                  default=None)
    groups = {name: [] for name in PHASES}
    for t, scene, css, used in beats:
        if t < first_change and scene == "offline/inactive" and css == 0 and t >= first_change - menu_window:
            groups["menu"].append(used)
        elif css == 1 and (training_at is None or t < training_at):
            groups["css"].append(used)
        elif scene.endswith("/training") and training_at is not None and training_at <= t < training_end:
            groups["load" if t - training_at < settle else "training"].append(used)
        elif training_at is not None and css_end is not None and css_end < t < training_at:
            groups["load"].append(used)
    out = {name: summary(groups[name]) for name in PHASES}
    out["peak"] = round(max(used for _, _, _, used in beats) / MIB, 1)
    out["training_entered_at"] = training_at
    return out


def scenes(text):
    """Runs of consecutive heartbeats with the same scene and css flag, in file order."""
    beats, _ = parse(text)
    if not beats:
        raise ValueError("no heartbeat lines")
    runs = []
    for t, scene, css, used in beats:
        if runs and runs[-1]["scene"] == scene and runs[-1]["css"] == css:
            runs[-1]["t_end"] = t
            runs[-1]["used"].append(used)
        else:
            runs.append({"scene": scene, "css": css, "t_start": t, "t_end": t, "used": [used]})
    return [{**{k: v for k, v in run.items() if k != "used"}, **summary(run["used"])} for run in runs]


def scenes_table(label, runs):
    lines = [f"{label}:", "| scene | css | t | median | min-max | n |", "| --- | --- | --- | --- | --- | --- |"]
    for run in runs:
        lines.append(f"| {run['scene']} | {run['css']} | {run['t_start']:.1f}-{run['t_end']:.1f} | {run['median']:,.1f} "
                     f"| {run['min']:,.1f}-{run['max']:,.1f} | {run['n']} |")
    return "\n".join(lines)


def compare(results, ref):
    """Each phase's median minus the reference's, in MiB (None where either side has no heartbeat)."""
    deltas = {}
    base = results[ref]
    for label, result in results.items():
        if label == ref:
            continue
        deltas[label] = {}
        for name in (*PHASES, "peak"):
            mine, theirs = result.get(name), base.get(name)
            if isinstance(mine, dict) and isinstance(theirs, dict):
                deltas[label][name] = round(mine["median"] - theirs["median"], 1)
            elif isinstance(mine, float) and isinstance(theirs, float):
                deltas[label][name] = round(mine - theirs, 1)
            else:
                deltas[label][name] = None
    return deltas


def table(results, deltas, ref):
    lines = ["| run | menu | CSS | load | Training | peak |", "| --- | --- | --- | --- | --- | --- |"]
    for label, result in results.items():
        cells = []
        for name in PHASES:
            item = result[name]
            if item is None:
                cells.append("-")
                continue
            cell = f"{item['median']:,.1f} ({item['min']:,.1f}-{item['max']:,.1f}, n={item['n']})"
            delta = deltas.get(label, {}).get(name)
            if delta is not None:
                cell += f" {delta:+.1f}"
            cells.append(cell)
        peak = f"{result['peak']:,.1f}"
        if deltas.get(label, {}).get("peak") is not None:
            peak += f" {deltas[label]['peak']:+.1f}"
        lines.append(f"| {label}{' (ref)' if label == ref else ''} | " + " | ".join(cells) + f" | {peak} |")
    return "\n".join(lines)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("logs", nargs="+", help="label=path to a hang-trace.log")
    parser.add_argument("--ref", help="reference label (default: the first)")
    parser.add_argument("--menu-window", type=float, default=30.0)
    parser.add_argument("--settle", type=float, default=15.0)
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--scenes", action="store_true", help="per scene run instead of the Training phases")
    args = parser.parse_args(argv)
    texts = {}
    for item in args.logs:
        label, sep, path = item.partition("=")
        if not sep:
            raise SystemExit(f"{item}: expected label=path")
        with open(path, encoding="utf-8", errors="replace") as handle:
            texts[label] = handle.read()
    if args.scenes:
        runs = {label: scenes(text) for label, text in texts.items()}
        print(json.dumps(runs, indent=1) if args.json else "\n\n".join(scenes_table(k, v) for k, v in runs.items()))
        return 0
    results = {label: phases(text, args.menu_window, args.settle) for label, text in texts.items()}
    ref = args.ref or next(iter(results))
    deltas = compare(results, ref)
    if args.json:
        print(json.dumps({"ref": ref, "results": results, "deltas": deltas}, indent=1))
    else:
        print(table(results, deltas, ref))
    return 0


if __name__ == "__main__":
    sys.exit(main())
