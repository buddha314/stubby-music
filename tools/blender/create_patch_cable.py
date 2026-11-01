"""Patch cable: the general cable and jack plug that connects any two module jacks
(add-art-direction 2.4).

Every module jack is a 3.5 mm jack at the world's x10 module scale, with a `Port_<port>`
socket whose local +Z points out of the panel. The plug is modelled along the same +Z,
so the game seats it by giving the plug the port's transform: the sleeve goes into the
hole, the body stands proud of the panel, and the cable leaves from `Cable_attach`.

The cable itself is drawn by the game as a tube of `cable.radius_m` with
`cable.sides` sides between the two plugs' `Cable_attach` points. It is **fully
elastic, as in VCV Rack**: it has no fixed length and stretches to wherever its ends
are dragged, and its droop comes from one tension setting. The curve is VCV Rack's: a
quadratic Bezier whose control point is the midpoint pulled down by
`(1 - tension) * (slump_base_m + span)`. VCV uses 150 px + span at a default tension
of 0.5; 150 px is 10 HP, so at the world's x10 module scale `slump_base_m` is 0.508.
`cable_curve()` below is the reference, and the previews are drawn with it. A short stub of cable ships in the
model so the game has the cable material and the look is reviewable.

Each cable has a colour, from the profile's `colors`; the game tints
`PatchCable_Cable` and `PatchCable_Collar` together. Plug, sleeve and body stay as
authored. Cables are not emissive: the signal's own light belongs to the modules.

  PatchCable_Plug      root of the plug, at the jack's mouth; +Z out of the panel
  Cable_attach         where the cable leaves the plug's strain relief; +Z along it

Reproduce: blender -b --factory-startup -P tools/blender/create_patch_cable.py
Writes assets/models/cables/patch_plug.glb and assets/cables/patch_plug/.
"""
from pathlib import Path
import math
import sys

import bmesh
import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_burbler import linear  # noqa: E402
from module_kit import ModuleKit  # noqa: E402

HOLE_RADIUS = 0.0175          # the module jacks' hole (3.5 mm jack at x10)
SLEEVE_LENGTH = 0.11          # into the jack, hidden once seated
BODY_LENGTH = 0.2
BODY_RADIUS = 0.04
RELIEF_LENGTH = 0.13          # the tapering strain relief, cable-coloured
CABLE_RADIUS = 0.016
CABLE_SIDES = 10
TENSION = 0.5                 # VCV Rack's default cable tension
SLUMP_BASE = 10 * 0.0508      # VCV's 150 px is 10 HP; one HP is 5.08 mm, x10
COLORS = {                    # sRGB, muted to sit in the bleak palette
    "rust": "#9c3b2a",
    "ochre": "#b8892f",
    "verdigris": "#3f7a6e",
    "slate": "#4b5a78",
    "bone": "#cfc6b2",
}


def cable_curve(a, b, n=32, tension=TENSION, down=(0, 0, -1)):
    """VCV Rack's elastic cable from plug `a` to plug `b` (world space): a quadratic
    Bezier through the midpoint pulled `down` by (1 - tension) * (SLUMP_BASE + span).
    Returns n+1 points. There is no length: drag an end and the cable stretches."""
    a, b = Vector(a), Vector(b)
    slump = (1 - tension) * (SLUMP_BASE + (b - a).length)
    c = (a + b) / 2 + Vector(down).normalized() * slump
    return [a * (1 - t) ** 2 + c * 2 * (1 - t) * t + b * t * t for t in (i / n for i in range(n + 1))]


def tube(bm, points, radius, sides=CABLE_SIDES):
    """A round tube along `points`, rings kept square to the path."""
    rings, normal = [], None
    for i, p in enumerate(points):
        a, b = points[max(i - 1, 0)], points[min(i + 1, len(points) - 1)]
        t = (b - a).normalized()
        normal = t.orthogonal().normalized() if normal is None else (normal - t * normal.dot(t)).normalized()
        side = t.cross(normal)
        rings.append([bm.verts.new(p + radius * (normal * math.cos(2 * math.pi * k / sides)
                                                 + side * math.sin(2 * math.pi * k / sides)))
                      for k in range(sides)])
    for ra, rb in zip(rings, rings[1:]):
        for k in range(sides):
            bm.faces.new((ra[k], ra[(k + 1) % sides], rb[(k + 1) % sides], rb[k]))
    return rings


class PatchPlug(ModuleKit):
    folder = "cables"

    def __init__(self):
        super().__init__("PatchCable", "patch_plug", hp=1)
        for key in ("module_kind", "scale"):
            del self.root[key]
        self.root["kind"] = "patch_plug"
        self.root["fits"] = "module jacks (3.5 mm at x10)"
        m = self.material
        first = linear(COLORS["rust"])
        self.sleeve = m("Sleeve", linear("#b8bcc2"), metallic=1.0, roughness=0.3)
        self.body = m("Body", linear("#1a1c1f"), metallic=0.7, roughness=0.35)
        self.collar = m("Collar", first, roughness=0.45)
        self.cable = m("Cable", first, roughness=0.55)

    def lathe(self, name, profile, mat, sides=24):
        """Spin an (r, z) profile about Z."""
        def fill(bm):
            verts = [bm.verts.new((r, 0.0, z)) for r, z in profile]
            edges = [bm.edges.new((a, b)) for a, b in zip(verts, verts[1:])]
            bmesh.ops.spin(bm, geom=verts + edges, cent=(0, 0, 0), axis=(0, 0, 1), angle=2 * math.pi,
                           steps=sides)
            bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)
        obj = self.link(bpy.data.objects.new(f"PatchCable_{name}", self.mesh_from(name, fill)), mat)
        for face in obj.data.polygons:
            face.use_smooth = True
        return obj

    def build(self):
        r, s = HOLE_RADIUS, SLEEVE_LENGTH
        # Tip and sleeve go into the jack: a rounded tip, a ring, the sleeve.
        self.lathe("Sleeve", [(0.0, -s - 0.012), (r * 0.7, -s - 0.006), (r * 0.8, -s + 0.012),
                              (r * 0.62, -s + 0.02), (r * 0.62, -s + 0.028), (r, -s + 0.036),
                              (r, 0.0), (0.0, 0.0)], self.sleeve)
        # Body: knurled graphene grip with a coloured collar at the panel end.
        b, L = BODY_RADIUS, BODY_LENGTH
        knurl = []
        for i in range(13):
            z = 0.035 + (L - 0.05) * i / 12
            knurl.append((b * (0.97 if i % 2 else 1.0), z))
        self.lathe("Body", [(0.0, 0.035), (b, 0.035)] + knurl + [(b * 0.9, L), (0.0, L)], self.body,
                   sides=16)
        self.lathe("Collar", [(0.0, 0.0), (b * 1.04, 0.0), (b * 1.04, 0.035), (0.0, 0.035)], self.collar,
                   sides=24)
        # Strain relief: tapers from the body down to the cable.
        rl = RELIEF_LENGTH
        self.lathe("Relief", [(0.0, L), (b * 0.78, L)] +
                   [(CABLE_RADIUS * 1.25 + (b * 0.78 - CABLE_RADIUS * 1.25) * (1 - i / 6) ** 1.6, L + rl * i / 6)
                    for i in range(1, 7)] + [(0.0, L + rl)], self.cable, sides=16)
        attach = self.socket("Cable_attach", (0, 0, L + rl))
        attach.rotation_euler = (0, 0, 0)   # +Z continues along the cable
        # A short stub: the cable material ships with the plug.
        bm = bmesh.new()
        tube(bm, [Vector((0, 0, L + rl - 0.01)), Vector((0, 0, L + rl + 0.12))], CABLE_RADIUS)
        mesh = bpy.data.meshes.new("PatchCable_Stub")
        bm.to_mesh(mesh)
        bm.free()
        stub = self.link(bpy.data.objects.new("PatchCable_Stub", mesh), self.cable)
        for face in stub.data.polygons:
            face.use_smooth = True
        self.root.name = "PatchCable_Plug"

        profile = {
            "id": self.asset_id,
            "model": f"{self.model_ref}#Scene0",
            "kind": "patch_plug",
            "fits": {"port_node": "Port_<port>", "hole_radius_m": HOLE_RADIUS,
                     "seat": "give the plug root the port's transform"},
            "attach_node": "Cable_attach",
            "cable": {
                "radius_m": CABLE_RADIUS, "sides": CABLE_SIDES, "material": "PatchCable_Cable",
                "elastic": True,
                "curve": "VCV Rack's: quadratic Bezier between the two Cable_attach points, control point "
                         "= midpoint + down * (1 - tension) * (slump_base_m + span). No fixed length. "
                         "Reference: cable_curve() in tools/blender/create_patch_cable.py",
                "tension": TENSION, "tension_range": [0.0, 1.0], "slump_base_m": round(SLUMP_BASE, 4),
                "down": "world down (-Y in Bevy); on a HUD rack, the screen's down",
            },
            "colors": COLORS,
            "tint": ["PatchCable_Cable", "PatchCable_Collar"],
            "default_colors": {"gate": "verdigris", "audio": "rust", "cv": "ochre"},
        }

        def preview():
            # Not exported: the stub swapped for a short drooping run, two colours.
            stub.hide_render = True
            for k, (name, x) in enumerate((("rust", 0.0), ("verdigris", 0.16))):
                bm = bmesh.new()
                pts = cable_curve((x, 0, L + rl), (x + 0.5, 0.05, L + rl), tension=0.75)
                tube(bm, pts, CABLE_RADIUS)
                mesh = bpy.data.meshes.new(f"PreviewCable{k}")
                bm.to_mesh(mesh)
                bm.free()
                mat = self.material(f"PreviewCable_{name}", linear(COLORS[name]), roughness=0.55)
                obj = self.link(bpy.data.objects.new(f"PreviewCable{k}", mesh), mat)
                for face in obj.data.polygons:
                    face.use_smooth = True
            self.stage_preview()
        return self.finish(profile, aim=(0.12, 0, 0.12), camera_location=(0.75, -0.95, 0.55),
                           before_render=preview)

    def stage_preview(self):
        ground = self.box("PreviewGround", (6, 6, 0.02), (0, 0, -0.14), self.material(
            "PreviewGround", linear("#2a2420"), roughness=0.9))
        ground.hide_select = True


if __name__ == "__main__":
    print(PatchPlug().build())
