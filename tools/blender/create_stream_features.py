"""Things that sit on a laid stream: the ripple pool (add-ripple-pool) and the reed water
purifier's check dam (add-reed-water-purifier D6).

Both share the stream segment's frame (`create_stream.py`): flow runs along +X, the origin
is the channel's centre on the ground, the channel is 1 m of water between banked sides.
The game places them at a point along a laid stream, turned to its flow there.

Ripple pool (`streams/ripple_pool/`)
  A small still pool of clear water just beyond the stream's left bank (+Y), where water
  seeps through the bank and settles: still and clear beside the moving mud, so it reads
  from a distance. It is symmetric along the flow, so turning it half a turn puts it on
  the right bank. Built up from the ground (the terrain covers anything below it).
  Ripples          shipped at scale 0 (`start_scale: 0`), extras `shown_while: beat`: the
                   game shows it while the pool keeps its stream's clock (the stream
                   sounds) and plays `Ripple_Beat` on that clock
  Ripple_<n>       pale luminous rings; `Ripple_Beat` (one beat, 24 frames at 24 fps)
                   sends a new ring out from the centre on the beat while last beat's ring
                   reaches the rim, so one ring leaves per beat and none pops
  Ripple_Drop      the beat's drop at the centre
  Interact         E: extras resource = ripple_pool (an oscilloscope)
  No levels of detail: it is small (about 2k triangles) and animated, and Jackdaw's LOD
  runtime flattens a level into bare meshes.

Check dam (`machines/reed_water_purifier/`)
  The purifier as the player sees it in the world: a small beaver dam, or check dam, of
  sticks and reed bundles packed with mud across the stream's bed and up both banks. The
  water pools behind it (upstream, -X) and spills through a notch in the middle.
  Pond             the pooled water behind the dam, rising toward the crest
  Spill, Foam_<n>  the water spilling through the notch and its foam below
  Interact         E: extras action = configure (opens the purifier's rack)
  Outlet           where the cleaned water leaves, extras clarity_lift = 1 (one level)
  reed_water_purifier_LOD0 / _LOD1  the near dam and a coarse far one

Reproduce: blender -b --factory-startup -P tools/blender/create_stream_features.py -- ripple_pool
           blender -b --factory-startup -P tools/blender/create_stream_features.py -- reed_water_purifier
Writes the assets' folders, their .blend sources and previews/<folder>/<id>/.
"""
from pathlib import Path
import math
import random
import sys

import bmesh
import bpy
from mathutils import Vector, noise

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_burbler import (  # noqa: E402
    BANK_HEIGHT, BANK_SKIRT, BANK_WIDTH, CHANNEL_HALF_WIDTH, STREAM_MUD_Z, Burbler, linear,
)
from create_reeds import COLORS as REED_COLORS  # noqa: E402
from create_stream import CLARITY  # noqa: E402

BEAT_FRAMES = 24
BANK_OUTER = CHANNEL_HALF_WIDTH + BANK_WIDTH + BANK_SKIRT   # 1.58 m from the centre line

# ---------------------------------------------------------------- ripple pool
POOL_Y = BANK_OUTER + 0.72      # the pool's centre, just beyond the left bank's foot
POOL_R = 0.72                   # the water's radius before its wobble
RIM_OUT = 0.5                   # the rim's width beyond the water
RIM_HEIGHT = 0.15
POOL_WATER_Z = 0.1
RING_MAX = 0.66                 # a ring's radius as it reaches the rim
# One beat: the new ring leaves the centre on the beat (frame 0) and is halfway out at the
# next beat, where last beat's ring takes over and reaches the rim, then goes.
RING_KEYS = {
    "new": [(0, 0.06), (BEAT_FRAMES, 0.5)],
    "old": [(0, 0.5), (BEAT_FRAMES - 1, 1.0), (BEAT_FRAMES, 0.0)],
}
DROP_KEYS = [(0, 1.0), (3, 1.3), (8, 0.0), (BEAT_FRAMES, 0.0)]

# ---------------------------------------------------------------- check dam
DAM_HALF_SPAN = 1.2             # across the flow, crest to crest and a little down the skirts
DAM_DEPTH = 0.55                # along the flow, at its foot
DAM_CREST = 0.32                # crest height over the bed in the channel
NOTCH_HALF = 0.3                # the spillway notch in the middle
NOTCH_DEPTH = 0.06
POND_Z = DAM_CREST - NOTCH_DEPTH - 0.015
POND_LENGTH = 1.7               # upstream, where the pond meets the stream's own surface


class Feature(Burbler):
    """The stream's materials and helpers, without the burbler's mine metadata."""

    def __init__(self, asset_id, kind, root_kind):
        super().__init__(asset_id, kind=kind)
        for key in ("mine", "produces"):
            del self.root[key]
        self.root["kind"] = root_kind
        self.rng = random.Random(sum(map(ord, asset_id)))
        stage = CLARITY[-1]
        self.clear_water = self.material("ClearWater", linear(stage["base_color"]), roughness=0.03)
        self.clear_water.node_tree.nodes["Principled BSDF"].inputs["Alpha"].default_value = 0.55
        self.clear_water.surface_render_method = "BLENDED"
        muddy = CLARITY[0]
        self.mud_water = self.material("MudWater", linear("#3a2a1c"), roughness=0.12)
        self.mud_water.node_tree.nodes["Principled BSDF"].inputs["Alpha"].default_value = 0.9
        self.mud_water.surface_render_method = "BLENDED"
        self.bed = self.material("Bed", linear("#6b5a44"), roughness=0.9)
        self.pebble = self.material("Pebble", linear("#7a6c5a"), roughness=0.7)

    def daylight(self):
        sun = bpy.data.objects.new("PreviewSun", bpy.data.lights.new("PreviewSun", "SUN"))
        sun.data.energy = 3.0
        sun.rotation_euler = (math.radians(55), 0, math.radians(-30))
        self.scene.collection.objects.link(sun)
        self.scene.world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.35, 0.4, 0.5, 1)

    def ground(self, size):
        """A preview-only sand floor and a stretch of the stream, not exported."""
        def fill(bm):
            bmesh.ops.create_grid(bm, x_segments=1, y_segments=1, size=size)
        floor = self.mesh_obj("PreviewFloor", fill, self.sand, (0, 0, -0.002))
        objs = [floor]

        def channel(bm):
            outer = BANK_OUTER
            nx, ny = 40, 24
            verts = [[bm.verts.new((-3.0 + 6.0 * i / nx, -outer + 2 * outer * j / ny, 0.0)) for i in range(nx + 1)]
                     for j in range(ny + 1)]
            for row in verts:
                for v in row:
                    s = abs(v.co.y) - CHANNEL_HALF_WIDTH
                    if s <= 0:
                        h = STREAM_MUD_Z
                    elif s < BANK_WIDTH:
                        h = self.smooth(s / BANK_WIDTH) * BANK_HEIGHT
                    else:
                        h = (1 - self.smooth((s - BANK_WIDTH) / BANK_SKIRT)) * BANK_HEIGHT
                    v.co.z = h
            for j in range(ny):
                for i in range(nx):
                    bm.faces.new((verts[j][i], verts[j][i + 1], verts[j + 1][i + 1], verts[j + 1][i]))
        objs.append(self.mesh_obj("PreviewStream", channel, self.flow))
        return objs


class RipplePool(Feature):
    folder = "streams"

    def __init__(self):
        super().__init__("ripple_pool", "RipplePool", "resource")
        self.root["resource"] = "ripple_pool"
        glint = linear("#cfe6e2")
        self.ring = self.translucent("Ring", glint, 1.4, 0.45)
        self.drop = self.translucent("Drop", glint, 2.0, 0.6)

    def radius(self, angle):
        n = noise.noise(Vector((math.cos(angle) * 1.4, math.sin(angle) * 1.4, 5.0)))
        return POOL_R * (1 + 0.1 * n + 0.04 * math.sin(3 * angle + 0.7))

    def basin(self):
        steps = 48

        def rim(bm):
            # Rings of vertices from the water's edge out over the rim to the ground.
            profile = [(0.0, POOL_WATER_Z - 0.02), (0.12, RIM_HEIGHT), (0.3, RIM_HEIGHT * 0.7),
                       (RIM_OUT, 0.0)]
            rings = []
            for d, z in profile:
                ring = []
                for k in range(steps):
                    a = math.tau * k / steps
                    r = self.radius(a) + d
                    zz = z * (1 + 0.15 * noise.noise(Vector((math.cos(a) * 3, math.sin(a) * 3, d * 4))))
                    ring.append(bm.verts.new((r * math.cos(a), POOL_Y + r * math.sin(a), zz)))
                rings.append(ring)
            for a, b in zip(rings, rings[1:]):
                for k in range(steps):
                    bm.faces.new((a[k], a[(k + 1) % steps], b[(k + 1) % steps], b[k]))

        def disc(z, inset):
            def fill(bm):
                verts = [bm.verts.new(((self.radius(math.tau * k / steps) - inset) * math.cos(math.tau * k / steps),
                                       POOL_Y + (self.radius(math.tau * k / steps) - inset) * math.sin(math.tau * k / steps),
                                       z)) for k in range(steps)]
                bm.faces.new(verts)
                bmesh.ops.triangulate(bm, faces=bm.faces[:])
            return fill
        self.mesh_obj("Rim", rim, self.wet)
        self.mesh_obj("PoolBed", disc(0.012, -0.05), self.bed)
        self.mesh_obj("PoolWater", disc(POOL_WATER_Z, -0.02), self.clear_water)
        for i in range(10):
            a = self.rng.uniform(0, math.tau)
            r = self.radius(a) + self.rng.uniform(0.08, RIM_OUT - 0.08)
            stone = self.sphere(f"Pebble{i}", self.rng.uniform(0.03, 0.07), self.pebble,
                                (r * math.cos(a), POOL_Y + r * math.sin(a), RIM_HEIGHT * 0.5),
                                segments=8, rings=4)
            stone.scale = (self.rng.uniform(1.0, 1.5), 1.0, 0.45)

    def ripples(self):
        ripples = self.node("Ripples", (0, POOL_Y, POOL_WATER_Z + 0.004), shown_while="beat")
        action = bpy.data.actions.new("Ripple_Beat")

        def ring_mesh(bm):
            bmesh.ops.create_circle(bm, cap_ends=False, segments=48, radius=RING_MAX)
            outer = [v for v in bm.verts]
            inner = [bm.verts.new(v.co * 0.96) for v in outer]
            for k in range(len(outer)):
                bm.faces.new((outer[k], outer[(k + 1) % len(outer)], inner[(k + 1) % len(outer)], inner[k]))
        for name, keys in RING_KEYS.items():
            ring = self.mesh_obj(f"Ripple_{name.title()}", ring_mesh, self.ring, parent=ripples, smooth=False)
            ring.animation_data_create()
            ring.animation_data.action = action
            for frame, s in keys:
                ring.scale = (s, s, 1.0)
                ring.keyframe_insert("scale", frame=frame)
        drop = self.sphere("Ripple_Drop", 0.035, self.drop, (0, 0, 0), parent=ripples, segments=12, rings=6)
        drop.animation_data_create()
        drop.animation_data.action = action
        for frame, s in DROP_KEYS:
            drop.scale = (s, s, s * 0.6)
            drop.keyframe_insert("scale", frame=frame)
        return ripples

    def build(self):
        self.basin()
        ripples = self.ripples()
        self.node("Interact", (0, POOL_Y, 0.45), role="interact", resource="ripple_pool",
                  prompt="Collect oscilloscope")
        self.scene.frame_start, self.scene.frame_end = 0, BEAT_FRAMES
        self.scene.frame_set(int(BEAT_FRAMES * 0.6))   # previews catch both rings mid-flight
        profile = {
            "id": "ripple_pool", "model": f"{self.model_ref}#Scene0", "kind": "resource",
            "resource": "ripple_pool",
            "placement": {"along": "laid stream", "frame": "the stream segment's: flow +X, origin "
                          "at the channel centre on the ground", "pool_centre_y_m": POOL_Y,
                          "side": "left bank (+Y); turned half a turn it sits on the right bank",
                          "spacing": "Poisson along the stream, mean 12 m (add-ripple-pool D1)"},
            "size_metres": {"diameter": round(2 * (POOL_R + RIM_OUT), 2), "height": RIM_HEIGHT},
            "ripples": {"node": "Ripples", "shown": "while the stream sounds; shipped at scale 0",
                        "clip": "Ripple_Beat", "seconds": BEAT_FRAMES / 24, "loop": True,
                        "speed": "BPM / 60", "ring_leaves_at_seconds": 0.0},
            "interact": "Interact",
            "lod": "none: small and animated",
            "licence": "CC0-1.0: generated by tools/blender/create_stream_features.py",
        }
        preview_only = []

        def stage():
            preview_only.extend(self.ground(8.0))
            self.daylight()
            ripples.scale = (1, 1, 1)
        ripples.scale = (0, 0, 0)
        return self.finish(profile, aim=(0, POOL_Y - 0.6, 0.0), camera_location=(-2.6, -1.0, 2.4),
                           before_render=stage)


class CheckDam(Feature):
    folder = "machines"

    def __init__(self):
        super().__init__("reed_water_purifier", "CheckDam", "machine")
        self.root["machine"] = "reed_water_purifier"
        self.stick = self.material("Stick", linear("#4e3d2c"), roughness=0.9)
        self.stick_peeled = self.material("StickPeeled", linear("#8a7356"), roughness=0.8)
        self.reed_stalk = self.material("ReedStalk", linear(REED_COLORS["stalk_dry"]), roughness=0.8)
        self.foam = self.material("Foam", linear("#a89c86"), roughness=0.6)
        self.spill = self.translucent("Spill", linear("#5e5444"), 0.05, 0.35)

    def crest(self, y):
        """Crest height across the flow: over the banks it rides their crest, and the
        notch in the middle is where the water spills."""
        s = abs(y) - CHANNEL_HALF_WIDTH
        bank = 0.0 if s <= 0 else (self.smooth(s / BANK_WIDTH) * BANK_HEIGHT if s < BANK_WIDTH
                                   else (1 - self.smooth((s - BANK_WIDTH) / BANK_SKIRT)) * BANK_HEIGHT)
        h = max(DAM_CREST, bank + 0.08)
        if abs(y) < NOTCH_HALF:
            h -= NOTCH_DEPTH * self.smooth(1 - abs(y) / NOTCH_HALF)
        taper = 1 - self.smooth((abs(y) - (DAM_HALF_SPAN - 0.25)) / 0.25)
        return h * max(taper, 0.0)

    def core(self, name, segments_y, segments_x, mat):
        def fill(bm):
            verts = []
            for j in range(segments_y + 1):
                y = -DAM_HALF_SPAN + 2 * DAM_HALF_SPAN * j / segments_y
                top = self.crest(y)
                row = []
                for i in range(segments_x + 1):
                    u = i / segments_x                 # 0 upstream foot .. 1 downstream foot
                    x = (u - 0.45) * DAM_DEPTH
                    z = top * math.sin(math.pi * min(u * 1.05, 1.0)) ** 0.6
                    z *= 1 + 0.06 * noise.noise(Vector((x * 6, y * 4, 1.5)))
                    row.append(bm.verts.new((x, y, max(z, 0.0))))
                verts.append(row)
            for j in range(segments_y):
                for i in range(segments_x):
                    bm.faces.new((verts[j][i], verts[j][i + 1], verts[j + 1][i + 1], verts[j + 1][i]))
        return self.mesh_obj(name, fill, mat)

    def sticks(self):
        objs = []
        for i in range(16):
            length = self.rng.uniform(0.8, 1.6)
            r = self.rng.uniform(0.022, 0.045)
            y = self.rng.uniform(-0.35, 0.35)
            x = self.rng.uniform(-0.2, 0.12)
            z = self.crest(y) * self.rng.uniform(0.55, 1.0)
            mat = self.stick_peeled if self.rng.random() < 0.3 else self.stick
            stick = self.cylinder(f"Stick_{i}", r, length, (x, y, z), mat, segments=7)
            stick.rotation_euler = (math.pi / 2 + self.rng.uniform(-0.25, 0.25), self.rng.uniform(-0.15, 0.15),
                                    self.rng.uniform(-0.3, 0.3))
            objs.append(stick)
        for i, y in enumerate((-0.62, -0.2, 0.22, 0.6)):   # stakes driven in below the dam
            stake = self.cylinder(f"Stake_{i}", 0.032, 0.62, (0.2, y, 0.2), self.stick, segments=7)
            stake.rotation_euler = (self.rng.uniform(-0.12, 0.12), -0.3, 0)
            objs.append(stake)
        return objs

    def reed_bundles(self):
        objs = []
        for b in range(5):
            y = -0.8 + 0.4 * b
            for k in range(6):
                stalk = self.cylinder(f"Reed_{b}_{k}", 0.009, 0.7, (-0.06 + 0.012 * k, y + self.rng.uniform(-0.05, 0.05),
                                      self.crest(y) + 0.015 + 0.01 * (k % 2)), self.reed_stalk, segments=5)
                stalk.rotation_euler = (math.pi / 2, 0, self.rng.uniform(-0.06, 0.06))
                objs.append(stalk)
        return objs

    def water(self):
        def pond(bm):
            nx, ny = 12, 10
            half = CHANNEL_HALF_WIDTH + 0.25
            verts = [[bm.verts.new((-0.12 - POND_LENGTH * i / nx, -half + 2 * half * j / ny, 0.0))
                      for i in range(nx + 1)] for j in range(ny + 1)]
            for row in verts:
                for v in row:
                    t = (-0.12 - v.co.x) / POND_LENGTH              # 0 at the dam .. 1 upstream
                    v.co.z = STREAM_MUD_Z + (POND_Z - STREAM_MUD_Z) * (1 - self.smooth(t))
            for j in range(ny):
                for i in range(nx):
                    bm.faces.new((verts[j][i], verts[j][i + 1], verts[j + 1][i + 1], verts[j + 1][i]))
        self.mesh_obj("Pond", pond, self.mud_water)

        def spill(bm):
            nx, ny = 10, 6
            verts = [[None] * (nx + 1) for _ in range(ny + 1)]
            for j in range(ny + 1):
                y = -NOTCH_HALF * 0.8 + 1.6 * NOTCH_HALF * j / ny
                for i in range(nx + 1):
                    u = i / nx
                    x = 0.02 + 0.42 * u
                    z = (DAM_CREST - NOTCH_DEPTH + 0.012) * (1 - u) ** 1.6 + STREAM_MUD_Z * u
                    verts[j][i] = bm.verts.new((x, y * (1 + 0.3 * u), max(z, STREAM_MUD_Z) + 0.01))
            for j in range(ny):
                for i in range(nx):
                    bm.faces.new((verts[j][i], verts[j][i + 1], verts[j + 1][i + 1], verts[j + 1][i]))
        self.mesh_obj("Spill", spill, self.spill)
        for i in range(14):
            self.sphere(f"Foam_{i}", self.rng.uniform(0.015, 0.035), self.foam,
                        (self.rng.uniform(0.4, 0.75), self.rng.uniform(-0.28, 0.28), STREAM_MUD_Z + 0.008),
                        segments=8, rings=4).scale = (1.5, 1.0, 0.3)

    def build(self):
        self.core("Core", 48, 10, self.mud)
        self.sticks()
        self.reed_bundles()
        self.water()
        far = [self.core("Core_Far", 12, 4, self.mud)]
        far[0].location.z += 0.02
        self.node("Interact", (0.0, -(BANK_OUTER - 0.2), 0.55), action="configure",
                  prompt="Configure purifier")
        self.node("Outlet", (0.75, 0.0, STREAM_MUD_Z), clarity_lift=1)
        self.lod_levels(far)
        tris = sum(len(p.vertices) - 2 for o in self.collection.objects if o.type == "MESH"
                   and not o.name.endswith("_Far") for p in o.data.polygons)
        profile = {
            "id": "reed_water_purifier", "model": f"{self.model_ref}#Scene0", "kind": "machine",
            "machine": "reed_water_purifier",
            "placement": {"on": "laid stream segment", "snap": "across the bed, square to the flow",
                          "frame": "the stream segment's: flow +X, origin at the channel centre on "
                                   "the ground", "elsewhere": "the hologram shows red"},
            "size_metres": {"across": 2 * DAM_HALF_SPAN, "along": DAM_DEPTH, "crest": DAM_CREST},
            "cleaning": {"node": "Outlet", "clarity_lift": 1,
                         "levels": ["muddy", "silty", "cloudy", "clear"]},
            "pond": {"node": "Pond", "behind": "-X (upstream)", "crest_z": POND_Z},
            "interact": "Interact",
            "configure": purifier_rack(),
            "triangles": {"near": tris},
            "licence": "CC0-1.0: generated by tools/blender/create_stream_features.py",
        }

        def stage():
            self.ground(8.0)
            self.daylight()
        return self.finish(profile, aim=(-0.3, 0, 0.12), camera_location=(1.2, -3.0, 2.3),
                           before_render=stage)


def purifier_rack():
    places = [
        ("clock", "clock", 8, "burbler_clock"),
        ("vco_low", "vco", 8, "saw_vco"),
        ("vco_high", "vco", 8, "triangle_vco"),
        ("mix", "mixer", 8, "mixer"),
        ("filter", "vcf", 8, "vcf"),
        ("vca", "vca", 4, "volume"),
        ("scope", "scope", 10, "scope"),
    ]
    cables = [
        ("rust", "vco_low", "out", "mix", "in_1", "Saw OUT to mixer IN 1"),
        ("ochre", "vco_high", "out", "mix", "in_2", "Triangle OUT to mixer IN 2"),
        ("verdigris", "mix", "mix", "filter", "in", "Mixer MIX to filter IN"),
        ("rust", "filter", "hpf", "vca", "in", "Filter HPF to volume IN"),
        ("ochre", "clock", "gate", "vca", "trig", "Clock GATE to volume TRIG"),
        ("verdigris", "vca", "out", "scope", "in", "Volume OUT to scope IN"),
    ]
    return {
        "title": "REED WATER PURIFIER",
        "hud": "hud/rack_frame/rack_frame.glb",
        "interact": {"node": "Interact", "range_m": 3.0, "prompt": "Configure the purifier"},
        "places": [
            {"place": place, "kind": kind, "hp": hp, "fixed": True, "module": module,
             **({"borrowed": True} if place == "clock" else {})}
            for place, kind, hp, module in places
        ],
        "normalled_output": {"place": "scope", "port": "out"},
        "hearing_radius_metres": 20.0,
        "cables": [
            {"color": color, "from": {"place": start, "port": output},
             "to": {"place": end, "port": input_port}, "hint": hint}
            for color, start, output, end, input_port, hint in cables
        ],
        "patched_hint": "Tune saw to C2, triangle to C4, and filter cutoff to C5.",
    }


def main():
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    builders = {"ripple_pool": RipplePool, "reed_water_purifier": CheckDam}
    for name in args or builders:
        print(builders[name]().build())


if __name__ == "__main__":
    main()
