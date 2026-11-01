"""The field recorder: the hand-held device the player collects with.

A handheld stereo field recorder in the shape players know from location sound work:
an upright body with a screen and transport buttons, a rubber grip, and a pair of
crossed (X/Y) microphone capsules in a guard cage on top, under a shaggy fur
windguard (a "dead cat"). No brand, no logo. It is held at real size (metres, 1:1).

The windguard is shell fur, the usual real-time technique, since glTF has no hair:
`SHELLS` nested copies of the windguard's base, pushed out along their normals, each
cut by the same baked strand texture at a rising alpha cutoff (glTF `MASK`). Strands
are tall where the texture is bright, so the shells stack into tufts.

Game-facing nodes (empties, glTF extras):
  FieldRecorder_Root  origin at the grip, where the hand holds it
  Grip                the hand socket (same place as the root)
  Capture             the X/Y capsules' centre; local +Z points where it listens
  Windguard           parent of the fur shells (hide it to show the bare capsules)
  Light_rec           the record light, a fairy light the game pulses while capturing
  Screen              the display; its `Level_<n>` bars can be lit as an input meter

Reproduce: blender -b --factory-startup -P tools/blender/create_field_recorder.py
Writes assets/models/equipment/field_recorder.glb and assets/equipment/field_recorder/.
"""
from pathlib import Path
import math
import random
import sys

import bmesh
import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_burbler import linear  # noqa: E402
from module_kit import ModuleKit  # noqa: E402

SEED = 4
WIDTH, DEPTH = 0.068, 0.034          # body, metres
BODY_BOTTOM, BODY_TOP = -0.045, 0.068  # relative to the grip
FRONT = -DEPTH / 2                    # the screen faces -Y (glTF +Z)
HEAD_Z = 0.088                        # X/Y capsule centre
FUR_RADIUS = (0.031, 0.027, 0.029)   # windguard base: x, y, z half-extents
FUR_CENTRE = (0.0, 0.0, 0.094)
FUR_LENGTH = 0.018
SHELLS = 14
LEVELS = 8                            # bars in the screen's input meter


class FieldRecorder(ModuleKit):
    folder = "equipment"

    def __init__(self):
        super().__init__("FieldRecorder", "field_recorder", hp=1)
        for key in ("module_kind", "scale"):
            del self.root[key]
        self.root["kind"] = "held_equipment"
        self.root["role"] = "collector"
        m = self.material
        self.body_mat = m("Body", linear("#9a9ea4"), metallic=0.85, roughness=0.38)
        self.rubber = m("Rubber", linear("#2a2b2d"), roughness=0.92)
        self.glass = m("ScreenGlass", linear("#101214"), metallic=0.2, roughness=0.08)
        self.button = m("Button", linear("#3a3c40"), roughness=0.6)
        self.rec_button = m("RecButton", linear("#8a2a20"), roughness=0.5)
        self.cage = m("Cage", linear("#c9ccd1"), metallic=1.0, roughness=0.25)
        self.capsule = m("Capsule", linear("#1b1c1e"), metallic=0.6, roughness=0.45)
        amber = linear("#ffb350")
        self.display = m("Display", (0.25, 0.1, 0.02), roughness=0.3, emission=amber, strength=3.0)
        self.display_dim = m("DisplayDim", (0.04, 0.03, 0.02), roughness=0.9)
        rec = linear("#ff7050")
        self.rec_light = m("RecLight", rec, roughness=0.3, emission=rec, strength=8.0)
        self.glow_halo = self.translucent("GlowHalo", rec, 1.2, 0.12)

    # ------------------------------------------------------------ helpers

    def rounded_box(self, name, size, centre, mat, radius):
        """A box with rounded edges (bevel modifier, applied on export)."""
        obj = self.box(name, size, centre, mat)
        mod = obj.modifiers.new("Round", "BEVEL")
        mod.width = radius
        mod.segments = 4
        mod.limit_method = "ANGLE"
        for face in obj.data.polygons:
            face.use_smooth = True
        return obj

    def empty(self, name, location, parent=None, **props):
        obj = bpy.data.objects.new(name, None)
        obj.empty_display_type = "PLAIN_AXES"
        obj.empty_display_size = 0.02
        self.link(obj, parent=parent)
        obj.location = location
        for key, value in props.items():
            obj[key] = value
        return obj

    # ------------------------------------------------------------ parts

    def body(self):
        h = BODY_TOP - BODY_BOTTOM
        self.rounded_box("Body", (WIDTH, DEPTH, h), (0, 0, (BODY_TOP + BODY_BOTTOM) / 2),
                         self.body_mat, 0.009)
        # Rubber grip: the lower back and sides, with horizontal ridges.
        self.rounded_box("Grip", (WIDTH + 0.003, DEPTH + 0.002, 0.05), (0, 0.002, -0.02),
                         self.rubber, 0.008)
        for k in range(7):
            z = -0.04 + k * 0.007
            self.box(f"GripRidge{k}", (WIDTH - 0.012, 0.0016, 0.0018),
                     (0, DEPTH / 2 + 0.0028, z), self.rubber)
        # Shoulder that the mic head sits on.
        self.rounded_box("Shoulder", (WIDTH * 0.8, DEPTH * 0.85, 0.012),
                         (0, 0, BODY_TOP + 0.002), self.body_mat, 0.004)
        # Tripod thread on the base.
        self.cylinder("Thread", 0.004, 0.003, (0, 0, BODY_BOTTOM - 0.001), self.capsule,
                      segments=16, facing_out=False)

    def front(self):
        y = FRONT - 0.0005
        # Screen: dark glass, a waveform trace and an input meter.
        sx, sz, sw, sh = 0.0, 0.036, 0.046, 0.028
        self.box("ScreenBezel", (sw + 0.006, 0.0015, sh + 0.006), (sx, y, sz), self.rubber)
        self.box("ScreenGlass", (sw, 0.0015, sh), (sx, y - 0.0008, sz), self.glass)
        screen = self.empty("Screen", (sx, y - 0.0016, sz), display="input_level", levels=LEVELS)
        wave = [(sx - 0.019 + 0.038 * i / 40,
                 sz + 0.006 + 0.004 * math.sin(i * 0.9) * math.exp(-((i - 20) / 14) ** 2))
                for i in range(41)]
        self.trace("ScreenWave", wave, "xz", y - 0.0018, 0.0009, self.display)
        for n in range(LEVELS):
            x = sx - 0.019 + n * 0.0052
            lit = n < 5
            bar = self.box(f"Level_{n}", (0.0036, 0.0006, 0.004),
                           (0, 0, 0), self.display if lit else self.display_dim)
            bar.name = f"Level_{n}"
            bar.parent = screen
            bar.location = (x - sx + 0.0018, 0, -0.008)
            bar["level"] = n
        # Transport: four small buttons around a centre key, and the record button.
        for k, (dx, dz) in enumerate(((-0.012, 0), (0.012, 0), (0, 0.009), (0, -0.009))):
            self.cylinder(f"Button{k}", 0.003, 0.002, (dx, y - 0.001, 0.008 + dz), self.button,
                          segments=16)
        self.cylinder("ButtonCentre", 0.0045, 0.0022, (0, y - 0.001, 0.008), self.button, segments=20)
        self.cylinder("RecButton", 0.0055, 0.0028, (0, y - 0.0014, -0.008), self.rec_button,
                      segments=24)
        light = self.soft_light("rec", (0.016, y - 0.002, -0.008), mat=self.rec_light,
                                core_r=0.0018, halo_r=0.003)
        light["pulse_while"] = "capturing"
        # Side: a level wheel.
        self.cylinder("LevelWheel", 0.007, 0.004, (WIDTH / 2 + 0.001, 0, 0.03), self.rubber,
                      segments=24, facing_out=False).rotation_euler = (0, math.pi / 2, 0)

    def mic_head(self):
        """Two crossed capsules (X/Y, 90 degrees apart) in a guard cage."""
        capture = self.empty("Capture", (0, 0, HEAD_Z), pattern="xy_stereo", angle_degrees=90.0)
        capture.rotation_euler = (0, 0, 0)  # local +Z: up the device, where it listens
        for side, angle in (("L", -45.0), ("R", 45.0)):
            cap = self.cylinder(f"Capsule{side}", 0.0062, 0.022, (0, 0, HEAD_Z), self.capsule,
                                segments=20, facing_out=False)
            # Tilt each capsule forward-up and out by 45 degrees, crossing over the centre.
            cap.rotation_euler = (math.radians(-60), 0, math.radians(angle))
            grille = self.cylinder(f"Grille{side}", 0.0064, 0.003, (0, 0, HEAD_Z), self.cage,
                                   segments=20, facing_out=False)
            grille.rotation_euler = cap.rotation_euler
            grille.location = Vector((0, 0, HEAD_Z)) + cap.matrix_basis.to_3x3() @ Vector((0, 0, 0.011))
        # Guard: two hoops over the capsules.
        for k, rot in enumerate((0.0, math.pi / 2)):
            def hoop(bm, r=0.022, t=0.0016, n=24):
                pts = [(r * math.cos(a), 0.0, r * math.sin(a))
                       for a in (math.pi * i / n for i in range(n + 1))]
                for (x0, _, z0), (x1, _, z1) in zip(pts, pts[1:]):
                    a, b = Vector((x0, 0, z0)), Vector((x1, 0, z1))
                    d = (b - a).normalized()
                    side = d.cross(Vector((0, 1, 0))).normalized()
                    vs = [bm.verts.new(p + s1 * side * t / 2 + Vector((0, s2 * t / 2, 0)))
                          for p in (a, b) for s1 in (-1, 1) for s2 in (-1, 1)]
                    for f in ((0, 1, 3, 2), (4, 6, 7, 5), (0, 2, 6, 4), (1, 5, 7, 3),
                              (0, 4, 5, 1), (2, 3, 7, 6)):
                        bm.faces.new([vs[i] for i in f])
            obj = self.link(bpy.data.objects.new(f"FieldRecorder_Guard{k}",
                                                 self.mesh_from(f"Guard{k}", hoop)), self.cage)
            obj.location = (0, 0, BODY_TOP + 0.004)
            obj.rotation_euler = (0, 0, rot)

    def strand_texture(self, size=256):
        """Strand heights in alpha (bright = long), fur colour in RGB, tips lighter."""
        rng = random.Random(SEED)
        img = bpy.data.images.new("FieldRecorder_Fur", size, size, alpha=True)
        base, tip = linear("#6e655a"), linear("#cfc3ae")
        heights = [0.0] * (size * size)
        # Tufts: clumps of strands, each strand a texel, clump height varies.
        for _ in range(900):
            cx, cy = rng.randrange(size), rng.randrange(size)
            clump = rng.uniform(0.55, 1.0)
            for _ in range(rng.randrange(4, 12)):
                x = (cx + rng.randrange(-3, 4)) % size
                y = (cy + rng.randrange(-3, 4)) % size
                heights[y * size + x] = max(heights[y * size + x], clump * rng.uniform(0.7, 1.0))
        # Under-fur everywhere so the base never shows bald.
        for i in range(size * size):
            if heights[i] == 0.0 and rng.random() < 0.55:
                heights[i] = rng.uniform(0.05, 0.35)
        px = []
        for h in heights:
            c = [b + (t - b) * h for b, t in zip(base, tip)]
            px += [*c, h]
        img.pixels = px
        img.pack()
        return img

    def fur_material(self, k, img):
        """Shell k of SHELLS: alpha-clipped at a cutoff that rises with height."""
        cutoff = 0.04 + 0.92 * k / SHELLS
        mat = bpy.data.materials.new(f"FieldRecorder_FurShell{k}")
        mat.use_nodes = True
        nt = mat.node_tree
        bsdf = nt.nodes["Principled BSDF"]
        bsdf.inputs["Roughness"].default_value = 1.0
        tex = nt.nodes.new("ShaderNodeTexImage")
        tex.image = img
        tex.interpolation = "Closest"
        nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
        less = nt.nodes.new("ShaderNodeMath")
        less.operation = "LESS_THAN"
        less.inputs[1].default_value = cutoff
        nt.links.new(tex.outputs["Alpha"], less.inputs[0])
        inv = nt.nodes.new("ShaderNodeMath")
        inv.operation = "SUBTRACT"
        inv.inputs[0].default_value = 1.0
        nt.links.new(less.outputs[0], inv.inputs[1])
        nt.links.new(inv.outputs[0], bsdf.inputs["Alpha"])
        mat.surface_render_method = "DITHERED"
        mat.use_backface_culling = False
        mat.diffuse_color = (*linear("#8a7f72"), 1.0)
        return mat

    def windguard(self):
        guard = self.empty("Windguard", FUR_CENTRE, kind="fur_windscreen", shells=SHELLS,
                           fur_length_metres=FUR_LENGTH)
        img = self.strand_texture()
        rng = random.Random(SEED + 1)
        lumps = [(Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-0.3, 1))).normalized(),
                  rng.uniform(0.04, 0.1)) for _ in range(9)]
        rx, ry, rz = FUR_RADIUS

        def base(bm):
            bmesh.ops.create_uvsphere(bm, u_segments=40, v_segments=20, radius=1.0)
            uv = bm.loops.layers.uv.new("UVMap")
            for face in bm.faces:
                for loop in face.loops:
                    v = loop.vert.co
                    u = (math.atan2(v.y, v.x) / math.tau) % 1.0
                    loop[uv].uv = (u * 4.0, (v.z * 0.5 + 0.5) * 2.0)
            # Squash to the windguard's shape: a lumpy dome, open-ish bottom sitting on
            # the shoulder, with a few soft bulges so it reads as fur, not a ball.
            for v in bm.verts:
                d = v.co.normalized()
                bulge = sum(a * max(0.0, d.dot(n)) ** 6 for n, a in lumps)
                s = 1.0 + bulge
                v.co = Vector((d.x * rx * s, d.y * ry * s, d.z * rz * s))
                if v.co.z < -rz * 0.55:  # flatten the base onto the body's shoulder
                    v.co.z = -rz * 0.55 + (v.co.z + rz * 0.55) * 0.25
        mesh = self.mesh_from("WindguardBase", base)
        for face in mesh.polygons:
            face.use_smooth = True
        # Opaque inner sock, then the shells.
        self.link(bpy.data.objects.new("FieldRecorder_WindguardSock", mesh),
                  self.fur_material_solid(img), guard)
        for k in range(1, SHELLS + 1):
            shell_mesh = mesh.copy()
            shell_mesh.name = f"FieldRecorder_FurShell{k}"
            push = FUR_LENGTH * k / SHELLS
            for v in shell_mesh.vertices:
                v.co = v.co + v.normal * push
            shell = self.link(bpy.data.objects.new(f"FieldRecorder_FurShell{k}", shell_mesh),
                              None, guard)
            shell.visible_shadow = False  # stacked shells would shadow each other black
            shell.data.materials.clear()
            shell.data.materials.append(self.fur_material(k, img))
        return guard

    def fur_material_solid(self, img):
        mat = bpy.data.materials.new("FieldRecorder_FurBase")
        mat.use_nodes = True
        nt = mat.node_tree
        bsdf = nt.nodes["Principled BSDF"]
        bsdf.inputs["Roughness"].default_value = 1.0
        tex = nt.nodes.new("ShaderNodeTexImage")
        tex.image = img
        nt.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
        mat.diffuse_color = (*linear("#6e655a"), 1.0)
        return mat

    def build(self):
        self.empty("Grip", (0, 0, 0), socket="hand")
        self.body()
        self.front()
        self.mic_head()
        self.windguard()
        profile = {
            "id": self.asset_id,
            "model": f"{self.model_ref}#Scene0",
            "kind": "held_equipment",
            "role": "collector",
            "size_metres": {"width": WIDTH, "depth": DEPTH,
                            "height": FUR_CENTRE[2] + FUR_RADIUS[2] + FUR_LENGTH - BODY_BOTTOM},
            "grip": "Grip",
            "capture": {"node": "Capture", "pattern": "xy_stereo", "listens_along": "+Z"},
            "windguard": {"node": "Windguard", "shells": SHELLS, "fur_length_metres": FUR_LENGTH,
                          "shells_cast_shadows": False,
                          "note": "give the FurShell meshes NotShadowCaster; the sock casts"},
            "lights": [{"node": "Light_rec", "light": "rec", "rest_strength": 0.0,
                        "peak_strength": 8.0, "pulse_while": "capturing"}],
            "screen": {"node": "Screen", "shows": "input_level", "bars": LEVELS,
                       "bar_node": "Level_{n}"},
            "held": {"note": "first-person: hold at Grip in the right hand, Capture pointing "
                             "forward and slightly up; real size"},
        }
        def fill_light():
            fill = bpy.data.objects.new("FieldRecorder_PreviewFill",
                                        bpy.data.lights.new("FieldRecorder_PreviewFill", "AREA"))
            fill.data.energy = 12
            fill.data.size = 0.4
            fill.location = (0.35, -0.3, 0.25)
            fill.rotation_euler = (Vector((0, 0, 0.04)) - fill.location).to_track_quat("-Z", "Y").to_euler()
            self.scene.collection.objects.link(fill)
            # Rays cross every fur shell; Cycles' default of 8 would render the gaps black.
            self.scene.cycles.transparent_max_bounces = 48

        return self.finish(profile, aim=(0, 0, 0.035), camera_location=(0.17, -0.27, 0.12),
                           before_render=fill_light)


if __name__ == "__main__":
    report = FieldRecorder().build()
    if "result" in globals():
        result.update({"field_recorder": report})
    else:
        print(report)
