"""Reed clump: a frequent renewable resource (add-reeds).

Reeds are the common, fast-returning resource of wet ground, the way wood or leaves
are in a builder game. A clump is a stand of tall reeds with a few cattail heads.
Harvesting it with E cuts it to stubble; after the catalog's replenishment interval it
has grown back. Because they are everywhere and come back quickly, a clump is cheap:
a couple of thousand triangles, with a far LOD of crossed strips.

Variation works like the scrub oak's: seeded variants here (stalk count, heights,
lean, blades, heads), and per-instance scale, yaw, height and colour in the game.

Game-facing nodes, per variant:

  ReedClump_Root       extras: kind = resource, resource = reed
  Interact             where E targets the clump
  Standing             the uncut reeds. Hide when harvested; while regrowing the game
                       may scale its Z from `regrow.from_scale` to 1
  Sway_<n>             groups of stalks under Standing, pivot at the ground: rock them
                       gently, each with its own phase
  Stubble              the cut stalks, shown while harvested and regrowing
  Far LOD              crossed strips in `reed_clump_<k>_LOD1.glb` beside it (Jackdaw's
                       LodGroup convention), shown beyond `far_lod_m`

The glb ships standing with Stubble at scale 0, so it shows correctly wherever it is
placed (Jackdaw, the game) before any game code poses it.

Reproduce: blender -b --factory-startup -P tools/blender/create_reeds.py
           (`-- 2` for one variant, `-- montage` for the lineup)
Writes assets/models/flora/reeds/reed_clump_<k>(_LOD1).glb and assets/flora/reeds/.
"""
from pathlib import Path
import json
import math
import random
import shutil
import subprocess
import sys

import bmesh
import bpy
from mathutils import Vector, noise

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_burbler import Burbler, linear  # noqa: E402
from module_kit import PROJECT  # noqa: E402

VARIANTS = [5, 17, 29, 44]
OUT = PROJECT / "assets/flora/reeds"
FAR_LOD_M = 30.0
SWAY_GROUPS = 3
STUBBLE_HEIGHT = 0.12
COLORS = {"stalk_dry": "#9a8460", "stalk_green": "#5f6e34", "blade_dry": "#8a7552",
          "blade_green": "#4f5e2c", "head": "#4a3222", "cut": "#c2ab7c"}
PER_INSTANCE = {
    "variant": "hash(world_seed, instance_id) % variant count",
    "yaw_deg": [0, 360],
    "scale": [0.85, 1.2],
    "height_stretch": [0.85, 1.2],
    "sway_phase": [0, 1],
    "greenness_jitter": 0.1,
}


class ReedClump(Burbler):
    folder = "flora/reeds"

    def __init__(self, index):
        self.index, self.seed = index, VARIANTS[index]
        Burbler.__init__(self, f"reed_clump_{index}", kind="ReedClump")
        for key in ("mine", "produces"):
            del self.root[key]
        self.root["kind"] = "resource"
        self.root["resource"] = "reed"
        self.root["variant"] = index
        self.rng = random.Random(self.seed)
        noise.seed_set(self.seed)
        m = self.material
        self.stalk = m("Stalk", linear(COLORS["stalk_dry"]), roughness=0.8)
        self.blade = m("Blade", linear(COLORS["blade_dry"]), roughness=0.85)
        self.head = m("Head", linear(COLORS["head"]), roughness=0.95)
        self.cut = m("Cut", linear(COLORS["cut"]), roughness=0.9)
        self.stubble_tops = []

    # ------------------------------------------------------------ parts

    @staticmethod
    def path(base, lean, height, curve, n=6):
        """A stalk's centre line: up from the base, leaning and bowing a little."""
        return [base + lean * height * (i / n) + Vector((0, 0, height * i / n)) + curve * (i / n) ** 2
                for i in range(n + 1)]

    @staticmethod
    def tube_into(bm, points, r0, r1, sides=4):
        rings = []
        for i, p in enumerate(points):
            r = r0 + (r1 - r0) * i / (len(points) - 1)
            rings.append([bm.verts.new(p + Vector((r * math.cos(a), r * math.sin(a), 0)))
                          for a in (2 * math.pi * k / sides + math.pi / 4 for k in range(sides))])
        for ra, rb in zip(rings, rings[1:]):
            for k in range(sides):
                bm.faces.new((ra[k], ra[(k + 1) % sides], rb[(k + 1) % sides], rb[k]))
        return rings

    @staticmethod
    def blade_into(bm, points, width):
        """A flat leaf blade, tapering to a point, its face turned across the bow."""
        rows = []
        for i, p in enumerate(points):
            a = points[max(i - 1, 0)]
            b = points[min(i + 1, len(points) - 1)]
            side = (b - a).cross(Vector((0, 0, 1)))
            side = side.normalized() if side.length > 1e-6 else Vector((1, 0, 0))
            w = width * (1 - i / (len(points) - 1)) * 0.5 + 0.002
            rows.append((bm.verts.new(p - side * w), bm.verts.new(p + side * w)))
        for (a0, a1), (b0, b1) in zip(rows, rows[1:]):
            bm.faces.new((a0, a1, b1, b0))

    def build_clump(self):
        rng = self.rng
        standing = self.node("Standing", (0, 0, 0), regrow_axis="Z")
        groups = [self.node(f"Sway_{g}", (0, 0, 0), standing, sway_max_deg=4.0) for g in range(SWAY_GROUPS)]
        stalk_bms = [bmesh.new() for _ in groups]
        blade_bms = [bmesh.new() for _ in groups]
        head_bms = [bmesh.new() for _ in groups]
        stub = bmesh.new()
        count = rng.randint(14, 22)
        spread = rng.uniform(0.18, 0.32)
        for s in range(count):
            a, d = rng.uniform(0, 2 * math.pi), spread * math.sqrt(rng.random())
            base = Vector((math.cos(a) * d, math.sin(a) * d, -0.02))
            out = Vector((math.cos(a), math.sin(a), 0))
            lean = out * rng.uniform(0.04, 0.22) + Vector((rng.uniform(-0.05, 0.05), rng.uniform(-0.05, 0.05), 0))
            height = rng.uniform(0.9, 1.9)
            curve = out * rng.uniform(0.0, 0.18)
            g = s % SWAY_GROUPS
            pts = self.path(base, lean, height, curve)
            r = rng.uniform(0.007, 0.012)
            self.tube_into(stalk_bms[g], pts, r, r * 0.25)
            if rng.random() < 0.3:   # a cattail head near the top
                top = pts[-2]
                head = bmesh.ops.create_uvsphere(head_bms[g], u_segments=6, v_segments=4, radius=1.0)
                for v in head["verts"]:
                    v.co = Vector((v.co.x * r * 2.2, v.co.y * r * 2.2, v.co.z * height * 0.07)) + top
            # Stubble: the same stalk cut low, a pale cut face on top.
            cut_pts = self.path(base, lean, STUBBLE_HEIGHT * rng.uniform(0.6, 1.3), Vector(), n=1)
            rings = self.tube_into(stub, cut_pts, r * 1.05, r * 1.0)
            stub.faces.new(rings[-1])
            self.stubble_tops.append(cut_pts[-1])
        for b in range(rng.randint(8, 13)):   # leaf blades arching out from the base
            a, d = rng.uniform(0, 2 * math.pi), spread * 0.8 * math.sqrt(rng.random())
            base = Vector((math.cos(a) * d, math.sin(a) * d, -0.02))
            out = Vector((math.cos(a), math.sin(a), 0))
            height = rng.uniform(0.5, 1.2)
            pts = self.path(base, out * rng.uniform(0.15, 0.4), height, out * height * rng.uniform(0.25, 0.6)
                            - Vector((0, 0, height * rng.uniform(0.1, 0.35))), n=6)
            self.blade_into(blade_bms[b % SWAY_GROUPS], pts, rng.uniform(0.025, 0.04))
        for g, group in enumerate(groups):
            for name, bm, mat in (("Stalks", stalk_bms[g], self.stalk), ("Blades", blade_bms[g], self.blade),
                                  ("Heads", head_bms[g], self.head)):
                if not bm.faces:
                    bm.free()
                    continue
                mesh = bpy.data.meshes.new(f"ReedClump_{name}_{g}")
                bm.to_mesh(mesh)
                bm.free()
                obj = self.link(bpy.data.objects.new(f"ReedClump_{name}_{g}", mesh), mat, group)
                for face in obj.data.polygons:
                    face.use_smooth = True
        stubble = self.node("Stubble", (0, 0, 0))
        mesh = bpy.data.meshes.new("ReedClump_Stubble")
        stub.to_mesh(mesh)
        stub.free()
        obj = self.link(bpy.data.objects.new("ReedClump_StubbleStalks", mesh), self.stalk, stubble)
        obj.data.materials.append(self.cut)
        for face in obj.data.polygons:   # the caps are the cut faces
            face.material_index = 1 if abs(face.normal.z) > 0.9 else 0
        self.node("Interact", (0, 0, 0.7), role="interact", resource="reed")
        return standing, stubble

    def far_lod(self):
        """Three crossed tapered strips with the clump's silhouette."""
        def fill(bm):
            for k in range(3):
                a = math.pi * k / 3
                dx, dy = math.cos(a) * 0.35, math.sin(a) * 0.35
                v = [bm.verts.new(c) for c in ((-dx, -dy, 0), (dx, dy, 0), (dx * 0.4, dy * 0.4, 1.6),
                                               (-dx * 0.4, -dy * 0.4, 1.6))]
                bm.faces.new(v)
        obj = self.mesh_obj("LOD_Far", fill, self.stalk, smooth=False)
        obj.name = "LOD_Far"
        obj["lod"] = "far"
        return obj

    # ------------------------------------------------------------ output

    def triangles(self, far=False):
        total = 0
        for obj in self.collection.objects:
            if obj.type == "MESH" and (obj.get("lod") == "far") == far:
                total += sum(len(p.vertices) - 2 for p in obj.data.polygons)
        return total

    def set_state(self, standing, stubble, state, greenness=0.0, regrow=1.0):
        def show(node, visible):
            for child in node.children_recursive:
                child.hide_render = not visible
        show(standing, state != "harvested")
        show(stubble, state != "standing")
        # Node scale carries the state into the glb: hidden render flags do not export.
        stubble.scale = (1, 1, 1) if state != "standing" else (0, 0, 0)
        standing.scale = (1, 1, 1) if state != "harvested" else (0, 0, 0)
        standing.scale.z = regrow if state == "regrowing" else 1.0
        for mat, dry, green in ((self.stalk, "stalk_dry", "stalk_green"), (self.blade, "blade_dry", "blade_green")):
            c = [x + (y - x) * greenness for x, y in zip(linear(COLORS[dry]), linear(COLORS[green]))]
            mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (*c, 1.0)

    def build(self):
        standing, stubble = self.build_clump()
        lod = self.far_lod()
        near, far = self.triangles(), self.triangles(far=True)
        self.export_lod([lod])
        self.set_state(standing, stubble, "standing")
        profile = {"id": self.asset_id, "model": f"{self.model_ref}#Scene0", "kind": "resource",
                   "resource": "reed", "variant": self.index, "seed": self.seed,
                   "triangles": {"near": near, "far": far}}

        def stage():
            self.stage_preview(resolution=(420, 560))
        report = self.finish(profile, aim=(0, 0, 0.7), camera_location=(1.4, -3.6, 1.3), before_render=stage)
        if bpy.app.background:
            out = OUT / self.asset_id
            for name, args in PREVIEWS.items():
                self.set_state(standing, stubble, *args)
                self.scene.render.filepath = str(out / f"preview_{name}.png")
                bpy.ops.render.render(write_still=True, scene=self.scene.name)
        report.update(near=near, far=far)
        return report


PREVIEWS = {  # name: (state, greenness, regrow)
    "dry": ("standing", 0.0, 1.0),
    "harvested": ("harvested", 0.0, 1.0),
    "regrowing": ("regrowing", 0.5, 0.45),
    "green": ("standing", 1.0, 1.0),
}


def write_profile():
    profiles = [json.loads((OUT / f"reed_clump_{k}/asset.json").read_text()) for k in range(len(VARIANTS))]
    profile = {
        "id": "reed_clump",
        "kind": "resource",
        "resource": "reed",
        "renewable": "frequent: harvest cuts the clump to stubble; it returns after the catalog's "
                     "replenishment interval (a short elapsed-time policy, add-resource-replenishment)",
        "variants": [p["model"] for p in profiles],
        "per_instance": PER_INSTANCE,
        "states": {
            "standing": "Standing scale 1, Stubble scale 0 (as shipped)",
            "harvested": "Standing scale 0, Stubble scale 1",
            "regrowing": "Stubble scale 1; Standing scale (1, 1, z), z from regrow.from_scale to 1 over the interval",
        },
        "regrow": {"from_scale": STUBBLE_HEIGHT / 1.4, "axis": "Z", "node": "Standing"},
        "greenness": {"value": "0 dry .. 1 green; follow the local water or light quality, as the game decides",
                      "colors": COLORS,
                      "materials": {"ReedClump_Stalk": ["stalk_dry", "stalk_green"],
                                    "ReedClump_Blade": ["blade_dry", "blade_green"]}},
        "sway": {"nodes": "Sway_<n>", "pivot": "ground", "max_deg": 4.0,
                 "phase": "per_instance.sway_phase plus the group index; follows the wind"},
        "interact": "Interact",
        "far_lod": {"distance_m": FAR_LOD_M, "file": "models/flora/reeds/reed_clump_<k>_LOD1.glb",
                    "note": "Jackdaw LodGroup level 1"},
        "triangles": {p["id"]: p["triangles"] for p in profiles},
    }
    (OUT / "asset.json").write_text(json.dumps(profile, indent=2) + "\n")


def montage():
    magick = shutil.which("magick")
    if not magick:
        return

    def tile(files, labels, dest):
        cmd = [magick, "montage"]
        for f, label in zip(files, labels):
            cmd += ["-label", label, str(f)]
        cmd += ["-tile", f"{len(files)}x1", "-geometry", "+4+4", "-background", "#15161a", "-fill", "#d8d4cc",
                "-pointsize", "16", str(dest)]
        subprocess.run(cmd, check=True)
    tile([OUT / f"reed_clump_0/preview_{p}.png" for p in PREVIEWS], list(PREVIEWS), OUT / "states.png")
    tile([OUT / f"reed_clump_{k}/preview_dry.png" for k in range(len(VARIANTS))],
         [f"variant {k}" for k in range(len(VARIANTS))], OUT / "lineup.png")
    for k in range(1, len(VARIANTS)):   # keep the repo light; re-rendered on every run
        for name in ("preview.png", "preview_far.png", "preview_harvested.png", "preview_regrowing.png",
                     "preview_green.png"):
            (OUT / f"reed_clump_{k}" / name).unlink(missing_ok=True)


if __name__ == "__main__":
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if args == ["montage"]:
        write_profile()
        montage()
    elif args:
        print(ReedClump(int(args[0])).build())
    else:
        for k in range(len(VARIANTS)):
            subprocess.run([bpy.app.binary_path, "-b", "--factory-startup", "-P", __file__, "--", str(k)],
                           check=True)
        write_profile()
        montage()
