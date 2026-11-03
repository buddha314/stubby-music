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
import sys
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
PORT = 15713


def kde():
    return 'KDE' in os.environ.get('XDG_CURRENT_DESKTOP', '')


def kde_outputs():
    import re
    out = subprocess.run(['kscreen-doctor', '-o'], capture_output=True, text=True, check=True).stdout
    out = re.sub(r'\x1b\[[0-9;]*m', '', out)
    outputs = []
    for block in re.split(r'(?m)^Output: ', out)[1:]:
        name = block.split()[1]
        if not re.search(r'(?m)^\s*enabled$', block):
            continue
        modes = re.findall(r'(\d+):(\d+)x(\d+)@([\d.]+)(\*?)', block.split('Modes:')[1].splitlines()[0])
        vrr = re.search(r'Vrr: (\S+)', block)
        outputs.append((name, [(i, int(w), int(h), float(r), cur == '*') for i, w, h, r, cur in modes],
                        vrr.group(1) if vrr else None))
    return outputs


def kde_display_mode(hz, restore=None):
    """Set every enabled output that has an hz mode at its current resolution.

    With restore ({output: mode id}), put those modes back instead.
    Returns (report, {output: original mode id}).
    """
    report, original, args = [], {}, []
    for name, modes, vrr in kde_outputs():
        current = next(m for m in modes if m[4])
        if restore is not None:
            target = next((m for m in modes if m[0] == restore.get(name)), current)
        else:
            candidates = [m for m in modes if m[1:3] == current[1:3]]
            target = min(candidates, key=lambda m: abs(m[3] - hz))
            if abs(target[3] - hz) > 1:
                report.append(f'{name}: unchanged {current[1]}x{current[2]}@{current[3]} (no {hz}Hz) vrr={vrr}')
                continue
        original[name] = current[0]
        args.append(f'output.{name}.mode.{target[0]}')
        report.append(f'{name}: {target[1]}x{target[2]}@{target[3]} vrr={vrr}')
    if restore is None and not original:
        raise RuntimeError(f'no output has a {hz}Hz mode at its current resolution')
    if args:
        subprocess.run(['kscreen-doctor', *args], capture_output=True, check=True, timeout=10)
    return report, original


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
    parser.add_argument('--refresh', type=int, choices=(60, 144, 165, 240), default=60)
    parser.add_argument('--adapter', help='Bevy WGPU_ADAPTER_NAME substring')
    parser.add_argument('--play', action='store_true')
    parser.add_argument('--prefabs', action='store_true', help='Review placed prefab runtime behaviour')
    parser.add_argument('--review', action='store_true', help='Run the opt-in dusk/replenishment review')
    parser.add_argument('--reset-output', action='store_true', help='Force display power-cycle after a 60Hz test too')
    parser.add_argument('--name', default='baseline')
    parser.add_argument('--command', nargs=argparse.REMAINDER,
                        help='Watchdog self-test only: no display changes, run this command')
    args = parser.parse_args()
    if not 0 < args.seconds <= 20:
        parser.error('--seconds must be greater than zero and no more than 20')
    if not args.command:
        subprocess.run([sys.executable, str(ROOT / 'tools/check_jackdaw_versions.py'), '--binaries'], check=True)
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
              'refresh_hz': args.refresh, 'adapter': args.adapter, 'play': args.play or args.review or args.prefabs, 'review': args.review, 'prefabs': args.prefabs}
    process = None
    display_changed = False
    kde_original = {}
    test_manifest = None
    try:
        if not args.command:
            try:
                existing = rpc('jackdaw/status')
            except (OSError, RuntimeError, ValueError):
                existing = None
            if existing is not None:
                raise RuntimeError('Test port already has an editor; refusing to touch it')
            if kde():
                report['mode'], kde_original = kde_display_mode(args.refresh)
            else:
                report['mode'] = display_mode(args.refresh)
            display_changed = True
            if args.play or args.review or args.prefabs:
                block = '[[run]]\nname = "Live listen: clock + square"\nargs = ["--seed=42", "--module-demo"]\n'
                if args.review:
                    block = '[[run]]\nname = "Review: dusk and replenishment"\nargs = ["--seed=42", "--replenishment-review=/tmp/stubby-replenishment-review"]\n'
                if args.prefabs:
                    block = '[[run]]\nname = "Review: placed prefab runtime"\nargs = ["--seed=42", "--scene=review_prefab_runtime.bsn", "--prefab-review=/tmp/stubby-prefab-review"]\n'
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
                        if (args.play or args.review or args.prefabs) and not played:
                            if args.review or args.prefabs:
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
            if kde():
                report['recovery_mode'], _ = kde_display_mode(60, restore=kde_original)
            else:
                report['recovery_mode'] = display_mode(60)
            if not kde() and (args.refresh != 60 or args.reset_output):
                reset_output()
        report['log'] = str(output)
        Path('/tmp', f'jackdaw-flicker-{args.name}.json').write_text(json.dumps(report, indent=2))
        print(json.dumps(report), flush=True)


if __name__ == '__main__':
    main()
