"""Run lifecycle.py preflight on a config (read-only) and print its FAIL lines; exit code = preflight's."""
import subprocess
import sys

lifecycle, config = sys.argv[1], sys.argv[2]
done = subprocess.run([sys.executable, '-B', '-I', lifecycle, 'preflight', config], capture_output=True, text=True)
fails = [l for l in done.stderr.splitlines() if 'FAIL' in l]
print(f'preflight exit={done.returncode} ' + (' | '.join(fails)[:600] if fails else 'no FAIL lines'))
sys.exit(done.returncode)
