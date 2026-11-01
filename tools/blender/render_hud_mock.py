"""Mocks of a machine's configuration screen, built from the game's assets and the
machine's own rack configuration (add-rack-configurations, add-mudpit-burbler 1.5).

E within range of a machine opens its configuration screen over the game, as a
Satisfactory machine screen does: the world dims and blurs, and a panel shows the
machine's rack. Every machine has its own rack configurator: the `configure` block in
its profile lists its slots (component kind, width in HP, fixed or not). The rack is
assembled from the shared frame, slot outlines and labels (create_hud_rack.py). Here,
the mudpit burbler's, in two states:

  hud_mock_empty.png   construction: the built-in clock in place, the VCO and VCA
                       slots empty, each a labelled outline of what goes there; the
                       burbler's two spare cables hang on the rack's cable comb
  hud_mock_ready.png   built, not patched: Square VCO and volume mounted, cables still
                       on the comb, every patch pod (jack halo) lit to invite patching
  hud_mock.png         patched: clock GATE to volume TRIG, VCO OUT to volume IN, drawn
                       with the game's elastic cable rule (`create_patch_cable.cable_curve`);
                       the comb is empty and the pods rest

A look reference, not a game asset.

Reproduce: blender -b --factory-startup -P tools/blender/render_hud_mock.py
Writes docs/art/hud_mock.png and docs/art/hud_mock_empty.png (needs ImageMagick).
"""
from pathlib import Path
import json
import math
import shutil
import subprocess
import sys

import bmesh
import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_clock_lamp import DIGITS  # noqa: E402
from create_hud_rack import POST_INSET, RAIL_MARGIN  # noqa: E402
from create_patch_cable import CABLE_RADIUS, COLORS, cable_curve, tube  # noqa: E402
from module_kit import HEIGHT, HP, PROJECT, model_file  # noqa: E402

MACHINE = "mudpit_burbler"
# What the built mock mounts in each non-fixed place (the player's choice in play).
BUILT = {"vco": "square_vco", "volume": "volume"}
# (from place, port) -> (to place, port), cable colour
PATCH = [(("clock", "gate"), ("volume", "trig"), "verdigris"),
         (("vco", "out"), ("volume", "in"), "rust")]
SHOW_BPM = " 90"
DOCS = PROJECT / "docs/art"


def configuration():
    return json.loads((PROJECT / f"assets/machines/{MACHINE}/asset.json").read_text())["configure"]


def linear(hex_srgb):
    c = [int(hex_srgb[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    return tuple(v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4 for v in c)


def cable_material(colour):
    name = f"HudCable_{colour}"
    mat = bpy.data.materials.get(name)
    if mat is None:
        mat = bpy.data.materials.new(name)
        mat.use_nodes = True
        bsdf = mat.node_tree.nodes["Principled BSDF"]
        bsdf.inputs["Base Color"].default_value = (*linear(COLORS[colour]), 1.0)
        bsdf.inputs["Roughness"].default_value = 0.55
    return mat


def import_glb(asset):
    """Import an asset; return its new objects and its root(s)."""
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=str(model_file(asset)))
    new = set(bpy.data.objects) - before
    return new, [o for o in new if o.parent is None]


def base(obj):
    return obj.name.split(".")[0]


def build_rack(places, built):
    """The rack for a configuration: frame, then each place mounted or outlined."""
    width = sum(p["hp"] for p in places) * HP
    new, _ = import_glb("rack_frame")
    comb = {base(o): o for o in new if base(o).startswith("Comb_")}
    for obj in new:
        if base(obj) in ("RailTop", "RailBottom"):
            obj.scale.x = width + RAIL_MARGIN
        elif base(obj) == "PostL":
            obj.location.x = -(width / 2 + POST_INSET)
        elif base(obj) == "PostR":
            obj.location.x = width / 2 + POST_INSET
    ports = {}
    x = -width / 2
    for p in places:
        centre = x + p["hp"] * HP / 2
        module = p.get("module") if p["fixed"] else (BUILT.get(p["place"]) if built else None)
        if module:
            new, roots = import_glb(module)
            for root in roots:
                root.location.x += centre
            for obj in new:
                if base(obj).startswith("Port_"):
                    ports[(p["place"], base(obj)[5:])] = obj
                if base(obj).startswith("Display_d"):
                    digit, seg = int(base(obj)[9]), base(obj)[11]
                    obj.hide_render = SHOW_BPM[digit] == " " or seg not in DIGITS[SHOW_BPM[digit]]
        else:
            new, roots = import_glb(f"slot_outline_{p['hp']}hp")
            for root in roots:
                root.location.x += centre
            bpy.context.view_layer.update()
            anchor = next(o for o in new if base(o) == "Label")
            labels, _ = import_glb("slot_labels")
            for obj in labels:
                if base(obj).startswith("Label_") and base(obj) != f"Label_{p['kind']}":
                    for part in [obj] + list(obj.children_recursive):
                        part.hide_render = True
            next(o for o in labels if base(o) == f"Label_{p['kind']}").matrix_world = anchor.matrix_world.copy()
        x += p["hp"] * HP
    bpy.context.view_layer.update()
    return ports, comb


def plug_at(matrix, colour):
    """A patch plug placed at `matrix`; returns its cable attach point."""
    new, roots = import_glb("patch_plug")
    roots[0].matrix_world = matrix.copy()
    bpy.context.view_layer.update()
    for o in new:
        if o.type == "MESH" and base(o) == "PatchCable_Stub":
            o.hide_render = True
        if o.type == "MESH":
            for s in o.material_slots:
                if s.material and base(s.material) in ("PatchCable_Cable", "PatchCable_Collar"):
                    s.material = cable_material(colour)
    return next(o for o in new if base(o) == "Cable_attach").matrix_world.translation.copy()


def cable_mesh(points, colour):
    bm = bmesh.new()
    tube(bm, points, CABLE_RADIUS)
    mesh = bpy.data.meshes.new(f"Cable_{colour}")
    bm.to_mesh(mesh)
    bm.free()
    cable = bpy.data.objects.new(f"Cable_{colour}", mesh)
    cable.data.materials.append(cable_material(colour))
    for face in cable.data.polygons:
        face.use_smooth = True
    bpy.context.scene.collection.objects.link(cable)


def hang(comb, colours):
    """Spare cables on the comb: draped over a slot, both plugs hanging tip down."""
    for n, colour in enumerate(colours):
        a = plug_at(comb[f"Comb_{n}_PlugA"].matrix_world, colour)
        b = plug_at(comb[f"Comb_{n}_PlugB"].matrix_world, colour)
        top = comb[f"Comb_{n}"].matrix_world.translation
        # Quadratic Bezier whose control point lifts the curve over the slot.
        c = 2 * top - (a + b) / 2 + Vector((0, 0, 0.03))
        n_pts = 32
        pts = [a * (1 - t) ** 2 + c * 2 * (1 - t) * t + b * t * t for t in (i / n_pts for i in range(n_pts + 1))]
        cable_mesh(pts, colour)


def patch(ports):
    for (src, dst, colour) in PATCH:
        ends = [plug_at(ports[place].matrix_world, colour) for place in (src, dst)]
        cable_mesh(cable_curve(ends[0], ends[1]), colour)


def render(built, patched, out, hints):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    config = configuration()
    ports, comb = build_rack(config["places"], built)
    if patched:
        patch(ports)
    else:
        hang(comb, [c["color"] for c in config.get("cables", [])])
    # Patch pods pulse while there is a cable to place; once patched they rest.
    for obj in bpy.data.objects:
        if "JackHalo" in obj.name:
            obj.hide_render = patched
    lamp = bpy.data.materials.get("BurblerClock_Lamp")
    if lamp:   # mid-flash, as in the module preview
        lamp.node_tree.nodes["Principled BSDF"].inputs["Emission Strength"].default_value = 4.0

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
    scene.render.resolution_x = scene.render.resolution_y = 1100
    scene.view_settings.view_transform = "AgX"
    scene.view_settings.look = "AgX - Medium High Contrast"
    rack = DOCS / "hud_rack.png"
    scene.render.filepath = str(rack)
    bpy.ops.render.render(write_still=True)

    magick = shutil.which("magick")
    backdrop = DOCS / "keys/bleak_dusk.png"
    if magick and backdrop.exists():
        subprocess.run([
            magick, str(backdrop), "-resize", "1920x1080^", "-gravity", "center", "-extent", "1920x1080",
            "-blur", "0x14", "-fill", "#0b0c10", "-colorize", "55%",
            "-fill", "#15161ad0", "-stroke", "#ffb35066", "-strokewidth", "2",
            "-draw", "roundrectangle 470,70 1450,1010 18,18", "-stroke", "none",
            "(", str(rack), "-resize", "900x900", ")", "-gravity", "center", "-geometry", "+0+30",
            "-composite",
            "-gravity", "north", "-fill", "#e7e5dd", "-pointsize", "34", "-annotate", "+0+92", config["title"],
            "-fill", "#ffb350", "-pointsize", "20", "-annotate", "+0+138", "90 BPM  ·  configure",
            "-gravity", "south", "-fill", "#adb8d4", "-pointsize", "18", "-annotate", "+0+92", hints,
            str(out)], check=True)
        rack.unlink()


def main():
    render(False, False, DOCS / "hud_mock_empty.png",
           "make a VCO and a VCA   ·   drag each into its outlined slot   ·   Esc  close")
    render(True, False, DOCS / "hud_mock_ready.png",
           "take a cable from the comb   ·   drag it between glowing jacks   ·   Esc  close")
    render(True, True, DOCS / "hud_mock.png",
           "turn BPM to set the tempo   ·   drag between jacks to patch   ·   Esc  close")


if __name__ == "__main__":
    main()
