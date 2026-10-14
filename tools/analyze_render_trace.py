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
    60: "backoff_frame_sleep",
    61: "backoff_frame_peak",
    62: "thread_setup_pointer",
    63: "thread_setup_info",
    64: "ui_dispatch",
    66: "dependency_wait",
    77: "ui_job",
}
CSS_KINDS = (60, 61, 62, 63, 64, 66, 77)
THREAD_HASH_MULTIPLIER = 0x9E3779B185EBCA87
U64_MASK = 0xFFFFFFFFFFFFFFFF
COLUMNS = ["event", "kind", "begin_tick", "end_tick", "index", "result"]
FRAME_BUDGET_MS = 16.6667
SPIKE_LIMITS = (
    ("gt_16_6667_ms", 16.6667),
    ("gt_20_ms", 20.0),
    ("gt_25_ms", 25.0),
    ("gt_33_3333_ms", 33.3333),
)
# CPU wrappers. A completion inside one tick is a real zero, not a point marker.
SPAN_KINDS = {1, 2, 3, 4, 5, 27, 66, 77}
# Phase, HID, draw, submit, handle, resize, backoff, and thread markers. Classified by kind.
POINT_KINDS = {
    6, 7, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 28,
    29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 60, 61, 62, 63, 64,
}
SUMMARY = (
    "present_call_intervals are CPU-side guest present cadence from kind 1 begin ticks. "
    "Kinds 1-5, 27, 66, and 77 are CPU API spans at any duration, including zero. "
    "They are not GPU completion or scanout."
)
POINT_NOTE = (
    "Kinds 6, 7, 12-26, 28-38, and 60-64 are point markers by kind. "
    "Kinds 1-5, 27, 66, and 77 stay in span statistics even when the duration is zero. "
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


def thread_label(pointer):
    """Hashed thread label: the pure function of a ThreadType pointer in src/render_trace.rs."""
    mixed = ((pointer + THREAD_HASH_MULTIPLIER) & U64_MASK) * THREAD_HASH_MULTIPLIER & U64_MASK
    rotated = ((mixed << 27) | (mixed >> 37)) & U64_MASK
    return (rotated & 0xFFFFFFFF) & 0x7FFFFFFF


def _distribution(values):
    """Mean, p50, p99, and max of raw sample values (not milliseconds)."""
    return {"mean": sum(values) / len(values), "p50": percentile(values, .5),
            "p99": percentile(values, .99), "max": max(values)}


def css_capture_report(events, stats, hz):
    """Character-select diagnostics for kinds 60-64, 66, 77. None when none are present."""
    by_kind = {kind: [e for e in events if e["kind"] == kind] for kind in CSS_KINDS}
    if not any(by_kind.values()):
        return None

    def span_ms(group):
        return [(e["end_tick"] - e["begin_tick"]) * 1000 / hz for e in group]

    report = {}
    waits, jobs = by_kind[66], by_kind[77]
    job_labels = {e["result"] for e in jobs}
    grouped = {}
    for event in waits:
        grouped.setdefault(event["index"], []).append(event)
    report["dependency_wait_by_counter"] = {
        (f"0x{offset:x}" if offset >= 0 else str(offset)): {
            **stats(span_ms(group)),
            "thread_labels": sorted({e["result"] for e in group}),
        }
        for offset, group in sorted(grouped.items())
    }
    report["ui_job"] = (
        {**stats(span_ms(jobs)), "thread_labels": sorted(job_labels)} if jobs else None
    )
    report["nested_waits"] = sum(e["result"] in job_labels for e in waits)
    sleeps, peaks = by_kind[60], by_kind[61]
    backoff = {}
    if sleeps:
        backoff["sleeps"] = _distribution([e["index"] for e in sleeps])
        backoff["sleep_us"] = _distribution([e["result"] for e in sleeps])
    if peaks:
        backoff["peak_sleep_us_max"] = max(e["index"] for e in peaks)
    report["backoff_per_frame"] = backoff
    threads = []
    for position, event in enumerate(events):
        if event["kind"] != 62:
            continue
        pointer = pointer_from_halves(event["index"], event["result"])
        info = events[position + 1] if position + 1 < len(events) else None
        if info is not None and (
            info["kind"] != 63 or info["begin_tick"] != event["begin_tick"]
        ):
            info = None
        core_info = None
        if info is not None and info["result"] != -1:
            core_info = info["result"] & 0xFFFFFFFF
        threads.append({
            "pointer": f"0x{pointer:016x}",
            "label": thread_label(pointer),
            "priority": info["index"] if info is not None else None,
            "core_info": core_info,
            "ideal_core": core_info & 0xFF if core_info is not None else None,
            "affinity_mask": core_info >> 8 if core_info is not None else None,
        })
    report["threads"] = threads
    dispatch = by_kind[64]
    report["ui_dispatch"] = {
        "count": len(dispatch),
        "worker_priority_pairs": sorted({
            (e["index"] & 0xFF, (e["index"] >> 8) & 0xFF) for e in dispatch
        }),
    }
    return report


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
    # `elapsed` ran its duration; `scene_end` is a CAPTURE_MATCH capture ended
    # by the match itself. Both have complete footers.
    if metadata["reason"] not in ("elapsed", "scene_end"):
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
    css_capture = css_capture_report(events, stats, hz)
    result = {
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
                  "Kinds 60-61 are per-present dispatcher backoff samples: sleep count and summed "
                  "microseconds, then longest single sleep in microseconds and idle streaks started.",
                  "Kinds 62-63 are one-shot thread setup rows: pointer halves, then priority and packed "
                  "core info. Kind 62 pointers hash to the thread labels in kinds 12-17, 64, 66, and 77.",
                  "Kind 64 is a UI job dispatcher registration: worker in the low byte of index, "
                  "queue priority in the next byte, result is the registering thread label.",
                  "Kinds 66 and 77 are wall-time spans: the dependency wait on main+0x354ccd0 "
                  "(index is the counter offset from the text base) and the UI job on its worker.",
                  POINT_NOTE],
        "not_measured": ["GPU execution", "displayed frames", "input-to-photon latency", "scanout"],
    }
    if css_capture is not None:
        result["css_capture"] = css_capture
    return result


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
