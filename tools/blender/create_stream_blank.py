"""Stream blank: the 2HP divider between a machine's inherited modules and its own
(add-stream-layers; user, 2026-11-18). On the campfire's rack it stands after the
burbler's clock and VCO, so the player sees which part came down the stream.

Modelled on real 2HP Eurorack blanks (10.16 mm wide, 3U; two mounting screws set
diagonally, top-left and bottom-right), at the kit's x10 scale. Dark anodised, against the
modules' brushed faces, so it reads as a break. The stream's staff runs down it, five thin
glowing lines, as it runs on the water, and a small burbler bubble glyph tops it.

No jacks, no knobs, no engine parts: `assets/modules/stream_blank/module.json` is empty.

Reproduce:  ~/.local/bin/blender -b --factory-startup -P tools/blender/create_stream_blank.py
"""
from pathlib import Path
import math
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from module_kit import AMBER, BODY_DEPTH, HEIGHT, PANEL_FRONT, PANEL_THICKNESS, ModuleKit, run  # noqa: E402

HP_WIDE = 2
STAFF_LINES = 5
STAFF_SPACING = 0.012       # between the staff's lines, across the 2HP face
STAFF_TOP = HEIGHT - 0.2    # below the bubble glyph
STAFF_BOTTOM = 0.09


def build():
    kit = ModuleKit("StreamBlank", "stream_blank", hp=HP_WIDE)
    anodised = kit.material("Anodised", (0.035, 0.04, 0.045), metallic=0.7, roughness=0.45)
    staff = kit.material("Staff", (0.25, 0.12, 0.03), roughness=0.4, emission=AMBER, strength=2.5)
    w = kit.width

    # Case and face, as the kit's `case` but with a dark face and a 2HP blank's two
    # diagonal screws.
    kit.box("Body", (w - 0.01, BODY_DEPTH, HEIGHT - 0.03), (0, 0, HEIGHT / 2), kit.body_mat, bevel=0.01)
    face_y = -BODY_DEPTH / 2 - PANEL_THICKNESS / 2
    kit.box("Faceplate", (w, PANEL_THICKNESS, HEIGHT), (0, face_y, HEIGHT / 2), anodised, bevel=0.004)
    for x, z in ((-w / 2 + 0.025, HEIGHT - 0.03), (w / 2 - 0.025, 0.03)):
        kit.cylinder("Screw", 0.011, 0.006, (x, PANEL_FRONT - 0.003, z), kit.brass_mat, segments=16)

    # The burbler's bubble: three rounded bubbles rising, the burbler panel's glyph in small.
    for i, (cx, cz, r) in enumerate(((-0.012, HEIGHT - 0.13, 0.014),
                                     (0.01, HEIGHT - 0.098, 0.01),
                                     (-0.004, HEIGHT - 0.074, 0.007))):
        pts = [(cx + r * math.cos(t), cz + r * math.sin(t)) for t in (math.tau * k / 24 for k in range(25))]
        kit.soft_trace(f"Bubble{i}", pts, "xz", PANEL_FRONT - 0.003, 0.003, staff)

    # The staff, running down the face as the stream runs on the water.
    for n in range(STAFF_LINES):
        x = (n - (STAFF_LINES - 1) / 2) * STAFF_SPACING
        kit.soft_trace(f"Staff{n}", [(x, STAFF_TOP), (x, STAFF_BOTTOM)], "xz", PANEL_FRONT - 0.003, 0.0022, staff)

    base = kit.base_profile()
    profile = {
        "id": base["id"],
        "model": base["model"],
        "module_kind": base["module_kind"],
        "scale": base["scale"],
        "size_metres": base["size_metres"],
        "silhouette": "plain",
        "reference": "Eurorack 2HP blank panels: 10.16 mm, two diagonal mounting screws; "
                     "decorated blanks (Materia, Synthrotek)",
        "role": "divider: between a machine's inherited modules (left) and its own (right)",
        "ports": [],
        "params": [],
        "lights": [],
        "placement": base["placement"],
    }
    return kit.finish(profile)


if __name__ == "__main__":
    report = run({"stream_blank": build}, __file__)
    if report is not None:
        if "result" in globals():
            result.update({"modules": report})
        else:
            print(report)
