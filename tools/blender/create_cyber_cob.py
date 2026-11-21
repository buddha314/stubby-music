"""The cyber-cob: the vardo's horse (add-vardo-and-cyber-cob-assets 3.x).

Built on a rigged, animated CC0 base instead of from scratch (user, 2026-11-20): "Horse" by
Quaternius (`assets-src/third_party/quaternius/`, CC0 1.0), whose 50-joint rig and gentle
clips are kept. The mesh is restyled and extended to `docs/art/vardo-and-cyber-cob.md`:

- one compact, sturdy, black-and-white piebald working cob, about 1.48 m at the withers;
  a black head with a white blaze, white lower legs, a black-and-cream mane and tail;
- feathered lower legs and muddy hooves;
- exactly **one** prosthetic limb, a front leg (`PROSTHESIS_SIDE`): a leather cuff on the
  stump, a brass knee housing, a steel pylon with a piston rod, an ankle joint and a
  hoof-shaped steel foot, with one tiny dim cyan indicator; skinned rigidly to the leg's
  bones, so every clip moves it;
- a few fine, dim cyan fibre-optic strands in both mane and tail, skinned to the neck and
  tail bones;
- a worn leather halter with a rope lead, and a plain collar, back pad and belly band with
  brass tugs for the vardo's shafts.

Clips kept (the cob is gentle; no attacks, falls or deaths): Idle, Idle_2, Idle_Headlow,
Eating, Walk, Gallop. The game loops `Idle` on a placed cob.

**Which front leg.** The brief says "near-side front leg (visible with the horse facing
right)", and its study shows the prosthesis on the leg facing the viewer with the horse
facing right: the horse's own right foreleg. That is what is built; set PROSTHESIS_SIDE to
"L" for the horse's left (the equestrian near side) if that was meant.

Frame (Blender, +Z up): the head toward -Y (glTF +Z, the model's front); origin on the
ground under the body. No LOD levels: Jackdaw's LOD runtime flattens a model into bare
meshes, which would drop the skin; the cob is one skinned level (~2.6k triangles).

Reproduce: blender -b --factory-startup -P tools/blender/create_cyber_cob.py
Writes assets/creatures/cyber_cob/ (model, profile), assets-src/blender/creatures/
cyber_cob.blend and previews/creatures/cyber_cob/ (side, three-quarter, front, rear,
walk pose, prosthesis close-up).
"""
from pathlib import Path
import json
import math
import random
import sys

import bmesh
import bpy
from mathutils import Vector, noise

sys.path.insert(0, str(Path(__file__).resolve().parent))
from create_burbler import Burbler, linear  # noqa: E402
from create_cottage import Cottage  # noqa: E402
from module_kit import PROJECT  # noqa: E402

SOURCE = PROJECT / "assets-src/third_party/quaternius/horse.glb"
PROSTHESIS_SIDE = "R"       # the horse's own right foreleg (see the docstring)
WITHERS_M = 1.48            # a cob: about 14.2 hands
KEEP_CLIPS = ("Idle", "Idle_2", "Idle_Headlow", "Eating", "Walk", "Gallop")


class CyberCob(Burbler):
    folder = "creatures"
    tube = staticmethod(Cottage.tube)

    def __init__(self):
        Burbler.__init__(self, "cyber_cob", kind="CyberCob")
        for key in ("mine", "produces"):
            del self.root[key]
        self.root["kind"] = "creature"
        self.rng = random.Random(1884)
        noise.seed_set(1884)
        m = self.material
        self.coat_black = m("CoatBlack", linear("#1d1a1c"), roughness=0.85)
        self.coat_white = m("CoatWhite", linear("#e0d8c8"), roughness=0.9)
        self.cream = m("Cream", linear("#cfc6b2"), roughness=0.95)        # bone: feathering, mane
        self.muzzle = m("Muzzle", linear("#6e5a58"), roughness=0.7)
        self.hoof = m("Hoof", linear("#3e2f22"), roughness=0.9)           # mud plate dark
        self.eye = m("Eye", linear("#100b19"), roughness=0.2)             # silhouette
        self.eye_white = m("EyeWhite", linear("#e7e5dd"), roughness=0.4)
        self.leather = m("Leather", linear("#4a3222"), roughness=0.8)     # cattail
        self.rope = m("Rope", linear("#8a7552"), roughness=1.0)
        self.steel = m("Steel", linear("#b8bcc2"), metallic=1.0, roughness=0.45)
        self.dark_steel = m("DarkSteel", linear("#2e2b33"), metallic=0.8, roughness=0.55)
        self.brass = m("Brass", linear("#c4aa76"), metallic=1.0, roughness=0.35)
        cyan = linear("#33ffd9")
        self.fibre = m("Fibre", cyan, emission=cyan, strength=1.2)
        self.indicator = m("Indicator", cyan, emission=cyan, strength=0.8)
        # Every material declares its acoustic class; a creature moves, so nothing is tagged.
        self.acoustics = {mat.name: None for mat in (
            self.coat_black, self.coat_white, self.cream, self.muzzle, self.hoof, self.eye,
            self.eye_white, self.leather, self.rope, self.steel, self.dark_steel, self.brass,
            self.fibre, self.indicator)}

    # ------------------------------------------------------------ import

    def load(self):
        before = set(bpy.data.objects)
        bpy.ops.import_scene.gltf(filepath=str(SOURCE))
        new = [o for o in bpy.data.objects if o not in before]
        for obj in new:
            for col in list(obj.users_collection):
                col.objects.unlink(obj)
            self.collection.objects.link(obj)
        for obj in [o for o in new if o.type == "MESH" and not o.vertex_groups]:
            bpy.data.objects.remove(obj)          # a stray icosphere in the source
        self.arm = next(o for o in self.collection.objects if o.type == "ARMATURE")
        self.body = next(o for o in self.collection.objects if o.type == "MESH" and o.vertex_groups)
        self.arm.name, self.body.name = "CyberCob_Armature", "CyberCob_Body"
        self.arm.parent = self.root
        for action in list(bpy.data.actions):
            if action.name not in KEEP_CLIPS:
                bpy.data.actions.remove(action)
        if self.arm.animation_data is None:
            self.arm.animation_data_create()
        self.arm.animation_data.action = bpy.data.actions.get("Idle")
        self.bones = {b.name: self.arm.matrix_world @ b.head_local for b in self.arm.data.bones}
        self.tails = {b.name: self.arm.matrix_world @ b.tail_local for b in self.arm.data.bones}
        mw = self.body.matrix_world
        # A cob, not a riding horse: widen the barrel, chest and shoulders in the rest pose
        # (skinning is untouched; the legs and head keep their width).
        inv = mw.inverted()
        for v in self.body.data.vertices:
            p = mw @ v.co
            if p.z > 1.25 and p.y > self.bones["Head"].y + 0.35:
                t = min(1.0, (p.z - 1.25) / 0.4)
                p.x *= 1.0 + 0.2 * t
                v.co = inv @ p
        self.verts = [(mw @ v.co, self.dominant(v)) for v in self.body.data.vertices]

    def dominant(self, v):
        if not v.groups:
            return None
        g = max(v.groups, key=lambda g: g.weight)
        return self.body.vertex_groups[g.group].name

    def section(self, y, slab=0.06, zmin=-1e9, zmax=1e9):
        """Centre and half extents (x, z) of the body's cross-section near y."""
        pts = [p for p, _ in self.verts if abs(p.y - y) < slab and zmin < p.z < zmax]
        xs, zs = [p.x for p in pts], [p.z for p in pts]
        return Vector(((max(xs) + min(xs)) / 2, y, (max(zs) + min(zs)) / 2)), (max(xs) - min(xs)) / 2, (max(zs) - min(zs)) / 2

    # ------------------------------------------------------------ coat

    def coat(self):
        mesh = self.body.data
        names = [s.material.name if s.material else "" for s in self.body.material_slots]
        for mat in (self.coat_black, self.coat_white, self.cream, self.muzzle, self.hoof, self.eye, self.eye_white):
            mesh.materials.append(mat)
        index = {mat.name: len(names) + k for k, mat in enumerate(
            (self.coat_black, self.coat_white, self.cream, self.muzzle, self.hoof, self.eye, self.eye_white))}
        mw = self.body.matrix_world
        head_y = self.bones["Head"].y
        for poly in mesh.polygons:
            old = names[poly.material_index]
            c = mw @ poly.center
            n = (mw.to_3x3() @ poly.normal).normalized()
            if old.startswith("Main"):
                if c.y < head_y + 0.25:                       # the head: black with a white blaze
                    blaze = abs(c.x) < 0.11 and n.y < -0.35 and c.z > 3.5
                    pick = self.coat_white if blaze else self.coat_black
                elif c.z < 1.05:                              # white lower legs
                    pick = self.coat_white
                else:                                         # big irregular piebald patches
                    v = noise.noise(Vector((c.x * 0.55, c.y * 0.45, c.z * 0.55)) + Vector((3.1, 7.7, 1.3)))
                    v += 0.35 * noise.noise(c * 1.6 + Vector((11.0, 2.0, 5.0)))
                    pick = self.coat_black if v > 0.05 else self.coat_white
            elif old == "Hair":
                v = noise.noise(c * 2.2 + Vector((4.0, 9.0, 1.0)))
                pick = self.cream if v > 0.15 else self.coat_black
            elif old == "Muzzle":
                pick = self.muzzle
            elif old == "Hooves":
                pick = self.hoof
            elif old == "Eye_White":
                pick = self.eye_white
            elif old == "Eye_Black":
                pick = self.eye
            else:
                pick = self.coat_black
            poly.material_index = index[pick.name]
        # The source's materials stay as unused slots; the exporter writes only used ones.

    # ------------------------------------------------------------ parts skinned to bones

    def part(self, name, fill, mat, weights):
        """A new mesh in world space whose vertices are skinned by `weights(point)` ->
        {bone: weight}; returned for joining into the body."""
        bm = bmesh.new()
        fill(bm)
        mesh = bpy.data.meshes.new(f"CyberCob_{name}")
        bm.to_mesh(mesh)
        bm.free()
        obj = bpy.data.objects.new(f"CyberCob_{name}", mesh)
        self.collection.objects.link(obj)
        obj.data.materials.append(mat)
        groups = {}
        for v in mesh.vertices:
            for bone, w in weights(v.co).items():
                if bone not in groups:
                    groups[bone] = obj.vertex_groups.new(name=bone)
                groups[bone].add([v.index], w, "REPLACE")
        for poly in mesh.polygons:
            poly.use_smooth = True
        return obj

    @staticmethod
    def rigid(bone):
        return lambda p: {bone: 1.0}

    def nearest(self, names):
        def weights(p):
            best = min(names, key=lambda n: (self.bones[n] - p).length)
            return {best: 1.0}
        return weights

    def ring(self, bm, centre, ax, az, thickness, segments=16, axis="y", sides=6):
        c = Vector(centre)
        if axis == "y":
            pts = [c + Vector((math.cos(a) * ax, 0, math.sin(a) * az)) for a in (math.tau * k / segments for k in range(segments + 1))]
        else:
            pts = [c + Vector((math.cos(a) * ax, math.sin(a) * az, 0)) for a in (math.tau * k / segments for k in range(segments + 1))]
        self.tube(bm, pts, [thickness] * len(pts), sides, cap=False)

    def leg_axis(self, side, front=True):
        upper = f"{'Front' if front else 'Back'}LowerLeg.{side}"
        top = self.bones[upper]
        return top

    def feathering(self):
        """Feathering on the three natural legs, as a cob's grows: long hair from the back and
        sides of the fetlock falling over the top of the hoof, the hoof's front left clear."""
        parts = []
        legs = [("Front", s) for s in "LR" if s != PROSTHESIS_SIDE] + [("Back", "L"), ("Back", "R")]
        for front, side in legs:
            top = self.bones[f"{front}LowerLeg.{side}"]
            foot = Vector((top.x, top.y, 0.0))
            near = [(p, g) for p, g in self.verts if (Vector((p.x, p.y, 0)) - foot).length < 0.35 and 0.15 < p.z < 0.6]
            bone = max(set(g for _, g in near), key=[g for _, g in near].count) if near else f"{front}LowerLeg.{side}"
            r = max(((Vector((p.x, p.y, 0)) - foot).length for p, _ in near), default=0.15)

            def fill(bm, foot=foot, r=r):
                # The model's front is -Y: the hoof's front faces -Y and stays clear.
                for k in range(30):
                    a = self.rng.uniform(math.radians(-130), math.radians(130)) + math.pi / 2
                    d = Vector((math.cos(a), math.sin(a), 0))
                    behind = max(0.0, d.y)                         # 1 straight behind, 0 at the sides
                    w = Vector((-d.y, d.x, 0)) * self.rng.uniform(0.03, 0.05)
                    top_p = foot + d * (r * 0.8) + Vector((0, 0, self.rng.uniform(0.55, 0.8)))
                    low = (foot + d * (r * (1.05 + 0.2 * behind))
                           + Vector((0, 0, self.rng.uniform(0.14, 0.24) - 0.1 * behind)))
                    bm.faces.new([bm.verts.new(p) for p in (top_p - w, top_p + w, low)])
            parts.append(self.part(f"Feather{front}{side}", fill, self.cream, self.rigid(bone)))
        return parts

    def prosthesis(self):
        """Replace one foreleg below the knee with a field-repaired prosthesis."""
        side = PROSTHESIS_SIDE
        lower, upper = f"FrontLowerLeg.{side}", f"FrontUpperLeg.{side}"
        knee = self.bones[lower]
        # Which bones drive the natural lower leg and hoof: the foot's dominant group.
        foot_verts = [(p, g) for p, g in self.verts if (Vector((p.x, p.y, 0)) - Vector((knee.x, knee.y, 0))).length < 0.4 and p.z < 0.25]
        foot_bone = max(set(g for _, g in foot_verts), key=[g for _, g in foot_verts].count)
        drop = {lower, foot_bone}
        mesh = self.body.data
        bm = bmesh.new()
        bm.from_mesh(mesh)
        deform = bm.verts.layers.deform.active
        idx = {vg.name: vg.index for vg in self.body.vertex_groups}
        mw = self.body.matrix_world
        # Everything of that leg below the knee, by position: within the leg's footprint, on
        # its side of the body (skin weights alone left scraps of hoof and pastern).
        axis = Vector((knee.x, knee.y, 0))
        def below_knee(v):
            p = mw @ v.co
            return (p.z < knee.z - 0.05 and (Vector((p.x, p.y, 0)) - axis).length < 0.42
                    and p.x * knee.x > 0)
        doomed = [f for f in bm.faces if all(below_knee(v) for v in f.verts)]
        bmesh.ops.delete(bm, geom=doomed, context="FACES")
        bm.to_mesh(mesh)
        bm.free()
        r = 0.16
        x, y = knee.x, knee.y
        parts = []

        def cuff(bm):
            for k, z in enumerate((knee.z - 0.22, knee.z + 0.18)):
                pass
            self.tube(bm, [(x, y, knee.z - 0.2), (x, y, knee.z + 0.05), (x, y, knee.z + 0.22)],
                      [r * 1.15, r * 1.2, r * 1.1], 10)
        parts.append(self.part("ProsthesisCuff", cuff, self.leather, self.rigid(upper)))

        def straps(bm):
            for z in (knee.z - 0.12, knee.z + 0.12):
                self.ring(bm, (x, y, z), r * 1.24, r * 1.24, 0.025, 14, axis="z", sides=4)
            for a in (0.4, 1.9, 3.4, 4.9):
                c = Vector((x + math.cos(a) * r * 1.27, y + math.sin(a) * r * 1.27, knee.z - 0.12))
                bmesh.ops.create_cube(bm, size=0.05, matrix=__import__("mathutils").Matrix.Translation(c))
        parts.append(self.part("ProsthesisStraps", straps, self.brass, self.rigid(upper)))

        def knee_housing(bm):
            self.tube(bm, [(x - r * 0.9, y, knee.z - 0.28), (x + r * 0.9, y, knee.z - 0.28)], [0.1, 0.1], 12)
        parts.append(self.part("ProsthesisKnee", knee_housing, self.brass, self.rigid(lower)))

        def pylon(bm):
            self.tube(bm, [(x, y, knee.z - 0.3), (x, y + 0.01, 0.42)], [0.065, 0.055], 10)
            self.tube(bm, [(x, y - 0.11, knee.z - 0.38), (x, y - 0.1, 0.5)], [0.03, 0.03], 6)    # piston rod
            for z in (knee.z - 0.5, 0.75):
                self.tube(bm, [(x, y, z - 0.03), (x, y, z + 0.03)], [0.075, 0.075], 10)
        parts.append(self.part("ProsthesisPylon", pylon, self.steel, self.rigid(lower)))

        def pylon_dark(bm):
            self.tube(bm, [(x, y - 0.1, knee.z - 0.32), (x, y - 0.1, knee.z - 0.44)], [0.045, 0.045], 8)
            self.tube(bm, [(x - 0.09, y, 0.38), (x + 0.09, y, 0.38)], [0.07, 0.07], 10)       # ankle
        parts.append(self.part("ProsthesisJoints", pylon_dark, self.dark_steel, self.rigid(lower)))

        def foot(bm):
            # A hoof shape: wide at the ground, flat-fronted, sloping to the pastern.
            ring_lo, ring_hi = [], []
            for k in range(14):
                a = math.tau * k / 14
                rr = 0.17 * (0.85 if math.sin(a) > 0.3 else 1.0)
                ring_lo.append(bm.verts.new((x + math.cos(a) * rr, y + math.sin(a) * rr * 1.1, 0.0)))
                ring_hi.append(bm.verts.new((x + math.cos(a) * rr * 0.62, y + math.sin(a) * rr * 0.6 + 0.03, 0.3)))
            for k in range(14):
                bm.faces.new((ring_lo[k], ring_lo[(k + 1) % 14], ring_hi[(k + 1) % 14], ring_hi[k]))
            bm.faces.new(ring_lo[::-1])
            bm.faces.new(ring_hi)
        parts.append(self.part("ProsthesisFoot", foot, self.steel, self.rigid(foot_bone)))

        def indicator(bm):
            c = Vector((x + (0.06 if side == "L" else -0.06), y, 0.95))
            bmesh.ops.create_cube(bm, size=1.0, matrix=__import__("mathutils").Matrix.LocRotScale(c, None, Vector((0.02, 0.03, 0.12))))
        parts.append(self.part("ProsthesisIndicator", indicator, self.indicator, self.rigid(lower)))
        self.prosthesis_bones = {"stump": upper, "leg": lower, "foot": foot_bone}
        return parts

    def fibres(self):
        """A few fine, dim cyan strands following the hair through mane and tail."""
        parts = []
        neck = ["Neck1", "Neck2", "Neck3", "Head"]
        tail = [f"Tail{k}" for k in range(1, 8)]

        def strand(bm, pts, width=0.022):
            for a, b in zip(pts, pts[1:]):
                d = (b - a).normalized()
                s = d.cross(Vector((1, 0, 0)))
                if s.length < 1e-3:
                    s = Vector((0, 1, 0))
                s = s.normalized() * width / 2
                bm.faces.new([bm.verts.new(p) for p in (a - s, a + s, b + s, b - s)])

        def mane(bm):
            for k, (side, off) in enumerate(((-1, 0.0), (-1, 0.15), (1, 0.08))):
                crest = [self.bones[n] + Vector((0, 0.05, 0.28)) for n in neck[:3]]
                start = crest[0].lerp(crest[-1], 0.25 + 0.25 * k)
                pts = [start + Vector((side * 0.06 * i, -0.02 * i, -0.13 * i + 0.02 * math.sin(i))) for i in range(4)]
                strand(bm, pts)
        parts.append(self.part("FibreMane", mane, self.fibre, self.nearest(neck[:3])))

        def tail_strands(bm):
            for dx in (-0.07, 0.0, 0.06):
                pts = [self.bones[n] + Vector((dx, 0.04, -0.05)) for n in tail[1:]]
                pts.append(self.tails[tail[-1]] + Vector((dx, 0.04, -0.05)))
                strand(bm, pts, 0.018)
        parts.append(self.part("FibreTail", tail_strands, self.fibre, self.nearest(tail)))
        return parts

    def tack(self):
        """A worn leather halter with a rope lead; a collar, back pad and belly band with
        brass tugs where the vardo's shafts ride."""
        parts = []
        head = self.bones["Head"]
        muzzle_y = min(p.y for p, _ in self.verts)
        ny = head.y + (muzzle_y - head.y) * 0.62
        c, ax, az = self.section(ny, 0.05, zmin=2.8)
        cy, cax, caz = self.section(head.y - 0.05, 0.05, zmin=3.6)

        def halter(bm):
            self.ring(bm, c, ax * 1.08, az * 1.06, 0.035, 18)                        # noseband
            self.ring(bm, cy, cax * 1.05, caz * 1.03, 0.035, 18)                     # crownpiece
            for sx in (-1, 1):                                                       # cheek pieces
                self.tube(bm, [Vector((c.x + sx * ax * 1.08, c.y, c.z)), Vector((cy.x + sx * cax * 1.05, cy.y, cy.z))], [0.03, 0.03], 5)
        parts.append(self.part("Halter", halter, self.leather, self.rigid("Head")))

        def lead(bm):
            chin = Vector((c.x, c.y, c.z - az * 1.06))
            pts = [chin + Vector((0.02 * math.sin(i), 0.03 * i, -0.16 * i)) for i in range(6)]
            self.tube(bm, pts, [0.022] * 6, 5)
        parts.append(self.part("LeadRope", lead, self.rope, self.rigid("Head")))

        def halter_rings(bm):
            self.ring(bm, (c.x, c.y, c.z - az * 1.1), 0.05, 0.05, 0.012, 10, axis="y", sides=4)
        parts.append(self.part("HalterRing", halter_rings, self.brass, self.rigid("Head")))
        # Collar at the base of the neck.
        n1 = self.bones["Neck1"]
        # The neck's base only: below the crest's hair, above the chest.
        cc, cax2, caz2 = self.section(n1.y - 0.08, 0.05, zmin=2.15, zmax=n1.z + 0.35)

        def collar(bm):
            self.ring(bm, cc, cax2 * 1.06, caz2 * 1.04, 0.075, 20, sides=8)
        parts.append(self.part("Collar", collar, self.leather, self.rigid("Torso3")))

        def hames(bm):
            for sx in (-1, 1):
                pts = [cc + Vector((sx * cax2 * 1.1, 0.02, caz2 * s)) for s in (-0.6, 0.0, 0.7)]
                self.tube(bm, pts, [0.03] * 3, 6)
        parts.append(self.part("Hames", hames, self.brass, self.rigid("Torso3")))
        # Back pad and belly band behind the withers, with the shaft tugs.
        t2 = self.bones["Torso2"]
        bc, bax, baz = self.section(t2.y - 0.25, 0.06, zmin=1.2)

        def band(bm):
            self.ring(bm, bc, bax * 1.03, baz * 1.02, 0.05, 22, sides=6)
        parts.append(self.part("BellyBand", band, self.leather, self.rigid("Torso2")))
        top = bc.z + baz * 1.02

        def pad(bm):
            rows = []
            for i in range(7):
                a = math.pi * (0.2 + 0.6 * i / 6)
                rows.append([bm.verts.new((bc.x - math.cos(a) * bax * 1.06, yy, bc.z + math.sin(a) * baz * 1.06 + 0.03)) for yy in (bc.y - 0.22, bc.y + 0.22)])
            for r0, r1 in zip(rows, rows[1:]):
                bm.faces.new((r0[0], r0[1], r1[1], r1[0]))
            bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.05)
        parts.append(self.part("BackPad", pad, self.leather, self.rigid("Torso2")))

        def tugs(bm):
            for sx in (-1, 1):
                self.ring(bm, (bc.x + sx * bax * 1.12, bc.y, bc.z - baz * 0.15), 0.07, 0.07, 0.016, 12, axis="y", sides=4)
        parts.append(self.part("ShaftTugs", tugs, self.brass, self.rigid("Torso2")))
        return parts

    # ------------------------------------------------------------ build

    def build(self):
        self.load()
        self.coat()
        parts = self.feathering() + self.prosthesis() + self.fibres() + self.tack()
        # Join the parts into the skinned body, inside the cob's own scene.
        layer = self.scene.view_layers[0]
        for obj in [self.body] + parts:
            obj.select_set(True, view_layer=layer)
        layer.objects.active = self.body
        with bpy.context.temp_override(scene=self.scene, view_layer=layer, active_object=self.body,
                                       object=self.body, selected_objects=[self.body] + parts,
                                       selected_editable_objects=[self.body] + parts):
            bpy.ops.object.join()
        assert not any(p.name in bpy.data.objects for p in [] ), "joined"
        left = [o.name for o in self.collection.objects if o.type == "MESH" and o is not self.body]
        if left:
            raise RuntimeError(f"parts not joined into the body: {left}")
        # Scale to a cob: the withers (the highest point of the back over the shoulders).
        withers = max(p.z for p, _ in self.verts if -1.3 < p.y < -0.7 and abs(p.x) < 0.2)
        k = WITHERS_M / withers
        self.arm.scale = tuple(s * k for s in self.arm.scale)
        bpy.context.view_layer.update()
        tris = sum(len(p.vertices) - 2 for p in self.body.data.polygons)
        bones = len(self.arm.data.bones)
        profile = {
            "id": "cyber_cob", "model": f"{self.model_ref}#Scene0", "kind": "creature",
            "about": "The vardo's horse: a black-and-white piebald working cob with one prosthetic "
                     "foreleg and dim fibre-optic strands in mane and tail. Brief: "
                     "docs/art/vardo-and-cyber-cob.md. Base: Quaternius 'Horse' (CC0 1.0), rig and "
                     "clips kept, restyled and extended by tools/blender/create_cyber_cob.py.",
            "size_metres": {"withers": WITHERS_M},
            "prosthesis": {"leg": f"front {'right' if PROSTHESIS_SIDE == 'R' else 'left'} (the horse's own)",
                           "bones": self.prosthesis_bones},
            "rig": {"joints": bones, "source": "Quaternius AnimalArmature"},
            "clips": {"loop": "Idle (the game loops it on a placed cob)",
                      "kept": list(KEEP_CLIPS)},
            "acoustics": {"materials": self.acoustics, "tagged_meshes": {},
                          "about": "a moving creature: not part of the static acoustic scene"},
            "triangles": {"near": tris},
            "lod": "none: one skinned level (Jackdaw's LOD runtime would flatten the skin)",
            "licence": "CC0-1.0: base by Quaternius (CC0 1.0, assets-src/third_party/quaternius/); "
                       "changes generated by tools/blender/create_cyber_cob.py",
        }
        self.scene.frame_set(20)

        def stage():
            light = bpy.data.objects.new("PreviewSun", bpy.data.lights.new("PreviewSun", "SUN"))
            light.data.energy = 3.5
            light.rotation_euler = (math.radians(50), 0, math.radians(-60))
            self.scene.collection.objects.link(light)
            self.scene.camera.data.lens = 40
        # Viewed from the horse's right, facing right, as the brief's study.
        report = self.finish(profile, aim=(0, -0.2, 0.95), camera_location=(-4.6, -1.6, 1.3), before_render=stage)
        if bpy.app.background:
            self.views()
        report.update(triangles=tris, joints=bones)
        return report

    def views(self):
        out = PROJECT / "previews/creatures/cyber_cob"
        scene, cam = self.scene, self.scene.camera
        shots = {
            "three_quarter": ((-3.6, -3.4, 1.6), (0, -0.3, 0.9), 40),
            "front": ((0, -5.0, 1.3), (0, 0, 1.0), 45),
            "rear": ((0, 4.6, 1.4), (0, 0, 1.0), 45),
            "prosthesis": ((-1.6, -1.6, 0.6) if PROSTHESIS_SIDE == "R" else (1.6, -1.6, 0.6), (0, -0.65, 0.45), 50),
        }
        for name, (loc, aim, lens) in shots.items():
            cam.location = Vector(loc)
            cam.rotation_euler = (Vector(aim) - cam.location).to_track_quat("-Z", "Y").to_euler()
            cam.data.lens = lens
            scene.render.filepath = str(out / f"{name}.png")
            bpy.ops.render.render(write_still=True, scene=scene.name)
        # Mid-stride: the Walk clip, side on.
        self.arm.animation_data.action = bpy.data.actions.get("Walk")
        scene.frame_set(7)
        cam.location = Vector((-4.6, -1.6, 1.3))
        cam.rotation_euler = (Vector((0, -0.2, 0.95)) - cam.location).to_track_quat("-Z", "Y").to_euler()
        cam.data.lens = 40
        scene.render.filepath = str(out / "walk.png")
        bpy.ops.render.render(write_still=True, scene=scene.name)
        self.arm.animation_data.action = bpy.data.actions.get("Idle")


if __name__ == "__main__":
    print(json.dumps(CyberCob().build()))
