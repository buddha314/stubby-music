"""The configuration rack: the mini Eurorack every machine's configuration screen
shows, built from a recipe (add-rack-configurations).

All machines are instruments. Pressing E within range of one opens its configuration
screen, a separate HUD over the game view in the manner of a Satisfactory machine's:
the world dims and blurs behind a panel, and the panel holds a rack. The rack's layout
comes from the machine's **configuration recipe**: a row of slots, each for one kind
of component (clock, VCO, VCF, VCA, ...) at a given width in HP. An empty slot shows
a **labelled outline** of the component that goes there, like a build ghost, so the
player can see what to make and where it goes.

Three generic assets, so racks can grow without new art:

  rack_frame.glb        PostL, PostR (wood, iron straps) and RailTop, RailBottom
                        (titanium). Rails are 1 m long on X, centred: scale X to the
                        recipe's width plus `rail_margin_m`; put the posts at
                        +-(width / 2 + post_inset_m). Modules mount on the rails with
                        their base centre at z = 0 and faceplates facing -Y.
  slot_outline_<hp>hp   one per common width: the outline of an empty slot, drawn at
                        the faceplate in the build-ghost colour. `Label` marks where
                        the slot's label goes.
  slot_labels.glb       Label_<kind> per component kind: a word and a glyph. Clone
                        the one a slot needs onto its outline's `Label` node.

The screen camera, panel and backdrop are in each profile's `screen` block; the
mocks in docs/art/ are rendered from these assets (render_hud_mock.py).

Reproduce: blender -b --factory-startup -P tools/blender/create_hud_rack.py
           (or `-- rack_frame`, `-- slot_outlines`, `-- slot_labels`)
Writes assets/models/hud/ and assets/hud/.
"""
from pathlib import Path
import sys

import bpy

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_burbler import linear  # noqa: E402
from module_kit import HEIGHT, HP, PANEL_FRONT, ModuleKit, run  # noqa: E402

RAIL = (0.05, 0.045)          # depth, height
POST = (0.09, 0.32)           # width, depth
RAIL_MARGIN = 0.12            # rails run past the modules by half this each side
POST_INSET = 0.1              # post centres sit this far outside the modules
HP_SIZES = (4, 6, 8, 10, 12, 14, 16)
INSET = 0.075                 # outline edges sit inside the rails' shadow
GHOST = "#ffd39a"             # the palette's build colour (Build: valid)
KINDS = {                     # component kind -> label word, glyph polyline (x, z) in metres
    "clock": ("CLOCK", [(-0.07, 0.0), (-0.05, 0.0), (-0.05, 0.05), (-0.03, 0.05), (-0.03, 0.0),
                        (0.01, 0.0), (0.01, 0.05), (0.03, 0.05), (0.03, 0.0), (0.07, 0.0)]),
    "vco": ("VCO", [(-0.07, 0.0), (-0.07, 0.05), (-0.023, 0.05), (-0.023, 0.0), (0.023, 0.0),
                    (0.023, 0.05), (0.07, 0.05), (0.07, 0.0)]),
    "vcf": ("VCF", [(-0.07, 0.045), (0.0, 0.045), (0.02, 0.04), (0.035, 0.025), (0.05, 0.008),
                    (0.07, 0.0)]),
    "vca": ("VCA", [(-0.07, 0.0), (0.07, 0.0), (0.07, 0.055), (-0.07, 0.0)]),
}
SCREEN = {
    "title": "{machine}",
    "subtitle": "{bpm} BPM  ·  configure",
    "hints": "turn BPM to set the tempo   ·   drag a component into its slot   ·   "
             "drag between jacks to patch   ·   Esc  close",
    "camera": {"projection": "orthographic", "frame": "the rack's full width and HEIGHT + 0.55 m",
               "view": "straight on, looking along +Y at the faceplates", "render_layer": "dedicated HUD layer"},
    "panel": {"color": "#15161a", "alpha": 0.82, "border": "#ffb350", "border_alpha": 0.4, "corner_px": 18},
    "backdrop": {"blur_px": 14, "dim_color": "#0b0c10", "dim": 0.55},
    "text": {"title": "#e7e5dd", "subtitle": "#ffb350", "hints": "#adb8d4"},
    "key_light": {"type": "area", "energy_w": 260, "size_m": 2.0, "color_srgb": "#ffd9b3",
                  "position_m": [-1.2, -2.6, 1.9], "aim_m": [0.0, 0.0, 0.65]},
}


class HudKit(ModuleKit):
    folder = "hud"

    def __init__(self, kind, asset_id, root):
        super().__init__(kind, asset_id, hp=1)
        for key in ("module_kind", "scale"):
            del self.root[key]
        for key, value in root.items():
            self.root[key] = value
        g = linear(GHOST)
        self.ghost_line = self.material("GhostLine", g, emission=g, strength=1.2)
        self.ghost_fill = self.material("GhostFill", g, emission=g, strength=0.4)
        bsdf = self.ghost_fill.node_tree.nodes["Principled BSDF"]
        bsdf.inputs["Alpha"].default_value = 0.08
        self.ghost_fill.surface_render_method = "BLENDED"

    def node_at(self, name, location, parent=None, **extras):
        empty = bpy.data.objects.new(name, None)
        empty.empty_display_type = "PLAIN_AXES"
        empty.empty_display_size = 0.1
        self.link(empty, parent=parent)
        empty.location = location
        for key, value in extras.items():
            empty[key] = value
        return empty


def build_rack_frame():
    kit = HudKit("HudRack", "rack_frame", {"kind": "hud_rack_frame"})
    m = kit.material
    titanium = m("Titanium", linear("#b8bcc2"), metallic=1.0, roughness=0.32)
    wood = m("Wood", linear("#5a4a3a"), roughness=0.9)
    iron = m("Iron", linear("#5e3a22"), metallic=0.5, roughness=0.85)
    for name, z in (("RailBottom", 0.02), ("RailTop", HEIGHT - 0.02)):
        rail = kit.node_at(name, (0, -0.2, z), stretch="x")
        kit.box(f"{name}Bar", (1.0, RAIL[0], RAIL[1]), (0, 0, 0), titanium).parent = rail
    for name in ("PostL", "PostR"):
        post = kit.node_at(name, (0, 0, 0), place="outside the modules: x = +-(width / 2 + post_inset_m)")
        kit.box(f"{name}Wood", (POST[0], POST[1], HEIGHT + 0.36), (0, 0, HEIGHT / 2 + 0.02), wood).parent = post
        for k, z in enumerate((0.02, HEIGHT - 0.02)):
            kit.box(f"{name}Strap{k}", (0.11, 0.34, 0.05), (0, 0, z), iron).parent = post
    # Preview only: lay it out for a 20 HP rack, as the burbler's.
    width = 20 * HP
    bpy.data.objects["RailBottom"].scale.x = bpy.data.objects["RailTop"].scale.x = 1.0

    def preview():
        for name in ("RailBottom", "RailTop"):
            bpy.data.objects[name].scale.x = width + RAIL_MARGIN
        bpy.data.objects["PostL"].location.x = -(width / 2 + POST_INSET)
        bpy.data.objects["PostR"].location.x = width / 2 + POST_INSET
    profile = {"id": kit.asset_id, "model": f"{kit.model_ref}#Scene0", "kind": "hud_rack_frame",
               "rails": {"nodes": ["RailTop", "RailBottom"], "length_m": 1.0, "axis": "x",
                         "scale_x_to": "rack width + rail_margin_m", "rail_margin_m": RAIL_MARGIN},
               "posts": {"nodes": ["PostL", "PostR"], "x": "+-(rack width / 2 + post_inset_m)",
                         "post_inset_m": POST_INSET},
               "rack_width": "sum of the recipe's slot widths, HP x 0.0508 m",
               "mount": "module base centre at z = 0 on the rack's centre line, faceplate facing -Y",
               "screen": SCREEN}
    return kit.finish(profile, aim=(0, 0, HEIGHT / 2), camera_location=(1.4, -3.4, 1.4), before_render=preview)


def build_slot_outlines():
    results = []
    for hp in HP_SIZES:
        bpy.ops.wm.read_factory_settings(use_empty=True)
        kit = HudKit("SlotOutline", f"slot_outline_{hp}hp", {"kind": "slot_outline", "hp": hp})
        w, front = hp * HP - 0.004, PANEL_FRONT - 0.004
        # Outline: a frame of thin strokes, heavier corner brackets, a faint fill.
        # Inset top and bottom so the rails in front of the faceplate never hide the edges.
        lo, hi = INSET, HEIGHT - INSET
        frame = [(-w / 2, lo), (w / 2, lo), (w / 2, hi), (-w / 2, hi), (-w / 2, lo)]
        kit.trace("Edge", frame, "xz", front, 0.006, kit.ghost_line)
        arm = min(0.08, w * 0.3)
        for k, (cx, cz, sx, sz) in enumerate(((-1, 0, 1, 1), (1, 0, -1, 1), (1, 1, -1, -1), (-1, 1, 1, -1))):
            x, z = cx * w / 2, lo if cz == 0 else hi
            kit.trace(f"Corner{k}", [(x + sx * arm, z), (x, z), (x, z + sz * arm)], "xz", front - 0.001, 0.016,
                      kit.ghost_line)
        kit.box("Fill", (w, 0.002, hi - lo), (0, front + 0.002, HEIGHT / 2), kit.ghost_fill)
        kit.node_at("Label", (0, front - 0.002, HEIGHT * 0.56), role="label_anchor")
        profile = {"id": kit.asset_id, "model": f"{kit.model_ref}#Scene0", "kind": "slot_outline", "hp": hp,
                   "width_m": round(hp * HP, 4), "label_node": "Label", "color": GHOST,
                   "materials": {"SlotOutline_GhostLine": "outline strokes", "SlotOutline_GhostFill": "faint fill"},
                   "state": "shown while the slot is empty; hide it when a component is mounted"}
        results.append(kit.finish(profile, aim=(0, 0, HEIGHT / 2), camera_location=(1.0, -2.8, 1.0)))
    return results


def build_slot_labels():
    kit = HudKit("SlotLabel", "slot_labels", {"kind": "slot_labels", "kinds": list(KINDS)})
    labels = {}
    for k, (kind, (word, glyph)) in enumerate(KINDS.items()):
        node = kit.node_at(f"Label_{kind}", (0, 0, 0), kind=kind, word=word)
        text = kit.label(f"Word_{kind}", word, (0, -0.002, -0.05), size=0.06, mat=kit.ghost_line)
        text.parent = node
        stroke = kit.trace(f"Glyph_{kind}", glyph, "xz", -0.002, 0.008, kit.ghost_line)
        stroke.parent = node
        labels[kind] = {"node": f"Label_{kind}", "word": word}

    def preview():
        for k, kind in enumerate(KINDS):
            bpy.data.objects[f"Label_{kind}"].location.x = (k - 1.5) * 0.24
    profile = {"id": kit.asset_id, "model": f"{kit.model_ref}#Scene0", "kind": "slot_labels",
               "labels": labels, "use": "clone Label_<kind> onto a slot outline's Label node",
               "add_kind": "add an entry to KINDS in tools/blender/create_hud_rack.py and re-export"}
    return kit.finish(profile, aim=(0, 0, 0), camera_location=(0, -1.2, 0.05), before_render=preview)


if __name__ == "__main__":
    print(run({"rack_frame": build_rack_frame, "slot_outlines": build_slot_outlines,
               "slot_labels": build_slot_labels}, __file__))
