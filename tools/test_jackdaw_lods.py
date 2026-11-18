#!/usr/bin/env python3
"""Focused checks for our authoring policy, not Jackdaw's LOD implementation."""
import copy
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import jackdaw_lods as lods


class Tuning(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.patch = patch.object(lods, 'MODELS', self.root)
        self.patch.start()
        self.addCleanup(self.patch.stop)
        self.meta = self.root / 'creatures/snail_shell/snail_shell.glb.meta'
        self.meta.parent.mkdir(parents=True)
        original = lods.REPO / 'assets/creatures/snail_shell/snail_shell.glb.meta'
        self.meta.write_bytes(original.read_bytes())
        self.policy = lods.tuning_policy()

    def test_reapply_is_identical_and_keeps_level_sources(self):
        original = self.meta.read_text()
        lods.set_switch_points(self.meta, self.policy)
        first = self.meta.read_text()
        lods.set_switch_points(self.meta, self.policy)
        self.assertEqual(first, self.meta.read_text())
        self.assertIn('snail_shell_LOD0', first)
        self.assertIn('snail_shell_LOD1', first)
        self.assertEqual(original.count('show:'), first.count('show:'))

    def test_bad_distance_order_fails_without_writing(self):
        original = self.meta.read_text()
        policy = copy.deepcopy(self.policy)
        snail = next(f for f in policy['families'] if 'snail_shell' in f['models'])
        snail['cull_metres'] = 0.5
        with self.assertRaisesRegex(ValueError, 'increase strictly'):
            lods.set_switch_points(self.meta, policy)
        self.assertEqual(original, self.meta.read_text())

    def test_new_unconfigured_model_is_not_silently_skipped(self):
        other = self.root / 'other.glb.meta'
        other.write_text(self.meta.read_text())
        with self.assertRaisesRegex(ValueError, 'expected one tuning policy'):
            lods.tuned_meta(other, self.policy)


if __name__ == '__main__':
    unittest.main()
