"""Scrub oak: the infinite mine that produces lightning bugs (add-art-direction 2.1,
for add-scrub-oak-production).

A scrub oak is a low, many-stemmed desert oak, about 2.5-4 m tall: a few gnarled
stems leaning out from one root crown, splitting into twisting limbs, with a dense
rounded crown of small leathery leaves.

It starts **dormant**, in winter: grey bare wood, tight buds, a few dry leaves still
clinging. Clean water wakes it. One growth value, 0 to 1, rises as clean water is
delivered and escalates it through budding, leaf-out and leafed to **full bloom**,
when luminous catkins hang from the crown. The catkins are where its lightning bugs
come from, so a dormant tree makes none. GROWTH holds the mapping the game applies;
the previews are rendered through the same mapping.

Variation works in two layers, so no two trees in the world match:

1. **Variants (here).** The tree is grown procedurally from a seed: stem count, lean,
   branching angles, splits, lengths and crown clusters all come from it. Each seed in
   VARIANTS is exported as its own glb, `scrub_oak_<k>.glb`.
2. **Per instance (in the game).** Each placed tree picks a variant and draws its
   scale, height stretch, yaw, lean, sway phase and bloom tint from the ranges in the
   profile, seeded by the world seed and its instance ID.

Game-facing nodes, per variant:

  ScrubOak_Root        extras: kind = mine, mine = scrub_oak, produces = lightning_bug,
                       variant
  Limb_<n>             one per stem, origin at its base: rotate it a little to sway
                       (to the music once played to). Everything on a stem is parented
                       to it, so leaves, blooms and spawn points sway with it
  Dead_<n>             dry winter leaves; fall (scale to 0) as the tree wakes
  Bud_<n>              buds on the outer tips; swell and warm, then open into leaves
  Foliage_<n>          leaf clusters; scale 0 -> 1 to leaf out
  Bloom_<k>            hanging luminous catkins, extras: order. Open one by one in
                       `order` over the last stretch of growth
  BugSpawn_<k>         where a produced bug appears: one in each bloom
Far LOD: `scrub_oak_<k>_LOD1.glb` beside it (Jackdaw's LodGroup convention), with
  ScrubOak_LOD_Far_Bark and ScrubOak_LOD_Far_Foliage_Node; the foliage node ships at scale 0
  (dormant) and follows leaf_out like Foliage_<n>.

The glb ships posed dormant using node scale only, so it shows correctly wherever it
is placed (Jackdaw, the game) before any game code poses it.

Reproduce: blender -b --factory-startup -P tools/blender/create_scrub_oak.py
           (or `-- 3` for one variant; `-- montage` rebuilds the lineup only)
Writes assets/models/flora/scrub_oak/scrub_oak_<k>(_LOD1).glb and assets/flora/scrub_oak/ (profiles,
previews per variant and stage, and the lineup).
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
from mathutils import Matrix, Quaternion, Vector, noise

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_burbler import Burbler, linear  # noqa: E402
from module_kit import PROJECT  # noqa: E402

VARIANTS = [11, 23, 37, 41, 58, 64]   # one seed per exported variant
OUT = PROJECT / "assets/flora/scrub_oak"
RING = 6                               # sides on a branch tube
FAR_LOD_M = 45.0
MAX_BLOOMS = 18
# Growth 0 (dormant) .. 1 (full bloom), driven by clean water. [from, to] per change.
GROWTH = {
    "bark_wakes": [0.0, 0.4],
    "dead_leaves_fall": [0.08, 0.25],
    "buds_swell": [0.1, 0.3],
    "buds_open": [0.42, 0.55],
    "leaf_out": [0.35, 0.65],
    "leaf_matures": [0.45, 0.8],
    "bloom": [0.72, 0.97],
    "bloom_open_span": 0.03,
}
COLORS = {"bark_dormant": "#625d57", "bark_alive": "#4b4037", "leaf_young": "#7b8c3c",
          "leaf_mature": "#36412a", "dead_leaf": "#7a5a3a", "bud": "#9a6a48", "catkin": "#c8f060"}
STAGES = {"dormant": 0.0, "budding": 0.3, "leafing": 0.5, "leafed": 0.7, "full_bloom": 1.0}
# Per-instance variation the game draws from (world seed, instance ID).
PER_INSTANCE = {
    "variant": "hash(world_seed, instance_id) % variant count",
    "yaw_deg": [0, 360],
    "scale": [0.8, 1.2],
    "height_stretch": [0.9, 1.15],
    "lean_deg": [0, 6],
    "sway_phase": [0, 1],
    "foliage_value_jitter": 0.08,
    "bloom_tint": ["#c8f060", "#8cd933", "#33ffd9"],
}


class ScrubOak(Burbler):
    folder = "flora/scrub_oak"

    def __init__(self, index):
        self.index, self.seed = index, VARIANTS[index]
        Burbler.__init__(self, f"scrub_oak_{index}", kind="ScrubOak")
        self.root["kind"] = "mine"
        self.root["mine"] = "scrub_oak"
        self.root["produces"] = "lightning_bug"
        self.root["variant"] = index
        self.rng = random.Random(self.seed)
        noise.seed_set(self.seed)

        m = self.material
        self.bark = m("Bark", linear(COLORS["bark_dormant"]), roughness=0.95)
        self.leaf = m("Leaf", linear(COLORS["leaf_mature"]), roughness=0.85)
        self.dead_leaf = m("DeadLeaf", linear(COLORS["dead_leaf"]), roughness=0.95)
        bud = linear(COLORS["bud"])
        self.bud = m("Bud", bud, emission=linear("#ffb350"), strength=0.0, roughness=0.7)
        catkin = linear(COLORS["catkin"])
        self.catkin = m("Catkin", catkin, emission=catkin, strength=5.0)
        self.catkin_halo = self.translucent("CatkinHalo", catkin, 0.7, 0.025)
        self.limbs, self.tips = [], []

    # ------------------------------------------------------------ growth

    def tube(self, bm, points, radii):
        """A tapered tube along a polyline, rings kept square to the path."""
        rings, normal = [], None
        for i, (p, r) in enumerate(zip(points, radii)):
            a, b = points[max(i - 1, 0)], points[min(i + 1, len(points) - 1)]
            t = (b - a).normalized()
            if normal is None:
                normal = t.orthogonal().normalized()
            normal = (normal - t * normal.dot(t)).normalized()
            side = t.cross(normal)
            rings.append([bm.verts.new(p + r * (normal * math.cos(2 * math.pi * k / RING)
                                                + side * math.sin(2 * math.pi * k / RING)))
                          for k in range(RING)])
        for ra, rb in zip(rings, rings[1:]):
            for k in range(RING):
                bm.faces.new((ra[k], ra[(k + 1) % RING], rb[(k + 1) % RING], rb[k]))
        bm.faces.new(rings[-1])

    def grow(self, bm, start, direction, length, radius, depth, limb):
        rng, steps = self.rng, 4
        points, radii, d, p = [start.copy()], [radius], direction.copy(), start.copy()
        for i in range(steps):
            # Gnarl: a seeded wander, a pull toward the light, and a little droop on thin wood.
            wander = Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-0.4, 0.6)))
            d = (d + wander * 0.32 + Vector((0, 0, 0.04 if depth > 1 else -0.06))).normalized()
            p = p + d * length / steps
            points.append(p.copy())
            radii.append(radius * (1 - 0.35 * (i + 1) / steps))
        self.tube(bm, points, radii)
        end_radius = radii[-1]
        if depth == 0 or end_radius < 0.01:
            self.tips.append((p.copy(), d.copy(), limb))
            return
        if depth <= 2 and rng.random() < 0.6:   # inner leaf mass along older wood
            self.tips.append((points[2].copy(), d.copy(), limb, "inner"))
        for c in range(rng.choice((2, 2, 3))):
            azimuth = rng.uniform(0, 2 * math.pi) if c == 0 else azimuth + 2 * math.pi / 3 + rng.uniform(-0.4, 0.4)
            spread = math.radians(rng.uniform(34, 62))
            axis = d.orthogonal().normalized()
            axis.rotate(Quaternion(d, azimuth))
            child = d.copy()
            child.rotate(Quaternion(axis, spread))
            self.grow(bm, p, child, length * rng.uniform(0.62, 0.8), end_radius * 0.92, depth - 1, limb)

    def stems(self):
        rng = self.rng
        count = rng.choice((2, 3, 3, 4, 5))
        for n in range(count):
            azimuth = 2 * math.pi * n / count + rng.uniform(-0.5, 0.5)
            lean = math.radians(rng.uniform(16, 40))
            base = Vector((math.cos(azimuth), math.sin(azimuth), 0)) * rng.uniform(0.05, 0.22)
            direction = Vector((math.sin(lean) * math.cos(azimuth), math.sin(lean) * math.sin(azimuth),
                                math.cos(lean)))
            limb = self.node(f"Limb_{n}", base, sway_max_deg=2.5)
            bm = bmesh.new()
            self.grow(bm, Vector((0, 0, -0.08)), direction, rng.uniform(0.85, 1.35),
                      rng.uniform(0.07, 0.11), rng.choice((3, 3, 4)), limb)
            mesh = bpy.data.meshes.new(f"ScrubOak_Limb_{n}")
            bm.to_mesh(mesh)
            bm.free()
            obj = self.link(bpy.data.objects.new(f"ScrubOak_LimbWood_{n}", mesh), self.bark, limb)
            for face in obj.data.polygons:
                face.use_smooth = True
            self.limbs.append(limb)

    # ------------------------------------------------------------ crown

    def blob(self, name, radius, mat, location, parent, squash=0.72, subdiv=1):
        def fill(bm):
            bmesh.ops.create_icosphere(bm, subdivisions=subdiv, radius=radius)
            for v in bm.verts:
                v.co *= 1 + 0.28 * noise.noise(v.co * 3.1 + Vector(location) * 0.7)
                v.co.z *= squash
        obj = self.mesh_obj(name, fill, mat, parent=parent, smooth=True)
        obj.location = location
        obj.scale = [self.rng.uniform(0.8, 1.25) for _ in range(3)]
        obj.rotation_euler.z = self.rng.uniform(0, math.pi)
        return obj

    def strand(self, name, length, parent, offset):
        """A hanging catkin: a thin luminous string with a slight curl."""
        rng = self.rng
        bend = Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), 0)).normalized() * 0.03

        def fill(bm):
            pts = [Vector(offset) + bend * (i / 4) ** 2 + Vector((0, 0, -length * i / 4)) for i in range(5)]
            rings = []
            for i, p in enumerate(pts):
                r = 0.011 * (1 - 0.6 * i / 4)
                rings.append([bm.verts.new(p + Vector((r * math.cos(a), r * math.sin(a), 0)))
                              for a in (k * math.pi / 2 for k in range(4))])
            for ra, rb in zip(rings, rings[1:]):
                for k in range(4):
                    bm.faces.new((ra[k], ra[(k + 1) % 4], rb[(k + 1) % 4], rb[k]))
        return self.mesh_obj(name, fill, self.catkin, parent=parent)

    def crown(self):
        rng = self.rng
        blooms = []
        for n, tip in enumerate(self.tips):
            p, d, limb = tip[:3]
            inner = len(tip) > 3
            foliage = self.node(f"Foliage_{n}", p, limb)
            r = rng.uniform(0.2, 0.3) if inner else rng.uniform(0.26, 0.4)
            self.blob(f"Foliage_{n}_Leaves", r, self.leaf, (0, 0, 0.04), foliage)
            if inner:
                continue
            # Winter: buds on every outer tip; dead leaves still clinging on some
            # (oaks hold their dry leaves through the winter).
            bud = self.node(f"Bud_{n}", p, limb)
            self.sphere(f"Bud_{n}_Bud", 0.03, self.bud, (0, 0, 0.01), bud, 6, 3).scale = (1, 1, 1.6)
            if rng.random() < 0.35:
                dead = self.node(f"Dead_{n}", p + Vector((0, 0, -0.06)), limb)
                self.blob(f"Dead_{n}_Leaves", rng.uniform(0.07, 0.11), self.dead_leaf, (0, 0, 0), dead,
                          squash=0.5, subdiv=0)
            if rng.random() < 0.55:
                out = Vector((d.x, d.y, 0)).normalized() if d.xy.length > 1e-3 else Vector((1, 0, 0))
                blooms.append((p + out * r * 0.9 + Vector((0, 0, -r * 0.45)), limb))
        rng.shuffle(blooms)
        blooms = blooms[:MAX_BLOOMS]
        for order, (p, limb) in enumerate(blooms):
            bloom = self.node(f"Bloom_{order}", p, limb, order=order)
            for k in range(3):   # a tassel of hanging catkins
                a = 2 * math.pi * k / 3 + rng.uniform(-0.4, 0.4)
                self.strand(f"Bloom_{order}_Catkin{k}", rng.uniform(0.12, 0.22), bloom,
                            (math.cos(a) * 0.035, math.sin(a) * 0.035, 0))
            self.sphere(f"Bloom_{order}_Halo", 0.16, self.catkin_halo, (0, 0, -0.1), bloom, 10, 5).scale = (1, 1, 1.4)
            self.node(f"BugSpawn_{order}", (0, 0, -0.1), bloom, emits="lightning_bug")
        self.bloom_count = len(blooms)

    # ------------------------------------------------------------ far LOD

    def far_lod(self):
        """Joined, decimated copies of the wood and the leaves."""
        self.scene.view_layers[0].update()
        made = []
        for name, prefix, ratio in (("LOD_Far_Bark", "ScrubOak_LimbWood_", 0.12),
                                    ("LOD_Far_Foliage", "ScrubOak_Foliage_", 0.14)):
            parts = [o for o in self.collection.objects if o.name.startswith(prefix) and o.type == "MESH"]
            bm = bmesh.new()
            for obj in parts:
                mesh = obj.data.copy()
                mesh.transform(obj.matrix_world)
                bm.from_mesh(mesh)
                bpy.data.meshes.remove(mesh)
            mesh = bpy.data.meshes.new(f"ScrubOak_{name}")
            bm.to_mesh(mesh)
            bm.free()
            obj = self.link(bpy.data.objects.new(f"ScrubOak_{name}", mesh),
                            self.bark if "Bark" in name else self.leaf)
            obj["lod"] = "far"
            mod = obj.modifiers.new("Decimate", "DECIMATE")
            mod.ratio = ratio
            made.append(obj)
        return made

    # ------------------------------------------------------------ growth: the game's mapping

    def set_growth(self, g):
        """Pose the tree at growth g (0 dormant .. 1 full bloom) exactly as GROWTH tells
        the game to; the previews are rendered through this."""
        def scale_nodes(prefix, s):
            for obj in self.collection.objects:
                if obj.type == "EMPTY" and obj.name.startswith(prefix) and obj.name[len(prefix):].isdigit():
                    obj.scale = (s, s, s)
                    for child in obj.children_recursive:
                        child.hide_render = s <= 1e-3
        scale_nodes("Dead_", 1 - ramp(g, *GROWTH["dead_leaves_fall"]))
        swell = ramp(g, *GROWTH["buds_swell"])
        scale_nodes("Bud_", (0.4 + 0.6 * swell) * (1 - ramp(g, *GROWTH["buds_open"])))
        scale_nodes("Foliage_", ramp(g, *GROWTH["leaf_out"]))
        start, end = GROWTH["bloom"]
        for k in range(self.bloom_count):
            opens = start + (end - start) * k / max(self.bloom_count, 1)
            s = ramp(g, opens, opens + GROWTH["bloom_open_span"])
            node = bpy.data.objects[f"Bloom_{k}"]
            node.scale = (s, s, s)
            for child in node.children_recursive:
                child.hide_render = s <= 1e-3
        mix = ramp(g, *GROWTH["bark_wakes"])
        set_color(self.bark, lerp_hex(COLORS["bark_dormant"], COLORS["bark_alive"], mix))
        mix = ramp(g, *GROWTH["leaf_matures"])
        set_color(self.leaf, lerp_hex(COLORS["leaf_young"], COLORS["leaf_mature"], mix))
        bud = self.bud.node_tree.nodes["Principled BSDF"]
        bud.inputs["Emission Strength"].default_value = 1.5 * swell

    def triangles(self, far=False):
        total = 0
        depsgraph = self.scene.view_layers[0].depsgraph
        depsgraph.update()
        for obj in self.collection.objects:
            if obj.type != "MESH" or (obj.get("lod") == "far") != far:
                continue
            mesh = obj.evaluated_get(depsgraph).to_mesh()
            total += sum(len(p.vertices) - 2 for p in mesh.polygons)
            obj.evaluated_get(depsgraph).to_mesh_clear()
        return total

    def profile(self, near, far):
        return {
            "id": self.asset_id,
            "model": f"{self.model_ref}#Scene0",
            "kind": "mine",
            "mine": "scrub_oak",
            "produces": "lightning_bug",
            "variant": self.index,
            "seed": self.seed,
            "limbs": len(self.limbs),
            "blooms": self.bloom_count,
            "triangles": {"near": near, "far": far},
        }

    def build(self):
        self.stems()
        self.crown()
        lod = self.far_lod()
        near, far = self.triangles(), self.triangles(far=True)
        # Dormant: no leaves, far or near. The scale sits on an empty above the mesh:
        # a mesh object at scale 0 crashes Blender 5.3's glTF exporter.
        foliage = next(obj for obj in lod if "Foliage" in obj.name)
        holder = self.node("ScrubOak_LOD_Far_Foliage_Node", (0, 0, 0), leaf_out=True)
        foliage.parent = holder
        holder.scale = (0, 0, 0)
        self.export_lod(lod + [holder])
        self.set_growth(0.0)   # ships dormant: the opening is winter

        def stage():
            self.stage_preview(resolution=(520, 720))
        report = self.finish(self.profile(near, far), aim=(0, 0, 1.35), camera_location=(2.7, -8.8, 2.5),
                             before_render=stage)
        if bpy.app.background:
            out = OUT / self.asset_id
            for name, g in STAGES.items():
                self.set_growth(g)
                self.scene.render.filepath = str(out / f"preview_{name}.png")
                bpy.ops.render.render(write_still=True, scene=self.scene.name)
        report.update(near=near, far=far, blooms=self.bloom_count)
        return report


def ramp(x, a, b):
    """0 below a, 1 above b, smoothstep between."""
    t = min(max((x - a) / (b - a), 0.0), 1.0)
    return t * t * (3 - 2 * t)


def lerp_hex(a, b, t):
    ca, cb = linear(a), linear(b)
    return tuple(x + (y - x) * t for x, y in zip(ca, cb))


def set_color(mat, rgb):
    mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (*rgb, 1.0)


def write_profile():
    profiles = [json.loads((OUT / f"scrub_oak_{k}/asset.json").read_text()) for k in range(len(VARIANTS))]
    profile = {
        "id": "scrub_oak",
        "kind": "mine",
        "mine": "scrub_oak",
        "produces": "lightning_bug",
        "variants": [p["model"] for p in profiles],
        "per_instance": PER_INSTANCE,
        "growth": {
            "value": "0 dormant (winter) .. 1 full bloom",
            "driver": "clean water delivered to the tree: it rises while clean water arrives "
                      "and eases back without it; muddy water does not wake it",
            "starts": 0.0,
            "ramps": {k: v for k, v in GROWTH.items()},
            "ramp": "smoothstep over [from, to] of growth",
            "nodes": {
                "Dead_<n>": "scale 1 - ramp(dead_leaves_fall)",
                "Bud_<n>": "scale (0.4 + 0.6 ramp(buds_swell)) * (1 - ramp(buds_open)); "
                           "ScrubOak_Bud emission 1.5 * ramp(buds_swell)",
                "Foliage_<n>": "scale ramp(leaf_out)",
                "Bloom_<k>": "opens over bloom_open_span starting at bloom.from + (bloom.to - bloom.from) * k / count",
            },
            "colors": COLORS,
            "materials": {"ScrubOak_Bark": "bark_dormant -> bark_alive by ramp(bark_wakes)",
                          "ScrubOak_Leaf": "leaf_young -> leaf_mature by ramp(leaf_matures)"},
            "preview_stages": STAGES,
        },
        "sway": {"nodes": "Limb_<n>", "pivot": "node origin (stem base)", "max_deg": 2.5,
                 "phase": "per_instance.sway_phase, plus the limb index"},
        "bug_spawn": "BugSpawn_<k>, one per bloom; only open blooms spawn, so a dormant tree makes no bugs",
        "far_lod": {"distance_m": FAR_LOD_M, "file": "models/flora/scrub_oak/scrub_oak_<k>_LOD1.glb",
                    "nodes": ["ScrubOak_LOD_Far_Bark", "ScrubOak_LOD_Far_Foliage_Node"],
                    "note": "Jackdaw LodGroup level 1. ScrubOak_LOD_Far_Foliage_Node ships at scale 0 and scales "
                            "with leaf_out; buds, dead leaves and blooms are not drawn far"},
        "materials": {"ScrubOak_Leaf": "also jittered by per_instance.foliage_value_jitter",
                      "ScrubOak_Catkin": "tinted by per_instance.bloom_tint",
                      "ScrubOak_CatkinHalo": "tinted by per_instance.bloom_tint"},
        "triangles": {p["id"]: p["triangles"] for p in profiles},
    }
    (OUT / "asset.json").write_text(json.dumps(profile, indent=2) + "\n")


def montage():
    magick = shutil.which("magick")
    if not magick:
        return

    def tile(files, labels, cols, dest):
        cmd = [magick, "montage"]
        for f, label in zip(files, labels):
            cmd += ["-label", label, str(f)]
        cmd += ["-tile", f"{cols}x1", "-geometry", "+4+4", "-background", "#15161a", "-fill", "#d8d4cc",
                "-pointsize", "18", str(dest)]
        subprocess.run(cmd, check=True)
    for stage in ("dormant", "full_bloom"):
        tile([OUT / f"scrub_oak_{k}/preview_{stage}.png" for k in range(len(VARIANTS))],
             [f"variant {k}" for k in range(len(VARIANTS))], len(VARIANTS), OUT / f"lineup_{stage}.png")
    tile([OUT / f"scrub_oak_0/preview_{s}.png" for s in STAGES],
         [f"{s.replace('_', ' ')} ({g:g})" for s, g in STAGES.items()], len(STAGES), OUT / "growth.png")
    # Keep the repo light: variant 0 keeps every preview; the others keep the two the
    # lineups use (the rest are rendered again whenever the generator runs).
    for k in range(1, len(VARIANTS)):
        for name in ("preview.png", "preview_far.png", "preview_budding.png", "preview_leafing.png",
                     "preview_leafed.png"):
            (OUT / f"scrub_oak_{k}" / name).unlink(missing_ok=True)


if __name__ == "__main__":
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if args == ["montage"]:
        write_profile()
        montage()
    elif args:
        print(ScrubOak(int(args[0])).build())
    else:
        for k in range(len(VARIANTS)):
            subprocess.run([bpy.app.binary_path, "-b", "--factory-startup", "-P", __file__, "--", str(k)],
                           check=True)
        write_profile()
        montage()
