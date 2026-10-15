#!/usr/bin/env python3
"""Difference two cumulative raw profiles from the same process.

The coverage build's counters only grow, and `live.profraw` is rewritten every
ten seconds, so a copy taken at the end of each scenario is a running total.
This subtracts an earlier snapshot's counters from a later one and writes a
profile that holds only the counts accumulated between the two, in the same
raw layout (header, data records and names are copied from the later file).
`llvm-profdata` and tools/coverage/report.py then treat it like any other
profile. Both files must come from one run of the same build: the data
section and names must be byte-identical.

Usage: profraw_diff.py <earlier.profraw> <later.profraw> <out.profraw>
"""
import struct
import sys

HEADER_WORDS = 16
DATA_RECORD_BYTES = 64


def parse(path):
    blob = open(path, "rb").read()
    header = struct.unpack_from(f"<{HEADER_WORDS}Q", blob, 0)
    magic, version, binary_ids, num_data, _pad_before, num_counters, pad_after, bitmap, _pad_bitmap, names_size = header[:10]
    if magic != 0xFF6C70726F667281 or version != 10 or bitmap:
        raise SystemExit(f"{path}: not a version-10 raw profile this tool understands")
    offset = HEADER_WORDS * 8 + binary_ids
    data = blob[offset:offset + num_data * DATA_RECORD_BYTES]
    offset += len(data)
    counters = blob[offset:offset + num_counters * 8]
    offset += len(counters) + pad_after
    names = blob[offset:offset + names_size]
    return blob, header, data, counters, names


def main():
    if len(sys.argv) != 4:
        raise SystemExit(__doc__)
    earlier, later, out = sys.argv[1:]
    _, header_a, data_a, counters_a, names_a = parse(earlier)
    blob_b, header_b, data_b, counters_b, names_b = parse(later)
    if data_a != data_b or names_a != names_b or header_a[3] != header_b[3]:
        raise SystemExit("profiles are not from the same build and run (data/names differ)")
    a = struct.unpack(f"<{len(counters_a) // 8}Q", counters_a)
    b = struct.unpack(f"<{len(counters_b) // 8}Q", counters_b)
    negative = sum(1 for x, y in zip(a, b) if y < x)
    diff = struct.pack(f"<{len(b)}Q", *(max(0, y - x) for x, y in zip(a, b)))
    start = HEADER_WORDS * 8 + header_b[2] + len(data_b)
    with open(out, "wb") as handle:
        handle.write(blob_b[:start])
        handle.write(diff)
        handle.write(blob_b[start + len(diff):])
    total = sum(y - x for x, y in zip(a, b))
    print(f"wrote {out}: {len(b)} counters, {total} increments between snapshots, {negative} counters went backwards")


if __name__ == "__main__":
    main()
