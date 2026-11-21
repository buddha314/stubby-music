"""Export the sampler's rack panel with the shared weathered-wood module kit."""
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_hud_modules import HudKit
from create_purifier_modules import ports
from module_kit import PANEL_FRONT, run


def build_sampler():
    kit = HudKit("Sampler", "sample_player", hp=8, module_kind="SamplePlayer")
    kit.case()
    kit.title("SAMPLER")
    kit.trace("Clip", [(-.13, .83), (-.08, .83), (-.04, .98), (0, .72),
                        (.04, .88), (.08, .83), (.13, .83)],
              "xz", PANEL_FRONT - .004, .009, kit.glow)
    kit.jack("trigger", "in", .55, "TRIG")
    kit.output_patch(.2, .2)
    kit.jack("out", "out", .2, "OUT", on_patch=True)
    return kit.finish(kit.profile({
        "silhouette": "sample_wave",
        "ports": ports(("trigger", "in", None), ("out", "out", None)),
        "params": [],
    }))


if __name__ == "__main__":
    run({"sample_player": build_sampler}, __file__)
