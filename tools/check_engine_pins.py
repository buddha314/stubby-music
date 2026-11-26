#!/usr/bin/env python3
"""Read-only check that the game's engine stack is one consistent, pinned set.

The set is recorded in compatibility.json: our Jackdaw fork's revision, the Bevy
it locks, and its Rust toolchain. This checks that the manifests, Cargo.lock,
jackdaw.toml and rust-toolchain.toml all agree with it, and (with --editor) that
the shared editor checkout is that revision. Nothing here edits a file.

    tools/check_engine_pins.py               the repository alone (CI runs this)
    tools/check_engine_pins.py --editor      also the shared ../jackdaw checkout
    tools/check_engine_pins.py --binaries    also the editor build receipt
    tools/check_engine_pins.py --tree FILE   also a `cargo tree` dump (one Bevy)

Generic: the crates checked are derived from the workspace and jackdaw.toml, so a
game made from this template needs no edits here. Shared by every game: change it in
the template (VerdantAI/us-outpost-31) and copy it out with tools/update_engine_tools.sh.
See docs/ENGINE-UPDATES.md.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tomllib

GUIDE = 'docs/ENGINE-UPDATES.md'


def read_toml(path):
    with path.open('rb') as stream:
        return tomllib.load(stream)


def git(source, *args):
    return subprocess.check_output(
        ['git', '-C', str(source), *args], text=True, stderr=subprocess.PIPE
    ).strip()


def bevy_sources(lock):
    return {(p['version'], p.get('source', '')) for p in lock['package'] if p['name'] == 'bevy'}


def jackdaw_lock_source(selected):
    return f"git+{selected['origin']}?rev={selected['revision']}#{selected['revision']}"


def members(root, workspace):
    """Each workspace member's relative path and manifest."""
    found = []
    for pattern in workspace['members']:
        for path in sorted(root.glob(pattern)):
            if (path / 'Cargo.toml').is_file():
                found.append((path.relative_to(root).as_posix(), read_toml(path / 'Cargo.toml')))
    return found


def dependency_tables(manifest):
    for section in ('dependencies', 'dev-dependencies', 'build-dependencies'):
        yield section, manifest.get(section, {})
    for target in manifest.get('target', {}).values():
        for section in ('dependencies', 'dev-dependencies', 'build-dependencies'):
            yield section, target.get(section, {})


def validate_engine_dependencies(root, workspace, selected):
    """Engine crates are declared once, in the workspace, and every member inherits them."""
    errors = []
    url, rev = selected['origin'], selected['revision']
    for name, dep in workspace.get('dependencies', {}).items():
        if not name.startswith('jackdaw_'):
            continue
        if not isinstance(dep, dict) or dep.get('git') != url or dep.get('rev') != rev \
                or any(key in dep for key in ('path', 'branch', 'tag', 'version')):
            errors.append(f'workspace {name} must be {{ git = "{url}", rev = "{rev}" }}, found {dep}')
    engine = {'bevy', 'avian3d', 'avian2d'} | {n for n in workspace.get('dependencies', {})
                                               if n.startswith('jackdaw_')}
    for member, manifest in members(root, workspace):
        if 'workspace' in manifest:
            errors.append(f'{member} must not define a separate workspace')
        if (root / member / 'Cargo.lock').exists():
            errors.append(f'remove {member}/Cargo.lock; the workspace has one Cargo.lock')
        for section, deps in dependency_tables(manifest):
            for name, dep in deps.items():
                if isinstance(dep, dict) and dep.get('workspace') is True:
                    continue
                if name in engine or name.startswith('jackdaw_'):
                    pass  # engine crates are always inherited
                elif name.startswith('bevy_') and isinstance(dep, dict) \
                        and 'bevyengine/bevy' in dep.get('git', ''):
                    pass  # a Bevy engine crate from git is a second Bevy source
                else:
                    # The game's own crates (`bevy_mygame_audio = { path = ... }`) and
                    # crates.io ecosystem crates; the lockfile check catches a second Bevy.
                    continue
                errors.append(f'{member}: {section}.{name} must inherit the pinned engine '
                              f'({name}.workspace = true)')
    expected = jackdaw_lock_source(selected)
    linked = []
    for package in read_toml(root / 'Cargo.lock')['package']:
        if not package['name'].startswith('jackdaw_'):
            continue
        source = package.get('source')
        if source is None:
            linked.append(package['name'])
        elif source != expected:
            errors.append(f"Cargo.lock resolves {package['name']} at {source.rsplit('#', 1)[-1]}, "
                          f'but compatibility.json records {rev}')
    if linked:
        errors.append(f"Cargo.lock records linked Jackdaw paths ({', '.join(sorted(linked))}); "
                      'run tools/jackdaw.sh unlink before committing')
    return errors


def validate_forks(root):
    """Every [patch.crates-io] entry is a fork from the manifest, at its branch. A game
    patches only the forks its graph uses, so missing entries are fine."""
    config = json.loads((root / 'bevy_main_forks.json').read_text())
    patch = read_toml(root / 'Cargo.toml').get('patch', {}).get('crates-io', {})
    errors = []
    for fork in config['forks']:
        want = {'git': fork['fork'], 'branch': config['branch']}
        if fork['crate'] in patch and patch[fork['crate']] != want:
            errors.append(f"[patch.crates-io] {fork['crate']} must be {want} (bevy_main_forks.json); "
                          f"found {patch.get(fork['crate'])}")
    extra = set(patch) - {f['crate'] for f in config['forks']}
    if extra:
        errors.append(f"[patch.crates-io] patches crates not in bevy_main_forks.json: {sorted(extra)}")
    return errors


def validate(root):
    """The repository alone: manifests, lockfile and pins agree with compatibility.json."""
    errors = []
    record = json.loads((root / 'compatibility.json').read_text())
    if record.get('schema_version') != 1:
        return ['unsupported compatibility record schema']
    selected, bevy = record['jackdaw'], record['bevy']
    workspace = read_toml(root / 'Cargo.toml')['workspace']
    dep = workspace['dependencies']['bevy']
    if bevy['source'].startswith('git+'):
        url, branch = bevy['source'].removeprefix('git+').split('#')[0].split('?branch=')
        good = isinstance(dep, dict) and dep.get('git') == url and dep.get('branch') == branch \
            and 'version' not in dep and 'path' not in dep
    else:
        good = isinstance(dep, dict) and dep.get('version') == '=' + bevy['version'] \
            and 'git' not in dep and 'path' not in dep
    if not good:
        errors.append('workspace Bevy source/version differs from the pinned Jackdaw')
    errors += validate_engine_dependencies(root, workspace, selected)
    errors += validate_forks(root)
    if bevy_sources(read_toml(root / 'Cargo.lock')) != {(bevy['version'], bevy['source'])}:
        errors.append(f"Cargo.lock must resolve one Bevy engine source: {bevy['source']}")
    project = read_toml(root / 'jackdaw.toml')
    rev = bevy['source'].rsplit('#', 1)[1] if bevy['source'].startswith('git+') else None
    pins = project['jackdaw']
    if (pins['version'], pins.get('bevy_rev'), pins['bevy']) != (selected['version'], rev, bevy['line']):
        errors.append('jackdaw.toml [jackdaw] differs from the pinned Jackdaw/Bevy versions')
    package = project.get('package')
    if package not in {m.get('package', {}).get('name') for _, m in members(root, workspace)}:
        errors.append(f'jackdaw.toml package {package!r} is not a workspace member')
    lock = read_toml(root / 'Cargo.lock')['package']
    for crate in ('avian3d', 'avian2d'):
        sources = sorted({p.get('source', 'path') for p in lock if p['name'] == crate})
        if len(sources) > 1:
            errors.append(f'Cargo.lock resolves {crate} from {len(sources)} sources {sources}; one avian only. '
                          "Jackdaw's Physics module brings jbuehler23's fork: keep it off, or patch "
                          'that fork to ours (docs/ENGINE-UPDATES.md)')
    if read_toml(root / 'rust-toolchain.toml')['toolchain']['channel'] != record['rust']:
        errors.append(f"project compiler must match Jackdaw: Rust {record['rust']}")
    return errors


def validate_editor(root, source):
    """The shared editor checkout is the pinned revision, clean, with the expected remotes."""
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
    """`cargo tree` resolves exactly one Bevy, the pinned one."""
    bevy = record['bevy']
    errors, found = [], False
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
    parser = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--source', type=Path,
                        help='shared Jackdaw checkout (default: $JACKDAW_SOURCE or ../jackdaw)')
    parser.add_argument('--tree', type=Path,
                        help='also check `cargo tree --workspace -e normal,build,dev --prefix none --format {p} --locked`')
    parser.add_argument('--editor', action='store_true', help='also check the shared editor checkout')
    parser.add_argument('--binaries', action='store_true',
                        help='also check the editor build receipt (implies --editor)')
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
            errors += validate_tree(args.tree.read_text(), json.loads((root / 'compatibility.json').read_text()))
    except (OSError, KeyError, ValueError, TypeError) as error:
        errors = [f'invalid or missing engine configuration: {error}']
    if errors:
        for error in errors:
            print(f'ERROR: {error}')
        print(f'Use a tested engine set. To move it, follow {GUIDE} (`tools/jackdaw.sh sync`); '
              'never update Bevy, avian or Jackdaw independently. A missing build receipt: '
              'run tools/jackdaw.sh prepare.')
        return 1
    print('Engine pins agree: Jackdaw, Bevy, avian forks, compiler and lockfile are one set.')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
