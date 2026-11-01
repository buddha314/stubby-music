"""Shared building blocks for world-scale Eurorack module assets.

A module is a 3U panel at x10 scale (1.285 m tall), on a dark case, with the
faceplate facing Blender -Y (glTF +Z) and its origin at the base centre. The kit
builds only its own asset scene, so generator scripts stay safe to run inside a
user's open Blender (e.g. through MCP), where `finish` writes only the .blend.

Game-facing parts are named empties with glTF extras: `Port_<port>` (cable sockets,
local +Z out of the panel), `Knob_<param>` (rotates about local Z), and
`Light_<light>` (emissive strength follows the rack's light value), and
`Jack_<port>_Halo` (a soft ring around each jack, pulsed while it waits for a cable).

Light follows the style guide's luminescence rule: jellyfish and fairies, not LEDs. A
light is a bright opaque core inside translucent shells (`soft_light`), a glyph is a
glowing thread with a translucent halo (`soft_trace`), and a flashing lamp is a
jellyfish bell with trailing tendrils (`jellyfish`).
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
        self.glow_halo = None  # made on first use, so modules without halos don't carry it

        self.root = bpy.data.objects.new(f"{kind}_Root", None)
        self.collection.objects.link(self.root)
        self.root["asset_id"] = asset_id
        self.root["module_kind"] = kind
        for key, value in root_props.items():
            self.root[key] = value
        self.root["scale"] = f"eurorack_3u_{hp}hp_x10"

    def lod_levels(self, *levels):
        """Put the model's levels of detail inside it, as nodes named `<asset_id>_LOD0`,
        `<asset_id>_LOD1`, ...: the convention Unity's model importer and Jackdaw's
        `model.lod.import` read. Level 0 takes everything already in the asset except
        the objects passed for the later levels (one list per level, LOD1 first)."""
        later = [obj for objs in levels for obj in objs]
        nodes = []
        for level in range(len(levels) + 1):
            empty = bpy.data.objects.new(f"{self.asset_id}_LOD{level}", None)
            empty.empty_display_type = "PLAIN_AXES"
            self.link(empty)
            nodes.append(empty)
        for child in list(self.root.children):
            if child not in nodes and child not in later:
                child.parent = nodes[0]   # parents sit at the root's origin: transforms hold
        for node, objs in zip(nodes[1:], levels):
            for obj in objs:
                obj.parent = node
        return nodes

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

    def translucent(self, name, color, strength, alpha):
        """An emissive, alpha-blended material: the shells around a light's core."""
        mat = self.material(name, color, roughness=0.3, emission=color, strength=strength)
        mat.node_tree.nodes["Principled BSDF"].inputs["Alpha"].default_value = alpha
        mat.surface_render_method = "BLENDED"
        return mat

    def halo(self):
        """The shared faint halo shell, in signal amber."""
        if self.glow_halo is None:
            self.glow_halo = self.translucent("GlowHalo", AMBER, 1.2, 0.12)
        return self.glow_halo

    def trace_halo(self):
        """Fainter than `halo`: traces layer three of these, so the steps stay gentle."""
        if getattr(self, "glow_halo_faint", None) is None:
            self.glow_halo_faint = self.translucent("GlowHaloFaint", AMBER, 0.8, 0.06)
        return self.glow_halo_faint

    def sphere(self, name, radius, mat, parent, location=(0, 0, 0), shape=None, segments=16, rings=8):
        def fill(bm):
            bmesh.ops.create_uvsphere(bm, u_segments=segments, v_segments=rings, radius=radius)
            if shape:
                shape(bm)
        obj = self.link(bpy.data.objects.new(f"{self.kind}_{name}", self.mesh_from(name, fill)), mat, parent)
        obj.location = location
        for face in obj.data.polygons:
            face.use_smooth = True
        return obj

    def ring(self, name, radius, tube, mat, parent, steps=48):
        """A torus around the parent's local Z (the panel normal, under a socket)."""
        def fill(bm):
            verts = [bm.verts.new((radius + tube * math.cos(a), 0.0, tube * math.sin(a)))
                     for a in (k / 8 * 2 * math.pi for k in range(9))]
            edges = [bm.edges.new((a, b)) for a, b in zip(verts, verts[1:])]
            bmesh.ops.spin(bm, geom=verts + edges, cent=(0, 0, 0), axis=(0, 0, 1),
                           angle=2 * math.pi, steps=steps)
            bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)
        obj = self.link(bpy.data.objects.new(f"{self.kind}_{name}", self.mesh_from(name, fill)), mat, parent)
        for face in obj.data.polygons:
            face.use_smooth = True
        return obj

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

    def soft_trace(self, name, points, plane, offset, stroke, mat, halo=True):
        """A glowing thread: a thin bright core with three translucent halo strokes
        behind it (1.7x to 3.5x its width), so the glow falls off instead of stopping at
        an edge. The halos sit between the core's front and the surface, so the core
        always shows through them. Pass `halo=False` where the glow itself pulses (the
        game drives a lamp material by name; a static halo would not flash with it)."""
        core = self.trace(name, points, plane, offset, stroke, mat)
        if halo:
            inward = -1.0 if offset > 0 else 1.0  # toward the surface the trace lies on
            for k, width in enumerate((1.7, 2.5, 3.5)):
                self.trace(f"{name}Halo{k}", points, plane, offset + inward * (0.0008 + 0.0003 * k),
                           stroke * width, self.trace_halo(), depth=0.0002)
        return core

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
        self.soft_trace("Glyph", front_points, "xz", PANEL_FRONT - 0.0035, 0.014, mat)
        side_x = (self.width - 0.01) / 2 + 0.002
        for side, x in (("L", -side_x), ("R", side_x)):
            points = side_points if side == "R" else [(-u, z) for u, z in side_points]
            self.soft_trace(f"SideGlyph{side}", points, "yz", x, 0.014, mat)

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
        # The patch pod's halo: a soft ring around the nut the game pulses while the jack
        # waits for a cable, so the player sees where to patch.
        halo = self.socket(f"Jack_{port}_Halo", (0, front - 0.004, z), port=port, pulse="until_patched",
                           rest_strength=0.0, peak_strength=2.0, period_s=1.2)
        self.ring(f"JackHalo_{port}", 0.041, 0.006, self.halo(), halo)
        self.ring(f"JackHaloOuter_{port}", 0.044, 0.011, self.halo(), halo)
        mat = self.inverted_print_mat if on_patch else None
        self.label(f"{port.capitalize()}Label", text, (0, front - 0.0015, z + 0.065), mat=mat)

    def output_patch(self, z, height):
        """Outputs sit on a dark patch, the usual Eurorack cue."""
        self.box("OutputPatch", (self.width - 0.08, 0.002, height), (0, PANEL_FRONT - 0.001, z), self.output_mat)

    def soft_light(self, light, location, mat=None, peak_strength=8.0, core_r=0.01, halo_r=0.018):
        """A fairy light in place of an LED: an opaque core inside two translucent shells.
        Every mesh under `Light_<light>` is driven by the game, so the halo pulses too."""
        node = self.socket(f"Light_{light}", location, light=light, rest_strength=0.0,
                           peak_strength=peak_strength)
        core = self.sphere("LightCore", core_r, mat or self.glow, node)
        core["light"] = light
        self.sphere("LightHalo", halo_r, self.halo(), node)
        self.sphere("LightHaloOuter", halo_r * 1.8, self.halo(), node)
        return node

    def led(self, light, location, mat, peak_strength):
        """A status light whose brightness follows the rack's light value: a soft light,
        lifted just off the faceplate so its shells clear it."""
        x, y, z = location
        return self.soft_light(light, (x, y - 0.004, z), mat, peak_strength)

    def jellyfish(self, name, location, mat, radius=0.07, tendrils=4, **props):
        """A flashing lamp as a jellyfish: a scalloped bell with a bright core, and
        tendrils trailing down the faceplate. Bell, core and tendrils share `mat` (make it
        translucent) so they flash together when the game drives it. Returns the lamp
        node, which carries `props` as extras."""
        x, z = location
        lamp = bpy.data.objects.new(name, None)
        lamp.empty_display_type = "CIRCLE"
        lamp.empty_display_size = radius
        self.link(lamp)
        lamp.location = (x, PANEL_FRONT - 0.002, z)
        lamp.rotation_euler = FACING_OUT
        for key, value in props.items():
            lamp[key] = value

        def bell(bm):
            bmesh.ops.delete(bm, geom=[v for v in bm.verts if v.co.z < -1e-4], context="VERTS")
            for v in bm.verts:
                angle = math.atan2(v.co.y, v.co.x)
                if abs(v.co.z) < 1e-4:  # scalloped rim
                    v.co.x *= 1 + 0.1 * math.sin(8 * angle)
                    v.co.y *= 1 + 0.1 * math.sin(8 * angle)
                v.co.z *= 0.6
        self.sphere(f"{name}Bell", radius, mat, lamp, shape=bell, segments=24, rings=10)
        self.sphere(f"{name}Core", radius * 23 / 70, mat, lamp, location=(0, 0, radius * 18 / 70))
        # No outer halo shell: on the flat faceplate it reads as a disc and hides the
        # bell's scalloped rim. The game's bloom gives the lamp its halo.
        spread = radius * 45 / 70
        for k in range(tendrils):
            u = -spread + 2 * spread * k / max(tendrils - 1, 1)
            pts = [(x + u * 0.85 + 0.01 * math.sin(j * 1.1 + k), z - radius * 6 / 7 - j * radius * 16 / 70)
                   for j in range(5)]
            self.trace(f"{name}Tendril{k}", pts, "xz", PANEL_FRONT - 0.0025, 0.006, mat)
        return lamp

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
