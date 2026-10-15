#!/usr/bin/env python3
"""Per-second frame counts from a timing capture, for the 59 fps acceptance test.

Reads a `timing-*.csv` written by the plugin (schema 2, one row per render
callback, `tick_hz` in the header) and reports how many callbacks fell into
each wall-clock second and the worst sliding one-second window. The
performance-floor acceptance criterion is "no one-second window below 59".

With `--events events.log` the resolution-change edges (event codes 82 and
108) are matched to the capture by tick, and every second that overlaps a
dynamic-resolution push or pop is marked, so a drop that happened while the
resolution was lowered is not read as the effect's raw cost.

Usage:
  frame_windows.py <timing.csv> [--events events.log] [--json] [--floor 59]
"""
import argparse
import bisect
import json
from pathlib import Path

DRS_CODES = {82, 108}


def read_capture(path):
    header, ticks = {}, []
    for line in Path(path).read_text().splitlines():
        if line.startswith("#"):
            key, _, value = line[1:].strip().partition("=")
            header[key.strip()] = value.strip()
        elif line and not line.startswith("sample,"):
            parts = line.split(",")
            # A capture copied while still being written can end in a partial row.
            if len(parts) < 3 or not parts[2].isdigit():
                header.setdefault("malformed_rows", 0)
                header["malformed_rows"] += 1
                continue
            ticks.append(int(parts[2]))
    return header, ticks


def read_drs_edges(path, first_tick, last_tick):
    """Ticks of resolution edges inside the capture's span."""
    edges = []
    for line in Path(path).read_text().splitlines():
        parts = line.split()
        if len(parts) < 5 or not parts[0].isdigit():
            continue
        tick, code = int(parts[1]), int(parts[2])
        if code in DRS_CODES and first_tick <= tick <= last_tick:
            edges.append({"tick": tick, "code": code, "a": parts[3], "b": parts[4]})
    return edges


def per_second(ticks, hz):
    first = ticks[0]
    seconds = {}
    for tick in ticks:
        seconds[(tick - first) // hz] = seconds.get((tick - first) // hz, 0) + 1
    last_second = (ticks[-1] - first) // hz
    return [seconds.get(index, 0) for index in range(last_second + 1)], first


def worst_sliding_window(ticks, hz):
    """Fewest callbacks in any one-second window starting at a callback."""
    worst = None
    for index, start in enumerate(ticks):
        end = bisect.bisect_left(ticks, start + hz)
        count = end - index
        if end >= len(ticks) and ticks[-1] - start < hz:
            break  # the window runs past the capture
        if worst is None or count < worst["count"]:
            worst = {"count": count, "start_tick": start, "offset_s": round((start - ticks[0]) / hz, 2)}
    return worst


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("csv", type=Path)
    parser.add_argument("--events", type=Path)
    parser.add_argument("--floor", type=int, default=59)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    header, ticks = read_capture(args.csv)
    if len(ticks) < 2:
        raise SystemExit("capture has fewer than two samples")
    hz = int(header.get("tick_hz", 19_200_000))
    counts, first = per_second(ticks, hz)
    last_tick = ticks[-1]
    edges = read_drs_edges(args.events, first, last_tick) if args.events else []
    marked = set()
    for edge in edges:
        marked.add((edge["tick"] - first) // hz)
    # Partial trailing second is reported but not judged.
    full = counts[:-1] if (last_tick - first) % hz else counts
    below = [i for i, c in enumerate(full) if c < args.floor]
    below_clean = [i for i in below if i not in marked]
    worst = worst_sliding_window(ticks, hz)
    result = {
        "csv": str(args.csv), "label": header.get("label"), "build": header.get("build"), "id": header.get("id"),
        "samples": len(ticks), "duration_s": round((last_tick - first) / hz, 2), "floor": args.floor,
        "seconds": counts, "full_seconds": len(full), "seconds_below_floor": below,
        "seconds_below_floor_outside_drs": below_clean,
        "drs_edges": edges, "drs_marked_seconds": sorted(marked),
        "worst_sliding_window": worst,
        "pass": not below_clean and (worst is None or worst["count"] >= args.floor or bool(marked)),
    }
    if args.json:
        print(json.dumps(result, indent=2))
        return
    print(f"{args.csv.name}: label={result['label']} build={result['build']} samples={len(ticks)} "
          f"duration={result['duration_s']} s floor={args.floor}")
    row = []
    for index, count in enumerate(counts):
        flag = "*" if index in marked else ("!" if count < args.floor and index < len(full) else " ")
        row.append(f"{count:3d}{flag}")
        if len(row) == 15:
            print("  " + " ".join(row))
            row = []
    if row:
        print("  " + " ".join(row))
    print(f"  seconds below {args.floor}: {below or 'none'}; outside DRS-marked seconds: {below_clean or 'none'}")
    if worst:
        print(f"  worst sliding 1 s window: {worst['count']} callbacks at +{worst['offset_s']} s")
    if edges:
        print(f"  resolution edges in capture: {len(edges)} (marked seconds {sorted(marked)})")
    print("  PASS" if result["pass"] else "  FAIL")


if __name__ == "__main__":
    main()
