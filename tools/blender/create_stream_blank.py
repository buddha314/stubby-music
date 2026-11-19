"""Stream blank: the 2HP divider between a machine's inherited modules and its own
(add-stream-layers; user, 2026-11-18). On the campfire's rack it stands after the
burbler's clock and VCO, so the player sees which part came down the stream.

Modelled on real 2HP Eurorack blanks (10.16 mm wide, 3U; two mounting screws set
diagonally, top-left and bottom-right), at the kit's x10 scale. Dark anodised, against the
modules' brushed faces, so it reads as a break. A small burbler bubble glyph tops it, and
"@bitlung_" runs up it in glowing blue neon (user, 2026-11-18), set in Mozilla Headline
(SIL Open Font License 1.1, Copyright 2025 The Mozilla Headline Project Authors,
https://github.com/mozilla/mozilla-headline-type). Only the letter shapes are baked into
the model; the font file is not in the repository. Fetch it once to
`~/.cache/stubby-music/fonts/MozillaHeadline.ttf` from
https://github.com/google/fonts/tree/main/ofl/mozillaheadline (or set STUBBY_HEADLINE_FONT).

No jacks, no knobs, no engine parts: `assets/modules/stream_blank/module.json` is empty.

Reproduce:  ~/.local/bin/blender -b --factory-startup -P tools/blender/create_stream_blank.py
"""
from pathlib import Path
import math
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from module_kit import (  # noqa: E402
    AMBER, BODY_DEPTH, FACING_OUT, HEIGHT, PANEL_FRONT, PANEL_THICKNESS, ModuleKit, run,
)
import os  # noqa: E402

import bpy  # noqa: E402
from mathutils import Matrix  # noqa: E402

HP_WIDE = 2
TEXT = "@bitlung_"
FONT = os.environ.get("STUBBY_HEADLINE_FONT",
                      os.path.expanduser("~/.cache/stubby-music/fonts/MozillaHeadline.ttf"))
NEON = (0.05, 0.35, 1.0)    # electric blue
TEXT_TOP = HEIGHT - 0.2     # below the bubble glyph
TEXT_BOTTOM = 0.1
TEXT_MAX_WIDTH = 0.074      # across the 2HP face, leaving the edges clear


def neon_text(kit, core):
    """`TEXT` in Mozilla Headline, reading bottom to top up the face, as a neon sign: blue
    emissive letters on the dark face. (A translucent glow behind them rendered as a solid
    blue plate, emission ignoring alpha, so the letters glow alone.)"""
    if not os.path.exists(FONT):
        raise SystemExit(f"Mozilla Headline not found at {FONT}; see this script's docstring")
    curve = bpy.data.curves.new("StreamBlank_SignText", "FONT")
    curve.body = TEXT
    curve.font = bpy.data.fonts.load(FONT)
    curve.size = 0.1
    curve.extrude = 0.0008
    curve.align_x = "CENTER"
    curve.align_y = "CENTER"
    text_obj = bpy.data.objects.new("StreamBlank_SignText", curve)
    kit.scene.collection.objects.link(text_obj)
    depsgraph = bpy.context.evaluated_depsgraph_get()
    mesh = bpy.data.meshes.new_from_object(text_obj.evaluated_get(depsgraph))
    bpy.data.objects.remove(text_obj)
    bpy.data.curves.remove(curve)
    # Turn it to run up the panel, then fit it between the glyph and the foot.
    mesh.transform(Matrix.Rotation(1.5707963, 4, "Z"))
    xs = [v.co.x for v in mesh.vertices]
    ys = [v.co.y for v in mesh.vertices]
    fit = min((TEXT_TOP - TEXT_BOTTOM) / (max(ys) - min(ys)), TEXT_MAX_WIDTH / (max(xs) - min(xs)))
    mesh.transform(Matrix.Scale(fit, 4))
    z = (TEXT_TOP + TEXT_BOTTOM) / 2
    sign = kit.link(bpy.data.objects.new("StreamBlank_Sign", mesh), core)
    sign.location = (0, PANEL_FRONT - 0.0015, z)
    sign.rotation_euler = FACING_OUT


def build():
    kit = ModuleKit("StreamBlank", "stream_blank", hp=HP_WIDE)
    anodised = kit.material("Anodised", (0.035, 0.04, 0.045), metallic=0.7, roughness=0.45)
    staff = kit.material("Staff", (0.25, 0.12, 0.03), roughness=0.4, emission=AMBER, strength=2.5)
    # Strong enough to glow, not so strong it washes out to white.
    neon = kit.material("Neon", (0.05, 0.3, 1.0), roughness=0.3, emission=NEON, strength=4.0)
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

    neon_text(kit, neon)

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
