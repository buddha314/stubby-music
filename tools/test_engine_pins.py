"""Regression checks for the fail-closed engine pin check; no editor, GPU or build needed.

    python3 tools/test_engine_pins.py
"""
import hashlib
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_engine_pins as pins  # noqa: E402

REPO = Path(__file__).resolve().parents[1]


class EnginePinTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / 'game'
        self.source = Path(self.temp.name) / 'jackdaw'
        workspace = pins.read_toml(REPO / 'Cargo.toml')['workspace']
        # Every workspace member's manifest, so a new crate never breaks the fixture.
        self.members = [m for m, _ in pins.members(REPO, workspace)]
        for name in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'jackdaw.toml',
                     'compatibility.json', 'bevy_main_forks.json',
                     *[m + '/Cargo.toml' for m in self.members]):
            dest = self.root / name
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(REPO / name, dest)
        self.game = 'crates/' + next(m.split('/')[-1] for m in self.members
                                     if pins.read_toml(REPO / m / 'Cargo.toml')['package']['name']
                                     == pins.read_toml(REPO / 'jackdaw.toml')['package'])
        self.source.mkdir()
        for name in ('Cargo.lock', 'rust-toolchain.toml'):
            shutil.copyfile(self.root / name, self.source / name)
        self.record = json.loads((self.root / 'compatibility.json').read_text())
        self.responses = {
            ('rev-parse', 'HEAD'): self.record['jackdaw']['revision'],
            ('status', '--porcelain', '--untracked-files=no'): '',
            ('remote', 'get-url', 'origin'): self.record['jackdaw']['origin'],
            ('remote', 'get-url', 'upstream'): self.record['jackdaw']['upstream'],
        }
        mock = patch.object(pins, 'git', side_effect=lambda source, *args: self.responses[args])
        mock.start()
        self.addCleanup(mock.stop)

    def check(self):
        return pins.validate(self.root) + pins.validate_editor(self.root, self.source)

    def edit(self, relative, old, new):
        path = self.root / relative
        text = path.read_text()
        self.assertIn(old, text, f'fixture {relative} lacks {old!r}')
        path.write_text(text.replace(old, new, 1))

    def replace_line(self, relative, prefix, replacement):
        path = self.root / relative
        line = next(line for line in path.read_text().splitlines() if line.startswith(prefix))
        self.edit(relative, line, replacement)

    def assertError(self, fragment, errors=None):
        errors = self.check() if errors is None else errors
        self.assertTrue(any(fragment in e for e in errors), f'no error containing {fragment!r}: {errors}')

    def test_matching_configuration_passes_and_is_read_only(self):
        before = {p: p.read_bytes() for p in Path(self.temp.name).rglob('*') if p.is_file()}
        self.assertEqual(self.check(), [])
        self.assertEqual(before, {p: p.read_bytes() for p in before})

    def test_source_drift_and_dirty_source_are_rejected(self):
        self.responses[('rev-parse', 'HEAD')] = '0' * 40
        self.responses[('status', '--porcelain', '--untracked-files=no')] = ' M Cargo.toml'
        errors = self.check()
        self.assertError('checkout drift', errors)
        self.assertError('tracked changes', errors)

    def test_wrong_fork_remote_is_rejected(self):
        self.responses[('remote', 'get-url', 'origin')] = self.record['jackdaw']['upstream']
        self.assertError('origin must be')

    def test_second_bevy_in_lockfile_is_rejected(self):
        self.edit('Cargo.lock', self.record['bevy']['version'], '0.0.0')
        self.assertError('Cargo.lock must resolve one Bevy')

    def test_member_with_its_own_bevy_or_lockfile_is_rejected(self):
        # `= false` rather than a version: games often add `bevy.features = [...]`
        # after it, and a plain `bevy = "..."` would then not parse.
        self.edit(self.game + '/Cargo.toml', 'bevy.workspace = true', 'bevy.workspace = false')
        (self.root / self.game / 'Cargo.lock').write_text('version = 4\n')
        errors = self.check()
        self.assertError('bevy must inherit the pinned engine', errors)
        self.assertError('one Cargo.lock', errors)

    def test_member_with_its_own_avian_is_rejected(self):
        self.edit(self.game + '/Cargo.toml', 'avian3d.workspace = true', 'avian3d.workspace = false')
        self.assertError('avian3d must inherit the pinned engine')

    def test_published_or_path_runtime_cannot_replace_pinned_fork(self):
        self.replace_line(self.game + '/Cargo.toml', 'jackdaw_runtime =', 'jackdaw_runtime = "0.21"')
        self.assertError('jackdaw_runtime must inherit')
        self.replace_line(self.game + '/Cargo.toml', 'jackdaw_runtime =',
                          'jackdaw_runtime = { path = "../../../jackdaw/crates/jackdaw_runtime" }')
        self.assertError('jackdaw_runtime must inherit')

    def test_workspace_pin_must_match_record(self):
        rev = self.record['jackdaw']['revision']
        self.edit('Cargo.toml', f'rev = "{rev}"', 'rev = "' + '1' * 40 + '"')
        self.assertError('workspace jackdaw_')

    def test_lock_revision_must_match_record(self):
        rev = self.record['jackdaw']['revision']
        self.edit('Cargo.lock', f'#{rev}"', '#' + '2' * 40 + '"')
        self.assertTrue(any('Cargo.lock resolves jackdaw_' in e and '2' * 40 in e and rev in e
                            for e in self.check()))

    def test_linked_lock_is_rejected(self):
        source = pins.jackdaw_lock_source(self.record['jackdaw'])
        self.edit('Cargo.lock', f'source = "{source}"\n', '')
        self.assertError('linked Jackdaw paths')

    def test_fork_patch_must_match_manifest(self):
        self.edit('Cargo.toml', 'https://github.com/buddha314/avian', 'https://github.com/jbuehler23/avian')
        self.assertError('[patch.crates-io] avian3d must be')

    def test_patch_outside_manifest_is_rejected(self):
        self.edit('Cargo.toml', '[patch.crates-io]\n',
                  '[patch.crates-io]\nparry3d = { git = "https://github.com/someone/parry" }\n')
        self.assertError('not in bevy_main_forks.json')

    def test_second_avian_in_lockfile_is_rejected(self):
        # What Jackdaw's Physics module does unpatched: a second avian3d, from another fork.
        path = self.root / 'Cargo.lock'
        path.write_text(path.read_text() + '\n[[package]]\nname = "avian3d"\nversion = "0.8.0"\n'
                        'source = "git+https://github.com/jbuehler23/avian?branch=bevy-main#' + '4' * 40 + '"\n')
        self.assertError('one avian only')

    def test_game_crates_named_bevy_are_allowed(self):
        # A game's own `bevy_*` crate by path, or a crates.io ecosystem crate, is not the engine.
        self.edit(self.game + '/Cargo.toml', '[dependencies]\n',
                  '[dependencies]\nbevy_mygame_audio = { path = "../mygame_audio" }\nbevy_egui = "0.42"\n')
        self.assertEqual(self.check(), [])

    def test_bevy_engine_crate_from_git_must_inherit(self):
        self.edit(self.game + '/Cargo.toml', '[dependencies]\n',
                  '[dependencies]\nbevy_ecs = { git = "https://github.com/bevyengine/bevy", branch = "main" }\n')
        self.assertError('bevy_ecs must inherit the pinned engine')

    def test_unknown_project_package_is_rejected(self):
        self.replace_line('jackdaw.toml', 'package =', 'package = "no_such_crate"')
        self.assertError('is not a workspace member')

    def test_project_pins_and_toolchain_must_match_record(self):
        self.replace_line('jackdaw.toml', 'bevy_rev =', 'bevy_rev = "' + '3' * 40 + '"')
        self.edit('rust-toolchain.toml', self.record['rust'], '1.0.0')
        errors = self.check()
        self.assertError('jackdaw.toml [jackdaw] differs', errors)
        self.assertError('project compiler must match', errors)

    def test_stale_shared_binaries_are_rejected(self):
        target = self.source / 'target/debug'
        target.mkdir(parents=True)
        receipt = {'revision': self.record['jackdaw']['revision'], 'sha256': {}}
        for name in ('jackdaw', 'jd'):
            (target / name).write_bytes(name.encode())
            receipt['sha256'][name] = hashlib.sha256(name.encode()).hexdigest()
        dest = self.root / '.jackdaw/editor-build.json'
        dest.parent.mkdir()
        dest.write_text(json.dumps(receipt))
        self.assertEqual(pins.validate_binaries(self.root, self.source), [])
        (target / 'jd').write_bytes(b'another binary')
        self.assertError('jd binary differs', pins.validate_binaries(self.root, self.source))
        receipt['revision'] = '0' * 40
        dest.write_text(json.dumps(receipt))
        self.assertError('another source revision', pins.validate_binaries(self.root, self.source))

    def tree_line(self, name):
        bevy = self.record['bevy']
        line = f"{name} v{bevy['version']}"
        if bevy['source'].startswith('git+'):
            url, rev = bevy['source'].removeprefix('git+').split('#')
            line += f' ({url}#{rev[:8]})'
        return line

    def test_active_second_bevy_is_rejected(self):
        good = self.tree_line('bevy')
        self.assertEqual(pins.validate_tree(good, self.record), [])
        self.assertTrue(pins.validate_tree(good + '\nbevy_ecs v0.19.1', self.record))
        self.assertTrue(pins.validate_tree('', self.record))

    def test_colored_tree_output_is_accepted(self):
        # CI forces cargo colour; the dedup marker arrives wrapped in ANSI codes.
        line = self.tree_line('bevy_ecs') + ' \x1b[33m\x1b[2m(*)\x1b[39m\x1b[22m'
        self.assertEqual(pins.validate_tree(line, self.record), [])


if __name__ == '__main__':
    unittest.main()
