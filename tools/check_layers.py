#!/usr/bin/env python3
"""Check the crate layering of split-game-into-systems (system-architecture spec).

cores     game rules and engines, never Bevy        stubby_audio, stubby_rack, stubby_recipes
services  what many systems share                   stubby_core, stubby_ui, bevy_stubby_audio
systems   one per part of the game                  bevy_stubby_rack, bevy_stubby_recipes, ...
game      plugs systems together and holds glue     stubby_game, stubby_jackdaw

A crate depends only on lower layers. A system never depends on another system, nor on
another system's own core. Test-only (dev) dependencies are not checked. Every crate in
the workspace must be listed below: a new crate is placed on purpose, not by default.
"""
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

LAYERS = {
    'stubby_audio': 'core',
    'stubby_rack': 'core',
    'stubby_recipes': 'core',
    'stubby_core': 'service',
    'stubby_ui': 'service',
    'bevy_stubby_audio': 'service',
    'bevy_stubby_rack': 'system',
    'bevy_stubby_recipes': 'system',
    'stubby_player': 'system',
    'stubby_life': 'system',
    'stubby_world': 'system',
    'stubby_game': 'game',
    'stubby_jackdaw': 'game',
}
# A core that belongs to one system; other systems may not use it.
OWNER = {'stubby_rack': 'bevy_stubby_rack', 'stubby_recipes': 'bevy_stubby_recipes'}
# What each layer may depend on.
ALLOWED = {
    'core': {'core'},
    'service': {'core'},
    'system': {'core', 'service'},
    'game': {'core', 'service', 'system', 'game'},
}
BEVY_ENGINE = re.compile(r'^bevy( |_app |_ecs |_render |_asset )')


def workspace_deps(metadata):
    """Crate -> workspace crates it depends on (normal and build dependencies)."""
    members = {p['name'] for p in metadata['packages']}
    return {
        p['name']: sorted({d['name'] for d in p['dependencies']
                           if d['name'] in members and d['kind'] in (None, 'build')})
        for p in metadata['packages']
    }


def layer_errors(deps):
    errors = []
    for crate, used in sorted(deps.items()):
        layer = LAYERS.get(crate)
        if layer is None:
            errors.append(f'{crate}: not placed in a layer; add it to LAYERS in tools/check_layers.py')
            continue
        for dep in used:
            dep_layer = LAYERS.get(dep)
            if dep_layer is None:
                continue  # reported on its own line
            if dep_layer not in ALLOWED[layer]:
                errors.append(f'{crate} ({layer}) may not depend on {dep} ({dep_layer})')
            elif dep in OWNER and OWNER[dep] != crate and layer != 'game':
                errors.append(f'{crate} may not depend on {dep}, the core of {OWNER[dep]}')
    return errors


def bevy_in_cores():
    errors = []
    for crate, layer in sorted(LAYERS.items()):
        if layer != 'core':
            continue
        tree = subprocess.run(
            ['cargo', 'tree', '-q', '-p', crate, '-e', 'normal', '--prefix', 'none', '--locked'],
            cwd=ROOT, capture_output=True, text=True, check=True).stdout
        found = sorted({line.split()[0] for line in tree.splitlines() if BEVY_ENGINE.match(line)})
        if found:
            errors.append(f'{crate} (core) depends on the Bevy engine: {", ".join(found)}')
    return errors


def main():
    metadata = json.loads(subprocess.run(
        ['cargo', 'metadata', '--format-version', '1', '--no-deps', '--locked'],
        cwd=ROOT, capture_output=True, text=True, check=True).stdout)
    errors = layer_errors(workspace_deps(metadata)) + bevy_in_cores()
    for error in errors:
        print(f'ERROR: {error}')
    if errors:
        print('See the system-architecture spec (openspec/changes/split-game-into-systems).')
        return 1
    print('Layer check passed: cores, services, systems and game depend only downward.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
