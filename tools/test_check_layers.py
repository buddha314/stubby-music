"""The layering rules, on made-up dependency lists; no cargo needed."""
import unittest

import check_layers as layers


class LayerRules(unittest.TestCase):
    def test_the_intended_shape_passes(self):
        deps = {'stubby_life': ['bevy_stubby_audio', 'stubby_core'],
                'bevy_stubby_rack': ['stubby_rack', 'stubby_core'],
                'stubby_game': ['stubby_life', 'bevy_stubby_rack', 'stubby_recipes']}
        self.assertEqual(layers.layer_errors(deps), [])

    def test_a_system_may_not_use_another_system(self):
        errors = layers.layer_errors({'stubby_life': ['stubby_player']})
        self.assertTrue(any('stubby_life (system) may not depend on stubby_player' in e for e in errors))

    def test_a_system_may_not_use_another_systems_core(self):
        errors = layers.layer_errors({'bevy_stubby_rack': ['stubby_recipes']})
        self.assertTrue(any('core of bevy_stubby_recipes' in e for e in errors))

    def test_a_core_may_not_reach_up(self):
        errors = layers.layer_errors({'stubby_rack': ['stubby_core']})
        self.assertTrue(any('stubby_rack (core) may not depend on stubby_core (service)' in e for e in errors))

    def test_an_unplaced_crate_is_reported(self):
        errors = layers.layer_errors({'stubby_unplaced': []})
        self.assertTrue(any('stubby_unplaced: not placed' in e for e in errors))


if __name__ == '__main__':
    unittest.main()
