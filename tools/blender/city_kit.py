"""Shared building blocks for the City kit (Phase 4: a dense, lush solarpunk city).

Brief: `docs/art/city-kit.md`; prompt: `docs/art/prompts/city-kit.md`. The generators
are `create_city_kit.py` (one placeable asset per module) and `create_city_district.py`
(the reference district and test strip, built from linked instances of those assets).

Conventions every piece keeps (the brief's *Modular contract*):

- Metres, Blender Z up (the exporter turns it into Bevy's Y up). 1 m grid, 0.25 m detail,
  4 m structural bay, 4 m floor to floor.
- **Linear modules** (facades, slabs, banks, walks, rails) run along +X from their origin,
  which sits at ground level on the left front snap corner. Their front (street or canal
  side) faces Blender -Y, which is glTF/Bevy +Z. Depth goes into +Y (Bevy -Z).
- **Standalone pieces** (props, plants, the monument) have their origin at the centre of
  their base.
- Every mesh is built in the asset's own frame with an identity transform: nothing to
  apply, no negative scale.
- Levels of detail go inside the model as `<id>_LOD0`, `<id>_LOD1` nodes (the repository
  convention); sockets are childless empties on the root, beside the levels, so they
  survive Jackdaw's LOD flattening.
- Every material declares an acoustic class, a Steam Audio preset by name
  (`stubby_core::AcousticOccluder::preset`: generic, rock, concrete, brick, wood, glass,
  metal, carpet, gravel), on the material (glTF material extras `acoustic`, "none" for
  materials that only dress small or living parts) and, as the game reads it today, on
  each large near-level mesh node (glTF node extras `acoustic`).
- Living light is jellyfish and fairies, not LEDs: an opaque emissive core and separate,
  translucent halo shells (alpha 0.2-0.4), authored at peak. The game's audio clock
  drives the breathing; nothing here animates on its own.
"""
from pathlib import Path
import json
import math
import random
import sys

import bmesh
import bpy
from mathutils import Vector

PROJECT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT / "tools/blender"))
from export_gltf import export_scene  # noqa: E402

FOLDER = "structures/city"
BAY = 4.0          # structural bay and floor-to-floor height
WALL = 0.4         # facade wall thickness
SLAB = 0.3         # floor slab thickness
WATER_Z = -0.9     # canal water surface, below the promenade (z 0)
BED_Z = -2.0       # canal bed
from city_catalog import CATALOG_ID, CATALOG_PATH  # noqa: E402


def linear(hex_srgb):
    """sRGB hex (the palette, the keys file) to Blender's linear colour."""
    h = hex_srgb.lstrip("#")
    out = []
    for i in (0, 2, 4):
        c = int(h[i:i + 2], 16) / 255
        out.append(c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4)
    return tuple(out)


# The City palette (style guide, *Palette*, City group). sRGB.
PALETTE = {
    "Plaster": "#d6c2a8",      # pale mineral lime plaster over rammed earth
    "Relief": "#bba187",       # carved relief and strata bands, a shade deeper
    "Stone": "#c9b9a3",        # paving, coping, steps: pale limestone
    "StoneDark": "#9c8a74",    # kerbs, joints, worn treads
    "Clay": "#a85b43",         # fired-clay accent bands and planters
    "Monument": "#eadbd2",     # the pale monument: rosy where the light falls
    "GrownWood": "#4b4037",    # grown timber ribs (living bark, the scrub oak's)
    "Timber": "#8a7a62",       # sawn heartwood decks and doors
    "Titanium": "#b8bcc2",
    "Graphene": "#1a1c1f",
    "Glass": "#b9d3cf",      # clear, faintly green: rooms behind it must read
    "Soil": "#302116",
    "Moss": "#3f5a24",
    "Leaf": "#4f6a2a",
    "LeafYoung": "#7b8c3c",
    "Sprout": "#6f9a34",
    "Agave": "#6f8f7a",        # blue-green succulent
    "GrassDry": "#9a8460",     # the reeds' dry stalk: desert grass
    "Bloom": "#c96d74",        # flowering shrubs, clay coral
    "Water": "#2f8a84",        # clear canal water, teal
    "Bed": "#5b7254",          # pale, algae-washed stone bed: clear water shows it
    "Mud": "#302116",          # the old mud stream (relic)
    "Crust": "#4a3a2a",
    "Canvas": "#cfc6b2",       # bone: shade sails, the rail car
    "Cyan": "#33ffd9",
    "Magenta": "#ff4dd9",
    "Green": "#8cd933",
    "Bulb": "#ecd8e3",         # pale lantern bulbs (the fairy-light language)
    "Amber": "#ffb350",        # machine signal: machines only
    # Domestic and commercial (the residential expansion, docs/art/city-residential.md)
    "InteriorWall": "#e9dfcf", # warm limewash inside rooms
    "Linen": "#e8dcc6",        # curtains, bedding
    "Coral": "#c96d74",        # textile tints: the Dancers' skin palette, reused for cloth
    "Teal": "#4f9d88",
    "Violet": "#8878ab",
    "Ochre": "#b8892f",        # the cable palette's ochre, verdigris and slate
    "Verdigris": "#3f7a6e",
    "Slate": "#4b5a78",
    "Rust": "#9c3b2a",
    "Railing": "#2b2a2e",      # dark forged balcony rails
    "Hearth": "#ffd2a1",       # warm domestic lamplight: paler than the amber signal
}


def rounded_rect(x0, x1, y0, y1, r, seg=4):
    """Counter-clockwise outline of a rectangle with rounded corners."""
    r = min(r, (x1 - x0) / 2, (y1 - y0) / 2)
    if r <= 1e-6:
        return [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
    pts = []
    for cx, cy, a0 in ((x1 - r, y0 + r, -90), (x1 - r, y1 - r, 0), (x0 + r, y1 - r, 90), (x0 + r, y0 + r, 180)):
        for k in range(seg + 1):
            a = math.radians(a0 + 90 * k / seg)
            pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return pts


def arch_opening(x0, x1, z0, spring, seg=10):
    """Counter-clockwise outline of a round-headed opening: straight jambs to `spring`,
    then a semicircle (or a segmental arch if the opening is wide)."""
    w = x1 - x0
    r = w / 2
    cx = (x0 + x1) / 2
    pts = [(x0, z0), (x1, z0), (x1, spring)]
    for k in range(1, seg):
        a = math.pi * k / seg
        pts.append((cx + r * math.cos(a), spring + r * math.sin(a)))
    pts.append((x0, spring))
    return pts


def circle(cx, cy, r, n=24):
    return [(cx + r * math.cos(math.tau * k / n), cy + r * math.sin(math.tau * k / n)) for k in range(n)]


class Piece:
    """One placeable City asset: its own scene, export collection and asset collection."""

    def __init__(self, asset_id, kind, *, catalog, priority, about, phases=("city",), seed=None):
        self.asset_id = asset_id
        self.kind = kind
        self.catalog = catalog
        self.priority = priority
        self.about = about
        self.phases = list(phases)
        self.rng = random.Random(seed if seed is not None else sum(map(ord, asset_id)) * 31)
        self.scene = bpy.data.scenes.new(f"{kind}Asset")
        self.scene.unit_settings.system = "METRIC"
        self.collection = bpy.data.collections.new(f"STUBBY_{kind}")
        self.scene.collection.children.link(self.collection)
        self.root = bpy.data.objects.new(f"{kind}_Root", None)
        self.collection.objects.link(self.root)
        self.root["asset_id"] = asset_id
        self.root["kind"] = "structure"
        self.root["family"] = "city"
        self.materials = {}
        self.acoustics = {}       # material -> preset or None
        self.sockets = {}         # name -> meaning
        self.socket_frames = {}   # name -> position, accepts (for the manifest and district)
        self.connectors = {}      # name -> (position, direction) in Blender coords
        self.lights = {}          # material name -> meaning
        self.colliders = []       # simple shapes, Blender coords (see collider())
        self.far = []             # LOD1 objects, if the piece has a far level
        self.small = set()        # meshes left out of the acoustic scene
        self.size = None
        self.notes = []

    # ------------------------------------------------------------ materials

    def mat(self, key, acoustic, *, metallic=0.0, roughness=0.8, emission=None, strength=0.0,
            alpha=None, color=None, double=False):
        """The kit's shared materials, made once per piece: `City_<key>`."""
        name = f"City_{key}"
        if name in self.materials:
            return self.materials[name]
        col = linear(color or PALETTE[key])
        m = bpy.data.materials.new(name)
        m.use_nodes = True
        bsdf = m.node_tree.nodes.get("Principled BSDF")
        bsdf.inputs["Base Color"].default_value = (*col, 1.0)
        bsdf.inputs["Metallic"].default_value = metallic
        bsdf.inputs["Roughness"].default_value = roughness
        if emission:
            e = linear(emission)
            bsdf.inputs["Emission Color"].default_value = (*e, 1.0)
            bsdf.inputs["Emission Strength"].default_value = strength
        if alpha is not None:
            bsdf.inputs["Alpha"].default_value = alpha
            m.surface_render_method = "BLENDED"
            m.use_backface_culling = False
        else:
            m.use_backface_culling = not double   # leaf cards and canvas are seen from both sides
        m.diffuse_color = (*col, alpha if alpha is not None else 1.0)
        m["acoustic"] = acoustic or "none"
        self.materials[name] = m
        self.acoustics[name] = acoustic
        return m

    # The shared families. Acoustic presets are the nearest physical match: lime plaster
    # over rammed earth is brick (as the cottage's rammed earth), dressed limestone is
    # rock, paving and slabs are concrete, water is glass (as the cottage's rain tank).
    def plaster(self): return self.mat("Plaster", "brick", roughness=0.9)
    def relief(self): return self.mat("Relief", "brick", roughness=0.92)
    def stone(self): return self.mat("Stone", "rock", roughness=0.85)
    def paving(self): return self.mat("Paving", "concrete", roughness=0.8, color=PALETTE["Stone"])
    def stone_dark(self): return self.mat("StoneDark", "concrete", roughness=0.9)
    def clay(self): return self.mat("Clay", "brick", roughness=0.85)
    def monument(self): return self.mat("Monument", "rock", roughness=0.6)
    def grown(self): return self.mat("GrownWood", "wood", roughness=0.8)
    def timber(self): return self.mat("Timber", "wood", roughness=0.75)
    def titanium(self): return self.mat("Titanium", "metal", metallic=1.0, roughness=0.32)
    def graphene(self): return self.mat("Graphene", "metal", metallic=0.7, roughness=0.15)
    def glass(self): return self.mat("Glass", "glass", metallic=0.0, roughness=0.05, alpha=0.18)
    def soil(self): return self.mat("Soil", "gravel", roughness=1.0)
    def moss(self): return self.mat("Moss", "gravel", roughness=0.95)
    def leaf(self): return self.mat("Leaf", None, roughness=0.75, double=True)
    def leaf_young(self): return self.mat("LeafYoung", None, roughness=0.75, double=True)
    def sprout(self): return self.mat("Sprout", None, roughness=0.7, double=True)
    def agave(self): return self.mat("Agave", None, roughness=0.6, double=True)
    def grass(self): return self.mat("GrassDry", None, roughness=0.95, double=True)
    def bloom(self): return self.mat("Bloom", None, roughness=0.7, double=True)
    def water(self): return self.mat("Water", "glass", roughness=0.03, alpha=0.55)
    def bed(self): return self.mat("Bed", "gravel", roughness=0.95)
    def mud(self): return self.mat("Mud", "gravel", roughness=0.95, double=True)
    def crust(self): return self.mat("Crust", "gravel", roughness=1.0)
    def canvas(self): return self.mat("Canvas", "carpet", roughness=0.9, double=True)

    def interior(self): return self.mat("InteriorWall", "brick", roughness=0.9)
    def railing(self): return self.mat("Railing", "metal", metallic=0.6, roughness=0.5)

    def fabric(self, colour, acoustic="carpet"):
        """Cloth (curtains, awnings, upholstery, bedding): double-sided, carpet acoustically."""
        return self.mat(f"Fabric{colour}", acoustic, roughness=0.95, color=PALETTE[colour], double=True)

    def hearth(self):
        """Warm domestic lamplight: a core and its halo, authored at peak, dimmed by the game
        by day. Paler than the machine amber, which stays the signal colour."""
        core = self.mat("Hearth", None, roughness=0.4, emission=PALETTE["Hearth"], strength=14.0)
        halo = self.mat("HearthHalo", None, roughness=0.3, emission=PALETTE["Hearth"], strength=1.6,
                        alpha=0.22, color=PALETTE["Hearth"])
        self.lights[core.name] = "warm domestic lamplight, emissive at peak; the game dims it by day"
        self.lights[halo.name] = "its halo shell (alpha 0.22)"
        return core, halo

    def curved_wall(self, name, centre, r_out, r_in, a0, a1, z0, z1, mat, openings=(), glass=None,
                    seg=12, frame=None):
        """A wall on an arc about `centre` (XY), from angle a0 to a1 (radians), z0..z1, with
        openings [(t0, t1, zb, zt)] given as fractions of the arc. Glass sits mid-wall."""
        cx, cy = centre

        def pt(r, a, z):
            return (cx + r * math.cos(a), cy + r * math.sin(a), z)

        def fill(bm):
            for i in range(seg):
                t0, t1 = i / seg, (i + 1) / seg
                aa, ab = a0 + (a1 - a0) * t0, a0 + (a1 - a0) * t1
                holes = sorted((zb, zt) for s0, s1, zb, zt in openings if s0 <= (t0 + t1) / 2 <= s1)
                spans, z = [], z0
                for zb, zt in holes:
                    if zb > z:
                        spans.append((z, zb))
                    z = max(z, zt)
                if z < z1:
                    spans.append((z, z1))
                for za, zb in spans:
                    o = [bm.verts.new(pt(r_out, a, zz)) for a, zz in ((aa, za), (ab, za), (ab, zb), (aa, zb))]
                    n = [bm.verts.new(pt(r_in, a, zz)) for a, zz in ((aa, za), (ab, za), (ab, zb), (aa, zb))]
                    bm.faces.new(o)
                    bm.faces.new(n[::-1])
                    bm.faces.new((o[0], o[1], n[1], n[0]))
                    bm.faces.new((o[3], n[3], n[2], o[2]))
                for k, (zb, zt) in enumerate(holes):   # opening reveals: sill and head
                    pass
            for ends, a in ((0, a0), (1, a1)):
                o = [bm.verts.new(pt(r, a, zz)) for r, zz in ((r_out, z0), (r_in, z0), (r_in, z1), (r_out, z1))]
                bm.faces.new(o)
        self.mesh(name, fill, mat)
        if glass is not None:
            rg = (r_out + r_in) / 2
            for k, (s0, s1, zb, zt) in enumerate(openings):
                def pane(bm, s0=s0, s1=s1, zb=zb, zt=zt):
                    n = max(2, int(seg * (s1 - s0)) + 1)
                    rows = []
                    for i in range(n + 1):
                        a = a0 + (a1 - a0) * (s0 + (s1 - s0) * i / n)
                        rows.append((bm.verts.new(pt(rg, a, zb)), bm.verts.new(pt(rg, a, zt))))
                    for (p0, p1), (q0, q1) in zip(rows, rows[1:]):
                        bm.faces.new((p0, q0, q1, p1))
                self.mesh(f"{name}Glass{k}", pane, glass)
                if frame is not None:
                    for i in range(max(2, int(seg * (s1 - s0))) + 1):
                        a = a0 + (a1 - a0) * (s0 + (s1 - s0) * i / max(2, int(seg * (s1 - s0))))
                        self.tube(f"{name}Mullion{k}_{i}", [pt(rg, a, zb), pt(rg, a, zt)], 0.03, frame,
                                  sides=4, small=True)

    def sector(self, name, centre, r_out, r_in, a0, a1, z0, z1, mat, seg=12, **kw):
        """An annular sector (r_in may be 0 for a pie slice) extruded z0..z1."""
        cx, cy = centre
        outer = [(cx + r_out * math.cos(a0 + (a1 - a0) * i / seg), cy + r_out * math.sin(a0 + (a1 - a0) * i / seg))
                 for i in range(seg + 1)]
        if r_in > 1e-6:
            inner = [(cx + r_in * math.cos(a0 + (a1 - a0) * i / seg), cy + r_in * math.sin(a0 + (a1 - a0) * i / seg))
                     for i in range(seg, -1, -1)]
        else:
            inner = [(cx, cy)]
        return self.plan(name, outer + inner, z0, z1, mat, **kw)

    def glow(self, colour, what="breathes with the audio clock"):
        """A luminous core (opaque, emissive at peak) and its halo shell material."""
        core = self.mat(f"Glow{colour}", None, roughness=0.4, emission=PALETTE[colour],
                        strength=10.0, color=PALETTE[colour])
        halo = self.mat(f"Glow{colour}Halo", None, roughness=0.3, emission=PALETTE[colour],
                        strength=1.4, alpha=0.25, color=PALETTE[colour])
        self.lights[core.name] = f"{colour.lower()} core, emissive at peak; the game {what}"
        self.lights[halo.name] = f"{colour.lower()} halo shell (alpha 0.25), at peak with its core"
        return core, halo

    def signal(self):
        """Amber machine signal: machines and instruments only, never architecture."""
        core = self.mat("Signal", None, roughness=0.4, emission=PALETTE["Amber"], strength=8.0,
                        color=PALETTE["Amber"])
        halo = self.mat("SignalHalo", None, roughness=0.3, emission=PALETTE["Amber"], strength=1.0,
                        alpha=0.22, color=PALETTE["Amber"])
        self.lights[core.name] = "machine signal light (amber), emissive at peak; the rack's light value drives it"
        self.lights[halo.name] = "its halo shell (alpha 0.22)"
        return core, halo

    def lantern(self):
        core = self.mat("Bulb", None, roughness=0.4, emission=PALETTE["Bulb"], strength=6.0)
        halo = self.mat("BulbHalo", None, roughness=0.3, emission=PALETTE["Bulb"], strength=0.8,
                        alpha=0.2, color=PALETTE["Bulb"])
        self.lights[core.name] = "pale lantern bulb, emissive at peak; the game dims it by day"
        self.lights[halo.name] = "its halo shell (alpha 0.2)"
        return core, halo

    # ------------------------------------------------------------ mesh plumbing

    def mesh(self, name, fill, mat, *, smooth=False, small=False, far=False, parent=None, post=None):
        me = bpy.data.meshes.new(f"{self.kind}_{name}")
        bm = bmesh.new()
        fill(bm)
        if post:
            for v in bm.verts:
                v.co = Vector(post(tuple(v.co)))
        bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        uv = bm.loops.layers.uv.new("UVMap")
        for f in bm.faces:   # planar UVs at 1 m per unit, by the face's dominant axis
            n = f.normal
            ax = max(range(3), key=lambda i: abs(n[i]))
            for loop in f.loops:
                c = loop.vert.co
                loop[uv].uv = (c.y, c.z) if ax == 0 else ((c.x, c.z) if ax == 1 else (c.x, c.y))
        bm.to_mesh(me)
        bm.free()
        if smooth:
            for p in me.polygons:
                p.use_smooth = True
        obj = bpy.data.objects.new(f"{self.kind}_{name}", me)
        self.collection.objects.link(obj)
        obj.parent = parent or self.root
        me.materials.append(mat)
        if small:
            self.small.add(obj)
        if far:
            self.far.append(obj)
        return obj

    def socket(self, name, location, meaning, rotation=(0, 0, 0), **extras):
        """A named empty the game reads (an anchor for a plant, a display, a sound)."""
        empty = bpy.data.objects.new(name, None)
        empty.empty_display_type = "ARROWS"
        empty.empty_display_size = 0.3
        self.collection.objects.link(empty)
        empty.parent = self.root
        empty.location = location
        empty.rotation_euler = rotation
        for k, v in extras.items():
            empty[k] = v
        self.sockets[name] = meaning
        self.socket_frames[name] = {"position": [round(c, 3) for c in location],
                                    "rotation_z": round(rotation[2], 4), **extras}
        return empty

    def connector(self, name, position, direction):
        """A snap connector, recorded in the manifest (not a node): where the next module
        joins and which way it continues."""
        self.connectors[name] = {"position": [round(c, 3) for c in position],
                                 "direction": [round(c, 3) for c in direction]}

    def collider(self, shape, centre, size, rotation_z=0.0, slope=0.0, walkable=False):
        """A simple collision proxy, Blender coords: `box` (size x, y, z) or `cylinder`
        (size radius, height). `slope` tilts a box about its local X (stairs as a ramp)."""
        self.colliders.append({"shape": shape, "centre": list(centre), "size": list(size),
                               "rotation_z": rotation_z, "slope": slope, "walkable": walkable})

    # ------------------------------------------------------------ primitives (asset frame)

    def box(self, name, mn, mx, mat, *, round=0.0, seg=3, **kw):
        """An axis-aligned box from corner `mn` to `mx`; `round` rounds its vertical edges."""
        def fill(bm):
            if round > 0:
                outline = rounded_rect(mn[0], mx[0], mn[1], mx[1], round, seg)
                self._extrude(bm, [(x, y, mn[2]) for x, y in outline], (0, 0, mx[2] - mn[2]))
            else:
                bmesh.ops.create_cube(bm, size=1.0)
                for v in bm.verts:
                    v.co = Vector(((mn[0] + mx[0]) / 2 + v.co.x * (mx[0] - mn[0]),
                                   (mn[1] + mx[1]) / 2 + v.co.y * (mx[1] - mn[1]),
                                   (mn[2] + mx[2]) / 2 + v.co.z * (mx[2] - mn[2])))
        return self.mesh(name, fill, mat, **kw)

    @staticmethod
    def _extrude(bm, ring, offset):
        """Cap a closed ring of points and extrude it by `offset` (both caps closed)."""
        a = [bm.verts.new(p) for p in ring]
        b = [bm.verts.new(Vector(p) + Vector(offset)) for p in ring]
        n = len(ring)
        bm.faces.new(a[::-1])
        bm.faces.new(b)
        for i in range(n):
            j = (i + 1) % n
            bm.faces.new((a[i], a[j], b[j], b[i]))

    def plan(self, name, outline_xy, z0, z1, mat, **kw):
        """A plan outline (XY) extruded from z0 to z1."""
        return self.mesh(name, lambda bm: self._extrude(bm, [(x, y, z0) for x, y in outline_xy],
                                                        (0, 0, z1 - z0)), mat, **kw)

    def elevation(self, name, outline_xz, y0, y1, mat, **kw):
        """An elevation outline (XZ) extruded along Y from y0 to y1."""
        return self.mesh(name, lambda bm: self._extrude(bm, [(x, y0, z) for x, z in outline_xz],
                                                        (0, y1 - y0, 0)), mat, **kw)

    def section(self, name, outline_yz, x0, x1, mat, **kw):
        """A cross-section outline (YZ) extruded along X from x0 to x1."""
        return self.mesh(name, lambda bm: self._extrude(bm, [(x0, y, z) for y, z in outline_yz],
                                                        (x1 - x0, 0, 0)), mat, **kw)

    def pierced(self, name, outer_xz, holes_xz, y0, y1, mat, reveal=True, **kw):
        """A wall panel (XZ outline) with openings, extruded along Y from y0 (front) to y1.
        `reveal` closes the openings' sides through the wall's depth."""
        def fill(bm):
            def loop(pts, y):
                vs = [bm.verts.new((x, y, z)) for x, z in pts]
                return vs, [bm.edges.new((vs[i], vs[(i + 1) % len(vs)])) for i in range(len(vs))]
            for y, normal in ((y0, (0, -1, 0)), (y1, (0, 1, 0))):
                edges = loop(outer_xz, y)[1]
                for h in holes_xz:
                    edges += loop(h, y)[1]
                bmesh.ops.triangle_fill(bm, use_beauty=True, use_dissolve=True, edges=edges, normal=normal)
            rings = [outer_xz] + (list(holes_xz) if reveal else [])
            for pts in rings:
                n = len(pts)
                for i in range(n):
                    (xa, za), (xb, zb) = pts[i], pts[(i + 1) % n]
                    vs = [bm.verts.new(c) for c in ((xa, y0, za), (xb, y0, zb), (xb, y1, zb), (xa, y1, za))]
                    bm.faces.new(vs)
        return self.mesh(name, fill, mat, **kw)

    def plan_holes(self, name, outer_xy, holes_xy, z0, z1, mat, **kw):
        """A plan outline (XY) with holes, extruded from z0 to z1: a planter's rim, a ring."""
        return self.pierced(name, outer_xy, holes_xy, z0, z1, mat, post=lambda c: (c[0], c[2], c[1]), **kw)

    def tube(self, name, points, radii, mat, sides=8, cap=True, **kw):
        if not isinstance(radii, (list, tuple)):
            radii = [radii] * len(points)

        def fill(bm):
            pts = [Vector(p) for p in points]
            rings, normal = [], None
            for i, (p, r) in enumerate(zip(pts, radii)):
                a, b = pts[max(i - 1, 0)], pts[min(i + 1, len(pts) - 1)]
                t = (b - a).normalized()
                normal = t.orthogonal().normalized() if normal is None else (normal - t * normal.dot(t)).normalized()
                side = t.cross(normal)
                rings.append([bm.verts.new(p + r * (normal * math.cos(math.tau * k / sides)
                                                    + side * math.sin(math.tau * k / sides))) for k in range(sides)])
            for ra, rb in zip(rings, rings[1:]):
                for k in range(sides):
                    bm.faces.new((ra[k], ra[(k + 1) % sides], rb[(k + 1) % sides], rb[k]))
            if cap:
                bm.faces.new(rings[0][::-1])
                bm.faces.new(rings[-1])
        kw.setdefault("smooth", True)
        return self.mesh(name, fill, mat, **kw)

    def lathe(self, name, profile_rz, mat, steps=24, centre=(0, 0, 0), **kw):
        """Spin an (r, z) profile about the vertical axis through `centre`."""
        def fill(bm):
            cx, cy, cz = centre
            rings = []
            for k in range(steps):
                a = math.tau * k / steps
                rings.append([bm.verts.new((cx + r * math.cos(a), cy + r * math.sin(a), cz + z)) for r, z in profile_rz])
            for k in range(steps):
                ra, rb = rings[k], rings[(k + 1) % steps]
                for i in range(len(profile_rz) - 1):
                    bm.faces.new((ra[i], rb[i], rb[i + 1], ra[i + 1]))
            if profile_rz[0][0] > 1e-6:
                bm.faces.new([r[0] for r in rings][::-1])
            if profile_rz[-1][0] > 1e-6:
                bm.faces.new([r[-1] for r in rings])
        kw.setdefault("smooth", True)
        return self.mesh(name, fill, mat, **kw)

    def sphere(self, name, centre, radius, mat, scale=(1, 1, 1), segments=12, rings=7, jitter=0.0, **kw):
        def fill(bm):
            bmesh.ops.create_uvsphere(bm, u_segments=segments, v_segments=rings, radius=radius)
            for v in bm.verts:
                j = 1 + (self.rng.uniform(-jitter, jitter) if jitter else 0)
                v.co = Vector((centre[0] + v.co.x * scale[0] * j, centre[1] + v.co.y * scale[1] * j,
                               centre[2] + v.co.z * scale[2] * j))
        kw.setdefault("smooth", True)
        return self.mesh(name, fill, mat, **kw)

    def blades(self, name, base, count, length, width, mat, spread=0.25, lean=0.6, curl=0.3, **kw):
        """A tuft of flat tapering blades (grass, agave leaves, reeds) rising from `base`."""
        def fill(bm):
            for k in range(count):
                a = math.tau * k / count + self.rng.uniform(-0.3, 0.3)
                d = Vector((math.cos(a), math.sin(a), 0))
                side = Vector((-d.y, d.x, 0))
                L = length * self.rng.uniform(0.7, 1.1)
                tilt = lean * self.rng.uniform(0.6, 1.2)
                p0 = Vector(base) + d * self.rng.uniform(0, spread)
                pts = []
                for i in range(5):
                    t = i / 4
                    pts.append(p0 + d * (math.sin(tilt) * L * t + curl * L * t * t)
                               + Vector((0, 0, math.cos(tilt) * L * t - curl * 0.6 * L * t * t)))
                row = []
                for i, p in enumerate(pts):
                    w = width * (1 - i / 4.2)
                    row.append((bm.verts.new(p - side * w / 2), bm.verts.new(p + side * w / 2)))
                for (a0, a1), (b0, b1) in zip(row, row[1:]):
                    bm.faces.new((a0, a1, b1, b0))
        return self.mesh(name, fill, mat, small=True, **kw)

    def canopy(self, name, centre, radius, mat, lobes=7, flat=0.7, **kw):
        """A cloud of overlapping, slightly jittered lobes: a stylised leaf mass."""
        def fill(bm):
            for k in range(lobes):
                a = math.tau * k / lobes + self.rng.uniform(-0.4, 0.4)
                d = self.rng.uniform(0.35, 0.75) * radius
                c = Vector(centre) + Vector((math.cos(a) * d, math.sin(a) * d, self.rng.uniform(-0.25, 0.3) * radius))
                if k == 0:
                    c = Vector(centre) + Vector((0, 0, 0.2 * radius))
                r = radius * self.rng.uniform(0.45, 0.65)
                geom = bmesh.ops.create_icosphere(bm, subdivisions=1, radius=r)
                for v in geom["verts"]:
                    v.co = Vector((v.co.x, v.co.y, v.co.z * flat)) * (1 + self.rng.uniform(-0.12, 0.12)) + c
        kw.setdefault("smooth", True)
        return self.mesh(name, fill, mat, **kw)

    def glow_point(self, name, centre, core_r, halo_r, core, halo, shape=(1, 1, 1)):
        """A luminous point: an opaque core inside a translucent halo shell."""
        self.sphere(f"{name}_Core", centre, core_r, core, scale=shape, segments=8, rings=5, small=True)
        self.sphere(f"{name}_Halo", centre, halo_r, halo, scale=shape, segments=12, rings=7, small=True)

    def bell(self, name, top, radius, height, core, halo):
        """A jellyfish bell: a translucent scalloped dome over a bright core and two tendrils."""
        prof = [(0.0, height), (radius * 0.55, height * 0.92), (radius * 0.9, height * 0.6),
                (radius, height * 0.15), (radius * 0.92, 0.0)]
        self.lathe(f"{name}_Halo", prof, halo, steps=14, centre=(top[0], top[1], top[2] - height), small=True)
        self.sphere(f"{name}_Core", (top[0], top[1], top[2] - height * 0.45), radius * 0.32, core,
                    segments=8, rings=5, small=True)
        for k, a in enumerate((0.6, 2.7)):
            x, y = top[0] + math.cos(a) * radius * 0.4, top[1] + math.sin(a) * radius * 0.4
            pts = [(x, y, top[2] - height), (x + 0.02, y, top[2] - height - 0.12), (x - 0.01, y + 0.02, top[2] - height - 0.26)]
            self.tube(f"{name}_Tendril{k}", pts, [0.012, 0.009, 0.004], core, sides=4, small=True)

    # ------------------------------------------------------------ relief motifs (original)

    def strata(self, name, x0, x1, z, heights, y, mat, depth=0.04, gap=0.08):
        """Horizontal strata bands, like canyon sediment, standing proud of a face at y
        (towards -Y). `heights` are the bands' heights, bottom up."""
        objs = []
        zz = z
        for k, h in enumerate(heights):
            objs.append(self.box(f"{name}{k}", (x0, y - depth, zz), (x1, y, zz + h), mat, small=True))
            zz += h + gap
        return objs

    def mesa_steps(self, name, cx, z0, width, steps, rise, y, mat, depth=0.05):
        """A stepped-mesa relief: a stack of narrowing slabs, a canyon skyline in stone."""
        half = width / 2
        tread = half / (steps + 1)
        left, right = [], []
        for k in range(steps):
            xl = cx - half + tread * k
            left += [(xl, z0 + rise * k), (xl, z0 + rise * (k + 1))]
        for k in reversed(range(steps)):
            xr = cx + half - tread * k
            right += [(xr, z0 + rise * (k + 1)), (xr, z0 + rise * k)]
        return self.elevation(name, left + right, y - depth, y, mat, small=True)

    def ripples(self, name, cx, cz, radii, y, mat, depth=0.03, width=0.06, arc=math.pi):
        """Concentric half rings, water ripples carved in relief (opening upwards)."""
        def fill(bm):
            for r in radii:
                ring_o, ring_i = [], []
                n = 16
                for k in range(n + 1):
                    a = (math.pi - arc) / 2 + arc * k / n
                    ring_o.append((cx + (r + width / 2) * math.cos(a), cz + (r + width / 2) * math.sin(a)))
                    ring_i.append((cx + (r - width / 2) * math.cos(a), cz + (r - width / 2) * math.sin(a)))
                for k in range(n):
                    quad = (ring_o[k], ring_o[k + 1], ring_i[k + 1], ring_i[k])
                    f = [bm.verts.new((x, y - depth, z)) for x, z in quad]
                    b = [bm.verts.new((x, y, z)) for x, z in quad]
                    bm.faces.new(f[::-1])
                    for i in range(4):
                        j = (i + 1) % 4
                        bm.faces.new((f[i], f[j], b[j], b[i]))
        return self.mesh(name, fill, mat, small=True)

    # ------------------------------------------------------------ output

    @property
    def model_ref(self):
        return f"{FOLDER}/{self.asset_id}/{self.asset_id}.glb"

    def meshes(self):
        return [o for o in self.collection.objects if o.type == "MESH"]

    @staticmethod
    def triangles(objs):
        return sum(sum(len(p.vertices) - 2 for p in o.data.polygons) for o in objs if o.type == "MESH")

    def bounds(self, objs):
        lo = Vector((1e9, 1e9, 1e9))
        hi = Vector((-1e9, -1e9, -1e9))
        for o in objs:
            for v in o.data.vertices:
                c = o.matrix_world @ v.co
                lo = Vector(map(min, lo, c))
                hi = Vector(map(max, hi, c))
        return lo, hi

    def tag_acoustics(self, near):
        counts = {}
        for obj in near:
            if obj in self.small:
                continue
            m = obj.data.materials[0] if obj.data.materials else None
            cls = self.acoustics.get(m.name) if m else None
            if cls:
                obj["acoustic"] = cls
                counts[cls] = counts.get(cls, 0) + 1
        return counts

    def lod_levels(self):
        """`<id>_LOD0` (and `_LOD1` when the piece has a far level); sockets stay on the root."""
        nodes = []
        for level in range(2 if self.far else 1):
            empty = bpy.data.objects.new(f"{self.asset_id}_LOD{level}", None)
            self.collection.objects.link(empty)
            empty.parent = self.root
            nodes.append(empty)
        for child in list(self.root.children):
            socket = child.type == "EMPTY" and not child.children
            if child in nodes or socket:
                continue
            child.parent = nodes[1] if child in self.far else nodes[0]
        return nodes

    def profile(self, near, acoustic, lo, hi):
        prof = {
            "id": self.asset_id,
            "model": f"{self.model_ref}#Scene0",
            "kind": "structure",
            "family": "city",
            "catalog": CATALOG_PATH[self.catalog],
            "priority": self.priority,
            "phases": self.phases,
            "about": self.about,
            "size_metres": {"width": round(hi.x - lo.x, 3), "depth": round(hi.y - lo.y, 3),
                            "height": round(hi.z - lo.z, 3)},
            "bounds_blender": {"min": [round(c, 3) for c in lo], "max": [round(c, 3) for c in hi]},
            "acoustics": {
                "materials": dict(sorted(self.acoustics.items())),
                "tagged_meshes": acoustic,
                "about": "materials carry glTF extras {\"acoustic\": <preset|\"none\">}; large "
                         "near-level meshes carry node extras {\"acoustic\": <preset>} (what the "
                         "game reads); relief, trim, clutter and plants are left out",
            },
            "triangles": {"near": self.triangles(near)},
            "licence": "CC0-1.0: generated by tools/blender/create_city_kit.py",
        }
        if self.far:
            prof["triangles"]["far"] = self.triangles(self.far)
        if self.sockets:
            prof["sockets"] = self.sockets
        if self.connectors:
            prof["connectors"] = self.connectors
        if self.lights:
            prof["lights"] = self.lights
        if self.notes:
            prof["notes"] = self.notes
        # Authoring data for the manifest and the district (Blender coordinates, Z up).
        prof["kit"] = {"sockets": self.socket_frames, "colliders": self.colliders,
                       "materials": sorted(self.acoustics)}
        return prof

    def finish(self):
        """Save the .blend (with an asset-marked collection of the near level), export the
        .glb, write asset.json and the prefab. Returns the manifest row."""
        near = [o for o in self.meshes() if o not in self.far]
        acoustic = self.tag_acoustics(near)
        lo, hi = self.bounds(near)
        self.lod_levels()
        prof = self.profile(near, acoustic, lo, hi)

        # The asset browser's view: the near level and the sockets, instanced by the
        # district. Objects can sit in two collections; the export collection holds all.
        browse = bpy.data.collections.new(self.asset_id)   # not linked to the scene: an asset only
        for obj in self.collection.objects:
            if obj not in self.far and obj.name != f"{self.asset_id}_LOD1":
                browse.objects.link(obj)
        browse.asset_mark()
        browse.asset_data.catalog_id = CATALOG_ID[self.catalog]
        browse.asset_data.description = self.about
        for tag in (self.catalog, self.priority, *self.phases):
            browse.asset_data.tags.new(tag, skip_if_exists=True)
        browse["asset_id"] = self.asset_id
        browse.use_fake_user = True

        source = PROJECT / f"assets-src/blender/{FOLDER}/{self.asset_id}.blend"
        model = PROJECT / "assets" / self.model_ref
        out = model.parent
        source.parent.mkdir(parents=True, exist_ok=True)
        out.mkdir(parents=True, exist_ok=True)
        if bpy.context.window:
            bpy.context.window.scene = self.scene
        for other in list(bpy.data.scenes):
            if other != self.scene:
                bpy.data.scenes.remove(other)
        bpy.context.preferences.filepaths.save_version = 0   # no .blend1 beside the source
        bpy.ops.wm.save_as_mainfile(filepath=str(source), check_existing=False)
        export_scene(self.scene, model)
        (out / "asset.json").write_text(json.dumps(prof, indent=2) + "\n")
        title = " ".join(w.capitalize() for w in self.asset_id.split("_"))
        (out / f"{self.asset_id}.bsn").write_text(
            f'#"{title}"\n'
            "jackdaw::prefab::components::Prefab\n"
            "jackdaw::prefab::components::PrefabEntityId(0)\n"
            "bevy_transform::components::transform::Transform\n"
            "bevy_camera::visibility::Visibility::Visible\n"
            f'jackdaw_scene_types::types::GltfSource {{ path: "{self.model_ref}", scene_index: 0 }}\n')
        return {"id": self.asset_id, "profile": prof, "colliders": self.colliders}
