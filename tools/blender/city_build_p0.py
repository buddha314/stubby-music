"""City kit, P0: the core construction kit (foundation, facade, floor and roof, outdoor
hallway, vertical circulation, canal, footbridge, planting). See city_kit.py for the
contract every piece keeps; `create_city_kit.py` runs these."""
import math

import bmesh
from mathutils import Vector

from city_kit import (BAY, BED_Z, SLAB, WALL, WATER_Z, Piece, arch_opening, circle,
                      rounded_rect)

P0 = {}


def asset(fn):
    P0["city_" + fn.__name__] = fn
    return fn


ALL = ("camp", "village", "town", "city")

# ------------------------------------------------------------------ shared facade parts


def facade_band(p, x0=0.0, x1=BAY):
    """The floor band at the top of every storey: the slab edge, standing proud 0.08 m.
    It carries the horizontal line across every facade variant, so variants stack."""
    p.box("Band", (x0, -0.08, 3.7), (x1, WALL, 4.0), p.relief())


def strata_band(p, z=3.2, x0=0.0, x1=BAY):
    p.strata("Strata", x0, x1, z, (0.07, 0.04, 0.1), 0.0, p.relief(), depth=0.035, gap=0.06)


def plinth(p, x0=0.0, x1=BAY):
    p.box("Plinth", (x0, -0.06, 0.0), (x1, WALL, 0.35), p.stone())


def facade_piece(asset_id, kind, about, variant="plaster"):
    p = Piece(asset_id, kind, catalog="facade", priority="P0", phases=("town", "city"), about=about)
    p.connector("start", (0, 0, 0), (1, 0, 0))
    p.connector("end", (BAY, 0, 0), (1, 0, 0))
    p.connector("above", (0, 0, BAY), (0, 0, 1))
    p.notes.append("front faces Blender -Y (glTF +Z); wall from y 0 to 0.4; storey 4 m; stacks on z")
    p.collider("box", (BAY / 2, WALL / 2, BAY / 2), (BAY, WALL, BAY))
    return p


def window_parts(p, x0, x1, z0, z1, r, y_glass=0.26, planter=True):
    """Frame, recessed glass, a stone sill and an (empty) clay sill planter with a socket."""
    opening = rounded_rect(x0, x1, z0, z1, r, 4)
    inner = rounded_rect(x0 + 0.06, x1 - 0.06, z0 + 0.06, z1 - 0.06, max(r - 0.06, 0.02), 4)
    p.pierced("Frame", opening, [inner], y_glass - 0.04, y_glass + 0.02, p.titanium(), small=True)
    p.elevation("Glass", inner, y_glass - 0.01, y_glass, p.glass())
    p.box("Sill", (x0 - 0.12, -0.14, z0 - 0.08), (x1 + 0.12, y_glass - 0.04, z0 + 0.02), p.stone(), small=True)
    if planter:
        p.box("Planter", (x0 - 0.05, -0.5, z0 - 0.5), (x1 + 0.05, -0.14, z0 - 0.1), p.clay(), small=True)
        p.box("PlanterSoil", (x0, -0.46, z0 - 0.1), (x1, -0.18, z0 - 0.08), p.soil(), small=True)
        p.socket("Plant_sill", ((x0 + x1) / 2, -0.32, z0 - 0.1), "sill planter: a small plant or trailing vine",
                 accepts="small")


# ------------------------------------------------------------------ STR01 foundation


@asset
def paving_4():
    p = Piece("city_paving_4", "CityPaving4", catalog="foundation", priority="P0", phases=ALL,
              about="A 4 x 4 m paving tile, top at its origin's level; laid on the 4 m grid.")
    p.box("Slab", (0, 0, -SLAB), (BAY, BAY, 0), p.paving())
    p.connector("east", (BAY, 0, 0), (1, 0, 0))
    p.connector("north", (0, BAY, 0), (0, 1, 0))
    p.collider("box", (2, 2, -SLAB / 2), (BAY, BAY, SLAB), walkable=True)
    return p


@asset
def paving_8():
    p = Piece("city_paving_8", "CityPaving8", catalog="foundation", priority="P0", phases=ALL,
              about="An 8 x 8 m paving tile: four 4 m tiles in one draw call, inlaid with a strata band.")
    p.box("Slab", (0, 0, -SLAB), (8, 8, 0), p.paving())
    # An inlaid strata line of darker stone, flush: a separate thin strip just above the slab.
    p.box("Inlay", (0, 3.85, 0), (8, 4.15, 0.01), p.stone_dark(), small=True)
    p.connector("east", (8, 0, 0), (1, 0, 0))
    p.connector("north", (0, 8, 0), (0, 1, 0))
    p.collider("box", (4, 4, -SLAB / 2), (8, 8, SLAB), walkable=True)
    return p


@asset
def curb_4():
    p = Piece("city_curb_4", "CityCurb4", catalog="foundation", priority="P0", phases=ALL,
              about="A 4 m kerb: a raised stone edge (0.15 m) along the front of a paving run.")
    p.box("Kerb", (0, -0.3, -0.3), (BAY, 0, 0.15), p.stone_dark(), round=0.0)
    p.box("KerbTop", (0, -0.32, 0.15), (BAY, 0.0, 0.17), p.stone(), small=True)
    p.connector("start", (0, 0, 0), (1, 0, 0))
    p.connector("end", (BAY, 0, 0), (1, 0, 0))
    p.collider("box", (2, -0.15, -0.07), (BAY, 0.3, 0.44))
    return p


@asset
def curb_corner():
    p = Piece("city_curb_corner", "CityCurbCorner", catalog="foundation", priority="P0", phases=ALL,
              about="An outside kerb corner, rounded (r 0.3), joining a run along +X to one along +Y.")
    outline = [(-0.3, 0), (-0.3, -0.0)]
    # A quarter-round block in the outside corner (x < 0, y < 0).
    pts = [(0, 0)] + [(-0.3 * math.cos(a), -0.3 * math.sin(a)) for a in
                      (math.radians(90 * k / 6) for k in range(7))]
    pts = [(0, 0), (0, -0.3)] + [(-0.3 * math.sin(math.radians(90 * k / 6)), -0.3 * math.cos(math.radians(90 * k / 6))) for k in range(1, 7)]
    p.plan("Kerb", pts, -0.3, 0.15, p.stone_dark())
    p.plan("KerbTop", [(x * 1.07, y * 1.07) for x, y in pts], 0.15, 0.17, p.stone(), small=True)
    p.connector("x", (0, 0, 0), (1, 0, 0))
    p.connector("y", (0, 0, 0), (0, 1, 0))
    return p


@asset
def plaza_steps_4():
    p = Piece("city_plaza_steps_4", "CityPlazaSteps4", catalog="foundation", priority="P0", phases=ALL,
              about="Broad plaza steps, 4 m wide, rising 1 m in five 0.2 m risers over 2 m: "
                    "from a lower paving level up to the next (front faces -Y, climbs towards +Y).")
    outline = [(-0.0, -SLAB), (2.0, -SLAB), (2.0, 1.0)]
    for k in range(5, 0, -1):
        y = 0.4 * (k - 1)
        outline += [(y, 0.2 * k), (y, 0.2 * (k - 1))] if False else []
    # Section outline (Y, Z), climbing from y 0 (z 0) to y 2 (z 1).
    sec = [(0, -SLAB), (2.0, -SLAB), (2.0, 1.0)]
    for k in range(5, 0, -1):
        sec += [(0.4 * (k - 1), 0.2 * k), (0.4 * (k - 1), 0.2 * (k - 1))]
    p.section("Steps", sec, 0, BAY, p.stone())
    for k in range(5):
        p.box(f"Nosing{k}", (0, 0.4 * k, 0.2 * (k + 1) - 0.03), (BAY, 0.4 * k + 0.06, 0.2 * (k + 1) + 0.005),
              p.stone_dark(), small=True)
    p.connector("bottom", (0, 0, 0), (0, -1, 0))
    p.connector("top", (0, 2, 1), (0, 1, 0))
    p.collider("box", (2, 1, 0.35), (BAY, 2.2, 0.3), slope=math.atan2(1, 2), walkable=True)
    return p


# ------------------------------------------------------------------ STR02 facade, variant A (plaster)


@asset
def facade_blank():
    p = facade_piece("city_facade_blank", "CityFacadeBlank",
                     "A blank 4 x 4 m facade bay of pale lime plaster: a stepped-mesa relief panel, "
                     "strata bands and a clay rain spout. Variant A (plaster).")
    p.box("Wall", (0, 0, 0), (BAY, WALL, 3.7), p.plaster())
    facade_band(p)
    strata_band(p)
    p.mesa_steps("Mesa", 2.0, 1.1, 2.4, 4, 0.28, 0.0, p.relief(), depth=0.05)
    p.ripples("Ripples", 2.0, 0.55, (0.25, 0.45), 0.0, p.relief())
    p.tube("Spout", [(3.4, 0.1, 3.55), (3.4, -0.45, 3.5)], 0.07, p.clay(), sides=6, small=True)
    p.socket("Vine_top", (2.0, -0.1, 3.7), "a vine strip hung from the floor band", accepts="vine")
    return p


@asset
def facade_window():
    p = facade_piece("city_facade_window", "CityFacadeWindow",
                     "A 4 x 4 m plaster facade bay with one deep, round-cornered window (1.6 x 2.0 m, "
                     "sill at 0.9 m), recessed glass, a stone sill and a clay sill planter. Variant A.")
    x0, x1, z0, z1 = 1.2, 2.8, 0.9, 2.9
    p.pierced("Wall", [(0, 0), (BAY, 0), (BAY, 3.7), (0, 3.7)], [rounded_rect(x0, x1, z0, z1, 0.35, 4)],
              0.0, WALL, p.plaster())
    window_parts(p, x0, x1, z0, z1, 0.35)
    facade_band(p)
    strata_band(p, z=3.25)
    return p


@asset
def facade_door():
    p = facade_piece("city_facade_door", "CityFacadeDoor",
                     "A ground-floor 4 x 4 m plaster bay with a round-headed doorway: a 1.2 x 2.1 m "
                     "timber door, a glazed fanlight, a stone threshold and a pale lantern.")
    opening = arch_opening(1.1, 2.9, 0.0, 2.35, 10)
    p.pierced("Wall", [(0, 0), (BAY, 0), (BAY, 3.7), (0, 3.7)], [opening], 0.0, WALL, p.plaster())
    plinth(p, 0.0, 1.12)
    plinth(p, 2.88, BAY)
    # The door leaf and its fanlight sit 0.3 m back in the reveal.
    p.box("DoorLeaf", (1.4, 0.28, 0.02), (2.6, 0.34, 2.12), p.timber())
    p.box("DoorFrame", (1.3, 0.27, 0.0), (2.7, 0.36, 2.22), p.grown(), small=True)
    fan = [(x, z) for x, z in arch_opening(1.12, 2.88, 2.22, 2.35, 10)[2:]] + [(1.12, 2.22), (2.88, 2.22)]
    fan = [(2.88, 2.22), (2.88, 2.35)] + [(2.0 + 0.88 * math.cos(math.pi * k / 10), 2.35 + 0.88 * math.sin(math.pi * k / 10)) for k in range(1, 10)] + [(1.12, 2.35), (1.12, 2.22)]
    p.elevation("Fanlight", fan, 0.3, 0.31, p.glass())
    p.box("Threshold", (1.0, -0.25, 0.0), (3.0, 0.4, 0.04), p.stone(), small=True)
    core, halo = p.lantern()
    p.box("LanternArm", (3.25, -0.25, 2.45), (3.35, 0.0, 2.5), p.titanium(), small=True)
    p.glow_point("Lantern", (3.3, -0.3, 2.3), 0.06, 0.16, core, halo, shape=(1, 1, 1.4))
    facade_band(p)
    strata_band(p, z=3.25)
    p.socket("Door", (2.0, -0.1, 0.0), "the doorway at threshold height; the way in", accepts="door")
    p.colliders.clear()
    p.collider("box", (0.55, WALL / 2, 2), (1.1, WALL, 4))
    p.collider("box", (3.45, WALL / 2, 2), (1.1, WALL, 4))
    p.collider("box", (2, WALL / 2, 3.35), (1.8, WALL, 1.3))
    return p


@asset
def facade_arcade():
    p = facade_piece("city_facade_arcade", "CityFacadeArcade",
                     "A ground-floor arcade bay: a 3 m round-headed arch (spring 2.0 m, crown 3.5 m) "
                     "through a 0.4 m plaster wall, half a 1 m pier each side. Stands at the street "
                     "line in front of a covered walk (city_walk_bay) for a deep shaded arcade.")
    opening = arch_opening(0.5, 3.5, 0.0, 2.0, 14)
    p.pierced("Wall", [(0, 0), (BAY, 0), (BAY, 3.7), (0, 3.7)], [opening], 0.0, WALL, p.plaster())
    plinth(p, 0.0, 0.52)
    plinth(p, 3.48, BAY)
    # A carved keystone voussoir band around the arch head.
    ring_o = [(2.0 + 1.68 * math.cos(math.pi * k / 14), 2.0 + 1.68 * math.sin(math.pi * k / 14)) for k in range(15)]
    ring_i = [(2.0 + 1.5 * math.cos(math.pi * k / 14), 2.0 + 1.5 * math.sin(math.pi * k / 14)) for k in range(15)]
    p.elevation("Archivolt", ring_o + ring_i[::-1], -0.04, 0.0, p.relief(), small=True)
    facade_band(p)
    p.socket("Vine_top", (2.0, -0.1, 3.7), "a vine strip hung from the floor band", accepts="vine")
    p.colliders.clear()
    p.collider("box", (0.25, WALL / 2, 2), (0.5, WALL, 4))
    p.collider("box", (3.75, WALL / 2, 2), (0.5, WALL, 4))
    return p


@asset
def facade_corner():
    p = Piece("city_facade_corner", "CityFacadeCorner", catalog="facade", priority="P0", phases=("town", "city"),
              about="An outside corner pier for one storey: a rounded buttress (r 0.5) centred on "
                    "the footprint corner, wrapping the ends of two facade runs (one along +X, one "
                    "along +Y, both with their fronts outward).")
    pier = [(-0.3, 0.7), (-0.3, 0.2)] + [(-0.3 + 0.5 - 0.5 * math.cos(a), -0.3 + 0.5 - 0.5 * math.sin(a))
                                          for a in (math.radians(90 * k / 6) for k in range(7))][::-1][1:-1] \
        + [(0.2, -0.3), (0.7, -0.3), (0.7, 0.7)]
    pier = [(0.7, 0.7), (-0.3, 0.7), (-0.3, 0.2)]
    for k in range(1, 7):
        a = math.radians(180 + 90 * k / 6)
        pier.append((0.2 + 0.5 * math.cos(a), 0.2 + 0.5 * math.sin(a)))
    pier += [(0.7, -0.3)]
    p.plan("Pier", pier, 0.0, 3.7, p.plaster())
    band = [(0.78, 0.78), (-0.38, 0.78), (-0.38, 0.2)] + \
        [(0.2 + 0.58 * math.cos(math.radians(180 + 90 * k / 6)), 0.2 + 0.58 * math.sin(math.radians(180 + 90 * k / 6))) for k in range(1, 7)] + [(0.78, -0.38)]
    p.plan("Band", band, 3.7, 4.0, p.relief())
    p.plan("Plinth", [(x * 1.0, y * 1.0) for x, y in band], 0.0, 0.35, p.stone(), small=True)
    p.connector("run_x", (0, 0, 0), (1, 0, 0))
    p.connector("run_y", (0, 0, 0), (0, 1, 0))
    p.collider("box", (0.2, 0.2, 2.0), (1.0, 1.0, 4.0))
    return p


@asset
def facade_end():
    p = Piece("city_facade_end", "CityFacadeEnd", catalog="facade", priority="P0", phases=("town", "city"),
              about="An end cap for one storey: a stepped pilaster centred on a run's free end, "
                    "closing the wall's cut edge.")
    p.box("Pilaster", (-0.35, -0.2, 0), (0.35, 0.6, 3.7), p.plaster(), round=0.15)
    p.box("Step", (-0.45, -0.28, 0), (0.45, 0.6, 1.2), p.plaster(), round=0.18)
    p.box("Band", (-0.43, -0.28, 3.7), (0.43, 0.68, 4.0), p.relief())
    p.connector("run", (0, 0, 0), (1, 0, 0))
    p.collider("box", (0, 0.2, 2), (0.9, 0.9, 4))
    return p


# ------------------------------------------------------------------ STR02 facade, variant B (garden glass)


@asset
def facade_window_b():
    p = facade_piece("city_facade_window_b", "CityFacadeWindowB",
                     "Variant B, the garden-glass bay: full-height glazing between grown-timber ribs on "
                     "titanium transoms, a plaster spandrel and floor band shared with variant A, and a "
                     "cantilevered planter trough. Mixes freely with variant A on the same grid.")
    p.box("Spandrel", (0, 0, 0), (BAY, WALL, 0.45), p.plaster())
    p.box("Head", (0, 0, 3.35), (BAY, WALL, 3.7), p.plaster())
    p.box("Glass", (0.3, 0.24, 0.45), (BAY, 0.26, 3.35), p.glass())
    for z in (0.45, 2.3, 3.3):
        p.box(f"Transom{z:.1f}", (0.3, 0.2, z), (BAY, 0.28, z + 0.05), p.titanium(), small=True)
    p.box("Mullion", (2.12, 0.2, 0.45), (2.18, 0.28, 3.35), p.titanium(), small=True)
    # A grown-timber rib, gently swelling: the variant's signature, one per bay at x 0.
    pts = [(0.15, 0.1, 0.0), (0.17, 0.05, 1.3), (0.2, 0.02, 2.6), (0.15, 0.08, 4.0)]
    p.tube("Rib", pts, [0.17, 0.15, 0.15, 0.17], p.grown(), sides=8)
    p.box("Trough", (0.35, -0.6, 0.05), (3.85, -0.02, 0.5), p.clay(), small=True)
    p.box("TroughSoil", (0.42, -0.54, 0.5), (3.78, -0.08, 0.52), p.soil(), small=True)
    p.socket("Plant_0", (1.2, -0.3, 0.52), "trough planting", accepts="small")
    p.socket("Plant_1", (2.9, -0.3, 0.52), "trough planting", accepts="small")
    facade_band(p)
    return p


@asset
def facade_green_b():
    p = facade_piece("city_facade_green_b", "CityFacadeGreenB",
                     "Variant B, the living wall: a plaster bay faced with a grown-timber lattice of "
                     "planting pockets, moss-filled. Plants and vines mount at its sockets.")
    p.box("Wall", (0, 0, 0), (BAY, WALL, 3.7), p.plaster())
    for k in range(4):   # stepped planting ledges
        z = 0.5 + 0.8 * k
        p.box(f"Ledge{k}", (0.25, -0.35, z), (3.75, 0.0, z + 0.12), p.grown(), small=True)
        p.box(f"Moss{k}", (0.3, -0.3, z + 0.12), (3.7, -0.04, z + 0.22), p.moss(), small=True)
        p.socket(f"Plant_{k}", (1.0 + 0.7 * (k % 3), -0.18, z + 0.22), "living-wall pocket", accepts="small")
    for x in (0.25, 2.0, 3.75):
        p.box(f"Stile{x:.2f}", (x - 0.06, -0.38, 0.3), (x + 0.06, 0.0, 3.6), p.grown(), small=True)
    facade_band(p)
    return p


# ------------------------------------------------------------------ STR03 floor and roof


@asset
def slab_4():
    p = Piece("city_slab_4", "CitySlab4", catalog="floor", priority="P0", phases=("town", "city"),
              about="A 4 x 4 m floor slab, top at its origin's level (place at z = 4 k for storey k).")
    p.box("Slab", (0, 0, -SLAB), (BAY, BAY, 0), p.paving())
    p.collider("box", (2, 2, -SLAB / 2), (BAY, BAY, SLAB), walkable=True)
    p.connector("east", (BAY, 0, 0), (1, 0, 0))
    p.connector("north", (0, BAY, 0), (0, 1, 0))
    return p


@asset
def parapet_4():
    p = Piece("city_parapet_4", "CityParapet4", catalog="floor", priority="P0", phases=("town", "city"),
              about="A 4 m roof parapet, 1.1 m, plaster with a stone coping and a clay rain spout; "
                    "sits on a facade's line (front -Y) at the roof level.")
    p.box("Wall", (0, 0, 0), (BAY, WALL, 1.1), p.plaster())
    p.box("Coping", (0, -0.06, 1.1), (BAY, WALL + 0.06, 1.18), p.stone())
    p.tube("Spout", [(1.0, 0.1, 0.15), (1.0, -0.5, 0.1)], 0.07, p.clay(), sides=6, small=True)
    p.socket("Vine_top", (2.0, -0.08, 1.18), "a vine strip hung over the parapet", accepts="vine")
    p.connector("start", (0, 0, 0), (1, 0, 0))
    p.connector("end", (BAY, 0, 0), (1, 0, 0))
    p.collider("box", (2, WALL / 2, 0.59), (BAY, WALL, 1.18))
    return p


@asset
def parapet_step_4():
    p = Piece("city_parapet_step_4", "CityParapetStep4", catalog="floor", priority="P0", phases=("town", "city"),
              about="A 4 m stepped parapet: three rising steps to 1.9 m at the centre, the stepped "
                    "Southwestern skyline. Same line and ends as city_parapet_4.")
    outline = [(0, 0), (BAY, 0), (BAY, 1.1), (3.2, 1.1), (3.2, 1.4), (2.7, 1.4), (2.7, 1.7), (2.35, 1.7),
               (2.35, 1.9), (1.65, 1.9), (1.65, 1.7), (1.3, 1.7), (1.3, 1.4), (0.8, 1.4), (0.8, 1.1), (0, 1.1)]
    p.elevation("Wall", outline, 0.0, WALL, p.plaster())
    p.box("Coping", (0, -0.06, 1.1), (0.8, WALL + 0.06, 1.18), p.stone(), small=True)
    p.box("Coping2", (3.2, -0.06, 1.1), (BAY, WALL + 0.06, 1.18), p.stone(), small=True)
    p.box("CrownCoping", (1.6, -0.06, 1.9), (2.4, WALL + 0.06, 1.98), p.stone(), small=True)
    p.ripples("Sun", 2.0, 1.15, (0.22, 0.36), 0.0, p.relief())
    p.connector("start", (0, 0, 0), (1, 0, 0))
    p.connector("end", (BAY, 0, 0), (1, 0, 0))
    p.collider("box", (2, WALL / 2, 0.59), (BAY, WALL, 1.18))
    return p


@asset
def parapet_corner():
    p = Piece("city_parapet_corner", "CityParapetCorner", catalog="floor", priority="P0", phases=("town", "city"),
              about="The corner pier's cap above the roof: the rounded buttress continuing 1.6 m "
                    "above the roof line, stepped once. Place on a city_facade_corner's top.")
    def pier(r_out, ext):
        pts = [(0.7 + ext, 0.7 + ext), (-0.3 - ext, 0.7 + ext), (-0.3 - ext, 0.2)]
        for k in range(1, 7):
            a = math.radians(180 + 90 * k / 6)
            pts.append((0.2 + r_out * math.cos(a), 0.2 + r_out * math.sin(a)))
        pts += [(0.7 + ext, -0.3 - ext)]
        return pts
    p.plan("Pier", pier(0.5, 0.0), 0.0, 1.2, p.plaster())
    p.plan("Step", pier(0.35, -0.15), 1.2, 1.6, p.plaster())
    p.plan("Cap", pier(0.4, -0.1), 1.6, 1.68, p.stone(), small=True)
    p.collider("box", (0.2, 0.2, 0.8), (1.0, 1.0, 1.6))
    return p


@asset
def roof_cap_4():
    p = Piece("city_roof_cap_4", "CityRoofCap4", catalog="floor", priority="P0", phases=("town", "city"),
              about="A 4 x 4 m roof: the slab with a gravel-ballast top and a drain, top at 0.06 m.")
    p.box("Slab", (0, 0, -SLAB), (BAY, BAY, 0), p.plaster())
    p.box("Ballast", (0.05, 0.05, 0.0), (BAY - 0.05, BAY - 0.05, 0.06), p.stone_dark())
    p.box("Drain", (1.85, 1.85, 0.06), (2.15, 2.15, 0.08), p.titanium(), small=True)
    p.collider("box", (2, 2, -0.12), (BAY, BAY, 0.36), walkable=True)
    return p


@asset
def roof_planted_4():
    p = Piece("city_roof_planted_4", "CityRoofPlanted4", catalog="floor", priority="P0", phases=("town", "city"),
              about="A 4 x 4 m planted roof insert: a stone-kerbed tray of soil and meadow on the "
                    "roof slab, with four planting sockets (shrubs, grasses, a small tree).")
    p.box("Slab", (0, 0, -SLAB), (BAY, BAY, 0), p.plaster())
    ring = rounded_rect(0.1, BAY - 0.1, 0.1, BAY - 0.1, 0.3, 3)
    inner = rounded_rect(0.3, BAY - 0.3, 0.3, BAY - 0.3, 0.15, 3)
    def kerb(bm):
        for z, pts in ((0.0, ring),):
            pass
    p.plan("Kerb", ring, 0.0, 0.45, p.stone())
    p.plan("Soil", inner, 0.45, 0.47, p.soil())
    p.plan("Meadow", rounded_rect(0.35, BAY - 0.35, 0.35, BAY - 0.35, 0.12, 3), 0.47, 0.5, p.moss())
    for k, (x, y) in enumerate(((1.1, 1.1), (2.9, 1.2), (1.3, 2.9), (2.8, 2.8))):
        p.socket(f"Plant_{k}", (x, y, 0.5), "roof planting", accepts="tree" if k == 3 else "shrub")
    p.collider("box", (2, 2, 0.1), (BAY, BAY, 0.8), walkable=True)
    return p


# ------------------------------------------------------------------ STR04 outdoor hallway


def _column(p, x, y, h, name="Column"):
    prof = [(0.3, 0.0), (0.3, 0.12), (0.22, 0.2), (0.2, h - 0.5), (0.24, h - 0.3), (0.36, h - 0.12), (0.36, h)]
    p.lathe(name, prof, p.stone(), steps=16, centre=(x, y, 0))


@asset
def walk_bay():
    p = Piece("city_walk_bay", "CityWalkBay", catalog="walk", priority="P0", phases=("village", "town", "city"),
              about="A 4 m bay of outdoor hallway, 3 m deep: a stone deck and one column at its "
                    "front start. It is covered by whatever sits above (the next storey's bay, or "
                    "city_walk_roof). Run along +X; finish a run with city_walk_end.")
    p.box("Deck", (0, 0, -SLAB), (BAY, 3.0, 0), p.paving())
    p.box("Edge", (0, -0.08, -0.45), (BAY, 0.0, 0.0), p.relief(), small=True)
    _column(p, 0.35, 0.35, 3.7)
    p.box("Beam", (0, 0.1, 3.4), (BAY, 0.6, 3.7), p.plaster())
    p.socket("Vine_top", (2.0, 0.05, 3.4), "a vine strip hung from the walk's beam", accepts="vine")
    p.connector("start", (0, 0, 0), (1, 0, 0))
    p.connector("end", (BAY, 0, 0), (1, 0, 0))
    p.connector("above", (0, 0, BAY), (0, 0, 1))
    p.collider("box", (2, 1.5, -SLAB / 2), (BAY, 3.0, SLAB), walkable=True)
    p.collider("cylinder", (0.35, 0.35, 1.85), (0.3, 3.7))
    return p


@asset
def walk_open():
    p = Piece("city_walk_open", "CityWalkOpen", catalog="walk", priority="P0", phases=("village", "town", "city"),
              about="A 4 m open walk bay, 3 m deep: a timber deck under a pergola of two arched "
                    "grown-timber ribs for vines. For the top storey or the canal-side promenade.")
    p.box("Deck", (0, 0, -SLAB), (BAY, 3.0, 0), p.timber())
    p.box("Edge", (0, -0.06, -0.45), (BAY, 0.0, 0.0), p.stone(), small=True)
    for x in (0.3, 2.3):
        pts = [(x, 0.3, 0.0), (x, 0.25, 2.2), (x, 0.6, 3.1), (x, 1.5, 3.45), (x, 2.4, 3.1), (x, 2.75, 2.2), (x, 2.7, 0.0)]
        p.tube(f"Rib{x}", pts, [0.14, 0.11, 0.1, 0.09, 0.1, 0.11, 0.14], p.grown(), sides=7)
    for k in range(5):
        y = 0.45 + 0.52 * k
        p.box(f"Lath{k}", (0.0, y - 0.03, 3.28 + 0.12 * math.sin(math.pi * (k + 0.5) / 5)),
              (BAY, y + 0.03, 3.34 + 0.12 * math.sin(math.pi * (k + 0.5) / 5)), p.grown(), small=True)
    p.socket("Vine_top", (2.0, 1.5, 3.4), "vines over the pergola", accepts="vine")
    p.connector("start", (0, 0, 0), (1, 0, 0))
    p.connector("end", (BAY, 0, 0), (1, 0, 0))
    p.collider("box", (2, 1.5, -SLAB / 2), (BAY, 3.0, SLAB), walkable=True)
    return p


@asset
def walk_roof():
    p = Piece("city_walk_roof", "CityWalkRoof", catalog="walk", priority="P0", phases=("village", "town", "city"),
              about="The roof over the top storey of a covered walk: a 4 x 3 m plaster slab at "
                    "3.7-4.0 m with a planted fascia lip. Place at the same origin as the bay below it.")
    p.box("Slab", (0, 0, 3.7), (BAY, 3.0, 4.0), p.plaster())
    p.box("Lip", (0, -0.1, 4.0), (BAY, 0.4, 4.35), p.stone())
    p.box("LipSoil", (0.05, -0.05, 4.3), (BAY - 0.05, 0.35, 4.36), p.soil(), small=True)
    p.socket("Plant_lip", (2.0, 0.15, 4.36), "lip planting, trailing over the edge", accepts="small")
    p.socket("Vine_top", (2.0, -0.12, 4.0), "a vine strip over the fascia", accepts="vine")
    p.collider("box", (2, 1.5, 3.85), (BAY, 3.0, 0.3))
    return p


@asset
def column():
    p = Piece("city_column", "CityColumn", catalog="walk", priority="P0", phases=("village", "town", "city"),
              about="A free-standing stone column, 3.7 m to the soffit, with a flared capital. "
                    "Origin at the centre of its base.")
    prof = [(0.3, 0.0), (0.3, 0.12), (0.22, 0.2), (0.2, 3.2), (0.24, 3.4), (0.36, 3.58), (0.36, 3.7)]
    p.lathe("Shaft", prof, p.stone(), steps=16)
    p.collider("cylinder", (0, 0, 1.85), (0.3, 3.7))
    return p


@asset
def walk_end():
    p = Piece("city_walk_end", "CityWalkEnd", catalog="walk", priority="P0", phases=("village", "town", "city"),
              about="The end of a walk run: the closing column and a pierced side screen 3 m deep "
                    "(stepped openings), at the run's end (x 0 of this piece = x 4 of the last bay).")
    _column(p, 0.0, 0.35, 3.7)
    holes = []
    for k in range(3):
        y0 = 0.8 + 0.7 * k
        holes.append([(y0, 1.0), (y0 + 0.4, 1.0), (y0 + 0.4, 2.6), (y0, 2.6)])
    outline = [(0.6, 0.0), (3.0, 0.0), (3.0, 3.4), (0.6, 3.4)]
    # The screen stands across the run's end, in the YZ plane: built in XZ and swapped.
    p.pierced("Screen", outline, holes, -0.15, 0.15, p.plaster(), post=lambda c: (c[1], c[0], c[2]))
    p.box("Beam", (-0.2, 0.1, 3.4), (0.2, 3.0, 3.7), p.plaster())
    p.collider("box", (0, 1.8, 1.7), (0.3, 2.4, 3.4))
    return p


# ------------------------------------------------------------------ STR05 vertical circulation


@asset
def stair_flight():
    p = Piece("city_stair_flight", "CityStairFlight", catalog="circulation", priority="P0",
              phases=("village", "town", "city"),
              about="A straight stair flight 4 m long and 2 m wide, rising 2 m in ten 0.2 m risers "
                    "(0.4 m treads) along +X, on a solid masonry base. Two flights and a landing "
                    "climb one 4 m storey (switchback).")
    sec = [(0, 0)]
    for k in range(10):
        sec += [(0.4 * k, 0.2 * (k + 1)), (0.4 * (k + 1), 0.2 * (k + 1))]
    sec += [(4.0, 0.0)]
    p.elevation("Flight", sec, 0.0, 2.0, p.stone())
    for k in range(10):
        p.box(f"Nosing{k}", (0.4 * k, 0.0, 0.2 * (k + 1) - 0.03), (0.4 * k + 0.05, 2.0, 0.2 * (k + 1) + 0.005),
              p.stone_dark(), small=True)
    p.connector("bottom", (0, 0, 0), (-1, 0, 0))
    p.connector("top", (4, 0, 2), (1, 0, 0))
    p.collider("box", (2, 1, 0.9), (4.47, 2.0, 0.3), slope=math.atan2(2, 4), walkable=True)
    return p


@asset
def stair_landing():
    p = Piece("city_stair_landing", "CityStairLanding", catalog="circulation", priority="P0",
              phases=("village", "town", "city"),
              about="A 2 x 4 m stair landing on a solid plaster pier, top at 2 m: the turn of a "
                    "switchback between two city_stair_flight pieces.")
    p.box("Pier", (0, 0, 0), (2.0, 4.0, 1.85), p.plaster())
    p.box("Top", (-0.04, -0.04, 1.85), (2.04, 4.04, 2.0), p.stone())
    p.connector("flight_up", (0, 0, 2), (-1, 0, 0))
    p.connector("flight_down", (0, 2, 2), (-1, 0, 0))
    p.collider("box", (1, 2, 1), (2, 4, 2), walkable=True)
    return p


@asset
def ramp_8():
    p = Piece("city_ramp_8", "CityRamp8", catalog="circulation", priority="P0", phases=ALL,
              about="An 8 m accessible ramp, 2 m wide, rising 0.5 m (1:16) along +X, with stone kerbs.")
    sec = [(0, 0), (8, 0), (8, 0.5), (0, 0.0)]
    p.elevation("Ramp", [(0, -0.0), (8, 0.0), (8, 0.5)], 0.0, 2.0, p.paving())
    for y0, y1 in ((-0.2, 0.0), (2.0, 2.2)):
        p.elevation(f"Kerb{y0}", [(0, 0), (8, 0), (8, 0.7), (0, 0.2)], y0, y1, p.stone_dark())
    p.connector("bottom", (0, 0, 0), (-1, 0, 0))
    p.connector("top", (8, 0, 0.5), (1, 0, 0))
    p.collider("box", (4, 1, 0.15), (8.02, 2.0, 0.2), slope=math.atan2(0.5, 8), walkable=True)
    return p


@asset
def guardrail_4():
    p = Piece("city_guardrail_4", "CityGuardrail4", catalog="circulation", priority="P0",
              phases=("village", "town", "city"),
              about="A 4 m guardrail (1.1 m): titanium posts at 2 m, three rods and a grown-timber "
                    "handrail, for walk edges, landings and terraces. Its line is y 0..0.1.")
    for x in (0.1, 2.1):
        p.box(f"Post{x}", (x - 0.04, 0.01, 0.0), (x + 0.04, 0.09, 1.05), p.titanium())
    for z in (0.3, 0.55, 0.8):
        p.tube(f"Rod{z}", [(0, 0.05, z), (BAY, 0.05, z)], 0.012, p.titanium(), sides=5, small=True)
    p.tube("Handrail", [(0, 0.05, 1.08), (BAY, 0.05, 1.08)], 0.045, p.grown(), sides=7)
    p.connector("start", (0, 0, 0), (1, 0, 0))
    p.connector("end", (BAY, 0, 0), (1, 0, 0))
    p.collider("box", (2, 0.05, 0.55), (BAY, 0.1, 1.1))
    return p


@asset
def lift():
    p = Piece("city_lift", "CityLift", catalog="circulation", priority="P0", phases=("town", "city"),
              about="A lift placeholder, one 4 m storey of shaft (stack it): a 2.4 x 2.4 m glass "
                    "and titanium shaft with a call point at its door (-Y). Origin at the centre "
                    "of its base. The car and its motion are runtime work.")
    for sx in (-1, 1):
        for sy in (-1, 1):
            p.box(f"Post{sx}{sy}", (sx * 1.2 - 0.06, sy * 1.2 - 0.06, 0), (sx * 1.2 + 0.06, sy * 1.2 + 0.06, BAY), p.titanium())
    for z in (0.0, 3.85):
        p.box(f"Ring{z}", (-1.2, -1.2, z), (1.2, 1.2, z + 0.15), p.graphene())
    p.box("GlassBack", (-1.15, 1.13, 0.15), (1.15, 1.15, 3.85), p.glass())
    p.box("GlassL", (-1.15, -1.15, 0.15), (-1.13, 1.15, 3.85), p.glass())
    p.box("GlassR", (1.13, -1.15, 0.15), (1.15, 1.15, 3.85), p.glass())
    p.box("DoorL", (-1.15, -1.15, 0.15), (-0.55, -1.13, 3.85), p.glass())
    p.box("DoorR", (0.55, -1.15, 0.15), (1.15, -1.13, 3.85), p.glass())
    core, halo = p.signal()
    p.box("CallPlate", (1.25, -1.3, 1.0), (1.45, -1.24, 1.4), p.graphene(), small=True)
    p.glow_point("Light_call", (1.35, -1.33, 1.2), 0.03, 0.07, core, halo)
    p.socket("Interact", (0.0, -1.4, 1.0), "call the lift", accepts="interact")
    p.socket("Car", (0.0, 0.0, 0.15), "where the runtime lift car stands (placeholder)", accepts="lift_car")
    p.connector("above", (0, 0, BAY), (0, 0, 1))
    p.collider("box", (0, 0, 2), (2.4, 2.4, 4))
    return p


# ------------------------------------------------------------------ WAT01 canal


def bank(p, length):
    """A canal bank tile: promenade paving behind a 0.6 m stone wall whose face is the
    water line (y 0); coping 0.12 m proud of the paving; the wall runs down to the bed."""
    p.box("Paving", (0, -BAY, -SLAB), (length, -0.6, 0), p.paving())
    p.box("Wall", (0, -0.6, -2.2), (length, 0.0, 0.0), p.stone())
    p.box("Coping", (0, -0.65, 0.0), (length, 0.1, 0.12), p.stone_dark())
    for k, z in enumerate((-0.55, -0.35, -1.6)):   # strata on the water face, proud towards +Y
        p.box(f"Strata{k}", (0, 0.0, z), (length, 0.035, z + (0.08 if k != 1 else 0.05)), p.relief(), small=True)
    for k in range(int(length // BAY)):
        p.socket(f"Reeds_{k}", (BAY * k + 2.0, 0.5, WATER_Z), "reeds at the water's edge", accepts="reeds")
        p.socket(f"Plant_{k}", (BAY * k + 2.0, -2.2, 0.0), "promenade planting", accepts="tree" if k % 2 == 0 else "shrub")
    p.connector("start", (0, 0, 0), (1, 0, 0))
    p.connector("end", (length, 0, 0), (1, 0, 0))
    p.notes.append("water side is +Y; the wall face is the water line at y 0; the canal is 8 m "
                   "between faces (two water tiles); promenade z 0, water z -0.9, bed z -2.0")
    p.collider("box", (length / 2, -2.3, -SLAB / 2), (length, 3.4, SLAB), walkable=True)
    p.collider("box", (length / 2, -0.25, -1.04), (length, 0.75, 2.32))


@asset
def canal_bank_4():
    p = Piece("city_canal_bank_4", "CityCanalBank4", catalog="canal", priority="P0", phases=("village", "town", "city"),
              about="A 4 m canal bank tile: 4 m of promenade behind a stone wall with strata on its "
                    "water face, coping, reeds and planting sockets. Origin at the coping line.")
    bank(p, BAY)
    return p


@asset
def canal_bank_8():
    p = Piece("city_canal_bank_8", "CityCanalBank8", catalog="canal", priority="P0", phases=("village", "town", "city"),
              about="An 8 m canal bank tile (two 4 m tiles in one draw call).")
    bank(p, 8.0)
    return p


@asset
def canal_corner_outer():
    p = Piece("city_canal_corner_outer", "CityCanalCornerOuter", catalog="canal", priority="P0",
              phases=("village", "town", "city"),
              about="A convex bank corner (the inside of a 90-degree bend): water on its +Y and "
                    "+X sides, the wall rounded (r 1 m) round the corner. Footprint x 0..4, y -4..0.")
    c = (3.0, -1.0)   # wall and coping arcs share this centre

    def arc(r, a0, a1, seg=6):
        return [(c[0] + r * math.cos(math.radians(a0 + (a1 - a0) * k / seg)),
                 c[1] + r * math.sin(math.radians(a0 + (a1 - a0) * k / seg))) for k in range(seg + 1)]

    def band(ro, ri):
        """The L-shaped band between radius ro (outside) and ri (inside) round the corner."""
        return ([(0, c[1] + ro)] + arc(ro, 90, 0) + [(c[0] + ro, -BAY), (c[0] + ri, -BAY)]
                + arc(ri, 0, 90) + [(0, c[1] + ri)])
    p.plan("Paving", [(0, -BAY), (3.4, -BAY)] + arc(0.4, 0, 90) + [(0, -0.6)], -SLAB, 0.0, p.paving())
    p.plan("Wall", band(1.0, 0.4), -2.2, 0.0, p.stone())
    p.plan("Coping", band(1.1, 0.35), 0.0, 0.12, p.stone_dark())
    r = 1.0
    # Water and bed in the corner the rounding leaves open.
    p.box("Bed", (BAY - r, -r, BED_Z - 0.2), (BAY, 0.0, BED_Z), p.bed(), small=True)
    p.box("Water", (BAY - r, -r, WATER_Z - 0.01), (BAY, 0.0, WATER_Z), p.water(), small=True)
    p.socket("Plant_0", (1.6, -2.0, 0.0), "corner planting", accepts="tree")
    p.connector("bank_x", (0, 0, 0), (-1, 0, 0))
    p.connector("bank_y", (BAY, -BAY, 0), (0, -1, 0))
    p.collider("box", (2, -2, -1.04), (BAY, BAY, 2.32), walkable=True)
    return p


@asset
def canal_corner_inner():
    p = Piece("city_canal_corner_inner", "CityCanalCornerInner", catalog="canal", priority="P0",
              phases=("village", "town", "city"),
              about="A concave bank corner (the outside of a bend): a full 4 x 4 m promenade tile "
                    "whose NE corner carries the coping block joining the two walls that meet there "
                    "(water lies diagonally beyond its NE corner).")
    p.box("Paving", (0, -BAY, -SLAB), (BAY, 0, 0), p.paving())
    p.box("Block", (BAY - 0.6, -0.6, -2.2), (BAY, 0.0, 0.0), p.stone())
    p.box("Coping", (BAY - 0.65, -0.65, 0.0), (BAY + 0.1, 0.1, 0.12), p.stone_dark())
    p.socket("Plant_0", (1.5, -1.5, 0.0), "corner planting", accepts="tree")
    p.collider("box", (2, -2, -SLAB / 2), (BAY, BAY, SLAB), walkable=True)
    return p


@asset
def canal_end():
    p = Piece("city_canal_end", "CityCanalEnd", catalog="canal", priority="P0", phases=("village", "town", "city"),
              about="The canal's head: an 8 m bank across the canal's end, where the old camp stream "
                    "arrives along a 1 m channel through the promenade and spills over a stone weir "
                    "lip into the canal. Pair with city_mud_relic upstream.")
    p.box("PavingL", (0, -BAY, -SLAB), (3.4, -0.6, 0), p.paving())
    p.box("PavingR", (4.6, -BAY, -SLAB), (8.0, -0.6, 0), p.paving())
    p.box("WallL", (0, -0.6, -2.2), (3.5, 0.0, 0.0), p.stone())
    p.box("WallR", (4.5, -0.6, -2.2), (8.0, 0.0, 0.0), p.stone())
    p.box("WallUnder", (3.5, -0.6, -2.2), (4.5, 0.0, -0.4), p.stone())
    p.box("CopingL", (0, -0.65, 0.0), (3.4, 0.1, 0.12), p.stone_dark())
    p.box("CopingR", (4.6, -0.65, 0.0), (8.0, 0.1, 0.12), p.stone_dark())
    # The channel: clean stone sides, a clear-water floor, the weir lip and the spill.
    p.box("ChannelL", (3.4, -BAY, -0.5), (3.5, 0.1, 0.12), p.stone_dark(), small=True)
    p.box("ChannelR", (4.5, -BAY, -0.5), (4.6, 0.1, 0.12), p.stone_dark(), small=True)
    p.box("ChannelBed", (3.5, -BAY, -0.5), (4.5, -0.0, -0.4), p.bed())
    p.box("ChannelWater", (3.5, -BAY, -0.2), (4.5, 0.0, -0.19), p.water(), small=True)
    p.box("Weir", (3.5, -0.1, -0.4), (4.5, 0.12, -0.18), p.stone(), small=True)
    def spill(bm):
        rows = []
        for i in range(7):
            t = i / 6
            y = 0.12 + 0.35 * math.sin(t * math.pi / 2)
            z = -0.18 - (-0.18 - WATER_Z) * t * t
            rows.append((bm.verts.new((3.55, y, z)), bm.verts.new((4.45, y, z))))
        for (a0, a1), (b0, b1) in zip(rows, rows[1:]):
            bm.faces.new((a0, a1, b1, b0))
    p.mesh("Spill", spill, p.water(), small=True)
    p.ripples("Ripples", 4.0, -1.5, (0.6, 0.9, 1.2), 0.0, p.relief())
    p.socket("Inflow", (4.0, -BAY, -0.2), "where the camp stream arrives (staff and notes come in here)",
             accepts="stream")
    p.socket("Audio_spill", (4.0, 0.3, WATER_Z), "the spill's water sound", accepts="audio")
    p.connector("bank_start", (0, 0, 0), (1, 0, 0))
    p.connector("bank_end", (8, 0, 0), (1, 0, 0))
    p.connector("stream", (4, -BAY, 0), (0, -1, 0))
    p.collider("box", (1.7, -2.3, -SLAB / 2), (3.4, 3.4, SLAB), walkable=True)
    p.collider("box", (6.3, -2.3, -SLAB / 2), (3.4, 3.4, SLAB), walkable=True)
    return p


@asset
def canal_water_4():
    p = Piece("city_canal_water_4", "CityCanalWater4", catalog="canal", priority="P0", phases=("village", "town", "city"),
              about="A 4 x 4 m canal water tile: the clear water surface at -0.9 m over the canal bed "
                    "at -2.0 m. Two side by side make the 8 m canal. Origin at the near wall's line.")
    p.box("Bed", (0, 0, BED_Z - 0.2), (BAY, BAY, BED_Z), p.bed())
    def surface(bm):
        v = [bm.verts.new(c) for c in ((0, 0, WATER_Z), (BAY, 0, WATER_Z), (BAY, BAY, WATER_Z), (0, BAY, WATER_Z))]
        bm.faces.new(v)
    p.mesh("Water", surface, p.water())
    p.notes.append("the water surface is a single translucent quad (glTF alpha blend); ripples, "
                   "reflection and flow are runtime shader work, not baked")
    return p


@asset
def canal_water_8():
    p = Piece("city_canal_water_8", "CityCanalWater8", catalog="canal", priority="P0", phases=("village", "town", "city"),
              about="An 8 x 8 m canal water tile: the full canal width over 8 m of its length.")
    p.box("Bed", (0, 0, BED_Z - 0.2), (8, 8, BED_Z), p.bed())
    def surface(bm):
        v = [bm.verts.new(c) for c in ((0, 0, WATER_Z), (8, 0, WATER_Z), (8, 8, WATER_Z), (0, 8, WATER_Z))]
        bm.faces.new(v)
    p.mesh("Water", surface, p.water())
    return p


@asset
def canal_steps():
    p = Piece("city_canal_steps", "CityCanalSteps", catalog="canal", priority="P0", phases=("village", "town", "city"),
              about="A 4 m bank tile with broad access steps down to a ledge just above the water "
                    "(-0.75 m), between two stone cheek walls: for sitting, reeds and listening.")
    sec = [(-BAY, -2.2), (0.0, -2.2), (0.0, -0.85), (0.1, -0.85), (0.1, -0.75), (-1.05, -0.75),
           (-1.05, -0.6), (-1.5, -0.6), (-1.5, -0.4), (-1.95, -0.4), (-1.95, -0.2), (-2.4, -0.2),
           (-2.4, 0.0), (-BAY, 0.0)]
    p.section("Steps", sec, 0.3, BAY - 0.3, p.stone())
    for x0, x1 in ((0.0, 0.3), (BAY - 0.3, BAY)):
        p.box(f"Cheek{x0}", (x0, -BAY, -2.2), (x1, 0.0, 0.0), p.stone())
        p.box(f"Coping{x0}", (x0 - 0.02, -2.4, 0.0), (x1 + 0.02, 0.1, 0.12), p.stone_dark(), small=True)
    p.socket("Reeds_0", (0.8, 0.6, WATER_Z), "reeds beside the steps", accepts="reeds")
    p.socket("Seat", (2.0, -0.6, -0.75), "a place to sit at the water (Dancers, the player)", accepts="seat")
    p.connector("start", (0, 0, 0), (1, 0, 0))
    p.connector("end", (BAY, 0, 0), (1, 0, 0))
    p.collider("box", (2, -1.6, -0.4), (3.4, 3.0, 0.3), slope=math.atan2(0.75, 2.4), walkable=True)
    return p


@asset
def mud_relic():
    p = Piece("city_mud_relic", "CityMudRelic", catalog="canal", priority="P0", phases=("camp", "village", "town", "city"),
              about="An 8 x 4 m tile of promenade keeping the original mud stream as a relic at the "
                    "city's approach: the 1 m camp channel down its middle, with its cracked crust and "
                    "dry reed stubble, framed by a low stone kerb. The stream runs along +Y into "
                    "city_canal_end, whose inflow it meets at its centre (x 4).")
    p.box("PavingL", (0, 0, -SLAB), (3.4, BAY, 0), p.paving())
    p.box("PavingR", (4.6, 0, -SLAB), (8.0, BAY, 0), p.paving())
    p.box("KerbL", (3.4, 0, -0.5), (3.5, BAY, 0.1), p.stone_dark(), small=True)
    p.box("KerbR", (4.5, 0, -0.5), (4.6, BAY, 0.1), p.stone_dark(), small=True)
    def channel(bm):
        rows = []
        for i in range(9):
            y = BAY * i / 8
            wob = 0.06 * math.sin(i * 1.3)
            prof = [(3.5, -0.1), (3.62 + wob, -0.3), (4.0 + wob, -0.38), (4.38 + wob, -0.3), (4.5, -0.1)]
            rows.append([bm.verts.new((x, y, z)) for x, z in prof])
        for a, b in zip(rows, rows[1:]):
            for k in range(len(a) - 1):
                bm.faces.new((a[k], a[k + 1], b[k + 1], b[k]))
    p.mesh("Mud", channel, p.mud(), smooth=True)
    for k in range(7):
        x = 3.65 + 0.7 * p.rng.random()
        y = 0.3 + 3.4 * k / 6
        poly = circle(x, y, 0.12 + 0.06 * p.rng.random(), 5)
        p.plan(f"Crust{k}", poly, -0.37, -0.33, p.crust(), small=True)
    p.blades("Stubble", (3.55, 2.0, -0.12), 9, 0.25, 0.03, p.grass(), spread=0.1, lean=0.2)
    p.socket("Stream_in", (4.0, 0.0, -0.3), "where the camp stream enters the relic", accepts="stream")
    p.socket("Stream_out", (4.0, BAY, -0.3), "where it leaves for the canal head", accepts="stream")
    p.socket("Plant_0", (1.5, 2.0, 0.0), "approach planting", accepts="tree")
    p.socket("Plant_1", (6.5, 2.0, 0.0), "approach planting", accepts="shrub")
    p.connector("upstream", (4, 0, 0), (0, -1, 0))
    p.connector("downstream", (4, BAY, 0), (0, 1, 0))
    p.collider("box", (1.7, 2, -SLAB / 2), (3.4, BAY, SLAB), walkable=True)
    p.collider("box", (6.3, 2, -SLAB / 2), (3.4, BAY, SLAB), walkable=True)
    return p


# ------------------------------------------------------------------ STR06 footbridge


def bridge(p, planted):
    L0, L1 = -0.65, 8.65

    def top(y):
        return 0.12 + 0.6 * math.sin(math.pi * (y - L0) / (L1 - L0))

    ys = [L0 + (L1 - L0) * k / 16 for k in range(17)]
    soffit = [(8.0 - 8.0 * k / 16, -0.6 + 0.85 * math.sin(math.pi * (1 - k / 16))) for k in range(17)]
    sec = [(y, top(y)) for y in ys] + [(L1, 0.12), (8.0, 0.12)] + soffit + [(0.0, 0.12), (L0, 0.12)]
    sec = [(y, top(y)) for y in ys[1:-1]][::-1] + [(L0, 0.12), (0.0, 0.12)] + soffit[::-1] + [(8.0, 0.12), (L1, 0.12)]
    p.section("Body", sec, 0.3, BAY - 0.3, p.stone())
    deck = [(y, top(y) + 0.04) for y in ys] + [(y, top(y)) for y in ys][::-1]
    p.section("Deck", deck, 0.3, BAY - 0.3, p.timber())
    for x0, x1 in ((0.0, 0.3), (BAY - 0.3, BAY)):
        wall = [(y, top(y) + 0.55) for y in ys][::-1] + [(L0, 0.12), (0.0, 0.12)] + soffit[::-1] + [(8.0, 0.12), (L1, 0.12)]
        wall = [(L1, 0.12), (8.0, 0.12)] + soffit + [(0.0, 0.12), (L0, 0.12)] + [(y, top(y) + 0.55) for y in ys]
        p.section(f"Spandrel{x0}", wall, x0, x1, p.plaster())
        cap = [(y, top(y) + 0.6) for y in ys] + [(y, top(y) + 0.55) for y in ys][::-1]
        p.section(f"Coping{x0}", cap, x0 - 0.03, x1 + 0.03, p.stone(), small=True)
        xc = (x0 + x1) / 2
        rail = [(xc, y, top(y) + 1.05) for y in ys]
        p.tube(f"Handrail{x0}", rail, 0.04, p.grown(), sides=6)
        for y in ys[1:-1:3]:
            p.tube(f"Post{x0}_{y:.1f}", [(xc, y, top(y) + 0.6), (xc, y, top(y) + 1.05)], 0.025, p.titanium(),
                   sides=5, small=True)
        if planted:
            trough = [(y, top(y) + 0.82) for y in ys[2:-2]] + [(y, top(y) + 0.6) for y in ys[2:-2]][::-1]
            out = x0 - 0.35 if x0 == 0.0 else x1
            p.section(f"Trough{x0}", trough, out, out + 0.35, p.clay(), small=True)
            soil = [(y, top(y) + 0.83) for y in ys[2:-2]] + [(y, top(y) + 0.78) for y in ys[2:-2]][::-1]
            p.section(f"TroughSoil{x0}", soil, out + 0.03, out + 0.32, p.soil(), small=True)
            for k, y in enumerate(ys[3:-3:4]):
                p.socket(f"Plant_{'L' if x0 == 0 else 'R'}{k}", (out + 0.17, y, top(y) + 0.83),
                         "trough planting on the bridge", accepts="small")
            p.socket(f"Vine_{'L' if x0 == 0 else 'R'}", (out + (0.0 if x0 == 0 else 0.35), 4.0, top(4.0) + 0.6),
                     "vines over the bridge's side", accepts="vine",
                     rotation=(0, 0, math.pi / 2 if x0 == 0 else -math.pi / 2))
    core, halo = p.lantern()
    for y in (L0 + 0.2, L1 - 0.2):
        for x in (0.15, BAY - 0.15):
            p.glow_point(f"Lamp{x:.1f}_{y:.1f}", (x, y, top(y) + 0.78), 0.05, 0.13, core, halo, shape=(1, 1, 1.3))
    p.socket("Audio_crossing", (2.0, 4.0, top(4.0)), "footsteps and the water below", accepts="audio")
    p.connector("near_bank", (0, L0, 0.12), (0, -1, 0))
    p.connector("far_bank", (0, L1, 0.12), (0, 1, 0))
    p.notes.append("spans the 8 m canal: y 0 and 8 are the two wall faces; bears on both copings "
                   "(y -0.65..0.1 and 7.9..8.65); deck crown 0.76 m, soffit crown 0.25 m, "
                   "clearance over the water about 1.1 m at the crown")
    p.collider("box", (2, 4, 0.5), (3.4, 9.3, 0.3), walkable=True)
    p.collider("box", (0.15, 4, 0.7), (0.3, 9.3, 1.2))
    p.collider("box", (BAY - 0.15, 4, 0.7), (0.3, 9.3, 1.2))


@asset
def footbridge_8():
    p = Piece("city_footbridge_8", "CityFootbridge8", catalog="bridge", priority="P0", phases=("village", "town", "city"),
              about="An 8 m clear-span footbridge, 4 m wide: a stone barrel arch with plaster "
                    "spandrels, a timber deck, grown-timber handrails and four pale lanterns. "
                    "Origin at the near wall face's line (y 0), left edge (x 0).")
    bridge(p, planted=False)
    return p


@asset
def footbridge_planted_8():
    p = Piece("city_footbridge_planted_8", "CityFootbridgePlanted8", catalog="bridge", priority="P0",
              phases=("town", "city"),
              about="The planted footbridge: city_footbridge_8 with clay troughs cantilevered off "
                    "both parapets, planting and vine sockets.")
    bridge(p, planted=True)
    return p


@asset
def bridge_abutment():
    p = Piece("city_bridge_abutment", "CityBridgeAbutment", catalog="bridge", priority="P0", phases=("village", "town", "city"),
              about="A bridge approach on the bank: two flared, stepped plaster wing walls and a "
                    "stone apron, 4 m wide, 2.5 m deep, ending at the coping (y 0). Place one at "
                    "each end of a footbridge (the far one turned 180 degrees).")
    p.box("Apron", (0.0, -2.5, 0.0), (BAY, -0.65, 0.04), p.stone(), small=True)
    for x0, sx in ((0.0, -1), (BAY, 1)):
        outline = [(x0, -0.65), (x0 + sx * 0.35, -0.65), (x0 + sx * 0.75, -2.5), (x0, -2.5)]
        if sx < 0:
            outline = outline[::-1]
        p.plan(f"Wing{sx}", outline, 0.0, 0.7, p.plaster())
        p.plan(f"WingCap{sx}", outline, 0.7, 0.76, p.stone(), small=True)
        p.socket(f"Plant_{'L' if sx < 0 else 'R'}", (x0 + sx * 0.35, -2.0, 0.76), "planting on the wing wall",
                 accepts="small")
    p.collider("box", (-0.2, -1.6, 0.38), (0.4, 1.9, 0.76))
    p.collider("box", (BAY + 0.2, -1.6, 0.38), (0.4, 1.9, 0.76))
    return p


# ------------------------------------------------------------------ VEG01 planting


@asset
def planter_4():
    p = Piece("city_planter_4", "CityPlanter4", catalog="planting", priority="P0", phases=ALL,
              about="A 4 x 1 m stone planter with a seat-height rim (0.5 m) and soil, four planting "
                    "sockets. Plants are separate assets placed at its sockets.")
    ring = rounded_rect(0, BAY, 0, 1.0, 0.25, 3)
    p.plan("Rim", ring, 0.0, 0.5, p.stone())
    p.plan("Soil", rounded_rect(0.12, BAY - 0.12, 0.12, 0.88, 0.15, 3), 0.5, 0.52, p.soil())
    p.box("Band", (0.1, -0.02, 0.15), (BAY - 0.1, 0.0, 0.25), p.clay(), small=True)
    for k in range(4):
        p.socket(f"Plant_{k}", (0.5 + k, 0.5, 0.52), "planter planting", accepts="tree" if k == 1 else "shrub")
    p.collider("box", (2, 0.5, 0.26), (BAY, 1.0, 0.52))
    return p


@asset
def agave():
    p = Piece("city_agave", "CityAgave", catalog="planting", priority="P0", phases=ALL,
              about="An agave rosette, 1.2 m across: thick blue-green leaves with dark tips.")
    def leaves(bm):
        n = 16
        for k in range(n):
            ring = k // 8
            a = math.tau * k / 8 + ring * 0.4 + p.rng.uniform(-0.1, 0.1)
            d = Vector((math.cos(a), math.sin(a), 0))
            s = Vector((-d.y, d.x, 0))
            L = (0.75 if ring == 0 else 0.6) * p.rng.uniform(0.9, 1.1)
            up = 0.9 if ring == 0 else 1.5
            base = Vector((0, 0, 0.05))
            mid = base + d * L * 0.5 + Vector((0, 0, up * L * 0.55))
            tip = base + d * L + Vector((0, 0, up * L * 0.75 - 0.08))
            w = 0.09
            v = [bm.verts.new(c) for c in (base - s * w * 0.6, base + s * w * 0.6, mid + s * w, tip,
                                          mid - s * w, mid + Vector((0, 0, 0.035)))]
            bm.faces.new((v[0], v[1], v[2], v[5]))
            bm.faces.new((v[5], v[2], v[3]))
            bm.faces.new((v[0], v[5], v[3], v[4]))
            bm.faces.new((v[0], v[4], v[3], v[2], v[1]))
    p.mesh("Leaves", leaves, p.agave(), smooth=False)
    p.sphere("Heart", (0, 0, 0.12), 0.12, p.agave(), scale=(1, 1, 1.4), segments=8, rings=5, small=True)
    p.lathe("Far", [(0.0, 0.8), (0.55, 0.4), (0.6, 0.05), (0.0, 0.0)], p.agave(), steps=6, far=True, smooth=False)
    return p


@asset
def grass():
    p = Piece("city_grass", "CityGrass", catalog="planting", priority="P0", phases=ALL,
              about="A clump of desert bunch grass, 0.7 m: dry and green blades together.")
    p.blades("Dry", (0, 0, 0), 22, 0.7, 0.035, p.grass(), spread=0.12, lean=0.35)
    p.blades("Green", (0, 0, 0), 14, 0.6, 0.035, p.leaf_young(), spread=0.1, lean=0.25)
    p.lathe("Far", [(0.0, 0.6), (0.3, 0.3), (0.12, 0.0), (0.0, 0.0)], p.grass(), steps=5, far=True, smooth=False)
    return p


@asset
def shrub():
    p = Piece("city_shrub", "CityShrub", catalog="planting", priority="P0", phases=ALL,
              about="A flowering shrub, 1.3 m: a rounded leaf mass on short grown-wood stems, with "
                    "clay-coral blooms.")
    for k in range(3):
        a = math.tau * k / 3
        p.tube(f"Stem{k}", [(0, 0, 0), (0.15 * math.cos(a), 0.15 * math.sin(a), 0.45)], [0.05, 0.03],
               p.grown(), sides=5, small=True)
    p.canopy("Leaves", (0, 0, 0.75), 0.6, p.leaf(), lobes=6, flat=0.75)
    def blooms(bm):
        for k in range(18):
            a = p.rng.uniform(0, math.tau)
            e = p.rng.uniform(0.1, 1.1)
            c = Vector((math.cos(a) * math.cos(e), math.sin(a) * math.cos(e), math.sin(e) * 0.8)) * 0.62 + Vector((0, 0, 0.78))
            g = bmesh.ops.create_icosphere(bm, subdivisions=1, radius=0.06)
            for v in g["verts"]:
                v.co += c
    p.mesh("Blooms", blooms, p.bloom(), small=True)
    p.sphere("Far", (0, 0, 0.75), 0.65, p.leaf(), scale=(1, 1, 0.8), segments=7, rings=4, far=True)
    return p


@asset
def tree():
    p = Piece("city_tree", "CityTree", catalog="planting", priority="P0", phases=("village", "town", "city"),
              about="A street and plaza tree, 7 m: a grown-wood trunk forking into three limbs under "
                    "a layered, rounded canopy (the scrub oak's grown cousin, for paving and planters).")
    p.tube("Trunk", [(0, 0, 0), (0.05, 0.0, 1.2), (0.0, 0.05, 2.4)], [0.28, 0.2, 0.17], p.grown(), sides=9)
    tips = []
    for k in range(3):
        a = math.tau * k / 3 + 0.4
        tip = (1.6 * math.cos(a), 1.6 * math.sin(a), 4.6)
        tips.append(tip)
        p.tube(f"Limb{k}", [(0.0, 0.05, 2.3), (0.7 * math.cos(a), 0.7 * math.sin(a), 3.4), tip],
               [0.15, 0.1, 0.05], p.grown(), sides=7)
    p.canopy("Crown", (0, 0, 5.1), 2.6, p.leaf(), lobes=10, flat=0.62)
    for k, tip in enumerate(tips):
        p.canopy(f"Cluster{k}", (tip[0], tip[1], tip[2] + 0.4), 1.5, p.leaf_young() if k == 0 else p.leaf(),
                 lobes=4, flat=0.7)
    p.tube("FarTrunk", [(0, 0, 0), (0, 0, 3.2)], [0.25, 0.15], p.grown(), sides=5, far=True, smooth=False)
    p.sphere("FarCrown", (0, 0, 5.0), 2.6, p.leaf(), scale=(1, 1, 0.65), segments=8, rings=5, far=True)
    p.collider("cylinder", (0, 0, 1.5), (0.3, 3.0))
    return p


@asset
def vine_strip():
    p = Piece("city_vine_strip", "CityVineStrip", catalog="planting", priority="P0", phases=("village", "town", "city"),
              about="A 4 m curtain of trailing vines, hanging 2.6 m from its origin (the top front edge "
                    "of a parapet, band or beam). Linear along +X; hangs in front (-Y).")
    def strands(bm):
        n = 14
        for k in range(n):
            x = BAY * (k + 0.5) / n + p.rng.uniform(-0.1, 0.1)
            L = p.rng.uniform(1.4, 2.6)
            prev = None
            for i in range(9):
                t = i / 8
                c = Vector((x + 0.08 * math.sin(t * 5 + k), -0.06 - 0.12 * t, -L * t))
                # a leaf card every step, alternating sides
                s = 1 if i % 2 else -1
                v = [bm.verts.new(c + Vector(d)) for d in ((0, 0, 0), (s * 0.13, -0.05, -0.05),
                                                         (s * 0.1, -0.08, -0.16), (0, -0.02, -0.12))]
                bm.faces.new(v)
    p.mesh("Leaves", strands, p.leaf(), small=True)
    def stems(bm):
        n = 14
        for k in range(n):
            pass
    p.box("Runner", (0, -0.08, -0.06), (BAY, -0.02, 0.0), p.grown(), small=True)
    p.box("Far", (0, -0.12, -2.2), (BAY, -0.1, 0.0), p.leaf(), far=True)
    p.connector("start", (0, 0, 0), (1, 0, 0))
    p.connector("end", (BAY, 0, 0), (1, 0, 0))
    return p
