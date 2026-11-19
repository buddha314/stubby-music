"""Omar's gloves: the first-person hand that holds the hand item (`docs/art/omar.md`,
`add-omar-gloves-and-shadow` D1).

A long midnight-navy glove, subtly segmented (darker seam bands across the back and the
gauntlet), gripping the field recorder: palm against its back, four fingers curled round
its left edge onto the front, the thumb along its right edge. The gauntlet runs back
toward the shoulder, into a band of Omar's flared dark-mint coat sleeve with an aged-brass
edge, as the art direction's short flared sleeves over long gloves.

Modelled in the field recorder's own frame (origin at its Grip), so it mounts with the
held item and needs no pose of its own. The arm aims at Omar's right shoulder, which is
worked out from the recorder's hold pose in `stubby_player::hand` (`HOLD_*` below; keep
them in step).

Reproduce: blender -b --factory-startup -P tools/blender/create_omar_gloves.py
Writes assets/characters/omar_gloves/ (glb and asset.json) and previews/characters/omar_gloves/.
"""
from pathlib import Path
import sys

import bpy
from mathutils import Quaternion, Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from module_kit import ModuleKit  # noqa: E402
from omar_kit import BRASS, COAT, GLOVE, GLOVE_SEAM, blob, ring_around, tube  # noqa: E402

# The recorder's hold pose relative to the player's camera, in Bevy's frame (+Y up, -Z
# forward): `stubby_player::hand` HAND_ITEMS[FieldRecorder].pose.
HOLD_AT = Vector((0.16, -0.13, -0.42))
HOLD_TURN = (Quaternion((0, 1, 0), 0.35) @ Quaternion((1, 0, 0), -0.7)
             @ Quaternion((0, 0, 1), 0.12))
# Omar's right shoulder from his eyes (Bevy's frame): right, down, a little behind.
SHOULDER = Vector((0.2, -0.3, 0.08))


def to_blender(v):
    """glTF/Bevy (+Y up) to Blender (+Z up)."""
    return Vector((v.x, -v.z, v.y))


def in_hand(v_camera):
    """A point given relative to the camera (Bevy) in the recorder's frame (Blender)."""
    return to_blender(HOLD_TURN.inverted() @ (v_camera - HOLD_AT))


class OmarGloves(ModuleKit):
    folder = "characters"

    def __init__(self):
        super().__init__("OmarGloves", "omar_gloves", hp=1)
        for key in ("module_kind", "scale"):
            del self.root[key]
        self.root["kind"] = "first_person_hands"
        m = self.material
        self.glove = m("Glove", GLOVE, roughness=0.5)
        self.seam = m("GloveSeam", GLOVE_SEAM, roughness=0.7)
        self.sleeve = m("Sleeve", COAT, roughness=0.88)
        self.brass = m("Brass", BRASS, metallic=1.0, roughness=0.4)

    def hand(self):
        # Palm: flat against the recorder's back (+Y), over the rubber grip.
        blob(self, "Palm", (0.004, 0.034, -0.022), (0.042, 0.016, 0.05), self.glove)
        # Four fingers round the left edge (-X) onto the front (-Y), index highest.
        for k, (z, length) in enumerate(((0.016, 1.0), (-0.006, 1.05), (-0.028, 1.0), (-0.048, 0.85))):
            r = 0.0085 if k < 3 else 0.0075
            path = [(-0.022, 0.036, z), (-0.04, 0.026, z), (-0.047, 0.004, z - 0.002),
                    (-0.042, -0.018 * length, z - 0.003), (-0.03, -0.026 * length, z - 0.004)]
            tube(self, f"Finger{k}", path, [r * 1.15, r * 1.1, r, r, r * 0.9], self.glove, segments=10)
            # A knuckle seam where each finger leaves the palm.
            ring_around(self, f"Knuckle{k}", (-0.04, 0.026, z), (-0.6, -0.8, 0), r * 1.12, 0.002, self.seam,
                        segments=12)
        # Thumb: from the heel of the palm up the right edge (+X), its tip on the front.
        tube(self, "Thumb", [(0.03, 0.036, -0.05), (0.045, 0.024, -0.03), (0.048, 0.004, -0.008),
                             (0.04, -0.016, 0.006), (0.028, -0.024, 0.012)],
             [0.013, 0.011, 0.0095, 0.009, 0.008], self.glove, segments=10)

    def arm(self):
        wrist = Vector((0.006, 0.036, -0.07))
        shoulder = in_hand(SHOULDER)
        along = (shoulder - wrist).normalized()
        # Gauntlet: from the wrist, widening, to under the sleeve.
        tube(self, "Gauntlet", [wrist, wrist + along * 0.12, wrist + along * 0.26],
             [0.03, 0.035, 0.041], self.glove, segments=16)
        for k, d in enumerate((0.05, 0.1, 0.16)):
            ring_around(self, f"GauntletSeam{k}", wrist + along * d, along, 0.033 + d * 0.04, 0.003,
                        self.seam)
        # The coat's short flared sleeve: open wide at the cuff, narrowing up the arm.
        cuff = wrist + along * 0.2
        tube(self, "Sleeve", [cuff, cuff + along * 0.08, cuff + along * 0.4],
             [0.072, 0.06, 0.055], self.sleeve, segments=18, cap=False)
        ring_around(self, "SleeveEdge", cuff, along, 0.073, 0.008, self.brass)

    def build(self):
        self.hand()
        self.arm()
        profile = {
            "id": self.asset_id,
            "model": f"{self.model_ref}#Scene0",
            "kind": "first_person_hands",
            "holds": "field_recorder",
            "frame": "the field recorder's: origin at its Grip; mount with the held item",
            "palette": {"glove": "midnight navy", "sleeve": "dark mint coat", "edge": "aged brass"},
            "shadow": "cast none: the shadow body's arm casts the hand's shadow",
            "reference": "docs/art/omar.md, docs/art/omar/turnaround-gloves-palette.png",
        }
        # The preview looks from Omar's eyes at the grip (the game's wider view shows it
        # in the lower right).
        return self.finish(profile, aim=(0, 0, -0.03), camera_location=tuple(in_hand(Vector((0, 0, 0)))))


if __name__ == "__main__":
    report = OmarGloves().build()
    if "result" in globals():
        result.update({"omar_gloves": report})
    else:
        print(report)
