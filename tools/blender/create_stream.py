"""Stream segment: a straight, tileable length of the stream a burbler feeds
(add-mudpit-burbler 1.7).

Streams carry the music: they pulse at the burbler's BPM, so a stream is the played
sequence laid out in space. This is one 4 m length of the burbler's own channel (its
bank profile and mud height are imported, so a segment meets the burbler's mouth
exactly): 1 m of water between banked sides, built up from the ground. Flow runs
along +X; the origin is the inlet's centre on the ground, so segments chain port to
port. It is subdivided along its length so the game can bend it along a path.

The water goes from muddy to clear as cleaning processes are identified. One water
surface sits over a pebbled bed; the profile's `clarity` table gives its look at each
stage, for the game to blend. Five faint luminous staff lines lie on the bed: hidden
while the water is muddy, revealed as it clears, so a clear stream is staff music.

  Port_in, Port_out    extras: stream = mud. +X along the flow
  Port_<end>_Halo      upright arch over each end: port highlighting while building
  Pulse_0              extras: travel_axis, length_m. The game slides it downstream on
                       each beat: a band of light across the surface while muddy
                       (`Pulse_0_Band`), a note head on the staff when clear
                       (`Pulse_0_Note`)
  Staff                the five staff lines, faded in with clarity

Reproduce: blender -b --factory-startup -P tools/blender/create_stream.py
Writes assets/streams/stream_segment/ (profile and previews: muddy, silty, clear).
"""
from pathlib import Path
import math
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

LENGTH = 4.0
WATER_Z = STREAM_MUD_Z
BED_Z = 0.006
STAFF_LINES = 5
STAFF_SPACING = 0.15
# The water's look at each stage of clarity (sRGB, for the game to blend between).
# Muddy is 0.98, not 1, so the material exports alpha-blended and can fade to clear.
CLARITY = [
    {"at": 0.0, "name": "muddy", "base_color": "#2a1d13", "roughness": 0.5, "alpha": 0.98},
    {"at": 0.5, "name": "silty", "base_color": "#5a4630", "roughness": 0.22, "alpha": 0.72},
    {"at": 1.0, "name": "clear", "base_color": "#6f8a88", "roughness": 0.04, "alpha": 0.22},
]
STAFF_VISIBLE_FROM = 0.6   # clarity at which the staff starts to show through
NOTE_FROM = 0.5            # clarity at which the pulse turns from a band into a note


class Stream(Burbler):
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

    def staff(self):
        """Five lines on the bed, seen only through clear water: a clear stream is staff music."""
        node = self.node("Staff", (0, 0, 0), visible_from_clarity=STAFF_VISIBLE_FROM)
        for k in range(STAFF_LINES):
            y = (k - (STAFF_LINES - 1) / 2) * STAFF_SPACING
            pts = [(LENGTH * i / 24, y, BED_Z + 0.012) for i in range(25)]
            self.ribbon(f"StaffLine{k}", pts, 0.008, self.glow, parent=node)
            self.ribbon(f"StaffHalo{k}", [(x, yy, z + 0.002) for x, yy, z in pts], 0.04, self.halo, parent=node)

    def pulse(self):
        node = self.node("Pulse_0", (LENGTH * 0.4, 0, 0), travel_axis="+X", length_m=LENGTH,
                         note_from_clarity=NOTE_FROM)
        # Muddy: a soft band of light across the surface.
        band = self.node("Pulse_0_Band", (0, 0, WATER_Z + 0.006), parent=node)
        for k, width in enumerate((0.06, 0.16, 0.3)):
            self.ribbon(f"Pulse_0_Band{k}", [(0, -CHANNEL_HALF_WIDTH + 0.04, 0), (0, CHANNEL_HALF_WIDTH - 0.04, 0)],
                        width, self.glow if k == 0 else self.halo, parent=band)
        # Clear: a note head riding the middle staff line.
        note = self.node("Pulse_0_Note", (0, 0, BED_Z + 0.03), parent=node)
        head = self.sphere("Pulse_0_NoteHead", 0.045, self.wisp_core, (0, 0, 0), note, 14, 7)
        head.scale = (1.4, 1.0, 0.7)
        head.rotation_euler.y = -0.35
        self.sphere("Pulse_0_NoteHalo", 0.09, self.halo, (0, 0, 0), note, 14, 7)
        return band, note

    def ports(self):
        radius = CHANNEL_HALF_WIDTH + 0.12
        for end, x in (("in", 0.0), ("out", LENGTH)):
            # +X along the flow; the arch lies in the port's YZ plane, spanning the stream.
            port = self.node(f"Port_{end}", (x, 0, WATER_Z), stream="mud", direction=end)
            halo = self.node(f"Port_{end}_Halo", (0, 0, 0), parent=port, role="port_highlight")
            for k, (tube, mat) in enumerate([(0.008, self.glow), (0.03, self.port_halo), (0.07, self.port_halo)]):
                self.arch(f"Port_{end}_Arch{k}", radius, tube, mat, halo)

    def set_clarity(self, stage, band, note):
        bsdf = self.water.node_tree.nodes["Principled BSDF"]
        bsdf.inputs["Base Color"].default_value = (*linear(stage["base_color"]), 1.0)
        bsdf.inputs["Roughness"].default_value = stage["roughness"]
        bsdf.inputs["Alpha"].default_value = stage["alpha"]
        for obj in bpy.data.objects["Staff"].children:
            obj.hide_render = stage["at"] < STAFF_VISIBLE_FROM
        for obj in band.children:
            obj.hide_render = stage["at"] >= NOTE_FROM
        for obj in note.children:
            obj.hide_render = stage["at"] < NOTE_FROM

    def profile(self):
        return {
            "id": self.asset_id,
            "model": f"models/{self.asset_id}.glb#Scene0",
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
            "pulse": {"node": "Pulse_0", "travel_axis": "+X",
                      "emit": "one per clock tick at the stream's source",
                      "flow_speed_mps": 1.5, "band": "Pulse_0_Band", "note": "Pulse_0_Note",
                      "note_from_clarity": NOTE_FROM},
            "clarity": {"material": "Stream_Water", "stages": CLARITY,
                        "staff": {"node": "Staff", "visible_from": STAFF_VISIBLE_FROM}},
            "lights": {"Stream_PortHalo": {"rest": 0.0, "peak": 1.0, "connected": 0.4,
                                           "note": "port highlighting while a stream is extended"}},
        }

    def build(self):
        self.channel()
        self.staff()
        band, note = self.pulse()
        self.ports()
        self.set_clarity(CLARITY[0], band, note)  # ships muddy

        def stage_preview():
            self.stage_preview(resolution=(640, 860))
        report = self.finish(self.profile(), aim=(LENGTH * 0.55, 0, 0.0), camera_location=(-2.9, -2.3, 3.0),
                             before_render=stage_preview, profile_dir="streams")
        if bpy.app.background:  # also render each clarity stage, then tile them
            out = PROJECT / "assets/streams/stream_segment"
            tiles = []
            for stage in CLARITY:
                self.set_clarity(stage, band, note)
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
