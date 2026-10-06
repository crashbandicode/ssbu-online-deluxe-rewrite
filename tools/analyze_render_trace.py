"""Summarize CPU-side render API spans, not GPU time or displayed frames."""
import argparse
import csv
import io
import json
from pathlib import Path
from analyze_timing import percentile

KINDS = {1: "present_call", 2: "acquire_call", 3: "texture_cpu_wait",
         4: "sync_zero_timeout_poll", 5: "dispatch_flush", 6: "hid_poll_marker", 7: "hid_sampling_counter"}
COLUMNS = ["event", "kind", "begin_tick", "end_tick", "index", "result"]


def analyze(text):
    lines = text.splitlines()
    if not lines or lines[-1] != "# complete=1":
        raise ValueError("incomplete API trace")
    metadata, data = {}, []
    for line in lines:
        if line.startswith("# "):
            key, sep, value = line[2:].partition("=")
            if not sep or key in metadata:
                raise ValueError("invalid or duplicate metadata")
            metadata[key] = value
        else:
            data.append(line)
    required = {"schema", "metric", "tick_hz", "id", "build", "label", "engine",
                "reason", "skipped", "event_count", "complete"}
    if not required <= metadata.keys():
        raise ValueError("missing metadata")
    if metadata["schema"] != "1" or metadata["metric"] != "cpu_api_spans":
        raise ValueError("unsupported schema")
    if metadata["reason"] != "elapsed":
        raise ValueError(f"incomplete capture: {metadata['reason']}")
    hz, skipped = int(metadata["tick_hz"]), int(metadata["skipped"])
    if hz <= 0 or skipped < 0:
        raise ValueError("invalid frequency or skip count")
    reader = csv.DictReader(io.StringIO("\n".join(data)))
    if reader.fieldnames != COLUMNS:
        raise ValueError("unexpected columns")
    events = []
    for n, row in enumerate(reader):
        if None in row or any(v is None for v in row.values()):
            raise ValueError("malformed event")
        e = {k: int(v) for k, v in row.items()}
        if e["event"] != n or e["kind"] not in KINDS:
            raise ValueError("invalid sequence or kind")
        if e["begin_tick"] < 0 or e["end_tick"] < e["begin_tick"]:
            raise ValueError("invalid span")
        if e["kind"] in (1, 2) and e["index"] not in range(-1, 64):
            raise ValueError("invalid texture index")
        events.append(e)
    if len(events) != int(metadata["event_count"]) or len(events) < 2:
        raise ValueError("event count mismatch")
    if max(e["end_tick"] for e in events) - min(e["begin_tick"] for e in events) < hz * 10:
        raise ValueError("trace shorter than ten seconds")

    def stats(values):
        return {"count": len(values), "mean_ms": sum(values) / len(values),
                "p50_ms": percentile(values, .5), "p95_ms": percentile(values, .95),
                "p99_ms": percentile(values, .99), "max_ms": max(values)}

    spans = {}
    for kind, name in KINDS.items():
        group = [e for e in events if e["kind"] == kind]
        if group:
            spans[name] = stats([(e["end_tick"] - e["begin_tick"]) * 1000 / hz for e in group])
            spans[name]["observed_indices"] = sorted({e["index"] for e in group if e["index"] >= 0})
            spans[name]["native_results"] = sorted({e["result"] for e in group})
    present_ticks = [e["begin_tick"] for e in events if e["kind"] == 1]
    intervals = [b - a for a, b in zip(present_ticks, present_ticks[1:])]
    if any(v <= 0 for v in intervals):
        raise ValueError("nonmonotonic present calls")
    warnings = []
    if skipped:
        warnings.append("Events were skipped; do not use this trace for an A/B claim.")
    if metadata["engine"] == "Mixed":
        warnings.append("Engine state changed during capture.")
    return {
        "metric": "cpu_api_spans", "engine": metadata["engine"],
        "build": metadata["build"], "id": metadata["id"], "label": metadata["label"],
        "event_count": len(events), "skipped": skipped, "spans": spans,
        "present_call_intervals": stats([v * 1000 / hz for v in intervals]) if intervals else None,
        "warnings": warnings,
        "notes": ["Spans may overlap; do not sum their durations.",
                  "Acquire result -1 means unavailable through the embedded backend.",
                  "Vanilla does not hook dispatch flush, so that span is unavailable there."],
        "not_measured": ["GPU execution", "displayed frames", "input-to-photon latency"],
    }


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("csv", type=Path)
    args = p.parse_args()
    try:
        print(json.dumps(analyze(args.csv.read_text()), indent=2))
    except (OSError, ValueError, csv.Error) as error:
        p.exit(2, f"Invalid API trace: {error}\n")


if __name__ == "__main__":
    main()