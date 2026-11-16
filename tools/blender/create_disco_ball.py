"""The disco ball: the forensic flashlight's light (add-forensic-flashlight). While the
flashlight is held on B, this mirror ball floats in front of the player and spins, and the
game throws a ring of narrow violet and cyan beams from it that sweep the ground.

A classic mirror ball at real size: small square mirror tiles in latitude bands over a
dark grout core, the tiles fewer toward the poles so each stays square. No hanger: it
floats.

Game-facing nodes (empties, glTF extras):
  DiscoBall_Root  the ball's centre; the game spins it about its local Y

Reproduce: blender -b --factory-startup -P tools/blender/create_disco_ball.py
Writes assets/equipment/disco_ball/disco_ball.glb, its asset.json and .blend, and
previews/equipment/disco_ball/.
"""
from pathlib import Path
import math
import sys

import bmesh
import bpy

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_burbler import linear  # noqa: E402
from module_kit import ModuleKit  # noqa: E402

RADIUS = 0.12          # metres: a 24 cm ball
BANDS = 18             # latitude bands from pole to pole
TILE_GAP = 0.12        # the grout between tiles, as a fraction of a tile


class DiscoBall(ModuleKit):
    folder = "equipment"

    def __init__(self):
        super().__init__("DiscoBall", "disco_ball", hp=1)
        for key in ("module_kind", "scale"):
            del self.root[key]
        self.root["kind"] = "held_equipment_light"
        m = self.material
        self.mirror = m("Mirror", linear("#d8dce2"), metallic=1.0, roughness=0.04)
        self.grout = m("Grout", linear("#141518"), roughness=0.9)
        self.tiles = 0

    def facets(self):
        """Square-ish tiles on the sphere: per band, as many as fit round its latitude."""
        def fill(bm):
            band = math.pi / BANDS
            for b in range(BANDS):
                lat0 = -math.pi / 2 + b * band
                lat1 = lat0 + band
                mid = (lat0 + lat1) / 2
                count = max(4, round(2 * math.pi * math.cos(mid) / band))
                step = 2 * math.pi / count
                inset_lat = band * TILE_GAP / 2
                inset_lon = step * TILE_GAP / 2
                for k in range(count):
                    lon0 = k * step + inset_lon
                    lon1 = (k + 1) * step - inset_lon
                    a0, a1 = lat0 + inset_lat, lat1 - inset_lat

                    def point(lat, lon):
                        # Slightly proud of the core, so the grout shows between tiles.
                        r = RADIUS * 1.004
                        return (r * math.cos(lat) * math.cos(lon), r * math.cos(lat) * math.sin(lon),
                                r * math.sin(lat))
                    verts = [bm.verts.new(point(lat, lon))
                             for lat, lon in ((a0, lon0), (a0, lon1), (a1, lon1), (a1, lon0))]
                    bm.faces.new(verts)
                    self.tiles += 1
        obj = self.link(bpy.data.objects.new(f"{self.kind}_Tiles", self.mesh_from("Tiles", fill)),
                        self.mirror)
        # Flat tiles: each catches the light at its own angle, which is the sparkle.
        for face in obj.data.polygons:
            face.use_smooth = False
        return obj

    def build(self):
        self.sphere("Core", RADIUS, self.grout, None, segments=32, rings=16)
        self.facets()
        profile = {
            "id": self.asset_id,
            "model": f"{self.model_ref}#Scene0",
            "kind": "held_equipment_light",
            "size_metres": {"diameter": 2 * RADIUS},
            "tiles": self.tiles,
            "spin": {"node": "DiscoBall_Root", "axis": "Y",
                     "note": "the game spins the root; its beams are spotlights it adds "
                             "(assets/ui/flashlight.json)"},
        }
        return self.finish(profile, aim=(0, 0, 0), camera_location=(0.35, -0.55, 0.22))


if __name__ == "__main__":
    report = DiscoBall().build()
    if "result" in globals():
        result.update({"disco_ball": report})
    else:
        print(report)
