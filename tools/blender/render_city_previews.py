"""Preview the City kit's exported models: import each .glb (which also proves it loads),
render it in Eevee at a three-quarter view, and tile contact sheets.

  blender -b --factory-startup -P tools/blender/render_city_previews.py [-- city_tree ...]

Writes previews/structures/city/sheet_<n>.png (with arguments: <id>.png for each piece named).
"""
from pathlib import Path
import math
import shutil
import subprocess
import sys

import bpy
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[2]
MODELS = ROOT / "assets/structures/city"
OUT = ROOT / "previews/structures/city"


def clear():
    for o in list(bpy.data.objects):
        bpy.data.objects.remove(o, do_unlink=True)
    for block in (bpy.data.meshes, bpy.data.materials):
        for b in list(block):
            if b.users == 0:
                block.remove(b)


def stage(scene):
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.resolution_x = scene.render.resolution_y = 384
    scene.view_settings.view_transform = "AgX"
    scene.render.film_transparent = False
    world = scene.world or bpy.data.worlds.new("W")
    scene.world = world
    world.use_nodes = True
    bg = world.node_tree.nodes["Background"]
    bg.inputs["Color"].default_value = (0.32, 0.55, 0.56, 1)   # teal day sky, roughly
    bg.inputs["Strength"].default_value = 0.9


def render(aid):
    clear()
    scene = bpy.context.scene
    bpy.ops.import_scene.gltf(filepath=str(MODELS / aid / f"{aid}.glb"))
    far = bpy.data.objects.get(f"{aid}_LOD1")
    if far:
        for o in [far] + list(far.children_recursive):
            o.hide_render = True
    meshes = [o for o in scene.objects if o.type == "MESH" and not o.hide_render]
    lo = Vector((1e9,) * 3)
    hi = Vector((-1e9,) * 3)
    for o in meshes:
        for c in o.bound_box:
            w = o.matrix_world @ Vector(c)
            lo = Vector(map(min, lo, w))
            hi = Vector(map(max, hi, w))
    centre = (lo + hi) / 2
    radius = max((hi - lo).length / 2, 0.3)
    sun = bpy.data.objects.new("Sun", bpy.data.lights.new("Sun", "SUN"))
    sun.data.energy = 4.0
    sun.data.color = (1.0, 0.93, 0.85)
    sun.rotation_euler = (math.radians(50), 0, math.radians(-35))
    scene.collection.objects.link(sun)
    cam = bpy.data.objects.new("Cam", bpy.data.cameras.new("Cam"))
    cam.data.lens = 50
    scene.collection.objects.link(cam)
    d = Vector((0.55, -1.0, 0.55)).normalized()
    cam.location = centre + d * radius * 3.3
    cam.rotation_euler = (centre - cam.location).to_track_quat("-Z", "Y").to_euler()
    cam.data.clip_end = 2000
    scene.camera = cam
    scene.render.filepath = str(OUT / f"{aid}.png")
    bpy.ops.render.render(write_still=True)


def main():
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    ids = args or sorted(p.name for p in MODELS.glob("city_*") if p.is_dir())
    OUT.mkdir(parents=True, exist_ok=True)
    stage(bpy.context.scene)
    for aid in ids:
        render(aid)
        print("rendered", aid, flush=True)
    magick = shutil.which("magick")
    if magick and not args:
        for old in OUT.glob("sheet_*.png"):
            old.unlink()
        for n in range(0, len(ids), 24):
            cmd = [magick, "montage"]
            for aid in ids[n:n + 24]:
                cmd += ["-label", aid.removeprefix("city_"), str(OUT / f"{aid}.png")]
            cmd += ["-tile", "6x4", "-geometry", "256x256+4+4", "-background", "#15161a", "-fill", "#d8d4cc",
                    "-pointsize", "14", str(OUT / f"sheet_{n // 24 + 1}.png")]
            subprocess.run(cmd, check=True)
        for aid in ids:   # keep the repository light: the sheets carry the pieces
            (OUT / f"{aid}.png").unlink(missing_ok=True)


if __name__ == "__main__":
    main()
