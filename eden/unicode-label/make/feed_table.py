"""Per-phase table of follow_feed's record (feed-b.json). Usage: feed_table.py <feed-b.json>"""
import json
import sys

d = json.load(open(sys.argv[1]))
print('problems', d['problems'], 'extra', d['extra'], 'seconds', d['seconds'])
for p in d['phases']:
    drawn = {k: (v['own'], v['fallback']) for k, v in (p.get('drawn') or {}).items()}
    print(p['phase'], p['name'], 'late' if p.get('late') else 'on time', 'rebuilds', p.get('rebuilds'),
          'dynamic', ' '.join(p.get('dynamic') or []) or 'none', 'index', p.get('index_bytes'),
          'texture', p.get('texture_bytes'), 'drawn(own,fallback)', drawn)
