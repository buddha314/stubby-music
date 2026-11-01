"""Regression checks for fail-closed version validation; no editor or GPU needed."""
import json
import hashlib
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

import check_jackdaw_versions as versions

REPO = Path(__file__).resolve().parents[1]


class CompatibilityTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / 'game'
        self.source = Path(self.temp.name) / 'jackdaw'
        for name in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml',
                     'crates/stubby_game/Cargo.toml', 'crates/bevy_stubby_recipes/Cargo.toml', 'jackdaw/Cargo.toml',
                     'jackdaw/jackdaw.toml', 'jackdaw/compatibility.json'):
            dest = self.root / name
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(REPO / name, dest)
        self.source.mkdir()
        for name in ('Cargo.lock', 'rust-toolchain.toml'):
            shutil.copyfile(self.root / name, self.source / name)
        self.record = json.loads((self.root / 'jackdaw/compatibility.json').read_text())
        self.responses = {
            ('rev-parse', 'HEAD'): self.record['jackdaw']['revision'],
            ('status', '--porcelain', '--untracked-files=no'): '',
            ('remote', 'get-url', 'origin'): self.record['jackdaw']['origin'],
            ('remote', 'get-url', 'upstream'): self.record['jackdaw']['upstream'],
        }
        mock = patch.object(versions, 'git', side_effect=lambda source, *args: self.responses[args])
        mock.start()
        self.addCleanup(mock.stop)

    def check(self):
        return versions.validate(self.root, self.source)

    def test_matching_configuration_is_read_only(self):
        before = {p: p.read_bytes() for p in Path(self.temp.name).rglob('*') if p.is_file()}
        self.assertEqual(self.check(), [])
        self.assertEqual(before, {p: p.read_bytes() for p in before})

    def test_source_drift_and_dirty_source_are_rejected(self):
        self.responses[('rev-parse', 'HEAD')] = '0' * 40
        self.responses[('status', '--porcelain', '--untracked-files=no')] = ' M Cargo.toml'
        errors = self.check()
        self.assertTrue(any('checkout drift' in e for e in errors))
        self.assertTrue(any('tracked changes' in e for e in errors))

    def test_lockfile_drift_is_rejected(self):
        path = self.root / 'Cargo.lock'
        path.write_text(path.read_text().replace(self.record['bevy']['version'], '0.0.0'))
        self.assertTrue(any('Cargo.lock must resolve' in e for e in self.check()))

    def test_independent_adapter_version_and_lockfile_are_rejected(self):
        path = self.root / 'jackdaw/Cargo.toml'
        path.write_text(path.read_text().replace('bevy.workspace = true', 'bevy.workspace = false'))
        (self.root / 'jackdaw/Cargo.lock').write_text('version = 4\n')
        errors = self.check()
        self.assertTrue(any('inherit Bevy' in e for e in errors))
        self.assertTrue(any('separate adapter lockfile' in e for e in errors))

    def test_published_runtime_cannot_replace_shared_source(self):
        path = self.root / 'jackdaw/Cargo.toml'
        original = path.read_text()
        line = next(line for line in original.splitlines() if line.startswith('jackdaw_runtime ='))
        path.write_text(original.replace(line, 'jackdaw_runtime = "0.21"'))
        self.assertTrue(any('jackdaw_runtime must use' in e for e in self.check()))

    def test_wrong_fork_remote_is_rejected(self):
        self.responses[('remote', 'get-url', 'origin')] = self.record['jackdaw']['upstream']
        self.assertTrue(any('origin must be' in e for e in self.check()))

    def test_stale_shared_binaries_are_rejected(self):
        target = self.source / 'target/debug'
        target.mkdir(parents=True)
        receipt = {'revision': self.record['jackdaw']['revision'], 'sha256': {}}
        for name in ('jackdaw', 'jd'):
            (target / name).write_bytes(name.encode())
            receipt['sha256'][name] = hashlib.sha256(name.encode()).hexdigest()
        dest = self.root / 'jackdaw/.jackdaw/editor-build.json'
        dest.parent.mkdir()
        dest.write_text(json.dumps(receipt))
        self.assertEqual(versions.validate_binaries(self.root, self.source), [])
        (target / 'jd').write_bytes(b'another binary')
        self.assertTrue(any('jd binary differs' in e for e in versions.validate_binaries(self.root, self.source)))
        receipt['revision'] = '0' * 40
        dest.write_text(json.dumps(receipt))
        self.assertTrue(any('another source revision' in e for e in versions.validate_binaries(self.root, self.source)))

    def test_active_second_bevy_is_rejected(self):
        good = 'bevy v' + self.record['bevy']['version']
        self.assertEqual(versions.validate_tree(good, self.record), [])
        self.assertTrue(versions.validate_tree(good + '\nbevy_ecs v0.19.1', self.record))
        self.assertTrue(versions.validate_tree('', self.record))


if __name__ == '__main__':
    unittest.main()
