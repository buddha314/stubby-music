"""The campfire: a bundle of sticks, the craft bench (add-campfire-craft-bench).

A teepee of dry sticks over a bed of ash, with kindling at its foot. It is lit by music:
a spark voice struck by a snail-shell sequencer on beats 2 and 4, timed by the nearest
running clock (a burbler or the stream it feeds) within `clock_reach_m`. Lit, it is the
craft bench, and it burns only while that clock runs; then it dies down to embers.

Game-facing nodes (empties and parts, glTF extras):
  Fire        the flames; ships hidden (start_scale 0), shown while burning
  Embers      glowing coals; ships hidden, shown as the fire dies down
  Light_fire  the fire's light socket (extras: light, rest/peak strength)
  Sparks      where a strike's sparks burst (the game's particles)
  Interact    where E reaches the camp (its rack unlit, the crafting screen lit)
  campfire_LOD0 / campfire_LOD1   the near and far levels

Reproduce: blender -b --factory-startup -P tools/blender/create_campfire.py
Writes assets/structures/campfire/ (model, profile), assets-src/blender/structures/
campfire.blend and previews/structures/campfire/ (unlit, burning, embers).
"""
from pathlib import Path
import json
import math
import random
import sys

import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_burbler import Burbler, linear  # noqa: E402
from module_kit import PROJECT  # noqa: E402

STICKS = 14
APEX = 0.95        # metres: where the sticks meet
BASE = 0.38        # metres: the radius of the bundle's foot
CLOCK_REACH_M = 5.0
LIGHT_PATTERN = [False, True, False, True]   # strike on 2 and 4: the backbeat


class Campfire(Burbler):
    folder = "structures"

    def __init__(self):
        Burbler.__init__(self, "campfire", kind="Campfire")
        for key in ("mine", "produces"):
            del self.root[key]
        self.root["kind"] = "structure"
        self.root["buildable"] = "campfire"
        self.rng = random.Random(61)
        m = self.material
        self.bark = m("Bark", linear("#5e4d3c"), roughness=0.92)
        self.cut = m("CutEnd", linear("#8a7558"), roughness=0.85)
        self.ash = m("Ash", linear("#2a2622"), roughness=1.0)
        amber, flare, coal = linear("#ffb350"), linear("#ff7a2a"), linear("#ff4a1a")
        self.flame = m("FlameCore", amber, emission=amber, strength=12.0)
        self.flame_outer = self.translucent("FlameOuter", flare, 6.0, 0.55)
        self.coal = m("Coal", linear("#3a1a10"), roughness=0.8, emission=coal, strength=4.0)

    @staticmethod
    def tube(bm, points, radii, sides=6):
        rings, normal = [], None
        for i, (p, r) in enumerate(zip(points, radii)):
            a, b = points[max(i - 1, 0)], points[min(i + 1, len(points) - 1)]
            t = (b - a).normalized()
            normal = t.orthogonal().normalized() if normal is None else (normal - t * normal.dot(t)).normalized()
            side = t.cross(normal)
            rings.append([bm.verts.new(p + r * (normal * math.cos(2 * math.pi * k / sides)
                                                + side * math.sin(2 * math.pi * k / sides))) for k in range(sides)])
        for ra, rb in zip(rings, rings[1:]):
            for k in range(sides):
                bm.faces.new((ra[k], ra[(k + 1) % sides], rb[(k + 1) % sides], rb[k]))
        bm.faces.new(rings[0][::-1])
        bm.faces.new(rings[-1])
        return rings

    def stick_line(self, k, count, jitter=True):
        """A stick from the ground at the bundle's foot, leaning in past the apex."""
        r = self.rng
        a = math.tau * k / count + (r.uniform(-0.12, 0.12) if jitter else 0)
        foot = Vector((math.cos(a) * (BASE + (r.uniform(-0.06, 0.06) if jitter else 0)),
                       math.sin(a) * (BASE + (r.uniform(-0.06, 0.06) if jitter else 0)), -0.03))
        apex = Vector((r.uniform(-0.04, 0.04) if jitter else 0, r.uniform(-0.04, 0.04) if jitter else 0,
                       APEX + (r.uniform(-0.08, 0.06) if jitter else 0)))
        tip = apex + (apex - foot).normalized() * (r.uniform(0.08, 0.2) if jitter else 0.12)
        return [foot.lerp(tip, i / 4) for i in range(5)]

    def bundle(self):
        def sticks(bm):
            for k in range(STICKS):
                pts = self.stick_line(k, STICKS)
                r0 = self.rng.uniform(0.022, 0.032)
                self.tube(bm, pts, [r0, r0 * 0.92, r0 * 0.85, r0 * 0.75, r0 * 0.6])

        def kindling(bm):
            for k in range(7):
                a = math.tau * k / 7 + self.rng.uniform(-0.3, 0.3)
                c = Vector((math.cos(a) * 0.2, math.sin(a) * 0.2, 0.03))
                d = Vector((-math.sin(a), math.cos(a), 0)) * self.rng.uniform(0.14, 0.22)
                self.tube(bm, [c - d, c, c + d], [0.012, 0.014, 0.01], sides=5)

        def ash(bm):
            centre = bm.verts.new((0, 0, 0.004))
            rim = [bm.verts.new((math.cos(t) * 0.55 * (1 + 0.08 * math.sin(5 * t)),
                                 math.sin(t) * 0.55 * (1 + 0.08 * math.sin(5 * t)), 0.004))
                   for t in (math.tau * i / 24 for i in range(24))]
            for a, b in zip(rim, rim[1:] + rim[:1]):
                bm.faces.new((centre, a, b))
        self.mesh_obj("Sticks", sticks, self.bark)
        self.mesh_obj("Kindling", kindling, self.cut)
        self.mesh_obj("Ash", ash, self.ash, smooth=False)

    def fire(self):
        fire = self.node("Fire", (0, 0, 0))

        def sheets(bm, height, width):
            for k in range(3):
                a = math.pi * k / 3
                dx, dy = math.cos(a) * width / 2, math.sin(a) * width / 2
                v = [bm.verts.new(c) for c in ((-dx, -dy, 0.04), (dx, dy, 0.04), (0, 0, height))]
                bm.faces.new(v)
        self.mesh_obj("FlameCore", lambda bm: sheets(bm, 0.55, 0.22), self.flame, parent=fire, smooth=False)
        self.mesh_obj("FlameOuter", lambda bm: sheets(bm, 0.8, 0.38), self.flame_outer, parent=fire, smooth=False)
        fire.scale = (0, 0, 0)   # hidden until lit (exported at 1, tagged start_scale 0)
        embers = self.node("Embers", (0, 0, 0))

        def coals(bm):
            for k in range(8):
                a = math.tau * k / 8 + self.rng.uniform(-0.3, 0.3)
                c = Vector((math.cos(a) * 0.14, math.sin(a) * 0.14, 0.02))
                s = self.rng.uniform(0.035, 0.055)
                v = [bm.verts.new(c + Vector(o) * s) for o in
                     ((1, 0, 0), (0, 1, 0), (-1, 0, 0), (0, -1, 0), (0, 0, 0.6))]
                for i in range(4):
                    bm.faces.new((v[i], v[(i + 1) % 4], v[4]))
        self.mesh_obj("Coals", coals, self.coal, parent=embers, smooth=False)
        embers.scale = (0, 0, 0)
        self.node("Light_fire", (0, 0, 0.45), light="fire", rest_strength=0.0, peak_strength=12.0)
        self.node("Sparks", (0, 0, APEX), role="sparks")
        self.node("Interact", (0, 0, 0.6), role="interact", buildable="campfire")

    def far(self):
        """Six plain sticks: the bundle's outline from afar."""
        def fill(bm):
            for k in range(6):
                self.tube(bm, self.stick_line(k, 6, jitter=False), [0.04, 0.035, 0.03, 0.025, 0.02], sides=4)
        obj = self.mesh_obj("Far", fill, self.bark, smooth=False)
        return [obj]

    def triangles(self, objs):
        return sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in objs if o.type == "MESH")

    def build(self):
        self.bundle()
        self.fire()
        far = self.far()
        near = [o for o in self.collection.objects if o.type == "MESH" and o not in far]
        self.lod_levels(far)
        profile = {
            "id": "campfire", "model": f"{self.model_ref}#Scene0", "kind": "structure",
            "buildable": "campfire",
            "about": "A bundle of sticks, the craft bench (add-campfire-craft-bench). Lit by a "
                     "spark voice struck by a snail-shell sequencer on beats 2 and 4, timed by "
                     "the nearest running clock within clock_reach_m; burns only while that "
                     "clock runs, then dies to embers. Lit, E opens the crafting screen, and "
                     "recipes with a bpm range are made only at that clock's tempo.",
            "size_metres": {"width": 2 * BASE + 0.1, "depth": 2 * BASE + 0.1, "height": APEX + 0.2},
            "clock_reach_m": CLOCK_REACH_M,
            "light_pattern": LIGHT_PATTERN,
            "states": {"unlit": "Fire and Embers at scale 0",
                       "burning": "Fire at scale 1, Light_fire at peak",
                       "embers": "Embers at scale 1, Fire at 0, Light_fire low"},
            "configure": {
                "screen": "eurorack_hud",
                "hud": "hud/rack_frame/rack_frame.glb",
                "title": "CAMPFIRE",
                "interact": {
                    "node": "Interact",
                    "key": "E",
                    "range_m": 3.0,
                    "prompt": "Light the campfire"
                },
                "places": [
                    {
                        "place": "clock",
                        "kind": "clock",
                        "hp": 8,
                        "fixed": True,
                        "module": "burbler_clock",
                        "borrowed": True
                    },
                    {
                        "place": "seq",
                        "kind": "sequencer",
                        "hp": 8,
                        "fixed": False,
                        "accepts": [
                            {
                                "item": "snail_shell",
                                "module": "snail_sequencer"
                            }
                        ]
                    },
                    {
                        "place": "voice",
                        "kind": "vco",
                        "hp": 8,
                        "fixed": False,
                        "accepts": [
                            {
                                "item": "square_wave",
                                "module": "square_vco"
                            }
                        ]
                    },
                    {
                        "place": "volume",
                        "kind": "vca",
                        "hp": 4,
                        "fixed": True,
                        "module": "volume"
                    }
                ],
                "normalled_output": {
                    "place": "volume",
                    "port": "out"
                },
                "hearing_radius_metres": 20.0,
                "cables": [
                    {
                        "color": "verdigris",
                        "from": {
                            "place": "clock",
                            "port": "gate"
                        },
                        "to": {
                            "place": "seq",
                            "port": "clock"
                        },
                        "hint": "clock GATE to sequencer CLOCK"
                    },
                    {
                        "color": "ochre",
                        "from": {
                            "place": "seq",
                            "port": "gate"
                        },
                        "to": {
                            "place": "volume",
                            "port": "trig"
                        },
                        "hint": "sequencer GATE to volume TRIG"
                    },
                    {
                        "color": "rust",
                        "from": {
                            "place": "voice",
                            "port": "out"
                        },
                        "to": {
                            "place": "volume",
                            "port": "in"
                        },
                        "hint": "VCO OUT to volume IN"
                    }
                ],
                "clock_source": "the nearest running clock within clock_reach_m (a sounding burbler, or a stream it feeds), lent to the borrowed clock place; none in reach, no clock"
            },
            "triangles": {"near": self.triangles(near), "far": self.triangles(far)},
            "licence": "CC0-1.0: generated by tools/blender/create_campfire.py",
        }

        def daylight():
            sun = bpy.data.objects.new("PreviewSun", bpy.data.lights.new("PreviewSun", "SUN"))
            sun.data.energy = 3.0
            sun.rotation_euler = (math.radians(55), 0, math.radians(-30))
            self.scene.collection.objects.link(sun)
        report = self.finish(profile, aim=(0, 0, 0.45), camera_location=(1.4, -2.2, 1.2),
                             before_render=daylight)
        if bpy.app.background:
            out = PROJECT / "previews/structures/campfire"
            fire = bpy.data.objects["Fire"]
            embers = bpy.data.objects["Embers"]
            for name, (f, e) in (("preview_burning", (1, 0)), ("preview_embers", (0, 1))):
                fire.scale = (f, f, f)
                embers.scale = (e, e, e)
                self.scene.render.filepath = str(out / f"{name}.png")
                bpy.ops.render.render(write_still=True, scene=self.scene.name)
        report.update(triangles=profile["triangles"])
        return report


print(json.dumps(Campfire().build()))
