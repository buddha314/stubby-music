"""The vardo: the player's base, a barrel-top living wagon drawn by the cyber-cob
(add-vardo-and-cyber-cob-assets 2.x).

Built from `docs/art/vardo-and-cyber-cob.md` (the written rules win over the images) and
its studies in `docs/art/vardo-cyber-cob/`: a compact Irish Traveller barrel-top wagon,
not a lavish Reading wagon. Dark forest-green planks with burgundy trim and cream-gold
folk shamrocks; a structural grown-wood barrel roof with moss and a few sprouts; four
wheels on two axles, the rear about 1.45 times the front; shafts and a turntable at the
towing end; the entrance at the rear: a half-door, removable steps, a pan box and a rack.
Light is restrained: a few soft cyan leaf glows, one narrow magenta ring on the chimney,
narrow cyan rim accents, and the warm amber of a lantern and the windows.

Frame (Blender, +Z up): length along Y, the towing end (shafts) at -Y, which glTF exports
as +Z; the entrance at +Y. Origin: the centre of the body's footprint, on the ground.

Game-facing nodes (empties, glTF extras):
  Door          the rear doorway, at sill height (role door)
  Chimney_top   where smoke rises (role smoke)
  Hitch         where the cob stands between the shafts (role hitch), for the pair
  vardo_LOD0 / vardo_LOD1   the near and far levels
Window glow is its own material (`Vardo_Window`), the lantern's its own (`Vardo_Lantern`).

Reproduce: blender -b --factory-startup -P tools/blender/create_vardo.py
Writes assets/structures/vardo/ (model, profile), assets-src/blender/structures/vardo.blend
and previews/structures/vardo/ (three-quarter, side, front, rear, top, night).
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
from create_cottage import Cottage  # noqa: E402
from module_kit import PROJECT  # noqa: E402

FLOOR = 1.15          # body floor (sill) height
WAIST = 1.65          # waist rail
EAVE = 2.3            # where the barrel roof springs
CROWN = 3.0           # roof crown (surface)
HALF_SILL = 0.85      # body half-width at the sill
HALF_EAVE = 0.95      # body half-width at the eave (the sides flare)
Y_FRONT, Y_REAR = -1.55, 1.45      # body ends
ROOF_REACH = 1.05     # roof half-width at the eave line
ROOF_OVER = 0.25      # roof overhang past the ends
FRONT_AXLE, REAR_AXLE = -0.95, 0.85
FRONT_R, REAR_R = 0.45, 0.65       # wheel radii (ratio 1.44)
REAR_TRACK = 1.04     # rear wheels' x
FRONT_TRACK = 0.78    # front wheels' x (under the body, free to lock)


def roof_arc(half, rise, n=13, z0=EAVE):
    """Points of the barrel arch, left eave to right eave: a half-ellipse."""
    return [(-half * math.cos(math.pi * i / (n - 1)), z0 + rise * math.sin(math.pi * i / (n - 1))) for i in range(n)]


class Vardo(Burbler):
    folder = "structures"
    box = Cottage.box
    tube = staticmethod(Cottage.tube)
    pipe = Cottage.pipe
    prism = Cottage.prism

    def __init__(self):
        Burbler.__init__(self, "vardo", kind="Vardo")
        for key in ("mine", "produces"):
            del self.root[key]
        self.root["kind"] = "structure"
        self.root["base"] = "vardo"
        self.rng = random.Random(1879)
        m = self.material
        self.green = m("PaintGreen", linear("#2f4a33"), roughness=0.85)      # dark forest green
        self.groove = m("PlankGroove", linear("#1e2e21"), roughness=0.9)
        self.burgundy = m("PaintBurgundy", linear("#6b2a28"), roughness=0.8)  # weathered trim
        self.gold = m("FolkGold", linear("#c4aa76"), roughness=0.6)           # brass: folk motifs
        self.timber = m("Timber", linear("#5a4a3a"), roughness=0.9)
        self.grown = m("GrownWood", linear("#4b4037"), roughness=0.85)
        self.shingle = m("Shingle", linear("#5e4d3c"), roughness=0.95)
        self.iron = m("Iron", linear("#2e2b33"), metallic=0.7, roughness=0.6)  # ridges: dark iron
        self.rust = m("RustIron", linear("#5e3a22"), metallic=0.5, roughness=0.85)
        self.moss = m("Moss", linear("#3f5a24"), roughness=1.0)
        self.leaf = m("Leaf", linear("#4f6a2a"), roughness=0.8)
        self.lace = m("Lace", linear("#e7e5dd"), roughness=0.9)
        self.wool = m("Wool", linear("#9c3b2a"), roughness=1.0)                # rust cable red
        self.wicker = m("Wicker", linear("#9a8460"), roughness=1.0)
        amber, cyan, magenta = linear("#ffb350"), linear("#33ffd9"), linear("#ff4dd9")
        self.window = m("Window", amber, emission=amber, strength=3.0)
        self.lantern = m("Lantern", amber, emission=amber, strength=8.0)
        self.lantern_halo = self.translucent("LanternHalo", amber, 0.8, 0.12)
        self.cyan = m("AccentCyan", cyan, emission=cyan, strength=0.6)
        self.magenta = m("AccentMagenta", magenta, emission=magenta, strength=2.0)
        self.leaf_glow = m("LeafGlow", cyan, emission=cyan, strength=3.0)
        self.leaf_halo = self.translucent("LeafGlowHalo", cyan, 0.7, 0.12)
        # Every material declares its acoustic class; None: small parts only.
        self.acoustics = {
            self.green: "wood", self.groove: None, self.burgundy: "wood", self.gold: None,
            self.timber: "wood", self.grown: "wood", self.shingle: "wood",
            self.iron: "metal", self.rust: "metal", self.moss: None, self.leaf: None,
            self.lace: None, self.wool: None, self.wicker: None, self.window: "glass",
            self.lantern: None, self.lantern_halo: None, self.cyan: None, self.magenta: None,
            self.leaf_glow: None, self.leaf_halo: None,
        }

    # ------------------------------------------------------------ wheels and running gear

    def ring(self, name, centre, radius, thickness, mat, sides=6, segments=28, axis="x"):
        cx, cy, cz = centre
        pts = [(cx, cy + math.cos(a) * radius, cz + math.sin(a) * radius)
               for a in (math.tau * k / segments for k in range(segments + 1))]
        if axis == "z":
            pts = [(cx + math.cos(a) * radius, cy + math.sin(a) * radius, cz)
                   for a in (math.tau * k / segments for k in range(segments + 1))]
        return self.mesh_obj(name, lambda bm: self.tube(bm, pts, [thickness] * len(pts), sides, cap=False), mat)

    def wheel(self, name, x, y, r, spokes):
        side = 1 if x > 0 else -1
        c = (x, y, r)
        self.ring(f"{name}Felloe", c, r * 0.93, 0.045, self.burgundy, 6)
        self.ring(f"{name}Tyre", c, r - 0.01, 0.028, self.iron, 5, 32)
        # A narrow cyan accent on the felloe's outer face, never a glowing disc.
        self.ring(f"{name}Accent", (x + side * 0.05, y, r), r * 0.9, 0.007, self.cyan, 4, 32)
        self.pipe(f"{name}Hub", [(x - 0.13, y, r), (x + 0.13, y, r)], 0.1, self.burgundy, 10)
        self.pipe(f"{name}HubCap", [(x + side * 0.13, y, r), (x + side * 0.17, y, r)], 0.06, self.iron, 8)

        def spoke_set(bm):
            for k in range(spokes):
                a = math.tau * k / spokes
                d = Vector((0, math.cos(a), math.sin(a)))
                self.tube(bm, [Vector(c) + d * 0.09, Vector(c) + d * (r * 0.9)], [0.022, 0.017], 5)
        self.mesh_obj(f"{name}Spokes", spoke_set, self.burgundy, smooth=False)

    def running_gear(self):
        # Chassis: two rails and cross members under the floor.
        for sx in (-1, 1):
            self.box(f"Rail{sx}", (sx * 0.5, (Y_FRONT + Y_REAR) / 2, FLOOR - 0.1), (0.12, Y_REAR - Y_FRONT + 0.1, 0.16), self.timber)
        for y in (Y_FRONT + 0.1, -0.2, Y_REAR - 0.1):
            self.box(f"Cross{y:.1f}", (0, y, FLOOR - 0.12), (1.5, 0.1, 0.12), self.timber)
        # Rear axle, springs and wheels.
        self.box("RearAxle", (0, REAR_AXLE, REAR_R), (2 * REAR_TRACK - 0.2, 0.1, 0.1), self.iron)
        for sx in (-1, 1):
            self.box(f"RearSpring{sx}", (sx * 0.5, REAR_AXLE, (REAR_R + FLOOR - 0.18) / 2), (0.08, 0.7, FLOOR - 0.18 - REAR_R), self.iron)
            self.wheel(f"Rear{'L' if sx < 0 else 'R'}", sx * REAR_TRACK, REAR_AXLE, REAR_R, 14)
        # Front: axle bed on a turntable (fifth wheel), wheels under the body.
        self.box("FrontAxle", (0, FRONT_AXLE, FRONT_R), (2 * FRONT_TRACK - 0.2, 0.1, 0.1), self.iron)
        self.box("FrontBed", (0, FRONT_AXLE, FRONT_R + 0.2), (1.3, 0.16, 0.14), self.timber)
        self.ring("Turntable", (0, FRONT_AXLE, FLOOR - 0.2), 0.34, 0.025, self.iron, 5, 24, axis="z")
        self.pipe("Kingpin", [(0, FRONT_AXLE, FRONT_R + 0.1), (0, FRONT_AXLE, FLOOR - 0.15)], 0.045, self.iron, 8)
        for sx in (-1, 1):
            self.wheel(f"Front{'L' if sx < 0 else 'R'}", sx * FRONT_TRACK, FRONT_AXLE, FRONT_R, 12)
        self.pipe("Perch", [(0, FRONT_AXLE, FRONT_R + 0.12), (0, REAR_AXLE, REAR_R + 0.08)], 0.05, self.timber, 6)
        # Shafts: from the front bed to the cob, narrowing and lifting to its shoulders.
        for sx in (-1, 1):
            pts = [(sx * 0.62, FRONT_AXLE - 0.1, FRONT_R + 0.22), (sx * 0.52, FRONT_AXLE - 1.1, 0.78),
                   (sx * 0.44, FRONT_AXLE - 2.2, 0.92), (sx * 0.42, FRONT_AXLE - 3.05, 0.98)]
            self.mesh_obj(f"Shaft{sx}", lambda bm, p=pts: self.tube(bm, p, [0.055, 0.05, 0.045, 0.04], 8), self.timber)
            tip = Vector(pts[-1])
            self.pipe(f"ShaftTip{sx}", [tip - Vector((0, 0.12, 0)), tip + Vector((0, -0.02, 0))], 0.048, self.iron, 8)
            self.ring(f"ShaftRing{sx}", (tip.x, tip.y - 0.04, tip.z - 0.07), 0.05, 0.012, self.iron, 4, 12)
        self.pipe("Swingletree", [(-0.55, FRONT_AXLE - 0.35, FRONT_R + 0.2), (0.55, FRONT_AXLE - 0.35, FRONT_R + 0.2)], 0.035, self.timber, 6)
        self.node("Hitch", (0, FRONT_AXLE - 2.4, 0), role="hitch")

    # ------------------------------------------------------------ the body

    def body(self):
        outline = [(-HALF_SILL, FLOOR), (HALF_SILL, FLOOR), (HALF_SILL, WAIST), (HALF_EAVE, EAVE),
                   (-HALF_EAVE, EAVE), (-HALF_SILL, WAIST)]
        self.prism("Body", outline, Y_FRONT, Y_REAR, self.green)
        # The end walls rise into the arch (lunettes) under the roof.
        arch = [(x * HALF_EAVE / ROOF_REACH, z) for x, z in roof_arc(ROOF_REACH, CROWN - EAVE - 0.08)]
        self.prism("LunetteFront", arch, Y_FRONT, Y_FRONT + 0.06, self.green)
        self.prism("LunetteRear", arch, Y_REAR - 0.06, Y_REAR, self.green)
        # Plank grooves on the upper sides.
        def grooves(bm):
            for side in (-1, 1):
                for k in range(1, 5):
                    z = WAIST + (EAVE - WAIST) * k / 5
                    x = side * (HALF_SILL + (HALF_EAVE - HALF_SILL) * k / 5 + 0.006)
                    v = [bm.verts.new((x, y, zz)) for y, zz in ((Y_FRONT + 0.05, z - 0.008), (Y_REAR - 0.05, z - 0.008),
                                                                 (Y_REAR - 0.05, z + 0.008), (Y_FRONT + 0.05, z + 0.008))]
                    bm.faces.new(v if side > 0 else v[::-1])
        self.mesh_obj("Grooves", grooves, self.groove, smooth=False)
        # Burgundy trim: sill, waist and eave rails; corner posts following the flare.
        for name, z, half, h in (("Sill", FLOOR, HALF_SILL + 0.05, 0.12), ("Waist", WAIST, HALF_SILL + 0.03, 0.07),
                                 ("Eave", EAVE, HALF_EAVE + 0.03, 0.08)):
            for sx in (-1, 1):
                self.box(f"{name}Rail{sx}", (sx * half, (Y_FRONT + Y_REAR) / 2, z), (0.06, Y_REAR - Y_FRONT + 0.08, h), self.burgundy)
        for sx in (-1, 1):
            for y in (Y_FRONT, Y_REAR):
                self.pipe(f"Corner{sx}{y:.1f}", [(sx * (HALF_SILL + 0.02), y, FLOOR), (sx * (HALF_SILL + 0.03), y, WAIST),
                                                  (sx * (HALF_EAVE + 0.02), y, EAVE)], 0.05, self.burgundy, 4)
        # Lower panels: framed in burgundy with gold lines and a shamrock each.
        ys = [Y_FRONT + 0.15, -0.55, 0.45, Y_REAR - 0.15]
        for sx in (-1, 1):
            x = sx * (HALF_SILL + 0.012)
            for k in range(3):
                y0, y1 = ys[k], ys[k + 1]
                yc, zc = (y0 + y1) / 2, (FLOOR + WAIST) / 2
                self.box(f"PanelPost{sx}{k}", (x, y0, zc), (0.03, 0.06, WAIST - FLOOR), self.burgundy)
                self.gold_frame(f"Panel{sx}{k}", x + sx * 0.004, yc, zc, y1 - y0 - 0.18, WAIST - FLOOR - 0.14, "x")
                self.shamrock(f"Shamrock{sx}{k}", (x + sx * 0.006, yc, zc), 0.07, "x", sx)
        # Front-end decoration (the towing end): a shamrock under the window.
        self.shamrock("ShamrockFront", (0, Y_FRONT - 0.012, FLOOR + 0.28), 0.08, "y", -1)

    def gold_frame(self, name, x, y, z, w, h, plane):
        t = 0.018
        for k, (dy, dz, sw, sh) in enumerate(((0, h / 2, w, t), (0, -h / 2, w, t), (w / 2, 0, t, h), (-w / 2, 0, t, h))):
            self.box(f"{name}Gold{k}", (x, y + dy, z + dz), (0.006, sw, sh), self.gold)

    def shamrock(self, name, centre, r, plane, facing):
        """Three heart-ish leaves and a stem, flat on a panel."""
        c = Vector(centre)

        def fill(bm):
            def disc(off, rr):
                p = c + off
                ring = []
                for k in range(10):
                    a = math.tau * k / 10
                    if plane == "x":
                        ring.append(bm.verts.new(p + Vector((0, math.cos(a) * rr, math.sin(a) * rr))))
                    else:
                        ring.append(bm.verts.new(p + Vector((math.cos(a) * rr, 0, math.sin(a) * rr))))
                bm.faces.new(ring if facing > 0 else ring[::-1])
            for a in (math.pi / 2, math.pi / 2 + 2.1, math.pi / 2 - 2.1):
                off = (Vector((0, math.cos(a) * r * 0.75, math.sin(a) * r * 0.75)) if plane == "x"
                       else Vector((math.cos(a) * r * 0.75, 0, math.sin(a) * r * 0.75)))
                disc(off, r * 0.55)
            stem = [Vector((0, 0, 0)), Vector((0, r * 0.25, -r * 1.3)) if plane == "x" else Vector((r * 0.25, 0, -r * 1.3))]
            w = Vector((0, 0.01, 0)) if plane == "x" else Vector((0.01, 0, 0))
            v = [bm.verts.new(c + p) for p in (stem[0] - w, stem[0] + w, stem[1] + w, stem[1] - w)]
            bm.faces.new(v if facing > 0 else v[::-1])
        self.mesh_obj(name, fill, self.gold, smooth=False)

    def windows(self):
        """Curtained windows on both sides and at the front; amber behind lace."""
        def window(name, centre, w, h, plane, facing):
            x, y, z = centre
            if plane == "x":
                self.box(f"{name}Glow", (x, y, z), (0.02, w, h), self.window)
                for k in (-1, 1):   # lace curtains drawn to each side
                    self.box(f"{name}Lace{k}", (x + facing * 0.012, y + k * w * 0.27, z), (0.01, w * 0.42, h * 0.95), self.lace)
                for dz in (-h / 2, h / 2, 0):
                    self.box(f"{name}Bar{dz:.2f}", (x + facing * 0.02, y, z + dz), (0.04, w + 0.08, 0.045 if dz else 0.025), self.burgundy)
                for dy in (-w / 2, w / 2, 0):
                    self.box(f"{name}Post{dy:.2f}", (x + facing * 0.02, y + dy, z), (0.04, 0.045 if dy else 0.025, h + 0.04), self.burgundy)
                self.box(f"{name}Sill", (x + facing * 0.05, y, z - h / 2 - 0.04), (0.08, w + 0.16, 0.04), self.burgundy)
            else:
                self.box(f"{name}Glow", (x, y, z), (w, 0.02, h), self.window)
                for k in (-1, 1):
                    self.box(f"{name}Lace{k}", (x + k * w * 0.27, y + facing * 0.012, z), (w * 0.42, 0.01, h * 0.95), self.lace)
                for dz in (-h / 2, h / 2, 0):
                    self.box(f"{name}Bar{dz:.2f}", (x, y + facing * 0.02, z + dz), (w + 0.08, 0.04, 0.045 if dz else 0.025), self.burgundy)
                for dx in (-w / 2, w / 2, 0):
                    self.box(f"{name}Post{dx:.2f}", (x + dx, y + facing * 0.02, z), (0.045 if dx else 0.025, 0.04, h + 0.04), self.burgundy)
                self.box(f"{name}Sill", (x, y + facing * 0.05, z - h / 2 - 0.04), (w + 0.16, 0.08, 0.04), self.burgundy)
        zc = (WAIST + EAVE) / 2 + 0.02
        for sx in (-1, 1):
            x = sx * ((HALF_SILL + HALF_EAVE) / 2 + 0.02)
            window(f"WindowSide{sx}", (x, -0.05, zc), 0.6, 0.5, "x", sx)
        window("WindowFront", (0, Y_FRONT - 0.01, 2.15), 0.62, 0.52, "y", -1)

    def rear(self):
        """The entrance end: an arched door frame, a half-door with its top leaf open,
        removable steps, a pan box with pans, a rack with a basket and a blanket, and a lantern."""
        y = Y_REAR + 0.01
        w, top = 0.72, 2.62
        self.box("DoorDark", (0, y - 0.03, (FLOOR + top) / 2), (w, 0.02, top - FLOOR), self.groove)
        self.box("InteriorGlow", (0, y - 0.06, 2.25), (w * 0.8, 0.02, 0.45), self.window)
        for sx in (-1, 1):
            self.box(f"DoorJamb{sx}", (sx * (w / 2 + 0.04), y + 0.02, (FLOOR + top) / 2), (0.08, 0.06, top - FLOOR), self.burgundy)
        arc = [(math.cos(math.pi * i / 8) * (w / 2 + 0.04), y + 0.02, top + math.sin(math.pi * i / 8) * 0.18) for i in range(9)]
        self.mesh_obj("DoorArch", lambda bm: self.tube(bm, arc, [0.045] * 9, 6), self.burgundy)
        self.box("DoorLower", (0, y + 0.03, FLOOR + 0.42), (w, 0.05, 0.82), self.burgundy)
        self.shamrock("ShamrockDoor", (0, y + 0.06, FLOOR + 0.45), 0.08, "y", 1)
        # The top leaf stands open inward: the lit interior shows above the lower door.
        for zz in (FLOOR + 0.18, FLOOR + 0.66):
            self.box(f"Hinge{zz:.2f}", (w / 2 - 0.08, y + 0.06, zz), (0.2, 0.02, 0.04), self.iron)
        self.node("Door", (0, y + 0.3, FLOOR), role="door")
        # Removable steps from the sill to the ground.
        foot = Y_REAR + 0.95
        for sx in (-1, 1):
            self.pipe(f"Stringer{sx}", [(sx * 0.3, Y_REAR + 0.05, FLOOR), (sx * 0.3, foot, 0.0)], 0.035, self.timber, 4)
        for k in range(1, 4):
            t = k / 4
            self.box(f"Tread{k}", (0, Y_REAR + 0.05 + (foot - Y_REAR - 0.05) * t, FLOOR * (1 - t) + 0.02), (0.66, 0.2, 0.04), self.timber)
        # Pan box on the right, pans hanging above it.
        bx = HALF_SILL - 0.12
        self.box("PanBox", (bx, Y_REAR + 0.2, FLOOR + 0.25), (0.42, 0.32, 0.3), self.rust)
        self.box("PanBoxLid", (bx, Y_REAR + 0.2, FLOOR + 0.42), (0.46, 0.36, 0.03), self.iron)
        for k, (px, r) in enumerate(((bx - 0.12, 0.11), (bx + 0.1, 0.09))):
            pz = 2.02 - k * 0.05
            self.pipe(f"Pan{k}", [(px, Y_REAR + 0.08, pz), (px, Y_REAR + 0.11, pz)], r, self.iron, 12)
            self.pipe(f"PanHandle{k}", [(px, Y_REAR + 0.1, pz + r), (px, Y_REAR + 0.1, pz + r + 0.18)], 0.012, self.iron, 4)
        # Rack on the left: a ladder with a basket and a folded wool blanket.
        rx = -HALF_SILL + 0.1
        for dy in (0.1, 0.32):
            self.pipe(f"Ladder{dy}", [(rx - 0.12, Y_REAR + dy, FLOOR + 0.05), (rx - 0.12, Y_REAR + dy, FLOOR + 1.0)], 0.02, self.timber, 4)
        for k in range(4):
            self.pipe(f"Rung{k}", [(rx - 0.12, Y_REAR + 0.1, FLOOR + 0.2 + 0.22 * k), (rx - 0.12, Y_REAR + 0.32, FLOOR + 0.2 + 0.22 * k)], 0.012, self.timber, 4)
        self.pipe("Basket", [(rx, Y_REAR + 0.2, FLOOR + 0.12), (rx, Y_REAR + 0.2, FLOOR + 0.42)], 0.14, self.wicker, 10)
        self.box("Blanket", (rx - 0.02, Y_REAR + 0.22, FLOOR + 0.58), (0.3, 0.28, 0.1), self.wool)
        # The lantern on a bracket at the rear right corner.
        lx, ly, lz = HALF_EAVE + 0.12, Y_REAR + 0.05, 2.05
        self.pipe("LanternArm", [(HALF_EAVE, ly, lz + 0.32), (lx, ly, lz + 0.32), (lx, ly, lz + 0.2)], 0.012, self.iron, 4)
        self.pipe("LanternCage", [(lx, ly, lz - 0.1), (lx, ly, lz + 0.16)], 0.065, self.iron, 6)
        self.glow_point("LanternFlame", (lx, ly, lz + 0.02), 0.025, 0.045, self.lantern, self.lantern_halo,
                        light="lantern")

    # ------------------------------------------------------------ the roof

    def roof(self):
        rise = CROWN - EAVE
        outer = roof_arc(ROOF_REACH, rise)
        inner = roof_arc(ROOF_REACH - 0.1, rise - 0.1)
        self.prism("Roof", outer + inner[::-1], Y_FRONT - ROOF_OVER, Y_REAR + ROOF_OVER, self.shingle)
        y0, y1 = Y_FRONT - ROOF_OVER, Y_REAR + ROOF_OVER
        # Grown ribs over the roof, thick at the ends, and root-like at the corners where the
        # end ribs grow down the corner posts into the sill.
        for k, y in enumerate((y0 + 0.02, -0.85, -0.05, 0.75, y1 - 0.02)):
            end = k in (0, 4)
            pts = [(x * 1.02, y, z + 0.04) for x, z in roof_arc(ROOF_REACH, rise + 0.03, n=11)]
            if end:
                for sx in (-1, 1):
                    tail = [(sx * (HALF_EAVE + 0.06), y, EAVE - 0.25), (sx * (HALF_SILL + 0.1), y, WAIST),
                            (sx * (HALF_SILL + 0.13), y, FLOOR + 0.1)]
                    pts = (tail[::-1] + pts) if sx < 0 else (pts + tail)
            radii = [0.085 if end else 0.05] * len(pts)
            if end:
                radii[0] = radii[-1] = 0.12
            self.mesh_obj(f"Rib{k}", lambda bm, p=pts, r=radii: self.tube(bm, p, r, 8), self.grown)
        # Twisting tendrils across the end walls, as in the front and rear studies.
        for y, name in ((Y_FRONT - 0.04, "Front"), (Y_REAR + 0.04, "Rear")):
            for sx in (-1, 1):
                pts = [(sx * (HALF_EAVE - 0.05), y, EAVE - 0.1), (sx * 0.62, y, EAVE + 0.15),
                       (sx * 0.5, y, EAVE + 0.45), (sx * 0.3, y, CROWN - 0.15)]
                self.mesh_obj(f"Tendril{name}{sx}", lambda bm, p=pts: self.tube(bm, p, [0.04, 0.035, 0.03, 0.02], 6), self.grown)
        # Moss along the roof and leaf sprays, a few with a soft cyan glow.
        def moss(bm):
            for _ in range(26):
                i = self.rng.uniform(0.15, 0.85)
                a = math.pi * i
                x, z = -ROOF_REACH * math.cos(a), EAVE + rise * math.sin(a)
                y = self.rng.uniform(y0 + 0.2, y1 - 0.2)
                n = Vector((-math.cos(a) * rise, 0, math.sin(a) * ROOF_REACH)).normalized()
                c = Vector((x, y, z)) + n * 0.02
                r = self.rng.uniform(0.12, 0.26)
                t1 = Vector((0, 1, 0))
                t2 = n.cross(t1)
                ring = [bm.verts.new(c + (t1 * math.cos(b) + t2 * math.sin(b)) * r) for b in (math.tau * k / 7 for k in range(7))]
                top = bm.verts.new(c + n * r * 0.4)
                for k in range(7):
                    bm.faces.new((ring[k], ring[(k + 1) % 7], top))
        self.mesh_obj("Moss", moss, self.moss)

        def leaves(bm, glow=False):
            count = 5 if glow else 70
            for _ in range(count):
                i = self.rng.uniform(0.1, 0.9)
                a = math.pi * i
                y = self.rng.choice((y0 + 0.05, y1 - 0.05, self.rng.uniform(y0, y1)))
                base = Vector((-ROOF_REACH * 1.03 * math.cos(a), y, EAVE + (rise + 0.05) * math.sin(a)))
                n = Vector((-math.cos(a), 0, math.sin(a))).normalized()
                for _ in range(2 if glow else 3):
                    d = (n + Vector((self.rng.uniform(-0.6, 0.6), self.rng.uniform(-0.6, 0.6), 0.3))).normalized()
                    side = d.cross(Vector((0, 0, 1)))
                    if side.length < 1e-3:
                        side = Vector((1, 0, 0))
                    side = side.normalized() * 0.035
                    tip = base + d * self.rng.uniform(0.1, 0.16)
                    mid = base.lerp(tip, 0.5)
                    v = [bm.verts.new(p) for p in (base, mid + side, tip, mid - side)]
                    bm.faces.new(v)
        self.mesh_obj("Leaves", leaves, self.leaf, smooth=False)
        self.mesh_obj("GlowLeaves", lambda bm: leaves(bm, glow=True), self.leaf_glow, smooth=False)
        for k, (i, y) in enumerate(((0.3, y0 + 0.08), (0.7, -0.4), (0.45, y1 - 0.08))):
            a = math.pi * i
            p = (-ROOF_REACH * 1.05 * math.cos(a), y, EAVE + (rise + 0.1) * math.sin(a))
            self.glow_point(f"LeafLight{k}", p, 0.01, 0.03, self.leaf_glow, self.leaf_halo)

    def chimney(self):
        x, y = 0.42, Y_FRONT + 0.6
        a = math.acos(-x / ROOF_REACH)
        base = EAVE + (CROWN - EAVE) * math.sin(a)
        top = base + 0.55
        self.pipe("Chimney", [(x, y, base - 0.05), (x, y, top)], 0.08, self.iron, 10)
        self.ring("ChimneyRing", (x, y, base + 0.3), 0.088, 0.01, self.magenta, 4, 16, axis="z")
        def cap(bm):
            ring = [bm.verts.new((x + math.cos(b) * 0.15, y + math.sin(b) * 0.15, top + 0.08)) for b in (math.tau * k / 10 for k in range(10))]
            apex = bm.verts.new((x, y, top + 0.22))
            for k in range(10):
                bm.faces.new((ring[k], ring[(k + 1) % 10], apex))
            bm.faces.new(ring[::-1])
        self.mesh_obj("ChimneyCap", cap, self.iron, smooth=False)
        for b in (0.4, 2.5, 4.6):
            self.pipe(f"ChimneyLeg{b}", [(x + math.cos(b) * 0.07, y + math.sin(b) * 0.07, top), (x + math.cos(b) * 0.13, y + math.sin(b) * 0.13, top + 0.09)], 0.01, self.iron, 4)
        self.node("Chimney_top", (x, y, top + 0.3), role="smoke")

    # ------------------------------------------------------------ the far level

    def far(self):
        objs = []
        outline = [(-HALF_SILL, FLOOR), (HALF_SILL, FLOOR), (HALF_EAVE, EAVE), (-HALF_EAVE, EAVE)]
        objs.append(self.prism("FarBody", outline, Y_FRONT, Y_REAR, self.green))
        objs.append(self.prism("FarRoof", roof_arc(ROOF_REACH, CROWN - EAVE, n=7), Y_FRONT - ROOF_OVER, Y_REAR + ROOF_OVER, self.shingle))
        for name, x, y, r in (("RL", -REAR_TRACK, REAR_AXLE, REAR_R), ("RR", REAR_TRACK, REAR_AXLE, REAR_R),
                              ("FL", -FRONT_TRACK, FRONT_AXLE, FRONT_R), ("FR", FRONT_TRACK, FRONT_AXLE, FRONT_R)):
            objs.append(self.mesh_obj(f"FarWheel{name}", lambda bm, x=x, y=y, r=r: self.tube(bm, [(x - 0.04, y, r), (x + 0.04, y, r)], [r, r], 8), self.burgundy, smooth=False))
        for sx in (-1, 1):
            objs.append(self.mesh_obj(f"FarShaft{sx}", lambda bm, s=sx: self.tube(bm, [(s * 0.62, FRONT_AXLE - 0.1, FRONT_R + 0.22), (s * 0.42, FRONT_AXLE - 3.05, 0.98)], [0.05, 0.04], 4), self.timber, smooth=False))
        objs.append(self.box("FarUnder", (0, (Y_FRONT + Y_REAR) / 2, FLOOR - 0.12), (1.4, Y_REAR - Y_FRONT, 0.2), self.timber))
        objs.append(self.box("FarWindow", (HALF_SILL + 0.06, -0.05, 2.0), (0.02, 0.6, 0.5), self.window))
        objs.append(self.box("FarWindowL", (-HALF_SILL - 0.06, -0.05, 2.0), (0.02, 0.6, 0.5), self.window))
        return objs

    # ------------------------------------------------------------ build

    SMALL = ("Groove", "Gold", "Shamrock", "Lace", "Bar", "Post", "Sill", "Hinge", "Pan", "Ladder", "Rung",
             "Basket", "Blanket", "Lantern", "Leaves", "Leaf", "Moss", "Tendril", "Accent", "Ring", "Leg",
             "HubCap", "Spokes", "Kingpin", "Turntable", "Swingletree", "Stringer", "Tread", "Cap")

    def tag_acoustics(self, near):
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

    @staticmethod
    def triangles(objs):
        return sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in objs if o.type == "MESH")

    def build(self):
        self.running_gear()
        self.body()
        self.windows()
        self.rear()
        self.roof()
        self.chimney()
        far = self.far()
        near = [o for o in self.collection.objects if o.type == "MESH" and o not in far]
        acoustic = self.tag_acoustics(near)
        self.lod_levels(far)
        profile = {
            "id": "vardo", "model": f"{self.model_ref}#Scene0", "kind": "structure", "base": "vardo",
            "about": "The player's base (design guide: the vardo, our HUB): a compact Irish Traveller "
                     "barrel-top living wagon with a structural grown-wood roof, drawn by the cyber-cob. "
                     "Towing end (shafts) at the model's front, entrance at the rear. Brief: "
                     "docs/art/vardo-and-cyber-cob.md.",
            "size_metres": {"width": round(2 * (REAR_TRACK + 0.17), 2), "length": round(Y_REAR + 1.0 - (FRONT_AXLE - 3.2), 2),
                            "height": round(CROWN + 0.8, 2)},
            "wheels": {"front_radius": FRONT_R, "rear_radius": REAR_R, "ratio": round(REAR_R / FRONT_R, 2)},
            "sockets": {"Door": "the rear doorway at sill height", "Chimney_top": "where smoke rises",
                        "Hitch": "where the cob stands between the shafts"},
            "lights": {"Vardo_Window": "window and interior glow (authored at peak)",
                       "Vardo_Lantern": "the rear lantern (LanternFlame, extras light: lantern)"},
            "acoustics": {"materials": {mat.name: cls for mat, cls in self.acoustics.items()},
                          "tagged_meshes": acoustic,
                          "about": "large near-level meshes carry glTF extras acoustic; trim, motifs, "
                                   "foliage, gear and wheels' small parts are left out"},
            "triangles": {"near": self.triangles(near), "far": self.triangles(far)},
            "licence": "CC0-1.0: generated by tools/blender/create_vardo.py",
        }
        sun = {}

        def daylight():
            light = bpy.data.objects.new("PreviewSun", bpy.data.lights.new("PreviewSun", "SUN"))
            light.data.energy = 3.5
            light.rotation_euler = (math.radians(50), 0, math.radians(35))
            self.scene.collection.objects.link(light)
            sun["obj"] = light
            self.scene.camera.data.lens = 38
            far_root = bpy.data.objects.get("vardo_LOD1")
            for obj in ([far_root] + list(far_root.children_recursive)) if far_root else []:
                obj.hide_render = True
        report = self.finish(profile, aim=(0, -0.6, 1.5), camera_location=(5.4, -6.8, 2.6), before_render=daylight)
        if bpy.app.background:
            self.views(sun["obj"])
        report.update(triangles=profile["triangles"])
        return report

    def views(self, sun):
        out = PROJECT / "previews/structures/vardo"
        scene, cam = self.scene, self.scene.camera
        cam.data.type = "ORTHO"
        cam.data.ortho_scale = 8.0
        scene.render.resolution_x, scene.render.resolution_y = 1024, 640
        centre = Vector((0, -1.2, 1.9))
        for name, d in {"side": (1, 0, 0), "front": (0, -1, 0), "rear": (0, 1, 0), "side_left": (-1, 0, 0)}.items():
            cam.location = centre + Vector(d) * 30
            cam.rotation_euler = (centre - cam.location).to_track_quat("-Z", "Y").to_euler()
            cam.data.ortho_scale = 9.0 if name.startswith("side") else 6.8
            # The sun behind the camera for each view, so every face is seen lit.
            sun.rotation_euler = (centre - cam.location).to_track_quat("-Z", "Y").to_euler()
            sun.rotation_euler.x += math.radians(-25) if name != "rear" else math.radians(-25)
            scene.render.filepath = str(out / f"{name}.png")
            bpy.ops.render.render(write_still=True, scene=scene.name)
        cam.data.ortho_scale = 8.0
        cam.location, cam.rotation_euler = Vector((0, -1.2, 40)), (0, 0, 0)
        scene.render.resolution_x, scene.render.resolution_y = 640, 1024
        scene.render.filepath = str(out / "top.png")
        bpy.ops.render.render(write_still=True, scene=scene.name)
        # Rear three-quarter, and night.
        cam.data.type = "PERSP"
        cam.data.lens = 28
        scene.render.resolution_x = scene.render.resolution_y = 768
        cam.data.lens = 38
        cam.location = Vector((-4.2, 6.2, 2.6))
        cam.rotation_euler = (Vector((0, 0.6, 1.5)) - cam.location).to_track_quat("-Z", "Y").to_euler()
        sun.rotation_euler = (Vector((0, 0.6, 1.5)) - cam.location).to_track_quat("-Z", "Y").to_euler()
        scene.render.filepath = str(out / "preview_rear.png")
        bpy.ops.render.render(write_still=True, scene=scene.name)
        sun.data.energy = 0.0
        for obj in scene.objects:
            if obj.type == "LIGHT" and obj is not sun:
                obj.data.energy *= 0.05
        scene.world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.08
        scene.render.filepath = str(out / "preview_night.png")
        bpy.ops.render.render(write_still=True, scene=scene.name)


if __name__ == "__main__":
    print(json.dumps(Vardo().build()))
