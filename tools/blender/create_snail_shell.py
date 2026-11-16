"""Snail shell: an empty, high-spired land-snail shell of four whorls (sync-snail-shell-sequencer,
ported from the Godot game's add-snail-shell-sequencer, wait-for-stubs d14edf0).

A collectible find on the mudflat; in a rack it mounts as the four-step Snail sequencer,
one step per whorl. Built as a helicospiral after Raup's coiling model: a circular
generating curve sweeps a logarithmic spiral that widens by W per whorl and slides down
the coiling axis (translation T), so each whorl overlaps the last and leaves a suture.
After the user's reference (2026-11-13): a turreted spire, glossy translucent brown.

Nodes (glTF):
  SnailShell_Root      extras: asset_id, whorls
  snail_shell_LOD0/1   the shell mesh at two densities (the import script makes them
                       visibility ranges)

Real size, about 3 cm tall; the game shows it at the profile's `world_scale`.

Writes assets/creatures/snail_shell/snail_shell.glb and asset.json,
assets-src/blender/creatures/snail_shell.blend and previews/creatures/snail_shell/.

Reproduce:
  ~/.local/bin/blender -b --factory-startup -P tools/blender/create_snail_shell.py
"""
import json
import math
import sys
from pathlib import Path

import bpy
from mathutils import Vector

PROJECT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT / "tools/blender"))
from export_gltf import export_scene  # noqa: E402

FOLDER = "creatures"
ASSET_ID = "snail_shell"
WHORLS = 4               # one per sequencer step
HEIGHT = 0.03            # apex to the aperture's lowest point, metres
W = 1.45                 # radius growth per whorl (Raup's W, per whorl here)
T = 6.0                  # translation down the axis per unit of radius (a tall spire)
APERTURE = 0.85          # the generating curve's horizontal semi-axis / spiral radius
APERTURE_TALL = 1.25     # its vertical semi-axis / spiral radius: whorls overlap when
                         # APERTURE_TALL * (1 + W) > T * (W - 1), which leaves sutures
WORLD_SCALE = 3.0        # shown 3x real size so a find is findable (as the bugs are)
LEVELS = [(64, 18), (16, 8)]  # (samples per whorl, segments round the tube) per LOD


MODEL = PROJECT / "assets" / FOLDER / ASSET_ID / f"{ASSET_ID}.glb"
MODEL_REF = f"{FOLDER}/{ASSET_ID}/{ASSET_ID}.glb"
PROFILE_DIR = PROJECT / "assets" / FOLDER / ASSET_ID
PREVIEWS = PROJECT / "previews" / FOLDER / ASSET_ID
SOURCE = PROJECT / "assets-src/blender" / FOLDER / f"{ASSET_ID}.blend"


def shell_points(per_whorl, around):
    """Rings of the swept tube, apex first, in shell units (scaled to HEIGHT later)."""
    rings = []
    turns = WHORLS * per_whorl
    for i in range(turns + 1):
        theta = 2 * math.pi * i / per_whorl
        r = W ** (theta / (2 * math.pi))
        centre = Vector((r * math.cos(theta), r * math.sin(theta), -T * r))
        # Frame: radial outwards, axis up; the generating circle lies in that plane.
        radial = Vector((math.cos(theta), math.sin(theta), 0.0))
        axis = Vector((0.0, 0.0, 1.0))
        ring = []
        for j in range(around):
            phi = 2 * math.pi * j / around
            ring.append(centre + r * (APERTURE * math.cos(phi) * radial + APERTURE_TALL * math.sin(phi) * axis))
        rings.append(ring)
    return rings


def build_mesh(name, per_whorl, around):
    rings = shell_points(per_whorl, around)
    flat = [p for ring in rings for p in ring]
    top = max(p.z for p in flat)
    bottom = min(p.z for p in flat)
    scale = HEIGHT / (top - bottom)
    verts = [((p.x) * scale, (p.y) * scale, (p.z - bottom) * scale) for p in flat]
    faces = []
    for i in range(len(rings) - 1):
        for j in range(around):
            a = i * around + j
            b = i * around + (j + 1) % around
            # Wound so normals face out of the tube (the targeted outline grows along them).
            faces.append((a, a + around, b + around, b))
    # Close the apex with a fan to a point.
    verts.append(tuple(sum(verts[j][k] for j in range(around)) / around for k in range(3)))
    apex = len(verts) - 1
    for j in range(around):
        faces.append((apex, j, (j + 1) % around))
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    for poly in mesh.polygons:
        poly.use_smooth = True
    return mesh


def shell_material():
    mat = bpy.data.materials.new("SnailShell_Shell")
    mat.use_nodes = True
    mat.use_backface_culling = False  # an open, thin shell: both sides show
    bsdf = mat.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = (0.2, 0.085, 0.035, 1.0)
    bsdf.inputs["Roughness"].default_value = 0.18
    bsdf.inputs["Coat Weight"].default_value = 0.6
    bsdf.inputs["Subsurface Weight"].default_value = 0.25
    mat.diffuse_color = (0.2, 0.085, 0.035, 1.0)
    return mat


def build():
    scene = bpy.data.scenes.new("SnailShellAsset")
    scene.unit_settings.system = "METRIC"
    collection = bpy.data.collections.new("STUBBY_SnailShell")
    scene.collection.children.link(collection)
    root = bpy.data.objects.new("SnailShell_Root", None)
    collection.objects.link(root)
    root["asset_id"] = ASSET_ID
    root["whorls"] = WHORLS
    material = shell_material()
    triangles = []
    for level, (per_whorl, around) in enumerate(LEVELS):
        node = bpy.data.objects.new(f"{ASSET_ID}_LOD{level}", None)
        collection.objects.link(node)
        node.parent = root
        mesh = build_mesh(f"SnailShell_LOD{level}", per_whorl, around)
        mesh.materials.append(material)
        obj = bpy.data.objects.new(f"SnailShell_Shell_LOD{level}", mesh)
        collection.objects.link(obj)
        obj.parent = node
        triangles.append(sum(len(p.vertices) - 2 for p in mesh.polygons))

    # Preview: lit from above on dark mud, as the opening is.
    world = bpy.data.worlds.new("SnailShell_PreviewWorld")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.05, 0.045, 0.04, 1)
    world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.6
    scene.world = world
    sun = bpy.data.objects.new("SnailShell_Sun", bpy.data.lights.new("SnailShell_Sun", "SUN"))
    sun.data.energy = 3.0
    sun.rotation_euler = (math.radians(40), 0, math.radians(30))
    scene.collection.objects.link(sun)
    camera = bpy.data.objects.new("SnailShell_Camera", bpy.data.cameras.new("SnailShell_Camera"))
    scene.collection.objects.link(camera)
    scene.camera = camera
    camera.data.clip_start = 0.002  # the shell is a few centimetres
    scene.render.resolution_x = scene.render.resolution_y = 512
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 24
    scene.view_settings.view_transform = "AgX"

    PREVIEWS.mkdir(parents=True, exist_ok=True)
    PROFILE_DIR.mkdir(parents=True, exist_ok=True)
    target = Vector((0, 0, HEIGHT * 0.5))
    for name, distance, lod in [("preview.png", 0.06, 0), ("preview_far.png", 0.5, 1)]:
        for level in range(len(LEVELS)):
            for obj in bpy.data.objects[f"{ASSET_ID}_LOD{level}"].children:
                obj.hide_render = level != lod
        camera.location = target + Vector((distance * 0.5, -distance, distance * 0.15))
        camera.rotation_euler = (target - camera.location).to_track_quat("-Z", "Y").to_euler()
        scene.render.filepath = str(PREVIEWS / name)
        bpy.ops.render.render(write_still=True, scene=scene.name)
    for level in range(len(LEVELS)):
        for obj in bpy.data.objects[f"{ASSET_ID}_LOD{level}"].children:
            obj.hide_render = False

    profile = {
        "id": ASSET_ID,
        "model": MODEL_REF,
        "kind": "find",
        "resource": "empty_snail_shell",
        "item": ASSET_ID,
        "whorls": WHORLS,
        "size_metres": {"height": HEIGHT},
        "world_scale": WORLD_SCALE,
        "coiling": {"model": "Raup helicospiral", "W_per_whorl": W, "T": T, "aperture": [APERTURE, APERTURE_TALL]},
        "triangles": triangles,
    }
    (PROFILE_DIR / "asset.json").write_text(json.dumps(profile, indent=2) + "\n")

    source = SOURCE
    source.parent.mkdir(parents=True, exist_ok=True)
    if bpy.app.background:
        if bpy.context.window:
            bpy.context.window.scene = scene
        for other in list(bpy.data.scenes):
            if other != scene:
                bpy.data.scenes.remove(other)
        bpy.ops.wm.save_as_mainfile(filepath=str(source), check_existing=False)
        export_scene(scene, MODEL)
    else:
        bpy.data.libraries.write(str(source), {scene}, fake_user=True)
    return {"model": str(MODEL), "triangles": triangles}


if __name__ == "__main__":
    print(build())
