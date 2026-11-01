#!/usr/bin/env python3
"""Bounded Jackdaw display diagnostic. Always ends graphical tests within 20s.

Runs in its own worktree; never targets an existing editor. GNU timeout is the
independent hard watchdog, including if this controller or the renderer hangs.
"""
import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
PORT = 15713


def display_mode(hz):
    from gi.repository import Gio, GLib
    proxy = Gio.DBusProxy.new_for_bus_sync(
        Gio.BusType.SESSION, Gio.DBusProxyFlags.NONE, None,
        'org.gnome.Mutter.DisplayConfig', '/org/gnome/Mutter/DisplayConfig',
        'org.gnome.Mutter.DisplayConfig', None)
    serial, monitors, logical, _ = proxy.call_sync(
        'GetCurrentState', None, Gio.DBusCallFlags.NONE, 5000, None).unpack()
    lookup = {spec[0]: modes for spec, modes, _ in monitors}
    configs, selected = [], []
    for x, y, scale, transform, primary, specs, _ in logical:
        outputs = []
        for spec in specs:
            connector = spec[0]
            modes = lookup[connector]
            current = next(m for m in modes if m[-1].get('is-current', False))
            candidates = [m for m in modes if m[1:3] == current[1:3] and '+vrr' not in m[0]]
            target = min(candidates, key=lambda m: abs(m[3] - hz))
            if abs(target[3] - hz) > 1:
                raise RuntimeError(f'{connector}: no fixed {hz}Hz mode at current resolution')
            outputs.append((connector, target[0], {}))
            selected.append((connector, target[0]))
        configs.append((x, y, scale, transform, primary, outputs))
    proxy.call_sync('ApplyMonitorsConfig', GLib.Variant(
        '(uua(iiduba(ssa{sv}))a{sv})', (serial, 1, configs, {})),
        Gio.DBusCallFlags.NONE, 10000, None)
    return selected


def reset_output():
    from gi.repository import Gio, GLib
    proxy = Gio.DBusProxy.new_for_bus_sync(
        Gio.BusType.SESSION, Gio.DBusProxyFlags.NONE, None,
        'org.gnome.Mutter.DisplayConfig', '/org/gnome/Mutter/DisplayConfig',
        'org.gnome.Mutter.DisplayConfig', None)
    def power(value):
        proxy.call_sync('org.freedesktop.DBus.Properties.Set', GLib.Variant(
            '(ssv)', ('org.gnome.Mutter.DisplayConfig', 'PowerSaveMode', GLib.Variant('i', value))),
            Gio.DBusCallFlags.NONE, 5000, None)
    try:
        power(3)
        time.sleep(0.5)
    finally:
        power(0)


def rpc(method, params=None):
    body = {'jsonrpc': '2.0', 'id': 1, 'method': method}
    if params is not None:
        body['params'] = params
    request = urllib.request.Request(f'http://127.0.0.1:{PORT}',
        data=json.dumps(body).encode(), headers={'Content-Type': 'application/json'})
    with urllib.request.urlopen(request, timeout=0.8) as response:
        result = json.load(response)
    if 'error' in result:
        raise RuntimeError(result['error'])
    return result['result']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--seconds', type=float, default=20)
    parser.add_argument('--refresh', type=int, choices=(60, 165), default=60)
    parser.add_argument('--adapter', help='Bevy WGPU_ADAPTER_NAME substring')
    parser.add_argument('--play', action='store_true')
    parser.add_argument('--review', action='store_true', help='Run the opt-in dusk/replenishment review')
    parser.add_argument('--reset-output', action='store_true', help='Force display power-cycle after a 60Hz test too')
    parser.add_argument('--name', default='baseline')
    parser.add_argument('--command', nargs=argparse.REMAINDER,
                        help='Watchdog self-test only: no display changes, run this command')
    args = parser.parse_args()
    if not 0 < args.seconds <= 20:
        parser.error('--seconds must be greater than zero and no more than 20')
    project = ROOT / 'jackdaw'
    manifest = project / 'jackdaw.toml'
    original = manifest.read_text()
    env = dict(os.environ)
    env.update(JACKDAW_REMOTE_PORT=str(PORT), JACKDAW_PIE_PREBUILD='0',
               JACKDAW_OPEN_PROJECT=str(project), CARGO_BUILD_JOBS='2')
    if args.adapter:
        env['WGPU_ADAPTER_NAME'] = args.adapter
    command = args.command or ['/home/buddha/src/jackdaw/target/debug/jackdaw']
    output = Path('/tmp') / f'jackdaw-flicker-{args.name}.log'
    report = {'name': args.name, 'limit_seconds': args.seconds,
              'refresh_hz': args.refresh, 'adapter': args.adapter, 'play': args.play or args.review, 'review': args.review}
    process = None
    display_changed = False
    test_manifest = None
    try:
        if not args.command:
            try:
                existing = rpc('jackdaw/status')
            except (OSError, RuntimeError, ValueError):
                existing = None
            if existing is not None:
                raise RuntimeError('Test port already has an editor; refusing to touch it')
            report['mode'] = display_mode(args.refresh)
            display_changed = True
            if args.play or args.review:
                block = '[[run]]\nname = "Live listen: clock + square"\nargs = ["--seed=42", "--module-demo"]\n'
                if args.review:
                    block = '[[run]]\nname = "Review: dusk and replenishment"\nargs = ["--seed=42", "--replenishment-review=/tmp/stubby-replenishment-review"]\n'
                if block not in original:
                    raise RuntimeError('Requested review/live-listen run config missing')
                remainder = original.replace('\n' + block, '')
                at = remainder.index('[[run]]')
                test_manifest = remainder[:at] + block + '\n' + remainder[at:]
                manifest.write_text(test_manifest)
        started = time.monotonic()
        with output.open('w') as log:
            process = subprocess.Popen(
                ['timeout', '--signal=KILL', f'{args.seconds}s', *command],
                cwd=ROOT, env=env, start_new_session=True, stdout=log, stderr=subprocess.STDOUT)
            played = False
            states = []
            while process.poll() is None:
                if not args.command:
                    try:
                        status = rpc('jackdaw/status')
                        if not states or states[-1] != status['pie']:
                            states.append(status['pie'])
                        if (args.play or args.review) and not played:
                            if args.review:
                                rpc('jackdaw/call_operator', {'id': 'window.resize_panel', 'params': {'window_id': 'jackdaw.game', 'height': 600.0}})
                            rpc('jackdaw/call_operator', {'id': 'pie.play'})
                            played = True
                    except (OSError, RuntimeError, ValueError):
                        pass
                time.sleep(0.1)
            report.update(exit_code=process.returncode,
                          runtime_seconds=round(time.monotonic() - started, 3),
                          states=states, play_requested=played)
    finally:
        if process is not None:
            # Also reap descendants in this diagnostic's own process group.
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait()
        if test_manifest is not None:
            if manifest.read_text() == test_manifest:
                manifest.write_text(original)
            else:
                report['manifest_restore_conflict'] = True
        if display_changed:
            report['recovery_mode'] = display_mode(60)
            if args.refresh != 60 or args.reset_output:
                reset_output()
        report['log'] = str(output)
        Path('/tmp', f'jackdaw-flicker-{args.name}.json').write_text(json.dumps(report, indent=2))
        print(json.dumps(report), flush=True)


if __name__ == '__main__':
    main()
