"""Placement layouts shared by the City district and the mixed-use assemblies: a list of
(piece, position, rotation about Z) in Blender coordinates, with frames for building-local
placement. No Blender needed."""
from pathlib import Path
import json
import math
import random

ROOT = Path(__file__).resolve().parents[2]


def manifest():
    return json.loads((ROOT / "docs/art/city-kit/manifest.json").read_text())


MANIFEST = manifest()


class Layout:
    """Placements: (asset id, world position, rotation about Z in degrees, group)."""

    def __init__(self):
        self.items = []
        self.masses = []      # blockout boxes: (name, min, max, group)
        self.dancers = []     # (shape, tone, pos, rot, clip)
        self.rng = random.Random(4)

    def put(self, aid, pos, rot=0.0, group="District", frame=None):
        if aid not in MANIFEST:
            raise KeyError(aid)
        if frame:
            pos, rot = frame.world(pos, rot)
        self.items.append({"id": aid, "pos": [round(c, 4) for c in pos], "rot": round(rot % 360, 4), "group": group})

    def mass(self, name, mn, mx, frame=None, group="Blockout"):
        if frame:
            a, _ = frame.world(mn, 0)
            b, _ = frame.world(mx, 0)
            mn = [min(a[i], b[i]) for i in range(3)]
            mx = [max(a[i], b[i]) for i in range(3)]
        self.masses.append((name, list(mn), list(mx), group))


class Frame:
    def __init__(self, origin, rot):
        self.o = type("Origin", (), {"x": origin[0], "y": origin[1], "z": origin[2] if len(origin) > 2 else 0.0})()
        self.rot = rot

    def world(self, pos, rot):
        r = math.radians(self.rot)
        x, y, z = pos
        return (self.o.x + x * math.cos(r) - y * math.sin(r), self.o.y + x * math.sin(r) + y * math.cos(r),
                self.o.z + z), self.rot + rot


