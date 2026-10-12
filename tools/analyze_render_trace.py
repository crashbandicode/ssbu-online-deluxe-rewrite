"""Summarize CPU-side render API spans, not GPU time or displayed frames."""
import argparse
import csv
import io
import json
from pathlib import Path
from analyze_timing import percentile

# Names match src/render_trace.rs kind constants.
KINDS = {
    1: "present_call",
    2: "acquire_call",
    3: "texture_cpu_wait",
    4: "sync_zero_timeout_poll",
    5: "dispatch_flush",
    6: "hid_poll_marker",
    7: "hid_sampling_counter",
    12: "index_selection",
    13: "present_entry",
    14: "late_flush",
    15: "record_fighters",
    16: "post_scene",
    17: "outstanding_after_present",
    18: "menu_draw",
    19: "menu_draw_ptr",
    20: "menu_submit_before",
    21: "menu_submit_before_ptr",
    22: "menu_submit_after",
    23: "menu_submit_after_ptr",
    24: "menu_end_recording",
    25: "menu_end_handle",
    26: "menu_end_buffer",
    27: "sync_infinite_other",
    28: "menu_resize",
    29: "flush_plan",
    30: "flush_object",
    31: "flush_narrow",
    32: "flush_word",
    33: "flush_wide_pre",
    34: "flush_wide_post",
    35: "submit_incoming",
    36: "submit_incoming_handle",
    37: "menu_gpu_poll",
    38: "menu_gpu_fence",
}
COLUMNS = ["event", "kind", "begin_tick", "end_tick", "index", "result"]
FRAME_BUDGET_MS = 16.6667
SPIKE_LIMITS = (
    ("gt_16_6667_ms", 16.6667),
    ("gt_20_ms", 20.0),
    ("gt_25_ms", 25.0),
    ("gt_33_3333_ms", 33.3333),
)
# CPU wrappers. A completion inside one tick is a real zero, not a point marker.
SPAN_KINDS = {1, 2, 3, 4, 5, 27}
# Phase, HID, draw, submit, handle, and resize markers. Classified by kind.
POINT_KINDS = {
    6, 7, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 28,
    29, 30, 31, 32, 33, 34, 35, 36, 37, 38,
}
SUMMARY = (
    "present_call_intervals are CPU-side guest present cadence from kind 1 begin ticks. "
    "Kinds 1-5 and 27 are CPU API spans at any duration, including zero. "
    "They are not GPU completion or scanout."
)
POINT_NOTE = (
    "Kinds 6, 7, 12-26, and 28-38 are point markers by kind. "
    "Kinds 1-5 and 27 stay in span statistics even when the duration is zero. "
    "Every one of those values is CPU API time, never GPU time or scanout."
)
UNKNOWN_RECORDING = (
    "No kind 24-26 samples. Redirect state is unknown. This is not a zero EndRecording pass."
)
if set(KINDS) != SPAN_KINDS | POINT_KINDS or not SPAN_KINDS.isdisjoint(POINT_KINDS):
    raise RuntimeError("span and point kinds must partition the kind table")


def pointer_from_halves(low, high):
    """Rebuild a guest u64 from the signed i32 halves stored in index and result."""
    return (low & 0xFFFFFFFF) | ((high & 0xFFFFFFFF) << 32)


def interval_acceptance(intervals_ms):
    total = len(intervals_ms)
    report = {}
    for key, limit in SPIKE_LIMITS:
        count = sum(value > limit for value in intervals_ms)
        report[key] = {"count": count, "fraction": count / total}
    longest = run = 0
    excess = 0.0
    for value in intervals_ms:
        if value > FRAME_BUDGET_MS:
            run += 1
            longest = max(longest, run)
            excess += value - FRAME_BUDGET_MS
        else:
            run = 0
    report["longest_run_gt_16_6667"] = longest
    report["excess_ms_above_16_6667"] = excess
    return report


def _draw_windows(events):
    """CPU timing boundary from menu-draw kind 18 through outer host submit kind 22.

    The pair is a sample-tick window for the same texture index. It is not a GPU lifetime.
    """
    pending = {}
    windows = []
    for event in events:
        index = event["index"]
        if event["kind"] == 18:
            pending.setdefault(index, []).append(event["begin_tick"])
        elif event["kind"] == 22 and pending.get(index):
            start = pending[index].pop(0)
            stop = event["begin_tick"]
            if stop >= start:
                windows.append((index, start, stop))
    return windows


def _recording_bucket(rows):
    return {
        "count": len(rows),
        "handles": sorted({f"0x{handle:016x}" for handle, _ in rows}),
        "buffers": sorted({f"0x{buffer:016x}" for _, buffer in rows}),
    }


def end_recording_report(events):
    """Host EndRecording is kinds 24-26. Absence is unknown, not a zero pass."""
    windows = _draw_windows(events)
    inside, other = [], []
    consumed = set()
    unpaired = 0
    index = 0
    while index < len(events):
        event = events[index]
        if event["kind"] != 24:
            index += 1
            continue
        triple = events[index:index + 3]
        same_tick = (
            len(triple) == 3
            and triple[1]["kind"] == 25
            and triple[2]["kind"] == 26
            and triple[1]["begin_tick"] == event["begin_tick"] == triple[2]["begin_tick"]
            and triple[1]["end_tick"] == event["end_tick"] == triple[2]["end_tick"]
        )
        if not same_tick:
            unpaired += 1
            index += 1
            continue
        handle = pointer_from_halves(triple[1]["index"], triple[1]["result"])
        buffer = pointer_from_halves(triple[2]["index"], triple[2]["result"])
        in_draw = any(
            texture == event["index"] and start <= event["begin_tick"] <= stop
            for texture, start, stop in windows
        )
        (inside if in_draw else other).append((handle, buffer))
        consumed.update({index, index + 1, index + 2})
        index += 3
    orphans = sum(
        1 for position, event in enumerate(events)
        if event["kind"] in (25, 26) and position not in consumed
    )
    if not inside and not other and unpaired == 0 and orphans == 0:
        return {
            "redirect": "unknown",
            "inside_ui_draw": None,
            "other_use": None,
            "note": UNKNOWN_RECORDING,
        }
    report = {
        "redirect": "samples_present" if inside or other else "incomplete",
        "inside_ui_draw": _recording_bucket(inside) if inside or other else None,
        "other_use": _recording_bucket(other) if inside or other else None,
        "note": (
            "Kind 24-26 samples are host embedded EndRecording observations. "
            "inside_ui_draw means the sample tick falls in the CPU timing boundary from "
            "menu-draw kind 18 through the outer host submit kind 22 for the same texture index. "
            "That boundary is not a GPU command lifetime. other_use is any other host EndRecording sample."
            if inside or other else
            "EndRecording kinds were present without a complete kind 24, 25, 26 triple. "
            "This is not a zero EndRecording pass."
        ),
    }
    if unpaired:
        report["unpaired"] = unpaired
    if orphans:
        report["orphan_halves"] = orphans
    return report


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
    point_samples = {}
    for kind, name in KINDS.items():
        group = [e for e in events if e["kind"] == kind]
        if not group:
            continue
        columns = {
            "observed_indices": sorted({e["index"] for e in group}),
            "native_results": sorted({e["result"] for e in group}),
        }
        if kind in SPAN_KINDS:
            spans[name] = stats([(e["end_tick"] - e["begin_tick"]) * 1000 / hz for e in group])
            spans[name].update(columns)
        else:
            point_samples[name] = {"count": len(group), **columns}
    present_ticks = [e["begin_tick"] for e in events if e["kind"] == 1]
    intervals = [b - a for a, b in zip(present_ticks, present_ticks[1:])]
    if any(v <= 0 for v in intervals):
        raise ValueError("nonmonotonic present calls")
    warnings = []
    if skipped:
        warnings.append("Events were skipped; do not use this trace for an A/B claim.")
    if metadata["engine"] == "Mixed":
        warnings.append("Engine state changed during capture.")
    present_intervals = None
    if intervals:
        present_ms = [v * 1000 / hz for v in intervals]
        present_intervals = stats(present_ms)
        present_intervals.update(interval_acceptance(present_ms))
    return {
        "metric": "cpu_api_spans", "engine": metadata["engine"],
        "build": metadata["build"], "id": metadata["id"], "label": metadata["label"],
        "event_count": len(events), "skipped": skipped, "spans": spans,
        "point_samples": point_samples,
        "present_call_intervals": present_intervals,
        "end_recording": end_recording_report(events),
        "warnings": warnings,
        "summary": SUMMARY,
        "notes": ["Spans may overlap; do not sum their durations.",
                  "Acquire result -1 means unavailable through the embedded backend.",
                  "Kinds 12-17 are phase markers. Their result is a hashed thread label, not an API result.",
                  "Kind 17 index is tracked-window acquires minus presents after that present returns. Zero is a real count.",
                  "Kind 5 is the flush wrapper and is installed for every startup mode, including Vanilla.",
                  "Kinds 18-23 are embedded menu draw and submit point samples. Kinds 19, 21, and 23 are pointer halves.",
                  "Kinds 24-26 are host embedded EndRecording: texture index, command-handle halves, buffer halves.",
                  "Kind 27 is an infinite wait on a sync that is not the texture sync. Its duration is CPU wait time.",
                  "Kind 28 counts an embedded backend Resizing log. It is a point sample, not a GPU resize.",
                  "Kinds 29-36 are flush-field and incoming-list point samples.",
                  "Kind 37 is a timeout-0 poll of a fence placed after an earlier menu submit. "
                  "Its result is NVNsyncWaitResult. It is not a duration and does not prove "
                  "command bytes were overwritten.",
                  "Kind 38 result is the slot fenced after the host menu submit returned, "
                  "or -1 when both slots were still pending.",
                  POINT_NOTE],
        "not_measured": ["GPU execution", "displayed frames", "input-to-photon latency", "scanout"],
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
