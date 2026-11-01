#!/usr/bin/env python3
"""Read-only validation of the compatibility set selected by shared Jackdaw."""
import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import subprocess
import tomllib


def read_toml(path):
    with path.open('rb') as stream:
        return tomllib.load(stream)


def git(source, *args):
    return subprocess.check_output(
        ['git', '-C', str(source), *args], text=True, stderr=subprocess.PIPE
    ).strip()


def bevy_sources(lock):
    # Cargo also locks Firewheel's disabled optional Bevy 0.19 integrations.
    # Those are not part of the resolved graph; validate that graph separately.
    return {(p['version'], p.get('source', '')) for p in lock['package'] if p['name'] == 'bevy'}


def validate(root, source):
    errors = []
    record = json.loads((root / 'jackdaw/compatibility.json').read_text())
    if record.get('schema_version') != 1:
        return ['unsupported compatibility record schema']
    selected = record['jackdaw']
    bevy = record['bevy']
    expected = (bevy['version'], bevy['source'])
    workspace = read_toml(root / 'Cargo.toml')['workspace']
    dep = workspace['dependencies']['bevy']
    if not isinstance(dep, dict) or dep.get('version') != '=' + bevy['version'] or 'git' in dep or 'path' in dep:
        errors.append('workspace Bevy source/version differs from the selected Jackdaw release')
    members = workspace['members']
    for member in ('crates/stubby_game', 'crates/bevy_stubby_recipes', 'jackdaw'):
        manifest = read_toml(root / member / 'Cargo.toml')
        member_bevy = manifest['dependencies']['bevy']
        if member not in members or not isinstance(member_bevy, dict) or member_bevy.get('workspace') is not True:
            errors.append(f'{member} must inherit Bevy from the unified workspace')
        if 'workspace' in manifest:
            errors.append(f'{member} must not define a separate workspace')
    adapter = read_toml(root / 'jackdaw/Cargo.toml')
    for section in ('dependencies', 'dev-dependencies'):
        for name, dep in adapter.get(section, {}).items():
            if not name.startswith('jackdaw_'):
                continue
            if not isinstance(dep, dict) or 'path' not in dep:
                errors.append(f'{name} must use the selected shared source path')
                continue
            actual = (root / 'jackdaw' / dep['path']).resolve()
            if actual != (source / 'crates' / name).resolve():
                errors.append(f'{name} path resolves to {actual}, not the selected shared checkout')
    if (root / 'jackdaw/Cargo.lock').exists():
        errors.append('remove the separate adapter lockfile; the workspace has one Cargo.lock')
    if bevy_sources(read_toml(root / 'Cargo.lock')) != {expected}:
        errors.append(f'Cargo.lock must resolve one Bevy engine source: {expected}')
    project = read_toml(root / 'jackdaw/jackdaw.toml')['jackdaw']
    if (project['version'], project.get('bevy_rev'), project['bevy']) != (selected['version'], None, bevy['line']):
        errors.append('jackdaw.toml differs from the selected Jackdaw/Bevy versions')
    if read_toml(root / 'rust-toolchain.toml')['toolchain']['channel'] != record['rust']:
        errors.append(f"project compiler must match Jackdaw: Rust {record['rust']}")
    try:
        actual = git(source, 'rev-parse', 'HEAD')
        if actual != selected['revision']:
            errors.append(f"Jackdaw checkout drift: expected {selected['revision']}, found {actual}")
        if git(source, 'status', '--porcelain', '--untracked-files=no'):
            errors.append('shared Jackdaw has tracked changes; validate a clean revision')
        for remote in ('origin', 'upstream'):
            actual = git(source, 'remote', 'get-url', remote).removesuffix('.git')
            if actual != selected[remote].removesuffix('.git'):
                errors.append(f'Jackdaw {remote} must be {selected[remote]}, found {actual}')
        if read_toml(source / 'rust-toolchain.toml')['toolchain']['channel'] != record['rust']:
            errors.append('shared Jackdaw compiler differs from the compatibility record')
        if bevy_sources(read_toml(source / 'Cargo.lock')) != {expected}:
            errors.append('shared Jackdaw Cargo.lock differs from the selected Bevy revision')
    except (subprocess.CalledProcessError, FileNotFoundError) as error:
        errors.append(f'cannot inspect shared Jackdaw at {source}: {error}')
    return errors


def validate_tree(tree, record):
    bevy = record['bevy']
    errors = []
    found = False
    for line in tree.splitlines():
        match = re.match(r'^(bevy|bevy_app|bevy_ecs|bevy_math|bevy_reflect|bevy_transform) v([^ ]+)(.*)$', line)
        if not match:
            continue
        found = True
        suffix = match[3].strip()
        if match[2] != bevy['version'] or suffix not in ('', '(*)'):
            errors.append(f'active Bevy dependency differs from Jackdaw: {line}')
    if not found:
        errors.append('Cargo tree contains no Bevy engine dependencies')
    return errors


def validate_binaries(root, source):
    record = json.loads((root / 'jackdaw/compatibility.json').read_text())
    receipt = json.loads((root / 'jackdaw/.jackdaw/editor-build.json').read_text())
    errors = []
    if receipt['revision'] != record['jackdaw']['revision']:
        errors.append('shared editor build receipt is for another source revision')
    for name in ('jackdaw', 'jd'):
        with (source / 'target/debug' / name).open('rb') as stream:
            actual = hashlib.file_digest(stream, 'sha256').hexdigest()
        if receipt['sha256'][name] != actual:
            errors.append(f'{name} binary differs from the verified local build; run tools/jackdaw.sh prepare')
    return errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--source', type=Path, help='shared checkout (defaults to JACKDAW_SOURCE or sibling jackdaw)')
    parser.add_argument('--tree', type=Path, help='also check cargo tree --workspace -e normal,build,dev --prefix none --format {p} --locked output')
    parser.add_argument('--binaries', action='store_true', help='also check the source-linked local build receipt')
    args = parser.parse_args()
    root = args.root.resolve()
    source = (args.source or Path(os.environ.get('JACKDAW_SOURCE', root.parent / 'jackdaw'))).resolve()
    try:
        errors = validate(root, source)
        if args.binaries:
            errors += validate_binaries(root, source)
        if args.tree:
            errors += validate_tree(args.tree.read_text(), json.loads((root / "jackdaw/compatibility.json").read_text()))
    except (OSError, KeyError, ValueError, TypeError) as error:
        errors = [f'invalid or missing compatibility configuration: {error}']
    if errors:
        for error in errors:
            print(f'ERROR: {error}')
        print('Use a tested Jackdaw revision. For an intentional upgrade, follow docs/research/jackdaw-version-policy.md; do not update Bevy independently. Run tools/jackdaw.sh prepare for a missing build receipt.')
        return 1
    print('Jackdaw compatibility check passed: source, compiler, manifests and unified Bevy lock agree.')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
