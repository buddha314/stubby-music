"""Shared building blocks for world-scale Eurorack module assets.

A module is a 3U panel at x10 scale (1.285 m tall), on a dark case, with the
faceplate facing Blender -Y (glTF +Z) and its origin at the base centre. The kit
builds only its own asset scene, so generator scripts stay safe to run inside a
user's open Blender (e.g. through MCP), where `finish` writes only the .blend.

Game-facing parts are named empties with glTF extras: `Port_<port>` (cable sockets,
local +Z out of the panel), `Knob_<param>` (rotates about local Z), and
`Light_<light>` (emissive strength follows the rack's light value).
"""
from pathlib import Path
import json
import math
import sys

import bmesh
import bpy
from mathutils import Vector

PROJECT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT / "tools/blender"))
from export_gltf import export_scene  # noqa: E402

HEIGHT = 1.285  # 3U x10
HP = 0.0508  # one Eurorack HP (5.08 mm) x10
BODY_DEPTH = 0.30
PANEL_THICKNESS = 0.02
PANEL_FRONT = -BODY_DEPTH / 2 - PANEL_THICKNESS  # y of the faceplate's front face
FACING_OUT = (math.pi / 2, 0.0, 0.0)  # rotates local +Z to point out of the panel (-Y)
AMBER = (1.0, 0.45, 0.08)


def fit(points, cycles, centre_u, centre_z, half, amp):
    """Map (t, v) points, t in [0, cycles] and v in [-1, 1], onto centre_u ± half
    and centre_z ± amp."""
    return [(centre_u + half * (2 * t / cycles - 1), centre_z + amp * v) for t, v in points]


def model_file(asset_id):
    """The exported model for `asset_id`, wherever its category folder puts it."""
    found = sorted((PROJECT / "assets/models").rglob(f"{asset_id}.glb"))
    if not found:
        raise FileNotFoundError(f"no assets/models/**/{asset_id}.glb")
    return found[0]


class ModuleKit:
    # Category folder shared by the model (assets/models/<folder>/), its Blender
    # source (assets-src/blender/<folder>/) and its profile and previews
    # (assets/<folder>/<asset_id>/). LODs sit beside their model.
    folder = "modules"

    def __init__(self, kind, asset_id, hp=8, **root_props):
        self.kind = kind  # also the name prefix, e.g. "SquareVco"
        self.asset_id = asset_id
        self.width = hp * HP
        self.scene = bpy.data.scenes.new(f"{kind}Asset")
        self.scene.unit_settings.system = "METRIC"
        self.scene.unit_settings.scale_length = 1.0
        self.collection = bpy.data.collections.new(f"STUBBY_{kind}")
        self.scene.collection.children.link(self.collection)

        # Muted, worn palette (Inside), readable industrial forms (Satisfactory).
        m = self.material
        self.body_mat = m("Body", (0.045, 0.047, 0.05), metallic=0.8, roughness=0.62)
        self.panel_mat = m("Faceplate", (0.42, 0.42, 0.40), metallic=1.0, roughness=0.38)
        self.output_mat = m("OutputPatch", (0.06, 0.06, 0.065), metallic=0.6, roughness=0.5)
        self.print_mat = m("Silkscreen", (0.015, 0.015, 0.015), roughness=0.7)
        # Outputs sit on a dark patch, so their print is inverted (light on dark).
        self.inverted_print_mat = m("SilkscreenInverted", (0.8, 0.78, 0.72), roughness=0.6)
        self.knob_mat = m("Knob", (0.02, 0.02, 0.022), roughness=0.55)
        self.pointer_mat = m("Pointer", (0.85, 0.83, 0.78), roughness=0.4)
        self.brass_mat = m("JackBrass", (0.55, 0.40, 0.18), metallic=1.0, roughness=0.32)
        self.hole_mat = m("JackHole", (0.0, 0.0, 0.0), roughness=0.9)
        self.screw_mat = m("Screw", (0.3, 0.3, 0.3), metallic=1.0, roughness=0.4)

        self.root = bpy.data.objects.new(f"{kind}_Root", None)
        self.collection.objects.link(self.root)
        self.root["asset_id"] = asset_id
        self.root["module_kind"] = kind
        for key, value in root_props.items():
            self.root[key] = value
        self.root["scale"] = f"eurorack_3u_{hp}hp_x10"

    def export_lod(self, objects, level=1):
        """Move `objects` out of the asset into `<asset_id>_LOD<level>.glb` beside it,
        the file Jackdaw's LodGroup (and the game) loads as that level of detail."""
        if bpy.app.background:
            # The glTF exporter switches the window to the scene it exports, which
            # segfaults while Blender's default scene is still around (5.3 alpha).
            # `finish` drops the other scenes anyway; do it before this export too.
            if bpy.context.window:
                bpy.context.window.scene = self.scene
            for other in list(bpy.data.scenes):
                if other != self.scene:
                    bpy.data.scenes.remove(other)
            export_scene(self.scene, PROJECT / f"assets/models/{self.folder}/{self.asset_id}_LOD{level}.glb",
                         only=objects)
        for obj in objects:
            bpy.data.objects.remove(obj, do_unlink=True)

    # ------------------------------------------------------------ primitives

    def material(self, name, color, metallic=0.0, roughness=0.5, emission=None, strength=0.0):
        mat = bpy.data.materials.new(f"{self.kind}_{name}")
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

    def link(self, obj, mat=None, parent=None):
        self.collection.objects.link(obj)
        obj.parent = parent or self.root
        if mat is not None and obj.type == "MESH":
            obj.data.materials.append(mat)
        return obj

    def mesh_from(self, name, fill):
        mesh = bpy.data.meshes.new(f"{self.kind}_{name}")
        bm = bmesh.new()
        fill(bm)
        bm.to_mesh(mesh)
        bm.free()
        return mesh

    def box(self, name, size, location, mat, bevel=0.0):
        def fill(bm):
            bmesh.ops.create_cube(bm, size=1.0)
            for v in bm.verts:
                v.co = Vector((v.co.x * size[0], v.co.y * size[1], v.co.z * size[2]))
        obj = self.link(bpy.data.objects.new(f"{self.kind}_{name}", self.mesh_from(name, fill)), mat)
        obj.location = location
        if bevel:
            mod = obj.modifiers.new("Bevel", "BEVEL")
            mod.width = bevel
            mod.segments = 2
            mod.limit_method = "ANGLE"
        return obj

    def cylinder(self, name, radius, depth, location, mat, segments=32, parent=None, smooth=True,
                 facing_out=True):
        def fill(bm):
            bmesh.ops.create_cone(bm, cap_ends=True, segments=segments,
                                  radius1=radius, radius2=radius, depth=depth)
        obj = self.link(bpy.data.objects.new(f"{self.kind}_{name}", self.mesh_from(name, fill)), mat, parent)
        obj.location = location
        obj.rotation_euler = FACING_OUT if parent is None and facing_out else (0.0, 0.0, 0.0)
        if smooth:
            for face in obj.data.polygons:
                face.use_smooth = len(face.vertices) == 4
        return obj

    def label(self, name, text, location, size=0.034, mat=None):
        curve = bpy.data.curves.new(f"{self.kind}_{name}Text", "FONT")
        curve.body = text
        curve.size = size
        curve.extrude = 0.0006
        curve.align_x = "CENTER"
        curve.align_y = "CENTER"
        text_obj = bpy.data.objects.new(f"{self.kind}_{name}Text", curve)
        self.scene.collection.objects.link(text_obj)
        depsgraph = bpy.context.evaluated_depsgraph_get()
        mesh = bpy.data.meshes.new_from_object(text_obj.evaluated_get(depsgraph))
        bpy.data.objects.remove(text_obj)
        bpy.data.curves.remove(curve)
        obj = self.link(bpy.data.objects.new(f"{self.kind}_{name}", mesh), mat or self.print_mat)
        obj.location = location
        obj.rotation_euler = FACING_OUT
        return obj

    def socket(self, name, location, **props):
        empty = bpy.data.objects.new(name, None)
        empty.empty_display_type = "SINGLE_ARROW"
        empty.empty_display_size = 0.08
        self.link(empty)
        empty.location = location
        empty.rotation_euler = FACING_OUT
        for key, value in props.items():
            empty[key] = value
        return empty

    @staticmethod
    def plane_point(plane, offset, u, z):
        return Vector((u, offset, z)) if plane == "xz" else Vector((offset, u, z))

    def trace(self, name, points, plane, offset, stroke, mat, depth=0.004):
        """Strokes a polyline lying in a plane: "xz" (front, offset is y) or "yz"
        (side, offset is x). Each segment is a bar built from its end points, so
        diagonals and curves work as well as right angles."""
        normal = Vector((0, 1, 0)) if plane == "xz" else Vector((1, 0, 0))

        def fill(bm):
            for (u0, z0), (u1, z1) in zip(points, points[1:]):
                a, b = self.plane_point(plane, offset, u0, z0), self.plane_point(plane, offset, u1, z1)
                along = b - a
                if along.length < 1e-6:
                    continue
                along.normalize()
                across = along.cross(normal).normalized()
                # Extend each end by half a stroke so neighbouring bars meet at corners.
                a, b = a - along * stroke / 2, b + along * stroke / 2
                corners = [p + across * (s * stroke / 2) + normal * (t * depth / 2)
                           for p in (a, b) for s in (-1, 1) for t in (-1, 1)]
                v = [bm.verts.new(c) for c in corners]
                for face in ((0, 1, 3, 2), (4, 6, 7, 5), (0, 2, 6, 4),
                             (1, 5, 7, 3), (0, 4, 5, 1), (2, 3, 7, 6)):
                    bm.faces.new([v[i] for i in face])
        return self.link(bpy.data.objects.new(f"{self.kind}_{name}", self.mesh_from(name, fill)), mat)

    def prism(self, name, outline, mat):
        """Extrudes a front outline (u, z) back through the case's depth."""
        depth = BODY_DEPTH / 2 - PANEL_FRONT

        def fill(bm):
            front = [bm.verts.new((u, PANEL_FRONT, z)) for u, z in outline]
            back = [bm.verts.new((u, PANEL_FRONT + depth, z)) for u, z in outline]
            bm.faces.new(front)
            bm.faces.new(list(reversed(back)))
            n = len(outline)
            for i in range(n):
                j = (i + 1) % n
                bm.faces.new((front[j], front[i], back[i], back[j]))
            bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
        return self.link(bpy.data.objects.new(f"{self.kind}_{name}", self.mesh_from(name, fill)), mat)

    # ------------------------------------------------------------ module parts

    def case(self):
        """Dark case, brushed faceplate and its four mounting screws."""
        w = self.width
        self.box("Body", (w - 0.01, BODY_DEPTH, HEIGHT - 0.03), (0, 0, HEIGHT / 2), self.body_mat, bevel=0.01)
        face_y = -BODY_DEPTH / 2 - PANEL_THICKNESS / 2
        self.box("Faceplate", (w, PANEL_THICKNESS, HEIGHT), (0, face_y, HEIGHT / 2), self.panel_mat, bevel=0.004)
        for x in (-w / 2 + 0.05, w / 2 - 0.05):
            for z in (0.03, HEIGHT - 0.03):
                self.cylinder("Screw", 0.013, 0.006, (x, PANEL_FRONT - 0.003, z), self.screw_mat, segments=16)

    def glyphs(self, front_points, side_points, mat, glyph_z=1.065):
        """The module's emblem: a large glyph on a dark recess on the faceplate, and the
        same glyph on both sides of the case (mirrored on the left, so it reads
        correctly from outside: +y runs left there)."""
        self.box("GlyphPlate", (0.36, 0.002, 0.2), (0, PANEL_FRONT - 0.001, glyph_z), self.output_mat)
        self.trace("Glyph", front_points, "xz", PANEL_FRONT - 0.0035, 0.024, mat)
        side_x = (self.width - 0.01) / 2 + 0.002
        for side, x in (("L", -side_x), ("R", side_x)):
            points = side_points if side == "R" else [(-u, z) for u, z in side_points]
            self.trace(f"SideGlyph{side}", points, "yz", x, 0.024, mat)

    def title(self, text):
        self.label("Title", text, (0, PANEL_FRONT - 0.0005, HEIGHT - 0.06), size=0.04)

    def knob(self, param, z, text, **extras):
        """A pivot empty the game rotates, with the knob parts as children."""
        front = PANEL_FRONT
        knob = self.socket(f"Knob_{param}", (0, front, z), param=param, **extras)
        self.cylinder("KnobSkirt", 0.085, 0.02, (0, 0, 0.01), self.knob_mat, segments=48, parent=knob)
        self.cylinder("KnobCap", 0.065, 0.055, (0, 0, 0.0475), self.knob_mat, segments=48, parent=knob)
        pointer = self.box("KnobPointer", (0.008, 0.004, 0.05), (0, 0, 0), self.pointer_mat)
        pointer.parent = knob
        pointer.location = (0, 0.035, 0.0755)
        pointer.rotation_euler = (math.pi / 2, 0, 0)
        self.label(f"{param.capitalize()}Label", text, (0, front - 0.0005, z - 0.13))
        return knob

    def jack(self, port, direction, z, text, on_patch=False):
        """Brass nut, dark hole, and the cable socket the game attaches to."""
        front = PANEL_FRONT
        nut = self.cylinder(f"Nut_{port}", 0.032, 0.012, (0, front - 0.006, z), self.brass_mat,
                            segments=6, smooth=False)
        nut["port"] = port
        self.cylinder(f"Bushing_{port}", 0.024, 0.02, (0, front - 0.01, z), self.brass_mat, segments=24)
        self.cylinder(f"Hole_{port}", 0.0175, 0.022, (0, front - 0.0115, z), self.hole_mat, segments=24)
        self.socket(f"Port_{port}", (0, front - 0.022, z), port=port, direction=direction, signal="audio_or_cv")
        mat = self.inverted_print_mat if on_patch else None
        self.label(f"{port.capitalize()}Label", text, (0, front - 0.0015, z + 0.065), mat=mat)

    def output_patch(self, z, height):
        """Outputs sit on a dark patch, the usual Eurorack cue."""
        self.box("OutputPatch", (self.width - 0.08, 0.002, height), (0, PANEL_FRONT - 0.001, z), self.output_mat)

    def led(self, light, location, mat, peak_strength):
        """An LED whose brightness follows the rack's light value."""
        led = self.socket(f"Light_{light}", location, light=light, rest_strength=0.0,
                          peak_strength=peak_strength)
        dome = self.cylinder("Led", 0.011, 0.012, (0, 0, 0.004), mat, segments=16, parent=led)
        dome["light"] = light
        return led

    # ------------------------------------------------------------ output

    @property
    def model_ref(self):
        """The model's path as the game and Jackdaw load it (relative to assets/)."""
        return f"models/{self.folder}/{self.asset_id}.glb"

    def finish(self, profile, aim=(0, 0, 0.66), camera_location=(1.05, -2.15, 1.25), before_render=None):
        """Preview lighting and camera, then save the .blend, export the .glb, write
        asset.json, and render preview.png and preview_far.png (about 12 m away).
        `before_render` runs after export, to stage the preview without changing the
        exported model (e.g. showing one reading on a display). Everything lands under the
        kit's `folder`."""
        scene = self.scene
        world = bpy.data.worlds.new(f"{self.kind}_PreviewWorld")
        world.use_nodes = True
        world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.05, 0.055, 0.065, 1)
        world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.6
        scene.world = world
        key = bpy.data.objects.new(f"{self.kind}_PreviewKey", bpy.data.lights.new(f"{self.kind}_PreviewKey", "AREA"))
        key.data.energy = 180
        key.data.size = 1.2
        key.data.color = (1.0, 0.82, 0.62)
        key.location = (-1.4, -1.9, 2.1)
        key.rotation_euler = (Vector((0, 0, 0.8)) - key.location).to_track_quat("-Z", "Y").to_euler()
        scene.collection.objects.link(key)
        camera = bpy.data.objects.new(f"{self.kind}_PreviewCamera", bpy.data.cameras.new(f"{self.kind}_PreviewCamera"))
        camera.location = camera_location
        aim = Vector(aim)
        camera.rotation_euler = (aim - camera.location).to_track_quat("-Z", "Y").to_euler()
        camera.data.lens = 50
        scene.collection.objects.link(camera)
        scene.camera = camera
        scene.render.resolution_x = scene.render.resolution_y = 768
        scene.render.engine = "CYCLES"
        scene.cycles.samples = 48
        scene.view_settings.view_transform = "AgX"
        scene.view_settings.look = "AgX - Medium High Contrast"

        source = PROJECT / f"assets-src/blender/{self.folder}/{self.asset_id}.blend"
        model = PROJECT / f"assets/models/{self.folder}/{self.asset_id}.glb"
        preview = PROJECT / f"assets/{self.folder}/{self.asset_id}/preview.png"
        model.parent.mkdir(parents=True, exist_ok=True)
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
            preview.with_name("asset.json").write_text(json.dumps(profile, indent=2) + "\n")
            if before_render:
                before_render()
            scene.render.filepath = str(preview)
            bpy.ops.render.render(write_still=True, scene=scene.name)
            near = camera.location.copy()
            camera.location = aim + (near - aim).normalized() * 12.0
            scene.render.filepath = str(preview.with_name("preview_far.png"))
            bpy.ops.render.render(write_still=True, scene=scene.name)
            camera.location = near
        else:
            # Inside a user's session: never save their file or switch their scene.
            bpy.data.libraries.write(str(source), {scene}, fake_user=True)
        return {"scene": scene.name, "source": str(source), "model": str(model),
                "objects": len(self.collection.objects)}

    def base_profile(self, extra_height=0.0):
        return {
            "id": self.asset_id,
            "model": f"{self.model_ref}#Scene0",
            "module_kind": self.kind,
            "scale": self.root["scale"],
            "size_metres": {"width": self.width, "height": HEIGHT + extra_height,
                            "depth": BODY_DEPTH + PANEL_THICKNESS},
            "placement": {"upright": True, "max_tilt_degrees": 15.0, "max_sink_metres": 0.25},
        }


def run(builders, script):
    """Command-line entry: build the named assets (all by default), one clean Blender
    process per asset when headless, so each .blend holds only its own scene."""
    import subprocess

    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    names = args or list(builders)
    unknown = [n for n in names if n not in builders]
    if unknown:
        raise SystemExit(f"unknown asset(s) {unknown}; choose from {list(builders)}")
    if bpy.app.background and len(names) > 1:
        for name in names:
            subprocess.run([bpy.app.binary_path, "-b", "--factory-startup", "-P", script, "--", name],
                           check=True)
        return None
    return [builders[name]() for name in names]
