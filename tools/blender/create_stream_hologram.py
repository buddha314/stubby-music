"""Stream hologram: the planned-stream preview shown while the player extends a stream
(add-mudpit-burbler 1.9).

Like a Satisfactory conveyor before it is built, the planned stream is drawn as a
luminous ghost of the real thing. This is the ghost of one `stream_segment`: the same
4 m length, the same banked channel and the same 24 subdivisions along +X, so the game
bends and chains it exactly as it will lay the segment, and the hologram sits where
the stream will be. The origin is the inlet's centre on the ground.

Nothing here is solid. Faint shells trace the banks and bed, a brighter sheet marks
the water line, thin lines run along the crests and water edges, and cross-section ribs
every half metre give the plan its readable, gridded hologram look. Chevrons on the
water point downstream; the game scrolls them to show which way and how fast the
stream will flow.

  Port_in, Port_out    same as stream_segment: stream = mud, +X along the flow
  Port_<end>_Ring      ghost arch over each end, where the plan snaps to a port
  Flow                 extras: scroll_axis, spacing_m. Parent of the chevrons; the game
                       slides it along +X and wraps it every `spacing_m`
  StreamHologram_*     all hologram materials; the game tints them by the `states` in
                       the profile (valid, invalid)

Reproduce: blender -b --factory-startup -P tools/blender/create_stream_hologram.py
Writes assets/streams/stream_hologram/ (profile and previews: valid, invalid, and the
hologram extending a laid stream).
"""
from pathlib import Path
import shutil
import subprocess
import sys

import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_burbler import (  # noqa: E402
    BANK_SKIRT, BANK_WIDTH, CHANNEL_HALF_WIDTH, Burbler, linear,
)
from create_stream import BED_Z, LENGTH, WATER_Z, Stream  # noqa: E402
from module_kit import PROJECT  # noqa: E402

NX = 24                  # the segment's length subdivisions, so both bend alike
RIB_SPACING = 0.5        # cross-section ribs along the plan
CHEVRON_SPACING = 0.5    # chevrons along the water; the game wraps the scroll at this
LIFT = 0.012             # lines and ribs ride just above the ghosted surfaces
# Build-mode colours (sRGB). Valid is a pale amber, so the plan belongs with the port
# arch it snaps to; invalid is a warm red. The game tints every hologram material.
STATES = {
    "valid": {"color": "#ffd39a", "note": "the plan can be laid"},
    "invalid": {"color": "#ff5a46", "note": "blocked, too steep, or unaffordable"},
}
# Per material: emission strength and alpha. The game keeps these ratios when it tints.
LAYERS = {
    "Line": (6.0, 1.0),
    "Rib": (2.2, 0.55),
    "Chevron": (4.0, 0.8),
    "Surface": (0.5, 0.09),
    "Shell": (0.2, 0.03),
}


class StreamHologram(Stream):
    def __init__(self):
        Burbler.__init__(self, "stream_hologram", kind="StreamHologram")
        for key in ("mine", "produces"):
            del self.root[key]
        self.root["kind"] = "stream_hologram"
        self.root["length_m"] = LENGTH
        self.root["previews"] = "stream_segment"
        self.layers = {name: self.ghost(name, STATES["valid"]["color"], *LAYERS[name]) for name in LAYERS}

    def ghost(self, name, color, strength, alpha):
        mat = self.translucent(name, linear(color), strength, alpha)
        mat.use_backface_culling = False
        return mat

    def tint(self, color):
        for name, mat in self.layers.items():
            bsdf = mat.node_tree.nodes["Principled BSDF"]
            bsdf.inputs["Base Color"].default_value = (*linear(color), 1.0)
            bsdf.inputs["Emission Color"].default_value = (*linear(color), 1.0)

    def surface_z(self, x, y):
        """The laid segment's surface: bank where there is bank, otherwise the bed."""
        return max(self.bank_height(x, y), BED_Z)

    def shells(self):
        outer = CHANNEL_HALF_WIDTH + BANK_WIDTH + BANK_SKIRT
        shell = self.layers["Shell"]
        for side, (y0, y1) in (("L", (CHANNEL_HALF_WIDTH - 0.05, outer)), ("R", (-outer, -CHANNEL_HALF_WIDTH + 0.05))):
            self.grid(f"Bank{side}", shell, y0, y1, 14, self.bank_height, nx=NX)
        w = CHANNEL_HALF_WIDTH + 0.08
        self.grid("Bed", shell, -w, w, 8, lambda x, y: BED_Z, nx=NX)
        self.grid("Surface", self.layers["Surface"], -CHANNEL_HALF_WIDTH, CHANNEL_HALF_WIDTH, 4,
                  lambda x, y: WATER_Z, nx=NX)

    def lines(self):
        """Long lines along the water edges and the bank crests."""
        line = self.layers["Line"]
        xs = [LENGTH * i / NX for i in range(NX + 1)]
        for side, sign in (("L", 1), ("R", -1)):
            edge = sign * CHANNEL_HALF_WIDTH
            crest = sign * (CHANNEL_HALF_WIDTH + BANK_WIDTH)
            self.ribbon(f"Edge{side}", [(x, edge, WATER_Z + 0.004) for x in xs], 0.012, line)
            self.ribbon(f"Crest{side}", [(x, crest, self.bank_height(x, crest) + LIFT) for x in xs], 0.012, line)

    def ribs(self):
        """Cross-sections every half metre, draped over banks and bed."""
        outer = CHANNEL_HALF_WIDTH + BANK_WIDTH + BANK_SKIRT
        n = int(round(LENGTH / RIB_SPACING))
        for k in range(n + 1):
            x = min(max(k * RIB_SPACING, 0.003), LENGTH - 0.003)
            ys = [-outer + 2 * outer * j / 40 for j in range(41)]
            self.ribbon(f"Rib{k}", [(x, y, self.surface_z(x, y) + LIFT) for y in ys], 0.01, self.layers["Rib"])

    def chevrons(self):
        flow = self.node("Flow", (0, 0, WATER_Z + 0.006), scroll_axis="+X", spacing_m=CHEVRON_SPACING,
                         flow_speed_from="stream_segment pulse.flow_speed_mps until the fluid sim supplies it")
        arm = CHANNEL_HALF_WIDTH * 0.42
        n = int(round(LENGTH / CHEVRON_SPACING))
        for k in range(n):
            x = (k + 0.5) * CHEVRON_SPACING
            tip, back = (x + 0.09, 0, 0), x - 0.09
            self.ribbon(f"Flow_Chevron{k}", [(back, arm, 0), tip, (back, -arm, 0)], 0.035,
                        self.layers["Chevron"], parent=flow)
        return flow

    def rings(self):
        radius = CHANNEL_HALF_WIDTH + 0.12
        for end, x in (("in", 0.0), ("out", LENGTH)):
            port = self.node(f"Port_{end}", (x, 0, WATER_Z), stream="mud", direction=end)
            ring = self.node(f"Port_{end}_Ring", (0, 0, 0), parent=port, role="snap_ghost")
            self.arch(f"Port_{end}_Ring0", radius, 0.008, self.layers["Rib"], ring)
            self.arch(f"Port_{end}_Ring1", radius, 0.035, self.layers["Shell"], ring)

    def profile(self):
        return {
            "id": self.asset_id,
            "model": f"models/{self.asset_id}.glb#Scene0",
            "kind": "stream_hologram",
            "previews": "stream_segment",
            "length_m": LENGTH,
            "flow_axis": "+X",
            "width_m": 2 * CHANNEL_HALF_WIDTH,
            "tileable": True,
            "bendable": {"length_subdivisions": NX},
            "ports": [{"node": "Port_in", "direction": "in", "ring": "Port_in_Ring"},
                      {"node": "Port_out", "direction": "out", "ring": "Port_out_Ring"}],
            "flow": {"node": "Flow", "scroll_axis": "+X", "spacing_m": CHEVRON_SPACING,
                     "speed": "the planned stream's flow speed; stream_segment's flow_speed_mps for now"},
            "states": STATES,
            "materials": {f"StreamHologram_{name}": {"emission_strength": s, "alpha": a}
                          for name, (s, a) in LAYERS.items()},
            "shimmer": {"note": "optional: a slow alpha flicker, 0.85 to 1.0 of each material's alpha"},
        }

    def stage_laid_stream(self):
        """Preview only: a laid, muddy stream_segment ending where the plan begins."""
        before = set(bpy.data.objects)
        with bpy.context.temp_override(scene=self.scene, view_layer=self.scene.view_layers[0]):
            bpy.ops.import_scene.gltf(filepath=str(PROJECT / "assets/models/stream_segment.glb"))
        for obj in set(bpy.data.objects) - before:
            if obj.name not in self.scene.objects:
                self.scene.collection.objects.link(obj)
            if obj.parent is None:
                obj.location += Vector((-LENGTH, 0, 0))
            if any(part in obj.name for part in ("Halo", "Pulse", "Staff")):
                obj.hide_render = True

    def build(self):
        self.shells()
        self.lines()
        self.ribs()
        self.chevrons()
        self.rings()

        def stage():
            self.stage_preview(resolution=(640, 860))
        aim, cam = (LENGTH * 0.55, 0, 0.0), (-2.9, -2.3, 3.0)
        report = self.finish(self.profile(), aim=aim, camera_location=cam, before_render=stage,
                             profile_dir="streams")
        if bpy.app.background:
            out = PROJECT / "assets/streams/stream_hologram"
            tiles = []
            for name, state in STATES.items():
                self.tint(state["color"])
                self.scene.render.filepath = str(out / f"preview_{name}.png")
                bpy.ops.render.render(write_still=True, scene=self.scene.name)
                tiles.append((name, out / f"preview_{name}.png"))
            # In context: the plan extending off the end of a laid stream.
            self.tint(STATES["valid"]["color"])
            self.stage_laid_stream()
            camera = self.scene.camera
            aim = Vector((0.4, 0, 0.0))
            camera.location = (-4.6, -4.4, 3.6)
            camera.rotation_euler = (aim - camera.location).to_track_quat("-Z", "Y").to_euler()
            self.scene.render.resolution_x, self.scene.render.resolution_y = 1280, 720
            self.scene.render.filepath = str(out / "preview_extending.png")
            bpy.ops.render.render(write_still=True, scene=self.scene.name)
            magick = shutil.which("magick")
            if magick:
                cmd = [magick, "montage"]
                for name, tile in tiles:
                    cmd += ["-label", name, str(tile)]
                cmd += ["-tile", f"{len(tiles)}x1", "-geometry", "+6+6", "-background", "#15161a",
                        "-fill", "#d8d4cc", "-pointsize", "20", str(out / "preview_states.png")]
                subprocess.run(cmd, check=True)
        return report


if __name__ == "__main__":
    print(StreamHologram().build())
