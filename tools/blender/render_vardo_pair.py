"""The vardo and the cyber-cob together, the cob at the vardo's Hitch: the pair sheet
(add-vardo-and-cyber-cob-assets 4.1, 4.3). Renders the exported models as the game loads them.

Reproduce: blender -b --factory-startup -P tools/blender/render_vardo_pair.py
Writes previews/structures/vardo/pair_side.png, pair_three_quarter.png, pair_harness.png
and pair_sheet.png.
"""
from pathlib import Path
import math
import shutil
import subprocess

import bpy
from mathutils import Vector

PROJECT = Path(__file__).resolve().parents[2]
OUT = PROJECT / "previews/structures/vardo"


def load(path):
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=str(path))
    return [o for o in bpy.data.objects if o not in before]


bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
vardo = load(PROJECT / "assets/structures/vardo/vardo.glb")
cob = load(PROJECT / "assets/creatures/cyber_cob/cyber_cob.glb")
# Hide the far levels; stand the cob at the Hitch.
for obj in vardo:
    if "_LOD1" in obj.name or any("_LOD1" in p.name for p in obj.children_recursive if p) :
        pass
for obj in bpy.data.objects:
    if obj.name.startswith("vardo_LOD1"):
        for o in [obj] + list(obj.children_recursive):
            o.hide_render = True
hitch = next(o for o in vardo if o.name == "Hitch").matrix_world.translation
for obj in cob:
    if obj.parent is None:
        obj.location += hitch
arm = next(o for o in cob if o.type == "ARMATURE")
if arm.animation_data and bpy.data.actions.get("Idle"):
    arm.animation_data.action = bpy.data.actions["Idle"]
scene.frame_set(0)

world = bpy.data.worlds.new("PairWorld")
world.use_nodes = True
world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.05, 0.055, 0.065, 1)
world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.7
scene.world = world
sun = bpy.data.objects.new("Sun", bpy.data.lights.new("Sun", "SUN"))
sun.data.energy = 3.5
sun.rotation_euler = (math.radians(50), 0, math.radians(-40))
scene.collection.objects.link(sun)
ground = bpy.data.meshes.new("Ground")
ground.from_pydata([(-20, -20, 0), (20, -20, 0), (20, 20, 0), (-20, 20, 0)], [], [(0, 1, 2, 3)])
g = bpy.data.objects.new("Ground", ground)
mat = bpy.data.materials.new("GroundMat")
mat.use_nodes = True
mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (0.25, 0.17, 0.11, 1)
g.data.materials.append(mat)
scene.collection.objects.link(g)
cam = bpy.data.objects.new("Cam", bpy.data.cameras.new("Cam"))
scene.collection.objects.link(cam)
scene.camera = cam
scene.render.engine = "CYCLES"
scene.cycles.samples = 48
scene.view_settings.view_transform = "AgX"
shots = {
    "pair_side": ((-9.5, -2.2, 2.0), (0, -2.2, 1.2), 35, (1280, 640)),
    "pair_three_quarter": ((-6.5, -9.0, 3.2), (0, -2.4, 1.2), 32, (1024, 768)),
    "pair_harness": ((-2.6, -5.6, 1.6), (0, -3.6, 0.9), 35, (1024, 768)),
}
for name, (loc, aim, lens, (w, h)) in shots.items():
    cam.location = Vector(loc)
    cam.rotation_euler = (Vector(aim) - cam.location).to_track_quat("-Z", "Y").to_euler()
    cam.data.lens = lens
    scene.render.resolution_x, scene.render.resolution_y = w, h
    scene.render.filepath = str(OUT / f"{name}.png")
    bpy.ops.render.render(write_still=True)
magick = shutil.which("magick")
if magick:
    subprocess.run([magick, "montage", str(OUT / "pair_side.png"), str(OUT / "pair_three_quarter.png"),
                    str(OUT / "pair_harness.png"), "-tile", "3x1", "-geometry", "640x480+4+4",
                    "-background", "#15161a", str(OUT / "pair_sheet.png")], check=True)
