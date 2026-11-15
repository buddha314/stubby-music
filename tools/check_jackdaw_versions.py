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


def jackdaw_lock_source(selected):
    return f"git+{selected['origin']}?rev={selected['revision']}#{selected['revision']}"


def validate_jackdaw_dependencies(root, workspace, selected):
    """Jackdaw crates come from the fork at the recorded revision, declared once."""
    errors = []
    url, rev = selected['origin'], selected['revision']
    for name, dep in workspace.get('dependencies', {}).items():
        if not name.startswith('jackdaw_'):
            continue
        if not isinstance(dep, dict) or dep.get('git') != url or dep.get('rev') != rev \
                or any(key in dep for key in ('path', 'branch', 'tag', 'version')):
            errors.append(f'workspace {name} must be {{ git = "{url}", rev = "{rev}" }}, found {dep}')
    for member in workspace['members']:
        manifest = read_toml(root / member / 'Cargo.toml')
        for section in ('dependencies', 'dev-dependencies', 'build-dependencies'):
            for name, dep in manifest.get(section, {}).items():
                if name.startswith('jackdaw_') and not (isinstance(dep, dict) and dep.get('workspace') is True):
                    errors.append(f'{member}: {name} must inherit the pinned Jackdaw ({name}.workspace = true)')
    expected = jackdaw_lock_source(selected)
    linked = []
    for package in read_toml(root / 'Cargo.lock')['package']:
        if not package['name'].startswith('jackdaw_'):
            continue
        source = package.get('source')
        if source is None:
            linked.append(package['name'])
        elif source != expected:
            locked = source.rsplit('#', 1)[-1]
            errors.append(f"Cargo.lock resolves {package['name']} at {locked}, but compatibility.json records {rev}")
    if linked:
        errors.append(f"Cargo.lock records linked Jackdaw paths ({', '.join(sorted(linked))}); "
                      'run tools/jackdaw.sh unlink before committing')
    return errors


def validate(root):
    """The repository alone: manifests, lockfile and pins agree with compatibility.json."""
    errors = []
    record = json.loads((root / 'compatibility.json').read_text())
    if record.get('schema_version') != 1:
        return ['unsupported compatibility record schema']
    selected = record['jackdaw']
    bevy = record['bevy']
    expected = (bevy['version'], bevy['source'])
    workspace = read_toml(root / 'Cargo.toml')['workspace']
    dep = workspace['dependencies']['bevy']
    if bevy['source'].startswith('git+'):
        # Bevy main: the workspace names the git branch; Cargo.lock pins the revision.
        url, branch = bevy['source'].removeprefix('git+').split('#')[0].split('?branch=')
        good = isinstance(dep, dict) and dep.get('git') == url and dep.get('branch') == branch \
            and 'version' not in dep and 'path' not in dep
    else:
        good = isinstance(dep, dict) and dep.get('version') == '=' + bevy['version'] \
            and 'git' not in dep and 'path' not in dep
    if not good:
        errors.append('workspace Bevy source/version differs from the selected Jackdaw release')
    members = workspace['members']
    for member in ('crates/stubby_game', 'crates/bevy_stubby_recipes', 'crates/stubby_jackdaw'):
        manifest = read_toml(root / member / 'Cargo.toml')
        member_bevy = manifest['dependencies']['bevy']
        if member not in members or not isinstance(member_bevy, dict) or member_bevy.get('workspace') is not True:
            errors.append(f'{member} must inherit Bevy from the unified workspace')
        if 'workspace' in manifest:
            errors.append(f'{member} must not define a separate workspace')
    errors += validate_jackdaw_dependencies(root, workspace, selected)
    if (root / 'crates/stubby_jackdaw/Cargo.lock').exists():
        errors.append('remove the separate adapter lockfile; the workspace has one Cargo.lock')
    if bevy_sources(read_toml(root / 'Cargo.lock')) != {expected}:
        errors.append(f'Cargo.lock must resolve one Bevy engine source: {expected}')
    rev = bevy['source'].rsplit('#', 1)[1] if bevy['source'].startswith('git+') else None
    for manifest in ('jackdaw.toml',):
        project = read_toml(root / manifest)['jackdaw']
        if (project['version'], project.get('bevy_rev'), project['bevy']) != (selected['version'], rev, bevy['line']):
            errors.append(f'{manifest} differs from the selected Jackdaw/Bevy versions')
    if read_toml(root / 'rust-toolchain.toml')['toolchain']['channel'] != record['rust']:
        errors.append(f"project compiler must match Jackdaw: Rust {record['rust']}")
    return errors


def validate_editor(root, source):
    """The shared editor checkout matches the pin; Play speaks Jackdaw's PIE protocol to the game."""
    errors = []
    record = json.loads((root / 'compatibility.json').read_text())
    selected = record['jackdaw']
    expected = (record['bevy']['version'], record['bevy']['source'])
    try:
        actual = git(source, 'rev-parse', 'HEAD')
        if actual != selected['revision']:
            errors.append(f"Jackdaw checkout drift: expected {selected['revision']}, found {actual}; "
                          'run tools/jackdaw.sh sync, or check out the recorded revision')
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
        line = re.sub(r'\x1b\[[0-9;]*m', '', line).strip()
        match = re.match(r'^(bevy|bevy_app|bevy_ecs|bevy_math|bevy_reflect|bevy_transform) v([^ ]+)(.*)$', line)
        if not match:
            continue
        found = True
        suffix = match[3].strip().removesuffix('(*)').strip()
        if bevy['source'].startswith('git+'):
            url, rev = bevy['source'].removeprefix('git+').split('#')
            source_ok = suffix == f'({url}#{rev[:8]})'
        else:
            source_ok = suffix == ''
        if match[2] != bevy['version'] or not source_ok:
            errors.append(f'active Bevy dependency differs from Jackdaw: {line}')
    if not found:
        errors.append('Cargo tree contains no Bevy engine dependencies')
    return errors


def validate_binaries(root, source):
    record = json.loads((root / 'compatibility.json').read_text())
    receipt = json.loads((root / '.jackdaw/editor-build.json').read_text())
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
    parser.add_argument('--editor', action='store_true', help='also check the shared editor checkout matches the pin')
    parser.add_argument('--binaries', action='store_true', help='also check the source-linked local build receipt (implies --editor)')
    args = parser.parse_args()
    root = args.root.resolve()
    source = (args.source or Path(os.environ.get('JACKDAW_SOURCE', root.parent / 'jackdaw'))).resolve()
    try:
        errors = validate(root)
        if args.editor or args.binaries:
            errors += validate_editor(root, source)
        if args.binaries:
            errors += validate_binaries(root, source)
        if args.tree:
            errors += validate_tree(args.tree.read_text(), json.loads((root / "compatibility.json").read_text()))
    except (OSError, KeyError, ValueError, TypeError) as error:
        errors = [f'invalid or missing compatibility configuration: {error}']
    if errors:
        for error in errors:
            print(f'ERROR: {error}')
        print('Use a tested Jackdaw revision. For an intentional upgrade, follow docs/research/jackdaw-version-policy.md; do not update Bevy independently. Run tools/jackdaw.sh prepare for a missing build receipt.')
        return 1
    print('Jackdaw compatibility check passed: pinned Jackdaw, compiler, manifests and unified Bevy lock agree.')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
