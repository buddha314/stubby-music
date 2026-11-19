"""Shared pieces for Omar's models (`docs/art/omar.md`): his palette and a lofted tube,
the one shape gloves, sleeves and limbs are made of.

The palette is the art direction's: midnight-navy gloves, a brightened dark-mint coat,
warm sand trousers, burnt-orange tagelmust, charcoal leather, aged brass.
"""
import math

import bmesh
import bpy
from mathutils import Vector

GLOVE = (0.018, 0.024, 0.07)      # midnight navy (linear)
GLOVE_SEAM = (0.008, 0.011, 0.035)
COAT = (0.09, 0.2, 0.15)          # brightened dark mint
BRASS = (0.55, 0.40, 0.18)
SAND = (0.48, 0.36, 0.2)
ORANGE = (0.8, 0.22, 0.03)
LEATHER = (0.03, 0.028, 0.026)


def tube(kit, name, points, radii, mat, segments=12, cap=True, squash=None):
    """A smooth tube through `points` (Blender space) with a radius per point: a ring of
    `segments` vertices at each point, square to the path, joined and capped. `squash`
    scales each ring along one axis of its frame (x for flat fingers and palms)."""
    points = [Vector(p) for p in points]

    def fill(bm):
        rings = []
        up = Vector((0, 0, 1))
        for i, p in enumerate(points):
            a = points[max(i - 1, 0)]
            b = points[min(i + 1, len(points) - 1)]
            t = (b - a).normalized()
            if abs(t.dot(up)) > 0.95:
                up = Vector((1, 0, 0))
            x = t.cross(up).normalized()
            y = x.cross(t).normalized()
            up = y
            sx, sy = squash or (1.0, 1.0)
            ring = [bm.verts.new(p + radii[i] * (math.cos(k / segments * math.tau) * x * sx
                                                 + math.sin(k / segments * math.tau) * y * sy))
                    for k in range(segments)]
            rings.append(ring)
        for r0, r1 in zip(rings, rings[1:]):
            for k in range(segments):
                bm.faces.new((r0[k], r0[(k + 1) % segments], r1[(k + 1) % segments], r1[k]))
        if cap:
            bm.faces.new(list(reversed(rings[0])))
            bm.faces.new(rings[-1])

    obj = bpy.data.objects.new(f"{kit.kind}_{name}", kit.mesh_from(name, fill))
    kit.link(obj, mat)
    for face in obj.data.polygons:
        face.use_smooth = True
    return obj


def blob(kit, name, centre, radii, mat, segments=16, rings=10):
    """An ellipsoid: palms, puffed trousers, the wrap's crown."""
    obj = kit.sphere(name, 1.0, mat, None, location=centre, segments=segments, rings=rings)
    obj.scale = radii
    return obj


def ring_around(kit, name, centre, axis, radius, thickness, mat, segments=24):
    """A thin band around `axis` at `centre`: glove seams, the sleeve's brass edge."""
    axis = Vector(axis).normalized()
    return tube(kit, name, [Vector(centre) - axis * thickness / 2, Vector(centre) + axis * thickness / 2],
                [radius, radius], mat, segments=segments)


def under(obj, parent):
    """Parent `obj` to `parent`, keeping where it is."""
    obj.parent = parent
    obj.matrix_parent_inverse = parent.matrix_world.inverted()
    return obj
