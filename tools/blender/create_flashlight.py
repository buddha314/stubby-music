"""The forensic flashlight: a hand-held alternate light source of the kind used at crime
scenes (add-forensic-flashlight), held on B. The game shines a violet beam from its lens
and shows collectibles in the beam as holograms.

A machined torch at real size (metres, 1:1): a black anodised body with a knurled grip, a
tail cap and a side switch, a flared head with a wide bezel, and a violet lens that glows
while the light is on. No brand, no logo.

Built along Blender +Y, so the lens points along glTF -Z, the game's forward.

Game-facing nodes (empties, glTF extras):
  Flashlight_Root  origin at the grip, where the hand holds it
  Grip             the hand socket (same place as the root)
  Lens             the lens centre; local -Z (glTF) is where the beam goes
  Light_lens       the lens glow, lit while the light is on

Reproduce: blender -b --factory-startup -P tools/blender/create_flashlight.py
Writes assets/models/equipment/flashlight.glb, assets/equipment/flashlight/asset.json,
assets-src/blender/equipment/flashlight.blend and previews/equipment/flashlight/.
"""
from pathlib import Path
import math
import sys

import bmesh
import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_burbler import linear  # noqa: E402
from module_kit import ModuleKit  # noqa: E402

BODY_RADIUS = 0.0175
HEAD_RADIUS = 0.029
TAIL_Y, GRIP_END_Y, BODY_END_Y, HEAD_END_Y = -0.075, 0.03, 0.085, 0.118
KNURL_BANDS = 9
# Violet, as forensic light sources are (about 405-450 nm).
VIOLET = "#7a3cff"
ALONG_Y = (-math.pi / 2, 0.0, 0.0)  # a Z-axis primitive turned to run along +Y


class Flashlight(ModuleKit):
    folder = "equipment"

    def __init__(self):
        super().__init__("Flashlight", "flashlight", hp=1)
        for key in ("module_kind", "scale"):
            del self.root[key]
        self.root["kind"] = "held_equipment"
        self.root["role"] = "finder"
        m = self.material
        self.anodised = m("Anodised", linear("#1d1f22"), metallic=0.9, roughness=0.42)
        self.knurl = m("Knurl", linear("#111214"), metallic=0.85, roughness=0.7)
        self.bezel = m("Bezel", linear("#5c6066"), metallic=1.0, roughness=0.28)
        self.rubber = m("Switch", linear("#2a2b2d"), roughness=0.9)
        violet = linear(VIOLET)
        self.lens_mat = m("Lens", (0.06, 0.02, 0.12), roughness=0.05, emission=violet, strength=6.0)

    # ------------------------------------------------------------ helpers

    def tube(self, name, radius_from, radius_to, y_from, y_to, mat, segments=40):
        """A cylinder or cone along +Y from `y_from` to `y_to`."""
        depth = y_to - y_from

        def fill(bm):
            bmesh.ops.create_cone(bm, cap_ends=True, segments=segments,
                                  radius1=radius_from, radius2=radius_to, depth=depth)
        obj = self.link(bpy.data.objects.new(f"{self.kind}_{name}", self.mesh_from(name, fill)), mat)
        obj.location = (0.0, (y_from + y_to) / 2, 0.0)
        obj.rotation_euler = ALONG_Y
        for face in obj.data.polygons:
            face.use_smooth = len(face.vertices) == 4
        return obj

    def empty(self, name, location, rotation=(0.0, 0.0, 0.0), **props):
        obj = bpy.data.objects.new(name, None)
        obj.empty_display_type = "PLAIN_AXES"
        obj.empty_display_size = 0.02
        self.link(obj)
        obj.location = location
        obj.rotation_euler = rotation
        for key, value in props.items():
            obj[key] = value
        return obj

    # ------------------------------------------------------------ parts

    def body(self):
        self.tube("TailCap", BODY_RADIUS * 1.08, BODY_RADIUS * 1.08, TAIL_Y - 0.012, TAIL_Y, self.anodised)
        self.tube("Grip", BODY_RADIUS * 0.96, BODY_RADIUS * 0.96, TAIL_Y, GRIP_END_Y, self.knurl)
        # Knurled bands: raised rings along the grip.
        step = (GRIP_END_Y - TAIL_Y) / (KNURL_BANDS + 1)
        for k in range(KNURL_BANDS):
            y = TAIL_Y + step * (k + 1)
            self.tube(f"Knurl_{k}", BODY_RADIUS, BODY_RADIUS, y - 0.0025, y + 0.0025, self.anodised)
        self.tube("Body", BODY_RADIUS, BODY_RADIUS, GRIP_END_Y, BODY_END_Y, self.anodised)
        # The side switch, on top where the thumb rests.
        switch = self.box("Switch", (0.009, 0.014, 0.004), (0.0, (GRIP_END_Y + BODY_END_Y) / 2,
                                                            BODY_RADIUS + 0.0012), self.rubber)
        for face in switch.data.polygons:
            face.use_smooth = False

    def head(self):
        self.tube("Head", BODY_RADIUS, HEAD_RADIUS, BODY_END_Y, HEAD_END_Y, self.anodised)
        # A collar behind the lens, and a rim around it.
        self.tube("Bezel", HEAD_RADIUS * 1.06, HEAD_RADIUS * 1.06, HEAD_END_Y, HEAD_END_Y + 0.004,
                  self.bezel)
        rim = self.ring("BezelRim", HEAD_RADIUS * 1.0, 0.0032, self.bezel, None)
        rim.location = (0.0, HEAD_END_Y + 0.0055, 0.0)
        rim.rotation_euler = ALONG_Y
        # The lens, proud of the collar inside the rim.
        lens = self.tube("Light_lens", HEAD_RADIUS * 0.94, HEAD_RADIUS * 0.94, HEAD_END_Y + 0.004,
                         HEAD_END_Y + 0.0062, self.lens_mat, segments=48)
        lens["light"] = "lens"
        lens["peak_strength"] = 6.0

    def build(self):
        self.empty("Grip", (0, 0, 0), socket="hand")
        # Blender +Y becomes glTF -Z: the Lens empty's local axes follow the root's.
        self.empty("Lens", (0.0, HEAD_END_Y + 0.007, 0.0), beam="forward")
        self.body()
        self.head()
        profile = {
            "id": self.asset_id,
            "model": f"{self.model_ref}#Scene0",
            "kind": "held_equipment",
            "role": "finder",
            "size_metres": {"length": HEAD_END_Y + 0.0087 - (TAIL_Y - 0.012),
                            "head_diameter": 2 * HEAD_RADIUS * 1.06},
            "grip": "Grip",
            "beam": {"node": "Lens", "along": "-Z (forward)", "colour": VIOLET,
                     "note": "the game's spotlight and reveal start here (assets/ui/flashlight.json)"},
            "lights": [{"node": "Light_lens", "light": "lens", "rest_strength": 0.0,
                        "peak_strength": 6.0, "lit_while": "held"}],
            "held": {"note": "first-person: hold at Grip in the right hand, lens forward; real size"},
        }
        return self.finish(profile, aim=(0, 0.04, 0), camera_location=(0.17, 0.3, 0.1))


if __name__ == "__main__":
    report = Flashlight().build()
    if "result" in globals():
        result.update({"flashlight": report})
    else:
        print(report)
