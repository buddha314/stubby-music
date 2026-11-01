"""Stream segment: a straight, tileable length of the stream a burbler feeds
(add-mudpit-burbler 1.7).

Streams carry the music: they pulse at the burbler's BPM, so a stream is the played
sequence laid out in space. This is one 4 m length of the burbler's own channel (its
bank profile and mud height are imported, so a segment meets the burbler's mouth
exactly): 1 m of water between banked sides, built up from the ground. Flow runs
along +X; the origin is the inlet's centre on the ground, so segments chain port to
port. It is subdivided along its length so the game can bend it along a path.

The water is sheet music. A five-line staff of luminous lines floats on the surface
like a ribbon: the lines keep a fixed distance (each is offset along the wave's normal,
so they never bunch or cross), and the whole ribbon waves from bank to bank, the wave
travelling downstream at the water's speed. Each clock tick at the source emits a note
that rides the staff downstream at that speed, so notes sit `flow_speed * 60 / BPM`
metres apart and stay on the same crest of the moving ribbon. Every `beats_per_bar`
ticks a bar line rides down with them, so measures are `beats_per_bar` beats long at
the clock's BPM: faster water spreads the music out, a faster clock packs it in. The water goes from muddy to clear as cleaning processes are identified; the
profile's `clarity` table gives its look at each stage, and the staff glows brighter
and crisper as the water clears.

  Reeds_L, Reeds_R     the reeds outlining each bank: a segment is built from 6 reeds
                       (add-reeds), and they line it. Dry while the water is muddy,
                       green as it clears; pivots at the ground, for a gentle sway
  Port_in, Port_out    extras: stream = mud. +X along the flow
  Port_<end>_Halo      upright arch over each end: port highlighting while building
  Staff                extras: wave_amplitude_m, wavelength_m. Five lines a fixed
                       spacing apart, built straight with two morph targets, WaveSin and
                       WaveCos; weights (cos p, -sin p) give the centre line
                       y = A sin(2 pi x / wavelength - p). glTF clip `Staff_Flow` loops p
                       over one wavelength at the profile's flow speed
  Pulse_0              The note template the game spawns per tick and slides downstream
                       (see the profile's `notes` for its position on the ribbon)
  Bar_0                The bar-line template: spans the staff across the flow; the game
                       spawns one every `beats_per_bar` ticks (see the profile's `bars`)

Reproduce: blender -b --factory-startup -P tools/blender/create_stream.py
Writes assets/streams/stream_segment/ (profile and previews: muddy, silty, clear).
"""
from pathlib import Path
import math
import random
import shutil
import subprocess
import sys

import bmesh
import bpy
from mathutils import Vector, noise

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_burbler import (  # noqa: E402
    BANK_HEIGHT, BANK_SKIRT, BANK_WIDTH, CHANNEL_HALF_WIDTH, STREAM_MUD_Z, Burbler, linear,
)
from module_kit import PROJECT  # noqa: E402
from create_reeds import COLORS as REED_COLORS, ReedClump  # noqa: E402

LENGTH = 4.0
FLOW_SPEED = 1.5           # m/s until the fluid simulation supplies it
WATER_Z = STREAM_MUD_Z
BED_Z = 0.006
STAFF_LINES = 5
STAFF_SPACING = 0.12       # line to line; the staff spans 0.48 m of the 1 m channel
STAFF_WAVE = 0.07          # the staff's sideways wave, amplitude in metres
STAFF_WAVELENGTH = 2.0     # two waves per segment, so it tiles
STAFF_Z = WATER_Z + 0.012  # floats just above the surface's ripple
BEATS_PER_BAR = 4
FLOW_FPS = 24              # frames of the baked Staff_Flow clip per second
PREVIEW_BPM = 90
# The water's look at each stage of clarity (sRGB, for the game to blend between).
# Muddy is 0.98, not 1, so the material exports alpha-blended and can fade to clear.
# `staff_strength` is the staff lines' emission; their halo shows from silty on.
CLARITY = [
    {"at": 0.0, "name": "muddy", "base_color": "#2a1d13", "roughness": 0.5, "alpha": 0.98,
     "staff_strength": 2.5, "staff_halo": False, "reed_greenness": 0.0},
    {"at": 0.5, "name": "silty", "base_color": "#5a4630", "roughness": 0.22, "alpha": 0.72,
     "staff_strength": 5.0, "staff_halo": True, "reed_greenness": 0.5},
    {"at": 1.0, "name": "clear", "base_color": "#6f8a88", "roughness": 0.04, "alpha": 0.22,
     "staff_strength": 8.0, "staff_halo": True, "reed_greenness": 1.0},
]


class Stream(Burbler):
    folder = "streams"

    def __init__(self):
        super().__init__("stream_segment", kind="Stream")
        for key in ("mine", "produces"):
            del self.root[key]
        self.root["kind"] = "stream"
        self.root["length_m"] = LENGTH
        self.water = self.material("Water", linear(CLARITY[0]["base_color"]), roughness=CLARITY[0]["roughness"])
        self.water.surface_render_method = "BLENDED"
        self.bed = self.material("Bed", linear("#6b5a44"), roughness=0.9)
        self.pebble = self.material("Pebble", linear("#7a6c5a"), roughness=0.7)
        self.port_halo = self.translucent("PortHalo", linear("#ffb350"), 1.6, 0.2)
        amber = linear("#ffb350")
        self.staff_line = self.material("Staff", amber, emission=amber, strength=CLARITY[0]["staff_strength"])
        self.staff_halo = self.translucent("StaffHalo", amber, 0.9, 0.1)
        self.reed_stalk = self.material("ReedStalk", linear(REED_COLORS["stalk_dry"]), roughness=0.8)
        self.reed_blade = self.material("ReedBlade", linear(REED_COLORS["blade_dry"]), roughness=0.85)
        self.reed_head = self.material("ReedHead", linear(REED_COLORS["head"]), roughness=0.95)

    @staticmethod
    def wave(x):
        """The staff's centre line: tiles, since the segment holds whole wavelengths."""
        return STAFF_WAVE * math.sin(2 * math.pi * x / STAFF_WAVELENGTH)

    @staticmethod
    def ribbon_point(x, offset, phase=0.0, shape=math.sin):
        """A point `offset` metres across the staff ribbon from its centre line
        y = A shape(2 pi x / wavelength - phase), measured along the line's normal, so
        every line of the staff keeps the same distance from its neighbours."""
        k = 2 * math.pi / STAFF_WAVELENGTH
        y = STAFF_WAVE * shape(k * x - phase)
        dshape = math.cos if shape is math.sin else (lambda a: -math.sin(a))
        slope = STAFF_WAVE * k * dshape(k * x - phase)
        n = math.hypot(slope, 1.0)
        return (x - offset * slope / n, y + offset / n)

    @staticmethod
    def ends_fade(x):
        """0 at both ends, 1 in the middle: irregularity fades out so segments tile."""
        return math.sin(math.pi * min(max(x / LENGTH, 0.0), 1.0))

    def bank_height(self, x, y):
        s = abs(y) - CHANNEL_HALF_WIDTH
        if s <= 0:
            return 0.0
        h = self.smooth(s / BANK_WIDTH) if s < BANK_WIDTH else 1 - self.smooth((s - BANK_WIDTH) / BANK_SKIRT)
        return h * BANK_HEIGHT * (1 + 0.25 * self.ends_fade(x) * noise.noise(Vector((x * 1.7, y * 2.3, 0.5))))

    def grid(self, name, mat, y0, y1, ny, height, nx=24, keep=None):
        def fill(bm):
            verts = [[bm.verts.new((LENGTH * i / nx, y0 + (y1 - y0) * j / ny, 0.0)) for i in range(nx + 1)]
                     for j in range(ny + 1)]
            for row in verts:
                for v in row:
                    v.co.z = height(v.co.x, v.co.y)
            for j in range(ny):
                for i in range(nx):
                    quad = (verts[j][i], verts[j][i + 1], verts[j + 1][i + 1], verts[j + 1][i])
                    if keep is None or keep(quad):
                        bm.faces.new(quad)
            bmesh.ops.delete(bm, geom=[v for v in bm.verts if not v.link_faces], context="VERTS")
        return self.mesh_obj(name, fill, mat)

    def channel(self):
        outer = CHANNEL_HALF_WIDTH + BANK_WIDTH + BANK_SKIRT
        for side, (y0, y1) in (("L", (CHANNEL_HALF_WIDTH - 0.05, outer)), ("R", (-outer, -CHANNEL_HALF_WIDTH + 0.05))):
            self.grid(f"Bank{side}", self.mud, y0, y1, 14, self.bank_height)
        w = CHANNEL_HALF_WIDTH + 0.08  # bed and water run on under the banks' inner slope
        self.grid("Bed", self.bed, -w, w, 8, lambda x, y: BED_Z + 0.004 * noise.noise(Vector((x * 3, y * 3, 9.0))))
        self.grid("Water", self.water, -w, w, 8,
                  lambda x, y: WATER_Z + 0.006 * self.ends_fade(x) * noise.noise(Vector((x * 1.4, y * 2, 3.0))))
        for i in range(18):
            x = self.rng.uniform(0.2, LENGTH - 0.2)
            y = self.rng.uniform(-CHANNEL_HALF_WIDTH + 0.08, CHANNEL_HALF_WIDTH - 0.08)
            stone = self.sphere(f"Pebble{i}", self.rng.uniform(0.025, 0.06), self.pebble, (x, y, BED_Z),
                                segments=8, rings=4)
            stone.scale = (self.rng.uniform(1.0, 1.6), 1.0, 0.35)
            stone.rotation_euler.z = self.rng.uniform(0, math.pi)

    def staff_ribbon(self, name, offset, width, mat, parent, z):
        """One staff line: built straight, with WaveSin and WaveCos morph targets."""
        n = 64
        xs = [LENGTH * i / n for i in range(n + 1)]
        straight = [(x, offset, z) for x in xs]
        obj = self.ribbon(name, straight, width, mat, parent=parent)
        obj.shape_key_add(name="Basis")
        for key_name, shape in (("WaveSin", math.sin), ("WaveCos", math.cos)):
            curve = [(*self.ribbon_point(x, offset, shape=shape), z) for x in xs]
            temp = self.ribbon(f"{name}_{key_name}Temp", curve, width, mat)
            key = obj.shape_key_add(name=key_name)
            key.slider_min, key.slider_max = -1.0, 1.0
            for v, t in zip(key.data, temp.data.vertices):
                v.co = t.co
            bpy.data.objects.remove(temp)
        obj.data.shape_keys.key_blocks["WaveSin"].value = 1.0  # at rest: phase 0
        return obj

    def staff(self):
        """Five lines a fixed distance apart on a ribbon that waves across the water."""
        node = self.node("Staff", (0, 0, 0), wave_amplitude_m=STAFF_WAVE, wavelength_m=STAFF_WAVELENGTH,
                         spacing_m=STAFF_SPACING)
        lines = []
        for k in range(STAFF_LINES):
            offset = (k - (STAFF_LINES - 1) / 2) * STAFF_SPACING
            lines.append(self.staff_ribbon(f"StaffLine{k}", offset, 0.009, self.staff_line, node, STAFF_Z))
            lines.append(self.staff_ribbon(f"StaffHalo{k}", offset, 0.035, self.staff_halo, node,
                                           STAFF_Z + 0.002))
        self.flow_clip(lines)
        return node

    def flow_clip(self, lines):
        """Bake one wavelength of travel at FLOW_SPEED into `Staff_Flow`, shared by every
        line, so the ribbon's wave moves downstream with the water and loops."""
        period = STAFF_WAVELENGTH / FLOW_SPEED
        frames = round(period * FLOW_FPS)
        self.scene.render.fps = FLOW_FPS
        action = bpy.data.actions.new("Staff_Flow")
        for obj in lines:
            keys = obj.data.shape_keys
            keys.animation_data_create()
            keys.animation_data.action = action
            for f in range(frames + 1):
                p = 2 * math.pi * f / frames
                for key_name, weight in (("WaveSin", math.cos(p)), ("WaveCos", -math.sin(p))):
                    block = keys.key_blocks[key_name]
                    block.value = weight
                    block.keyframe_insert("value", frame=f)
            keys.key_blocks["WaveSin"].value, keys.key_blocks["WaveCos"].value = 1.0, 0.0
        self.flow_frames = frames

    def bar(self, name, x, parent=None):
        """A bar line across the staff at x, square to the ribbon there."""
        span = (STAFF_LINES - 1) / 2 * STAFF_SPACING
        cx, cy = self.ribbon_point(x, 0.0)
        k = 2 * math.pi / STAFF_WAVELENGTH
        yaw = math.atan(STAFF_WAVE * k * math.cos(k * x))
        node = self.node(name, (cx, cy, STAFF_Z + 0.003), parent=parent)
        node.rotation_euler.z = yaw
        self.ribbon(f"{name}_Line", [(0, -span, 0), (0, span, 0)], 0.012, self.staff_line, up=(0, 0, 1),
                    parent=node)
        self.ribbon(f"{name}_Halo", [(0, -span, 0.002), (0, span, 0.002)], 0.04, self.staff_halo,
                    up=(0, 0, 1), parent=node)
        return node

    def note(self, name, x, parent=None, step=0):
        """A quarter note lying on the staff: head on a line, stem toward the +Y bank."""
        px, py = self.ribbon_point(x, step * STAFF_SPACING / 2)
        node = self.node(name, (px, py, STAFF_Z + 0.004), parent=parent)
        head = self.sphere(f"{name}_Head", 0.062, self.wisp_core, (0, 0, 0), node, 14, 7)
        head.scale = (1.35, 1.0, 0.45)
        head.rotation_euler.z = 0.45
        self.sphere(f"{name}_Halo", 0.1, self.halo, (0, 0, 0), node, 14, 7).scale = (1.3, 1.0, 0.5)
        self.ribbon(f"{name}_Stem", [(0.072, 0.02, 0), (0.072, 3.0 * STAFF_SPACING, 0)], 0.014, self.wisp_core,
                    parent=node)
        return node

    def pulse(self):
        return self.note("Pulse_0", LENGTH * 0.4, step=0)

    def bar_template(self):
        # Half a beat downstream of the downbeat note: between the measures.
        return self.bar("Bar_0", LENGTH * 0.4 + 0.5 * FLOW_SPEED * 60 / PREVIEW_BPM)

    def ports(self):
        radius = CHANNEL_HALF_WIDTH + 0.12
        for end, x in (("in", 0.0), ("out", LENGTH)):
            # +X along the flow; the arch lies in the port's YZ plane, spanning the stream.
            port = self.node(f"Port_{end}", (x, 0, WATER_Z), stream="mud", direction=end)
            halo = self.node(f"Port_{end}_Halo", (0, 0, 0), parent=port, role="port_highlight")
            for k, (tube, mat) in enumerate([(0.008, self.glow), (0.03, self.port_halo), (0.07, self.port_halo)]):
                self.arch(f"Port_{end}_Arch{k}", radius, tube, mat, halo)

    def reeds(self):
        """Reeds outline both banks, rising from the waterline up the inner slope. They
        stay clear of the ends so chained segments do not double up at the joins."""
        rng = random.Random(19)
        for side, sign in (("L", 1), ("R", -1)):
            node = self.node(f"Reeds_{side}", (0, 0, 0), role="outline", sway_max_deg=3.0)
            stalks, blades, heads = bmesh.new(), bmesh.new(), bmesh.new()
            x = rng.uniform(0.04, 0.12)
            while x < LENGTH - 0.06:
                for _ in range(rng.choice((1, 2, 2, 3))):   # little tufts, not a fence
                    px = min(max(x + rng.uniform(-0.05, 0.05), 0.03), LENGTH - 0.03)
                    py = sign * (CHANNEL_HALF_WIDTH + rng.uniform(0.02, 0.2))
                    base = Vector((px, py, max(self.bank_height(px, py), WATER_Z) - 0.03))
                    out = Vector((0, sign, 0))
                    height = rng.uniform(0.45, 1.15)
                    lean = out * rng.uniform(0.04, 0.2) + Vector((rng.uniform(-0.06, 0.06), 0, 0))
                    pts = ReedClump.path(base, lean, height, out * rng.uniform(0, 0.08), n=4)
                    r = rng.uniform(0.006, 0.01)
                    ReedClump.tube_into(stalks, pts, r, r * 0.25)
                    if rng.random() < 0.2:
                        head = bmesh.ops.create_uvsphere(heads, u_segments=6, v_segments=4, radius=1.0)
                        for v in head["verts"]:
                            v.co = Vector((v.co.x * r * 2.2, v.co.y * r * 2.2, v.co.z * height * 0.07)) + pts[-2]
                    if rng.random() < 0.35:
                        bh = rng.uniform(0.3, 0.7)
                        drift = Vector((rng.uniform(-0.25, 0.25), sign * rng.uniform(0.1, 0.3), 0))
                        bpts = ReedClump.path(base, drift, bh, drift * bh * 0.5 - Vector((0, 0, bh * 0.2)), n=4)
                        ReedClump.blade_into(blades, bpts, rng.uniform(0.02, 0.032))
                x += rng.uniform(0.12, 0.24)
            for name, bm, mat in (("Stalks", stalks, self.reed_stalk), ("Blades", blades, self.reed_blade),
                                  ("Heads", heads, self.reed_head)):
                mesh = bpy.data.meshes.new(f"Stream_Reed{name}_{side}")
                bm.to_mesh(mesh)
                bm.free()
                obj = self.link(bpy.data.objects.new(f"Stream_Reed{name}_{side}", mesh), mat, node)
                for face in obj.data.polygons:
                    face.use_smooth = True

    def set_clarity(self, stage):
        bsdf = self.water.node_tree.nodes["Principled BSDF"]
        bsdf.inputs["Base Color"].default_value = (*linear(stage["base_color"]), 1.0)
        bsdf.inputs["Roughness"].default_value = stage["roughness"]
        bsdf.inputs["Alpha"].default_value = stage["alpha"]
        self.staff_line.node_tree.nodes["Principled BSDF"].inputs["Emission Strength"].default_value = \
            stage["staff_strength"]
        g = stage["reed_greenness"]
        for mat, dry, green in ((self.reed_stalk, "stalk_dry", "stalk_green"),
                                (self.reed_blade, "blade_dry", "blade_green")):
            c = [a + (b - a) * g for a, b in zip(linear(REED_COLORS[dry]), linear(REED_COLORS[green]))]
            mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (*c, 1.0)
        for obj in bpy.data.objects["Staff"].children:
            if "StaffHalo" in obj.name:
                obj.hide_render = not stage["staff_halo"]
                obj.scale = (1, 1, 1) if stage["staff_halo"] else (0, 0, 0)   # exports the stage too

    def preview_notes(self, pulse):
        """Not exported: the notes one burbler at PREVIEW_BPM would have on this segment."""
        spacing = FLOW_SPEED * 60 / PREVIEW_BPM
        x, k = LENGTH * 0.4 - spacing, 1
        while x > 0.15:
            self.note(f"PreviewNote{k}", x, step=(k * 3) % 5 - 2)
            if k % BEATS_PER_BAR == 0:
                self.bar(f"PreviewBar{k}", x + 0.5 * spacing)
            x, k = x - spacing, k + 1
        x = LENGTH * 0.4 + spacing
        while x < LENGTH - 0.15:
            self.note(f"PreviewNote{k}", x, step=(k * 3) % 5 - 2)
            x, k = x + spacing, k + 1

    def profile(self):
        return {
            "id": self.asset_id,
            "model": f"{self.model_ref}#Scene0",
            "kind": "stream_segment",
            "length_m": LENGTH,
            "flow_axis": "+X",
            "width_m": 2 * CHANNEL_HALF_WIDTH,
            "size_metres": {"length": LENGTH, "width": 2 * (CHANNEL_HALF_WIDTH + BANK_WIDTH + BANK_SKIRT),
                            "height": BANK_HEIGHT},
            "tileable": True,
            "bendable": {"length_subdivisions": 24},
            "ports": [{"node": "Port_in", "direction": "in", "halo": "Port_in_Halo"},
                      {"node": "Port_out", "direction": "out", "halo": "Port_out_Halo"}],
            "staff": {"node": "Staff", "lines": STAFF_LINES, "spacing_m": STAFF_SPACING, "height_m": STAFF_Z,
                      "ribbon": "lines keep spacing_m apart along the centre line's normal; they never cross",
                      "wave": {"amplitude_m": STAFF_WAVE, "wavelength_m": STAFF_WAVELENGTH,
                               "centre_y": "amplitude * sin(2 pi x / wavelength - phase), x along the segment",
                               "phase": "2 pi * flow_speed_mps * t / wavelength_m: the wave travels downstream"},
                      "morph_targets": {"WaveSin": "cos(phase)", "WaveCos": "-sin(phase)"},
                      "clip": {"name": "Staff_Flow", "loops": True,
                               "seconds_at_profile_speed": STAFF_WAVELENGTH / FLOW_SPEED,
                               "speed": "play at flow_speed_mps / profile flow_speed_mps, or drive the weights"},
                      "material": "Stream_Staff", "halo_material": "Stream_StaffHalo"},
            "notes": {"node": "Pulse_0", "travel_axis": "+X",
                      "emit": "one note per clock tick at the stream's source, riding the staff downstream",
                      "flow_speed_mps": FLOW_SPEED,
                      "spacing": "flow_speed_mps * 60 / bpm metres between notes: velocity reads as spread",
                      "position": "the ribbon's centre line at the note's x, plus staff_step * spacing_m / 2 "
                                  "along its normal (0 = middle line); notes move with the wave, so each "
                                  "stays on its crest",
                      "yaw": "atan of the centre line's slope"},
            "bars": {"node": "Bar_0", "beats_per_bar": BEATS_PER_BAR,
                     "emit": "one bar line every beats_per_bar clock ticks, on the downbeat",
                     "offset": "half a note spacing downstream of the downbeat note (between measures)",
                     "spacing": "beats_per_bar * flow_speed_mps * 60 / bpm metres: measures follow the clock",
                     "position": "the ribbon's centre line at its x, square to it (yaw as notes)"},
            "clarity": {"material": "Stream_Water", "stages": CLARITY},
            "reeds": {"nodes": ["Reeds_L", "Reeds_R"], "built_from": {"item": "reed", "quantity": 6},
                      "greenness": "each stage's reed_greenness: dry while muddy, green when clear",
                      "colors": {k: REED_COLORS[k] for k in ("stalk_dry", "stalk_green", "blade_dry", "blade_green")},
                      "materials": {"Stream_ReedStalk": ["stalk_dry", "stalk_green"],
                                    "Stream_ReedBlade": ["blade_dry", "blade_green"]},
                      "sway_max_deg": 3.0},
            "lights": {"Stream_PortHalo": {"rest": 0.0, "peak": 1.0, "connected": 0.4,
                                           "note": "port highlighting while a stream is extended"}},
        }

    def build(self):
        self.channel()
        self.reeds()
        self.staff()
        pulse = self.pulse()
        self.bar_template()
        self.ports()
        self.set_clarity(CLARITY[0])  # ships muddy

        def stage_preview():
            self.stage_preview(resolution=(640, 860))
            self.preview_notes(pulse)
        report = self.finish(self.profile(), aim=(LENGTH * 0.55, 0, 0.0), camera_location=(-2.9, -2.3, 3.0),
                             before_render=stage_preview)
        if bpy.app.background:  # also render each clarity stage, then tile them
            out = PROJECT / "assets/streams/stream_segment"
            tiles = []
            for stage in CLARITY:
                self.set_clarity(stage)
                self.scene.render.filepath = str(out / f"preview_{stage['name']}.png")
                bpy.ops.render.render(write_still=True, scene=self.scene.name)
                tiles.append(out / f"preview_{stage['name']}.png")
            magick = shutil.which("magick")
            if magick:
                cmd = [magick, "montage"]
                for tile, stage in zip(tiles, CLARITY):
                    cmd += ["-label", stage["name"], str(tile)]
                cmd += ["-tile", f"{len(tiles)}x1", "-geometry", "+6+6", "-background", "#15161a",
                        "-fill", "#d8d4cc", "-pointsize", "20", str(out / "preview_stages.png")]
                subprocess.run(cmd, check=True)
        return report


if __name__ == "__main__":
    print(Stream().build())
