"""The cottage: the first Dancer dwelling, an off-grid homestead for four (add-settlement-economy).

Built from the brief `docs/art/cottage.md` and its adopted concept, the hero view of
`docs/art/cottage/design-sheet-chatgpt-2026-11-20.png`: rammed-earth walls on a stone
plinth, a steep gable roof of living sod running front to back, and one great grown-wood
rib sweeping from the ridge down past the front-left corner to the ground. Crude parts
carry the weight; high-tech parts are lashed on with visible joins: a graphene solar row
on titanium rails (left slope), a vertical-axis wind turbine (rear-left corner), a gutter
into a caged water tank with a filter column (front-left corner), a battery cabinet left
of the door, a stovepipe chimney (rear-right), and a porch shaded by reed mats along the
right side. Soft cyan fungi at the wall base and amber window glow are the only light.

Frame (Blender, +Z up): width along X (6 m), length along Y (8 m), the front (door) at
-Y, which glTF exports as +Z. The origin is the centre of the footprint at ground level.

Game-facing nodes (empties, glTF extras):
  Door          the doorway, at threshold height (role door; Dancers' way in later)
  Chimney_top   where smoke rises (role smoke; later)
  cottage_LOD0 / cottage_LOD1   the near and far levels
The windows' glow is its own material (`Cottage_Window`), so the game can light it.

Reproduce: blender -b --factory-startup -P tools/blender/create_cottage.py
Writes assets/structures/cottage/ (model, profile), assets-src/blender/structures/
cottage.blend and previews/structures/cottage/ (hero, far, elevations, top, night).
"""
from pathlib import Path
import json
import math
import random
import sys

import bmesh
import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_burbler import Burbler, linear  # noqa: E402
from module_kit import PROJECT  # noqa: E402

HALF_W = 3.0          # walls, half width (X)
HALF_L = 4.0          # walls, half length (Y)
PLINTH = 0.25         # stone plinth height
EAVE = 2.7            # wall top
RIDGE = 5.5           # ridge height (roof surface)
OVERHANG = 0.5        # roof past the walls, every side
SOD = 0.28            # roof slab thickness
SLOPE = (RIDGE - EAVE) / HALF_W
WALL = 0.35           # wall thickness


def roof_z(x):
    """The roof's top surface above x (either slope)."""
    return RIDGE - abs(x) * SLOPE


class Cottage(Burbler):
    folder = "structures"

    def __init__(self):
        Burbler.__init__(self, "cottage", kind="Cottage")
        for key in ("mine", "produces"):
            del self.root[key]
        self.root["kind"] = "structure"
        self.root["dwelling"] = "cottage"
        self.rng = random.Random(83)
        m = self.material
        # Crude
        self.earth = m("RammedEarth", linear("#8c7656"), roughness=1.0)
        self.band = m("EarthBand", linear("#5a4433"), roughness=1.0)
        self.stone = m("Stone", linear("#6a5d4d"), roughness=0.95)
        self.timber = m("Timber", linear("#5a4a3a"), roughness=0.9)
        self.grown = m("GrownWood", linear("#4b4037"), roughness=0.85)
        self.rust = m("RustIron", linear("#5e3a22"), metallic=0.5, roughness=0.85)
        self.moss = m("Moss", linear("#3f5a24"), roughness=1.0)
        self.leaf = m("Leaf", linear("#4f6a2a"), roughness=0.9)
        self.reed = m("ReedMat", linear("#9a8460"), roughness=1.0)
        # High-tech (titanium, graphene from the kit)
        self.tank_water = self.translucent("TankWater", linear("#b8bcc2"), 0.0, 0.45)
        # Light: soft, organic; the window is its own material for the game to drive.
        amber, cyan = linear("#ffb350"), linear("#33ffd9")
        self.window = m("Window", amber, emission=amber, strength=4.0)
        self.stem = m("FungusStem", linear("#d9d2c0"), roughness=0.7)
        self.cap = m("FungusCap", cyan, emission=cyan, strength=3.0)
        self.cap_halo = self.translucent("FungusHalo", cyan, 0.8, 0.15)
        self.signal = m("Signal", amber, emission=amber, strength=6.0)
        # Every material declares its acoustic class (Steam Audio presets by name,
        # `AcousticOccluder::preset`; docs/audio/acoustic-level-authoring.md). None: the
        # material only dresses small parts, which stay out of the acoustic scene.
        self.acoustics = {
            self.earth: "brick",       # rammed earth: dense masonry
            self.band: "brick",
            self.stone: "rock",
            self.timber: "wood",
            self.grown: "wood",
            self.rust: "metal",
            self.moss: "gravel",       # a sod roof: porous, absorbent
            self.leaf: None,           # grass tufts
            self.reed: "carpet",       # woven reed mats: soft, porous
            self.titanium: "metal",
            self.graphene: "metal",
            self.tank_water: "glass",       # a smooth, rigid, reflective tank of water
            self.window: "glass",
            self.stem: None, self.cap: None, self.cap_halo: None, self.signal: None,
        }

    # ------------------------------------------------------------ helpers

    def box(self, name, centre, size, mat, rotate_z=0.0, parent=None):
        def fill(bm):
            bmesh.ops.create_cube(bm, size=1.0)
            for v in bm.verts:
                v.co = Vector((v.co.x * size[0], v.co.y * size[1], v.co.z * size[2]))
                if rotate_z:
                    c, s = math.cos(rotate_z), math.sin(rotate_z)
                    v.co = Vector((v.co.x * c - v.co.y * s, v.co.x * s + v.co.y * c, v.co.z))
        return self.mesh_obj(name, fill, mat, location=centre, parent=parent, smooth=False)

    @staticmethod
    def tube(bm, points, radii, sides=8, cap=True):
        points = [Vector(p) for p in points]
        rings, normal = [], None
        for i, (p, r) in enumerate(zip(points, radii)):
            a, b = points[max(i - 1, 0)], points[min(i + 1, len(points) - 1)]
            t = (b - a).normalized()
            normal = t.orthogonal().normalized() if normal is None else (normal - t * normal.dot(t)).normalized()
            side = t.cross(normal)
            rings.append([bm.verts.new(p + r * (normal * math.cos(math.tau * k / sides)
                                                + side * math.sin(math.tau * k / sides))) for k in range(sides)])
        for ra, rb in zip(rings, rings[1:]):
            for k in range(sides):
                bm.faces.new((ra[k], ra[(k + 1) % sides], rb[(k + 1) % sides], rb[k]))
        if cap:
            bm.faces.new(rings[0][::-1])
            bm.faces.new(rings[-1])

    def pipe(self, name, points, radius, mat, sides=8, parent=None):
        return self.mesh_obj(name, lambda bm: self.tube(bm, points, [radius] * len(points), sides),
                             mat, parent=parent)

    def prism(self, name, outline, y0, y1, mat):
        """An XZ outline extruded along Y from y0 to y1."""
        def fill(bm):
            front = [bm.verts.new((x, y0, z)) for x, z in outline]
            back = [bm.verts.new((x, y1, z)) for x, z in outline]
            n = len(outline)
            bm.faces.new(front[::-1])
            bm.faces.new(back)
            for i in range(n):
                bm.faces.new((front[i], front[(i + 1) % n], back[(i + 1) % n], back[i]))
        return self.mesh_obj(name, fill, mat, smooth=False)

    # ------------------------------------------------------------ the house

    def walls(self):
        w, l = HALF_W, HALF_L
        self.box("Plinth", (0, 0, PLINTH / 2), (2 * w + 0.3, 2 * l + 0.3, PLINTH), self.stone)
        h = EAVE - PLINTH
        z = PLINTH + h / 2
        self.box("WallFront", (0, -l + WALL / 2, z), (2 * w, WALL, h), self.earth)
        self.box("WallBack", (0, l - WALL / 2, z), (2 * w, WALL, h), self.earth)
        self.box("WallLeft", (-w + WALL / 2, 0, z), (WALL, 2 * l - 2 * WALL, h), self.earth)
        self.box("WallRight", (w - WALL / 2, 0, z), (WALL, 2 * l - 2 * WALL, h), self.earth)
        # Mycelium-composite block courses: thin joints proud of the earth walls.
        def joints(bm):
            def quad(a, b, c, d):
                bm.faces.new([bm.verts.new(v) for v in (a, b, c, d)])
            courses = [PLINTH + 0.42 * k for k in range(1, 6)]
            for side in ("front", "back", "left", "right"):
                if side in ("front", "back"):
                    y = (-l if side == "front" else l) + (-0.012 if side == "front" else 0.012)
                    span, fixed = (-w + 0.15, w - 0.15), lambda u, z: (u, y, z)
                else:
                    x = (-w if side == "left" else w) + (-0.012 if side == "left" else 0.012)
                    span, fixed = (-l + 0.15, l - 0.15), lambda u, z: (x, u, z)
                for z in courses:
                    a, b = span
                    quad(fixed(a, z - 0.02), fixed(b, z - 0.02), fixed(b, z + 0.02), fixed(a, z + 0.02))
                for k, (z0, z1) in enumerate(zip([PLINTH] + courses, courses + [EAVE - 0.1])):
                    u = span[0] + (0.35 if k % 2 else 0.7)
                    while u < span[1] - 0.2:
                        quad(fixed(u - 0.02, z0), fixed(u + 0.02, z0), fixed(u + 0.02, z1), fixed(u - 0.02, z1))
                        u += 0.7
        self.mesh_obj("BlockJoints", joints, self.band, smooth=False)
        # Timber corner posts carry the roof's ends.
        for sx in (-1, 1):
            for sy in (-1, 1):
                self.box(f"Post{sx}{sy}", (sx * (w - 0.05), sy * (l - 0.05), EAVE / 2),
                         (0.24, 0.24, EAVE), self.timber)
        # Gables: timber boards above the eave, front and back.
        gable = [(-w, EAVE), (w, EAVE), (0, RIDGE - 0.05)]
        self.prism("GableFront", gable, -l - 0.02, -l + 0.12, self.timber)
        self.prism("GableBack", gable, l - 0.12, l + 0.02, self.timber)
        # A wall plate along each eave.
        for sx in (-1, 1):
            self.box(f"Plate{sx}", (sx * (w - 0.1), 0, EAVE - 0.08), (0.26, 2 * l + 0.2, 0.18), self.timber)

    def roof(self):
        y0, y1 = -HALF_L - OVERHANG, HALF_L + OVERHANG
        reach = HALF_W + OVERHANG
        for side, name in ((-1, "Left"), (1, "Right")):
            outline = [(0, RIDGE + SOD), (side * reach, roof_z(reach) + SOD * 0.6),
                       (side * reach, roof_z(reach) - 0.05), (0, RIDGE - 0.12)]
            if side > 0:
                outline = outline[::-1]
            self.prism(f"Roof{name}", outline, y0, y1, self.moss)
            # Fascia boards along each eave.
            self.box(f"Fascia{name}", (side * (reach - 0.05), 0, roof_z(reach) + 0.05),
                     (0.08, y1 - y0, 0.24), self.timber)
        # The ridge beam, a little wavy: grown, not milled.
        pts = [(0.03 * math.sin(i * 1.3), y0 - 0.1 + i * (y1 - y0 + 0.2) / 6, RIDGE + SOD + 0.06) for i in range(7)]
        self.mesh_obj("RidgeBeam", lambda bm: self.tube(bm, pts, [0.16] * 7, 8), self.grown)
        # Grass tufts on the sod, in clumps, with a fringe hanging over each eave.
        def tufts(bm):
            def tuft(base, hgt, r, lean=(0.0, 0.0)):
                # A fan of crossed blades, each leaning out from the clump's centre.
                spin = self.rng.uniform(0, math.pi)
                for k in range(3):
                    a = spin + math.pi * k / 3
                    d = Vector((math.cos(a), math.sin(a), 0))
                    for s in (-1, 1):
                        out = d * s
                        w = Vector((-out.y, out.x, 0)) * r * 0.35
                        tip = base + out * r * 1.6 + Vector((lean[0], lean[1], hgt * self.rng.uniform(0.7, 1.0)))
                        bm.faces.new((bm.verts.new(base - w), bm.verts.new(base + w), bm.verts.new(tip)))
            for side in (-1, 1):
                for i in range(26):
                    y = y0 + 0.2 + (y1 - y0 - 0.4) * i / 25 + self.rng.uniform(-0.1, 0.1)
                    x = side * (reach - 0.08)
                    tuft(Vector((x, y, roof_z(reach) + SOD * 0.6)), -self.rng.uniform(0.25, 0.5),
                         self.rng.uniform(0.1, 0.16), (side * 0.15, 0.0))
            for _ in range(230):
                x = self.rng.uniform(-reach + 0.3, reach - 0.3)
                y = self.rng.uniform(y0 + 0.2, y1 - 0.2)
                if x < -0.6 and -3.4 < y < 1.9:   # under the solar row
                    continue
                base = Vector((x, y, roof_z(x) + SOD * (0.6 + 0.4 * (1 - abs(x) / reach)) - 0.02))
                tuft(base, self.rng.uniform(0.2, 0.5), self.rng.uniform(0.1, 0.18),
                     (self.rng.uniform(-0.08, 0.08), self.rng.uniform(-0.08, 0.08)))
        self.mesh_obj("Grass", tufts, self.leaf, smooth=False)

        # Moss mounds: low lumps that break the roof's straight line.
        def mounds(bm):
            for _ in range(36):
                x = self.rng.uniform(-reach + 0.4, reach - 0.4)
                y = self.rng.uniform(y0 + 0.3, y1 - 0.3)
                if x < -0.4 and -3.5 < y < 2.0:
                    continue
                r = self.rng.uniform(0.25, 0.5)
                c = Vector((x, y, roof_z(x) + SOD * (0.6 + 0.4 * (1 - abs(x) / reach)) - 0.05))
                ring = [bm.verts.new(c + Vector((math.cos(a) * r, math.sin(a) * r * 1.3, 0)))
                        for a in (math.tau * k / 7 for k in range(7))]
                mid = [bm.verts.new(c + Vector((math.cos(a) * r * 0.6, math.sin(a) * r * 0.8, r * 0.35)))
                       for a in (math.tau * k / 7 for k in range(7))]
                top = bm.verts.new(c + Vector((0, 0, r * 0.5)))
                for i in range(7):
                    j = (i + 1) % 7
                    bm.faces.new((ring[i], ring[j], mid[j], mid[i]))
                    bm.faces.new((mid[i], mid[j], top))
        self.mesh_obj("MossMounds", mounds, self.moss)
        # Corrugated rust patch on the left slope's front eave.
        self.corrugated("RoofPatch", x_from=-reach + 0.05, x_to=-1.9, y0=y0 + 0.05, y1=y0 + 2.2)
        # And on the left wall, low at the front.
        def wall_patch(bm):
            x = -HALF_W - 0.03
            for i in range(12):
                ya, yb = -3.7 + i * 0.12, -3.7 + (i + 1) * 0.12
                da, db = 0.03 * math.sin(i * math.pi), 0.03 * math.sin((i + 1) * math.pi)
                v = [bm.verts.new((x - da, ya, 0.35)), bm.verts.new((x - db, yb, 0.35)),
                     bm.verts.new((x - db, yb, 1.6)), bm.verts.new((x - da, ya, 1.6))]
                bm.faces.new(v)
        self.mesh_obj("WallPatch", wall_patch, self.rust, smooth=False)

    def corrugated(self, name, x_from, x_to, y0, y1, ribs=18):
        """A rusted corrugated sheet lying on the left slope between x_from and x_to."""
        def fill(bm):
            rows = []
            for i in range(ribs + 1):
                y = y0 + (y1 - y0) * i / ribs
                lift = 0.04 * (1 + math.sin(i * math.pi))
                rows.append([bm.verts.new((x, y, roof_z(x) + SOD * (0.6 + 0.4 * (1 - abs(x) / (HALF_W + OVERHANG))) + lift))
                             for x in (x_from, x_to)])
            for a, b in zip(rows, rows[1:]):
                bm.faces.new((a[0], a[1], b[1], b[0]))
        self.mesh_obj(name, fill, self.rust, smooth=False)

    def rib(self):
        """The signature: one great grown-wood rib from the ridge's front end, sweeping out
        and down past the front-left corner to a flared root in the ground."""
        # It follows the left roof edge from the ridge, then drops in front of the wall to a
        # root left of the window, as in the concept's hero view.
        y = -HALF_L - OVERHANG - 0.16
        pts = [(0.25, y, RIDGE + SOD + 0.18), (-1.0, y, roof_z(1.0) + SOD + 0.12), (-2.0, y - 0.02, roof_z(2.0) + SOD + 0.02),
               (-2.75, y - 0.08, 2.75), (-2.95, y - 0.16, 1.75), (-2.85, y - 0.24, 0.8), (-2.6, y - 0.3, -0.05)]
        radii = [0.13, 0.15, 0.17, 0.19, 0.2, 0.23, 0.34]
        self.mesh_obj("GrownRib", lambda bm: self.tube(bm, pts, radii, 10), self.grown)
        # Lesser grown barge ribs along the other gable edges.
        for name, yy, sx in (("BargeFrontRight", y + 0.08, 1), ("BargeBackLeft", -y - 0.08, -1),
                             ("BargeBackRight", -y - 0.08, 1)):
            reach = HALF_W + OVERHANG
            pts2 = [(sx * x, yy, roof_z(x) + SOD + 0.08 + 0.12 * math.sin(math.pi * x / reach)) for x in
                    (0.0, reach * 0.3, reach * 0.6, reach * 0.85, reach)]
            self.mesh_obj(name, lambda bm, p=pts2: self.tube(bm, p, [0.12, 0.12, 0.11, 0.1, 0.09], 8), self.grown)

    def openings(self):
        """Door, windows and their frames, proud of the walls; the glass glows amber."""
        fy = -HALF_L - 0.02

        def framed(name, x, y, z0, z1, width, facing):
            # facing: 'front' (-Y), 'back' (+Y), 'left' (-X), 'right' (+X)
            t = 0.08
            if facing in ("front", "back"):
                s = -1 if facing == "front" else 1
                self.box(f"{name}Glass", (x, y + s * 0.01, (z0 + z1) / 2), (width, 0.04, z1 - z0), self.window)
                for dz, zz in ((t, z0), (t, z1)):
                    self.box(f"{name}FrameH{zz:.1f}", (x, y + s * 0.04, zz), (width + 2 * t, 0.1, dz), self.timber)
                for xx in (x - width / 2 - t / 2, x + width / 2 + t / 2):
                    self.box(f"{name}FrameV{xx:.1f}", (xx, y + s * 0.04, (z0 + z1) / 2), (t, 0.1, z1 - z0 + t), self.timber)
                self.box(f"{name}Mullion", (x, y + s * 0.035, (z0 + z1) / 2), (0.05, 0.06, z1 - z0), self.timber)
            else:
                s = -1 if facing == "left" else 1
                self.box(f"{name}Glass", (x + s * 0.01, y, (z0 + z1) / 2), (0.04, width, z1 - z0), self.window)
                for zz in (z0, z1):
                    self.box(f"{name}FrameH{zz:.1f}", (x + s * 0.04, y, zz), (0.1, width + 2 * t, t), self.timber)
                for yy in (y - width / 2 - t / 2, y + width / 2 + t / 2):
                    self.box(f"{name}FrameV{yy:.1f}", (x + s * 0.04, yy, (z0 + z1) / 2), (0.1, t, z1 - z0 + t), self.timber)
        framed("WindowFront", -1.65, fy, 1.05, 2.0, 1.2, "front")
        framed("WindowGable", 0.0, fy - 0.12, 3.35, 4.25, 0.9, "front")
        framed("WindowRight", HALF_W + 0.02, -1.6, 1.05, 2.0, 1.1, "right")
        framed("WindowLeft", -HALF_W - 0.02, 1.2, 1.05, 2.0, 1.1, "left")
        framed("WindowBack", 0.8, HALF_L + 0.02, 1.2, 1.9, 0.8, "back")
        # The door: boarded timber with iron straps, a stone step.
        dx = 0.95
        self.box("Door", (dx, fy - 0.03, PLINTH + 1.0), (1.0, 0.08, 2.0), self.timber)
        for zz in (0.75, 1.55, 2.05):
            self.box(f"DoorStrap{zz}", (dx, fy - 0.08, zz), (0.95, 0.03, 0.06), self.rust)
        self.box("DoorFrame", (dx, fy - 0.02, PLINTH + 2.06), (1.25, 0.14, 0.14), self.timber)
        self.box("Step", (dx, fy - 0.45, 0.1), (1.4, 0.7, 0.2), self.stone)
        self.node("Door", (dx, fy - 0.3, PLINTH), role="door")

    def battery(self):
        x, y = -0.25, -HALF_L - 0.3
        self.box("Battery", (x, y, PLINTH + 0.65), (0.7, 0.45, 1.3), self.titanium)
        self.box("BatteryVent", (x, y - 0.235, PLINTH + 0.95), (0.5, 0.02, 0.35), self.graphene)
        # The amber signal: a small soft glyph (a bolt) on the door of the cabinet.
        bolt = [(x - 0.03, PLINTH + 0.62), (x + 0.06, PLINTH + 0.62), (x - 0.01, PLINTH + 0.48),
                (x + 0.04, PLINTH + 0.48), (x - 0.06, PLINTH + 0.3), (x - 0.0, PLINTH + 0.45),
                (x - 0.05, PLINTH + 0.45)]
        self.ribbon("BatteryBolt", [(px, y - 0.24, pz) for px, pz in bolt], 0.025, self.signal, up=(0, -1, 0))
        # Strapped to the wall with iron bands (the join shows).
        for zz in (PLINTH + 0.3, PLINTH + 1.05):
            self.box(f"BatteryStrap{zz:.2f}", (x, y + 0.02, zz), (0.78, 0.5, 0.05), self.rust)

    def solar(self):
        """A row of graphene panels on titanium rails over the left slope, on timber stubs
        strapped to the sod roof with iron bands."""
        lift = 0.22
        x_hi, x_lo = -0.55, -2.15
        z_hi, z_lo = roof_z(x_hi) + SOD + lift, roof_z(x_lo) + SOD + lift
        y_start, panel_l, gap = -3.3, 1.12, 0.08
        for k in range(4):
            ya = y_start + k * (panel_l + gap)
            def panel(bm, ya=ya):
                v = [bm.verts.new(c) for c in ((x_lo, ya, z_lo), (x_lo, ya + panel_l, z_lo),
                                               (x_hi, ya + panel_l, z_hi), (x_hi, ya, z_hi))]
                bm.faces.new(v)
                bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.05)
            self.mesh_obj(f"Solar{k}", panel, self.graphene, smooth=False)
        y_end = y_start + 4 * (panel_l + gap)
        for t, name in ((0.15, "Low"), (0.85, "High")):
            x = x_lo + (x_hi - x_lo) * t
            z = z_lo + (z_hi - z_lo) * t - 0.06
            self.pipe(f"SolarRail{name}", [(x, y_start - 0.15, z), (x, y_end + 0.1, z)], 0.035, self.titanium, 6)
            for k, y in enumerate((y_start, y_start + 2.4, y_end - 0.05)):
                base = roof_z(x) + SOD * 0.8
                self.box(f"SolarStub{name}{k}", (x, y, (base + z) / 2), (0.12, 0.12, z - base), self.timber)
                self.box(f"SolarBand{name}{k}", (x, y, z - 0.02), (0.16, 0.16, 0.04), self.rust)

    def chimney(self):
        x, y = 1.15, 2.7
        base = roof_z(x) + SOD * 0.7
        top = RIDGE + 1.15
        self.box("ChimneyBoot", (x, y, base + 0.12), (0.55, 0.55, 0.3), self.rust)
        self.pipe("Chimney", [(x, y, base), (x, y, top)], 0.17, self.rust, 10)
        for zz in (base + 0.6, top - 0.35):
            self.pipe(f"ChimneyBand{zz:.1f}", [(x, y, zz - 0.03), (x, y, zz + 0.03)], 0.19, self.titanium, 10)

        def cap(bm):
            ring = [bm.verts.new((x + math.cos(a) * 0.3, y + math.sin(a) * 0.3, top + 0.12))
                    for a in (math.tau * k / 10 for k in range(10))]
            apex = bm.verts.new((x, y, top + 0.38))
            for i in range(10):
                bm.faces.new((ring[i], ring[(i + 1) % 10], apex))
            bm.faces.new(ring[::-1])
        self.mesh_obj("ChimneyCap", cap, self.titanium, smooth=False)
        for a in (0.3, 2.4, 4.4):
            self.pipe(f"ChimneyLeg{a}", [(x + math.cos(a) * 0.16, y + math.sin(a) * 0.16, top),
                                         (x + math.cos(a) * 0.26, y + math.sin(a) * 0.26, top + 0.13)], 0.015, self.titanium, 4)
        self.node("Chimney_top", (x, y, top + 0.45), role="smoke")

    def water(self):
        """A gutter along the left eave into a caged tank at the front-left corner."""
        reach = HALF_W + OVERHANG
        gz = roof_z(reach) - 0.12
        gx = -reach + 0.05
        self.pipe("Gutter", [(gx, HALF_L + OVERHANG, gz), (gx, -HALF_L - 0.1, gz - 0.08)], 0.08, self.titanium, 8)
        tx, ty = -HALF_W - 0.75, -HALF_L + 0.75
        self.pipe("Downpipe", [(gx, -HALF_L - 0.1, gz - 0.08), (gx, -HALF_L - 0.1, 1.75),
                               (tx + 0.2, ty - 0.1, 1.45)], 0.06, self.titanium, 8)
        # The tank: a pallet, the water, a titanium cage.
        self.box("TankPallet", (tx, ty, 0.08), (1.1, 1.25, 0.16), self.timber)
        self.box("TankWater", (tx, ty, 0.72), (0.98, 1.12, 1.08), self.tank_water)
        def cage(bm):
            hx, hy, z0, z1 = 0.52, 0.6, 0.16, 1.3
            for x in (-hx, -hx / 3, hx / 3, hx):
                for y in (-hy, hy):
                    self.tube(bm, [(tx + x, ty + y, z0), (tx + x, ty + y, z1)], [0.018, 0.018], 4)
            for y in (-hy / 3, hy / 3):
                for x in (-hx, hx):
                    self.tube(bm, [(tx + x, ty + y, z0), (tx + x, ty + y, z1)], [0.018, 0.018], 4)
            for z in (z0 + 0.02, (z0 + z1) / 2, z1):
                corners = [(tx - hx, ty - hy, z), (tx + hx, ty - hy, z), (tx + hx, ty + hy, z), (tx - hx, ty + hy, z)]
                for a, b in zip(corners, corners[1:] + corners[:1]):
                    self.tube(bm, [a, b], [0.02, 0.02], 4)
        self.mesh_obj("TankCage", cage, self.titanium, smooth=False)
        # The filter column beside it, banded with iron.
        fx, fy = tx + 0.15, ty + 0.95
        self.pipe("FilterColumn", [(fx, fy, 0.0), (fx, fy, 1.35)], 0.13, self.titanium, 10)
        self.pipe("FilterCore", [(fx, fy, 0.35), (fx, fy, 1.1)], 0.135, self.graphene, 10)
        self.pipe("FilterOut", [(fx, fy - 0.1, 0.3), (tx, ty + 0.3, 0.3)], 0.03, self.titanium, 6)

    def turbine(self):
        """A vertical-axis wind turbine on a salvaged mast at the rear-left corner."""
        mx, my = -HALF_W - 0.75, HALF_L + 0.45
        top = 7.3
        self.pipe("Mast", [(mx, my, 0.0), (mx, my, top - 1.3)], 0.08, self.rust, 8)
        self.pipe("MastTop", [(mx, my, top - 1.3), (mx, my, top + 0.15)], 0.045, self.titanium, 8)
        for zz in (0.9, 2.2, EAVE - 0.2):
            self.box(f"MastBand{zz:.1f}", (mx, my, zz), (0.22, 0.22, 0.06), self.titanium)
        self.box("MastBrace", ((mx - HALF_W + 0.05) / 2, my - 0.2, EAVE - 0.2), (abs(mx + HALF_W - 0.05), 0.08, 0.08), self.timber)
        self.box("MastFoot", (mx, my, 0.1), (0.5, 0.5, 0.2), self.stone)
        for k in range(3):
            a0 = math.tau * k / 3
            pts = []
            for i in range(9):
                t = i / 8
                r = 0.08 + 0.55 * math.sin(math.pi * t)
                a = a0 + 0.5 * t
                pts.append((mx + math.cos(a) * r, my + math.sin(a) * r, top - 1.25 + 1.35 * t))
            self.ribbon(f"Blade{k}", pts, 0.16, self.titanium, up=(math.cos(a0 + 1.57), math.sin(a0 + 1.57), 0))
        for zz in (top - 1.25, top + 0.1):
            self.pipe(f"Hub{zz:.1f}", [(mx, my, zz - 0.05), (mx, my, zz + 0.05)], 0.1, self.graphene, 8)

    def porch(self):
        """A porch along the right side, shaded by reed mats on pole rafters."""
        x0, x1 = HALF_W, HALF_W + 2.3
        y0, y1 = -HALF_L - 0.3, 1.2
        z_in, z_out = EAVE - 0.05, 2.35
        self.box("PorchFloor", ((x0 + x1) / 2, (y0 + y1) / 2, 0.1), (x1 - x0, y1 - y0, 0.2), self.timber)
        posts = (y0 + 0.15, (y0 + y1) / 2, y1 - 0.15)
        for k, y in enumerate(posts):
            self.pipe(f"PorchPost{k}", [(x1 - 0.15, y, 0.0), (x1 - 0.13, y, z_out + 0.05)], 0.11, self.grown, 8)
        self.pipe("PorchBeam", [(x1 - 0.15, y0 - 0.1, z_out), (x1 - 0.15, y1 + 0.1, z_out)], 0.1, self.timber, 8)
        rafters = 6
        for k in range(rafters):
            y = y0 + (y1 - y0) * k / (rafters - 1)
            self.pipe(f"PorchRafter{k}", [(x0, y, z_in), (x1 + 0.1, y, z_out - 0.02)], 0.05, self.timber, 6)
            self.box(f"PorchLash{k}", (x1 - 0.15, y, z_out + 0.02), (0.24, 0.12, 0.12), self.rust)

        def mat(bm):
            rows = 22
            for i in range(rows):
                ya, yb = y0 + (y1 - y0) * i / rows, y0 + (y1 - y0) * (i + 1) / rows
                lift = 0.015 * (i % 2)
                v = [bm.verts.new((x0, ya, z_in + 0.06 + lift)), bm.verts.new((x1 + 0.15, ya, z_out + 0.04 + lift)),
                     bm.verts.new((x1 + 0.15, yb, z_out + 0.04 + lift)), bm.verts.new((x0, yb, z_in + 0.06 + lift))]
                bm.faces.new(v)
            bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.03)
        self.mesh_obj("PorchShade", mat, self.reed, smooth=False)

    def chimes(self):
        """Wind chimes of scrap tubes hanging from a bracket off the front wall."""
        x, y, z = -0.95, -HALF_L - 0.55, 2.45
        self.pipe("ChimeArm", [(x, -HALF_L, z + 0.15), (x, y, z + 0.15)], 0.025, self.rust, 6)
        self.pipe("ChimeRing", [(x, y, z + 0.02), (x, y, z + 0.06)], 0.13, self.timber, 10)
        for k, length in enumerate((0.55, 0.42, 0.62, 0.35, 0.48)):
            a = math.tau * k / 5
            cx, cy = x + math.cos(a) * 0.1, y + math.sin(a) * 0.1
            mat = self.titanium if k % 2 else self.rust
            self.pipe(f"Chime{k}", [(cx, cy, z - 0.05), (cx, cy, z - 0.05 - length)], 0.018, mat, 6)

    def fungi(self):
        """Soft cyan fungi at the wall base: stems and glowing caps in a faint shell."""
        spots = [(-HALF_W - 0.25, -2.2), (-HALF_W - 0.3, -1.85), (-HALF_W - 0.2, 0.4), (-1.15, -HALF_L - 0.3),
                 (-0.95, -HALF_L - 0.45), (HALF_W + 2.45, 1.4), (1.9, -HALF_L - 0.35)]

        def stems(bm):
            for x, y in spots:
                self.tube(bm, [(x, y, 0.0), (x, y, 0.16)], [0.025, 0.02], 6)

        def caps(bm, scale=1.0):
            for x, y in spots:
                ring = [bm.verts.new((x + math.cos(a) * 0.09 * scale, y + math.sin(a) * 0.09 * scale, 0.15))
                        for a in (math.tau * k / 8 for k in range(8))]
                top = bm.verts.new((x, y, 0.15 + 0.07 * scale))
                for i in range(8):
                    bm.faces.new((ring[i], ring[(i + 1) % 8], top))
                bm.faces.new(ring[::-1])
        self.mesh_obj("FungusStems", stems, self.stem)
        self.mesh_obj("FungusCaps", caps, self.cap)
        self.mesh_obj("FungusHalos", lambda bm: caps(bm, 2.0), self.cap_halo)

    # ------------------------------------------------------------ the far level

    def far(self):
        objs = []
        reach = HALF_W + OVERHANG
        y0, y1 = -HALF_L - OVERHANG, HALF_L + OVERHANG
        objs.append(self.box("FarWalls", (0, 0, EAVE / 2), (2 * HALF_W, 2 * HALF_L, EAVE), self.earth))
        objs.append(self.prism("FarGables", [(-HALF_W, EAVE), (HALF_W, EAVE), (0, RIDGE)],
                               -HALF_L, HALF_L, self.timber))
        for side in (-1, 1):
            outline = [(0, RIDGE + SOD), (side * reach, roof_z(reach) + SOD * 0.6), (side * reach, roof_z(reach) - 0.05)]
            if side > 0:
                outline = outline[::-1]
            objs.append(self.prism(f"FarRoof{side}", outline, y0, y1, self.moss))
        y = y0 - 0.16
        pts = [(0.25, y, RIDGE + SOD + 0.18), (-2.0, y, roof_z(2.0) + SOD), (-2.95, y - 0.16, 1.75), (-2.6, y - 0.3, -0.05)]
        objs.append(self.mesh_obj("FarRib", lambda bm: self.tube(bm, pts, [0.15, 0.18, 0.21, 0.32], 5), self.grown, smooth=False))

        def panels(bm):
            v = [bm.verts.new(c) for c in ((-2.15, -3.3, roof_z(-2.15) + SOD + 0.22), (-2.15, 1.5, roof_z(-2.15) + SOD + 0.22),
                                           (-0.55, 1.5, roof_z(-0.55) + SOD + 0.22), (-0.55, -3.3, roof_z(-0.55) + SOD + 0.22))]
            bm.faces.new(v)
        objs.append(self.mesh_obj("FarSolar", panels, self.graphene, smooth=False))
        objs.append(self.box("FarTank", (-HALF_W - 0.75, -HALF_L + 0.75, 0.65), (1.05, 1.2, 1.3), self.titanium))
        objs.append(self.mesh_obj("FarMast", lambda bm: self.tube(bm, [(-HALF_W - 0.75, HALF_L + 0.45, 0), (-HALF_W - 0.75, HALF_L + 0.45, 7.4)], [0.08, 0.05], 4), self.rust, smooth=False))
        objs.append(self.box("FarRotor", (-HALF_W - 0.75, HALF_L + 0.45, 6.7), (1.1, 0.1, 1.35), self.titanium))
        objs.append(self.mesh_obj("FarChimney", lambda bm: self.tube(bm, [(1.15, 2.7, roof_z(1.15)), (1.15, 2.7, RIDGE + 1.4)], [0.2, 0.2], 6), self.rust, smooth=False))

        def shade(bm):
            v = [bm.verts.new(c) for c in ((HALF_W, -HALF_L - 0.3, EAVE), (HALF_W + 2.45, -HALF_L - 0.3, 2.4),
                                           (HALF_W + 2.45, 1.2, 2.4), (HALF_W, 1.2, EAVE))]
            bm.faces.new(v)
            bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.05)
        objs.append(self.mesh_obj("FarPorch", shade, self.reed, smooth=False))
        for k, yy in enumerate((-HALF_L - 0.15, 1.05)):
            objs.append(self.box(f"FarPost{k}", (HALF_W + 2.15, yy, 1.2), (0.18, 0.18, 2.4), self.grown))
        objs.append(self.box("FarWindow", (-1.65, -HALF_L - 0.03, 1.5), (1.2, 0.04, 0.95), self.window))
        return objs

    # ------------------------------------------------------------ acoustics

    # Small parts stay out of the acoustic scene (the guide: no trim, clutter or moving
    # parts); the turbine's blades will turn.
    SMALL = ("Joints", "Chime", "Fungus", "Grass", "Strap", "Band", "Bolt", "Leg", "Lash", "Hub",
             "Frame", "Mullion", "Vent", "Rail", "Stub", "Blade", "Cage", "Filter", "Gutter",
             "Downpipe", "Pallet", "Mast", "Rafter")

    def tag_acoustics(self, near):
        """Mark each large near-level mesh with its material's acoustic class (glTF extras
        `acoustic`); returns the counts per class."""
        counts = {}
        for obj in near:
            mats = [s.material for s in obj.material_slots if s.material]
            if not mats or any(word in obj.name for word in self.SMALL):
                continue
            cls = self.acoustics[mats[0]]
            if cls:
                obj["acoustic"] = cls
                counts[cls] = counts.get(cls, 0) + 1
        return counts

    # ------------------------------------------------------------ build

    @staticmethod
    def triangles(objs):
        return sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in objs if o.type == "MESH")

    def build(self):
        self.walls()
        self.roof()
        self.rib()
        self.openings()
        self.battery()
        self.solar()
        self.chimney()
        self.water()
        self.turbine()
        self.porch()
        self.chimes()
        self.fungi()
        far = self.far()
        near = [o for o in self.collection.objects if o.type == "MESH" and o not in far]
        acoustic = self.tag_acoustics(near)
        self.lod_levels(far)
        width = 2 * (HALF_W + 0.75) + 2.3
        profile = {
            "id": "cottage", "model": f"{self.model_ref}#Scene0", "kind": "structure",
            "dwelling": "cottage",
            "about": "The first Dancer dwelling (add-settlement-economy: a cottage houses at most "
                     "four): an off-grid homestead of rammed earth, salvaged timber and a living "
                     "sod roof, with one great grown-wood rib sweeping from the ridge to the "
                     "ground; graphene solar on titanium rails, a vertical-axis turbine, a caged "
                     "rain tank with a filter column, a battery cabinet, a stovepipe and a "
                     "reed-shaded porch. Brief: docs/art/cottage.md.",
            "size_metres": {"width": round(width, 2), "depth": round(2 * (HALF_L + OVERHANG) + 0.4, 2),
                            "height": 7.45},
            "sockets": {"Door": "the doorway at threshold height", "Chimney_top": "where smoke rises"},
            "lights": {"Cottage_Window": "the windows' amber glow (emissive, authored at peak); "
                                         "the game may dim it by day and light it at night"},
            "acoustics": {
                "materials": {mat.name: cls for mat, cls in self.acoustics.items()},
                "tagged_meshes": acoustic,
                "about": "near-level meshes carry glTF extras {\"acoustic\": <preset>}; small trim, "
                         "clutter and the turbine blades are left out",
            },
            "triangles": {"near": self.triangles(near), "far": self.triangles(far)},
            "licence": "CC0-1.0: generated by tools/blender/create_cottage.py",
        }
        sun = {}

        def daylight():
            light = bpy.data.objects.new("PreviewSun", bpy.data.lights.new("PreviewSun", "SUN"))
            light.data.energy = 3.5
            light.rotation_euler = (math.radians(50), 0, math.radians(35))
            self.scene.collection.objects.link(light)
            sun["obj"] = light
            self.scene.camera.data.lens = 20
            far_root = bpy.data.objects.get("cottage_LOD1")
            for obj in ([far_root] + list(far_root.children_recursive)) if far_root else []:
                obj.hide_render = True
        report = self.finish(profile, aim=(0.2, 0, 2.6), camera_location=(5.4, -9.4, 3.6),
                             before_render=daylight)
        if bpy.app.background:
            self.elevations(sun["obj"])
        report.update(triangles=profile["triangles"])
        return report

    def elevations(self, sun):
        """Orthographic front, right, back, left and top views, and a night view, for
        comparing with the concept sheet."""
        out = PROJECT / "previews/structures/cottage"
        scene, cam = self.scene, self.scene.camera
        far_root = bpy.data.objects.get("cottage_LOD1")
        if far_root:
            far_root.hide_render = True
            for child in far_root.children_recursive:
                child.hide_render = True
        cam.data.type = "ORTHO"
        cam.data.ortho_scale = 12.0
        scene.render.resolution_x, scene.render.resolution_y = 1024, 768
        centre = Vector((0.6, 0, 3.2))
        views = {"front": (0, -1, 0), "right": (1, 0, 0), "back": (0, 1, 0), "left": (-1, 0, 0)}
        for name, d in views.items():
            cam.location = centre + Vector(d) * 30
            cam.rotation_euler = (centre - cam.location).to_track_quat("-Z", "Z").to_euler()
            scene.render.filepath = str(out / f"elevation_{name}.png")
            bpy.ops.render.render(write_still=True, scene=scene.name)
        cam.location = Vector((0.6, 0, 40))
        cam.rotation_euler = (0, 0, 0)
        scene.render.resolution_x, scene.render.resolution_y = 768, 1024
        scene.render.filepath = str(out / "top.png")
        bpy.ops.render.render(write_still=True, scene=scene.name)
        # Night: the sun down, the windows and fungi carry it.
        cam.data.type = "PERSP"
        cam.data.lens = 24
        scene.render.resolution_x = scene.render.resolution_y = 768
        cam.location = Vector((5.4, -9.4, 3.6))
        cam.rotation_euler = (Vector((0.2, 0, 2.6)) - cam.location).to_track_quat("-Z", "Y").to_euler()
        # The front-left: the solar row, the tank and the turbine face this way.
        cam.location = Vector((-6.4, -8.6, 4.4))
        cam.rotation_euler = (Vector((-0.6, 0, 2.8)) - cam.location).to_track_quat("-Z", "Y").to_euler()
        cam.data.lens = 20
        scene.render.filepath = str(out / "preview_left.png")
        bpy.ops.render.render(write_still=True, scene=scene.name)
        cam.location = Vector((5.4, -9.4, 3.6))
        cam.rotation_euler = (Vector((0.2, 0, 2.6)) - cam.location).to_track_quat("-Z", "Y").to_euler()
        cam.data.lens = 24
        sun.data.energy = 0.0
        for obj in scene.objects:
            if obj.type == "LIGHT" and obj is not sun:
                obj.data.energy *= 0.05
        scene.world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.08
        scene.render.filepath = str(out / "preview_night.png")
        bpy.ops.render.render(write_still=True, scene=scene.name)


print(json.dumps(Cottage().build()))
