"""The reed water purifier's rack modules (add-reed-water-purifier 3.1).

The purifier copies the user's VCV Rack patch (`assets/rack-recipies/reed-water-filter.vcv`):
a saw VCO and a triangle VCO into a mixer, the mix through a VCF's high-pass, gated by the
borrowed clock through the VCA (`volume`), out through a scope. The VCOs and the VCA
exist; these are the other three panels. They are panels only: how each sounds is its
`module.json`, written with its engine module (add-reed-water-purifier 2.2-2.4).

- `mixer` (8HP, engine `Mixer`): four channels, each an IN jack beside a LEVEL knob, and
  the MIX output with a soft light. Crown: streams meeting, two slopes rising to one peak.
- `vcf` (8HP, engine `Vcf`, state-variable): a large CUTOFF knob (Hz, log; its module data
  quantizes it to semitones, so it names its note), a RES knob, an IN jack, and LPF and HPF
  outputs, each with a soft light. Crown: a resonant filter's response, flat with a peak
  at the cutoff.
- `scope` (10HP, engine `Scope`): a dark screen with a faint graticule, where the game draws
  the recent samples as a glowing trace (`Screen`), TIME and SCALE knobs for the view, an
  IN jack and OUT passing the signal through. Crown: a ripple dying away, the ripple pool
  the oscilloscope comes from.

Drawn in the burbler modules' language (`create_hud_modules.HudKit`): weathered wood case,
rusted iron, titanium faceplate, outputs on graphene, fairy lights.

Reproduce: blender -b --factory-startup -P tools/blender/create_purifier_modules.py [-- mixer vcf scope]
"""
from pathlib import Path
import math
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_hud_modules import CROWN_HEIGHT, HudKit, linear  # noqa: E402
from module_kit import HEIGHT, PANEL_FRONT, run  # noqa: E402

TRACE = linear("#cfe6e2")   # the scope's trace: the ripple pool's pale glint
KNOB = dict(rotation_degrees=300.0)


def ports(*spec):
    return [{"node": f"Port_{p}", "port": p, "direction": d, **({"note": n} if n else {})}
            for p, d, n in spec]


def light(name, peak=8.0):
    return {"node": f"Light_{name}", "light": name, "rest_strength": 0.0, "peak_strength": peak}


def build_mixer():
    kit = HudKit("Mixer", "mixer", hp=8, module_kind="Mixer")
    w = kit.width
    kit.case()
    # Two streams rising to meet at the middle: the mix.
    kit.crown([(-w / 2, HEIGHT + 0.012), (0.0, HEIGHT + CROWN_HEIGHT), (w / 2, HEIGHT + 0.012)], kit.glow)
    confluence = [(-0.09, 1.08), (0.0, 0.99), (0.09, 1.08)]
    kit.side_glyphs(confluence + [(0.0, 0.99), (0.0, 0.92)], kit.glow)
    kit.title("MIX")
    kit.box("GlyphPlate", (0.22, 0.002, 0.13), (0, PANEL_FRONT - 0.001, 1.08), kit.output_mat)
    for k, x in enumerate((-0.06, -0.02, 0.02, 0.06)):   # four channels into one
        kit.trace(f"Glyph_{k}", [(x * 1.4, 1.13), (0.0, 1.05)], "xz", PANEL_FRONT - 0.0035, 0.01, kit.glow)
    kit.trace("Glyph_out", [(0.0, 1.05), (0.0, 1.03)], "xz", PANEL_FRONT - 0.0035, 0.014, kit.glow)

    params, rows = [], (0.9, 0.75, 0.6, 0.45)
    for ch, z in enumerate(rows, start=1):
        kit.jack(f"in_{ch}", "in", z, f"IN {ch}", x=-0.1)
        kit.knob(f"level_{ch}", z, "", x=0.09, radius=0.05, param_min=0.0, param_max=1.0,
                 taper="linear", **KNOB)
        params.append({"node": f"Knob_level_{ch}", "param": f"level_{ch}", "min": 0.0, "max": 1.0,
                       "taper": "linear", **KNOB})
    kit.output_patch(0.2, 0.2)
    kit.jack("mix", "out", 0.2, "MIX", on_patch=True)
    kit.soft_light("mix", (0.13, PANEL_FRONT - 0.004, 0.2))

    profile = kit.profile({
        "silhouette": "confluence_crown",
        "ports": ports(*[(f"in_{ch}", "in", None) for ch in range(1, 5)], ("mix", "out", None)),
        "params": params,
        "lights": [light("mix")],
        "hud": {"place": "mixer", "fixed": True},
    })
    return kit.finish(profile, aim=(0, 0, 0.7), camera_location=(1.05, -2.15, 1.3))


def resonant_edge(w, samples=64):
    """Flat, then a resonant peak at the cutoff a third of the way in, falling away."""
    pts = []
    for i in range(samples + 1):
        u = i / samples
        peak = math.exp(-((u - 0.4) / 0.08) ** 2)
        shelf = 1 / (1 + math.exp((u - 0.55) * 18))   # low-pass: flat below, gone above
        pts.append((-w / 2 + w * u, HEIGHT + 0.012 + 0.035 * shelf + 0.055 * peak))
    return pts


def build_vcf():
    kit = HudKit("Vcf", "vcf", hp=8, module_kind="Vcf")
    w = kit.width
    kit.case()
    kit.crown(resonant_edge(w), kit.glow)
    side = [(-0.1 + 0.2 * u, 0.95 + 0.06 / (1 + math.exp((u - 0.55) * 18)) + 0.09 * math.exp(-((u - 0.4) / 0.08) ** 2))
            for u in (i / 32 for i in range(33))]
    kit.side_glyphs(side, kit.glow)
    kit.title("VCF")

    # Faceplate: low-pass and high-pass crossing at the cutoff.
    kit.box("GlyphPlate", (0.3, 0.002, 0.13), (0, PANEL_FRONT - 0.001, 1.09), kit.output_mat)
    low = [(-0.12 + 0.24 * u, 1.04 + 0.08 / (1 + math.exp((u - 0.5) * 16))) for u in (i / 28 for i in range(29))]
    high = [(-0.12 + 0.24 * u, 1.04 + 0.08 / (1 + math.exp(-(u - 0.5) * 16))) for u in (i / 28 for i in range(29))]
    kit.trace("GlyphLow", low, "xz", PANEL_FRONT - 0.0035, 0.01, kit.glow)
    kit.trace("GlyphHigh", high, "xz", PANEL_FRONT - 0.0035, 0.01, kit.glow)

    kit.knob("cutoff", 0.82, "CUTOFF", radius=0.1, param_min=32.7, param_max=8372.0, taper="log", **KNOB)
    kit.knob("res", 0.63, "RES", radius=0.055, label_dz=-0.085, param_min=0.0, param_max=1.0, taper="linear", **KNOB)
    kit.jack("in", "in", 0.39, "IN")
    kit.output_patch(0.2, 0.2)
    kit.jack("lpf", "out", 0.2, "LPF", on_patch=True, x=-0.09)
    kit.jack("hpf", "out", 0.2, "HPF", on_patch=True, x=0.09)
    kit.soft_light("lpf", (-0.165, PANEL_FRONT - 0.004, 0.15), peak_strength=6.0)
    kit.soft_light("hpf", (0.165, PANEL_FRONT - 0.004, 0.15), peak_strength=6.0)

    profile = kit.profile({
        "silhouette": "resonant_crown",
        "ports": ports(("in", "in", None), ("lpf", "out", "low-pass: solids and nutrients (later filters)"),
                       ("hpf", "out", "high-pass: pure liquid; the purifier's patch uses it")),
        "params": [{"node": "Knob_cutoff", "param": "cutoff", "min": 32.7, "max": 8372.0, "taper": "log",
                    "unit": "Hz", "note": "C1 to C9; its module data quantizes it to semitones", **KNOB},
                   {"node": "Knob_res", "param": "res", "min": 0.0, "max": 1.0, "taper": "linear", **KNOB}],
        "lights": [light("lpf", 6.0), light("hpf", 6.0)],
        "hud": {"place": "vcf", "fixed": True},
    })
    return kit.finish(profile, aim=(0, 0, 0.7), camera_location=(1.05, -2.15, 1.3))


def ripple_edge(w, samples=96):
    """A ripple dying away from the left: the pool the oscilloscope comes from."""
    pts = []
    for i in range(samples + 1):
        u = i / samples
        pts.append((-w / 2 + w * u, HEIGHT + 0.045 + 0.035 * math.exp(-3.2 * u) * math.cos(u * 6 * math.tau)))
    return pts


def build_scope():
    kit = HudKit("Scope", "scope", hp=10, module_kind="Scope")
    w = kit.width
    trace_mat = kit.material("Trace", TRACE, roughness=0.3, emission=TRACE, strength=6.0)
    grid_mat = kit.material("Graticule", (0.03, 0.05, 0.05), roughness=0.6, emission=TRACE, strength=0.25)
    window_mat = kit.material("ScreenWindow", (0.006, 0.009, 0.009), roughness=0.06)
    kit.case()
    kit.crown(ripple_edge(w), trace_mat)
    side = [(-0.1 + 0.2 * u, 0.98 + 0.05 * math.exp(-3.2 * u) * math.cos(u * 4 * math.tau)) for u in (i / 48 for i in range(49))]
    kit.side_glyphs(side, trace_mat)
    kit.title("SCOPE")

    # The screen. The game draws the recent samples across it as a line in `Trace` light.
    sw, sh, sz = 0.42, 0.34, 0.95
    kit.box("ScreenBezel", (sw + 0.03, 0.006, sh + 0.03), (0, PANEL_FRONT - 0.002, sz), kit.screw_mat, bevel=0.006)
    kit.box("ScreenWindow", (sw, 0.004, sh), (0, PANEL_FRONT - 0.005, sz), window_mat)
    for k in range(1, 8):          # graticule: eight divisions across, six up
        x = -sw / 2 + sw * k / 8
        kit.trace(f"GridV_{k}", [(x, sz - sh / 2 + 0.01), (x, sz + sh / 2 - 0.01)], "xz", PANEL_FRONT - 0.0075,
                  0.0025 if k != 4 else 0.004, grid_mat, depth=0.001)
    for k in range(1, 6):
        z = sz - sh / 2 + sh * k / 6
        kit.trace(f"GridH_{k}", [(-sw / 2 + 0.01, z), (sw / 2 - 0.01, z)], "xz", PANEL_FRONT - 0.0075,
                  0.0025 if k != 3 else 0.004, grid_mat, depth=0.001)
    kit.socket("Screen", (0, PANEL_FRONT - 0.008, sz), display="waveform", width_m=sw - 0.02,
               height_m=sh - 0.02, samples=256, trace_material="Scope_Trace", channel="in")

    kit.knob("time", 0.64, "TIME", x=-0.1, radius=0.055, label_dz=-0.1, param_min=0.002, param_max=0.25,
             taper="log", **KNOB)
    kit.knob("scale", 0.64, "SCALE", x=0.1, radius=0.055, label_dz=-0.1, param_min=0.1, param_max=4.0,
             taper="log", **KNOB)
    kit.jack("in", "in", 0.38, "IN")
    kit.output_patch(0.2, 0.2)
    kit.jack("out", "out", 0.2, "OUT", on_patch=True)
    kit.soft_light("out", (0.15, PANEL_FRONT - 0.004, 0.2), mat=trace_mat)

    preview = []

    def show_trace():
        # Preview only, after export: a gated high-passed saw, as the purifier draws it.
        pts = []
        for i in range(161):
            u = i / 160
            gate = 1.0 if (u * 2) % 1 < 0.6 else 0.0
            v = gate * (0.9 * ((u * 5) % 1 - 0.5) + 0.12 * math.sin(u * 15 * math.tau))
            pts.append((-sw / 2 + 0.015 + (sw - 0.03) * u, sz + v * (sh - 0.06) / 1.3))
        preview.append(kit.trace("PreviewTrace", pts, "xz", PANEL_FRONT - 0.009, 0.005, trace_mat))

    profile = kit.profile({
        "silhouette": "ripple_crown",
        "ports": ports(("in", "in", None), ("out", "out", "IN passed through unchanged")),
        "params": [{"node": "Knob_time", "param": "time", "min": 0.002, "max": 0.25, "taper": "log",
                    "unit": "seconds across the screen", **KNOB},
                   {"node": "Knob_scale", "param": "scale", "min": 0.1, "max": 4.0, "taper": "log",
                    "unit": "screen heights per volt", **KNOB}],
        "lights": [light("out")],
        "screen": {"node": "Screen", "display": "waveform", "width_metres": sw - 0.02,
                   "height_metres": sh - 0.02, "samples": 256, "trace_material": "Scope_Trace",
                   "colour_linear": list(TRACE), "graticule": [8, 6],
                   "draw": "the recent samples of IN as a line across the screen, centred on 0 V"},
        "hud": {"place": "scope", "fixed": True},
    })
    return kit.finish(profile, aim=(0, 0, 0.75), camera_location=(1.2, -2.3, 1.4), before_render=show_trace)


if __name__ == "__main__":
    print(run({"mixer": build_mixer, "vcf": build_vcf, "scope": build_scope}, __file__))
