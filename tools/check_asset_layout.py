#!/usr/bin/env python3
"""Resolve runtime model/prefab references after asset moves (no editor needed)."""
from pathlib import Path
import json
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
REFERENCE = re.compile(r"^[\w./-]+\.(?:glb|bsn)(?:#Scene\d+)?$")


def references(value):
    if isinstance(value, str) and REFERENCE.fullmatch(value):
        yield value
    elif isinstance(value, dict):
        for key, child in value.items():
            yield from references(key)
            yield from references(child)
    elif isinstance(value, list):
        for child in value:
            yield from references(child)


def check(assets):
    errors = []
    for file in sorted(assets.rglob('*')):
        if not file.is_file():
            continue
        if file.suffix == '.json':
            refs = references(json.loads(file.read_text()))
        elif file.suffix == '.bsn':
            refs = re.findall(r'(?:path|source):\s*"([^"\n]+\.(?:glb|bsn))"', file.read_text())
        else:
            continue
        for ref in refs:
            path = ref.split('#', 1)[0]
            target = assets / path
            # Jackdaw prepare creates this ignored procedural floor, not an authored model.
            if path == "generated/mudflat.glb":
                continue
            if not target.is_file():
                errors.append(f'{file.relative_to(assets)}: missing {ref}')
    for old in ('models', 'prefabs'):
        if (assets / old).exists():
            errors.append(f'{old}/: split asset directory must not return')
    for model in sorted(assets.rglob('*.glb')):
        # The same ignored, generated floor as above.
        if model.relative_to(assets).as_posix() == "generated/mudflat.glb":
            continue
        if model.parent.name != model.stem:
            errors.append(f'{model.relative_to(assets)}: model must live in its asset folder')
    return errors


def main():
    errors = check(ROOT / 'assets')
    if errors:
        print('\n'.join(errors), file=sys.stderr)
        return 1
    print('Asset layout passed: profile, collider and prefab references resolve.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
