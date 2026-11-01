"""Desert props for the bleak opening, each with its bloomed counterpart
(add-art-direction 2.6).

Three families scatter the cracked mudflat:

  desert_brush_<k>     dead brush: a low, grey, twiggy shrub. Bloomed, it leafs out
                       in small clusters and opens a few luminous flowers
  desert_stump_<k>     a bone-dry stump: a bleached snag with a jagged broken top and
                       flared roots. Bloomed, its wood darkens with damp, moss grows on
                       it and luminous mushrooms cluster at its foot
  desert_debris_<k>    cracked-mud debris: a scatter of curled, broken mud plates.
                       Bloomed, sprouts push up between the plates

One value per prop, **bloom** (0 bleak .. 1 bloomed), follows the local light quality,
the same field that blends the world from bleak to bloomed. Bleak is the authored
look; everything that blooms is a `Bloom_<n>` node the game scales in, each over its
own slice of the range (extras `from`, `to`), so a prop blooms gradually, not at once.
Materials listed in the profile blend from their bleak to their bloomed colour.

Variation: seeded variants here; per-instance scale, yaw and tilt in the game.

Reproduce: blender -b --factory-startup -P tools/blender/create_desert_props.py
           (`-- brush 1` for one prop, `-- montage` for the sheets)
Writes assets/models/desert_<family>_<k>.glb and assets/props/desert/.
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
from mathutils import Quaternion, Vector, noise

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_burbler import Burbler, linear  # noqa: E402
from create_reeds import ReedClump  # noqa: E402
from module_kit import PROJECT  # noqa: E402

OUT = PROJECT / "assets/props/desert"
FAMILIES = {"brush": [3, 14, 27], "stump": [8, 19, 33], "debris": [6, 21, 40]}
# bleak -> bloomed colours (sRGB) for the materials the game blends
BLEND = {
    "Twig": ("#7d7468", "#4f4234"),
    "Wood": ("#8f897d", "#4f4134"),
    "MudPlate": ("#5a4433", "#3e2f22"),
}
FIXED = {
    "Leaf": "#4f6a2a", "Flower": "#ff4dd9", "Moss": "#3f5a24", "MushroomStem": "#d9d2c0",
    "MushroomCap": "#33ffd9", "Sprout": "#6f9a34", "Heartwood": "#8a7a62",
}
PER_INSTANCE = {"variant": "hash(world_seed, instance_id) % variant count", "yaw_deg": [0, 360],
                "scale": [0.8, 1.25], "tilt_deg": [0, 5]}


def ramp(x, a, b):
    t = min(max((x - a) / (b - a), 0.0), 1.0)
    return t * t * (3 - 2 * t)


class Prop(Burbler):
    def __init__(self, family, index):
        self.family, self.index = family, index
        self.seed = FAMILIES[family][index]
        Burbler.__init__(self, f"desert_{family}_{index}", kind=f"Desert{family.capitalize()}")
        for key in ("mine", "produces"):
            del self.root[key]
        self.root["kind"] = "prop"
        self.root["family"] = family
        self.root["variant"] = index
        self.rng = random.Random(self.seed)
        noise.seed_set(self.seed)
        self.blooms = []
        m = self.material
        self.mats = {name: m(name, linear(bleak), roughness=0.92) for name, (bleak, _) in BLEND.items()}
        self.leaf = m("Leaf", linear(FIXED["Leaf"]), roughness=0.8)
        self.moss = m("Moss", linear(FIXED["Moss"]), roughness=1.0)
        self.sprout = m("Sprout", linear(FIXED["Sprout"]), roughness=0.75)
        self.heartwood = m("Heartwood", linear(FIXED["Heartwood"]), roughness=0.95)
        self.stem = m("MushroomStem", linear(FIXED["MushroomStem"]), roughness=0.7)
        for name in ("Flower", "MushroomCap"):
            c = linear(FIXED[name])
            setattr(self, name.lower(), m(name, c, emission=c, strength=4.0))
            setattr(self, name.lower() + "_halo", self.translucent(name + "Halo", c, 0.8, 0.05))

    def bloom_node(self, location, parent=None, start=0.3, span=0.4):
        a = start + self.rng.uniform(0, 1 - span - start) if 1 - span > start else start
        node = self.node(f"Bloom_{len(self.blooms)}", location, parent, **{"from": round(a, 3),
                                                                           "to": round(a + span, 3)})
        self.blooms.append(node)
        return node

    @staticmethod
    def tube(bm, points, radii, sides=5):
        rings, normal = [], None
        for i, (p, r) in enumerate(zip(points, radii)):
            a, b = points[max(i - 1, 0)], points[min(i + 1, len(points) - 1)]
            t = (b - a).normalized()
            normal = t.orthogonal().normalized() if normal is None else (normal - t * normal.dot(t)).normalized()
            side = t.cross(normal)
            rings.append([bm.verts.new(p + r * (normal * math.cos(2 * math.pi * k / sides)
                                                + side * math.sin(2 * math.pi * k / sides))) for k in range(sides)])
        for ra, rb in zip(rings, rings[1:]):
            for k in range(sides):
                bm.faces.new((ra[k], ra[(k + 1) % sides], rb[(k + 1) % sides], rb[k]))
        return rings

    def blob(self, name, radius, mat, location, parent, squash=0.7, subdiv=1):
        def fill(bm):
            bmesh.ops.create_icosphere(bm, subdivisions=subdiv, radius=radius)
            for v in bm.verts:
                v.co *= 1 + 0.25 * noise.noise(v.co * 4.0 + Vector(location) * 0.9)
                v.co.z *= squash
        obj = self.mesh_obj(name, fill, mat, parent=parent)
        obj.location = location
        return obj

    def lumen(self, name, mat, halo, location, parent, r=0.025):
        self.sphere(f"{name}_Core", r, mat, location, parent, 6, 3)
        self.sphere(f"{name}_Halo", r * 3.2, halo, location, parent, 8, 4)

    # ------------------------------------------------------------ dead brush

    def brush(self):
        rng, tips = self.rng, []
        bm = bmesh.new()

        def grow(p, d, length, r, depth):
            pts, radii = [p.copy()], [r]
            for i in range(3):
                d = (d + Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-0.2, 0.5))) * 0.4).normalized()
                p = p + d * length / 3
                pts.append(p.copy())
                radii.append(r * (1 - 0.4 * (i + 1) / 3))
            self.tube(bm, pts, radii, sides=4)
            if depth == 0:
                tips.append(p.copy())
                return
            for c in range(rng.choice((2, 3))):
                axis = d.orthogonal().normalized()
                axis.rotate(Quaternion(d, rng.uniform(0, 2 * math.pi)))
                child = d.copy()
                child.rotate(Quaternion(axis, math.radians(rng.uniform(25, 55))))
                grow(p, child, length * rng.uniform(0.6, 0.8), radii[-1] * 0.85, depth - 1)
        for n in range(rng.randint(3, 5)):
            a = rng.uniform(0, 2 * math.pi)
            lean = math.radians(rng.uniform(25, 60))
            d = Vector((math.sin(lean) * math.cos(a), math.sin(lean) * math.sin(a), math.cos(lean)))
            grow(Vector((0, 0, -0.02)), d, rng.uniform(0.22, 0.38), rng.uniform(0.012, 0.02), 3)
        mesh = bpy.data.meshes.new("DesertBrush_Twigs")
        bm.to_mesh(mesh)
        bm.free()
        obj = self.link(bpy.data.objects.new("DesertBrush_Twigs", mesh), self.mats["Twig"])
        for face in obj.data.polygons:
            face.use_smooth = True
        rng.shuffle(tips)
        for k, p in enumerate(tips[:26]):
            node = self.bloom_node(p, start=0.2, span=0.35)
            self.blob(f"Leaves_{k}", rng.uniform(0.03, 0.05), self.leaf, (0, 0, 0), node, subdiv=1)
            if k % 4 == 0:
                self.lumen(f"Flower_{k}", self.flower, self.flower_halo, (0, 0, 0.06), node, 0.016)
        return (0.0, 0.0, 0.3), (1.0, -1.9, 0.8)

    # ------------------------------------------------------------ stump

    def stump(self):
        rng = self.rng
        height, radius = rng.uniform(0.45, 0.95), rng.uniform(0.13, 0.22)
        rings_n, sides = 8, 16
        lean = Vector((rng.uniform(-0.12, 0.12), rng.uniform(-0.12, 0.12), 0))

        def fill(bm):
            rings = []
            for i in range(rings_n + 1):
                t = i / rings_n
                ring = []
                for k in range(sides):
                    a = 2 * math.pi * k / sides
                    flare = 1 + 0.9 * (1 - t) ** 4 * (0.6 + 0.4 * math.cos(3 * a + self.seed))
                    groove = 1 + 0.1 * math.sin(9 * a + 4 * t) + 0.05 * noise.noise(Vector((a * 2, t * 3, self.seed)))
                    r = radius * flare * groove * (1 - 0.15 * t)
                    z = height * t
                    if i == rings_n:   # a jagged, split top where it snapped
                        z += height * (0.12 + 0.4 * abs(noise.noise(Vector((math.cos(a) * 3, math.sin(a) * 3, self.seed))))) * (1.6 if k % 4 == 0 else 1)
                    ring.append(bm.verts.new(Vector((r * math.cos(a), r * math.sin(a), z - 0.03)) + lean * t))
                rings.append(ring)
            for ra, rb in zip(rings, rings[1:]):
                for k in range(sides):
                    bm.faces.new((ra[k], ra[(k + 1) % sides], rb[(k + 1) % sides], rb[k]))
            top = bm.faces.new(rings[-1])
            top.material_index = 1
        obj = self.mesh_obj("Trunk", fill, self.mats["Wood"])
        obj.data.materials.append(self.heartwood)
        if rng.random() < 0.7:   # a broken-off branch stub
            bm = bmesh.new()
            z = height * rng.uniform(0.4, 0.75)
            a = rng.uniform(0, 2 * math.pi)
            d = Vector((math.cos(a), math.sin(a), rng.uniform(0.2, 0.7))).normalized()
            base = Vector((math.cos(a) * radius * 0.7, math.sin(a) * radius * 0.7, z)) + lean * (z / height)
            self.tube(bm, [base, base + d * 0.14, base + d * 0.24], [radius * 0.3, radius * 0.24, radius * 0.16])
            mesh = bpy.data.meshes.new("DesertStump_Branch")
            bm.to_mesh(mesh)
            bm.free()
            self.link(bpy.data.objects.new("DesertStump_Branch", mesh), self.mats["Wood"])
        # Bloomed: moss on the trunk's shaded side, mushrooms around the foot.
        for k in range(rng.randint(3, 5)):
            a, z = rng.uniform(0.6, 2.6), rng.uniform(0.05, height * 0.7)
            r = radius * (1 + 0.9 * (1 - z / height) ** 4) * 0.98
            node = self.bloom_node((math.cos(a) * r, math.sin(a) * r, z), start=0.15, span=0.4)
            self.blob(f"Moss_{k}", rng.uniform(0.06, 0.11), self.moss, (0, 0, 0), node, squash=0.35)
        for k in range(rng.randint(4, 7)):
            a = rng.uniform(0, 2 * math.pi)
            d = radius * rng.uniform(1.6, 2.6)
            node = self.bloom_node((math.cos(a) * d, math.sin(a) * d, 0), start=0.45, span=0.35)
            h = rng.uniform(0.05, 0.14)
            stem = self.sphere(f"Mushroom_{k}_Stem", 0.012, self.stem, (0, 0, h / 2), node, 6, 3)
            stem.scale = (1, 1, h / 0.024)
            cap = self.sphere(f"Mushroom_{k}_Cap", rng.uniform(0.03, 0.05), self.mushroomcap, (0, 0, h), node, 10, 5)
            cap.scale = (1, 1, 0.45)
            self.sphere(f"Mushroom_{k}_Halo", 0.065, self.mushroomcap_halo, (0, 0, h), node, 10, 5).scale = (1, 1, 0.7)
        return (0.0, 0.0, height * 0.45), (1.1, -2.1, 0.95)

    # ------------------------------------------------------------ cracked-mud debris

    def debris(self):
        rng = self.rng
        bm = bmesh.new()
        plates = []
        for k in range(rng.randint(5, 9)):
            c = Vector((rng.uniform(-0.55, 0.55), rng.uniform(-0.55, 0.55), 0))
            if any((c - q).length < 0.22 for q in plates):
                continue
            plates.append(c)
            n, size = rng.randint(5, 7), rng.uniform(0.1, 0.2)
            curl, thick = rng.uniform(1.0, 3.2), rng.uniform(0.02, 0.035)
            angles = sorted(rng.uniform(0, 2 * math.pi) for _ in range(n))
            outline = [Vector((math.cos(a), math.sin(a), 0)) * size * rng.uniform(0.7, 1.15) for a in angles]
            # Curl: edges lift as the plate dries and cups.
            top = [bm.verts.new(c + v + Vector((0, 0, thick + curl * v.length ** 2))) for v in outline]
            bot = [bm.verts.new(c + v * 0.97 + Vector((0, 0, curl * v.length ** 2))) for v in outline]
            mid_t = bm.verts.new(c + Vector((0, 0, thick)))
            for i in range(n):
                j = (i + 1) % n
                bm.faces.new((mid_t, top[i], top[j]))
                bm.faces.new((top[j], top[i], bot[i], bot[j]))
        mesh = bpy.data.meshes.new("DesertDebris_Plates")
        bm.to_mesh(mesh)
        bm.free()
        self.link(bpy.data.objects.new("DesertDebris_Plates", mesh), self.mats["MudPlate"])
        # Bloomed: sprouts push up through the gaps.
        for k in range(rng.randint(5, 9)):
            p = Vector((rng.uniform(-0.6, 0.6), rng.uniform(-0.6, 0.6), 0))
            if any((p - q).length < 0.12 for q in plates):
                p += (p - min(plates, key=lambda q: (p - q).length)).normalized() * 0.12
            node = self.bloom_node(p, start=0.2, span=0.45)
            sb = bmesh.new()
            for b in range(rng.randint(2, 4)):
                a = rng.uniform(0, 2 * math.pi)
                out = Vector((math.cos(a), math.sin(a), 0))
                h = rng.uniform(0.08, 0.2)
                ReedClump.blade_into(sb, ReedClump.path(Vector(), out * 0.25, h, out * h * 0.4, n=3),
                                     rng.uniform(0.015, 0.025))
            mesh = bpy.data.meshes.new(f"DesertDebris_Sprout_{k}")
            sb.to_mesh(mesh)
            sb.free()
            self.link(bpy.data.objects.new(f"DesertDebris_Sprout_{k}", mesh), self.sprout, node)
        return (0.0, 0.0, 0.05), (0.9, -1.8, 1.1)

    # ------------------------------------------------------------ output

    def set_bloom(self, b):
        for node in self.blooms:
            s = ramp(b, node["from"], node["to"])
            node.scale = (s, s, s)
            for child in node.children_recursive:
                child.hide_render = s <= 1e-3
        for name, (bleak, bloomed) in BLEND.items():
            c = [x + (y - x) * b for x, y in zip(linear(bleak), linear(bloomed))]
            self.mats[name].node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (*c, 1.0)

    def triangles(self):
        return sum(sum(len(p.vertices) - 2 for p in o.data.polygons)
                   for o in self.collection.objects if o.type == "MESH")

    def build(self):
        aim, cam = getattr(self, self.family)()
        tris = self.triangles()
        self.set_bloom(0.0)   # ships bleak
        profile = {"id": self.asset_id, "model": f"models/{self.asset_id}.glb#Scene0", "kind": "prop",
                   "family": self.family, "variant": self.index, "seed": self.seed,
                   "blooms": len(self.blooms), "triangles": tris}

        def stage():
            self.stage_preview(resolution=(420, 420))
        report = self.finish(profile, aim=aim, camera_location=cam, before_render=stage,
                             profile_dir="props/desert")
        if bpy.app.background:
            for name, b in (("bleak", 0.0), ("bloomed", 1.0)):
                self.set_bloom(b)
                self.scene.render.filepath = str(OUT / self.asset_id / f"preview_{name}.png")
                bpy.ops.render.render(write_still=True, scene=self.scene.name)
        report.update(triangles=tris)
        return report


def assets():
    return [(f, k) for f, seeds in FAMILIES.items() for k in range(len(seeds))]


def write_profile():
    profiles = {f"desert_{f}_{k}": json.loads((OUT / f"desert_{f}_{k}/asset.json").read_text()) for f, k in assets()}
    profile = {
        "id": "desert_props",
        "kind": "prop",
        "families": {f: [f"models/desert_{f}_{k}.glb#Scene0" for k in range(len(s))] for f, s in FAMILIES.items()},
        "per_instance": PER_INSTANCE,
        "bloom": {
            "value": "0 bleak .. 1 bloomed; follow the local light quality",
            "nodes": "Bloom_<n>: scale smoothstep(bloom, from, to), from and to in each node's extras",
            "materials": {f"Desert{f.capitalize()}_{name}": list(BLEND[name]) for f, name in
                          (("brush", "Twig"), ("stump", "Wood"), ("debris", "MudPlate"))},
            "blend": "material base colour lerps bleak -> bloomed with bloom",
        },
        "triangles": {k: v["triangles"] for k, v in profiles.items()},
    }
    (OUT / "asset.json").write_text(json.dumps(profile, indent=2) + "\n")


def montage():
    magick = shutil.which("magick")
    if not magick:
        return
    for name in ("bleak", "bloomed"):
        cmd = [magick, "montage"]
        for f, k in assets():
            cmd += ["-label", f"{f} {k}", str(OUT / f"desert_{f}_{k}/preview_{name}.png")]
        cmd += ["-tile", "3x3", "-geometry", "+4+4", "-background", "#15161a", "-fill", "#d8d4cc",
                "-pointsize", "16", str(OUT / f"sheet_{name}.png")]
        subprocess.run(cmd, check=True)
    for f, k in assets():   # keep the repo light: sheets carry the variants
        if k:
            for n in ("preview.png", "preview_far.png"):
                (OUT / f"desert_{f}_{k}" / n).unlink(missing_ok=True)
    for path in OUT.glob("*/*.png"):
        subprocess.run([magick, str(path), "-strip", "-define", "png:compression-level=9", str(path)], check=True)


if __name__ == "__main__":
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if args == ["montage"]:
        write_profile()
        montage()
    elif args:
        print(Prop(args[0], int(args[1])).build())
    else:
        for f, k in assets():
            subprocess.run([bpy.app.binary_path, "-b", "--factory-startup", "-P", __file__, "--", f, str(k)],
                           check=True)
        write_profile()
        montage()
