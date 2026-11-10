"""The index-bytes trace of follow_feed's record (feed-b.json): one line per renderer-status.txt rewrite, with the
phase on screen at that moment. Usage: index_trace.py <feed-b.json>"""
import json
import sys

d = json.load(open(sys.argv[1]))
print(f'{"t (s)":>6}  {"index":<16}{"index bytes":>12}{"rebuilds":>9}{"dynamic":>8}  dynamic set')
for s in d.get('index_trace', []):
    print(f'{s["t"]:>6}  {s["index"]:<16}{s["index_bytes"]:>12,}{s["rebuilds"]:>9}{s["dynamic_glyphs"]:>8}  {s["dynamic_set"]}')
