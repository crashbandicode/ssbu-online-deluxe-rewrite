"""Analyze raw guest render-callback timing; never infer displayed-frame loss."""
import argparse
import csv
import io
import json
import math
from pathlib import Path

COLUMNS = ["sample", "callback", "tick", "buttons", "fx_mode", "fx_scope", "fx_effective_off", "overlay_view"]


def percentile(values, fraction):
    ordered = sorted(values)
    pos = (len(ordered) - 1) * fraction
    lo, hi = math.floor(pos), math.ceil(pos)
    return ordered[lo] + (ordered[hi] - ordered[lo]) * (pos - lo)


def analyze(text):
    metadata, data = {}, []
    lines = text.splitlines()
    if not lines or lines[-1] != "# complete=1":
        raise ValueError("incomplete CSV: missing final completion marker")
    for line in lines:
        if line.startswith("# "):
            key, sep, value = line[2:].partition("=")
            if not sep or key in metadata:
                raise ValueError("invalid or duplicate metadata")
            metadata[key] = value
        else:
            data.append(line)
    required = {"schema", "metric", "engine", "tick_hz", "duration_ticks", "id", "build",
                "label", "reason", "skipped", "sample_count", "complete"}
    if not required <= metadata.keys():
        raise ValueError("missing timing metadata")
    if metadata["schema"] != "1" or metadata["metric"] != "render_callback":
        raise ValueError("unsupported schema or metric")
    if metadata["reason"] != "elapsed":
        raise ValueError(f"capture is not complete: {metadata['reason']}")
    hz = int(metadata["tick_hz"])
    skipped = int(metadata["skipped"])
    if hz <= 0 or skipped < 0:
        raise ValueError("invalid clock frequency or skip count")
    reader = csv.DictReader(io.StringIO("\n".join(data)))
    if reader.fieldnames != COLUMNS:
        raise ValueError("unexpected CSV columns")
    samples = []
    for n, row in enumerate(reader):
        if None in row or any(v is None for v in row.values()):
            raise ValueError("malformed CSV row")
        sample = {k: int(v) for k, v in row.items()}
        if sample["sample"] != n or any(v < 0 for v in sample.values()):
            raise ValueError("invalid sample sequence or negative value")
        if any(sample[k] not in (0, 1) for k in ("fx_mode", "fx_scope", "fx_effective_off")):
            raise ValueError("invalid FX condition")
        if sample["overlay_view"] not in (0, 1, 2, 3):
            raise ValueError("invalid overlay view")
        samples.append(sample)
    if len(samples) != int(metadata["sample_count"]) or len(samples) < 2:
        raise ValueError("sample count mismatch or insufficient samples")
    elapsed_ticks = samples[-1]["tick"] - samples[0]["tick"]
    if elapsed_ticks < int(metadata["duration_ticks"]) or int(metadata["duration_ticks"]) <= 0:
        raise ValueError("capture shorter than its declared duration")
    intervals, missing_callbacks, excluded_intervals = [], 0, 0
    for before, after in zip(samples, samples[1:]):
        tick_delta = after["tick"] - before["tick"]
        count_delta = after["callback"] - before["callback"]
        if tick_delta <= 0 or count_delta <= 0:
            raise ValueError("nonmonotonic clock or callback sequence")
        if count_delta != 1:
            missing_callbacks += count_delta - 1
            excluded_intervals += 1
        else:
            intervals.append(tick_delta * 1000 / hz)
    if not intervals:
        raise ValueError("no consecutive callback intervals")
    conditions = sorted({(s["fx_mode"], s["fx_scope"], s["fx_effective_off"], s["overlay_view"]) for s in samples})
    warnings = []
    if skipped or missing_callbacks:
        warnings.append("Capture gaps: affected intervals excluded; trace is unsuitable for an A/B claim.")
    if len(conditions) != 1:
        warnings.append("FX settings or overlay view changed during capture; do not pool conditions for A/B comparison.")
    if len(intervals) < 120:
        warnings.append("Fewer than 120 consecutive intervals; percentile estimates are weak.")
    return {
        "metric": "render_callback", "engine": metadata["engine"],
        "build": metadata["build"], "label": metadata["label"], "id": metadata["id"],
        "samples": len(samples), "intervals": len(intervals),
        "elapsed_guest_seconds": elapsed_ticks / hz,
        "p50_ms": percentile(intervals, .50), "p95_ms": percentile(intervals, .95),
        "p99_ms": percentile(intervals, .99), "max_ms": max(intervals),
        "callback_gaps_over_ms": {str(t): sum(v > t for v in intervals) for t in (20, 25, 1000 / 30)},
        "skipped": skipped, "missing_callbacks": missing_callbacks,
        "excluded_intervals": excluded_intervals,
        "conditions": [{"fx_mode": c[0], "fx_scope": c[1], "fx_effective_off": bool(c[2]), "overlay_view": c[3]} for c in conditions],
        "warnings": warnings,
        "not_measured": ["displayed frames", "missed vblanks", "GPU execution time", "input-to-photon latency"],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("csv", type=Path)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    try:
        result = analyze(args.csv.read_text())
    except (ValueError, OSError, csv.Error) as error:
        parser.exit(2, f"Invalid timing trace: {error}\n")
    if args.json:
        print(json.dumps(result, indent=2))
        return
    print(f"{result['label']}: {result['samples']} callback samples in {result['elapsed_guest_seconds']:.3f} guest seconds")
    print("Callback interval ms: " + ", ".join(f"{k}={result[k]:.3f}" for k in ("p50_ms", "p95_ms", "p99_ms", "max_ms")))
    print("Callback gaps above thresholds: " + str(result["callback_gaps_over_ms"]))
    for warning in result["warnings"]:
        print(f"Warning: {warning}")
    print("These are guest callback intervals, not displayed frames or input latency.")


if __name__ == "__main__":
    main()
