"""Mudpit burbler: a gurgling hot-spring mud pot that is a clock mine (add-mudpit-burbler).

Two assets:

- `pit` -> mudpit_burbler: the burbler itself. A natural hot spring of mud that
  bubbles at its BPM; every burble is a beat. The player configures it through a
  Eurorack HUD (game-side UI), so nothing is built in the world.
- `rig` -> mudpit_burbler_rig: a concept kept for later. The same pit with a rack
  beside it, built from crude parts (sand, rusted iron, weathered wood) lashed to
  high-tech ones (titanium, graphene), a probe in the mud and a conduit to the rack.

The light is jellyfish and fairy light, not LEDs (docs/art/style-guide.md): wisps
rise from the bursts like small jellyfish, motes drift, and amber veins glow under
the mud. The models ship at rest; the game animates the named nodes from the
burbler's clock ticks.

  Bubble_<n>           extras: division (bursts on every beat / division)
  Bubble_<n>_Ripple    (pit) a ring the game spreads outward from each burst
  Wisp_<n>             extras: rise_m, life_s (rises from a burst and fades)
  Steam_<n>            (pit) extras: rise_m, life_s (drifts up, slow)
  Outlet_<n>           (pit) extras: stream, pulse. Where a mud stream leaves the pit; +X along the flow
  Outlet_<n>_Halo      (pit) lit only while something is connected (Burbler_OutletHalo)
  Mote_<n>             drifts; brightness breathes with the beat
  Interact             extras: action = configure (opens the Eurorack HUD)
  Lantern_<n>, Conduit_Bead_<n>, Slot_<place>   (rig only)

Reproduce: blender -b --factory-startup -P tools/blender/create_burbler.py [-- pit rig]
"""
from pathlib import Path
import math
import random
import sys

import bmesh
import bpy
from mathutils import Vector, noise

sys.path.insert(0, str(Path(__file__).resolve().parent))
from module_kit import HEIGHT, HP, PROJECT, ModuleKit, run  # noqa: E402

POOL_RADIUS = 1.15
POOL_Z = 0.12
SPRING_SCALE = 1.4  # the pit's pool is this much wider than the rig's
SPRING_POOL_Z = 0.05
OUTLET_ANGLE = math.radians(-50)  # where the mud stream leaves the pit (toward the approach)
RACK_HP = {"clock": 8, "vco": 8, "volume": 4}
RACK_WIDTH = sum(RACK_HP.values()) * HP
RACK_BASE = 0.30  # module base height in the rack
RACK_AT = (2.35, -0.25, 0.0)
RACK_YAW = math.radians(-20)  # turned a little toward the approach, pit on its left


def linear(hex_srgb):
    """Style-guide hex (sRGB) to the linear colour Blender materials take."""
    c = [int(hex_srgb[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    return tuple(v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4 for v in c)


class Burbler(ModuleKit):
    def __init__(self, asset_id):
        super().__init__("Burbler", asset_id, hp=sum(RACK_HP.values()))
        for key in ("module_kind", "scale"):
            del self.root[key]
        self.root["kind"] = "mine"
        self.root["mine"] = "mudpit_burbler"
        self.root["produces"] = "clock"
        self.rng = random.Random(31)
        noise.seed_set(7)

        m = self.material
        # Crude
        self.mud = m("MudDry", linear("#302116"), roughness=0.95)
        self.wet = m("MudWet", linear("#1f160f"), roughness=0.16)
        self.flow = m("MudFlow", linear("#2a1d13"), roughness=0.5)
        self.crust = m("Crust", linear("#4a3a2a"), roughness=1.0)
        self.sinter = m("Sinter", linear("#6a5d4d"), roughness=0.95)
        self.steam = self.translucent("Steam", linear("#9a9086"), 0.0, 0.025)
        self.sand = m("Sand", linear("#8c7656"), roughness=1.0)
        self.iron = m("Iron", linear("#5e3a22"), metallic=0.5, roughness=0.85)
        self.wood = m("Wood", linear("#5a4a3a"), roughness=0.9)
        self.twine = m("Twine", linear("#6b5a44"), roughness=1.0)
        # High-tech
        self.titanium = m("Titanium", linear("#b8bcc2"), metallic=1.0, roughness=0.32)
        self.graphene = m("Graphene", linear("#1a1c1f"), metallic=0.7, roughness=0.12)
        # Luminescence: a bright opaque core inside faint translucent shells.
        amber, flash, mote = linear("#ffb350"), linear("#ffce89"), linear("#ffe2b0")
        self.glow = m("Glow", amber, emission=amber, strength=8.0)
        self.halo = self.translucent("GlowHalo", amber, 0.9, 0.1)
        self.wisp_core = m("WispCore", flash, emission=flash, strength=10.0)
        self.wisp_bell = self.translucent("WispBell", amber, 1.6, 0.28)
        self.mote = m("Mote", mote, emission=mote, strength=12.0)
        self.mote_halo = self.translucent("MoteHalo", mote, 0.8, 0.08)
        # The outlet's connection halo: authored lit; the game holds it at 0 until
        # something is connected, then lets it breathe at the BPM.
        self.outlet_halo = self.translucent("OutletHalo", amber, 1.6, 0.2)

    def translucent(self, name, color, strength, alpha):
        mat = self.material(name, color, roughness=0.3, emission=color, strength=strength)
        mat.node_tree.nodes["Principled BSDF"].inputs["Alpha"].default_value = alpha
        mat.surface_render_method = "BLENDED"
        return mat

    # ------------------------------------------------------------ helpers

    def mesh_obj(self, name, fill, mat, location=(0, 0, 0), parent=None, smooth=True):
        obj = self.link(bpy.data.objects.new(f"Burbler_{name}", self.mesh_from(name, fill)), mat, parent)
        obj.location = location
        if smooth:
            for face in obj.data.polygons:
                face.use_smooth = True
        return obj

    def node(self, name, location, parent=None, **extras):
        empty = bpy.data.objects.new(name, None)
        empty.empty_display_type = "PLAIN_AXES"
        empty.empty_display_size = 0.1
        self.link(empty, parent=parent)
        empty.location = location
        for key, value in extras.items():
            empty[key] = value
        return empty

    @staticmethod
    def lathe(bm, profile, steps=48):
        """Spin an (r, z) profile around Z."""
        verts = [bm.verts.new((r, 0.0, z)) for r, z in profile]
        edges = [bm.edges.new((a, b)) for a, b in zip(verts, verts[1:])]
        bmesh.ops.spin(bm, geom=verts + edges, cent=(0, 0, 0), axis=(0, 0, 1),
                       angle=2 * math.pi, steps=steps)
        bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)

    def ribbon(self, name, points, width, mat, up=(0, 0, 1), parent=None, taper=False):
        """A flat strip along a polyline; `up` is the strip's normal direction."""
        up = Vector(up)

        def fill(bm):
            rows = []
            for i, p in enumerate(points):
                p = Vector(p)
                a = Vector(points[max(i - 1, 0)])
                b = Vector(points[min(i + 1, len(points) - 1)])
                side = up.cross(b - a).normalized()
                w = width * (1 - i / len(points) * 0.85 if taper else 1) / 2
                rows.append((bm.verts.new(p - side * w), bm.verts.new(p + side * w)))
            for (a0, a1), (b0, b1) in zip(rows, rows[1:]):
                bm.faces.new((a0, a1, b1, b0))
        return self.mesh_obj(name, fill, mat, parent=parent)

    def sphere(self, name, radius, mat, location=(0, 0, 0), parent=None, segments=12, rings=6, shape=None):
        def fill(bm):
            bmesh.ops.create_uvsphere(bm, u_segments=segments, v_segments=rings, radius=radius)
            if shape:
                shape(bm)
        return self.mesh_obj(name, fill, mat, location, parent)

    def glow_point(self, name, location, core_r, halo_r, core_mat, halo_mat, parent=None, **extras):
        """A luminous point: a node with an opaque core inside two nested translucent
        shells. Light passes through more shell near the centre, so the glow falls off
        instead of ending at an edge."""
        node = self.node(name, location, parent, **extras)
        self.sphere(f"{name}_Core", core_r, core_mat, parent=node, segments=8, rings=4)
        self.sphere(f"{name}_Halo", halo_r, halo_mat, parent=node, segments=14, rings=7)
        self.sphere(f"{name}_HaloOuter", halo_r * 1.9, halo_mat, parent=node, segments=16, rings=8)
        return node

    # ------------------------------------------------------------ the pit

    def pit(self):
        def rim(bm):
            self.lathe(bm, [(1.05, POOL_Z - 0.02), (1.22, 0.24), (1.45, 0.30), (1.72, 0.20),
                            (1.95, 0.07), (2.15, 0.0)])
            for v in bm.verts:
                angle = math.atan2(v.co.y, v.co.x)
                n = noise.noise(Vector((math.cos(angle) * 1.7, math.sin(angle) * 1.7, 0.3)))
                v.co.x *= 1 + 0.07 * n
                v.co.y *= 1 + 0.07 * n
                if v.co.z > 0.01:
                    v.co.z *= 1 + 0.45 * noise.noise(v.co * 2.3)
        self.mesh_obj("Rim", rim, self.mud)

        def pool(bm):
            self.lathe(bm, [(0.0, POOL_Z), (0.3, POOL_Z), (0.6, POOL_Z), (0.9, POOL_Z), (POOL_RADIUS, POOL_Z)])
            for v in bm.verts:
                v.co.z += 0.018 * noise.noise(v.co * 2.0 + Vector((5, 5, 0)))
        self.mesh_obj("Pool", pool, self.wet)

        # Dried crust slabs broken off around the rim: the silhouette's broken edge.
        for i in range(11):
            angle = self.rng.uniform(0, 2 * math.pi)
            r = self.rng.uniform(1.35, 2.0)
            size = (self.rng.uniform(0.18, 0.38), self.rng.uniform(0.12, 0.26), 0.035)
            slab = self.box(f"Crust{i}", size, (r * math.cos(angle), r * math.sin(angle), 0.17), self.crust,
                            bevel=0.01)
            slab.rotation_euler = (self.rng.uniform(-0.35, 0.35), self.rng.uniform(-0.35, 0.35), angle)

        self.veins()

    def veins(self, scale=1.0, pool_z=POOL_Z):
        """Amber veins under the wet surface: plankton light, not a lit disc."""
        for i in range(5):
            angle = self.rng.uniform(0, 2 * math.pi)
            r0 = self.rng.uniform(0.15, 0.45) * scale
            sweep = self.rng.uniform(1.2, 1.8)
            pts = []
            for k in range(30):
                t = k / 29
                a = angle + t * sweep
                r = r0 + t * 0.55 * scale
                pts.append((r * math.cos(a), r * math.sin(a), pool_z + 0.03))
            self.ribbon(f"Vein{i}", pts, 0.012, self.glow, taper=True)
            for k, width in enumerate((0.05, 0.11)):
                self.ribbon(f"VeinHalo{i}_{k}", [(x, y, z + 0.003 * (k + 1)) for x, y, z in pts], width,
                            self.halo, taper=True)

    def bubbles(self, scale=1.0, pool_z=POOL_Z, ripples=False):
        # (x, y, radius, division): the big one bursts on the beat, the rest subdivide it.
        for i, (x, y, r, division) in enumerate([(0.25, 0.20, 0.24, 1), (-0.45, 0.35, 0.12, 2),
                                                  (0.05, -0.55, 0.10, 2), (-0.35, -0.30, 0.08, 4),
                                                  (0.62, -0.18, 0.07, 4)]):
            r *= scale ** 0.5
            node = self.node(f"Bubble_{i}", (x * scale, y * scale, pool_z), division=division)
            if ripples:
                # A raised wet ring that catches the light; the game spreads it from the burst.
                def ring(bm, r=r * 1.5):
                    tube = 0.012
                    self.lathe(bm, [(r + tube * math.cos(a), tube * math.sin(a))
                                    for a in (k / 8 * 2 * math.pi for k in range(9))], steps=32)
                self.mesh_obj(f"Bubble_{i}_Ripple", ring, self.wet, (0, 0, 0.03), node)

            def dome(bm, r=r):
                bmesh.ops.delete(bm, geom=[v for v in bm.verts if v.co.z < -1e-4], context="VERTS")
                for v in bm.verts:
                    v.co.z *= 0.8
            self.sphere(f"Bubble_{i}_Dome", r, self.wet, parent=node, segments=16, rings=8, shape=dome)
            # Lit from inside the mud: a soft glow pooled around the bubble's foot.
            for k, scale in enumerate((1.25, 1.6)):
                self.mesh_obj(f"Bubble_{i}_Glow{k}", lambda bm, r=r * scale: bmesh.ops.create_circle(
                    bm, cap_ends=True, segments=32, radius=r), self.halo, (0, 0, 0.012 + 0.003 * k), node)

    def wisps(self, scale=1.0):
        for i, (x, y, z, r) in enumerate([(0.30, 0.25, 0.75, 0.15), (-0.40, 0.45, 1.30, 0.11),
                                           (0.10, -0.10, 1.85, 0.09)]):
            node = self.node(f"Wisp_{i}", (x * scale, y * scale, z), rise_m=2.2, life_s=1.8)

            def bell(bm, r=r):
                bmesh.ops.delete(bm, geom=[v for v in bm.verts if v.co.z < -1e-4], context="VERTS")
                for v in bm.verts:
                    angle = math.atan2(v.co.y, v.co.x)
                    if abs(v.co.z) < 1e-4:  # scalloped rim
                        v.co.x *= 1 + 0.14 * math.sin(8 * angle)
                        v.co.y *= 1 + 0.14 * math.sin(8 * angle)
                        v.co.z -= 0.12 * r * (1 + math.sin(8 * angle))
                    v.co.z *= 0.75
            self.sphere(f"Wisp_{i}_Bell", r, self.wisp_bell, parent=node, segments=16, rings=8, shape=bell)
            self.sphere(f"Wisp_{i}_Core", r * 0.28, self.wisp_core, (0, 0, r * 0.25), node, 8, 4)
            for k in range(5):  # trailing tendrils, waving
                a = k / 5 * 2 * math.pi
                pts = [(r * 0.5 * math.cos(a) + 0.03 * math.sin(j * 1.3 + k), r * 0.5 * math.sin(a),
                        -0.02 - j * r * 0.45) for j in range(8)]
                self.ribbon(f"Wisp_{i}_Tendril{k}", pts, r * 0.16, self.wisp_bell, up=(0, 1, 0),
                            parent=node, taper=True)

    def motes(self, scale=1.0):
        for i in range(8):
            angle = self.rng.uniform(0, 2 * math.pi)
            r = self.rng.uniform(0.4, 2.4) * scale
            self.glow_point(f"Mote_{i}", (r * math.cos(angle), r * math.sin(angle), self.rng.uniform(0.45, 2.3)),
                            0.014, 0.055, self.mote, self.mote_halo)

    # ------------------------------------------------------------ the hot spring

    @staticmethod
    def notch(v, depth):
        """Lower geometry near the outlet, so the rim has a gap for the stream."""
        d = (math.atan2(v.co.y, v.co.x) - OUTLET_ANGLE + math.pi) % (2 * math.pi) - math.pi
        v.co.z *= 1 - depth * math.exp(-(d / 0.2) ** 2)

    def spring(self):
        """A mud pot: a wide, low pool of wet mud in a pale mineral apron that steps
        down in small terraces, spattered with mud thrown out by the bubbles."""
        z = SPRING_POOL_Z
        R = POOL_RADIUS * SPRING_SCALE

        def inner(bm):
            self.lathe(bm, [(R - 0.05, z - 0.02), (R + 0.12, 0.14), (R + 0.35, 0.2), (R + 0.55, 0.15)])
            for v in bm.verts:
                angle = math.atan2(v.co.y, v.co.x)
                n = noise.noise(Vector((math.cos(angle) * 1.4, math.sin(angle) * 1.4, 0.7)))
                v.co.x *= 1 + 0.06 * n
                v.co.y *= 1 + 0.06 * n
                if v.co.z > 0.02:
                    v.co.z *= 1 + 0.35 * noise.noise(v.co * 2.6)
                self.notch(v, 0.8)
        self.mesh_obj("Rim", inner, self.mud)

        def apron(bm):
            self.lathe(bm, [(R + 0.5, 0.15), (R + 0.75, 0.12), (R + 0.8, 0.08), (R + 1.1, 0.07),
                            (R + 1.15, 0.03), (R + 1.55, 0.0)], steps=72)
            for v in bm.verts:
                angle = math.atan2(v.co.y, v.co.x)
                n = noise.noise(Vector((math.cos(angle) * 2.1, math.sin(angle) * 2.1, 1.9)))
                v.co.x *= 1 + 0.09 * n
                v.co.y *= 1 + 0.09 * n
                if v.co.z > 0.01:  # uneven terraces, not a turned plate
                    v.co.z *= 1 + 0.6 * noise.noise(v.co * 3.1)
                self.notch(v, 0.85)
        self.mesh_obj("Apron", apron, self.sinter)

        def pool(bm):
            self.lathe(bm, [(R * t, z) for t in (0.0, 0.2, 0.4, 0.6, 0.8, 1.0)])
            for v in bm.verts:  # slow swells where the mud is about to burble
                v.co.z += 0.03 * noise.noise(v.co * 1.4 + Vector((5, 5, 0)))
        self.mesh_obj("Pool", pool, self.wet)

        self.outlet(R, z)

        # Mud thrown out onto the rim by earlier bursts.
        for i in range(14):
            angle = self.rng.uniform(0, 2 * math.pi)
            r = self.rng.uniform(R + 0.05, R + 0.6)
            size = self.rng.uniform(0.04, 0.11)
            blob = self.sphere(f"Spatter{i}", size, self.wet, (r * math.cos(angle), r * math.sin(angle), 0.15),
                               segments=10, rings=5)
            blob.scale = (self.rng.uniform(1.0, 1.8), 1.0, 0.3)
            blob.rotation_euler.z = angle

    def outlet(self, R, z):
        """Where the burbler's mud stream leaves the pit. The stream itself is laid by
        the player and pulses at the BPM; this is its source: a gap in the rim, a
        tongue of wet mud spilling through it, and the `Outlet_0` node (+X along the
        flow) that the game starts the stream from."""
        out = Vector((math.cos(OUTLET_ANGLE), math.sin(OUTLET_ANGLE), 0))
        side = Vector((-out.y, out.x, 0))
        start_r, end_r = R - 0.3, R + 1.7

        # Raw mud, not a channel: a lumpy tongue that spreads into a lobe where it stops.
        def tongue(bm):
            rows = []
            for k in range(18):
                t = k / 17
                centre = out * (start_r + t * (end_r - start_r)) + side * 0.08 * math.sin(t * 4.5)
                half = 0.28 + 0.25 * t ** 2 + 0.2 * math.sin(math.pi * min(t * 1.15, 1.0)) * t
                row = []
                for j in range(7):
                    u = j / 6 * 2 - 1
                    p = centre + side * u * half * (1 + 0.15 * noise.noise(centre * 3 + Vector((u, 0, 0))))
                    h = z + 0.02 - 0.035 * t + 0.035 * (1 - u * u) + 0.02 * noise.noise(p * 4.0)
                    row.append(bm.verts.new((p.x, p.y, max(h, 0.005))))
                rows.append(row)
            for a, b in zip(rows, rows[1:]):
                for j in range(6):
                    bm.faces.new((a[j], a[j + 1], b[j + 1], b[j]))
        self.mesh_obj("Outlet_Spill", tongue, self.flow)

        node = self.node("Outlet_0", tuple(out * end_r), stream="mud", pulse="bpm")
        node.rotation_euler.z = OUTLET_ANGLE
        halo = self.node("Outlet_0_Halo", (0, 0, 0.06), parent=node, connected_only=True)
        for k, (radius, tube, mat) in enumerate([(0.42, 0.012, self.glow), (0.42, 0.05, self.outlet_halo),
                                                 (0.42, 0.11, self.outlet_halo)]):
            def ring(bm, radius=radius, tube=tube):
                self.lathe(bm, [(radius + tube * math.cos(a), tube * math.sin(a))
                                for a in (i / 8 * 2 * math.pi for i in range(9))], steps=40)
            self.mesh_obj(f"Outlet_0_Ring{k}", ring, mat, parent=halo)

    def steam_plumes(self, scale=1.0):
        """Slow, pale steam: not luminous, it catches the light around it."""
        for i, (x, y, z) in enumerate([(0.3, 0.35, 0.5), (-0.55, 0.45, 0.9)]):
            node = self.node(f"Steam_{i}", (x * scale, y * scale, z), rise_m=3.0, life_s=4.5)
            for k in range(4):  # faint, overlapping puffs widening as they rise
                puff = self.sphere(f"Steam_{i}_Puff{k}", 0.22 + 0.06 * k, self.steam,
                                   (self.rng.uniform(-0.15, 0.15) * (1 + k * 0.4),
                                    self.rng.uniform(-0.15, 0.15), k * 0.2), node, 12, 6)
                puff.scale = (1.0, 1.0, 0.55)

    # ------------------------------------------------------------ the machine

    def probe(self):
        """A titanium probe in the mud, held by a wooden stake with a rusted clamp."""
        top, tip = Vector((-1.55, -1.05, 0.95)), Vector((-0.65, -0.45, POOL_Z - 0.02))
        rod = self.cylinder("ProbeRod", 0.025, (tip - top).length, (top + tip) / 2, self.titanium,
                            segments=12, facing_out=False)
        rod.rotation_euler = (tip - top).to_track_quat("Z", "Y").to_euler()
        head = self.cylinder("ProbeHead", 0.09, 0.02, tip + Vector((0, 0, 0.02)), self.graphene,
                             segments=24, facing_out=False)
        head.rotation_euler = rod.rotation_euler
        stake = self.box("Stake", (0.09, 0.09, 1.15), (-1.66, -1.12, 0.5), self.wood, bevel=0.01)
        stake.rotation_euler = (0.06, -0.08, 0.3)
        clamp = self.cylinder("Clamp", 0.085, 0.07, (-1.6, -1.08, 0.88), self.iron, segments=12, facing_out=False)
        clamp.rotation_euler = (0.06, -0.08, 0.3)
        for i, (dx, dy) in enumerate([(0.18, -0.06), (-0.12, 0.17)]):
            bag = self.box(f"StakeBag{i}", (0.34, 0.2, 0.13), (-1.66 + dx, -1.12 + dy, 0.065), self.sand, bevel=0.05)
            bag.rotation_euler.z = 0.4 + i

    def conduit(self):
        """Graphene conduit along the ground, stake to rack; its beads carry the beat."""
        rack = Vector(RACK_AT) + Vector((-RACK_WIDTH / 2 * math.cos(RACK_YAW), -RACK_WIDTH / 2 * math.sin(RACK_YAW), 0))
        control = [Vector((-1.6, -1.1, 0.15)), Vector((-1.2, -2.0, 0.04)), Vector((0.3, -2.3, 0.04)),
                   Vector((1.4, -1.4, 0.04)), rack + Vector((0, 0.05, 0.3))]
        curve = bpy.data.curves.new("Burbler_ConduitCurve", "CURVE")
        curve.dimensions = "3D"
        curve.bevel_depth = 0.028
        curve.bevel_resolution = 2
        spline = curve.splines.new("BEZIER")
        spline.bezier_points.add(len(control) - 1)
        for point, co in zip(spline.bezier_points, control):
            point.co = co
            point.handle_left_type = point.handle_right_type = "AUTO"
        temp = bpy.data.objects.new("Burbler_ConduitTemp", curve)
        self.scene.collection.objects.link(temp)
        depsgraph = bpy.context.evaluated_depsgraph_get()
        mesh = bpy.data.meshes.new_from_object(temp.evaluated_get(depsgraph))
        samples = [temp.matrix_world @ p for p in self.sample_curve(spline, 9)]
        bpy.data.objects.remove(temp)
        bpy.data.curves.remove(curve)
        mesh.name = "Burbler_Conduit"
        self.link(bpy.data.objects.new("Burbler_Conduit", mesh), self.graphene)
        for i, p in enumerate(samples[1:-1]):
            self.glow_point(f"Conduit_Bead_{i}", p + Vector((0, 0, 0.03)), 0.022, 0.06, self.glow, self.halo,
                            order=i)

    @staticmethod
    def sample_curve(spline, n):
        from mathutils.geometry import interpolate_bezier
        pts = []
        bp = spline.bezier_points
        for a, b in zip(bp, bp[1:]):
            pts += interpolate_bezier(a.co, a.handle_right, b.handle_left, b.co, 24)[:-1]
        pts.append(bp[-1].co.copy())
        return [pts[round(i * (len(pts) - 1) / (n - 1))] for i in range(n)]

    def rack(self):
        rack = self.node("Rack", RACK_AT)
        rack.rotation_euler.z = RACK_YAW
        half = RACK_WIDTH / 2
        top = RACK_BASE + HEIGHT

        def part(obj):
            obj.parent = rack
            return obj
        # Crude: rough-hewn posts in sandbags, lashed with rusted iron straps.
        for side in (-1, 1):
            x = side * (half + 0.1)
            post = part(self.box(f"Post{side:+d}", (0.13, 0.13, 2.15), (x, 0.05, 1.07), self.wood, bevel=0.012))
            post.rotation_euler.y = side * 0.03
            for k in range(3):
                bag = part(self.box(f"PostBag{side:+d}{k}", (0.36, 0.22, 0.14),
                                    (x + side * 0.12 * (k - 1), 0.05 + 0.2 * (k % 2 * 2 - 1), 0.07 + 0.11 * (k == 1)),
                                    self.sand, bevel=0.055))
                bag.rotation_euler.z = 0.3 * (k - 1)
            for z in (RACK_BASE + 0.03, top - 0.03):
                part(self.box(f"Strap{side:+d}{z:.2f}", (0.17, 0.17, 0.045), (x, 0.05, z), self.iron, bevel=0.004))
        # High-tech: titanium rails and a graphene backplate.
        for z in (RACK_BASE + 0.03, top - 0.03):
            part(self.box(f"Rail{z:.2f}", (RACK_WIDTH + 0.24, 0.06, 0.05), (0, 0.0, z), self.titanium, bevel=0.004))
            part(self.box(f"RailInset{z:.2f}", (RACK_WIDTH, 0.004, 0.018), (0, -0.031, z), self.graphene))
        part(self.box("Backplate", (RACK_WIDTH + 0.04, 0.02, HEIGHT), (0, 0.18, RACK_BASE + HEIGHT / 2),
                      self.graphene))

        # Corrugated rusted-iron roof, pitched back.
        def roof(bm):
            bmesh.ops.create_grid(bm, x_segments=28, y_segments=2, size=0.5)
            for v in bm.verts:
                v.co.x *= 1.75
                v.co.y *= 0.8
                v.co.z = 0.012 * math.sin(v.co.x * 40)
        r = part(self.mesh_obj("Roof", roof, self.iron, (0, 0.05, 2.17), smooth=False))
        r.rotation_euler.x = 0.22  # pitched toward the approach so it reads from the front

        # Fairy lanterns: teardrops of light hung on twine from the roof's front edge.
        for i, (x, drop) in enumerate([(-0.55, 0.32), (0.05, 0.22), (0.6, 0.38)]):
            z = 2.1 - drop
            part(self.cylinder(f"LanternThread{i}", 0.004, drop, (x, -0.33, z + drop / 2), self.twine,
                               segments=6, facing_out=False))
            lantern = part(self.node(f"Lantern_{i}", (x, -0.33, z - 0.07)))

            def tear(bm):
                for v in bm.verts:
                    if v.co.z > 0:
                        v.co.z *= 1.8
                        k = 1 - 0.65 * (v.co.z / 0.14) ** 1.3
                        v.co.x *= k
                        v.co.y *= k
            self.sphere(f"Lantern_{i}_Shell", 0.08, self.wisp_bell, parent=lantern, segments=14, rings=8,
                        shape=tear)
            self.sphere(f"Lantern_{i}_Core", 0.025, self.wisp_core, parent=lantern, segments=8, rings=4)

        # Where module models go: origin at each module's base centre, faceplate to -Y.
        x = -half
        for place, hp in RACK_HP.items():
            width = hp * HP
            part(self.node(f"Slot_{place}", (x + width / 2, 0.0, RACK_BASE), hp=hp, accepts=place,
                           fixed=place == "clock"))
            x += width
        part(self.node("Interact", (0.0, -0.75, 1.0), action="configure", prompt="Configure"))
        return rack

    # ------------------------------------------------------------ build

    def profile(self):
        nodes = sorted(o.name for o in self.collection.objects if o.type == "EMPTY")
        return {
            "id": self.asset_id,
            "model": f"models/{self.asset_id}.glb#Scene0",
            "kind": "mine",
            "mine": "mudpit_burbler",
            "produces": "clock",
            "size_metres": self.size,
            "params": {"bpm": {"min": 40.0, "max": 180.0, "default": 90.0,
                               "note": "set per instance from the world seed"}},
            "configure": {"screen": "eurorack_hud",
                          "places": [{"place": p, "hp": hp, "fixed": p == "clock"} for p, hp in RACK_HP.items()]},
            "lights": {
                "Burbler_Glow": {"rest": 0.15, "peak": 1.0, "attack_s": 0.04, "decay_s": 0.45},
                "Burbler_WispBell": {"rest": 0.0, "peak": 1.0, "attack_s": 0.1, "decay_s": 1.8},
                "Burbler_WispCore": {"rest": 0.0, "peak": 1.0, "attack_s": 0.05, "decay_s": 1.6},
                "Burbler_Mote": {"rest": 0.35, "peak": 1.0, "attack_s": 0.2, "decay_s": 0.8},
                "Burbler_OutletHalo": {"rest": 0.0, "peak": 1.0, "attack_s": 0.05, "decay_s": 0.5,
                                       "note": "0 until connected; then breathes at the BPM"},
            },
            "nodes": nodes,
        }

    def build_spring(self):
        self.size = {"width": 6.3, "depth": 6.3, "height": 0.3}
        self.spring()
        self.veins(SPRING_SCALE, SPRING_POOL_Z)
        self.bubbles(SPRING_SCALE, SPRING_POOL_Z, ripples=True)
        self.wisps(SPRING_SCALE)
        self.steam_plumes(SPRING_SCALE)
        self.motes(SPRING_SCALE)
        self.node("Interact", (0.0, -POOL_RADIUS * SPRING_SCALE - 0.6, 0.5), action="configure",
                  prompt="Configure")
        return self.finish(self.profile(), aim=(0.7, -0.8, 0.5), camera_location=(1.0, -10.2, 4.6),
                           before_render=lambda: self.stage_preview(), profile_dir="machines")

    def build_rig(self):
        self.size = {"width": 5.2, "depth": 4.4, "height": 2.4}
        self.pit()
        self.bubbles()
        self.wisps()
        self.motes()
        self.probe()
        self.conduit()
        self.rack()
        return self.finish(self.profile(), aim=(0.55, -0.2, 0.95), camera_location=(0.9, -9.2, 3.5),
                           before_render=lambda: self.stage_preview(
                               {"Slot_clock": "clock_lamp", "Slot_vco": "square_vco"}),
                           profile_dir="machines")

    def stage_preview(self, slots=None):
        """Not exported: ground, light, bloom, and stand-in modules in any slots."""
        ground = self.box("PreviewGround", (40, 40, 0.02), (0, 0, -0.011), self.mud)
        ground.hide_select = True
        for slot, model in (slots or {}).items():
            before = set(bpy.data.objects)
            with bpy.context.temp_override(scene=self.scene, view_layer=self.scene.view_layers[0]):
                bpy.ops.import_scene.gltf(filepath=str(PROJECT / f"assets/models/{model}.glb"))
            target = bpy.data.objects[slot].matrix_world
            for obj in set(bpy.data.objects) - before:
                if obj.name not in self.scene.objects:
                    self.scene.collection.objects.link(obj)
                if obj.parent is None:
                    obj.matrix_world = target @ obj.matrix_world
                if obj.name.startswith("Display_d"):  # read 090, not 888
                    digit, seg = int(obj.name[9]), obj.name[11]
                    obj.hide_render = seg not in ("abcdef", "abcdfg", "abcdef")[digit]
        key = bpy.data.objects["Burbler_PreviewKey"]
        key.data.energy = 900
        key.data.size = 3.0
        key.location = (-3.0, -4.5, 4.5)
        key.rotation_euler = (Vector((0.5, 0, 0.6)) - key.location).to_track_quat("-Z", "Y").to_euler()
        self.scene.render.resolution_x, self.scene.render.resolution_y = 1280, 800
        # Bloom, as the game's camera will have; without it halos read as flat shapes.
        tree = bpy.data.node_groups.new("Burbler_PreviewBloom", "CompositorNodeTree")
        tree.interface.new_socket("Image", in_out="OUTPUT", socket_type="NodeSocketColor")
        layers, glare = tree.nodes.new("CompositorNodeRLayers"), tree.nodes.new("CompositorNodeGlare")
        out = tree.nodes.new("NodeGroupOutput")
        glare.inputs["Type"].default_value = "Bloom"
        glare.inputs["Threshold"].default_value = 0.8
        glare.inputs["Strength"].default_value = 0.7
        glare.inputs["Size"].default_value = 0.6
        tree.links.new(layers.outputs["Image"], glare.inputs["Image"])
        tree.links.new(glare.outputs["Image"], out.inputs[0])
        self.scene.compositing_node_group = tree


if __name__ == "__main__":
    print(run({"pit": lambda: Burbler("mudpit_burbler").build_spring(),
               "rig": lambda: Burbler("mudpit_burbler_rig").build_rig()}, __file__))
