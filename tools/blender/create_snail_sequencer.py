"""Snail sequencer: the four-step sequencer a snail shell mounts as (sync-snail-shell-sequencer,
ported from the Godot game's add-snail-shell-sequencer, wait-for-stubs d14edf0).

Laid out like a hardware step sequencer (Intellijel Steppy, KORG SQ-1): per step, a CV
knob and a lit push button; CLOCK and RESET in; GATE and CV out on the dark output patch.
The steps run down the panel like the shell's whorls, step 1 at the top, and a
four-whorl glyph tops the panel.

Reproduce:  ~/.local/bin/blender -b --factory-startup -P tools/blender/create_snail_sequencer.py

8HP. Game-facing nodes: `Knob_cv_<n>`, `Button_gate_<n>` (lit: light `step_<n>`),
`Port_clock`, `Port_reset`, `Port_gate`, `Port_cv` (see module_kit.py).
"""
from pathlib import Path
import math
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from module_kit import AMBER, PANEL_FRONT, ModuleKit, run  # noqa: E402

STEPS = 4
STEP_TOP = 0.98          # z of step 1
STEP_PITCH = 0.15        # z between steps
KNOB_X = -0.085
BUTTON_X = 0.105
CV_MAX = 2.0             # volts: two octaves on V/OCT, in semitone steps (the engine snaps)


def whorl_glyph(cx, top, mat, kit):
    """Four stacked whorls, widest at the bottom (the item icon's glyph)."""
    for i, (w, h, dz) in enumerate(((0.04, 0.018, 0.0), (0.06, 0.024, 0.024), (0.084, 0.032, 0.054), (0.112, 0.04, 0.094))):
        z = top - dz
        pts = [(cx + w / 2 * math.cos(t), z + h / 2 * math.sin(t))
               for t in (math.tau * k / 40 for k in range(41))]
        kit.soft_trace(f"Whorl{i}", pts, "xz", PANEL_FRONT - 0.003, 0.006, mat)


def build():
    kit = ModuleKit("SnailSequencer", "snail_sequencer", steps=STEPS)
    glyph_mat = kit.material("Glyph", (0.2, 0.08, 0.02), roughness=0.4, emission=AMBER, strength=4.0)
    step_mat = kit.material("StepKey", (0.32, 0.16, 0.05), roughness=0.35, emission=AMBER, strength=6.0)

    kit.case()
    kit.title("SNAIL")
    whorl_glyph(0.0, 1.175, glyph_mat, kit)

    params, buttons, lights = [], [], []
    for n in range(1, STEPS + 1):
        z = STEP_TOP - (n - 1) * STEP_PITCH
        kit.knob(f"cv_{n}", z, "", x=KNOB_X, radius=0.06)
        kit.label(f"Step{n}Label", str(n), (0.0, PANEL_FRONT - 0.0005, z), size=0.03)
        kit.button(f"gate_{n}", f"step_{n}", BUTTON_X, z, step_mat)
        params.append({"node": f"Knob_cv_{n}", "param": f"cv_{n}", "min": 0.0, "max": CV_MAX,
                       "default": 0.0, "taper": "linear", "rotation_degrees": 300.0})
        buttons.append({"node": f"Button_gate_{n}", "param": f"gate_{n}", "light": f"step_{n}"})
        lights.append({"node": f"Button_gate_{n}", "light": f"step_{n}", "rest_strength": 0.0, "peak_strength": 6.0})

    kit.jack("clock", "in", 0.36, "CLOCK", x=-0.095)
    kit.jack("reset", "in", 0.36, "RESET", x=0.095)
    kit.output_patch(0.14, 0.22)
    kit.jack("gate", "out", 0.11, "GATE", on_patch=True, x=-0.095)
    kit.jack("cv", "out", 0.11, "CV", on_patch=True, x=0.095)

    base = kit.base_profile()
    profile = {
        "id": base["id"],
        "model": base["model"],
        "module_kind": base["module_kind"],
        "scale": base["scale"],
        "size_metres": base["size_metres"],
        "silhouette": "plain",
        "reference": "Intellijel Steppy, KORG SQ-1: clock in, a knob and a lit push button per step",
        "steps": STEPS,
        "ports": [
            {"node": "Port_clock", "port": "clock", "direction": "in"},
            {"node": "Port_reset", "port": "reset", "direction": "in"},
            {"node": "Port_gate", "port": "gate", "direction": "out"},
            {"node": "Port_cv", "port": "cv", "direction": "out"},
        ],
        "params": params,
        "buttons": buttons,
        "lights": lights,
        "placement": base["placement"],
    }
    return kit.finish(profile)


if __name__ == "__main__":
    report = run({"snail_sequencer": build}, __file__)
    if report is not None:
        if "result" in globals():
            result.update({"modules": report})
        else:
            print(report)
