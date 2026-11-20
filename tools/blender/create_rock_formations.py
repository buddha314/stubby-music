"""Rock formations for the seeded opening (add-high-pass-sources D2): clusters of the
echo grotto's weathered granite, placed whole, some with openings that pass short waves
and stop long ones. Each opening is a high-pass filter source.

A formation is one model made of several Rock Generator rocks (`create_boulders.make_rock`,
same granite, same buried fraction). Openings are cut or left by the layout:

  window   an elliptical tunnel bored through a slab (a boolean, before decimation)
  slot     a narrow gap between two fins
  bridge   a capstone across two supports, a passage under it

Every opening is an `Opening_<k>` socket (a childless empty on the root, outside the
`_LOD` nodes, so the game's socket binding keeps it) at the middle of the passage, its
local +Z along the passage, with extras {"resource": "rock_opening", "opening": k,
"width_m", "height_m"}. Formations face Blender -Y: every passage runs along Y.

  rock_formation_<k>_LOD0   the near rocks, one mesh per rock, each with extras
                            {"acoustic": "rock"}; the game builds solid colliders from
                            these triangles, so the openings stay open
  rock_formation_<k>_LOD1   coarse far rocks, not acoustic

Reproduce (the Rock Generator is a user extension, so not --factory-startup):
  blender -b -P tools/blender/create_rock_formations.py -- <k>      one formation
  for k in 0 1 2 3 4 5; do blender -b -P tools/blender/create_rock_formations.py -- $k; done
Writes assets/props/rocks/rock_formation_<k>/, assets-src/blender/props/rocks/ and
previews/props/rocks/rock_formation_<k>/.
"""
from pathlib import Path
import json
import math
import random
import sys

import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_boulders import (GRANITE, SUNK, decimated, enable_rock_generator,  # noqa: E402
                             make_rock)
from create_burbler import linear  # noqa: E402
from module_kit import ModuleKit  # noqa: E402

ROUND = dict(deform=2.5, rough=1.0, smooth_it=3)       # the grotto's rounded granite
CRAGGY = dict(deform=3.5, rough=1.4, smooth_it=2)      # fins and small rocks, more broken
NEAR_TRIANGLES = 4000    # per rock, at most: about 500 per metre of its longest side
FAR_TRIANGLES = 240      # per rock; a bored rock keeps more, so its hole survives
FAR_BORED_TRIANGLES = 600

# Each formation: a role, its rocks and its openings.
#   rock:    (seed, size (x, y, z) m, shape, location (x, y, z) of its ground origin,
#             yaw, roll about Y, in degrees)
#   window:  bores an ellipsoid through rock `rock`: centre (x, z), half width, half height
#   opening: a socket (x, z) at the passage's middle, its width and height in metres
FORMATIONS = [
    {"role": "a pile of three boulders, no opening",
     "rocks": [(61, (5.0, 4.5, 3.8), ROUND, (0.0, 0.0, 0.0), 10, 0),
               (67, (3.5, 3.0, 2.6), ROUND, (3.4, 1.0, 0.0), 35, 0),
               (71, (2.2, 2.0, 1.6), CRAGGY, (-2.9, -1.6, 0.0), -20, 0)]},
    {"role": "a scatter of five rocks over about 12 m, no opening",
     "rocks": [(113, (2.6, 2.2, 1.8), ROUND, (0.0, 0.0, 0.0), 0, 0),
               (127, (1.6, 1.4, 1.1), CRAGGY, (3.6, 2.2, 0.0), 40, 0),
               (131, (2.0, 1.7, 1.4), ROUND, (-4.2, 1.4, 0.0), -25, 0),
               (137, (1.2, 1.0, 0.8), CRAGGY, (1.8, -3.4, 0.0), 70, 0),
               (139, (1.4, 1.3, 1.0), CRAGGY, (-2.0, -4.1, 0.0), 15, 0)]},
    {"role": "a window rock: a slab bored through near the ground",
     "rocks": [(73, (9.0, 4.5, 6.0), dict(deform=3.0, rough=1.3, smooth_it=2), (0.0, 0.0, 0.0), 0, 0),
               (151, (2.0, 1.8, 1.4), CRAGGY, (5.6, -2.4, 0.0), 30, 0)],
     "windows": [(0, (0.0, 1.9), 1.2, 1.5)],
     "openings": [((0.0, 1.6), 2.4, 3.0)]},
    {"role": "a slot: two leaning fins with a narrow gap between",
     "rocks": [(79, (3.5, 5.0, 7.0), CRAGGY, (-2.35, 0.0, 0.0), -8, 3),
               (83, (3.0, 4.5, 6.0), CRAGGY, (2.05, 0.3, 0.0), 12, -3),
               (157, (1.8, 1.6, 1.2), CRAGGY, (-4.6, -2.6, 0.0), 0, 0)],
     "openings": [((0.0, 1.5), 1.0, 4.0)]},
    {"role": "a bridge: a capstone across two supports, a passage under it",
     "rocks": [(89, (4.0, 4.0, 3.5), ROUND, (-2.85, 0.0, 0.0), 5, 0),
               (97, (3.5, 4.0, 3.2), ROUND, (2.6, 0.2, 0.0), -10, 0),
               (101, (8.0, 4.0, 2.4), ROUND, (-0.1, 0.1, 2.75), 3, -3)],
     "openings": [((0.0, 1.2), 1.6, 2.3)]},
    {"role": "a tor: a great boulder bored through, and a slot beside it",
     "rocks": [(103, (10.0, 6.0, 6.0), dict(deform=3.5, rough=1.3, smooth_it=2), (0.0, 0.0, 0.0), 0, 0),
               (107, (4.0, 4.0, 5.0), ROUND, (7.9, 0.5, 0.0), 15, -5),
               (109, (2.0, 2.0, 1.5), CRAGGY, (-6.2, -3.4, 0.0), 0, 0)],
     "windows": [(0, (-1.5, 1.9), 1.4, 1.7)],
     "openings": [((-1.5, 1.7), 2.8, 3.4), ((5.45, 1.6), 0.9, 3.5)]},
]


def bore(rock, location, centre, half_width, half_height, seed):
    """Bore an elliptical tunnel along Y through `rock`, which stands at `location`."""
    bpy.ops.mesh.primitive_uv_sphere_add(segments=32, ring_count=16)
    cutter = bpy.context.view_layer.objects.active
    cutter.scale = (half_width, 12.0, half_height)
    cutter.location = (centre[0] - location[0], 0.0, centre[1] - location[2])
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    # Weathered, not drilled: push each ring of the cutter in and out by seeded noise,
    # smooth along the tunnel, so the window's edge is irregular.
    rng = random.Random(seed)
    phases = [rng.uniform(0, math.tau) for _ in range(3)]
    for v in cutter.data.vertices:
        rel = v.co - Vector((cutter.location.x + centre[0] - location[0], 0.0,
                             centre[1] - location[2]))
        a = math.atan2(rel.z, rel.x)
        wobble = 1.0 + 0.12 * math.sin(3 * a + phases[0]) + 0.07 * math.sin(5 * a + phases[1]) \
            + 0.08 * math.sin(0.4 * v.co.y + phases[2])
        v.co.x = centre[0] - location[0] + rel.x * wobble
        v.co.z = centre[1] - location[2] + rel.z * wobble
    cutter.rotation_euler = (0, 0, 0)
    bpy.context.scene.collection.objects.link(rock)
    mod = rock.modifiers.new("Bore", "BOOLEAN")
    mod.operation = "DIFFERENCE"
    mod.solver = "EXACT"
    mod.object = cutter
    bpy.context.view_layer.objects.active = rock
    bpy.ops.object.modifier_apply(modifier=mod.name)
    bpy.context.scene.collection.objects.unlink(rock)
    bpy.data.objects.remove(cutter)
    # The boolean gives the tunnel's faces the cutter's material slot; one granite for all.
    rock.data.materials.clear()
    for poly in rock.data.polygons:
        poly.material_index = 0


class Formation(ModuleKit):
    folder = "props/rocks"

    def __init__(self, index):
        self.index = index
        self.spec = FORMATIONS[index]
        super().__init__("RockFormation", f"rock_formation_{index}", hp=1)
        for key in ("module_kind", "scale"):
            del self.root[key]
        self.root["kind"] = "prop"
        self.root["family"] = "rock_formation"
        self.root["variant"] = index
        self.granite = self.material("Granite", linear(GRANITE), roughness=0.9)

    def build(self):
        enable_rock_generator()
        windows = {w[0]: w[1:] for w in self.spec.get("windows", [])}
        fars, triangles = [], {"near": 0, "far": 0}
        lo, hi = Vector((1e9,) * 3), Vector((-1e9,) * 3)
        for r, (seed, size, shape, location, yaw, roll) in enumerate(self.spec["rocks"]):
            near = make_rock(seed, size, shape)
            if r in windows:
                # The bore is in the rock's own frame; formations bore only unturned rocks.
                assert yaw == 0 and roll == 0, "bore only an unrotated rock"
                bore(near, location, *windows[r], seed)
            near.name = near.data.name = f"Rock_{r}"
            far = near.copy()
            far.data = near.data.copy()
            far.name = far.data.name = f"Rock_{r}_Far"
            for obj in (near, far):
                self.link(obj, self.granite)
                bpy.context.scene.collection.objects.link(obj)
            decimated(near, max(800, min(NEAR_TRIANGLES, int(500 * max(size)))))
            decimated(far, FAR_BORED_TRIANGLES if r in windows else FAR_TRIANGLES)
            for obj in (near, far):
                bpy.context.scene.collection.objects.unlink(obj)
                obj.location = location
                obj.rotation_mode = "XYZ"
                obj.rotation_euler = (0.0, math.radians(roll), math.radians(yaw))
            near["acoustic"] = "rock"
            fars.append(far)
            for key, obj in (("near", near), ("far", far)):
                triangles[key] += sum(len(p.vertices) - 2 for p in obj.data.polygons)
            for corner in (Vector(c) for c in near.bound_box):
                world = near.matrix_basis @ corner
                lo = Vector(map(min, lo, world))
                hi = Vector(map(max, hi, world))
        openings = []
        for k, ((x, z), width, height) in enumerate(self.spec.get("openings", [])):
            self.socket(f"Opening_{k}", (x, 0.0, z), resource="rock_opening", opening=k,
                        width_m=width, height_m=height)
            openings.append({"socket": f"Opening_{k}", "at": [x, 0.0, z],
                             "width_metres": width, "height_metres": height})
        self.lod_levels(fars)
        size = hi - lo
        profile = {"id": self.asset_id, "model": f"{self.model_ref}#Scene0", "kind": "prop",
                   "family": "rock_formation", "variant": self.index, "role": self.spec["role"],
                   "rocks": [{"mesh": f"Rock_{r}", "seed": s, "size_metres": list(sz), "at": list(at),
                              "bored": r in windows}
                             for r, (s, sz, _, at, _, _) in enumerate(self.spec["rocks"])],
                   "openings": openings,
                   "size_metres": {"width": round(size.x, 2), "depth": round(size.y, 2),
                                   "height": round(hi.z, 2)},
                   "sunk_fraction": SUNK, "acoustic": "rock (near level)",
                   "triangles": triangles,
                   "licence": "CC0-1.0: generated by tools/blender/create_rock_formations.py "
                              "with Blender's Rock Generator add-on (GPL tool; the output is ours)"}
        reach = max(size.x, size.y, hi.z)

        def daylight():
            sun = bpy.data.objects.new("PreviewSun", bpy.data.lights.new("PreviewSun", "SUN"))
            sun.data.energy = 3.5
            sun.rotation_euler = (math.radians(50), 0, math.radians(-35))
            self.scene.collection.objects.link(sun)
            self.scene.world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.4, 0.5, 0.7, 1)
        centre = (lo + hi) / 2
        # Looking along the passages (+Y), a little off axis, so the openings show daylight.
        report = self.finish(profile, aim=(centre.x, 0, hi.z * 0.35),
                             camera_location=(centre.x + reach * 0.25, -reach * 1.7, hi.z * 0.45),
                             before_render=daylight)
        report.update(triangles=triangles, openings=len(openings))
        return report


def main():
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if args == ["list"]:
        print(json.dumps([f["role"] for f in FORMATIONS], indent=1))
        return
    for index in [int(a) for a in args] or range(len(FORMATIONS)):
        print(json.dumps(Formation(index).build()))


if __name__ == "__main__":
    main()
