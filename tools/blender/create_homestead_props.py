"""The homestead prop kit: the furniture and gear around a Dancer cottage.

The cottage's brief (`docs/art/cottage.md`) keeps these out of the cottage model so its
porch can be dressed or left bare; the concept sheet shows them on and around the porch.
Same register as the cottage: salvaged timber and grown wood carry the weight, iron and
titanium do the precise work, joins showing.

  homestead_table     a trestle table of salvaged planks on grown-wood legs
  homestead_bench     a split-log bench on two stumps
  homestead_stool     a stump stool with a rope-wrapped seat
  homestead_barrel    a timber barrel with rusted iron hoops
  homestead_bucket    a wooden bucket with an iron hoop and a rope handle
  homestead_planter   a half-barrel planter of leafy greens
  homestead_hammock   a canvas hammock slung between two grown-wood posts

Each prop is its own placeable asset with a near and a far level (`<id>_LOD0`, `_LOD1`).
Every material declares an acoustic class (`docs/audio/acoustic-level-authoring.md`);
these are small clutter, so no mesh carries the tag (the guide: leave small props out).

Reproduce: blender -b --factory-startup -P tools/blender/create_homestead_props.py
           (`-- table` for one prop, `-- montage` for the sheet)
Writes assets/props/homestead/homestead_<name>/, assets/props/homestead/asset.json and
previews/props/homestead/ (one preview each and sheet.png).
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
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_burbler import Burbler, linear  # noqa: E402
from create_cottage import Cottage  # noqa: E402
from module_kit import PROJECT  # noqa: E402

PROPS = ("table", "bench", "stool", "barrel", "bucket", "planter", "hammock")
OUT = PROJECT / "assets/props/homestead"
PREVIEW_OUT = PROJECT / "previews/props/homestead"


class HomesteadProp(Burbler):
    folder = "props/homestead"
    box = Cottage.box
    tube = staticmethod(Cottage.tube)
    pipe = Cottage.pipe

    def __init__(self, name):
        self.name = name
        Burbler.__init__(self, f"homestead_{name}", kind=f"Homestead{name.capitalize()}")
        for key in ("mine", "produces"):
            del self.root[key]
        self.root["kind"] = "prop"
        self.root["family"] = "homestead"
        self.rng = random.Random(sum(map(ord, name)) * 7)
        m = self.material
        self.timber = m("Timber", linear("#5a4a3a"), roughness=0.9)
        self.cut = m("CutWood", linear("#8a7a62"), roughness=0.9)      # heartwood: sawn faces
        self.grown = m("GrownWood", linear("#4b4037"), roughness=0.85)
        self.rust = m("RustIron", linear("#5e3a22"), metallic=0.5, roughness=0.85)
        self.rope = m("Rope", linear("#8a7552"), roughness=1.0)        # reed blade dry
        self.canvas = m("Canvas", linear("#cfc6b2"), roughness=0.95)    # bone
        self.soil = m("Soil", linear("#302116"), roughness=1.0)
        self.leaf = m("Leaf", linear("#4f6a2a"), roughness=0.8)
        self.sprout = m("Sprout", linear("#6f9a34"), roughness=0.75)
        self.acoustics = {self.timber: "wood", self.cut: "wood", self.grown: "wood",
                          self.rust: "metal", self.rope: "carpet", self.canvas: "carpet",
                          self.soil: "gravel", self.leaf: None, self.sprout: None}

    # ------------------------------------------------------------ shapes

    def staves(self, name, radius_at, height, mat, count=14, bulge=0.0):
        """A ring of staves: a barrel or bucket wall, radius_at(t) at height t*height."""
        def fill(bm):
            rings = []
            for i in range(7):
                t = i / 6
                r = radius_at(t) + bulge * math.sin(math.pi * t)
                rings.append([bm.verts.new((math.cos(a) * r, math.sin(a) * r, t * height))
                              for a in (math.tau * k / count for k in range(count))])
            for a, b in zip(rings, rings[1:]):
                for k in range(count):
                    bm.faces.new((a[k], a[(k + 1) % count], b[(k + 1) % count], b[k]))
            bm.faces.new(rings[0][::-1])
        return self.mesh_obj(name, fill, mat, smooth=False)

    def disc(self, name, radius, z, mat, sides=14):
        def fill(bm):
            ring = [bm.verts.new((math.cos(a) * radius, math.sin(a) * radius, z))
                    for a in (math.tau * k / sides for k in range(sides))]
            bm.faces.new(ring)
        return self.mesh_obj(name, fill, mat, smooth=False)

    def hoop(self, name, radius, z, mat, thickness=0.02):
        pts = [(math.cos(a) * radius, math.sin(a) * radius, z) for a in (math.tau * k / 16 for k in range(17))]
        return self.mesh_obj(name, lambda bm: self.tube(bm, pts, [thickness] * 17, 4, cap=False), mat, smooth=False)

    def leaves(self, name, centre, spread, count, mat, height=0.35):
        def fill(bm):
            for _ in range(count):
                a = self.rng.uniform(0, math.tau)
                base = Vector(centre) + Vector((math.cos(a), math.sin(a), 0)) * self.rng.uniform(0, spread)
                d = Vector((math.cos(a), math.sin(a), 0))
                w = Vector((-d.y, d.x, 0)) * 0.05
                tip = base + d * self.rng.uniform(0.08, 0.2) + Vector((0, 0, self.rng.uniform(0.6, 1.0) * height))
                mid = base.lerp(tip, 0.5) + Vector((0, 0, 0.03))
                v = [bm.verts.new(p) for p in (base, mid + w * 1.6, tip, mid - w * 1.6)]
                bm.faces.new(v)
        return self.mesh_obj(name, fill, mat, smooth=False)

    # ------------------------------------------------------------ the props
    # Each returns (far objects, preview aim, preview camera).

    def table(self):
        L, W, H = 1.7, 0.85, 0.78
        for k in range(5):
            w = W / 5
            self.box(f"Plank{k}", (0, -W / 2 + w * (k + 0.5), H - 0.03), (L, w - 0.015, 0.06), self.cut)
        for sx in (-1, 1):
            x = sx * (L / 2 - 0.25)
            for sy in (-1, 1):
                foot = (x + sx * 0.08, sy * (W / 2 - 0.05), 0.0)
                self.pipe(f"Leg{sx}{sy}", [foot, (x, sy * (W / 2 - 0.12), H - 0.06)], 0.045, self.grown, 6)
            self.box(f"Cleat{sx}", (x, 0, H - 0.09), (0.09, W - 0.05, 0.06), self.timber)
            self.box(f"Strap{sx}", (x, 0, H - 0.04), (0.11, W + 0.01, 0.025), self.rust)
        self.pipe("Stretcher", [(-L / 2 + 0.25, 0, 0.3), (L / 2 - 0.25, 0, 0.3)], 0.035, self.grown, 6)
        far = [self.box("FarTop", (0, 0, H - 0.03), (L, W, 0.06), self.cut),
               self.box("FarLegs", (0, 0, (H - 0.06) / 2), (L - 0.4, 0.1, H - 0.06), self.grown)]
        return far, (0, 0, 0.45), (1.9, -2.3, 1.5)

    def bench(self):
        L, H = 1.5, 0.45
        def half_log(bm):
            for i in range(2):
                pass
            pts = []
            n = 9
            prof = [(math.cos(math.pi * k / (n - 1)) * 0.17, -math.sin(math.pi * k / (n - 1)) * 0.11) for k in range(n)]
            ends = []
            for x in (-L / 2, L / 2):
                ends.append([bm.verts.new((x, py, H + pz)) for py, pz in prof])
            for k in range(n - 1):
                bm.faces.new((ends[0][k], ends[0][k + 1], ends[1][k + 1], ends[1][k]))
            bm.faces.new((ends[0][0], ends[1][0], ends[1][-1], ends[0][-1]))
            bm.faces.new(ends[0])
            bm.faces.new(ends[1][::-1])
        self.mesh_obj("Seat", half_log, self.grown, smooth=False)
        self.box("SeatTop", (0, 0, H + 0.005), (L - 0.02, 0.32, 0.01), self.cut)
        for sx in (-1, 1):
            self.staves(f"Stump{sx}", lambda t: 0.16 - 0.02 * t, H - 0.1, self.timber, count=9).location = (sx * (L / 2 - 0.25), 0, 0)
            self.disc(f"StumpTop{sx}", 0.14, H - 0.1, self.cut, 9).location = (sx * (L / 2 - 0.25), 0, 0)
        far = [self.box("FarSeat", (0, 0, H - 0.05), (L, 0.32, 0.12), self.grown),
               self.box("FarStumps", (0, 0, (H - 0.1) / 2), (L - 0.3, 0.28, H - 0.1), self.timber)]
        return far, (0, 0, 0.3), (1.6, -1.9, 1.1)

    def stool(self):
        H = 0.46
        self.staves("Stump", lambda t: 0.2 - 0.03 * t, H, self.timber, count=11)
        self.disc("Seat", 0.17, H, self.cut, 11)
        for z in (H - 0.08, H - 0.14):
            self.hoop(f"Rope{z:.2f}", 0.172 + (H - z) * 0.12, z, self.rope, 0.015)
        far = [self.box("Far", (0, 0, H / 2), (0.36, 0.36, H), self.timber)]
        return far, (0, 0, 0.25), (0.9, -1.1, 0.8)

    def barrel(self):
        H = 0.92
        self.staves("Staves", lambda t: 0.27, H, self.timber, count=16, bulge=0.05)
        self.disc("Lid", 0.26, H - 0.02, self.cut, 16)
        for z, r in ((0.1, 0.285), (0.32, 0.31), (H - 0.32, 0.31), (H - 0.1, 0.285)):
            self.hoop(f"Hoop{z:.2f}", r + 0.008, z, self.rust, 0.018)
        far = [self.mesh_obj("Far", lambda bm: self.tube(bm, [(0, 0, 0), (0, 0, H / 2), (0, 0, H)], [0.27, 0.32, 0.27], 6), self.timber, smooth=False)]
        return far, (0, 0, 0.45), (1.1, -1.4, 1.1)

    def bucket(self):
        H = 0.34
        self.staves("Staves", lambda t: 0.13 + 0.04 * t, H, self.timber, count=12)
        self.disc("Contents", 0.155, H - 0.06, self.soil, 12)
        self.hoop("Hoop", 0.155, H * 0.55, self.rust, 0.012)
        arc = [(math.cos(a) * 0.17, 0, H + math.sin(a) * 0.16) for a in (math.pi * k / 10 for k in range(11))]
        self.mesh_obj("Handle", lambda bm: self.tube(bm, arc, [0.012] * 11, 5), self.rope, smooth=False)
        far = [self.box("Far", (0, 0, H / 2), (0.32, 0.32, H), self.timber)]
        return far, (0, 0, 0.2), (0.7, -0.8, 0.6)

    def planter(self):
        H = 0.42
        self.staves("Staves", lambda t: 0.33 + 0.03 * t, H, self.timber, count=16)
        self.disc("Soil", 0.34, H - 0.05, self.soil, 16)
        for z in (0.1, H - 0.08):
            self.hoop(f"Hoop{z:.2f}", 0.345 + z * 0.07, z, self.rust, 0.016)
        self.leaves("Greens", (0, 0, H - 0.05), 0.28, 34, self.leaf, 0.42)
        self.leaves("Shoots", (0, 0, H - 0.05), 0.22, 16, self.sprout, 0.6)
        far = [self.box("FarTub", (0, 0, H / 2), (0.7, 0.7, H), self.timber),
               self.box("FarGreens", (0, 0, H + 0.2), (0.55, 0.55, 0.4), self.leaf)]
        return far, (0, 0, 0.4), (1.2, -1.5, 1.2)

    def hammock(self):
        span, H = 3.2, 1.9
        for sx in (-1, 1):
            x = sx * span / 2
            pts = [(x + sx * 0.05, 0, 0.0), (x, 0.03, H * 0.5), (x - sx * 0.04, -0.02, H)]
            self.mesh_obj(f"Post{sx}", lambda bm, p=pts: self.tube(bm, p, [0.1, 0.085, 0.07], 8), self.grown)
            self.box(f"Ring{sx}", (x - sx * 0.04, 0, H - 0.25), (0.16, 0.16, 0.04), self.rust)
        # The bed: a sagging canvas sheet between spreader bars, hung by ropes.
        a, b = -span / 2 + 0.6, span / 2 - 0.6
        def sag(x):
            return 0.62 + 0.28 * (1 - (2 * (x - a) / (b - a) - 1) ** 2) * -1 + 0.28
        def bed(bm):
            rows = []
            n, m = 12, 5
            for i in range(n + 1):
                x = a + (b - a) * i / n
                rows.append([bm.verts.new((x, -0.4 + 0.8 * j / m, sag(x) - 0.08 * (1 - (2 * j / m - 1) ** 2))) for j in range(m + 1)])
            for r0, r1 in zip(rows, rows[1:]):
                for j in range(m):
                    bm.faces.new((r0[j], r0[j + 1], r1[j + 1], r1[j]))
            bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.015)
        self.mesh_obj("Canvas", bed, self.canvas)
        for sx, x in ((-1, a), (1, b)):
            self.pipe(f"Spreader{sx}", [(x, -0.42, sag(x)), (x, 0.42, sag(x))], 0.02, self.grown, 5)
            for y in (-0.38, 0.0, 0.38):
                self.pipe(f"Rope{sx}{y}", [(x, y, sag(x)), (sx * (span / 2 - 0.04), 0, H - 0.25)], 0.008, self.rope, 4)
        far = [self.box("FarPosts", (0, 0, H / 2), (span, 0.12, H), self.grown),
               self.box("FarBed", ((a + b) / 2, 0, sag(0) + 0.05), (b - a, 0.8, 0.12), self.canvas)]
        return far, (0, 0, 0.9), (2.8, -3.6, 2.0)

    # ------------------------------------------------------------ build

    def triangles(self, objs):
        return sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in objs if o.type == "MESH")

    def build(self):
        far, aim, cam = getattr(self, self.name)()
        near = [o for o in self.collection.objects if o.type == "MESH" and o not in far]
        self.lod_levels(far)
        profile = {
            "id": self.asset_id, "model": f"{self.model_ref}#Scene0", "kind": "prop",
            "family": "homestead",
            "acoustics": {"materials": {mat.name: cls for mat, cls in self.acoustics.items()},
                          "tagged_meshes": {}, "about": "small clutter: no mesh carries the tag"},
            "triangles": {"near": self.triangles(near), "far": self.triangles(far)},
            "licence": "CC0-1.0: generated by tools/blender/create_homestead_props.py",
        }

        def stage():
            light = bpy.data.objects.new("PreviewSun", bpy.data.lights.new("PreviewSun", "SUN"))
            light.data.energy = 3.5
            light.rotation_euler = (math.radians(50), 0, math.radians(35))
            self.scene.collection.objects.link(light)
            far_root = bpy.data.objects.get(f"{self.asset_id}_LOD1")
            for obj in ([far_root] + list(far_root.children_recursive)) if far_root else []:
                obj.hide_render = True
            self.scene.render.resolution_x = self.scene.render.resolution_y = 420
        report = self.finish(profile, aim=aim, camera_location=cam, before_render=stage)
        report.update(triangles=profile["triangles"])
        return report


def write_profile():
    profiles = {p: json.loads((OUT / f"homestead_{p}/asset.json").read_text()) for p in PROPS}
    (OUT / "asset.json").write_text(json.dumps({
        "id": "homestead_props", "kind": "prop",
        "about": "The furniture and gear around a Dancer cottage (docs/art/cottage.md): placed "
                 "beside it, so the cottage model stays one clean glb.",
        "props": {p: f"props/homestead/homestead_{p}/homestead_{p}.glb#Scene0" for p in PROPS},
        "triangles": {p: v["triangles"] for p, v in profiles.items()},
    }, indent=2) + "\n")


def montage():
    magick = shutil.which("magick")
    if not magick:
        return
    cmd = [magick, "montage"]
    for p in PROPS:
        path = PREVIEW_OUT / f"homestead_{p}/preview.png"
        cmd += ["-label", p, str(path)]
    cmd += ["-tile", "4x2", "-geometry", "+4+4", "-background", "#15161a", "-fill", "#d8d4cc",
            "-pointsize", "16", str(PREVIEW_OUT / "sheet.png")]
    subprocess.run(cmd, check=True)
    for p in PROPS:   # keep the repo light: the sheet carries the props
        (PREVIEW_OUT / f"homestead_{p}/preview_far.png").unlink(missing_ok=True)


if __name__ == "__main__":
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if args == ["montage"]:
        write_profile()
        montage()
    elif args:
        print(json.dumps(HomesteadProp(args[0]).build()))
    else:
        for p in PROPS:
            subprocess.run([bpy.app.binary_path, "-b", "--factory-startup", "-P", __file__, "--", p], check=True)
        write_profile()
        montage()
