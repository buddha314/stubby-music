"""The Dancers: the settlement's residents (add-settlement-economy; brief in
openspec/changes/add-settlement-phase-reviews/design.md, "Dancer appearance and clocks").

Built on a rigged, animated CC0 base, as the cyber-cob is: Quaternius's Universal Animation
Library mannequin (`assets-src/third_party/quaternius/`, CC0 1.0), its 53-joint humanoid rig
and the clips a resident needs. Mostly silhouettes with simplified (blank) faces and clear
poses; low-saturation skin that reads where the light reaches.

Four body shapes on the one rig, each its own model: the mesh is thickened or thinned about
each bone, blended by the skin weights (so joints stay smooth and the rig still fits), and
the whole figure scaled a little:

  dancer_compact   shorter stance, rounded torso
  dancer_lanky     tall frame, long thin limbs
  dancer_broad     wide shoulders, sturdy stance
  dancer_full      softer, rounded torso and hips

Skin tone is chosen per Dancer in the game, not baked: one material, `Dancer_Skin`, which a
placement recolours (`stubby_world::prefab_presentation::MaterialTint`). The four draft tones
are in the family profile; the model ships Solar amber.

Clips kept and renamed: Idle (loops in time with the nearest clock), Dance, Walk, Talk, Sit,
Tend (kneeling at work), Interact. The source's weapons, punches, spells, deaths and the
rest are dropped.

The source is 27k triangles; each Dancer is reduced to about 2.7k, skin weights kept, since a
city holds hundreds. One skinned level, no LOD metadata (Jackdaw's LOD runtime would flatten
the skin), as the cob.

Reproduce: blender -b --factory-startup -P tools/blender/create_dancers.py
           (`-- compact` for one shape, `-- montage` for the sheet)
Writes assets/characters/dancers/dancer_<shape>/, assets/characters/dancers/asset.json and
previews/characters/dancers/ (each shape in each tone, and sheet.png).
"""
from pathlib import Path
import json
import shutil
import subprocess
import sys

import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_burbler import Burbler, linear  # noqa: E402
from module_kit import PROJECT  # noqa: E402

SOURCE = PROJECT / "assets-src/third_party/quaternius/universal_animation_library.glb"
OUT = PROJECT / "assets/characters/dancers"
PREVIEW_OUT = PROJECT / "previews/characters/dancers"
DECIMATE = 0.1      # 27k triangles to about 2.7k

CLIPS = {"Idle_Loop": "Idle", "Dance_Loop": "Dance", "Walk_Loop": "Walk",
         "Idle_Talking_Loop": "Talk", "Sitting_Idle_Loop": "Sit", "Fixing_Kneeling": "Tend",
         "Interact": "Interact"}

TONES = {"solar_amber": "#D89B54", "clay_coral": "#C96D74", "leaf_teal": "#4F9D88",
         "dusk_violet": "#8878AB"}

# Per shape: overall scale, and radial thickness about each bone group (blended by weight).
TORSO_LOW = ("DEF-hips", "DEF-spine.001")
TORSO_HIGH = ("DEF-spine.002", "DEF-spine.003", "DEF-shoulder.L", "DEF-shoulder.R", "DEF-neck")
ARMS = ("DEF-upper_arm.L", "DEF-upper_arm.R", "DEF-forearm.L", "DEF-forearm.R")
LEGS = ("DEF-thigh.L", "DEF-thigh.R", "DEF-shin.L", "DEF-shin.R")
SHAPES = {
    "compact": {"scale": 0.88, TORSO_LOW: 1.4, TORSO_HIGH: 1.35, ARMS: 1.3, LEGS: 1.32},
    "lanky": {"scale": 1.08, TORSO_LOW: 0.82, TORSO_HIGH: 0.85, ARMS: 0.76, LEGS: 0.78},
    "broad": {"scale": 1.0, TORSO_LOW: 1.15, TORSO_HIGH: 1.6, ARMS: 1.35, LEGS: 1.18},
    "full": {"scale": 0.96, TORSO_LOW: 1.65, TORSO_HIGH: 1.3, ARMS: 1.25, LEGS: 1.38},
}
DESCRIPTIONS = {"compact": "shorter stance and rounded torso", "lanky": "tall frame with long limbs",
                "broad": "wide shoulders and a sturdy stance", "full": "softer, rounded profile through the torso and hips"}


class Dancer(Burbler):
    folder = "characters/dancers"

    def __init__(self, shape):
        self.shape = shape
        Burbler.__init__(self, f"dancer_{shape}", kind=f"Dancer{shape.capitalize()}")
        for key in ("mine", "produces"):
            del self.root[key]
        self.root["kind"] = "character"
        self.root["family"] = "dancers"
        self.skin = self.material("Skin", linear(TONES["solar_amber"]), roughness=0.85)
        # Rename so the game's tint finds it by the family's name, not this shape's.
        self.skin.name = "Dancer_Skin"

    def load(self):
        before = set(bpy.data.objects)
        bpy.ops.import_scene.gltf(filepath=str(SOURCE))
        new = [o for o in bpy.data.objects if o not in before]
        for obj in new:
            for col in list(obj.users_collection):
                col.objects.unlink(obj)
            self.collection.objects.link(obj)
        for obj in [o for o in new if o.type == "MESH" and not o.vertex_groups]:
            bpy.data.objects.remove(obj)          # the source's stray icosphere
        self.arm = next(o for o in self.collection.objects if o.type == "ARMATURE")
        self.body = next(o for o in self.collection.objects if o.type == "MESH")
        self.arm.name, self.body.name = "Dancer_Armature", "Dancer_Body"
        self.arm.parent = self.root
        for action in list(bpy.data.actions):
            if action.name in CLIPS:
                action.name = CLIPS[action.name]
            else:
                bpy.data.actions.remove(action)
        if self.arm.animation_data is None:
            self.arm.animation_data_create()
        self.arm.animation_data.action = bpy.data.actions.get("Idle")
        # One skin material: the silhouette reads, the tone is the game's to choose.
        self.body.data.materials.clear()
        self.body.data.materials.append(self.skin)
        for poly in self.body.data.polygons:
            poly.material_index = 0

    def decimate(self):
        layer = self.scene.view_layers[0]
        self.arm.data.pose_position = "REST"
        mod = self.body.modifiers.new("Decimate", "DECIMATE")
        mod.ratio = DECIMATE
        mod.use_collapse_triangulate = True
        with bpy.context.temp_override(scene=self.scene, view_layer=layer, object=self.body,
                                       active_object=self.body):
            bpy.ops.object.modifier_move_to_index(modifier=mod.name, index=0)
            bpy.ops.object.modifier_apply(modifier=mod.name)
        self.arm.data.pose_position = "POSE"

    def shape_body(self):
        """Thicken or thin about each bone, blended by the skin weights."""
        factors = {}
        for groups, f in SHAPES[self.shape].items():
            if groups != "scale":
                for g in groups:
                    factors[g] = f
        bones = {b.name: (self.arm.matrix_world @ b.head_local, self.arm.matrix_world @ b.tail_local)
                 for b in self.arm.data.bones}
        names = {vg.index: vg.name for vg in self.body.vertex_groups}
        mw, inv = self.body.matrix_world, self.body.matrix_world.inverted()
        for v in self.body.data.vertices:
            p = mw @ v.co
            total = sum(g.weight for g in v.groups) or 1.0
            shift = Vector()
            for g in v.groups:
                name = names.get(g.group)
                f = factors.get(name)
                if f is None or name not in bones:
                    continue
                head, tail = bones[name]
                axis = tail - head
                t = max(0.0, min(1.0, (p - head).dot(axis) / max(axis.length_squared, 1e-9)))
                proj = head + axis * t
                shift += (p - proj) * (f - 1.0) * (g.weight / total)
            v.co = inv @ (p + shift)
        s = SHAPES[self.shape]["scale"]
        self.arm.scale = tuple(c * s for c in self.arm.scale)

    def build(self):
        self.load()
        self.decimate()
        self.shape_body()
        bpy.context.view_layer.update()
        tris = sum(len(p.vertices) - 2 for p in self.body.data.polygons)
        height = round(1.83 * SHAPES[self.shape]["scale"], 2)
        profile = {
            "id": self.asset_id, "model": f"{self.model_ref}#Scene0", "kind": "character",
            "family": "dancers", "shape": self.shape, "about": DESCRIPTIONS[self.shape],
            "height_m": height, "skin_material": "Dancer_Skin",
            "clips": sorted(CLIPS.values()), "loop": "Idle unless a LoopClip names another",
            "acoustics": {"materials": {"Dancer_Skin": None},
                          "about": "a moving character: not part of the static acoustic scene"},
            "triangles": {"near": tris},
            "lod": "none: one skinned level (Jackdaw's LOD runtime would flatten the skin)",
            "licence": "CC0-1.0: base by Quaternius (Universal Animation Library, CC0 1.0); "
                       "changes generated by tools/blender/create_dancers.py",
        }
        self.scene.frame_set(10)

        def stage():
            import math
            sun = bpy.data.objects.new("PreviewSun", bpy.data.lights.new("PreviewSun", "SUN"))
            sun.data.energy = 3.5
            sun.rotation_euler = (math.radians(50), 0, math.radians(30))
            self.scene.collection.objects.link(sun)
            self.scene.camera.data.lens = 50
            self.scene.render.resolution_x, self.scene.render.resolution_y = 360, 480
        # One camera for every shape, so their heights and builds compare on the sheet.
        report = self.finish(profile, aim=(0, 0, 0.95), camera_location=(1.6, -3.6, 1.3),
                             before_render=stage)
        if bpy.app.background:
            out = PREVIEW_OUT / self.asset_id
            for tone, hex_ in TONES.items():
                self.skin.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (*linear(hex_), 1.0)
                self.scene.render.filepath = str(out / f"{tone}.png")
                bpy.ops.render.render(write_still=True, scene=self.scene.name)
            self.arm.animation_data.action = bpy.data.actions.get("Dance")
            self.scene.frame_set(14)
            self.scene.render.filepath = str(out / "dance.png")
            bpy.ops.render.render(write_still=True, scene=self.scene.name)
            (out / "preview_far.png").unlink(missing_ok=True)
        report.update(triangles=tris)
        return report


def write_profile():
    profiles = {s: json.loads((OUT / f"dancer_{s}/asset.json").read_text()) for s in SHAPES}
    (OUT / "asset.json").write_text(json.dumps({
        "id": "dancers", "kind": "character",
        "about": "The settlement's residents: four body shapes on one rig, each tinted per Dancer "
                 "(MaterialTint on Dancer_Skin). Body shape and skin tone combine independently.",
        "shapes": {s: f"characters/dancers/dancer_{s}/dancer_{s}.glb#Scene0" for s in SHAPES},
        "tones": TONES,
        "clips": sorted(CLIPS.values()),
        "triangles": {s: p["triangles"]["near"] for s, p in profiles.items()},
    }, indent=2) + "\n")


def montage():
    magick = shutil.which("magick")
    if not magick:
        return
    cmd = [magick, "montage"]
    for s in SHAPES:
        for tone in TONES:
            cmd += ["-label", f"{s} / {tone.replace('_', ' ')}", str(PREVIEW_OUT / f"dancer_{s}/{tone}.png")]
    cmd += ["-tile", "4x4", "-geometry", "+4+4", "-background", "#15161a", "-fill", "#d8d4cc",
            "-pointsize", "14", str(PREVIEW_OUT / "sheet.png")]
    subprocess.run(cmd, check=True)


if __name__ == "__main__":
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if args == ["montage"]:
        write_profile()
        montage()
    elif args:
        print(json.dumps(Dancer(args[0]).build()))
    else:
        for s in SHAPES:
            subprocess.run([bpy.app.binary_path, "-b", "--factory-startup", "-P", __file__, "--", s], check=True)
        write_profile()
        montage()
