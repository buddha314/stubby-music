"""Style sheet: every exported module, under every lighting key.

Assets must read across the whole arc of the game: from the bleak, hazy opening to
the bloomed world (solarpunk by day, lunarpunk by night), at every time of day. This
renders the module lineup under each key in assets/lighting/lighting_keys.json and
tiles them into docs/art/style_sheet.png (rows: light quality 0 and 1; columns: time of day).

The composition matches the opening: the player faces the modules with the sun low
behind them, so faceplates are backlit. That is the hardest case for legibility.

Reproduce: blender -b --factory-startup -P tools/blender/render_style_sheet.py [-- bleak:day ...]
Writes docs/art/keys/<mood>_<time>.png and docs/art/style_sheet.png (needs ImageMagick).
"""
from pathlib import Path
import json
import math
import shutil
import subprocess
import sys

import bpy
from mathutils import Vector

PROJECT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT / "tools/blender"))
from module_kit import model_file  # noqa: E402
KEYS = json.loads((PROJECT / "assets/lighting/lighting_keys.json").read_text())
MODELS = ["square_vco", "saw_vco", "triangle_vco", "sine_vco", "clock_lamp"]
SPACING = 0.85
SHOW_BPM = "120"
DIGITS = {"0": "abcdef", "1": "bc", "2": "abdeg", "3": "abcdg", "4": "bcfg", "5": "acdfg",
          "6": "acdefg", "7": "abc", "8": "abcdefg", "9": "abcdfg"}
# Render scale: lux to Blender sun strength, and the exposure the bleak day key is
# authored against. Emission is compensated for exposure, then scaled by emissive_gain.
LUX_PER_SUN_UNIT = 2400.0
REFERENCE_EV = 10.5
HAZE_RADIUS = 30.0
BLOOM_GLOW = 2.0
HAZE_GAIN = 4.0  # the box is shallower than the game's view distance


def linear(srgb):
    return tuple(c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4 for c in srgb)


def build_lineup(scene):
    emissive = []  # (principled node, authored strength)
    for i, name in enumerate(MODELS):
        before = set(bpy.data.objects)
        with bpy.context.temp_override(scene=scene, view_layer=scene.view_layers[0]):
            bpy.ops.import_scene.gltf(filepath=str(model_file(name)))
        x = (i - (len(MODELS) - 1) / 2) * SPACING
        for obj in set(bpy.data.objects) - before:
            if obj.name not in scene.objects:
                scene.collection.objects.link(obj)
            if obj.parent is None:
                obj.location.x += x
            # Show a reading on the clock instead of every segment.
            if obj.name.startswith("Display_d"):
                digit, seg = int(obj.name[9]), obj.name[11]
                obj.hide_render = seg not in DIGITS[SHOW_BPM[digit]]
    for mat in bpy.data.materials:
        node = mat.node_tree.nodes.get("Principled BSDF") if mat.use_nodes else None
        if node and node.inputs["Emission Strength"].default_value > 0:
            emissive.append((node, node.inputs["Emission Strength"].default_value))

    ground_mat = bpy.data.materials.new("StyleSheet_Ground")
    ground_mat.use_nodes = True
    ground_mat.node_tree.nodes["Principled BSDF"].inputs["Roughness"].default_value = 0.95
    size = 60.0
    mesh = bpy.data.meshes.new("StyleSheet_Ground")
    mesh.from_pydata([(-size, -size, 0), (size, -size, 0), (size, size, 0), (-size, size, 0)], [], [(0, 1, 2, 3)])
    ground = bpy.data.objects.new("StyleSheet_Ground", mesh)
    ground.data.materials.append(ground_mat)
    scene.collection.objects.link(ground)

    sun = bpy.data.objects.new("StyleSheet_Sun", bpy.data.lights.new("StyleSheet_Sun", "SUN"))
    sun.data.angle = math.radians(1.5)
    scene.collection.objects.link(sun)

    world = bpy.data.worlds.new("StyleSheet_World")
    world.use_nodes = True
    scene.world = world

    # Haze lives in a bounded box, not the world: a world volume sits between the
    # sun and everything over infinite distance and absorbs it all.
    haze_mat = bpy.data.materials.new("StyleSheet_Haze")
    haze_mat.use_nodes = True
    nodes, links = haze_mat.node_tree.nodes, haze_mat.node_tree.links
    nodes.remove(nodes["Principled BSDF"])
    volume = nodes.new("ShaderNodeVolumePrincipled")
    links.new(volume.outputs["Volume"], nodes["Material Output"].inputs["Volume"])
    r = HAZE_RADIUS
    haze_mesh = bpy.data.meshes.new("StyleSheet_Haze")
    haze_mesh.from_pydata(
        [(x, y, z) for x in (-r, r) for y in (-r, r) for z in (-0.1, r)], [],
        [(0, 1, 3, 2), (4, 6, 7, 5), (0, 4, 5, 1), (2, 3, 7, 6), (0, 2, 6, 4), (1, 5, 7, 3)])
    haze = bpy.data.objects.new("StyleSheet_Haze", haze_mesh)
    haze.data.materials.append(haze_mat)
    scene.collection.objects.link(haze)

    # Stand-in bloom: accent-lit stalks behind the lineup, shown only in bloomed moods,
    # until real flora exists. They prove the accents read against each key.
    bloom = []
    for i, (x, y, h) in enumerate([(-3.8, 7.0, 1.6), (-2.2, 9.5, 2.4), (-0.4, 6.5, 1.1), (1.3, 8.4, 2.0),
                                   (2.6, 6.8, 1.4), (4.2, 9.0, 2.6), (-5.2, 10.5, 2.2), (5.6, 7.2, 1.2)]):
        mat = bpy.data.materials.new(f"StyleSheet_Bloom{i}")
        mat.use_nodes = True
        bpy.ops.mesh.primitive_cone_add(vertices=6, radius1=0.08, radius2=0.015, depth=h, location=(x, y, h / 2))
        stalk = bpy.context.active_object
        bpy.ops.mesh.primitive_uv_sphere_add(radius=0.11, location=(x, y, h), segments=12, ring_count=6)
        bud = bpy.context.active_object
        for obj in (stalk, bud):
            obj.data.materials.append(mat)
            for coll in obj.users_collection:
                coll.objects.unlink(obj)
            scene.collection.objects.link(obj)
        bloom.append((mat, [stalk, bud], i))

    camera = bpy.data.objects.new("StyleSheet_Camera", bpy.data.cameras.new("StyleSheet_Camera"))
    camera.location = (0.0, -6.2, 1.55)
    camera.rotation_euler = (Vector((0.0, 0.0, 0.85)) - camera.location).to_track_quat("-Z", "Y").to_euler()
    camera.data.lens = 50
    scene.collection.objects.link(camera)
    scene.camera = camera

    scene.render.engine = "CYCLES"
    scene.cycles.samples = 48
    scene.cycles.volume_step_rate = 4.0
    scene.render.resolution_x, scene.render.resolution_y = 800, 400
    scene.view_settings.view_transform = "AgX"
    scene.view_settings.look = "AgX - Medium High Contrast"
    return emissive, ground_mat, sun, world, volume, bloom


def apply_key(scene, mood, key, emissive, ground_mat, sun, world, volume, bloom):
    ev_offset = REFERENCE_EV - key["exposure_ev100"]
    scene.view_settings.exposure = ev_offset
    ground_mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (
        *linear(mood["ground_tint"]), 1.0)
    sun.data.color = linear(key["sun_color"])
    sun.data.energy = key["sun_lux"] / LUX_PER_SUN_UNIT
    # Behind the modules (they face -Y), at the key's elevation.
    sun.rotation_euler = (math.radians(90 - key["sun_elevation_deg"]), 0.0, math.radians(200))
    bg = world.node_tree.nodes["Background"]
    bg.inputs["Color"].default_value = (*linear(key["ambient_color"]), 1.0)
    bg.inputs["Strength"].default_value = key["ambient_lux"] / 6000.0
    volume.inputs["Color"].default_value = (*linear(key["fog_color"]), 1.0)
    # Bevy's exponential fog is exp(-density * distance); a volume's extinction matches.
    volume.inputs["Density"].default_value = key["fog_density"] * HAZE_GAIN
    accents = mood.get("accents", [])[1:]  # the first is the modules' amber
    for mat, objs, i in bloom:
        for obj in objs:
            obj.hide_render = not accents
        if accents:
            node = mat.node_tree.nodes["Principled BSDF"]
            color = (*linear(accents[i % len(accents)]), 1.0)
            node.inputs["Base Color"].default_value = color
            node.inputs["Emission Color"].default_value = color
            node.inputs["Emission Strength"].default_value = BLOOM_GLOW * key["emissive_gain"] * 2 ** (-ev_offset)
    # Emission looks the same at every exposure unless gained: compensate, then gain.
    for node, authored in emissive:
        node.inputs["Emission Strength"].default_value = authored * key["emissive_gain"] * 2 ** (-ev_offset)


def main():
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    wanted = [tuple(a.split(":")) for a in args] or [(m, t) for m in KEYS["moods"] for t in KEYS["times"]]
    scene = bpy.data.scenes.new("StyleSheet")
    if bpy.context.window:
        bpy.context.window.scene = scene
    parts = build_lineup(scene)
    out = PROJECT / "docs/art/keys"
    out.mkdir(parents=True, exist_ok=True)
    for mood_name, time in wanted:
        mood = KEYS["moods"][mood_name]
        apply_key(scene, mood, mood["keys"][time], *parts)
        scene.render.filepath = str(out / f"{mood_name}_{time}.png")
        bpy.ops.render.render(write_still=True, scene=scene.name)

    magick = shutil.which("magick") or shutil.which("montage")
    tiles = [out / f"{m}_{t}.png" for m in KEYS["moods"] for t in KEYS["times"]]
    if magick and all(t.exists() for t in tiles):
        cmd = [magick, "montage"] if magick.endswith("magick") else [magick]
        for tile in tiles:
            cmd += ["-label", tile.stem.replace("_", " · "), str(tile)]
        cmd += ["-tile", f"{len(KEYS['times'])}x{len(KEYS['moods'])}", "-geometry", "+6+6",
                "-background", "#15161a", "-fill", "#d8d4cc", "-pointsize", "18",
                str(PROJECT / "docs/art/style_sheet.png")]
        subprocess.run(cmd, check=True)


if __name__ == "__main__":
    main()
