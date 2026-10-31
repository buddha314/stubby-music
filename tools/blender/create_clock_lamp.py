"""The Clock, the opening's second discovery: a tempo module with a BPM readout and a
flashing clock light, in the spirit of performance clock modules.

The clock manifests as a pulsating light (desert-opening spec). The panel's large
domed lens is that light (`Lamp`), and the same flash material lights the tick-train
crown along the top and the glyphs on both sides, so the beat reads from a distance
and from any side. Above the lens, a three-digit seven-segment display shows the
tempo; each segment is its own node so the game can show any BPM. Below: a TEMPO
encoder, and GATE and PHASE outputs with their own LEDs. The engine's Clock has no
inputs.

Reproduce: blender -b --factory-startup -P tools/blender/create_clock_lamp.py

Game-facing nodes (see module_kit.py): `Port_gate`, `Port_phase`, `Knob_bpm`,
`Light_gate`, `Light_phase`, `Lamp` (flash envelope in its extras, like the lightning
bug's), and `Display` with segment children `Display_d<digit>_<a-g>`.
"""
from pathlib import Path
import sys

import bpy

sys.path.insert(0, str(Path(__file__).resolve().parent))
from module_kit import AMBER, HEIGHT, PANEL_FRONT, ModuleKit, fit, run  # noqa: E402

CROWN_HEIGHT = 0.09
LAMP_WARM = (1.0, 0.62, 0.25)
FLASH = {  # the lightning bug's envelope fields, so pulses read as one family
    "attack_seconds": 0.015,
    "flash_seconds": 0.25,
    "rest_strength": 0.15,
    "peak_strength": 30.0,
    "light_lumens": 6000.0,
    "light_range_metres": 40.0,
}
PREVIEW_BPM = 120

# Seven-segment layout: which segments light for each digit.
SEGMENTS = "abcdefg"
DIGITS = {
    "0": "abcdef", "1": "bc", "2": "abdeg", "3": "abcdg", "4": "bcfg",
    "5": "acdfg", "6": "acdefg", "7": "abc", "8": "abcdefg", "9": "abcdfg",
}


def gate_pulses(cycles):
    """Narrow centred pulses, like a clock's tick train."""
    pts = [(0.0, -1.0)]
    for c in range(cycles):
        pts += [(c + 0.375, -1.0), (c + 0.375, 1.0), (c + 0.625, 1.0), (c + 0.625, -1.0), (c + 1.0, -1.0)]
    return pts


def segment_boxes(cx, cz, w, h, t):
    """Each segment's (size_x, size_z, centre_x, centre_z) for a digit centred at (cx, cz)."""
    hw, hh = w / 2, h / 2
    horizontal = (w - t, t)
    vertical = (t, hh - t / 2)
    return {
        "a": (*horizontal, cx, cz + hh - t / 2),
        "b": (*vertical, cx + hw - t / 2, cz + hh / 2),
        "c": (*vertical, cx + hw - t / 2, cz - hh / 2),
        "d": (*horizontal, cx, cz - hh + t / 2),
        "e": (*vertical, cx - hw + t / 2, cz - hh / 2),
        "f": (*vertical, cx - hw + t / 2, cz + hh / 2),
        "g": (*horizontal, cx, cz),
    }


def build():
    kit = ModuleKit("Clock", "clock_lamp")
    # One flash material for the lens, the crown trace and the side glyphs: the game
    # modulates it once per tick and every view of the clock flashes together.
    lamp_mat = kit.material("Lamp", (0.6, 0.35, 0.12), roughness=0.15, emission=LAMP_WARM,
                            strength=FLASH["peak_strength"])
    segment_mat = kit.material("Segment", (0.3, 0.07, 0.02), roughness=0.3, emission=(1.0, 0.32, 0.06),
                               strength=6.0)
    ghost_mat = kit.material("SegmentGhost", (0.05, 0.025, 0.02), roughness=0.4)
    window_mat = kit.material("DisplayWindow", (0.01, 0.008, 0.008), roughness=0.08)
    led_mat = kit.material("LedGate", (0.25, 0.08, 0.01), roughness=0.3, emission=AMBER, strength=8.0)

    kit.case()
    w = kit.width

    # Silhouette: a tick-train crown, traced in the flash material.
    edge = fit(gate_pulses(2), 2, 0.0, HEIGHT + CROWN_HEIGHT / 2, w / 2, CROWN_HEIGHT / 2)
    outline = edge + [(w / 2, HEIGHT), (-w / 2, HEIGHT)]
    outline = [p for i, p in enumerate(outline)
               if i == 0 or abs(p[0] - outline[i - 1][0]) > 1e-6 or abs(p[1] - outline[i - 1][1]) > 1e-6]
    if abs(outline[0][0] - outline[-1][0]) < 1e-6 and abs(outline[0][1] - outline[-1][1]) < 1e-6:
        outline.pop()
    kit.prism("Crown", outline, kit.body_mat)
    kit.trace("CrownTrace", edge, "xz", PANEL_FRONT - 0.002, 0.012, lamp_mat)
    side_x = (w - 0.01) / 2 + 0.002
    side = fit(gate_pulses(2), 2, 0.0, 1.0, 0.11, 0.06)
    for name, x in (("L", -side_x), ("R", side_x)):
        points = side if name == "R" else [(-u, z) for u, z in side]
        kit.trace(f"SideGlyph{name}", points, "yz", x, 0.024, lamp_mat)

    kit.title("CLOCK")

    # BPM display: dark glass window, faint "8"s behind, lit segments in front.
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
            obj["digit"] = digit
            obj["segment"] = seg
            lit[(digit, seg)] = obj
    kit.label("BpmLabel", "BPM", (0.12, PANEL_FRONT - 0.0005, display_z - 0.125), size=0.03)

    # The clock light: a large domed lens with a dark bezel.
    lens_z = 0.875
    kit.cylinder("LensBezel", 0.085, 0.02, (0, PANEL_FRONT - 0.01, lens_z), kit.knob_mat, segments=48)
    lamp = bpy.data.objects.new("Lamp", None)
    lamp.empty_display_type = "CIRCLE"
    lamp.empty_display_size = 0.07
    kit.link(lamp)
    lamp.location = (0, PANEL_FRONT - 0.02, lens_z)
    lamp.rotation_euler = (1.5707963, 0.0, 0.0)
    lamp["light"] = "gate"
    lamp["pulse_driver"] = "audio_sample_timestamp"
    for key, value in FLASH.items():
        lamp[key] = value
    lens = kit.cylinder("Lens", 0.068, 0.03, (0, 0, 0.0), lamp_mat, segments=48, parent=lamp)
    lens["light"] = "gate"
    for face in lens.data.polygons:
        face.use_smooth = True

    kit.knob("bpm", 0.64, "TEMPO", param_min=1.0, param_max=999.0, taper="linear", rotation_degrees=300.0,
             control="encoder")

    kit.output_patch(0.27, 0.30)
    kit.jack("gate", "out", 0.34, "GATE", on_patch=True)
    kit.jack("phase", "out", 0.19, "PHASE", on_patch=True)
    kit.led("gate", (0.13, PANEL_FRONT, 0.34), led_mat, peak_strength=8.0)
    kit.led("phase", (0.13, PANEL_FRONT, 0.19), led_mat, peak_strength=8.0)

    def show_preview_bpm():
        shown = f"{PREVIEW_BPM:3d}"
        for (digit, seg), obj in lit.items():
            ch = shown[digit]
            on = ch != " " and seg in DIGITS[ch]
            obj.hide_render = not on

    base = kit.base_profile(extra_height=CROWN_HEIGHT)
    profile = {
        "id": base["id"],
        "model": base["model"],
        "module_kind": base["module_kind"],
        "scale": base["scale"],
        "size_metres": base["size_metres"],
        "silhouette": "tick_train_crown",
        "ports": [
            {"node": "Port_gate", "port": "gate", "direction": "out"},
            {"node": "Port_phase", "port": "phase", "direction": "out"},
        ],
        "params": [{"node": "Knob_bpm", "param": "bpm", "min": 1.0, "max": 999.0, "taper": "linear",
                    "rotation_degrees": 300.0, "control": "encoder"}],
        "lights": [
            {"node": "Light_gate", "light": "gate", "rest_strength": 0.0, "peak_strength": 8.0},
            {"node": "Light_phase", "light": "phase", "rest_strength": 0.0, "peak_strength": 8.0},
        ],
        "display": {"node": "Display", "shows": "bpm", "digits": 3, "segments": SEGMENTS,
                    "segment_node": "Display_d{digit}_{segment}", "digit_segments": DIGITS},
        "lamp": {"node": "Lamp", "light": "gate", "material": "Clock_Lamp",
                 "pulse_driver": "audio_sample_timestamp", "colour_linear": list(LAMP_WARM), **FLASH},
        "placement": base["placement"],
    }
    return kit.finish(profile, aim=(0, 0, 0.7), camera_location=(1.05, -2.15, 1.3),
                      before_render=show_preview_bpm)


if __name__ == "__main__":
    report = run({"clock_lamp": build}, __file__)
    if report is not None:
        if "result" in globals():
            result.update({"modules": report})
        else:
            print(report)
