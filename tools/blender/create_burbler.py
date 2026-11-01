"""Mudpit burbler: a gurgling hot-spring mud pot that is a clock mine (add-mudpit-burbler).

Two assets:

- `pit` -> mudpit_burbler: the burbler itself. An organically round pool of mud,
  gurgling at its BPM (every burble is a beat), with a stream about 1 m wide leaving
  it between banked sides. The player configures it through a
  Eurorack HUD (game-side UI), so nothing is built in the world.
- `rig` -> mudpit_burbler_rig: a concept kept for later. The same pit with a rack
  beside it, built from crude parts (sand, rusted iron, weathered wood) lashed to
  high-tech ones (titanium, graphene), a probe in the mud and a conduit to the rack.

The light is jellyfish and fairy light, not LEDs (docs/art/style-guide.md): wisps
rise from the bursts like small jellyfish, motes drift, and amber veins glow under
the mud. The models ship at rest; the game animates the named nodes from the
burbler's clock ticks.

  Bubble_<n>           extras: division (bursts on every beat / division)
  Bubble_<n>_Burble    (pit) glTF animation: one beat, burst at 0 s; play at BPM/60/division
  Bubble_<n>_Ripple    (pit) a ring the game spreads outward from each burst
  Wisp_<n>             extras: rise_m, life_s (rises from a burst and fades)
  Steam_<n>            (pit) extras: rise_m, life_s (drifts up, slow)
  Outlet_<n>           (pit) extras: stream, pulse. Where the stream leaves the mouth; +X along the flow
  Outlet_<n>_Halo      (pit) an upright arch over the mouth: the snap point while building a stream
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
from module_kit import HEIGHT, HP, PROJECT, ModuleKit, model_file, run  # noqa: E402

POOL_RADIUS = 1.15
POOL_Z = 0.12
# The burbler is an organically round pool of mud that bubbles at its BPM, with a
# stream about 1 m wide leaving it between banked sides. Pool and channel are one
# landform: the banks follow a smooth union of the two. Banks are built up from the
# ground, not cut into it, so the asset sits on any terrain.
POOL_CENTRE = (0.0, 1.15)
POOL_R = 1.35             # pool radius before its organic wobble
CHANNEL_HALF_WIDTH = 0.5  # the stream: 1 m of mud between the banks
STREAM_MOUTH_Y = -2.3     # where the banks run out and the player's stream carries on
BANK_WIDTH = 0.38         # inner slope, mud edge to crest
BANK_HEIGHT = 0.36        # crest height above the ground
BANK_SKIRT = 0.7          # outer slope, crest down to the ground
STREAM_MUD_Z = 0.07       # mud surface height
# (x, y, radius, division) in the pool: the big one bursts on the beat, the rest subdivide it.
STREAM_BUBBLES = [(0.1, 1.3, 0.26, 1), (-0.6, 1.65, 0.13, 2), (0.6, 0.8, 0.11, 2),
                  (-0.45, 0.55, 0.09, 4), (0.5, 1.9, 0.08, 4)]
STREAM_WISPS = [(0.1, 1.25, 0.75, 0.15), (-0.45, 1.6, 1.3, 0.11), (0.35, 0.9, 1.85, 0.09)]
BEAT_FRAMES = 24          # one beat at 24 fps: each bubble's clip lasts 1 s at speed 1

# E opens the configuration screen within this reach (add-mudpit-burbler).
INTERACT = {
    "node": "Interact",
    "key": "E",
    "range_m": 3.0,
    "measured": "ground distance from the player to the Interact node, the same reach as E pickups",
    "prompt": "E | Configure burbler",
    "priority": "when a pickup is also in reach, the nearer target wins"
}
RACK_HP = {"clock": 8, "vco": 8, "volume": 4}
RACK_KIND = {"clock": "clock", "vco": "vco", "volume": "vca"}   # slot label per place
RACK_WIDTH = sum(RACK_HP.values()) * HP
RACK_BASE = 0.30  # module base height in the rack
RACK_AT = (2.35, -0.25, 0.0)
RACK_YAW = math.radians(-20)  # turned a little toward the approach, pit on its left


def linear(hex_srgb):
    """Style-guide hex (sRGB) to the linear colour Blender materials take."""
    c = [int(hex_srgb[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    return tuple(v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4 for v in c)


class Burbler(ModuleKit):
    folder = "machines"

    def __init__(self, asset_id, kind="Burbler"):
        super().__init__(kind, asset_id, hp=sum(RACK_HP.values()))
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
        obj = self.link(bpy.data.objects.new(f"{self.kind}_{name}", self.mesh_from(name, fill)), mat, parent)
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

    def bubbles(self, pool_z=POOL_Z, ripples=False, layout=None):
        # (x, y, radius, division): the big one bursts on the beat, the rest subdivide it.
        for i, (x, y, r, division) in enumerate(layout or [(0.25, 0.20, 0.24, 1), (-0.45, 0.35, 0.12, 2),
                                                           (0.05, -0.55, 0.10, 2), (-0.35, -0.30, 0.08, 4),
                                                           (0.62, -0.18, 0.07, 4)]):
            node = self.node(f"Bubble_{i}", (x, y, pool_z), division=division)
            parts = {}
            if ripples:
                # A raised wet ring that catches the light; the game spreads it from the burst.
                def ring(bm, r=r * 1.5):
                    tube = 0.012
                    self.lathe(bm, [(r + tube * math.cos(a), tube * math.sin(a))
                                    for a in (k / 8 * 2 * math.pi for k in range(9))], steps=32)
                parts["ripple"] = self.mesh_obj(f"Bubble_{i}_Ripple", ring, self.wet, (0, 0, 0.03), node)

            def dome(bm, r=r):
                bmesh.ops.delete(bm, geom=[v for v in bm.verts if v.co.z < -1e-4], context="VERTS")
                for v in bm.verts:
                    v.co.z *= 0.8
            parts["dome"] = self.sphere(f"Bubble_{i}_Dome", r, self.wet, parent=node, segments=16, rings=8,
                                        shape=dome)
            # Lit from inside the mud: a soft glow pooled around the bubble's foot.
            for k, scale in enumerate((1.25, 1.6)):
                parts[f"glow{k}"] = self.mesh_obj(f"Bubble_{i}_Glow{k}", lambda bm, r=r * scale: bmesh.ops.create_circle(
                    bm, cap_ends=True, segments=32, radius=r), self.halo, (0, 0, 0.012 + 0.003 * k), node)
            if ripples:
                self.burble(f"Bubble_{i}_Burble", parts)

    # One beat of a bubble, frame 0 = the burst, on the clock tick. The game plays the
    # clip looping at speed BPM / 60 / division. Scale keys per part.
    BURBLE_KEYS = {
        "dome": [(0, (1.12, 1.12, 0.5)), (1, (0.02, 0.02, 0.02)), (4, (0.1, 0.1, 0.06)),
                 (20, (0.9, 0.9, 0.85)), (BEAT_FRAMES, (1.0, 1.0, 1.0))],
        "ripple": [(0, (0.35, 0.35, 1.0)), (16, (1.7, 1.7, 0.25)), (17, (0.0, 0.0, 0.0)),
                   (BEAT_FRAMES, (0.0, 0.0, 0.0))],
        "glow0": [(0, (1.4, 1.4, 1.0)), (6, (0.8, 0.8, 1.0)), (BEAT_FRAMES, (1.0, 1.0, 1.0))],
        "glow1": [(0, (1.5, 1.5, 1.0)), (8, (0.75, 0.75, 1.0)), (BEAT_FRAMES, (1.0, 1.0, 1.0))],
    }

    def burble(self, name, parts):
        """Bake one bubble's beat into a single action shared by its parts, so it exports
        as one glTF animation named `name`."""
        action = bpy.data.actions.new(name)
        for key, obj in parts.items():
            obj.animation_data_create()
            obj.animation_data.action = action
            for frame, scale in self.BURBLE_KEYS[key]:
                obj.scale = scale
                obj.keyframe_insert("scale", frame=frame)

    def wisps(self, layout=None):
        for i, (x, y, z, r) in enumerate(layout or [(0.30, 0.25, 0.75, 0.15), (-0.40, 0.45, 1.30, 0.11),
                                                    (0.10, -0.10, 1.85, 0.09)]):
            node = self.node(f"Wisp_{i}", (x, y, z), rise_m=2.2, life_s=1.8)

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

    def motes(self, scale=(1.0, 1.0), offset=(0.0, 0.0)):
        for i in range(8):
            angle = self.rng.uniform(0, 2 * math.pi)
            r = self.rng.uniform(0.4, 2.4)
            at = (offset[0] + r * math.cos(angle) * scale[0], offset[1] + r * math.sin(angle) * scale[1])
            self.glow_point(f"Mote_{i}", (*at, self.rng.uniform(0.45, 2.3)),
                            0.014, 0.055, self.mote, self.mote_halo)

    # ------------------------------------------------------------ the hot spring

    @staticmethod
    def smooth(t):
        t = min(max(t, 0.0), 1.0)
        return t * t * (3 - 2 * t)

    def pool_radius(self, angle):
        """The pool's organic edge: round, but never a circle."""
        n = noise.noise(Vector((math.cos(angle) * 1.3, math.sin(angle) * 1.3, 2.0)))
        return POOL_R * (1 + 0.1 * n + 0.03 * math.sin(3 * angle + 1.0))

    def channel_half_width(self, y):
        return CHANNEL_HALF_WIDTH * (1 + 0.08 * noise.noise(Vector((y * 0.9, 3.3, 0.0))))  # banks wander

    def bank_distance(self, x, y):
        """Signed distance outward from the mud's edge: negative in the mud, positive
        across the banks. A smooth union of the round pool and the stream channel."""
        cx, cy = POOL_CENTRE
        pool = math.hypot(x - cx, y - cy) - self.pool_radius(math.atan2(y - cy, x - cx))
        channel = abs(x) - self.channel_half_width(y) if y < cy else 99.0
        k = 0.3
        return -k * math.log(math.exp(-pool / k) + math.exp(-channel / k))

    def bank_height(self, x, y):
        s = self.bank_distance(x, y)
        if s <= 0:
            return 0.0
        if s < BANK_WIDTH:
            h = self.smooth(s / BANK_WIDTH)
        else:
            h = 1 - self.smooth((s - BANK_WIDTH) / BANK_SKIRT)
        h *= BANK_HEIGHT * (1 + 0.25 * noise.noise(Vector((x * 2.3, y * 1.7, 0.5))))
        # The banks run out at the mouth, where the player's stream takes over.
        return h * self.smooth((y - STREAM_MOUTH_Y + 0.15) / 0.75)

    def channel(self):
        """The stream bed: banked earthen sides and the mud between them."""
        x0, x1 = -2.6, 2.6
        y0, y1 = STREAM_MOUTH_Y - 0.4, POOL_CENTRE[1] + POOL_R + 1.4
        nx, ny = 38, 48

        def grid(bm, height, keep):
            verts = [[bm.verts.new((x0 + (x1 - x0) * i / nx, y0 + (y1 - y0) * j / ny, 0.0))
                      for i in range(nx + 1)] for j in range(ny + 1)]
            for row in verts:
                for v in row:
                    v.co.z = height(v.co.x, v.co.y)
            for j in range(ny):
                for i in range(nx):
                    quad = (verts[j][i], verts[j][i + 1], verts[j + 1][i + 1], verts[j + 1][i])
                    if keep(quad):
                        bm.faces.new(quad)
            bmesh.ops.delete(bm, geom=[v for v in bm.verts if not v.link_faces], context="VERTS")

        def banks(bm):
            # Keep only the raised bank and its foot under the mud's edge; drop flat ground.
            grid(bm, lambda x, y: self.bank_height(x, y),
                 lambda q: any(v.co.z > 0.004 for v in q)
                 and not all(self.bank_distance(v.co.x, v.co.y) < -0.12 for v in q))
        self.mesh_obj("Banks", banks, self.mud)

        def mud(bm):  # the mud fill, gently swelling, flowing out past the mouth
            grid(bm, lambda x, y: STREAM_MUD_Z + 0.02 * noise.noise(Vector((x * 1.6, y * 1.2, 7.0))),
                 # Runs on under the banks' inner slope, so its grid edge is hidden.
                 lambda q: any(self.bank_distance(v.co.x, v.co.y) < 0.1 for v in q))
        self.mesh_obj("Mud", mud, self.wet)

        # Mud thrown up onto the banks around the pool by the bubbles.
        cx, cy = POOL_CENTRE
        for i in range(14):
            angle = self.rng.uniform(-0.2 * math.pi, 1.2 * math.pi)  # not across the stream's mouth
            r = self.pool_radius(angle) + self.rng.uniform(0.08, 0.4)
            x, y = cx + r * math.cos(angle), cy + r * math.sin(angle)
            blob = self.sphere(f"Spatter{i}", self.rng.uniform(0.04, 0.1), self.wet,
                               (x, y, self.bank_height(x, y) + 0.01), segments=10, rings=5)
            blob.scale = (1.0, self.rng.uniform(1.0, 1.8), 0.3)
            blob.rotation_euler.z = angle

    def stream_veins(self):
        """Amber veins that swirl in the pool and are drawn out down the stream: the
        beat's path out of the pit."""
        cx, cy = POOL_CENTRE
        entrance = Vector((0.0, cy - POOL_R * 0.85))
        for i in range(3):
            angle = self.rng.uniform(0.2 * math.pi, 0.8 * math.pi) + i * 0.6
            start = Vector((cx, cy)) + Vector((math.cos(angle), math.sin(angle))) * self.rng.uniform(0.4, 0.8)
            bend = Vector((cx, cy)) + Vector((math.cos(angle - 1.4), math.sin(angle - 1.4))) * 0.9
            phase = self.rng.uniform(0, 2 * math.pi)
            pts = []
            for k in range(14):  # swirl toward the channel's entrance (quadratic Bezier)
                t = k / 14
                p = start.lerp(bend, t).lerp(bend.lerp(entrance, t), t)
                pts.append((p.x, p.y, STREAM_MUD_Z + 0.035))
            for k in range(20):  # then down the stream, wandering within its banks
                t = k / 19
                y = entrance.y + t * (STREAM_MOUTH_Y - 0.3 - entrance.y)
                w = self.channel_half_width(y) * 0.75
                pts.append((max(-w, min(w, 0.15 * math.sin(phase + t * 7))), y, STREAM_MUD_Z + 0.035))
            self.ribbon(f"Vein{i}", pts, 0.008, self.glow, taper=True)
            self.ribbon(f"VeinHalo{i}", [(x, y, z + 0.003) for x, y, z in pts], 0.06, self.halo, taper=True)

    def arch(self, name, radius, tube, mat, parent, segments=40):
        """An upright half-ring across the stream, in the node's YZ plane."""
        def fill(bm):
            rings = []
            for i in range(segments + 1):
                phi = math.pi * i / segments
                centre = Vector((0, radius * math.cos(phi), radius * math.sin(phi)))
                normal = Vector((0, math.cos(phi), math.sin(phi)))
                rings.append([bm.verts.new(centre + tube * (normal * math.cos(a) + Vector((1, 0, 0)) * math.sin(a)))
                              for a in (k / 8 * 2 * math.pi for k in range(8))])
            for ra, rb in zip(rings, rings[1:]):
                for k in range(8):
                    bm.faces.new((ra[k], ra[(k + 1) % 8], rb[(k + 1) % 8], rb[k]))
        return self.mesh_obj(name, fill, mat, parent=parent)

    def mouth(self):
        """Where the stream leaves the burbler: `Outlet_0` (+X along the flow), with an
        upright halo arch over the mouth. The arch is the snap point: it lights while the
        player is extending a stream from here, as a generator's port does when you drag a
        conveyor from it in Satisfactory, and stays softly lit once connected."""
        node = self.node("Outlet_0", (0.0, STREAM_MOUTH_Y - 0.3, STREAM_MUD_Z), stream="mud", pulse="bpm")
        node.rotation_euler.z = -math.pi / 2
        halo = self.node("Outlet_0_Halo", (0, 0, 0), parent=node, role="port_highlight")
        radius = self.channel_half_width(STREAM_MOUTH_Y) + 0.12
        for k, (tube, mat) in enumerate([(0.008, self.glow), (0.03, self.outlet_halo), (0.07, self.outlet_halo)]):
            self.arch(f"Outlet_0_Arch{k}", radius, tube, mat, halo)

    def steam_plumes(self, layout):
        """Slow, pale steam: not luminous, it catches the light around it."""
        for i, (x, y, z) in enumerate(layout):
            node = self.node(f"Steam_{i}", (x, y, z), rise_m=3.0, life_s=4.5)
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
            "model": f"{self.model_ref}#Scene0",
            "kind": "mine",
            "mine": "mudpit_burbler",
            "produces": "clock",
            "size_metres": self.size,
            "params": {"bpm": {"min": 40.0, "max": 180.0, "default": 90.0,
                               "note": "set per instance from the world seed"}},
            # This machine's rack configurator: its configuration recipe's slots.
            "configure": {"screen": "eurorack_hud", "hud": "models/hud/rack_frame.glb",
                          "title": "MUDPIT BURBLER", "interact": INTERACT,
                          "places": [{"place": p, "kind": RACK_KIND[p], "hp": hp, "fixed": p == "clock",
                                      **({"module": "burbler_clock"} if p == "clock" else {})}
                                     for p, hp in RACK_HP.items()],
                          # One spare cable per patch the voice needs, hanging on the comb.
                          "cables": [{"color": "verdigris", "for": "clock GATE -> volume TRIG"},
                                     {"color": "rust", "for": "VCO OUT -> volume IN"}]},
            "lights": {
                "Burbler_Glow": {"rest": 0.15, "peak": 1.0, "attack_s": 0.04, "decay_s": 0.45},
                "Burbler_WispBell": {"rest": 0.0, "peak": 1.0, "attack_s": 0.1, "decay_s": 1.8},
                "Burbler_WispCore": {"rest": 0.0, "peak": 1.0, "attack_s": 0.05, "decay_s": 1.6},
                "Burbler_Mote": {"rest": 0.35, "peak": 1.0, "attack_s": 0.2, "decay_s": 0.8},
                "Burbler_OutletHalo": {"rest": 0.0, "peak": 1.0, "connected": 0.4, "attack_s": 0.05,
                                       "decay_s": 0.5,
                                       "note": "0 normally; peak while a stream is being extended from "
                                               "here (the snap point); 'connected' once attached, "
                                               "breathing at the BPM"},
            },
            "nodes": nodes,
            "animations": {a.name: {"seconds": BEAT_FRAMES / 24, "loop": True, "burst_at_seconds": 0.0,
                                    "speed": "bpm / 60 / division (the bubble node's extras)"}
                           for a in bpy.data.actions if a.name.endswith("_Burble")},
        }

    def build_stream(self):
        self.size = {"width": 5.0, "depth": 6.6, "height": BANK_HEIGHT}
        self.channel()
        self.stream_veins()
        self.bubbles(STREAM_MUD_Z, ripples=True, layout=STREAM_BUBBLES)
        self.wisps(STREAM_WISPS)
        self.steam_plumes([(0.15, 1.4, 0.5), (-0.4, 1.0, 0.9)])
        self.motes(scale=(0.9, 1.1), offset=(0.0, 0.7))
        self.mouth()
        self.node("Interact", (-1.75, 0.7, 0.6), action="configure", prompt="Configure")
        # Rest pose (and preview): every bubble most of the way through its swell.
        self.scene.render.fps = 24
        self.scene.frame_start, self.scene.frame_end = 0, BEAT_FRAMES
        self.scene.frame_set(20)
        # Looking up the stream from beyond its mouth: the stream runs up the frame.
        return self.finish(self.profile(), aim=(0.0, 0.5, 0.1), camera_location=(1.0, -7.0, 5.0),
                           before_render=lambda: self.stage_preview(resolution=(900, 1200)))

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
                               {"Slot_clock": "clock_lamp", "Slot_vco": "square_vco"}))

    def stage_preview(self, slots=None, resolution=(1280, 800)):
        """Not exported: ground, light, bloom, and stand-in modules in any slots."""
        ground = self.box("PreviewGround", (40, 40, 0.02), (0, 0, -0.011), self.mud)
        ground.hide_select = True
        for slot, model in (slots or {}).items():
            before = set(bpy.data.objects)
            with bpy.context.temp_override(scene=self.scene, view_layer=self.scene.view_layers[0]):
                bpy.ops.import_scene.gltf(filepath=str(model_file(model)))
            target = bpy.data.objects[slot].matrix_world
            for obj in set(bpy.data.objects) - before:
                if obj.name not in self.scene.objects:
                    self.scene.collection.objects.link(obj)
                if obj.parent is None:
                    obj.matrix_world = target @ obj.matrix_world
                if obj.name.startswith("Display_d"):  # read 090, not 888
                    digit, seg = int(obj.name[9]), obj.name[11]
                    obj.hide_render = seg not in ("abcdef", "abcdfg", "abcdef")[digit]
        key = bpy.data.objects[f"{self.kind}_PreviewKey"]
        key.data.energy = 900
        key.data.size = 3.0
        key.location = (-3.0, -4.5, 4.5)
        key.rotation_euler = (Vector((0.5, 0, 0.6)) - key.location).to_track_quat("-Z", "Y").to_euler()
        self.scene.render.resolution_x, self.scene.render.resolution_y = resolution
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
    print(run({"pit": lambda: Burbler("mudpit_burbler").build_stream(),
               "rig": lambda: Burbler("mudpit_burbler_rig").build_rig()}, __file__))
