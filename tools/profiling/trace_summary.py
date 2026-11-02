"""Summarise a Bevy `trace_chrome` JSON (one event per line): self-inclusive time per
span name over a time window, per thread. Usage: trace_summary.py trace.json [from_s] [to_s]"""
import json, sys, collections
path = sys.argv[1]
lo = float(sys.argv[2]) * 1e6 if len(sys.argv) > 2 else 0
hi = float(sys.argv[3]) * 1e6 if len(sys.argv) > 3 else float('inf')
stacks = collections.defaultdict(list)
total = collections.Counter(); count = collections.Counter()
threads = {}
frames = 0
with open(path) as f:
    for line in f:
        line = line.strip().rstrip(',')
        if not line.startswith('{'):
            continue
        try:
            e = json.loads(line)
        except ValueError:
            continue
        ph = e.get('ph')
        if ph == 'M':
            threads[e['tid']] = e['args']['name']; continue
        tid, ts, name = e.get('tid'), e.get('ts', 0), e.get('name', '')
        if ph == 'B':
            stacks[tid].append((name, ts))
        elif ph == 'E' and stacks[tid]:
            n, start = stacks[tid].pop()
            if lo <= start <= hi:
                key = (threads.get(tid, tid) if not str(threads.get(tid, '')).startswith(('Compute', 'Async', 'IO')) else 'pool', n)
                total[key] += ts - start; count[key] += 1
                if n.startswith('frame'): frames += 1
span = (hi if hi != float('inf') else 0) - lo
print(f'frames(spans named frame*): {frames}')
for (thread, name), t in total.most_common(45):
    print(f'{t/1e3:10.1f} ms  {count[(thread, name)]:7d}x  {str(thread)[:14]:14s}  {name[:120]}')
