"""Mock of the burbler's configuration HUD (add-mudpit-burbler 1.5).

Configuring a burbler opens a basic three-place Eurorack over the game, as a
Satisfactory machine screen does: the burbler's clock (fixed), a VCO slot (here a
Square VCO) and the volume module, in a small rack of titanium rails on a weathered
wood frame. This renders the rack straight on with a transparent background, then
lays it over a dimmed, blurred desert view with the screen's title and hints.

A look reference for the HUD, not a game asset: the game builds the screen itself.

Reproduce: blender -b --factory-startup -P tools/blender/render_hud_mock.py
Writes docs/art/hud_mock.png (needs ImageMagick).
"""
from pathlib import Path
import math
import shutil
import subprocess
import sys

import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_clock_lamp import DIGITS  # noqa: E402
from module_kit import HEIGHT, HP, PROJECT  # noqa: E402

PLACES = [("burbler_clock", 8), ("square_vco", 8), ("volume", 4)]
SHOW_BPM = " 90"
OUT = PROJECT / "docs/art/hud_mock.png"


def linear(hex_srgb):
    c = [int(hex_srgb[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    return tuple(v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4 for v in c)


def material(name, hex_srgb, metallic=0.0, roughness=0.5):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = (*linear(hex_srgb), 1.0)
    bsdf.inputs["Metallic"].default_value = metallic
    bsdf.inputs["Roughness"].default_value = roughness
    return mat


def box(scene, name, size, location, mat):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=location)
    obj = bpy.context.active_object
    obj.name = name
    obj.scale = size
    obj.data.materials.append(mat)
    return obj


def main():
    scene = bpy.context.scene
    for obj in list(scene.objects):
        bpy.data.objects.remove(obj)
    width = sum(hp for _, hp in PLACES) * HP
    x = -width / 2
    for asset, hp in PLACES:
        before = set(bpy.data.objects)
        bpy.ops.import_scene.gltf(filepath=str(PROJECT / f"assets/models/{asset}.glb"))
        for obj in set(bpy.data.objects) - before:
            if obj.parent is None:
                obj.location.x += x + hp * HP / 2
            if obj.name.startswith("Display_d"):
                digit, seg = int(obj.name[9]), obj.name[11]
                obj.hide_render = SHOW_BPM[digit] == " " or seg not in DIGITS[SHOW_BPM[digit]]
        x += hp * HP
    # Lamp mid-flash, as in the module preview.
    lamp = bpy.data.materials.get("BurblerClock_Lamp")
    if lamp:
        lamp.node_tree.nodes["Principled BSDF"].inputs["Emission Strength"].default_value = 4.0

    # The rack: titanium rails on a weathered wood frame (crude meets high-tech).
    titanium = material("HudRack_Titanium", "#b8bcc2", metallic=1.0, roughness=0.32)
    wood = material("HudRack_Wood", "#5a4a3a", roughness=0.9)
    iron = material("HudRack_Iron", "#5e3a22", metallic=0.5, roughness=0.85)
    for z in (0.02, HEIGHT - 0.02):
        box(scene, "Rail", (width + 0.12, 0.05, 0.045), (0, -0.2, z), titanium)
    for side in (-1, 1):
        box(scene, "Post", (0.09, 0.32, HEIGHT + 0.36), (side * (width / 2 + 0.1), 0.0, HEIGHT / 2 + 0.02), wood)
        for z in (0.02, HEIGHT - 0.02):
            box(scene, "Strap", (0.11, 0.34, 0.05), (side * (width / 2 + 0.1), 0.0, z), iron)

    key = bpy.data.objects.new("Key", bpy.data.lights.new("Key", "AREA"))
    key.data.energy, key.data.size, key.data.color = 260, 2.0, (1.0, 0.85, 0.7)
    key.location = (-1.2, -2.6, 1.9)
    key.rotation_euler = (Vector((0, 0, 0.65)) - key.location).to_track_quat("-Z", "Y").to_euler()
    scene.collection.objects.link(key)
    world = bpy.data.worlds.new("HudWorld")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.04, 0.045, 0.055, 1)
    world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.5
    scene.world = world

    camera = bpy.data.objects.new("Camera", bpy.data.cameras.new("Camera"))
    camera.data.type = "ORTHO"
    camera.data.ortho_scale = HEIGHT + 0.55
    camera.location = (0, -5.0, HEIGHT / 2 + 0.03)
    camera.rotation_euler = (math.pi / 2, 0, 0)
    scene.collection.objects.link(camera)
    scene.camera = camera

    scene.render.engine = "CYCLES"
    scene.cycles.samples = 64
    scene.render.film_transparent = True
    scene.render.resolution_x, scene.render.resolution_y = 1100, 1100
    scene.view_settings.view_transform = "AgX"
    scene.view_settings.look = "AgX - Medium High Contrast"
    rack = PROJECT / "docs/art/hud_rack.png"
    scene.render.filepath = str(rack)
    bpy.ops.render.render(write_still=True)

    magick = shutil.which("magick")
    backdrop = PROJECT / "docs/art/keys/bleak_dusk.png"
    if magick and backdrop.exists():
        subprocess.run([
            magick, str(backdrop), "-resize", "1920x1080^", "-gravity", "center", "-extent", "1920x1080",
            "-blur", "0x14", "-fill", "#0b0c10", "-colorize", "55%",
            # the HUD panel: a dark translucent card with a thin amber edge
            "-fill", "#15161ad0", "-stroke", "#ffb35066", "-strokewidth", "2",
            "-draw", "roundrectangle 470,70 1450,1010 18,18", "-stroke", "none",
            "(", str(rack), "-resize", "900x900", ")", "-gravity", "center", "-geometry", "+0+30",
            "-composite",
            "-gravity", "north", "-fill", "#e7e5dd", "-pointsize", "34", "-annotate", "+0+92",
            "MUDPIT BURBLER",
            "-fill", "#ffb350", "-pointsize", "20", "-annotate", "+0+138", "90 BPM  ·  configure",
            "-gravity", "south", "-fill", "#adb8d4", "-pointsize", "18", "-annotate", "+0+92",
            "drag a VCO from inventory into the slot      ·      Esc  close",
            str(OUT)], check=True)
        rack.unlink()


if __name__ == "__main__":
    main()
