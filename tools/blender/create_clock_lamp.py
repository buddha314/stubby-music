"""The Clock, the opening's second discovery: a module crowned by a caged beacon.

The clock manifests as a pulsating light (desert-opening spec), so instead of a
waveform crown it carries a glass beacon that flashes on every tick. That gives it a
silhouette nothing else has, and a large emissive volume that can pulse visibly
through haze at about 100 m. The faceplate follows the VCO family: a narrow-pulse
"gate" glyph (deliberately unlike the square's 50% wave), a BPM knob, and GATE and
PHASE outputs. The engine's Clock has no inputs.

Reproduce: blender -b --factory-startup -P tools/blender/create_clock_lamp.py

Game-facing nodes (see module_kit.py): `Port_gate`, `Port_phase`, `Knob_bpm`,
`Light_gate` (LED), and `Lamp`, the beacon, whose extras describe the flash the game
drives from the clock's sample-stamped ticks, like the lightning bug's.
"""
from pathlib import Path
import math
import sys

import bmesh
import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from module_kit import AMBER, HEIGHT, PANEL_FRONT, ModuleKit, fit, run  # noqa: E402

COLLAR_HEIGHT = 0.04
DOME_RADIUS = 0.16
DOME_STRETCH = 1.1  # slightly taller than round, like a lantern globe
CAGE_RADIUS = 0.178
CAGE_HEIGHT = 0.34
CAP_HEIGHT = 0.02
LAMP_WARM = (1.0, 0.72, 0.42)
FLASH = {  # matches the lightning bug's flash envelope, so pulses read as one family
    "attack_seconds": 0.015,
    "flash_seconds": 0.25,
    "rest_strength": 0.15,
    "peak_strength": 30.0,
    "light_lumens": 6000.0,
    "light_range_metres": 40.0,
}


def gate_pulses(cycles):
    """Narrow centred pulses, like a clock's tick train."""
    pts = [(0.0, -1.0)]
    for c in range(cycles):
        pts += [(c + 0.375, -1.0), (c + 0.375, 1.0), (c + 0.625, 1.0), (c + 0.625, -1.0), (c + 1.0, -1.0)]
    return pts


def build():
    kit = ModuleKit("Clock", "clock_lamp")
    glyph_mat = kit.material("Glyph", (0.2, 0.08, 0.02), roughness=0.4, emission=AMBER, strength=4.0)
    led_mat = kit.material("LedGate", (0.25, 0.08, 0.01), roughness=0.3, emission=AMBER, strength=8.0)
    # Previewed at the peak of a flash; the game drives it between rest and peak.
    lamp_mat = kit.material("Lamp", (0.9, 0.75, 0.55), roughness=0.15, emission=LAMP_WARM,
                            strength=FLASH["peak_strength"])
    cage_mat = kit.material("Cage", (0.03, 0.03, 0.032), metallic=0.9, roughness=0.5)

    kit.case()

    # The beacon: collar on the case top, glass globe, and a cage of bars and rings.
    collar_top = HEIGHT + COLLAR_HEIGHT
    kit.cylinder("Collar", 0.13, COLLAR_HEIGHT, (0, -0.01, HEIGHT + COLLAR_HEIGHT / 2), cage_mat,
                 segments=32, facing_out=False)
    dome_centre = Vector((0.0, -0.01, collar_top + DOME_RADIUS * DOME_STRETCH))
    lamp = bpy.data.objects.new("Lamp", None)
    lamp.empty_display_type = "SPHERE"
    lamp.empty_display_size = DOME_RADIUS
    kit.link(lamp)
    lamp.location = dome_centre
    lamp["light"] = "gate"
    lamp["pulse_driver"] = "audio_sample_timestamp"
    for key, value in FLASH.items():
        lamp[key] = value

    def globe(bm):
        bmesh.ops.create_uvsphere(bm, u_segments=32, v_segments=16, radius=DOME_RADIUS)
        for v in bm.verts:
            v.co.z *= DOME_STRETCH
    dome = kit.link(bpy.data.objects.new("Clock_Globe", kit.mesh_from("Globe", globe)), lamp_mat, parent=lamp)
    for face in dome.data.polygons:
        face.use_smooth = True
    dome["light"] = "gate"

    def cage(bm):
        def bar(a, b, thickness):
            a, b = Vector(a), Vector(b)
            along = (b - a).normalized()
            side = along.cross(Vector((0, 0, 1)) if abs(along.z) < 0.9 else Vector((1, 0, 0))).normalized()
            up = along.cross(side).normalized()
            corners = [p + side * (s * thickness / 2) + up * (t * thickness / 2)
                       for p in (a, b) for s in (-1, 1) for t in (-1, 1)]
            v = [bm.verts.new(c) for c in corners]
            for face in ((0, 1, 3, 2), (4, 6, 7, 5), (0, 2, 6, 4), (1, 5, 7, 3), (0, 4, 5, 1), (2, 3, 7, 6)):
                bm.faces.new([v[i] for i in face])

        cx, cy = dome_centre.x, dome_centre.y
        top = collar_top + CAGE_HEIGHT
        for i in range(6):
            angle = math.tau * i / 6 + math.pi / 6
            x, y = cx + CAGE_RADIUS * math.cos(angle), cy + CAGE_RADIUS * math.sin(angle)
            bar((x, y, collar_top), (x, y, top), 0.014)
        for z in (collar_top + 0.01, collar_top + CAGE_HEIGHT * 0.55, top):
            ring = [(cx + CAGE_RADIUS * math.cos(math.tau * k / 24),
                     cy + CAGE_RADIUS * math.sin(math.tau * k / 24), z) for k in range(25)]
            for a, b in zip(ring, ring[1:]):
                bar(a, b, 0.012)
    kit.link(bpy.data.objects.new("Clock_Cage", kit.mesh_from("Cage", cage)), cage_mat)
    kit.cylinder("Cap", 0.11, CAP_HEIGHT, (0, -0.01, collar_top + CAGE_HEIGHT + CAP_HEIGHT / 2), cage_mat,
                 segments=32, facing_out=False)

    # Faceplate: tick-train glyph, BPM knob, and the two outputs.
    kit.glyphs(fit(gate_pulses(3), 3, 0.0, 1.065, 0.155, 0.06), fit(gate_pulses(2), 2, 0.0, 1.0, 0.11, 0.06),
               glyph_mat)
    kit.title("CLOCK")
    kit.knob("bpm", 0.80, "BPM", param_min=1.0, param_max=999.0, taper="linear", rotation_degrees=300.0)
    kit.output_patch(0.27, 0.30)
    kit.jack("gate", "out", 0.34, "GATE", on_patch=True)
    kit.jack("phase", "out", 0.19, "PHASE", on_patch=True)
    kit.led("gate", (0.13, PANEL_FRONT, 0.40), led_mat, peak_strength=8.0)

    extra_height = COLLAR_HEIGHT + CAGE_HEIGHT + CAP_HEIGHT
    base = kit.base_profile(extra_height=extra_height)
    profile = {
        "id": base["id"],
        "model": base["model"],
        "module_kind": base["module_kind"],
        "scale": base["scale"],
        "size_metres": base["size_metres"],
        "silhouette": "caged_beacon",
        "ports": [
            {"node": "Port_gate", "port": "gate", "direction": "out"},
            {"node": "Port_phase", "port": "phase", "direction": "out"},
        ],
        "params": [{"node": "Knob_bpm", "param": "bpm", "min": 1.0, "max": 999.0,
                    "taper": "linear", "rotation_degrees": 300.0}],
        "lights": [{"node": "Light_gate", "light": "gate", "rest_strength": 0.0, "peak_strength": 8.0}],
        "lamp": {"node": "Lamp", "light": "gate", "pulse_driver": "audio_sample_timestamp",
                 "colour_linear": list(LAMP_WARM), **FLASH},
        "placement": base["placement"],
    }
    return kit.finish(profile, aim=(0, 0, 0.82), camera_location=(1.25, -2.55, 1.5))


if __name__ == "__main__":
    report = run({"clock_lamp": build}, __file__)
    if report is not None:
        if "result" in globals():
            result.update({"modules": report})
        else:
            print(report)
