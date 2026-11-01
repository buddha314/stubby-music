"""Player start marker: what the level designer sees where the player will begin
(add-opening-region 2.7a).

An editor-only marker, like Unreal's PlayerStart icon: the prefab carries
`HiddenInGame`, so it is never drawn in play. It shows three things at a glance:

  PlayerStart_Footprint   a ring the size of the player's footprint, on the ground
  PlayerStart_Facing      an arrow on the ground along the way the player will look
                          (Bevy forward, -Z; Blender +Y before export)
  PlayerStart_Eye         a post up to eye height (1.65 m, `controls::EYE_HEIGHT`) with
                          a sight tick pointing forward: where the camera starts

The origin is on the ground at the footprint's centre, so dropping the prefab snaps
it to the floor; the game stands the player there, facing the marker's yaw.

Reproduce: blender -b --factory-startup -P tools/blender/create_player_start.py
Writes assets/models/actors/player_start_marker.glb and assets/actors/player_start_marker/.
"""
from pathlib import Path
import math
import sys

import bmesh
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_burbler import linear  # noqa: E402
from create_patch_cable import tube  # noqa: E402
from module_kit import ModuleKit  # noqa: E402

EYE_HEIGHT = 1.65        # crates/stubby_game/src/controls.rs
FOOTPRINT = 0.4          # radius of the ring, about a standing player's footprint
LINE = 0.02              # stroke width on the ground
LIFT = 0.01              # strokes float just above the ground so they never z-fight
COLOR = "#ffd39a"        # the palette's build colour


class PlayerStartMarker(ModuleKit):
    folder = "actors"

    def __init__(self):
        super().__init__("PlayerStart", "player_start_marker", hp=1)
        for key in ("module_kind", "scale"):
            del self.root[key]
        self.root["kind"] = "editor_marker"
        self.root["marks"] = "player_start"
        c = linear(COLOR)
        self.line = self.material("Line", c, emission=c, strength=2.0)
        self.fill = self.material("Fill", c, emission=c, strength=0.6)
        bsdf = self.fill.node_tree.nodes["Principled BSDF"]
        bsdf.inputs["Alpha"].default_value = 0.18
        self.fill.surface_render_method = "BLENDED"

    def mesh(self, name, fill, mat):
        obj = self.link(__import__("bpy").data.objects.new(f"PlayerStart_{name}", self.mesh_from(name, fill)), mat)
        return obj

    def build(self):
        def ring(bm, r=FOOTPRINT, w=LINE, n=48):
            outer = [bm.verts.new((math.cos(a) * (r + w / 2), math.sin(a) * (r + w / 2), LIFT))
                     for a in (2 * math.pi * i / n for i in range(n))]
            inner = [bm.verts.new((math.cos(a) * (r - w / 2), math.sin(a) * (r - w / 2), LIFT))
                     for a in (2 * math.pi * i / n for i in range(n))]
            for i in range(n):
                j = (i + 1) % n
                bm.faces.new((inner[i], inner[j], outer[j], outer[i]))
        self.mesh("Footprint", ring, self.line)

        def disc(bm, r=FOOTPRINT, n=48):
            centre = bm.verts.new((0, 0, LIFT * 0.5))
            rim = [bm.verts.new((math.cos(a) * r, math.sin(a) * r, LIFT * 0.5))
                   for a in (2 * math.pi * i / n for i in range(n))]
            for i in range(n):
                bm.faces.new((centre, rim[i], rim[(i + 1) % n]))
        self.mesh("FootprintFill", disc, self.fill)

        def arrow(bm):
            # Shaft from the centre past the ring, then a head; +Y is forward (-Z in Bevy).
            shaft = [(-LINE, 0.0), (LINE, 0.0), (LINE, 0.62), (-LINE, 0.62)]
            v = [bm.verts.new((x, y, LIFT)) for x, y in shaft]
            bm.faces.new(v)
            head = [(-0.13, 0.58), (0.13, 0.58), (0.0, 0.82)]
            bm.faces.new([bm.verts.new((x, y, LIFT)) for x, y in head])
        self.mesh("Facing", arrow, self.line)

        def eye(bm):
            tube(bm, [Vector((0, 0, 0)), Vector((0, 0, EYE_HEIGHT))], 0.012, sides=8)
            tube(bm, [Vector((0, 0, EYE_HEIGHT)), Vector((0, 0.22, EYE_HEIGHT))], 0.012, sides=8)
            bmesh.ops.create_uvsphere(bm, u_segments=12, v_segments=6, radius=0.045,
                                      matrix=__import__("mathutils").Matrix.Translation((0, 0, EYE_HEIGHT)))
        self.mesh("Eye", eye, self.line)

        profile = {
            "id": self.asset_id,
            "model": f"{self.model_ref}#Scene0",
            "kind": "editor_marker",
            "marks": "PlayerStart (add-opening-region 2.7)",
            "hidden_in_game": True,
            "origin": "on the ground at the footprint's centre",
            "forward": "-Z in Bevy (the arrow); the player faces the marker's yaw",
            "eye_height_m": EYE_HEIGHT,
            "footprint_radius_m": FOOTPRINT,
            "color": COLOR,
        }
        return self.finish(profile, aim=(0, 0.2, 0.6), camera_location=(2.2, -2.6, 1.9))


if __name__ == "__main__":
    print(PlayerStartMarker().build())
