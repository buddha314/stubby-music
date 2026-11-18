"""Granite boulders for the echo grotto (add-echo-grotto), after the rounded, weathered
monzogranite of the Mojave (Joshua Tree): big smooth boulders leaning on each other, a
small grotto at the base of the cluster (the user's reference, 2026-11-17).

Each boulder comes from Blender's Rock Generator add-on (Extra Mesh Objects extension,
`mesh.add_mesh_rock`) at a fixed seed, its subdivision and displacement modifiers
applied (their textures are Blender-only), scaled to its part of the cluster, sunk a
little into the ground and decimated. The add-on is GPL; its output geometry is ours,
and these models are CC0.

  boulder_<k>_LOD0   the near mesh; its mesh node carries extras {"acoustic": "rock"},
                     so the game marks it as acoustic rock (Steam Audio's rock preset)
  boulder_<k>_LOD1   a coarse far mesh, not acoustic

LAYOUT is the cluster: where each boulder sits (Blender metres, Z up, the grotto's mouth
facing -Y) so that the leaning top boulder roofs a small alcove at the base. The scene
placement and the cluster preview both read it (`-- layout` prints it as JSON).

Reproduce (the add-on is a user extension, so not --factory-startup):
  blender -b -P tools/blender/create_boulders.py            all five boulders
  blender -b -P tools/blender/create_boulders.py -- 2       one boulder
  blender -b -P tools/blender/create_boulders.py -- cluster the cluster preview
Writes assets/props/rocks/boulder_<k>/, assets-src/blender/props/rocks/ and
previews/props/rocks/.
"""
from pathlib import Path
import json
import math
import sys

import addon_utils
import bmesh
import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_burbler import linear  # noqa: E402
from module_kit import PROJECT, ModuleKit  # noqa: E402

ROCK_ADDON = "bl_ext.blender_org.extra_mesh_objects"
# Weathered granite, toned down for the bleak opening (style guide: hazy, desaturated).
GRANITE = "#9f8a6e"
LOD0_TRIANGLES = 6000
LOD1_TRIANGLES = 240
SUNK = 0.12            # the fraction of each boulder's height below the ground

# (seed, size in metres (x, y, z), Rock Generator shape) per boulder. Rounded granite:
# low roughness and deformation, smoothed (the add-on's River Rock, gentler).
BOULDERS = [
    (11, (9.0, 7.0, 6.5), dict(deform=2.5, rough=1.0, smooth_it=3)),   # back wall
    (23, (7.0, 6.0, 5.5), dict(deform=2.5, rough=1.0, smooth_it=3)),   # left
    (37, (6.5, 6.0, 5.0), dict(deform=3.0, rough=1.2, smooth_it=3)),   # right
    (41, (8.0, 6.0, 3.6), dict(deform=2.0, rough=0.8, smooth_it=3)),   # the leaning top
    (53, (3.0, 2.5, 2.0), dict(deform=3.0, rough=1.2, smooth_it=2)),   # a small rock in front
]

# The cluster: (boulder, location (x, y, z) of its origin at ground level, yaw, tilt
# about X and roll about Y, in degrees). The top boulder leans across the left and right
# ones, roofing an alcove about 6 m wide, 4 m deep and 3 m high in front of the back wall.
LAYOUT = [
    (0, (0.0, 4.2, 0.0), 8.0, 0.0, 0.0),
    (1, (-5.6, 0.6, 0.0), -14.0, 0.0, 0.0),
    (2, (5.2, 0.9, 0.0), 21.0, 0.0, 0.0),
    (3, (-0.4, 0.9, 3.6), -6.0, -4.0, 8.0),
    (4, (7.4, -4.2, 0.0), 40.0, 0.0, 0.0),
]


def enable_rock_generator():
    addon_utils.enable(ROCK_ADDON, default_set=True)
    if not hasattr(bpy.ops.mesh, "add_mesh_rock"):
        raise RuntimeError(f"Rock Generator not found: install the {ROCK_ADDON} extension")


def decimated(obj, triangles):
    tris = sum(len(p.vertices) - 2 for p in obj.data.polygons)
    if tris > triangles:
        mod = obj.modifiers.new("Decimate", "DECIMATE")
        mod.ratio = triangles / tris
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.modifier_apply(modifier=mod.name)
    for p in obj.data.polygons:
        p.use_smooth = True
    return obj


class Boulder(ModuleKit):
    folder = "props/rocks"

    def __init__(self, index):
        self.index = index
        self.seed, self.size, self.shape = BOULDERS[index]
        super().__init__("Boulder", f"boulder_{index}", hp=1)
        for key in ("module_kind", "scale"):
            del self.root[key]
        self.root["kind"] = "prop"
        self.root["family"] = "boulder"
        self.root["variant"] = index
        self.granite = self.material("Granite", linear(GRANITE), roughness=0.9)

    def rock(self):
        """One Rock Generator rock, baked and fitted to this boulder's size."""
        work = bpy.context.scene
        bpy.ops.mesh.add_mesh_rock(num_of_rocks=1, use_random_seed=False, user_seed=self.seed,
                                   use_generate=True, detail=3, display_detail=3,
                                   smooth_fac=2.0, **self.shape)
        rock = bpy.context.view_layer.objects.active
        for mod in list(rock.modifiers):
            bpy.ops.object.modifier_apply(modifier=mod.name)
        bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
        # Fit the size, then sit it on the ground with a little of it buried.
        dims = rock.dimensions
        scale = Vector([t / max(d, 1e-6) for t, d in zip(self.size, dims)])
        bm = bmesh.new()
        bm.from_mesh(rock.data)
        lo = Vector([min(v.co[i] for v in bm.verts) for i in range(3)])
        hi = Vector([max(v.co[i] for v in bm.verts) for i in range(3)])
        centre = (lo + hi) / 2
        for v in bm.verts:
            v.co = Vector(((v.co.x - centre.x) * scale.x, (v.co.y - centre.y) * scale.y,
                           (v.co.z - lo.z) * scale.z - SUNK * self.size[2]))
        bm.to_mesh(rock.data)
        bm.free()
        rock.data.materials.clear()
        for coll in list(rock.users_collection):
            coll.objects.unlink(rock)
        work.view_layers[0].update()
        return rock

    def build(self):
        enable_rock_generator()
        near = self.rock()
        near.name = near.data.name = f"Boulder_{self.index}"
        far = near.copy()
        far.data = near.data.copy()
        far.name = far.data.name = f"Boulder_{self.index}_Far"
        for obj in (near, far):
            self.link(obj, self.granite)
            bpy.context.scene.collection.objects.link(obj)
        decimated(near, LOD0_TRIANGLES)
        decimated(far, LOD1_TRIANGLES)
        for obj in (near, far):
            bpy.context.scene.collection.objects.unlink(obj)
        near["acoustic"] = "rock"
        self.lod_levels([far])
        tris = {name: sum(len(p.vertices) - 2 for p in o.data.polygons)
                for name, o in (("near", near), ("far", far))}
        w, d, h = self.size
        profile = {"id": self.asset_id, "model": f"{self.model_ref}#Scene0", "kind": "prop",
                   "family": "boulder", "variant": self.index, "seed": self.seed,
                   "size_metres": {"width": w, "depth": d, "height": h},
                   "sunk_fraction": SUNK, "acoustic": "rock (near level)",
                   "triangles": tris,
                   "licence": "CC0-1.0: generated by tools/blender/create_boulders.py with "
                              "Blender's Rock Generator add-on (GPL tool; the output is ours)"}
        reach = max(w, d, h)

        def daylight():
            # The kit lights small modules; a boulder needs the sun. Previews only: this
            # runs after the export.
            sun = bpy.data.objects.new("PreviewSun", bpy.data.lights.new("PreviewSun", "SUN"))
            sun.data.energy = 3.5
            sun.rotation_euler = (math.radians(50), 0, math.radians(-35))
            self.scene.collection.objects.link(sun)
            self.scene.world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.4, 0.5, 0.7, 1)
        report = self.finish(profile, aim=(0, 0, h * 0.4),
                             camera_location=(reach * 0.9, -reach * 1.9, h * 0.9),
                             before_render=daylight)
        report.update(triangles=tris)
        return report


def cluster():
    """Render the assembled grotto from its mouth, from the exported boulders."""
    scene = bpy.context.scene
    for obj in list(scene.objects):   # the startup scene's cube, light and camera
        bpy.data.objects.remove(obj)
    for k, location, yaw, tilt, roll in LAYOUT:
        before = set(bpy.data.objects)
        bpy.ops.import_scene.gltf(filepath=str(PROJECT / f"assets/props/rocks/boulder_{k}/boulder_{k}.glb"))
        for obj in set(bpy.data.objects) - before:
            if obj.name.endswith("_LOD1") or obj.parent and obj.parent.name.endswith("_LOD1"):
                obj.hide_render = True
            if obj.parent is None:
                obj.location = location
                obj.rotation_mode = "XYZ"
                obj.rotation_euler = (math.radians(tilt), math.radians(roll), math.radians(yaw))
    bpy.ops.mesh.primitive_plane_add(size=60, location=(0, 0, 0))
    ground = bpy.context.object
    mat = bpy.data.materials.new("Sand")
    mat.use_nodes = True
    mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (*linear("#8c7656"), 1)
    ground.data.materials.append(mat)
    sun = bpy.data.objects.new("Sun", bpy.data.lights.new("Sun", "SUN"))
    sun.data.energy = 4.0
    sun.rotation_euler = (math.radians(50), 0, math.radians(-35))
    scene.collection.objects.link(sun)
    world = bpy.data.worlds.new("Sky")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.45, 0.55, 0.75, 1)
    scene.world = world
    out = PROJECT / "previews/props/rocks"
    out.mkdir(parents=True, exist_ok=True)
    views = (("grotto", (4.0, -22.0, 4.5)), ("grotto_inside", (0.6, -4.5, 1.7)), ("grotto_top", (0.0, 0.5, 40.0)))
    for name, eye in views:
        camera = bpy.data.objects.new(name, bpy.data.cameras.new(name))
        camera.location = eye
        aim = {"grotto": Vector((0, 2.5, 2.8)), "grotto_inside": Vector((-0.4, 3.0, 2.0)),
               "grotto_top": Vector((0, 0.5, 0))}[name]
        camera.rotation_euler = (aim - camera.location).to_track_quat("-Z", "Y").to_euler()
        camera.data.lens = {"grotto": 28, "grotto_inside": 18, "grotto_top": 35}[name]
        scene.collection.objects.link(camera)
        scene.camera = camera
        scene.render.engine = "CYCLES"
        scene.cycles.samples = 48
        scene.render.resolution_x, scene.render.resolution_y = 1280, 720
        scene.view_settings.view_transform = "AgX"
        scene.render.filepath = str(out / f"{name}.png")
        bpy.ops.render.render(write_still=True)


def main():
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if args == ["layout"]:
        print(json.dumps([{"boulder": k, "location": loc, "yaw_degrees": yaw, "tilt_degrees": tilt,
                           "roll_degrees": roll} for k, loc, yaw, tilt, roll in LAYOUT]))
        return
    if args == ["cluster"]:
        cluster()
        return
    indices = [int(a) for a in args] or range(len(BOULDERS))
    for index in indices:
        print(json.dumps(Boulder(index).build()))


main()
