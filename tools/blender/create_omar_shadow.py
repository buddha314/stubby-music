"""Omar's shadow body: his silhouette, never seen, only cast (`docs/art/omar.md`,
`add-omar-gloves-and-shadow` D2).

A low-poly figure about 1.86 m to the top of the wrap, feet at the origin, facing
Blender +Y (Bevy's -Z, the player's forward). Only its outline matters, so it carries what
the art direction's silhouette studies read by: the tall tagelmust with its fringed end
hanging over the left shoulder, the fitted torso and knee-length coat flaring at the hem,
short flared sleeves over long gloves, sand trousers puffed at the knee and gathered into
heavy boots.

Limbs hang from pivot empties the game turns (`stubby_player::shadow`): `Omar_Hip_L`,
`Omar_Hip_R` (legs, from the hip), `Omar_Shoulder_L`, `Omar_Shoulder_R` (arms, from the
shoulder). Materials are plain: the game draws the body to no camera.

Reproduce: blender -b --factory-startup -P tools/blender/create_omar_shadow.py
Writes assets/characters/omar_shadow/ (glb and asset.json) and previews/characters/omar_shadow/.
"""
from pathlib import Path
import sys

import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from module_kit import ModuleKit  # noqa: E402
from omar_kit import COAT, GLOVE, LEATHER, ORANGE, SAND, blob, tube, under  # noqa: E402

HIP_Z = 0.92
SHOULDER_Z = 1.44
HIP_X = 0.1
SHOULDER_X = 0.21


class OmarShadow(ModuleKit):
    folder = "characters"

    def __init__(self):
        super().__init__("OmarShadow", "omar_shadow", hp=1)
        for key in ("module_kind", "scale"):
            del self.root[key]
        self.root["kind"] = "shadow_body"
        m = self.material
        self.coat = m("Coat", COAT, roughness=0.9)
        self.wrap = m("Wrap", ORANGE, roughness=0.9)
        self.sand = m("Trousers", SAND, roughness=0.9)
        self.boot = m("Boot", LEATHER, roughness=0.8)
        self.glove = m("Glove", GLOVE, roughness=0.6)

    def pivot(self, name, at):
        obj = bpy.data.objects.new(f"Omar_{name}", None)
        obj.empty_display_type = "PLAIN_AXES"
        obj.empty_display_size = 0.08
        self.link(obj)
        obj.location = at
        obj.matrix_world.translation = Vector(at)
        return obj

    def torso(self):
        # The coat: fitted at the chest, belted, flaring to the knee (z 0.5).
        tube(self, "Coat", [(0, 0, 0.5), (0, 0, 0.72), (0, 0, 0.98), (0, 0, 1.2), (0, 0, 1.42), (0, 0, 1.5)],
             [0.3, 0.25, 0.19, 0.2, 0.22, 0.12], self.coat, segments=16, squash=(1.0, 0.72))
        # Head and the tall wrap: a rounded crown well above the skull, the veil over the face.
        blob(self, "Head", (0, 0.01, 1.62), (0.1, 0.11, 0.13), self.wrap)
        blob(self, "WrapCrown", (0, -0.01, 1.76), (0.12, 0.125, 0.11), self.wrap)
        # The tagelmust's fringed end over the left shoulder, hanging to the chest.
        tube(self, "WrapTail", [(-0.08, -0.02, 1.62), (-0.16, 0.0, 1.5), (-0.19, 0.03, 1.32), (-0.2, 0.04, 1.18)],
             [0.05, 0.05, 0.055, 0.06], self.wrap, segments=8, squash=(1.0, 0.35))

    def leg(self, side):
        x = HIP_X * side
        hip = self.pivot("Hip_L" if side < 0 else "Hip_R", (x, 0, HIP_Z))
        parts = [
            # Sand trousers, puffed at the knee, gathered into the boot.
            tube(self, f"Thigh{side}", [(x, 0, HIP_Z), (x, 0.01, 0.6), (x, 0.01, 0.4), (x, 0, 0.27)],
                 [0.09, 0.1, 0.11, 0.07], self.sand, segments=10),
            # Heavy boot: shaft and a long, deep sole.
            tube(self, f"BootShaft{side}", [(x, 0, 0.3), (x, 0, 0.08)], [0.07, 0.075], self.boot, segments=10),
            self.box(f"BootFoot{side}", (0.11, 0.28, 0.1), (x, 0.06, 0.05), self.boot),
        ]
        for part in parts:
            under(part, hip)

    def arm(self, side):
        x = SHOULDER_X * side
        shoulder = self.pivot("Shoulder_L" if side < 0 else "Shoulder_R", (x, 0, SHOULDER_Z))
        parts = [
            # Short flared coat sleeve to mid-forearm.
            tube(self, f"Sleeve{side}", [(x, 0, SHOULDER_Z), (x + 0.03 * side, 0, 1.2), (x + 0.04 * side, 0, 1.06)],
                 [0.07, 0.075, 0.1], self.coat, segments=10),
            # The long glove from inside the sleeve to the hand.
            tube(self, f"Glove{side}", [(x + 0.04 * side, 0, 1.12), (x + 0.045 * side, 0.01, 0.86),
                                        (x + 0.045 * side, 0.02, 0.76)],
                 [0.045, 0.045, 0.05], self.glove, segments=8),
        ]
        for part in parts:
            under(part, shoulder)

    def build(self):
        self.torso()
        for side in (-1, 1):
            self.leg(side)
            self.arm(side)
        profile = {
            "id": self.asset_id,
            "model": f"{self.model_ref}#Scene0",
            "kind": "shadow_body",
            "height_metres": 1.87,
            "feet_at_origin": True,
            "facing": "-Z (the player's forward)",
            "pivots": {"legs": ["Omar_Hip_L", "Omar_Hip_R"], "arms": ["Omar_Shoulder_L", "Omar_Shoulder_R"]},
            "rendered": "never to a camera: shadow only (stubby_core::SHADOW_ONLY_LAYER)",
            "reference": "docs/art/omar.md, silhouettes in docs/art/omar/turnaround-gloves-palette.png",
        }
        return self.finish(profile, aim=(0, 0, 0.95), camera_location=(2.6, 3.6, 1.4))


if __name__ == "__main__":
    report = OmarShadow().build()
    if "result" in globals():
        result.update({"omar_shadow": report})
    else:
        print(report)
