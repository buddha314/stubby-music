"""VCO modules (square, saw, triangle, sine) as world-scale Eurorack modules.

Each oscillator's waveform must read from a distance, so it appears at three
scales: the case's crown is the waveform in silhouette (traced in amber along its
front edge), a large glyph sits on a dark recess on the faceplate, and the same glyph
is on both sides of the case.

Reproduce every VCO:  blender -b --factory-startup -P tools/blender/create_vco.py
Or one:               blender -b --factory-startup -P tools/blender/create_vco.py -- saw

8HP, 1.375 m tall with the crown. Game-facing nodes: `Port_voct`, `Port_out`,
`Knob_freq`, `Light_out` (see module_kit.py).
"""
from pathlib import Path
import math
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from module_kit import AMBER, HEIGHT, PANEL_FRONT, ModuleKit, fit, run  # noqa: E402

CROWN_HEIGHT = 0.09  # the waveform's silhouette on top of the case
CYCLES = 2


def square(cycles):
    pts = []
    for c in range(cycles):
        pts += [(c, -1.0), (c + 0.5, -1.0), (c + 0.5, 1.0), (c + 1.0, 1.0)]
    return pts


def saw(cycles):
    # Rising ramp with a vertical reset: the usual analog VCO saw and panel icon.
    pts = [(0.0, -1.0)]
    for c in range(cycles):
        pts += [(c + 1.0, 1.0), (c + 1.0, -1.0)]
    return pts


def triangle(cycles):
    pts = [(0.0, -1.0)]
    for c in range(cycles):
        pts += [(c + 0.5, 1.0), (c + 1.0, -1.0)]
    return pts


def sine(cycles, steps_per_cycle=32):
    n = cycles * steps_per_cycle
    return [(cycles * i / n, -math.cos(math.tau * cycles * i / n)) for i in range(n + 1)]


WAVEFORMS = {
    "square": {"title": "SQUARE", "kind": "SquareVco", "shape": square},
    "saw": {"title": "SAW", "kind": "SawVco", "shape": saw},
    "triangle": {"title": "TRIANGLE", "kind": "TriangleVco", "shape": triangle},
    "sine": {"title": "SINE", "kind": "SineVco", "shape": sine},
}


def build(waveform):
    spec = WAVEFORMS[waveform]
    kit = ModuleKit(spec["kind"], f"{waveform}_vco", waveform=waveform)
    shape = spec["shape"](CYCLES)

    def wave(centre_u, centre_z, half, amp):
        return fit(shape, CYCLES, centre_u, centre_z, half, amp)

    led_mat = kit.material("LedOut", (0.25, 0.08, 0.01), roughness=0.3, emission=AMBER, strength=8.0)
    glyph_mat = kit.material("Glyph", (0.2, 0.08, 0.02), roughness=0.4, emission=AMBER, strength=4.0)

    kit.case()

    # Silhouette: the crown is the waveform, from the case top up to CROWN_HEIGHT.
    w = kit.width
    edge = wave(0.0, HEIGHT + CROWN_HEIGHT / 2, w / 2, CROWN_HEIGHT / 2)
    outline = edge + [(w / 2, HEIGHT), (-w / 2, HEIGHT)]
    # Drop repeated points (e.g. where the waveform meets a base corner).
    outline = [p for i, p in enumerate(outline)
               if i == 0 or abs(p[0] - outline[i - 1][0]) > 1e-6 or abs(p[1] - outline[i - 1][1]) > 1e-6]
    if abs(outline[0][0] - outline[-1][0]) < 1e-6 and abs(outline[0][1] - outline[-1][1]) < 1e-6:
        outline.pop()
    kit.prism("Crown", outline, kit.body_mat)
    kit.trace("CrownTrace", edge, "xz", PANEL_FRONT - 0.002, 0.012, glyph_mat)

    kit.glyphs(wave(0.0, 1.065, 0.155, 0.06), wave(0.0, 1.0, 0.11, 0.06), glyph_mat)
    kit.title(spec["title"])
    kit.knob("freq", 0.80, "FREQ", param_min=1.0, param_max=20000.0, taper="log", rotation_degrees=300.0)
    kit.jack("voct", "in", 0.47, "V/OCT")
    kit.output_patch(0.25, 0.24)
    kit.jack("out", "out", 0.22, "OUT", on_patch=True)
    kit.led("out", (0.13, PANEL_FRONT, 0.33), led_mat, peak_strength=8.0)

    base = kit.base_profile(extra_height=CROWN_HEIGHT)
    profile = {
        "id": base["id"],
        "model": base["model"],
        "module_kind": base["module_kind"],
        "waveform": waveform,
        "scale": base["scale"],
        "size_metres": base["size_metres"],
        "silhouette": f"{waveform}_crown",
        "ports": [
            {"node": "Port_voct", "port": "voct", "direction": "in"},
            {"node": "Port_out", "port": "out", "direction": "out"},
        ],
        "params": [{"node": "Knob_freq", "param": "freq", "min": 1.0, "max": 20000.0,
                    "taper": "log", "rotation_degrees": 300.0}],
        "lights": [{"node": "Light_out", "light": "out", "rest_strength": 0.0, "peak_strength": 8.0}],
        "placement": base["placement"],
    }
    return kit.finish(profile)


if __name__ == "__main__":
    report = run({name: (lambda name=name: build(name)) for name in WAVEFORMS}, __file__)
    if report is not None:
        if "result" in globals():
            result.update({"modules": report})
        else:
            print(report)
