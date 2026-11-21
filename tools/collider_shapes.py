#!/usr/bin/env python3
"""Collider shapes for the world assets, computed from the exported models.

Solid things the player must not walk through get simple static colliders, the usual
practice (a capsule or cylinder for a tree trunk, a box for a machine), not mesh
colliders. Small things stay walk-through. The shapes are written per asset family
to `assets/<folder>/colliders.json`, ready to be authored on the prefabs with
Jackdaw's Physics module once it is on (`add-desert-opening` 4.2d): each entry maps
to an `AvianCollider(ColliderConstructor::<shape> { .. })` on a child entity of the
prefab at `translation`, with no `RigidBody` (a static body).

Coordinates are the model's own, in Bevy's axes (Y up, forward -Z), metres. Avian's
field names are used: `Cylinder { radius, height }`, `Capsule { radius, height }`
(height of the cylinder part between the caps), `Cuboid { x_length, y_length,
z_length }`.

Re-run after regenerating models:  python3 tools/collider_shapes.py
"""
from pathlib import Path
import json
import math
import struct

import numpy as np

REPO = Path(__file__).resolve().parents[1]
MODELS = REPO / "assets"
PLAYER_STEP_M = 0.35   # what the player steps over; geometry below this stays walkable


def load(glb):
    data = glb.read_bytes()
    n = struct.unpack("<I", data[12:16])[0]
    doc = json.loads(data[20:20 + n])
    binary = data[20 + n + 8:]
    return doc, binary


def positions(doc, binary, mesh):
    out = []
    for prim in doc["meshes"][mesh]["primitives"]:
        acc = doc["accessors"][prim["attributes"]["POSITION"]]
        view = doc["bufferViews"][acc["bufferView"]]
        start = view.get("byteOffset", 0) + acc.get("byteOffset", 0)
        stride = view.get("byteStride", 12)
        raw = np.frombuffer(binary, dtype=np.uint8, count=stride * (acc["count"] - 1) + 12, offset=start)
        pts = np.lib.stride_tricks.as_strided(raw, shape=(acc["count"], 12), strides=(stride, 1)).copy()
        out.append(pts.view(np.float32).reshape(-1, 3))
    return np.concatenate(out) if out else np.zeros((0, 3), np.float32)


def matrix(node):
    if "matrix" in node:
        return np.array(node["matrix"], dtype=np.float64).reshape(4, 4).T
    t = node.get("translation", [0, 0, 0])
    x, y, z, w = node.get("rotation", [0, 0, 0, 1])
    s = node.get("scale", [1, 1, 1])
    # Parts exported at scale 1 but shipped hidden carry `start_scale` (export_gltf.py):
    # collide with the shipped (hidden) pose, as before the export fix.
    start = node.get("extras", {}).get("start_scale")
    if start is not None:
        s = [start] * 3
    r = np.array([[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
                  [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
                  [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]])
    m = np.eye(4)
    m[:3, :3] = r * np.array(s)
    m[:3, 3] = t
    return m


def world_meshes(glb):
    """{node name: world-space vertices} for every mesh node, ignoring scale-0 nodes
    (parts the model ships hidden, such as dormant foliage)."""
    doc, binary = load(glb)
    nodes, result = doc["nodes"], {}

    def walk(index, parent):
        node = nodes[index]
        m = parent @ matrix(node)
        if np.allclose(m[:3, :3], 0):
            return
        if "mesh" in node:
            pts = positions(doc, binary, node["mesh"])
            result[node.get("name", f"node{index}")] = (np.c_[pts, np.ones(len(pts))] @ m.T)[:, :3]
        for child in node.get("children", []):
            walk(child, m)
    for root in doc["scenes"][doc.get("scene", 0)]["nodes"]:
        walk(root, np.eye(4))
    return result


def r4(v):
    return round(float(v), 3)


def cylinder_around(points, top=None):
    """A vertical cylinder from the ground around `points` (its widest reach)."""
    radius = np.max(np.hypot(points[:, 0], points[:, 2]))
    height = top if top is not None else np.max(points[:, 1])
    return {"shape": "Cylinder", "radius": r4(radius), "height": r4(height), "translation": [0, r4(height / 2), 0]}


def cuboid_around(points):
    lo, hi = points.min(axis=0), points.max(axis=0)
    size, centre = hi - lo, (hi + lo) / 2
    return {"shape": "Cuboid", "x_length": r4(size[0]), "y_length": r4(size[1]), "z_length": r4(size[2]),
            "translation": [r4(c) for c in centre]}


def oak(glb):
    """One cylinder around the trunks, 2.2 m tall, so the player stops at the wood and
    cannot push through head-height branches; drooping branch tips stay open.

    Each stem mesh is built trunk-first (create_scrub_oak.py: grow()), so its first
    tube, 5 rings of 6 vertices, is the trunk up to its first split."""
    parts = world_meshes(glb)
    trunks = np.concatenate([p[:30] for name, p in parts.items() if "LimbWood" in name])
    radius = float(np.max(np.hypot(trunks[:, 0], trunks[:, 2]))) + 0.1
    height = 2.2
    return [{"shape": "Cylinder", "radius": r4(radius), "height": height, "translation": [0, height / 2, 0]}]


def stump(glb):
    parts = world_meshes(glb)
    trunk = parts["DesertStump_Trunk"]
    low = trunk[trunk[:, 1] < 0.3]
    return [{"shape": "Cylinder", "radius": r4(np.percentile(np.hypot(low[:, 0], low[:, 2]), 80)),
             "height": r4(np.max(trunk[:, 1])), "translation": [0, r4(np.max(trunk[:, 1]) / 2), 0]}]


def module(glb):
    parts = world_meshes(glb)
    return [cuboid_around(np.concatenate(list(parts.values())))]


def burbler(glb):
    """The pool is hot mud: a cylinder the player cannot wade into. The low stream
    banks stay walkable (under PLAYER_STEP_M)."""
    parts = world_meshes(glb)
    mud = parts["Burbler_Mud"]
    pool = mud[mud[:, 2] < 0.3]   # the round pool; the stream channel runs on toward +Z
    centre = (pool.min(axis=0) + pool.max(axis=0)) / 2
    reach = np.hypot(pool[:, 0] - centre[0], pool[:, 2] - centre[2])
    return [{"shape": "Cylinder", "radius": r4(np.percentile(reach, 95)), "height": 1.0,
             "translation": [r4(centre[0]), 0.5, r4(centre[2])]}]


def boulder(glb):
    """A cuboid around the near mesh, shrunk to 85%: a rounded boulder fills less than
    its box, and a full box would narrow the grotto's alcove (add-echo-grotto)."""
    parts = world_meshes(glb)
    near = np.concatenate([p for name, p in parts.items() if not name.endswith("_Far")])
    box = cuboid_around(near)
    for axis in ("x_length", "y_length", "z_length"):
        box[axis] = r4(box[axis] * 0.85)
    return [box]


def rock_formation(glb):
    """A cuboid per rock of a formation (85%, as a boulder), so the gaps between rocks stay
    open. A bored rock (a window) gets three boxes instead, beside and above its opening,
    from `openings` in the formation's asset.json, so the window stays open too."""
    if not glb.stem.startswith("rock_formation_"):
        return boulder(glb)
    profile = json.loads((glb.parent / "asset.json").read_text())
    parts = world_meshes(glb)
    shapes = []
    for rock in profile["rocks"]:
        near = parts[rock["mesh"]]
        if not rock["bored"]:
            box = cuboid_around(near)
            for axis in ("x_length", "y_length", "z_length"):
                box[axis] = r4(box[axis] * 0.85)
            shapes.append(box)
            continue
        # Blender (x, y, z) is glTF (x, z, -y): an opening at Blender (x, 0, z) is Bevy (x, z).
        ox, _, oz = next(o["at"] for o in profile["openings"]
                         if abs(o["at"][0]) < 1e9 and near[:, 0].min() < o["at"][0] < near[:, 0].max())
        width, height = next((o["width_metres"], o["height_metres"]) for o in profile["openings"]
                             if o["at"][0] == ox)
        for piece in (near[near[:, 0] < ox - width / 2], near[near[:, 0] > ox + width / 2],
                      near[near[:, 1] > oz + height / 2]):
            if len(piece):
                shapes.append(cuboid_around(piece))
    return shapes


def campfire(glb):
    """A cylinder around the bundle of sticks: the player walks around the fire."""
    parts = world_meshes(glb)
    near = np.concatenate([p for name, p in parts.items() if not name.endswith("_Far")])
    return [cylinder_around(near)]


def cottage(glb):
    """A box around the house's walls (the near level's earth and timber), and one around
    the water tank; the porch, mast and roof overhangs stay open."""
    parts = world_meshes(glb)
    def gather(*words):
        found = [p for name, p in parts.items() if any(w in name for w in words) and "Far" not in name]
        return np.concatenate(found) if found else None
    shapes = []
    for words in (("Wall", "Plinth"), ("TankWater", "TankPallet")):
        pts = gather(*words)
        if pts is not None:
            shapes.append(cuboid_around(pts))
    return shapes


FAMILIES = {
    # family folder (under assets/): (model glob under assets/, shaper or why it is walk-through)
    "flora/scrub_oak": ("flora/scrub_oak/*/*.glb", oak),
    "flora/reeds": ("flora/reeds/*/*.glb", "thin reeds: the player walks through, like grass"),
    "props/desert": ("props/desert/*/*.glb", None),
    "props/rocks": ("props/rocks/*/*.glb", rock_formation),
    "structures/campfire": ("structures/campfire/campfire.glb", campfire),
    "structures/cottage": ("structures/cottage/cottage.glb", cottage),
    "machines/mudpit_burbler": ("machines/mudpit_burbler/mudpit_burbler.glb", burbler),
    "creatures/lightning_bug": ("creatures/lightning_bug/lightning_bug.glb", "a small flying creature, collected with E"),
    "streams/stream_segment": ("streams/stream_segment/stream_segment.glb", "low banks under the step height; the player wades across"),
    "streams/ripple_pool": ("streams/ripple_pool/ripple_pool.glb", "a low rim and still water; the player steps in to collect"),
    "machines/reed_water_purifier": ("machines/reed_water_purifier/reed_water_purifier.glb", "a low dam across the stream; the player wades over it as over the stream"),
}


def modules(glb):
    if glb.stem in ("burbler_clock", "volume", "mixer", "vcf", "scope"):
        return "a rack (HUD) module only, never placed in the world"
    return module(glb)


def props(glb):
    name = glb.stem
    if "stump" in name:
        return stump(glb)
    return "brush and mud debris: the player walks through them"


def main():
    families = dict(FAMILIES)
    for model in sorted((MODELS / "modules").glob("*/*.glb")):
        families[model.parent.relative_to(MODELS).as_posix()] = (model.relative_to(MODELS).as_posix(), modules)
    for family, (pattern, shaper) in families.items():
        entries = {}
        for glb in sorted(MODELS.glob(pattern)):
            rel = glb.relative_to(MODELS).as_posix()
            how = shaper if shaper is not None else props
            result = how(glb) if callable(how) else how
            entries[rel] = ({"colliders": result, "rigid_body": "none (static)"} if isinstance(result, list)
                            else {"walk_through": True, "why": result})
        if not entries:
            raise SystemExit(f"{family}: no models matched {pattern}")
        out = REPO / "assets" / family / "colliders.json"
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps({
            "about": "Static collider shapes for the prefabs; see tools/collider_shapes.py. Each "
                     "shape is an AvianCollider(ColliderConstructor) on a child entity at "
                     "`translation`, model coordinates (Bevy: Y up), metres.",
            "models": entries}, indent=2) + "\n")
        print(f"{out.relative_to(REPO)}: {len(entries)} models")


if __name__ == "__main__":
    main()
