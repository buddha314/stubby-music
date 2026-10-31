"""Square VCO: the opening's first discovery, as a world-scale Eurorack module.

Builds only its own asset scene, so it is safe to run inside a user's open Blender
(e.g. through MCP) as well as headlessly.

Reproduce: blender -b --factory-startup -P tools/blender/create_square_vco.py

Scale: a 3U, 8HP Eurorack module, x10 (1.285 m tall, 0.406 m wide). Origin at the
base centre; the faceplate faces Blender -Y, which is glTF +Z. Named empties mark
the game-facing parts: `Port_voct` and `Port_out` (cable sockets, local +Z out of the
panel), `Knob_freq` (rotates about its local Z), and `Light_out` (driven by the rack's
light value). Each carries custom properties the game reads from glTF extras.
"""
from pathlib import Path
import math
import sys

import bmesh
import bpy
from mathutils import Vector

PROJECT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT / "tools/blender"))
from export_gltf import export_scene  # noqa: E402

ASSET_ID = "square_vco"
HEIGHT = 1.285  # 3U x10
WIDTH = 0.4064  # 8HP x10
BODY_DEPTH = 0.30
PANEL_THICKNESS = 0.02
PANEL_FRONT = -BODY_DEPTH / 2 - PANEL_THICKNESS  # y of the faceplate's front face
FACING_OUT = (math.pi / 2, 0.0, 0.0)  # rotates local +Z to point out of the panel (-Y)


def material(name, color, metallic=0.0, roughness=0.5, emission=None, strength=0.0):
    mat = bpy.data.materials.new(f"SquareVco_{name}")
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = (*color, 1.0)
    bsdf.inputs["Metallic"].default_value = metallic
    bsdf.inputs["Roughness"].default_value = roughness
    if emission:
        bsdf.inputs["Emission Color"].default_value = (*emission, 1.0)
        bsdf.inputs["Emission Strength"].default_value = strength
    mat.diffuse_color = (*color, 1.0)
    return mat


def build():
    scene = bpy.data.scenes.new("SquareVcoAsset")
    scene.unit_settings.system = "METRIC"
    scene.unit_settings.scale_length = 1.0
    collection = bpy.data.collections.new("STUBBY_SquareVco")
    scene.collection.children.link(collection)

    # Muted, worn palette (Inside), readable industrial forms (Satisfactory).
    body_mat = material("Body", (0.045, 0.047, 0.05), metallic=0.8, roughness=0.62)
    panel_mat = material("Faceplate", (0.42, 0.42, 0.40), metallic=1.0, roughness=0.38)
    output_mat = material("OutputPatch", (0.06, 0.06, 0.065), metallic=0.6, roughness=0.5)
    print_mat = material("Silkscreen", (0.015, 0.015, 0.015), roughness=0.7)
    # Outputs sit on a dark patch, so their print is inverted (light on dark).
    inverted_print_mat = material("SilkscreenInverted", (0.8, 0.78, 0.72), roughness=0.6)
    knob_mat = material("Knob", (0.02, 0.02, 0.022), roughness=0.55)
    pointer_mat = material("Pointer", (0.85, 0.83, 0.78), roughness=0.4)
    brass_mat = material("JackBrass", (0.55, 0.40, 0.18), metallic=1.0, roughness=0.32)
    hole_mat = material("JackHole", (0.0, 0.0, 0.0), roughness=0.9)
    screw_mat = material("Screw", (0.3, 0.3, 0.3), metallic=1.0, roughness=0.4)
    amber = (1.0, 0.45, 0.08)
    led_mat = material("LedOut", (0.25, 0.08, 0.01), roughness=0.3, emission=amber, strength=8.0)
    glyph_mat = material("Glyph", (0.2, 0.08, 0.02), roughness=0.4, emission=amber, strength=1.5)

    root = bpy.data.objects.new("SquareVco_Root", None)
    collection.objects.link(root)
    root["asset_id"] = ASSET_ID
    root["module_kind"] = "SquareVco"
    root["scale"] = "eurorack_3u_8hp_x10"

    def link(obj, mat=None, parent=root):
        collection.objects.link(obj)
        obj.parent = parent
        if mat is not None and obj.type == "MESH":
            obj.data.materials.append(mat)
        return obj

    def mesh_from(name, fill):
        mesh = bpy.data.meshes.new(f"SquareVco_{name}")
        bm = bmesh.new()
        fill(bm)
        bm.to_mesh(mesh)
        bm.free()
        return mesh

    def box(name, size, location, mat, bevel=0.0):
        def fill(bm):
            bmesh.ops.create_cube(bm, size=1.0)
            for v in bm.verts:
                v.co = Vector((v.co.x * size[0], v.co.y * size[1], v.co.z * size[2]))
        obj = link(bpy.data.objects.new(f"SquareVco_{name}", mesh_from(name, fill)), mat)
        obj.location = location
        if bevel:
            mod = obj.modifiers.new("Bevel", "BEVEL")
            mod.width = bevel
            mod.segments = 2
            mod.limit_method = "ANGLE"
        return obj

    def cylinder(name, radius, depth, location, mat, segments=32, parent=root, smooth=True):
        def fill(bm):
            bmesh.ops.create_cone(bm, cap_ends=True, segments=segments,
                                  radius1=radius, radius2=radius, depth=depth)
        obj = link(bpy.data.objects.new(f"SquareVco_{name}", mesh_from(name, fill)), mat, parent)
        obj.location = location
        obj.rotation_euler = FACING_OUT if parent is root else (0.0, 0.0, 0.0)
        if smooth:
            for face in obj.data.polygons:
                face.use_smooth = len(face.vertices) == 4
        return obj

    def label(name, text, location, size=0.034, mat=None):
        curve = bpy.data.curves.new(f"SquareVco_{name}Text", "FONT")
        curve.body = text
        curve.size = size
        curve.extrude = 0.0006
        curve.align_x = "CENTER"
        curve.align_y = "CENTER"
        text_obj = bpy.data.objects.new(f"SquareVco_{name}Text", curve)
        scene.collection.objects.link(text_obj)
        depsgraph = bpy.context.evaluated_depsgraph_get()
        mesh = bpy.data.meshes.new_from_object(text_obj.evaluated_get(depsgraph))
        bpy.data.objects.remove(text_obj)
        bpy.data.curves.remove(curve)
        obj = link(bpy.data.objects.new(f"SquareVco_{name}", mesh), mat or print_mat)
        obj.location = location
        obj.rotation_euler = FACING_OUT
        return obj

    def socket(name, location, **props):
        empty = bpy.data.objects.new(name, None)
        empty.empty_display_type = "SINGLE_ARROW"
        empty.empty_display_size = 0.08
        link(empty)
        empty.location = location
        empty.rotation_euler = FACING_OUT
        for key, value in props.items():
            empty[key] = value
        return empty

    # Case and faceplate.
    box("Body", (WIDTH - 0.01, BODY_DEPTH, HEIGHT - 0.03), (0, 0, HEIGHT / 2), body_mat, bevel=0.01)
    face_y = -BODY_DEPTH / 2 - PANEL_THICKNESS / 2
    box("Faceplate", (WIDTH, PANEL_THICKNESS, HEIGHT), (0, face_y, HEIGHT / 2), panel_mat, bevel=0.004)
    front = PANEL_FRONT
    for x in (-WIDTH / 2 + 0.05, WIDTH / 2 - 0.05):
        for z in (0.03, HEIGHT - 0.03):
            cylinder("Screw", 0.013, 0.006, (x, front - 0.003, z), screw_mat, segments=16)

    # The waveform this module makes, engraved and faintly lit near the top.
    glyph_z, amp, half, stroke = HEIGHT - 0.17, 0.035, 0.12, 0.009
    xs = [-half, -half / 2, -half / 2, 0.0, 0.0, half / 2, half / 2, half]
    zs = [glyph_z - amp, glyph_z - amp, glyph_z + amp, glyph_z + amp,
          glyph_z - amp, glyph_z - amp, glyph_z + amp, glyph_z + amp]
    for i in range(len(xs) - 1):
        (x0, z0), (x1, z1) = (xs[i], zs[i]), (xs[i + 1], zs[i + 1])
        size = (abs(x1 - x0) + stroke, 0.004, abs(z1 - z0) + stroke)
        box(f"Glyph{i}", size, ((x0 + x1) / 2, front - 0.001, (z0 + z1) / 2), glyph_mat)
    label("Title", "SQUARE", (0, front - 0.0005, HEIGHT - 0.075), size=0.04)

    # FREQ knob: a pivot empty the game rotates, with the knob parts as children.
    knob_z = 0.80
    knob = socket("Knob_freq", (0, front, knob_z), param="freq", param_min=1.0,
                  param_max=20000.0, taper="log", rotation_degrees=300.0)
    cylinder("KnobSkirt", 0.085, 0.02, (0, 0, 0.01), knob_mat, segments=48, parent=knob)
    cylinder("KnobCap", 0.065, 0.055, (0, 0, 0.0475), knob_mat, segments=48, parent=knob)
    pointer = box("KnobPointer", (0.008, 0.004, 0.05), (0, 0, 0), pointer_mat)
    pointer.parent = knob
    pointer.location = (0, 0.035, 0.0755)
    pointer.rotation_euler = (math.pi / 2, 0, 0)
    label("FreqLabel", "FREQ", (0, front - 0.0005, knob_z - 0.13))

    # Jacks: brass nut, dark hole, and the cable socket the game attaches to.
    def jack(port, direction, z, text, label_mat=None):
        nut = cylinder(f"Nut_{port}", 0.032, 0.012, (0, front - 0.006, z), brass_mat, segments=6, smooth=False)
        nut["port"] = port
        cylinder(f"Bushing_{port}", 0.024, 0.02, (0, front - 0.01, z), brass_mat, segments=24)
        cylinder(f"Hole_{port}", 0.0175, 0.022, (0, front - 0.0115, z), hole_mat, segments=24)
        socket(f"Port_{port}", (0, front - 0.022, z), port=port, direction=direction, signal="audio_or_cv")
        label(f"{port.capitalize()}Label", text, (0, front - 0.0015, z + 0.065), mat=label_mat)

    jack("voct", "in", 0.47, "V/OCT")
    # Outputs sit on a dark patch, the usual Eurorack cue.
    box("OutputPatch", (WIDTH - 0.08, 0.002, 0.24), (0, front - 0.001, 0.25), output_mat)
    jack("out", "out", 0.22, "OUT", label_mat=inverted_print_mat)

    # LED above the output: its brightness follows the rack's light value.
    led = socket("Light_out", (0.13, front, 0.33), light="out", rest_strength=0.0, peak_strength=8.0)
    led_dome = cylinder("Led", 0.011, 0.012, (0, 0, 0.004), led_mat, segments=16, parent=led)
    led_dome["light"] = "out"

    # Preview: a dim, cold world and a low warm key, like the mudflat.
    world = bpy.data.worlds.new("SquareVco_PreviewWorld")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.05, 0.055, 0.065, 1)
    world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.6
    scene.world = world
    key = bpy.data.objects.new("SquareVco_PreviewKey", bpy.data.lights.new("SquareVco_PreviewKey", "AREA"))
    key.data.energy = 180
    key.data.size = 1.2
    key.data.color = (1.0, 0.82, 0.62)
    key.location = (-1.4, -1.9, 2.1)
    key.rotation_euler = (Vector((0, 0, 0.8)) - key.location).to_track_quat("-Z", "Y").to_euler()
    scene.collection.objects.link(key)
    camera = bpy.data.objects.new("SquareVco_PreviewCamera", bpy.data.cameras.new("SquareVco_PreviewCamera"))
    camera.location = (1.05, -2.15, 1.25)
    camera.rotation_euler = (Vector((0, 0, 0.66)) - camera.location).to_track_quat("-Z", "Y").to_euler()
    camera.data.lens = 50
    scene.collection.objects.link(camera)
    scene.camera = camera
    scene.render.resolution_x = scene.render.resolution_y = 768
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 48
    scene.view_settings.view_transform = "AgX"
    scene.view_settings.look = "AgX - Medium High Contrast"

    source = PROJECT / "assets-src/blender/square_vco.blend"
    model = PROJECT / "assets/models/square_vco.glb"
    preview = PROJECT / "assets/modules/square_vco/preview.png"
    source.parent.mkdir(parents=True, exist_ok=True)
    if bpy.app.background:
        if bpy.context.window:
            bpy.context.window.scene = scene
        for other in list(bpy.data.scenes):
            if other != scene:
                bpy.data.scenes.remove(other)
        bpy.ops.wm.save_as_mainfile(filepath=str(source), check_existing=False)
        export_scene(scene, model)
        preview.parent.mkdir(parents=True, exist_ok=True)
        scene.render.filepath = str(preview)
        bpy.ops.render.render(write_still=True, scene=scene.name)
    else:
        # Inside a user's session: never save their file or switch their scene.
        bpy.data.libraries.write(str(source), {scene}, fake_user=True)
    return {"scene": scene.name, "source": str(source), "model": str(model),
            "objects": len(collection.objects)}


if __name__ == "__main__":
    report = build()
    if "result" in globals():
        result.update(report)
    else:
        print(report)
