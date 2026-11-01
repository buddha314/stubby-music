"""The burbler's configuration HUD modules (add-mudpit-burbler 1.3, 1.4).

Configuring a mudpit burbler opens a basic three-place Eurorack: the burbler's own
clock, a VCO slot and a volume module. The VCOs already exist; these are the other two.

- `burbler_clock` (8HP, engine `Clock`): the burbler's beat. A BPM knob sets `bpm`
  (it starts at the burbler's own seeded tempo), a BPM display reads it, a
  jellyfish-bell lamp flashes on the beat, and GATE and PHASE are its outputs. Its
  crown is a row of bubbles.
- `volume` (4HP, engine `Vca` with a built-in envelope): one large knob setting `gain`,
  TRIG, IN and OUT jacks, and a soft ring of light around the knob. A trigger at TRIG
  fires the envelope that opens it; the player patches the clock's GATE there. Its
  crown is a rising wedge.

The player patches the voice with cables (`create_patch_cable.py`): clock GATE to
volume TRIG, VCO OUT to volume IN. Volume OUT is normalled to the burbler's outlet.

Both are drawn in the style guide's newer language: crude meets high-tech (a weathered
wood case and rusted iron hardware behind a titanium faceplate, outputs on graphene),
and light like jellyfish and fairies (a bright core inside translucent shells), not
LEDs. The BPM display stays segmented, because a number has to be read.

Reproduce: blender -b --factory-startup -P tools/blender/create_hud_modules.py [-- burbler_clock volume]

Game-facing nodes follow module_kit.py: `Port_<port>`, `Knob_<param>`, `Light_<light>`,
`Lamp`, `Display` / `Display_d<digit>_<segment>`.
"""
from pathlib import Path
import math
import sys

import bpy

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_clock_lamp import DIGITS, FLASH, SEGMENTS, segment_boxes  # noqa: E402
from module_kit import HEIGHT, PANEL_FRONT, ModuleKit, run  # noqa: E402

CROWN_HEIGHT = 0.09
PREVIEW_BPM = 90  # the burbler's default tempo


def linear(hex_srgb):
    """Style-guide hex (sRGB) to the linear colour Blender materials take."""
    c = [int(hex_srgb[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    return tuple(v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4 for v in c)


AMBER, FLASH_COLOUR = linear("#ffb350"), linear("#ffce89")


class HudKit(ModuleKit):
    """ModuleKit in the crude-meets-high-tech materials, with soft lights."""

    def __init__(self, kind, asset_id, hp, module_kind):
        super().__init__(kind, asset_id, hp=hp)
        self.module_kind = module_kind  # the engine module, not the name prefix
        self.root["module_kind"] = module_kind
        m = self.material
        self.body_mat = m("Wood", linear("#5a4a3a"), roughness=0.9)
        self.panel_mat = m("Titanium", linear("#b8bcc2"), metallic=1.0, roughness=0.32)
        self.output_mat = m("Graphene", linear("#1a1c1f"), metallic=0.7, roughness=0.12)
        self.screw_mat = m("Iron", linear("#5e3a22"), metallic=0.5, roughness=0.85)
        self.brass_mat = self.screw_mat  # jack nuts in the same rusted iron
        self.glow = m("Glow", AMBER, roughness=0.3, emission=AMBER, strength=8.0)
        self.glow_halo = self.translucent("GlowHalo", AMBER, 1.2, 0.12)

    def crown(self, edge, mat):
        """A silhouette along the top edge, traced in light, as every module has."""
        w = self.width
        outline = edge + [(w / 2, HEIGHT), (-w / 2, HEIGHT)]
        self.prism("Crown", outline, self.body_mat)
        self.trace("CrownTrace", edge, "xz", PANEL_FRONT - 0.002, 0.012, mat)

    def side_glyphs(self, points, mat):
        side_x = (self.width - 0.01) / 2 + 0.002
        for name, x in (("L", -side_x), ("R", side_x)):
            pts = points if name == "R" else [(-u, z) for u, z in points]
            self.trace(f"SideGlyph{name}", pts, "yz", x, 0.024, mat)

    def profile(self, extra, **fields):
        base = self.base_profile(extra_height=CROWN_HEIGHT) | {"module_kind": self.module_kind}
        return {k: base[k] for k in ("id", "model", "module_kind", "scale", "size_metres")} | fields | {
            "placement": base["placement"], **extra}


def bubble_edge(w, bubbles, samples=72):
    """The top edge with semicircular bubbles rising out of it: [(centre_x, radius)]."""
    pts = []
    for i in range(samples + 1):
        x = -w / 2 + w * i / samples
        rise = max([math.sqrt(max(r * r - (x - c) ** 2, 0.0)) for c, r in bubbles] + [0.0])
        pts.append((x, HEIGHT + rise))
    return pts


def bubble_glyph(cx, cz, scale):
    """Three bubbles as one open stroke: small, large, small arcs along a baseline."""
    pts = [(cx - 1.0 * scale, cz)]
    for c, r in ((-0.6, 0.3), (0.05, 0.42), (0.68, 0.22)):
        for k in range(13):
            a = math.pi - math.pi * k / 12
            pts.append((cx + (c + r * math.cos(a)) * scale, cz + r * math.sin(a) * scale))
    pts.append((cx + 1.0 * scale, cz))
    return pts


def build_burbler_clock():
    kit = HudKit("BurblerClock", "burbler_clock", hp=8, module_kind="Clock")
    w = kit.width
    # The lamp material pulses on the beat. It is translucent so the bell, core and
    # halo can share it and all flash together (the game drives it by name).
    lamp_mat = kit.translucent("Lamp", FLASH_COLOUR, FLASH["peak_strength"], 0.55)
    segment_mat = kit.material("Segment", (0.3, 0.12, 0.03), roughness=0.3, emission=AMBER, strength=6.0)
    # Unlit segments: dark and matte, so they never catch a light and read as lit.
    ghost_mat = kit.material("SegmentGhost", (0.018, 0.012, 0.01), roughness=0.95)
    window_mat = kit.material("DisplayWindow", (0.01, 0.009, 0.008), roughness=0.08)

    kit.case()
    kit.crown(bubble_edge(w, [(-0.12, 0.045), (0.015, 0.075), (0.135, 0.035)]), lamp_mat)
    kit.side_glyphs(bubble_glyph(0.0, 1.0, 0.11), lamp_mat)
    kit.title("BURBLER")

    # Tempo: the display reads the BPM knob below it.
    display_z, digit_w, digit_h, stroke, gap = 1.09, 0.08, 0.13, 0.014, 0.025
    kit.box("DisplayWindow", (0.33, 0.004, 0.19), (0, PANEL_FRONT - 0.002, display_z), window_mat)
    display = bpy.data.objects.new("Display", None)
    kit.link(display)
    display["display"] = "bpm"
    display["digits"] = 3
    display["segments"] = SEGMENTS
    lit = {}
    for digit in range(3):
        cx = (digit - 1) * (digit_w + gap)
        for seg, (sx, sz, x, z) in segment_boxes(cx, display_z, digit_w, digit_h, stroke).items():
            kit.box(f"SegmentGhost_{digit}{seg}", (sx, 0.002, sz), (x, PANEL_FRONT - 0.0045, z), ghost_mat)
            obj = kit.box(f"Display_d{digit}_{seg}", (sx, 0.002, sz), (0, 0, 0), segment_mat)
            obj.name = f"Display_d{digit}_{seg}"
            obj.parent = display
            obj.location = (x, PANEL_FRONT - 0.0055, z)
            lit[(digit, seg)] = obj

    kit.knob("bpm", 0.84, "BPM", param_min=30.0, param_max=240.0, taper="linear", rotation_degrees=300.0,
             default="burbler")

    # The beat lamp: a jellyfish bell on the faceplate, its tendrils trailing below.
    kit.jellyfish("Lamp", (0, 0.58), lamp_mat, light="gate", pulse_driver="audio_sample_timestamp", **FLASH)

    kit.output_patch(0.27, 0.30)
    kit.jack("gate", "out", 0.34, "GATE", on_patch=True)
    kit.jack("phase", "out", 0.19, "PHASE", on_patch=True)
    kit.soft_light("gate", (0.13, PANEL_FRONT - 0.004, 0.34))
    kit.soft_light("phase", (0.13, PANEL_FRONT - 0.004, 0.19))

    def show_preview_bpm():
        shown = f"{PREVIEW_BPM:3d}"
        for (digit, seg), obj in lit.items():
            obj.hide_render = not (shown[digit] != " " and seg in DIGITS[shown[digit]])
        # Preview mid-flash: at peak the bell's shape is lost in its own glow. The
        # exported material stays authored at peak.
        lamp_mat.node_tree.nodes["Principled BSDF"].inputs["Emission Strength"].default_value = 4.0

    profile = kit.profile({
        "silhouette": "bubble_crown",
        "ports": [{"node": "Port_gate", "port": "gate", "direction": "out"},
                  {"node": "Port_phase", "port": "phase", "direction": "out"}],
        "params": [{"node": "Knob_bpm", "param": "bpm", "min": 30.0, "max": 240.0, "taper": "linear",
                    "rotation_degrees": 300.0, "default": "the burbler's seeded tempo"}],
        "tempo": {"param": "bpm", "source": "Knob_bpm", "editable": True,
                  "note": "starts at the burbler instance's seeded tempo; the display shows the knob's value"},
        "lights": [{"node": "Light_gate", "light": "gate", "rest_strength": 0.0, "peak_strength": 8.0},
                   {"node": "Light_phase", "light": "phase", "rest_strength": 0.0, "peak_strength": 8.0}],
        "display": {"node": "Display", "shows": "bpm", "digits": 3, "segments": SEGMENTS,
                    "segment_node": "Display_d{digit}_{segment}", "digit_segments": DIGITS},
        "lamp": {"node": "Lamp", "light": "gate", "material": "BurblerClock_Lamp",
                 "pulse_driver": "audio_sample_timestamp", "colour_linear": list(FLASH_COLOUR), **FLASH},
        "hud": {"place": "clock", "fixed": True},
    })
    return kit.finish(profile, aim=(0, 0, 0.7), camera_location=(1.05, -2.15, 1.3),
                      before_render=show_preview_bpm)


def build_volume():
    kit = HudKit("Volume", "volume", hp=4, module_kind="Vca")
    w = kit.width
    kit.case()
    wedge = [(-w / 2, HEIGHT + 0.012), (w / 2, HEIGHT + CROWN_HEIGHT)]
    kit.crown(wedge, kit.glow)
    kit.side_glyphs([(-0.09, 0.95), (0.09, 0.95), (0.09, 1.07), (-0.09, 0.95)], kit.glow)
    kit.title("VOL")

    # Faceplate glyph: the same rising wedge, on a dark recess.
    kit.box("GlyphPlate", (0.16, 0.002, 0.13), (0, PANEL_FRONT - 0.001, 1.07), kit.output_mat)
    kit.trace("Glyph", [(-0.06, 1.03), (0.06, 1.03), (0.06, 1.11), (-0.06, 1.03)], "xz",
              PANEL_FRONT - 0.0035, 0.016, kit.glow)

    knob_z = 0.78
    kit.knob("gain", knob_z, "LEVEL", param_min=0.0, param_max=1.0, taper="linear", rotation_degrees=300.0)
    # A soft ring of light around the knob, following the level actually passing through.
    ring = kit.socket("Light_level", (0, PANEL_FRONT - 0.003, knob_z), light="level", rest_strength=0.0,
                      peak_strength=6.0)
    kit.ring("LevelRing", 0.093, 0.004, kit.glow, ring)
    kit.ring("LevelRingHalo", 0.093, 0.012, kit.glow_halo, ring)
    kit.ring("LevelRingHaloOuter", 0.093, 0.017, kit.glow_halo, ring)

    kit.jack("trig", "in", 0.5, "TRIG")
    kit.jack("in", "in", 0.36, "IN")
    kit.output_patch(0.2, 0.2)
    kit.jack("out", "out", 0.2, "OUT", on_patch=True)

    profile = kit.profile({
        "silhouette": "rising_wedge",
        "ports": [{"node": "Port_trig", "port": "trig", "direction": "in",
                   "note": "fires the built-in envelope that opens the VCA"},
                  {"node": "Port_in", "port": "in", "direction": "in"},
                  {"node": "Port_out", "port": "out", "direction": "out",
                   "normalled_to": "the burbler's outlet when nothing is patched"}],
        "envelope": {"built_in": True, "trigger": "trig", "note": "fixed shape; engine Envelope -> Vca cv"},
        "params": [{"node": "Knob_gain", "param": "gain", "min": 0.0, "max": 1.0, "taper": "linear",
                    "rotation_degrees": 300.0}],
        "lights": [{"node": "Light_level", "light": "level", "rest_strength": 0.0, "peak_strength": 6.0}],
        "hud": {"place": "volume", "fixed": False},
    })
    return kit.finish(profile, aim=(0, 0, 0.7), camera_location=(0.75, -2.0, 1.3))


if __name__ == "__main__":
    print(run({"burbler_clock": build_burbler_clock, "volume": build_volume}, __file__))
