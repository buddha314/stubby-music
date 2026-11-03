#!/usr/bin/env python3
"""Reconcile the chain Bevy main -> jbuehler23/jackdaw main -> our fork -> this game.

1. Merge upstream (jbuehler23) main into our fork's main: in a temporary worktree,
   pushed to buddha314 as sync/<date>, merged by PR. Stops on a conflict.
2. Fast-forward the shared Jackdaw checkout to the fork's main.
3. Rebuild our Bevy-main forks of third-party crates (tools/bevy_main_forks.py).
4. Re-pin the game: compatibility.json, jackdaw/jackdaw.toml pins, rust-toolchain.toml,
   and Cargo.lock's Bevy revision to the one our Jackdaw locks.
5. Verify (tools/check_jackdaw_versions.py) and build the game and adapter.

tools/jackdaw.sh sync runs this and then `prepare` (the shared editor build).
Never pushes to jbuehler23: the upstream remote is fetch-only.
"""
import datetime
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]
SOURCE = Path(os.environ.get('JACKDAW_SOURCE', ROOT.parent / 'jackdaw'))
RECORD = ROOT / 'jackdaw/compatibility.json'


def run(*args, cwd=ROOT, capture=False, check=True):
    result = subprocess.run(args, cwd=cwd, text=True, check=check,
                            stdout=subprocess.PIPE if capture else None)
    return result.stdout.strip() if capture else result.returncode


def git(*args, capture=True, check=True):
    return run('git', '-C', str(SOURCE), *args, capture=capture, check=check)


def is_ancestor(a, b):
    return git('merge-base', '--is-ancestor', a, b, capture=False, check=False) == 0


def reconcile_fork(record):
    branch, upstream_branch = record['jackdaw']['branch'], record['jackdaw']['upstream_branch']
    git('fetch', '-q', 'origin', capture=False)
    git('fetch', '-q', 'upstream', capture=False)
    if is_ancestor(f'upstream/{upstream_branch}', f'origin/{branch}'):
        print(f'Fork {branch} already contains upstream/{upstream_branch}.')
        return
    date = datetime.date.today().isoformat()
    sync_branch = f'sync/{date}'
    with tempfile.TemporaryDirectory(prefix='jackdaw-sync-') as temp:
        work = Path(temp) / 'jackdaw'
        git('worktree', 'add', '-q', '-B', sync_branch, str(work), f'origin/{branch}', capture=False)
        try:
            if run('git', 'merge', '--no-edit', '-q', f'upstream/{upstream_branch}', cwd=work, check=False):
                run('git', 'merge', '--abort', cwd=work, check=False)
                sys.exit(f'Merging upstream/{upstream_branch} into the fork conflicts; integrate by hand '
                         f'(policy: docs/research/jackdaw-version-policy.md).')
            run('git', 'push', '-q', '-u', 'origin', sync_branch, cwd=work)
        finally:
            git('worktree', 'remove', '--force', str(work), capture=False, check=False)
    repo = git('remote', 'get-url', 'origin').removeprefix('https://github.com/').removesuffix('.git')
    if not repo.startswith('buddha314/'):
        sys.exit(f'Refusing to open a PR on {repo}; the fork must be buddha314.')
    count = git('rev-list', '--count', f'origin/{branch}..upstream/{upstream_branch}')
    run('gh', 'pr', 'create', '--repo', repo, '--base', branch, '--head', sync_branch,
        '--title', f'Sync upstream {upstream_branch} ({date})',
        '--body', f'Merges {count} upstream commits from jbuehler23/jackdaw {upstream_branch}. '
                  'Opened by stubby-music tools/jackdaw_sync.py.')
    run('gh', 'pr', 'merge', sync_branch, '--repo', repo, '--merge')
    git('fetch', '-q', 'origin', capture=False)
    print(f'Fork {branch} merged {count} upstream commits.')


def advance_checkout(record):
    branch = record['jackdaw']['branch']
    if git('branch', '--show-current') != branch:
        sys.exit(f'Shared Jackdaw ({SOURCE}) is not on {branch}; switch it while the editor is closed.')
    if git('status', '--porcelain', '--untracked-files=no'):
        sys.exit('Shared Jackdaw has tracked changes; it must be clean to advance.')
    if not is_ancestor('HEAD', f'origin/{branch}'):
        sys.exit(f'Shared Jackdaw HEAD is not contained in origin/{branch}; integrate by hand.')
    git('merge', '-q', '--ff-only', f'origin/{branch}', capture=False)
    return git('rev-parse', 'HEAD')


def repin(record, revision):
    lock = tomllib.loads((SOURCE / 'Cargo.lock').read_text())
    bevy = next(p for p in lock['package'] if p['name'] == 'bevy')
    workspace = tomllib.loads((SOURCE / 'Cargo.toml').read_text())['workspace']['package']
    rust = tomllib.loads((SOURCE / 'rust-toolchain.toml').read_text())['toolchain']['channel']
    source = bevy['source']
    git_bevy = source.startswith('git+')
    rev = source.rsplit('#', 1)[1] if git_bevy else None
    jackdaw = record['jackdaw']
    jackdaw.update(revision=revision, version=workspace['version'],
                   upstream_revision=git('merge-base', revision, f"upstream/{jackdaw['upstream_branch']}"))
    record['bevy'] = {'version': bevy['version'], 'source': source,
                      'line': 'main' if git_bevy else '.'.join(bevy['version'].split('.')[:2])}
    record['rust'] = rust
    RECORD.write_text(json.dumps(record, indent=2) + '\n')

    pins = f'version = "{jackdaw["version"]}"\nbevy = "{record["bevy"]["line"]}"\n'
    pins += f'channel = "bleeding-edge"\nbevy_rev = "{rev}"\n' if git_bevy else 'channel = "stable"\n'
    for manifest in (ROOT / 'jackdaw/jackdaw.toml',):
        text = manifest.read_text()
        text = re.sub(r'(?m)^\[jackdaw\]\n(?:(?:version|bevy|bevy_rev|channel|build) = .*\n)*',
                      lambda _: '[jackdaw]\n' + pins, text, count=1)
        manifest.write_text(text)
    toolchain = ROOT / 'rust-toolchain.toml'
    toolchain.write_text(re.sub(r'(?m)^channel = ".*"$', f'channel = "{rust}"', toolchain.read_text()))

    # Refresh the lock for moved path crates, then pin Bevy and the forks.
    run('cargo', 'metadata', '--format-version', '1', capture=True)
    # Forks first: updating them re-resolves Bevy's git branch, so Bevy is pinned last.
    forks = json.loads((ROOT / 'jackdaw/bevy_main_forks.json').read_text())['forks']
    run('cargo', 'update', '-q', *[a for f in forks for a in ('-p', f['crate'])])
    if git_bevy:
        url = source.removeprefix('git+').split('#')[0]
        run('cargo', 'update', '-q', '-p', f'git+{url}#bevy@{bevy["version"]}', '--precise', rev)
    return record


def main():
    record = json.loads(RECORD.read_text())
    reconcile_fork(record)
    before = git('rev-parse', 'HEAD')
    revision = advance_checkout(record)
    run(sys.executable, str(ROOT / 'tools/bevy_main_forks.py'), 'ensure')
    repin(record, revision)
    run(sys.executable, str(ROOT / 'tools/check_jackdaw_versions.py'), '--source', str(SOURCE))
    # Separately: built together, Cargo unifies features and the adapter build that
    # embedded Play runs would start cold.
    if any(run('cargo', 'build', '--locked', '-p', package, check=False)
           for package in ('stubby_game', 'stubby_jackdaw')):
        sys.exit('The game does not build on the new Jackdaw/Bevy; fix the game (or a fork) and rerun. '
                 f'Shared Jackdaw moved {before[:8]} -> {revision[:8]}; '
                 f'roll back with: git -C {SOURCE} reset --keep {before} && '
                 f'git -C {ROOT} checkout -- jackdaw/jackdaw.toml jackdaw/compatibility.json '
                 'rust-toolchain.toml Cargo.lock')
    print(f'Synced: Jackdaw {revision[:8]}, Bevy {record["bevy"]["source"].rsplit("#", 1)[-1][:8]}. '
          'Commit the pins and Cargo.lock on a branch.')


if __name__ == '__main__':
    main()
