#!/usr/bin/env python3
"""Move the game to the newest engine set: Bevy main -> avian forks -> our Jackdaw -> here.

1. Upstream Jackdaw (jbuehler23) main, merged into our fork's main in a temporary
   worktree, pushed to buddha314 as sync/<date> and merged by PR. Stops on a conflict.
   **Avian first:** if upstream moved Bevy, that commit is probed against this game's
   avian forks first (the Jackdaw checkout's scripts/bevy_ceiling.py), and the sync
   stops if they do not build there. A Bevy the forks cannot follow is not usable.
2. Fast-forward the shared Jackdaw checkout to the fork's main.
3. Rebuild our Bevy-main forks of third-party crates (tools/bevy_main_forks.py).
4. Re-pin the game: compatibility.json, jackdaw.toml pins, rust-toolchain.toml, the
   Jackdaw git `rev` in Cargo.toml, and Cargo.lock's Jackdaw, fork and Bevy revisions.
5. Verify (tools/check_engine_pins.py) and build: the jackdaw.toml package, then
   every workspace target.

`tools/jackdaw.sh sync` runs this and then `prepare` (the shared editor build). It
never pushes to jbuehler23: the upstream remote is fetch-only. Nothing here is
specific to one game; it reads the package and crates from jackdaw.toml and the
workspace. See docs/ENGINE-UPDATES.md.
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
RECORD = ROOT / 'compatibility.json'
FORKS = ROOT / 'bevy_main_forks.json'
GUIDE = 'docs/ENGINE-UPDATES.md'


def run(*args, cwd=ROOT, capture=False, check=True):
    result = subprocess.run(args, cwd=cwd, text=True, check=check,
                            stdout=subprocess.PIPE if capture else None)
    return result.stdout.strip() if capture else result.returncode


def git(*args, capture=True, check=True):
    return run('git', '-C', str(SOURCE), *args, capture=capture, check=check)


def is_ancestor(a, b):
    return git('merge-base', '--is-ancestor', a, b, capture=False, check=False) == 0


def locked_bevy(ref):
    """The Bevy revision a Jackdaw commit's root Cargo.lock pins (None for crates.io Bevy)."""
    lock = tomllib.loads(git('show', f'{ref}:Cargo.lock'))
    source = next(p['source'] for p in lock['package'] if p['name'] == 'bevy')
    return source.rsplit('#', 1)[1] if source.startswith('git+') else None


def ceiling_allows(rev):
    """Do this game's avian forks build against Bevy `rev`? (bevy_ceiling.py probe)"""
    script = SOURCE / 'scripts/bevy_ceiling.py'
    if not script.is_file():
        print(f'warning: {script} is missing; cannot check the avian ceiling for {rev[:9]}')
        return True
    return run(sys.executable, str(script), 'probe', rev, '--forks', str(FORKS),
               cwd=SOURCE, check=False) == 0


def reconcile_fork(record):
    branch, upstream_branch = record['jackdaw']['branch'], record['jackdaw']['upstream_branch']
    git('fetch', '-q', 'origin', capture=False)
    git('fetch', '-q', 'upstream', capture=False)
    if is_ancestor(f'upstream/{upstream_branch}', f'origin/{branch}'):
        print(f'Fork {branch} already contains upstream/{upstream_branch}.')
        return
    ours, theirs = locked_bevy(f'origin/{branch}'), locked_bevy(f'upstream/{upstream_branch}')
    if theirs and theirs != ours:
        print(f'Upstream moved Bevy {str(ours)[:9]} -> {theirs[:9]}; probing the avian forks there first.')
        if not ceiling_allows(theirs):
            sys.exit(f'Upstream Jackdaw pins Bevy {theirs[:9]}, which our avian forks do not build '
                     f'against (past the avian ceiling). Not merging. Raise the ceiling first '
                     f'(Jackdaw docs/upgrading-bevy.md, step 3), or wait for upstream avian.')
    date = datetime.date.today().isoformat()
    sync_branch = f'sync/{date}'
    with tempfile.TemporaryDirectory(prefix='jackdaw-sync-') as temp:
        work = Path(temp) / 'jackdaw'
        git('worktree', 'add', '-q', '-B', sync_branch, str(work), f'origin/{branch}', capture=False)
        try:
            if run('git', 'merge', '--no-edit', '-q', f'upstream/{upstream_branch}', cwd=work, check=False):
                run('git', 'merge', '--abort', cwd=work, check=False)
                sys.exit(f'Merging upstream/{upstream_branch} into the fork conflicts; integrate by hand '
                         f"(Jackdaw docs/upgrading-bevy.md, 'Merging upstream Jackdaw afterwards').")
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
                  f'Opened by {ROOT.name} tools/engine_sync.py.')
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
    manifest = ROOT / 'jackdaw.toml'
    manifest.write_text(re.sub(r'(?m)^\[jackdaw\]\n(?:(?:version|bevy|bevy_rev|channel|build) = .*\n)*',
                               lambda _: '[jackdaw]\n' + pins, manifest.read_text(), count=1))
    toolchain = ROOT / 'rust-toolchain.toml'
    toolchain.write_text(re.sub(r'(?m)^channel = ".*"$', f'channel = "{rust}"', toolchain.read_text()))

    # The Jackdaw crates are git dependencies on the fork, declared once in the workspace.
    cargo = ROOT / 'Cargo.toml'
    cargo.write_text(re.sub(r'(?m)^(jackdaw_\w+ = \{ git = "[^"]+", rev = )"[0-9a-f]{40}"',
                            lambda m: f'{m[1]}"{revision}"', cargo.read_text()))

    # Re-resolve the Jackdaw crates at the new rev, then the forks this game's graph
    # uses, then Bevy last: updating the forks re-resolves Bevy's git branch.
    run('cargo', 'metadata', '--format-version', '1', capture=True)
    in_graph = {p['name'] for p in tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']}
    forks = [f['crate'] for f in json.loads(FORKS.read_text())['forks'] if f['crate'] in in_graph]
    if forks:
        run('cargo', 'update', '-q', *[a for crate in forks for a in ('-p', crate)])
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
    run(sys.executable, str(ROOT / 'tools/check_engine_pins.py'), '--source', str(SOURCE), '--editor')
    package = tomllib.loads((ROOT / 'jackdaw.toml').read_text())['package']
    if run('cargo', 'build', '--locked', '-p', package, check=False) \
            or run('cargo', 'check', '--locked', '--workspace', '--all-targets', check=False):
        sys.exit('The game does not build on the new engine set; port it (Bevy migration guides: '
                 f'{GUIDE}) and rerun. Shared Jackdaw moved {before[:8]} -> {revision[:8]}; roll back with: '
                 f'git -C {SOURCE} reset --keep {before} && git -C {ROOT} checkout -- jackdaw.toml '
                 'compatibility.json rust-toolchain.toml Cargo.toml Cargo.lock')
    print(f'Synced: Jackdaw {revision[:8]}, Bevy {record["bevy"]["source"].rsplit("#", 1)[-1][:8]}. '
          'Commit the pins, Cargo.toml and Cargo.lock on a branch.')


if __name__ == '__main__':
    main()
