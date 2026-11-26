#!/usr/bin/env python3
"""One bounded Play of this project in the shared editor: the smoke test for a re-pin.

Starts its own editor (never an existing one) on a private remote port, opens this
project, presses Play on the first [[run]] in jackdaw.toml, and lets it run. GNU
`timeout --signal=KILL` is an independent hard watchdog: whatever hangs, everything
it started is dead within --seconds (at most 20). Display settings are left alone.

Passes (exit 0) when Play reached `running` and the log holds no panic. Build the
game first (`tools/jackdaw.sh build`): Play compiles it otherwise, and a cold build
does not finish in 20 seconds.

    tools/jackdaw.sh smoke [--seconds N] [--name NAME]
"""
import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
PORT = 15713


def rpc(method, params=None):
    body = {'jsonrpc': '2.0', 'id': 1, 'method': method}
    if params is not None:
        body['params'] = params
    request = urllib.request.Request(f'http://127.0.0.1:{PORT}', data=json.dumps(body).encode(),
                                     headers={'Content-Type': 'application/json'})
    with urllib.request.urlopen(request, timeout=0.8) as response:
        result = json.load(response)
    if 'error' in result:
        raise RuntimeError(result['error'])
    return result['result']


def main():
    parser = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    parser.add_argument('--seconds', type=float, default=20)
    parser.add_argument('--name', default='smoke')
    parser.add_argument('--source', type=Path,
                        default=Path(os.environ.get('JACKDAW_SOURCE', ROOT.parent / 'jackdaw')))
    args = parser.parse_args()
    if not 0 < args.seconds <= 20:
        parser.error('--seconds must be greater than zero and no more than 20')
    source = args.source.resolve()
    subprocess.run([sys.executable, str(ROOT / 'tools/check_engine_pins.py'),
                    '--source', str(source), '--binaries'], check=True)
    try:
        rpc('jackdaw/status')
        sys.exit(f'An editor already answers on port {PORT}; refusing to touch it.')
    except (OSError, RuntimeError, ValueError):
        pass

    env = dict(os.environ, JACKDAW_REMOTE_PORT=str(PORT), JACKDAW_PIE_PREBUILD='0',
               JACKDAW_OPEN_PROJECT=str(ROOT), CARGO_BUILD_JOBS='2')
    log_path = Path('/tmp') / f'{ROOT.name}-play-{args.name}.log'
    report = {'name': args.name, 'limit_seconds': args.seconds, 'log': str(log_path)}
    process, states, played = None, [], False
    started = time.monotonic()
    try:
        with log_path.open('w') as log:
            process = subprocess.Popen(
                ['timeout', '--signal=KILL', f'{args.seconds}s', str(source / 'target/debug/jackdaw')],
                cwd=source, env=env, start_new_session=True, stdout=log, stderr=subprocess.STDOUT)
            while process.poll() is None:
                try:
                    status = rpc('jackdaw/status')
                    if not states or states[-1] != status['pie']:
                        states.append(status['pie'])
                    if not played:
                        rpc('jackdaw/call_operator', {'id': 'pie.play'})
                        played = True
                except (OSError, RuntimeError, ValueError):
                    pass
                time.sleep(0.1)
    finally:
        if process is not None:
            try:  # also reap descendants (the game) in this run's process group
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait()
    text = log_path.read_text(errors='replace')
    panics = text.count('panicked at')
    report.update(runtime_seconds=round(time.monotonic() - started, 3), states=states,
                  play_requested=played, panics=panics,
                  passed='running' in states and panics == 0)
    print(json.dumps(report))
    if not report['passed']:
        hint = ' (still building: run tools/jackdaw.sh build first)' if states[-1:] == ['building'] else ''
        print(f'Smoke test failed{hint}; see {log_path}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
