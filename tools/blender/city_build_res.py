"""City kit, residential expansion: mixed-use frontages (retail, office), residential facades
and balconies, dwelling shell, domestic interiors, access cores, communal roofs and distant
proxies. Brief: docs/art/city-residential.md. Same contract as P0 (city_kit.py): 4 m bay,
4 m storey, floor band 3.7-4.0 m, facade wall y 0..0.4 with its front facing -Y.

Furniture, fabric, plants and lamps are separate pieces, placed behind or on the facades:
nothing domestic is baked into structure."""
import math

import bmesh
from mathutils import Vector

from city_kit import BAY, SLAB, WALL, Piece, arch_opening, circle, rounded_rect

RES = {}


def asset(fn):
    RES["city_" + fn.__name__] = fn
    return fn


ROUND = (4.0, 4.0)                       # a rounded corner's centre, in its corner cell
A0, A1 = math.pi, 1.5 * math.pi          # from the side line (x 0) round to the front line (y 0)
ALL = ("village", "town", "city")


def piece(aid, kind, catalog, about, phases=("town", "city")):
    return Piece(aid, kind, catalog=catalog, priority="RES", phases=phases, about=about)


def bay(aid, kind, catalog, about, phases=("town", "city")):
    p = piece(aid, kind, catalog, about, phases)
    p.connector("start", (0, 0, 0), (1, 0, 0))
    p.connector("end", (BAY, 0, 0), (1, 0, 0))
    p.connector("above", (0, 0, BAY), (0, 0, 1))
    p.notes.append("a facade bay: wall y 0..0.4, front -Y, storey 4 m, floor band 3.7-4.0 m")
    p.collider("box", (BAY / 2, WALL / 2, BAY / 2), (BAY, WALL, BAY))
    return p


def band(p, x0=0.0, x1=BAY):
    p.box("Band", (x0, -0.08, 3.7), (x1, WALL, 4.0), p.relief())


def frame_ring(p, name, outline, inset, y0, y1, mat):
    """A frame around an opening: the outline with an inset hole, y0..y1."""
    xs = [x for x, _ in outline]
    zs = [z for _, z in outline]
    inner = rounded_rect(min(xs) + inset, max(xs) - inset, min(zs) + inset, max(zs) - inset, 0.02, 1)
    p.pierced(name, outline, [inner], y0, y1, mat, small=True)
    return inner


def window(p, name, x0, x1, z0, z1, *, y=0.3, r=0.06, frame=None, mullion=None, sill=True):
    """A glazed opening's parts (the wall's hole is cut by the caller): a frame, the glass, a
    sill. `mullion` splits it vertically at that x."""
    frame = frame or p.timber()
    outline = rounded_rect(x0, x1, z0, z1, r, 3)
    inner = frame_ring(p, f"{name}Frame", outline, 0.06, y - 0.03, y + 0.03, frame)
    p.elevation(f"{name}Glass", inner, y - 0.005, y + 0.005, p.glass())
    if mullion is not None:
        p.box(f"{name}Mullion", (mullion - 0.03, y - 0.03, z0), (mullion + 0.03, y + 0.03, z1), frame, small=True)
    if sill:
        p.box(f"{name}Sill", (x0 - 0.08, -0.1, z0 - 0.06), (x1 + 0.08, y - 0.03, z0 + 0.02), p.stone(), small=True)


# ====================================================================== MIX01 retail


@asset
def shop_glazed():
    p = bay("city_shop_glazed", "CityShopGlazed", "retail",
            "A ground-floor shopfront bay: plaster piers, a stone stallriser, a timber-framed "
            "display window and glazed door with a transom, and a sign band with an empty name "
            "panel. Pair with an awning and a display insert behind the glass.")
    p.pierced("Wall", [(0, 0), (BAY, 0), (BAY, 3.7), (0, 3.7)], [[(0.3, 0.0), (3.7, 0.0), (3.7, 3.0), (0.3, 3.0)]],
              0.0, WALL, p.plaster())
    p.box("Stallriser", (0.3, 0.05, 0.0), (2.6, 0.32, 0.45), p.stone())
    window(p, "Display", 0.35, 2.55, 0.45, 2.7, y=0.24, r=0.02, sill=False, mullion=1.45)
    window(p, "Door", 2.75, 3.65, 0.02, 2.7, y=0.24, r=0.02, sill=False)
    window(p, "Transom", 0.35, 3.65, 2.72, 2.98, y=0.24, r=0.02, sill=False)
    p.box("Threshold", (2.7, -0.2, 0.0), (3.7, 0.4, 0.03), p.stone(), small=True)
    p.box("SignPanel", (0.6, -0.04, 3.1), (3.4, 0.0, 3.55), p.graphene(), small=True)
    band(p)
    p.socket("Sign", (2.0, -0.05, 3.32), "the shop's name or sign, written at runtime", accepts="sign",
             rotation=(math.pi / 2, 0, 0))
    p.socket("Door", (3.2, -0.1, 0.0), "the shop door", accepts="door")
    p.socket("Awning", (0.0, 0.0, 0.0), "an awning (city_awning_*) at this origin", accepts="awning")
    p.socket("Display", (0.0, 0.0, 0.0), "a display insert behind the glass (city_shop_display)", accepts="shop_insert")
    p.colliders.clear()
    p.collider("box", (0.15, 0.2, 2), (0.3, 0.4, 4))
    p.collider("box", (3.85, 0.2, 2), (0.3, 0.4, 4))
    p.collider("box", (1.45, 0.2, 1.5), (2.3, 0.3, 3.0))
    return p


@asset
def shop_cafe():
    p = bay("city_shop_cafe", "CityShopCafe", "retail",
            "A cafe front: the same piers and sign band as city_shop_glazed, but a folding glass "
            "front pushed open to the street, with a counter ledge at the opening. Tables go "
            "outside at its Seating socket.")
    p.pierced("Wall", [(0, 0), (BAY, 0), (BAY, 3.7), (0, 3.7)], [[(0.3, 0.0), (3.7, 0.0), (3.7, 3.0), (0.3, 3.0)]],
              0.0, WALL, p.plaster())
    for k, x in enumerate((0.35, 0.65, 3.05, 3.35)):   # bifold leaves stacked at the sides
        p.box(f"Leaf{k}", (x, 0.2 - 0.12 * (k % 2), 0.02), (x + 0.28, 0.24 - 0.12 * (k % 2), 2.7), p.glass())
        p.box(f"LeafFrame{k}", (x, 0.19 - 0.12 * (k % 2), 2.66), (x + 0.28, 0.25 - 0.12 * (k % 2), 2.72), p.timber(), small=True)
    p.box("Ledge", (0.95, 0.05, 1.0), (3.05, 0.45, 1.08), p.timber(), small=True)
    p.box("Head", (0.3, 0.15, 2.7), (3.7, 0.3, 3.0), p.timber())
    p.box("SignPanel", (0.6, -0.04, 3.1), (3.4, 0.0, 3.55), p.graphene(), small=True)
    band(p)
    p.socket("Sign", (2.0, -0.05, 3.32), "the cafe's name, written at runtime", accepts="sign", rotation=(math.pi / 2, 0, 0))
    p.socket("Seating", (2.0, -2.0, 0.0), "outdoor tables (city_bistro_set)", accepts="bistro")
    p.socket("Awning", (0.0, 0.0, 0.0), "an awning at this origin", accepts="awning")
    p.socket("Display", (0.0, 0.0, 0.0), "a counter insert behind the opening", accepts="shop_insert")
    p.colliders.clear()
    p.collider("box", (0.15, 0.2, 2), (0.3, 0.4, 4))
    p.collider("box", (3.85, 0.2, 2), (0.3, 0.4, 4))
    return p


def awning(colour):
    def build():
        p = piece(f"city_awning_{colour.lower()}", f"CityAwning{colour}", "retail",
                  f"A 3.4 m shop awning in {colour.lower()} canvas on titanium arms, sloping out 1.6 m "
                  "from the sign band. Origin at the bay's left front corner (as the shopfront).",
                  phases=ALL)
        cloth = p.fabric(colour)
        def sheet(bm):
            rows = []
            for i in range(6):
                t = i / 5
                y, z = -1.6 * t, 3.0 - 0.55 * t - 0.12 * math.sin(math.pi * t)
                rows.append((bm.verts.new((0.3, y, z)), bm.verts.new((3.7, y, z))))
            for (a0, a1), (b0, b1) in zip(rows, rows[1:]):
                bm.faces.new((a0, a1, b1, b0))
            # the scalloped valance
            v = []
            for k in range(9):
                x = 0.3 + 3.4 * k / 8
                v.append((bm.verts.new((x, -1.6, 2.45)), bm.verts.new((x, -1.6, 2.28 + 0.06 * (k % 2)))))
            for (a0, a1), (b0, b1) in zip(v, v[1:]):
                bm.faces.new((a0, b0, b1, a1))
        p.mesh("Canvas", sheet, cloth)
        for x in (0.35, 3.65):
            p.tube(f"Arm{x}", [(x, 0.0, 2.6), (x, -1.6, 2.45)], 0.02, p.titanium(), sides=4, small=True)
        p.box("Rail", (0.3, -0.06, 2.98), (3.7, 0.0, 3.04), p.titanium(), small=True)
        return p
    build.__name__ = f"awning_{colour.lower()}"
    return asset(build)


for _c in ("Coral", "Teal", "Ochre"):
    awning(_c)


@asset
def shop_display():
    p = piece("city_shop_display", "CityShopDisplay", "retail",
              "A shop interior insert for one bay: a back wall of shelves stocked with goods "
              "(jars, crates, bolts of cloth, instruments' cases), a counter and a pendant lamp, "
              "1.8 m deep behind the glass. Origin as the shopfront's.", phases=ALL)
    p.box("Back", (0.05, 2.1, 0.0), (3.95, 2.2, 3.7), p.interior())
    p.box("Side", (0.0, 0.4, 0.0), (0.06, 2.2, 3.7), p.interior())
    p.box("Floor", (0.0, 0.4, 0.0), (BAY, 2.2, 0.03), p.timber())
    goods = (p.clay(), p.mat("Ochre", None, roughness=0.8), p.mat("Verdigris", None, roughness=0.8), p.canvas())
    for s in range(4):
        z = 0.4 + 0.6 * s
        p.box(f"Shelf{s}", (0.3, 1.75, z), (3.7, 2.1, z + 0.04), p.timber(), small=True)
        for k in range(7):
            x = 0.4 + 0.47 * k
            h = 0.18 + 0.2 * p.rng.random()
            p.box(f"Good{s}_{k}", (x, 1.8, z + 0.04), (x + 0.32, 2.05, z + 0.04 + h), goods[(s + k) % 4], small=True)
    p.box("Counter", (2.4, 0.9, 0.0), (3.7, 1.5, 1.0), p.timber())
    p.box("CounterTop", (2.35, 0.85, 1.0), (3.75, 1.55, 1.05), p.stone(), small=True)
    core, halo = p.hearth()
    p.tube("Cord", [(2.0, 1.3, 3.7), (2.0, 1.3, 2.7)], 0.006, p.railing(), sides=3, small=True)
    p.glow_point("Pendant", (2.0, 1.3, 2.62), 0.08, 0.22, core, halo, shape=(1.4, 1.4, 0.8))
    p.socket("Shopkeeper", (3.0, 1.7, 0.03), "where a Dancer keeps shop", accepts="performer")
    return p


@asset
def bistro_set():
    p = piece("city_bistro_set", "CityBistroSet", "retail",
              "A cafe table for two: a round timber top on a dark iron stem and two chairs. "
              "Origin at the table's centre.", phases=ALL)
    p.lathe("Table", [(0.0, 0.74), (0.38, 0.74), (0.38, 0.7), (0.04, 0.68), (0.04, 0.05), (0.25, 0.02), (0.0, 0.0)],
            p.timber(), steps=14)
    for k, a in enumerate((0.3, math.pi + 0.3)):
        c = Vector((math.cos(a) * 0.62, math.sin(a) * 0.62, 0))
        p.box(f"Seat{k}", tuple(c + Vector((-0.2, -0.2, 0.44))), tuple(c + Vector((0.2, 0.2, 0.48))), p.timber(), small=True)
        back = c + Vector((math.cos(a), math.sin(a), 0)) * 0.2
        p.tube(f"Back{k}", [tuple(back + Vector((0, 0, 0.48))), tuple(back + Vector((0, 0, 0.9)))], 0.02, p.railing(), sides=4, small=True)
        for dx, dy in ((-0.17, -0.17), (0.17, 0.17), (-0.17, 0.17), (0.17, -0.17)):
            p.tube(f"Leg{k}{dx}{dy}", [tuple(c + Vector((dx, dy, 0))), tuple(c + Vector((dx, dy, 0.44)))], 0.015,
                   p.railing(), sides=4, small=True)
    p.socket("Seat_0", (0.62 * math.cos(0.3), 0.62 * math.sin(0.3), 0.48), "a seat", accepts="seat")
    p.socket("Seat_1", (-0.62 * math.cos(0.3), -0.62 * math.sin(0.3), 0.48), "a seat", accepts="seat")
    p.collider("cylinder", (0, 0, 0.37), (0.4, 0.74))
    return p


@asset
def shop_corner_round():
    p = piece("city_shop_corner_round", "CityShopCornerRound", "retail",
              "A rounded shop corner, one storey: a quarter-round glass front (r 4 m) on a stone "
              "stallriser, wrapping a 4 x 4 m corner cell. Its runs continue at x 4 (front, -Y) "
              "and y 4 (side, -X).")
    p.curved_wall("Wall", ROUND, 4.0, 3.6, A0, A1, 0.0, 3.7, p.plaster(),
                  openings=[(0.08, 0.92, 0.45, 3.0)], glass=p.glass(), frame=p.timber(), seg=14)
    p.curved_wall("Band", ROUND, 4.08, 3.6, A0, A1, 3.7, 4.0, p.relief(), seg=14)
    p.socket("Display", (0, 0, 0), "a display insert", accepts="shop_insert")
    p.connector("front", (4, 0, 0), (1, 0, 0))
    p.connector("side", (0, 4, 0), (0, 1, 0))
    p.collider("cylinder", (4, 4, 2), (4.0, 4.0))
    return p


# ====================================================================== MIX02 office


@asset
def office_glazed():
    p = bay("city_office_glazed", "CityOfficeGlazed", "office",
            "An office bay: a plaster spandrel to desk height, then a continuous ribbon of "
            "glazing (0.9-3.4 m) with slim titanium mullions every metre. Reads as work, not home: "
            "wide, unbroken glass and no balcony.")
    p.pierced("Wall", [(0, 0), (BAY, 0), (BAY, 3.7), (0, 3.7)], [[(0.0, 0.9), (BAY, 0.9), (BAY, 3.4), (0.0, 3.4)]],
              0.0, WALL, p.plaster())
    p.box("Glass", (0.0, 0.25, 0.9), (BAY, 0.26, 3.4), p.glass())
    for x in (0.0, 1.0, 2.0, 3.0):
        p.box(f"Mullion{x}", (x, 0.22, 0.9), (x + 0.05, 0.3, 3.4), p.titanium(), small=True)
    for z in (0.9, 3.35):
        p.box(f"Transom{z}", (0.0, 0.22, z), (BAY, 0.3, z + 0.05), p.titanium(), small=True)
    band(p)
    p.socket("Sunshade", (0, 0, 0), "a sunshade (city_sunshade_4) at this origin", accepts="sunshade")
    p.socket("Interior", (0, 0, 0), "an office interior (city_workstation, city_proxy_office)", accepts="office_insert")
    return p


@asset
def office_service():
    p = bay("city_office_service", "CityOfficeService", "office",
            "An opaque office service bay: plaster with a titanium louvre panel (plant and "
            "ventilation) and a strata band. For backs and sides of office floors.")
    p.box("Wall", (0, 0, 0), (BAY, WALL, 3.7), p.plaster())
    for k in range(8):
        z = 1.0 + 0.24 * k
        p.box(f"Louvre{k}", (0.8, -0.12, z), (3.2, 0.0, z + 0.08), p.titanium(), small=True)
    p.strata("Strata", 0.0, BAY, 3.2, (0.07, 0.04), 0.0, p.relief(), depth=0.035)
    band(p)
    return p


@asset
def office_corner_round():
    p = piece("city_office_corner_round", "CityOfficeCornerRound", "office",
              "A rounded office corner, one storey: a curved spandrel and a continuous curved "
              "ribbon of glass (0.9-3.4 m) round a 4 x 4 m corner cell.")
    p.curved_wall("Wall", ROUND, 4.0, 3.6, A0, A1, 0.0, 3.7, p.plaster(),
                  openings=[(0.0, 1.0, 0.9, 3.4)], glass=p.glass(), frame=p.titanium(), seg=12)
    p.curved_wall("Band", ROUND, 4.08, 3.6, A0, A1, 3.7, 4.0, p.relief(), seg=12)
    p.connector("front", (4, 0, 0), (1, 0, 0))
    p.connector("side", (0, 4, 0), (0, 1, 0))
    p.collider("cylinder", (4, 4, 2), (4.0, 4.0))
    return p


@asset
def sunshade_4():
    p = piece("city_sunshade_4", "CitySunshade4", "office",
              "A 4 m brise-soleil for office glazing: three horizontal titanium fins on slim "
              "brackets, 0.7 m out, at the head of the glass. Origin as the facade bay's.")
    for k, z in enumerate((3.2, 2.85, 2.5)):
        p.box(f"Fin{k}", (0.05, -0.75 + 0.1 * k, z), (3.95, -0.1 + 0.05 * k, z + 0.05), p.titanium(), small=k > 0)
    for x in (0.2, 3.8):
        p.box(f"Bracket{x}", (x - 0.03, -0.75, 2.5), (x + 0.03, 0.0, 3.25), p.titanium(), small=True)
    return p


@asset
def workstation():
    p = piece("city_workstation", "CityWorkstation", "office",
              "A workstation: a 1.6 x 0.8 m timber desk, a graphene screen, a chair and a potted "
              "plant. Origin at the desk's centre; the worker faces +Y.", phases=ALL)
    p.box("Top", (-0.8, -0.4, 0.72), (0.8, 0.4, 0.76), p.timber())
    for x in (-0.75, 0.75):
        p.box(f"Leg{x}", (x - 0.03, -0.35, 0.0), (x + 0.03, 0.35, 0.72), p.railing(), small=True)
    p.box("Screen", (-0.35, 0.2, 0.86), (0.35, 0.23, 1.26), p.graphene(), small=True)
    p.box("Stand", (-0.04, 0.2, 0.76), (0.04, 0.25, 0.86), p.titanium(), small=True)
    p.box("Seat", (-0.25, -0.85, 0.44), (0.25, -0.4, 0.5), p.fabric("Slate"), small=True)
    p.box("SeatBack", (-0.25, -0.88, 0.5), (0.25, -0.82, 0.95), p.fabric("Slate"), small=True)
    p.tube("Stem", [(0, -0.62, 0.0), (0, -0.62, 0.44)], 0.025, p.railing(), sides=4, small=True)
    p.lathe("Pot", [(0.0, 0.0), (0.1, 0.0), (0.12, 0.22), (0.0, 0.22)], p.clay(), steps=8, centre=(0.62, 0.22, 0.76), small=True)
    p.canopy("Plant", (0.62, 0.22, 1.08), 0.16, p.leaf(), lobes=3, flat=0.9, small=True)
    p.socket("Worker", (0.0, -0.62, 0.5), "where a Dancer works", accepts="seat")
    return p


@asset
def meeting_table():
    p = piece("city_meeting_table", "CityMeetingTable", "office",
              "A meeting table for six: a 2.4 x 1.0 m timber top on two stone trestles, with "
              "upholstered chairs. Origin at its centre.", phases=ALL)
    p.box("Top", (-1.2, -0.5, 0.72), (1.2, 0.5, 0.77), p.timber())
    for x in (-0.8, 0.8):
        p.box(f"Trestle{x}", (x - 0.06, -0.35, 0.0), (x + 0.06, 0.35, 0.72), p.stone(), small=True)
    for k, (x, y) in enumerate(((-0.8, -0.85), (0, -0.85), (0.8, -0.85), (-0.8, 0.85), (0, 0.85), (0.8, 0.85))):
        s = -1 if y < 0 else 1
        p.box(f"Seat{k}", (x - 0.22, y - 0.22, 0.44), (x + 0.22, y + 0.22, 0.5), p.fabric("Teal"), small=True)
        p.box(f"Back{k}", (x - 0.22, y + s * 0.18, 0.5), (x + 0.22, y + s * 0.24, 0.9), p.fabric("Teal"), small=True)
    return p


# ====================================================================== RES01 residential facades


def res_wall(p, holes, mat=None):
    p.pierced("Wall", [(0, 0), (BAY, 0), (BAY, 3.7), (0, 3.7)], holes, 0.0, WALL, mat or p.plaster())
    band(p)


@asset
def res_window_pair():
    p = bay("city_res_window_pair", "CityResWindowPair", "residential",
            "A home's facade bay: two separate, domestic-sized windows (1.0 x 1.6 m, sill 0.9 m) "
            "in deep reveals, a relief band between them. Variant A: plaster. Curtains and a "
            "room go behind it.", phases=ALL)
    holes = [rounded_rect(0.6, 1.6, 0.9, 2.5, 0.12, 3), rounded_rect(2.4, 3.4, 0.9, 2.5, 0.12, 3)]
    res_wall(p, holes)
    for k, (x0, x1) in enumerate(((0.6, 1.6), (2.4, 3.4))):
        window(p, f"W{k}", x0, x1, 0.9, 2.5, r=0.12, mullion=(x0 + x1) / 2)
        p.socket(f"Curtain_{k}", ((x0 + x1) / 2, 0.36, 0.0), "curtains inside this window", accepts="curtain")
    p.ripples("Ripples", 2.0, 2.65, (0.2, 0.32), 0.0, p.relief())
    p.socket("Interior", (0, 0, 0), "a room (furniture or city_proxy_home_*)", accepts="home_insert")
    return p


@asset
def res_window_pair_b():
    p = bay("city_res_window_pair_b", "CityResWindowPairB", "residential",
            "Variant B of the window pair: the same two openings in a wall faced with vertical "
            "grown-timber battens over the plaster, warmer and quieter.", phases=ALL)
    holes = [rounded_rect(0.6, 1.6, 0.9, 2.5, 0.06, 2), rounded_rect(2.4, 3.4, 0.9, 2.5, 0.06, 2)]
    res_wall(p, holes)
    for k, (x0, x1) in enumerate(((0.6, 1.6), (2.4, 3.4))):
        window(p, f"W{k}", x0, x1, 0.9, 2.5, r=0.06, frame=p.railing())
        p.socket(f"Curtain_{k}", ((x0 + x1) / 2, 0.36, 0.0), "curtains inside this window", accepts="curtain")
    for k in range(17):   # battens, interrupted at the windows
        x = 0.12 + 0.235 * k
        spans = [(0.0, 3.7)]
        for x0, x1 in ((0.6, 1.6), (2.4, 3.4)):
            if x0 - 0.05 < x < x1 + 0.05:
                spans = [(0.0, 0.82), (2.58, 3.7)]
        for j, (za, zb) in enumerate(spans):
            p.box(f"Batten{k}_{j}", (x - 0.04, -0.06, za), (x + 0.04, 0.0, zb), p.grown(), small=True)
    p.socket("Interior", (0, 0, 0), "a room", accepts="home_insert")
    return p


def slider_parts(p, timber):
    holes = [rounded_rect(0.4, 2.6, 0.0, 2.6, 0.1, 3), rounded_rect(3.0, 3.7, 1.0, 2.4, 0.08, 2)]
    for k, x0 in enumerate((0.45, 1.5)):   # two sliding leaves, one drawn half open
        y = 0.24 + 0.06 * k
        p.box(f"Leaf{k}", (x0, y - 0.005, 0.03), (x0 + 1.1, y + 0.005, 2.52), p.glass())
        for z0, z1 in ((0.0, 0.06), (2.5, 2.56)):
            p.box(f"LeafRail{k}{z0}", (x0, y - 0.03, z0), (x0 + 1.1, y + 0.03, z1), timber, small=True)
        for x in (x0, x0 + 1.04):
            p.box(f"LeafStile{k}{x:.2f}", (x, y - 0.03, 0.03), (x + 0.06, y + 0.03, 2.52), timber, small=True)
    window(p, "Side", 3.0, 3.7, 1.0, 2.4, r=0.08, frame=timber)
    p.box("Threshold", (0.4, 0.0, 0.0), (2.6, 0.4, 0.03), p.stone(), small=True)
    p.socket("Curtain_0", (1.5, 0.36, 0.0), "curtains inside the balcony door", accepts="curtain")
    p.socket("Curtain_1", (3.35, 0.36, 0.0), "curtains inside the side window", accepts="curtain")
    p.socket("Balcony", (0, 0, 0), "a balcony (city_balcony_res_4) at this origin", accepts="balcony")
    p.socket("Interior", (0, 0, 0), "a room", accepts="home_insert")
    p.socket("Door", (1.5, -0.1, 0.0), "the balcony door", accepts="door")
    return holes


@asset
def res_slider():
    p = bay("city_res_slider", "CityResSlider", "residential",
            "A home's balcony bay: a 2.2 m sliding glass door, one leaf drawn half open, and a "
            "small side window. Variant A: plaster. Takes a balcony at its origin.", phases=ALL)
    holes = [rounded_rect(0.4, 2.6, 0.0, 2.6, 0.1, 3), rounded_rect(3.0, 3.7, 1.0, 2.4, 0.08, 2)]
    res_wall(p, holes)
    slider_parts(p, p.timber())
    p.colliders.clear()
    p.collider("box", (0.2, 0.2, 2), (0.4, 0.4, 4))
    p.collider("box", (3.3, 0.2, 2), (1.4, 0.4, 4))
    return p


@asset
def res_slider_b():
    p = bay("city_res_slider_b", "CityResSliderB", "residential",
            "Variant B of the balcony bay: the sliding door and side window in a clay-washed "
            "wall with dark rails and a timber lintel.", phases=ALL)
    holes = [rounded_rect(0.4, 2.6, 0.0, 2.6, 0.04, 2), rounded_rect(3.0, 3.7, 1.0, 2.4, 0.04, 2)]
    res_wall(p, holes, mat=p.mat("ClayWash", "brick", roughness=0.9, color="#c79a7c"))
    slider_parts(p, p.railing())
    p.box("Lintel", (0.3, -0.06, 2.6), (2.7, 0.0, 2.85), p.grown(), small=True)
    p.colliders.clear()
    p.collider("box", (0.2, 0.2, 2), (0.4, 0.4, 4))
    p.collider("box", (3.3, 0.2, 2), (1.4, 0.4, 4))
    return p


@asset
def res_loggia():
    p = bay("city_res_loggia", "CityResLoggia", "residential",
            "A recessed loggia: a 3.2 m opening into a 1.4 m deep outdoor room (timber floor, "
            "side walls, soffit), with a glazed door and window at its back wall. Private "
            "outdoor space inside the building line.", phases=ALL)
    res_wall(p, [rounded_rect(0.4, 3.6, 0.0, 3.3, 0.3, 4)])
    p.box("Floor", (0.4, 0.0, 0.0), (3.6, 1.8, 0.05), p.timber())
    p.box("Soffit", (0.4, 0.0, 3.3), (3.6, 1.8, 3.7), p.plaster())
    for x0, x1 in ((0.2, 0.4), (3.6, 3.8)):
        p.box(f"Side{x0}", (x0, 0.4, 0.0), (x1, 1.8, 3.3), p.plaster())
    p.pierced("Back", [(0.4, 0.0), (3.6, 0.0), (3.6, 3.3), (0.4, 3.3)],
              [rounded_rect(0.7, 1.9, 0.05, 2.4, 0.05, 2), rounded_rect(2.3, 3.3, 0.9, 2.4, 0.05, 2)],
              1.8, 2.0, p.plaster())
    window(p, "Door", 0.7, 1.9, 0.05, 2.4, y=1.9, r=0.05, sill=False)
    window(p, "Window", 2.3, 3.3, 0.9, 2.4, y=1.9, r=0.05, sill=False)
    p.box("Rail", (0.4, 0.02, 0.95), (3.6, 0.08, 1.0), p.grown(), small=True)
    for x in (0.6, 1.2, 1.8, 2.4, 3.0, 3.4):
        p.box(f"Baluster{x}", (x - 0.015, 0.03, 0.05), (x + 0.015, 0.07, 0.95), p.railing(), small=True)
    p.socket("Curtain_0", (1.3, 2.05, 0.0), "curtains inside the loggia door", accepts="curtain")
    p.socket("Seat", (2.8, 0.9, 0.05), "a chair or two in the loggia", accepts="balcony_furniture")
    p.socket("Planter_0", (0.8, 0.6, 0.05), "a pot in the loggia", accepts="small")
    p.socket("Interior", (0, 1.6, 0), "a room behind the loggia", accepts="home_insert")
    p.colliders.clear()
    p.collider("box", (0.2, 0.2, 2), (0.4, 0.4, 4))
    p.collider("box", (3.8, 0.2, 2), (0.4, 0.4, 4))
    p.collider("box", (2.0, 1.9, 1.65), (3.2, 0.2, 3.3))
    return p


@asset
def res_privacy():
    p = bay("city_res_privacy", "CityResPrivacy", "residential",
            "A private, mostly solid bay: a bathroom's high narrow window, a small square window "
            "and a stepped-mesa relief, behind a grown-timber lattice screen. Homes are not "
            "showrooms: this is the bay that keeps a wall a wall.", phases=ALL)
    holes = [rounded_rect(0.6, 1.2, 1.9, 3.0, 0.05, 2), rounded_rect(2.6, 3.4, 1.2, 2.2, 0.08, 2)]
    res_wall(p, holes)
    window(p, "High", 0.6, 1.2, 1.9, 3.0, r=0.05)
    window(p, "Square", 2.6, 3.4, 1.2, 2.2, r=0.08)
    p.mesa_steps("Mesa", 1.6, 0.25, 1.2, 3, 0.18, 0.0, p.relief(), depth=0.04)
    for k in range(7):   # a lattice over the square window
        x = 2.45 + 0.18 * k
        p.box(f"Lath{k}", (x, -0.18, 1.0), (x + 0.05, -0.13, 2.4), p.grown(), small=True)
    for z in (1.05, 2.3):
        p.box(f"LathRail{z}", (2.4, -0.13, z), (3.6, -0.08, z + 0.05), p.grown(), small=True)
    p.socket("Curtain_1", (3.0, 0.36, 0.0), "a blind inside the small window", accepts="curtain")
    p.socket("Interior", (0, 0, 0), "a room", accepts="home_insert")
    return p


@asset
def res_corner_round():
    p = piece("city_res_corner_round", "CityResCornerRound", "residential",
              "A rounded residential corner, one storey: a curved plaster wall (r 4 m) with two "
              "windows and a curved sliding door to a curved balcony (city_balcony_round). Wraps "
              "a 4 x 4 m corner cell; runs continue at x 4 (front) and y 4 (side).", phases=("town", "city"))
    p.curved_wall("Wall", ROUND, 4.0, 3.6, A0, A1, 0.0, 3.7, p.plaster(),
                  openings=[(0.1, 0.28, 0.9, 2.5), (0.38, 0.62, 0.0, 2.6), (0.72, 0.9, 0.9, 2.5)],
                  glass=p.glass(), frame=p.timber(), seg=16)
    p.curved_wall("Band", ROUND, 4.08, 3.6, A0, A1, 3.7, 4.0, p.relief(), seg=16)
    p.socket("Balcony", (0, 0, 0), "a curved balcony at this origin", accepts="balcony_round")
    p.socket("Interior", (2.6, 2.6, 0.0), "a corner room", accepts="home_insert")
    p.connector("front", (4, 0, 0), (1, 0, 0))
    p.connector("side", (0, 4, 0), (0, 1, 0))
    p.collider("cylinder", (4, 4, 2), (4.0, 4.0))
    return p


@asset
def res_door():
    p = bay("city_res_door", "CityResDoor", "residential",
            "An apartment's front door off a shared access gallery: a solid timber door with a "
            "house-number plaque, a kitchen window beside it, a doormat and a pot. Variant A.",
            phases=ALL)
    holes = [rounded_rect(0.5, 1.5, 0.0, 2.2, 0.05, 2), rounded_rect(2.2, 3.4, 1.1, 2.3, 0.1, 3)]
    res_wall(p, holes)
    p.box("Door", (0.55, 0.28, 0.02), (1.45, 0.34, 2.15), p.timber())
    p.box("DoorFrame", (0.5, 0.26, 0.0), (1.55, 0.36, 2.2), p.grown(), small=True)
    window(p, "Kitchen", 2.2, 3.4, 1.1, 2.3, r=0.1)
    p.box("Plaque", (1.65, -0.03, 1.5), (1.95, 0.0, 1.75), p.titanium(), small=True)
    p.box("Mat", (0.6, -0.7, 0.0), (1.4, -0.1, 0.02), p.fabric("Rust"), small=True)
    p.socket("Door", (1.0, -0.1, 0.0), "an apartment's front door", accepts="door")
    p.socket("Number", (1.8, -0.04, 1.62), "the apartment's number (runtime)", accepts="sign")
    p.socket("Planter_0", (1.85, -0.35, 0.0), "a pot by the door", accepts="small")
    p.socket("Interior", (0, 0, 0), "a kitchen", accepts="home_insert")
    return p


# ====================================================================== RES02 balconies


def balcony_rail(p, pts, z=0.0, h=1.05):
    p.tube("Handrail", [(x, y, z + h) for x, y in pts], 0.03, p.timber(), sides=5)
    p.tube("MidRail", [(x, y, z + 0.12) for x, y in pts], 0.015, p.railing(), sides=4, small=True)
    for i, (x, y) in enumerate(pts):
        p.tube(f"Post{i}", [(x, y, z), (x, y, z + h)], 0.018, p.railing(), sides=4, small=True)
    # dense balusters between posts
    for i, ((xa, ya), (xb, yb)) in enumerate(zip(pts, pts[1:])):
        n = max(2, int(math.hypot(xb - xa, yb - ya) / 0.13))
        for j in range(1, n):
            t = j / n
            x, y = xa + (xb - xa) * t, ya + (yb - ya) * t
            p.box(f"Bal{i}_{j}", (x - 0.008, y - 0.008, z + 0.12), (x + 0.008, y + 0.008, z + h), p.railing(), small=True)


@asset
def balcony_res_4():
    p = piece("city_balcony_res_4", "CityBalconyRes4", "balcony",
              "A home's balcony, 4 m wide and 1.6 m deep: a rounded slab on the floor line, a "
              "plaster fascia and a dark rail with a timber handrail. Origin at the facade's front "
              "base corner; projects to -Y. Furnish it at its sockets.", phases=ALL)
    outline = [(0.0, 0.0), (4.0, 0.0)] + [(x, y) for x, y in rounded_rect(0.0, 4.0, -1.6, 0.4, 0.5, 4) if y < 0.0]
    slab = rounded_rect(0.0, 4.0, -1.6, 0.0, 0.0, 1)
    p.box("Slab", (0.0, -1.6, -0.25), (4.0, 0.0, 0.0), p.paving())
    p.box("Fascia", (0.0, -1.65, -0.35), (4.0, -1.6, 0.05), p.plaster(), small=True)
    balcony_rail(p, [(0.05, -0.05), (0.05, -1.55), (3.95, -1.55), (3.95, -0.05)])
    p.socket("Furniture", (1.2, -0.8, 0.0), "a chair, a small table or a sofa", accepts="balcony_furniture")
    p.socket("Laundry", (3.0, -0.8, 0.0), "a drying rack", accepts="laundry")
    p.socket("Planter_0", (0.4, -1.4, 0.0), "a pot", accepts="small")
    p.socket("RailPlanter", (2.0, -1.55, 1.05), "a planter hooked on the rail", accepts="rail_planter")
    p.socket("Shade", (0, 0, 0), "a shade fabric (city_shade_fabric) at this origin", accepts="shade")
    p.socket("Screen", (3.95, -0.8, 0.0), "a privacy screen between neighbours", accepts="screen")
    p.connector("facade", (0, 0, 0), (1, 0, 0))
    p.collider("box", (2, -0.8, -0.12), (4, 1.6, 0.25), walkable=True)
    p.collider("box", (2, -1.55, 0.55), (4, 0.1, 1.1))
    return p


@asset
def balcony_round():
    p = piece("city_balcony_round", "CityBalconyRound", "balcony",
              "A curved corner balcony: a quarter-ring slab from r 4 to 5.6 m round a rounded "
              "corner (centre 4, 4), with the same rail. Origin as the rounded corner's.", phases=("town", "city"))
    p.sector("Slab", ROUND, 5.6, 4.0, A0, A1, -0.25, 0.0, p.paving(), seg=14)
    p.sector("Fascia", ROUND, 5.65, 5.6, A0, A1, -0.35, 0.05, p.plaster(), seg=14, small=True)
    pts = [(4 + 5.55 * math.cos(A0 + (A1 - A0) * i / 10), 4 + 5.55 * math.sin(A0 + (A1 - A0) * i / 10)) for i in range(11)]
    balcony_rail(p, pts)
    a = (A0 + A1) / 2
    p.socket("Furniture", (4 + 4.8 * math.cos(a), 4 + 4.8 * math.sin(a), 0.0), "balcony furniture", accepts="balcony_furniture")
    p.socket("Planter_0", (4 + 5.2 * math.cos(A0 + 0.2), 4 + 5.2 * math.sin(A0 + 0.2), 0.0), "a pot", accepts="small")
    p.collider("box", (1.2, 1.2, -0.12), (3.0, 3.0, 0.25), walkable=True)
    return p


@asset
def balcony_end():
    p = piece("city_balcony_end", "CityBalconyEnd", "balcony",
              "A balcony end panel: a solid plaster cheek 1.6 m deep and 1.1 m high closing a "
              "balcony's side against wind and neighbours. Origin at the facade, at the panel's line.",
              phases=ALL)
    p.box("Panel", (-0.06, -1.6, 0.0), (0.06, 0.0, 1.1), p.plaster(), round=0.0)
    p.box("Cap", (-0.08, -1.62, 1.1), (0.08, 0.0, 1.15), p.stone(), small=True)
    return p


@asset
def balcony_planter():
    p = piece("city_balcony_planter", "CityBalconyPlanter", "balcony",
              "A 1.2 m planter box that hooks over a balcony rail, outside it. Origin on the rail's "
              "top centre.", phases=ALL)
    p.box("Box", (-0.6, -0.32, -0.32), (0.6, -0.04, -0.02), p.clay())
    p.box("Soil", (-0.55, -0.28, -0.04), (0.55, -0.08, -0.01), p.soil(), small=True)
    p.box("Hook", (-0.5, -0.06, -0.02), (0.5, 0.04, 0.02), p.railing(), small=True)
    p.canopy("Greens", (0.0, -0.18, 0.08), 0.32, p.leaf(), lobes=4, flat=0.6, small=True)
    def blooms(bm):
        for k in range(7):
            c = Vector((-0.5 + k / 6, -0.18 + 0.08 * math.sin(k), 0.2 + 0.05 * (k % 2)))
            g = bmesh.ops.create_icosphere(bm, subdivisions=1, radius=0.05)
            for v in g["verts"]:
                v.co += c
    p.mesh("Blooms", blooms, p.bloom(), small=True)
    return p


@asset
def privacy_screen():
    p = piece("city_privacy_screen", "CityPrivacyScreen", "balcony",
              "A 1.6 x 2.1 m grown-timber lattice screen standing on a balcony's side between "
              "neighbours. Origin at the centre of its foot; the screen lies in the YZ plane.",
              phases=ALL)
    for k in range(9):
        y = -0.8 + 0.2 * k
        p.box(f"Slat{k}", (-0.03, y - 0.03, 0.0), (0.03, y + 0.03, 2.1), p.grown(), small=k % 4 != 0)
    for z in (0.2, 1.0, 1.9):
        p.box(f"Rail{z}", (-0.05, -0.82, z), (0.05, 0.82, z + 0.06), p.grown(), small=True)
    return p


@asset
def shade_fabric():
    p = piece("city_shade_fabric", "CityShadeFabric", "balcony",
              "A balcony shade: a loose triangle of coral cloth tied from the floor band to two "
              "titanium arms. Origin as the balcony's.", phases=ALL)
    cloth = p.fabric("Coral")
    def sheet(bm):
        pts = [Vector((0.3, 0.0, 3.5)), Vector((3.7, 0.0, 3.5)), Vector((2.6, -1.5, 2.7)), Vector((0.9, -1.5, 2.9))]
        rows = []
        for i in range(5):
            t = i / 4
            a, b = pts[0].lerp(pts[3], t), pts[1].lerp(pts[2], t)
            rows.append([bm.verts.new(a.lerp(b, s / 4) - Vector((0, 0, 0.12 * math.sin(math.pi * s / 4) * t))) for s in range(5)])
        for r0, r1 in zip(rows, rows[1:]):
            for s in range(4):
                bm.faces.new((r0[s], r0[s + 1], r1[s + 1], r1[s]))
    p.mesh("Cloth", sheet, cloth, smooth=True)
    for x in (0.9, 2.6):
        p.tube(f"Arm{x}", [(x, -1.5, 1.05), (x, -1.5, 2.9)], 0.02, p.titanium(), sides=4, small=True)
    return p


@asset
def laundry():
    p = piece("city_laundry", "CityLaundry", "balcony",
              "A drying rack with the week's washing: shirts, a sheet and towels in linen and the "
              "textile tints. The plainest sign that people live here. Origin at its centre.",
              phases=ALL)
    for x in (-0.7, 0.7):
        p.tube(f"Leg{x}", [(x, -0.25, 0.0), (x, 0.0, 1.2), (x, 0.25, 0.0)], 0.015, p.railing(), sides=4, small=True)
    for k, y in enumerate((-0.12, 0.12)):
        p.tube(f"Line{k}", [(-0.7, y, 1.15), (0.7, y, 1.15)], 0.008, p.railing(), sides=3, small=True)
    cols = ("Linen", "Coral", "Teal", "Violet", "Linen", "Ochre")
    for k in range(6):
        x0 = -0.65 + 0.22 * k
        y = -0.12 if k % 2 else 0.12
        h = 0.35 + 0.25 * ((k * 7) % 3) / 2
        p.box(f"Cloth{k}", (x0, y - 0.01, 1.15 - h), (x0 + 0.2, y + 0.01, 1.15), p.fabric(cols[k]), small=True)
    return p


# ====================================================================== RES03 dwelling shell


@asset
def room_floor_4():
    p = piece("city_room_floor_4", "CityRoomFloor4", "dwelling",
              "A 4 x 4 m timber floor finish laid on a slab inside a home (top 3 cm above the slab).",
              phases=ALL)
    p.box("Boards", (0.0, 0.0, 0.0), (BAY, BAY, 0.03), p.timber())
    p.collider("box", (2, 2, 0.015), (BAY, BAY, 0.03), walkable=True)
    return p


@asset
def partition_4():
    p = piece("city_partition_4", "CityPartition4", "dwelling",
              "A 4 m interior wall between rooms or homes, 0.15 m thick and 3.7 m to the soffit, "
              "limewashed. Its line is y 0..0.15.", phases=ALL)
    p.box("Wall", (0.0, 0.0, 0.0), (BAY, 0.15, 3.7), p.interior())
    p.box("Skirting", (0.0, -0.02, 0.0), (BAY, 0.17, 0.1), p.timber(), small=True)
    p.connector("start", (0, 0, 0), (1, 0, 0))
    p.connector("end", (BAY, 0, 0), (1, 0, 0))
    p.collider("box", (2, 0.075, 1.85), (BAY, 0.15, 3.7))
    return p


@asset
def partition_door():
    p = piece("city_partition_door", "CityPartitionDoor", "dwelling",
              "A 4 m interior wall with a 0.9 m door (2.1 m) and its leaf standing open: the "
              "doorway between rooms, or a home's door off a corridor.", phases=ALL)
    p.pierced("Wall", [(0, 0), (BAY, 0), (BAY, 3.7), (0, 3.7)], [[(1.2, 0.0), (2.1, 0.0), (2.1, 2.1), (1.2, 2.1)]],
              0.0, 0.15, p.interior())
    p.box("Leaf", (2.1, 0.15, 0.0), (2.15, 1.0, 2.08), p.timber())
    p.box("Skirting", (0.0, -0.02, 0.0), (1.2, 0.17, 0.1), p.timber(), small=True)
    p.box("Skirting2", (2.1, -0.02, 0.0), (BAY, 0.17, 0.1), p.timber(), small=True)
    p.socket("Door", (1.65, 0.075, 0.0), "the doorway", accepts="door")
    p.collider("box", (0.6, 0.075, 1.85), (1.2, 0.15, 3.7))
    p.collider("box", (3.05, 0.075, 1.85), (1.9, 0.15, 3.7))
    return p


@asset
def slab_round():
    p = piece("city_slab_round", "CitySlabRound", "dwelling",
              "The floor slab of a rounded corner cell: a quarter disc (r 4 m, centre 4, 4), top "
              "at its origin's level. Use with the rounded corner facades.", phases=("town", "city"))
    p.sector("Slab", ROUND, 4.0, 0.0, A0, A1, -SLAB, 0.0, p.paving(), seg=14)
    p.collider("cylinder", (4, 4, -0.15), (4.0, 0.3), walkable=True)
    return p


@asset
def roof_round():
    p = piece("city_roof_round", "CityRoofRound", "dwelling",
              "The roof of a rounded corner cell: a planted quarter disc with a curved parapet and "
              "coping, matching city_parapet_4.", phases=("town", "city"))
    p.sector("Slab", ROUND, 4.0, 0.0, A0, A1, -SLAB, 0.0, p.plaster(), seg=14)
    p.sector("Parapet", ROUND, 4.0, 3.6, A0, A1, 0.0, 1.1, p.plaster(), seg=14)
    p.sector("Coping", ROUND, 4.06, 3.54, A0, A1, 1.1, 1.18, p.stone(), seg=14, small=True)
    p.sector("Meadow", ROUND, 3.5, 0.0, A0, A1, 0.0, 0.3, p.moss(), seg=12)
    p.socket("Plant_0", (2.4, 2.4, 0.3), "corner roof planting", accepts="tree")
    p.collider("cylinder", (4, 4, 0.4), (4.0, 1.2))
    return p


# ====================================================================== RES04 domestic interior


def furniture(aid, kind, about):
    return piece(aid, kind, "interior", about, phases=ALL)


@asset
def sofa():
    p = furniture("city_sofa", "CitySofa", "A three-seat sofa, 2.1 m, ochre upholstery on a timber "
                  "plinth, with two cushions. Origin at its centre; sitters face -Y.")
    up = p.fabric("Ochre")
    p.box("Plinth", (-1.05, -0.45, 0.0), (1.05, 0.45, 0.12), p.timber())
    p.box("Seat", (-1.0, -0.42, 0.12), (1.0, 0.3, 0.45), up, round=0.06)
    p.box("Back", (-1.05, 0.25, 0.12), (1.05, 0.45, 0.85), up, round=0.06)
    for x in (-1.05, 0.85):
        p.box(f"Arm{x}", (x, -0.42, 0.12), (x + 0.2, 0.45, 0.62), up, round=0.05)
    for k, (x, c) in enumerate(((-0.55, "Teal"), (0.5, "Coral"))):
        p.box(f"Cushion{k}", (x - 0.2, 0.12, 0.45), (x + 0.2, 0.25, 0.8), p.fabric(c), round=0.04, small=True)
    p.socket("Seat_0", (-0.5, -0.1, 0.45), "a seat", accepts="seat")
    p.socket("Seat_1", (0.5, -0.1, 0.45), "a seat", accepts="seat")
    p.collider("box", (0, 0, 0.42), (2.1, 0.9, 0.85))
    return p


@asset
def armchair():
    p = furniture("city_armchair", "CityArmchair", "A deep armchair in teal upholstery on timber "
                  "legs. Origin at its centre; faces -Y.")
    up = p.fabric("Teal")
    p.box("Seat", (-0.4, -0.38, 0.18), (0.4, 0.3, 0.45), up, round=0.06)
    p.box("Back", (-0.42, 0.22, 0.18), (0.42, 0.4, 0.9), up, round=0.06)
    for x in (-0.42, 0.28):
        p.box(f"Arm{x}", (x, -0.38, 0.18), (x + 0.14, 0.4, 0.62), up, round=0.05)
    for x in (-0.35, 0.35):
        for y in (-0.33, 0.33):
            p.box(f"Leg{x}{y}", (x - 0.025, y - 0.025, 0.0), (x + 0.025, y + 0.025, 0.18), p.timber(), small=True)
    p.socket("Seat", (0, -0.05, 0.45), "a seat", accepts="seat")
    return p


@asset
def coffee_table():
    p = furniture("city_coffee_table", "CityCoffeeTable", "A low round coffee table with a book "
                  "and a cup. Origin at its centre.")
    p.lathe("Table", [(0.0, 0.42), (0.45, 0.42), (0.45, 0.38), (0.06, 0.36), (0.06, 0.0), (0.0, 0.0)], p.timber(), steps=14)
    p.box("Book", (-0.2, -0.1, 0.42), (0.05, 0.08, 0.46), p.fabric("Slate"), small=True)
    p.lathe("Cup", [(0.0, 0.42), (0.04, 0.42), (0.045, 0.5), (0.0, 0.5)], p.canvas(), steps=8, centre=(0.18, 0.1, 0), small=True)
    return p


@asset
def dining_set():
    p = furniture("city_dining_set", "CityDiningSet", "A dining table for four (1.6 x 0.9 m) with "
                  "four chairs, a jug and a bowl of fruit. Origin at the table's centre.")
    p.box("Top", (-0.8, -0.45, 0.72), (0.8, 0.45, 0.76), p.timber())
    for x in (-0.72, 0.72):
        for y in (-0.38, 0.38):
            p.box(f"Leg{x}{y}", (x - 0.03, y - 0.03, 0.0), (x + 0.03, y + 0.03, 0.72), p.timber(), small=True)
    for k, (x, y, s) in enumerate(((-0.4, -0.7, -1), (0.4, -0.7, -1), (-0.4, 0.7, 1), (0.4, 0.7, 1))):
        p.box(f"Seat{k}", (x - 0.2, y - 0.2, 0.44), (x + 0.2, y + 0.2, 0.48), p.timber(), small=True)
        p.box(f"Back{k}", (x - 0.2, y + s * 0.17, 0.48), (x + 0.2, y + s * 0.21, 0.9), p.timber(), small=True)
        for dx in (-0.17, 0.17):
            for dy in (-0.17, 0.17):
                p.box(f"CLeg{k}{dx}{dy}", (x + dx - 0.015, y + dy - 0.015, 0.0), (x + dx + 0.015, y + dy + 0.015, 0.44),
                      p.timber(), small=True)
    p.lathe("Jug", [(0.0, 0.76), (0.07, 0.76), (0.08, 0.9), (0.05, 1.0), (0.0, 1.0)], p.clay(), steps=8, centre=(-0.3, 0.05, 0), small=True)
    p.lathe("Bowl", [(0.0, 0.76), (0.12, 0.8), (0.14, 0.84), (0.0, 0.8)], p.stone(), steps=10, centre=(0.3, 0.0, 0), small=True)
    for k in range(4):
        p.sphere(f"Fruit{k}", (0.3 + 0.05 * math.cos(k * 1.6), 0.05 * math.sin(k * 1.6), 0.86), 0.04,
                 p.mat("Ochre", None, roughness=0.6) if k % 2 else p.clay(), segments=6, rings=4, small=True)
    for k in range(4):
        p.socket(f"Seat_{k}", (((-0.4, 0.4, -0.4, 0.4)[k]), ((-0.7, -0.7, 0.7, 0.7)[k]), 0.48), "a seat", accepts="seat")
    p.collider("box", (0, 0, 0.38), (1.6, 0.9, 0.76))
    return p


@asset
def bed():
    p = furniture("city_bed", "CityBed", "A double bed (1.6 x 2.0 m) on a timber frame with a "
                  "headboard, linen bedding, a coral throw and two pillows. Origin at its centre; "
                  "headboard at +Y.")
    p.box("Frame", (-0.85, -1.05, 0.0), (0.85, 1.05, 0.3), p.timber())
    p.box("Mattress", (-0.8, -1.0, 0.3), (0.8, 1.0, 0.52), p.fabric("Linen"), round=0.06)
    p.box("Throw", (-0.82, -1.02, 0.4), (0.82, -0.3, 0.55), p.fabric("Coral"), small=True)
    for x in (-0.4, 0.4):
        p.box(f"Pillow{x}", (x - 0.32, 0.6, 0.52), (x + 0.32, 0.92, 0.65), p.fabric("Linen"), round=0.05, small=True)
    p.box("Headboard", (-0.88, 1.0, 0.0), (0.88, 1.1, 1.1), p.timber())
    p.box("Nightstand", (0.95, 0.65, 0.0), (1.4, 1.05, 0.5), p.timber(), small=True)
    p.collider("box", (0, 0, 0.27), (1.7, 2.1, 0.55))
    return p


@asset
def kitchen():
    p = furniture("city_kitchen", "CityKitchen", "A 2.4 m kitchen run against a wall: stone "
                  "worktop with a sink and hob, timber doors, open shelves with crockery and a "
                  "hanging pan. Origin at its front centre; the wall is at +Y.")
    p.box("Base", (-1.2, 0.0, 0.0), (1.2, 0.6, 0.86), p.timber())
    p.box("Top", (-1.22, -0.03, 0.86), (1.22, 0.62, 0.9), p.stone())
    p.box("Sink", (-0.7, 0.12, 0.88), (-0.25, 0.48, 0.91), p.titanium(), small=True)
    p.box("Hob", (0.35, 0.12, 0.9), (0.85, 0.5, 0.91), p.graphene(), small=True)
    for z in (1.5, 1.9):
        p.box(f"Shelf{z}", (-1.2, 0.35, z), (1.2, 0.6, z + 0.04), p.timber(), small=True)
    for k in range(6):
        x = -1.0 + 0.4 * k
        p.lathe(f"Pot{k}", [(0.0, 0.0), (0.06, 0.0), (0.07, 0.14), (0.0, 0.14)], p.clay() if k % 2 else p.canvas(),
                steps=8, centre=(x, 0.47, 1.54), small=True)
    for k in range(4):
        p.box(f"Door{k}", (-1.15 + 0.58 * k, -0.01, 0.08), (-0.6 + 0.58 * k, 0.0, 0.8), p.timber(), small=True)
    p.collider("box", (0, 0.3, 0.45), (2.4, 0.6, 0.9))
    return p


@asset
def bookshelf():
    p = furniture("city_bookshelf", "CityBookshelf", "A 1.0 x 2.0 m bookshelf, five shelves of "
                  "books in the textile tints, a plant on top. Origin at its front centre; wall +Y.")
    p.box("Back", (-0.5, 0.3, 0.0), (0.5, 0.35, 2.0), p.timber())
    for x in (-0.5, 0.47):
        p.box(f"Side{x}", (x, 0.0, 0.0), (x + 0.03, 0.35, 2.0), p.timber())
    cols = ("Slate", "Coral", "Teal", "Ochre", "Linen", "Violet", "Rust")
    for s in range(5):
        z = 0.05 + 0.4 * s
        p.box(f"Shelf{s}", (-0.48, 0.0, z), (0.48, 0.32, z + 0.03), p.timber(), small=True)
        x = -0.45
        k = 0
        while x < 0.38:
            w = 0.035 + 0.03 * ((k * 5 + s) % 3) / 2
            h = 0.22 + 0.08 * ((k * 3 + s) % 4) / 3
            p.box(f"Book{s}_{k}", (x, 0.05, z + 0.03), (x + w, 0.28, z + 0.03 + h), p.fabric(cols[(k + s) % 7]), small=True)
            x += w + 0.005
            k += 1
    p.lathe("Pot", [(0.0, 2.0), (0.08, 2.0), (0.1, 2.16), (0.0, 2.16)], p.clay(), steps=8, small=True)
    p.blades("Plant", (0.0, 0.15, 2.16), 7, 0.3, 0.05, p.leaf(), spread=0.03, lean=0.7)
    p.collider("box", (0, 0.17, 1.0), (1.0, 0.35, 2.0))
    return p


def curtain(colour):
    def build():
        p = furniture(f"city_curtain_{colour.lower()}", f"CityCurtain{colour}",
                      f"A pair of {colour.lower()} curtains for a window or door: two gathered panels "
                      "drawn to the sides of a 1.8 m opening, on a titanium rod at 2.7 m, falling to "
                      "the floor. Origin at the opening's centre on the floor, inside the glass.")
        cloth = p.fabric(colour)
        def panel(bm, x0, x1):
            n = 6
            rows = []
            for i in range(n + 1):
                x = x0 + (x1 - x0) * i / n
                dy = 0.05 * (1 if i % 2 else -1)
                rows.append((bm.verts.new((x, 0.1 + dy, 0.02)), bm.verts.new((x, 0.1 + dy * 0.6, 2.7))))
            for (a0, a1), (b0, b1) in zip(rows, rows[1:]):
                bm.faces.new((a0, b0, b1, a1))
        p.mesh("PanelL", lambda bm: panel(bm, -0.95, -0.55), cloth, smooth=True)
        p.mesh("PanelR", lambda bm: panel(bm, 0.55, 0.95), cloth, smooth=True)
        p.tube("Rod", [(-1.0, 0.1, 2.72), (1.0, 0.1, 2.72)], 0.012, p.titanium(), sides=4, small=True)
        return p
    build.__name__ = f"curtain_{colour.lower()}"
    return asset(build)


for _c in ("Linen", "Coral", "Teal"):
    curtain(_c)


@asset
def floor_lamp():
    p = furniture("city_floor_lamp", "CityFloorLamp", "A floor lamp, 1.6 m: a timber stem and a "
                  "linen shade over a warm domestic core and halo. Origin at its base's centre.")
    p.lathe("Base", [(0.0, 0.0), (0.16, 0.0), (0.14, 0.04), (0.0, 0.04)], p.railing(), steps=10, small=True)
    p.tube("Stem", [(0, 0, 0.04), (0, 0, 1.4)], 0.015, p.timber(), sides=5, small=True)
    p.lathe("Shade", [(0.14, 1.32), (0.22, 1.32), (0.14, 1.6), (0.1, 1.6)], p.fabric("Linen"), steps=12, small=True)
    core, halo = p.hearth()
    p.glow_point("Bulb", (0, 0, 1.45), 0.05, 0.2, core, halo)
    return p


@asset
def pendant_lamp():
    p = furniture("city_pendant_lamp", "CityPendantLamp", "A woven pendant lamp on a cord, warm "
                  "core and halo. Origin at the ceiling attachment (3.7 m on a storey).")
    p.tube("Cord", [(0, 0, 0.0), (0, 0, -0.9)], 0.005, p.railing(), sides=3, small=True)
    p.lathe("Shade", [(0.02, -0.9), (0.18, -1.05), (0.22, -1.2), (0.16, -1.25)], p.fabric("Ochre"), steps=12, small=True)
    core, halo = p.hearth()
    p.glow_point("Bulb", (0, 0, -1.12), 0.05, 0.18, core, halo)
    return p


@asset
def rug():
    p = furniture("city_rug", "CityRug", "A 2.4 x 1.6 m woven rug: a rust field with a stepped "
                  "border in linen. Origin at its centre.")
    p.box("Field", (-1.2, -0.8, 0.0), (1.2, 0.8, 0.012), p.fabric("Rust"))
    for k in range(4):
        x0 = -1.0 + 0.5 * k
        p.box(f"Step{k}", (x0, -0.65, 0.012), (x0 + 0.4, -0.55, 0.016), p.fabric("Linen"), small=True)
        p.box(f"StepB{k}", (x0, 0.55, 0.012), (x0 + 0.4, 0.65, 0.016), p.fabric("Linen"), small=True)
    return p


@asset
def balcony_chairs():
    p = furniture("city_balcony_chairs", "CityBalconyChairs", "Two folding balcony chairs and a "
                  "small round table with a pot of herbs. Origin at the table's centre.")
    p.lathe("Table", [(0.0, 0.6), (0.25, 0.6), (0.25, 0.57), (0.02, 0.55), (0.02, 0.0), (0.0, 0.0)], p.railing(), steps=10)
    p.lathe("Herbs", [(0.0, 0.6), (0.06, 0.6), (0.07, 0.72), (0.0, 0.72)], p.clay(), steps=8, small=True)
    p.canopy("Leaves", (0, 0, 0.8), 0.08, p.sprout(), lobes=3, small=True)
    for k, x in enumerate((-0.55, 0.55)):
        p.box(f"Seat{k}", (x - 0.2, -0.2, 0.42), (x + 0.2, 0.2, 0.46), p.fabric("Teal" if k else "Coral"), small=True)
        p.box(f"Back{k}", (x - 0.2, 0.18, 0.46), (x + 0.2, 0.22, 0.85), p.timber(), small=True)
        p.tube(f"LegA{k}", [(x - 0.18, -0.2, 0.0), (x + 0.18, 0.2, 0.42)], 0.012, p.railing(), sides=4, small=True)
        p.tube(f"LegB{k}", [(x + 0.18, -0.2, 0.0), (x - 0.18, 0.2, 0.42)], 0.012, p.railing(), sides=4, small=True)
    return p


# ====================================================================== RES08 interior proxies and distant facades


def proxy(aid, kind, about, build):
    def fn():
        p = piece(aid, kind, "proxy", about, phases=ALL)
        p.box("Back", (0.05, 2.1, 0.03), (3.95, 2.2, 3.7), p.interior())
        p.box("Side", (0.0, 0.4, 0.03), (0.06, 2.2, 3.7), p.interior())
        p.box("Floor", (0.0, 0.4, 0.0), (BAY, 2.2, 0.03), p.timber())
        build(p)
        p.notes.append("a shallow room set (1.8 m deep) behind one facade bay: what a distant "
                       "or inaccessible window shows. Its back wall hides the rest of the floor.")
        return p
    fn.__name__ = aid.removeprefix("city_")
    return asset(fn)


def _living(p):
    up = p.fabric("Ochre")
    p.box("Sofa", (0.6, 1.55, 0.03), (2.6, 2.1, 0.45), up)
    p.box("SofaBack", (0.6, 1.9, 0.45), (2.6, 2.1, 0.85), up)
    p.box("Picture", (1.1, 2.08, 1.5), (2.1, 2.1, 2.2), p.fabric("Teal"), small=True)
    p.box("Shelf", (3.0, 1.85, 0.03), (3.8, 2.1, 1.9), p.timber())
    for k in range(4):
        p.box(f"Books{k}", (3.05, 1.9, 0.4 + 0.42 * k), (3.75, 2.05, 0.65 + 0.42 * k), p.fabric(("Coral", "Slate", "Linen", "Violet")[k]), small=True)
    core, halo = p.hearth()
    p.tube("LampStem", [(2.85, 1.8, 0.03), (2.85, 1.8, 1.4)], 0.015, p.timber(), sides=4, small=True)
    p.glow_point("Lamp", (2.85, 1.8, 1.5), 0.06, 0.5, core, halo)
    p.box("Rug", (0.6, 0.6, 0.03), (2.8, 1.5, 0.04), p.fabric("Rust"), small=True)


def _bedroom(p):
    p.box("Bed", (0.4, 0.9, 0.03), (2.4, 2.1, 0.5), p.fabric("Linen"))
    p.box("Throw", (0.4, 0.9, 0.35), (2.4, 1.3, 0.53), p.fabric("Coral"), small=True)
    p.box("Headboard", (0.35, 2.05, 0.03), (2.45, 2.1, 1.1), p.timber(), small=True)
    p.box("Wardrobe", (2.9, 1.6, 0.03), (3.9, 2.1, 2.2), p.timber())
    core, halo = p.hearth()
    p.glow_point("Lamp", (2.6, 1.9, 0.75), 0.05, 0.45, core, halo)


def _kitchen(p):
    p.box("Counter", (0.3, 1.5, 0.03), (2.8, 2.1, 0.9), p.timber())
    p.box("Top", (0.28, 1.48, 0.9), (2.82, 2.1, 0.94), p.stone(), small=True)
    p.box("Shelf", (0.3, 1.85, 1.6), (2.8, 2.1, 1.64), p.timber(), small=True)
    for k in range(5):
        p.box(f"Jar{k}", (0.4 + 0.5 * k, 1.9, 1.64), (0.6 + 0.5 * k, 2.05, 1.84), p.clay() if k % 2 else p.canvas(), small=True)
    p.box("Table", (2.9, 0.7, 0.72), (3.8, 1.5, 0.76), p.timber())
    p.box("TableLeg", (3.3, 1.05, 0.03), (3.4, 1.15, 0.72), p.timber(), small=True)
    core, halo = p.hearth()
    p.glow_point("Pendant", (3.35, 1.1, 1.9), 0.07, 0.5, core, halo, shape=(1.3, 1.3, 0.8))


def _office(p):
    for k, x in enumerate((0.9, 2.9)):
        p.box(f"Desk{k}", (x - 0.75, 1.2, 0.72), (x + 0.75, 1.9, 0.76), p.timber())
        p.box(f"Screen{k}", (x - 0.3, 1.65, 0.8), (x + 0.3, 1.68, 1.15), p.graphene(), small=True)
        p.box(f"Chair{k}", (x - 0.22, 0.7, 0.03), (x + 0.22, 1.05, 0.95), p.fabric("Slate"), small=True)
    p.box("Board", (1.0, 2.08, 1.3), (3.0, 2.1, 2.3), p.canvas(), small=True)
    p.lathe("Pot", [(0.0, 0.03), (0.15, 0.03), (0.18, 0.4), (0.0, 0.4)], p.clay(), steps=8, centre=(3.7, 1.8, 0), small=True)
    p.canopy("Plant", (3.7, 1.8, 0.9), 0.35, p.leaf(), lobes=4, small=True)


def _shop(p):
    goods = (p.clay(), p.canvas(), p.fabric("Verdigris"), p.fabric("Ochre"))
    for s in range(4):
        z = 0.4 + 0.6 * s
        p.box(f"Shelf{s}", (0.3, 1.75, z), (3.7, 2.1, z + 0.04), p.timber(), small=True)
        for k in range(5):
            p.box(f"Goods{s}_{k}", (0.4 + 0.65 * k, 1.8, z + 0.04), (0.9 + 0.65 * k, 2.05, z + 0.3), goods[(s + k) % 4], small=True)
    p.box("Counter", (2.6, 0.8, 0.03), (3.7, 1.4, 1.0), p.timber())


proxy("city_proxy_home_living", "CityProxyHomeLiving", "A living-room set behind a home's window: a sofa, a shelf of books, a picture, a rug and a warm lamp.", _living)
proxy("city_proxy_home_bedroom", "CityProxyHomeBedroom", "A bedroom set behind a home's window: a bed with a throw, a wardrobe and a bedside lamp.", _bedroom)
proxy("city_proxy_home_kitchen", "CityProxyHomeKitchen", "A kitchen set behind a home's window: a counter with jars, an open shelf, a small table under a pendant.", _kitchen)
proxy("city_proxy_office", "CityProxyOffice", "An office set behind a ribbon of glass: two desks with screens and chairs, a pin board and a plant.", _office)
proxy("city_proxy_shop", "CityProxyShop", "A cheap shop set: stocked shelves and a counter (the light version of city_shop_display).", _shop)


@asset
def far_res():
    p = piece("city_far_res", "CityFarRes", "proxy",
              "A distant residential storey bay: a flat plaster face with two dark window panes, "
              "one lit warm, one with a curtain colour, and a balcony line. Tens of triangles; for "
              "buildings seen from far away.", phases=ALL)
    p.box("Wall", (0.0, 0.0, 0.0), (BAY, 0.3, 4.0), p.plaster())
    p.box("PaneA", (0.6, -0.01, 0.9), (1.6, 0.0, 2.5), p.mat("Window", "glass", roughness=0.2, color="#3c3a3f"))
    p.box("PaneB", (2.4, -0.01, 0.9), (3.4, 0.0, 2.5), p.fabric("Coral"))
    p.box("Balcony", (0.0, -1.0, -0.2), (BAY, 0.0, 0.0), p.paving())
    p.box("Rail", (0.0, -1.0, 0.0), (BAY, -0.95, 1.0), p.railing())
    return p


@asset
def far_office():
    p = piece("city_far_office", "CityFarOffice", "proxy",
              "A distant office storey bay: a spandrel and one long dark glass band. For far "
              "buildings.", phases=ALL)
    p.box("Wall", (0.0, 0.0, 0.0), (BAY, 0.3, 4.0), p.plaster())
    p.box("Band", (0.0, -0.01, 0.9), (BAY, 0.0, 3.4), p.mat("Window", "glass", roughness=0.2, color="#3c3a3f"))
    return p


@asset
def far_tower_storey():
    p = piece("city_far_tower_storey", "CityFarTowerStorey", "proxy",
              "One storey of a distant 12 x 12 m residential tower, all four sides in one mesh: "
              "rounded corners, window bands and balcony lips. Stack it for the skyline. Origin at "
              "its base's centre.", phases=("town", "city"))
    p.box("Core", (-6.0, -6.0, 0.0), (6.0, 6.0, 4.0), p.plaster(), round=2.0, seg=3)
    p.box("Windows", (-6.02, -6.02, 1.0), (6.02, 6.02, 2.6), p.mat("Window", "glass", roughness=0.2, color="#3c3a3f"), round=2.0, seg=3)
    p.box("Lip", (-6.8, -6.8, 0.0), (6.8, 6.8, 0.2), p.paving(), round=2.4, seg=3)
    p.box("Planting", (-6.7, -6.7, 0.2), (6.7, 6.7, 0.45), p.leaf(), round=2.4, seg=3)
    return p


# ====================================================================== RES05 access


@asset
def res_entry():
    p = bay("city_res_entry", "CityResEntry", "access",
            "The street door to the homes above, a ground-floor bay that is plainly not a shop: "
            "a recessed pair of timber doors under a stone hood, a house-name plaque, a bank of "
            "titanium letterboxes, a bench and planters, and small windows to the lobby. Origin "
            "and line as a facade bay.")
    holes = [arch_opening(1.1, 2.9, 0.0, 2.2, 10), rounded_rect(0.35, 0.85, 1.4, 2.4, 0.06, 2)]
    res_wall(p, holes)
    p.box("RevealL", (1.1, 0.4, 0.0), (1.2, 1.4, 3.1), p.plaster())
    p.box("RevealR", (2.8, 0.4, 0.0), (2.9, 1.4, 3.1), p.plaster())
    p.box("Lobby", (1.1, 0.4, 3.1), (2.9, 1.4, 3.2), p.plaster(), small=True)
    for k, x in enumerate((1.2, 2.0)):
        p.box(f"Door{k}", (x, 1.3, 0.02), (x + 0.8, 1.36, 2.4), p.timber())
        p.box(f"Glass{k}", (x + 0.2, 1.29, 1.3), (x + 0.6, 1.3, 2.1), p.glass(), small=True)
    p.box("Hood", (0.9, -0.6, 3.0), (3.1, 0.4, 3.15), p.stone())
    p.box("Plaque", (1.5, -0.03, 3.25), (2.5, 0.0, 3.55), p.graphene(), small=True)
    window(p, "Lobby", 0.35, 0.85, 1.4, 2.4, r=0.06)
    p.box("Letterboxes", (3.05, -0.04, 1.0), (3.8, 0.0, 1.8), p.titanium(), small=True)
    for k in range(3):
        p.box(f"Slot{k}", (3.1, -0.05, 1.15 + 0.22 * k), (3.75, -0.04, 1.18 + 0.22 * k), p.graphene(), small=True)
    p.box("Bench", (0.2, -0.6, 0.0), (1.0, -0.15, 0.45), p.stone(), small=True)
    p.box("Step", (1.0, -0.4, 0.0), (3.0, 0.4, 0.06), p.stone(), small=True)
    core, halo = p.lantern()
    p.glow_point("Lamp", (2.0, -0.35, 2.85), 0.05, 0.14, core, halo)
    p.socket("Door", (2.0, 0.9, 0.0), "the residents' street door, separate from the shops", accepts="door")
    p.socket("Name", (2.0, -0.04, 3.4), "the building's name (runtime)", accepts="sign", rotation=(math.pi / 2, 0, 0))
    p.socket("Planter_0", (3.4, -0.4, 0.0), "a planter by the door", accepts="shrub")
    p.colliders.clear()
    p.collider("box", (0.55, 0.2, 2), (1.1, 0.4, 4))
    p.collider("box", (3.45, 0.2, 2), (1.1, 0.4, 4))
    return p


@asset
def core_spiral():
    p = piece("city_core_spiral", "CityCoreSpiral", "access",
              "One storey of the residents' stair tower: a 4 x 4 m glass and titanium drum with a "
              "spiral stair (16 risers of 0.25 m) round a stone newel, arriving at the same "
              "quarter every storey, and a door to the landing. Stack it on z 4 k; pair with "
              "city_lift. Origin at the cell's left front corner.")
    cx, cy, R = 2.0, 2.0, 1.9
    p.lathe("Newel", [(0.18, 0.0), (0.18, 4.0)], p.stone(), steps=10, centre=(cx, cy, 0))
    for k in range(16):
        a0 = math.tau * k / 16
        a1 = a0 + math.tau / 16
        z = 0.25 * (k + 1)
        def tread(bm, a0=a0, a1=a1, z=z):
            pts = [(cx + 0.18 * math.cos(a0), cy + 0.18 * math.sin(a0)), (cx + 1.75 * math.cos(a0), cy + 1.75 * math.sin(a0)),
                   (cx + 1.75 * math.cos(a1), cy + 1.75 * math.sin(a1)), (cx + 0.18 * math.cos(a1), cy + 0.18 * math.sin(a1))]
            Piece._extrude(bm, [(x, y, z - 0.06) for x, y in pts], (0, 0, 0.06))
        p.mesh(f"Tread{k}", tread, p.stone(), small=k % 4 != 0)
    rail = [(cx + 1.72 * math.cos(math.tau * i / 32), cy + 1.72 * math.sin(math.tau * i / 32), 0.125 * i + 0.95)
            for i in range(33)]
    p.tube("Handrail", rail, 0.025, p.timber(), sides=5)
    p.curved_wall("Drum", (cx, cy), R, R - 0.06, 0.0, math.tau * 0.82, 0.0, 4.0, p.glass(), seg=24)
    for i in range(9):
        a = math.tau * 0.82 * i / 8
        p.tube(f"Mullion{i}", [(cx + R * math.cos(a), cy + R * math.sin(a), 0.0), (cx + R * math.cos(a), cy + R * math.sin(a), 4.0)],
               0.035, p.titanium(), sides=5, small=True)
    p.sector("Ring", (cx, cy), R + 0.1, R - 0.1, 0.0, math.tau * 0.999, 3.75, 4.0, p.titanium(), seg=24)
    p.socket("Landing", (cx + 1.0, cy - 1.4, 0.0), "where the stair arrives, by the door to the corridor", accepts="door")
    p.connector("above", (0, 0, BAY), (0, 0, 1))
    p.notes.append("the drum leaves an opening facing -Y +X (the last 18% of the circle) to the landing")
    p.collider("cylinder", (cx, cy, 2.0), (R, 4.0))
    return p


@asset
def roof_access():
    p = piece("city_roof_access", "CityRoofAccess", "access",
              "The stair head on a roof garden: a small rounded plaster kiosk (4 x 4 m, 3 m) with a "
              "door to the roof, a planted top and a lantern. Sits over the stair tower.")
    p.box("Body", (0.2, 0.2, 0.0), (3.8, 3.8, 2.9), p.plaster(), round=0.6)
    p.box("Roof", (0.0, 0.0, 2.9), (4.0, 4.0, 3.1), p.stone(), round=0.7)
    p.box("Green", (0.2, 0.2, 3.1), (3.8, 3.8, 3.25), p.moss(), round=0.5)
    p.box("Door", (1.4, 0.15, 0.0), (2.4, 0.22, 2.2), p.timber())
    core, halo = p.lantern()
    p.glow_point("Lamp", (2.7, 0.05, 2.3), 0.05, 0.13, core, halo)
    p.socket("Door", (1.9, 0.0, 0.0), "the way down to the homes", accepts="door")
    p.collider("box", (2, 2, 1.5), (3.6, 3.6, 3.0))
    return p


@asset
def mailboxes():
    p = piece("city_mailboxes", "CityMailboxes", "access",
              "A lobby's wall of letterboxes (1.6 x 1.2 m, titanium) on a stone base, with a shelf "
              "for parcels. Origin at its foot's centre; faces -Y.", phases=ALL)
    p.box("Base", (-0.8, -0.2, 0.0), (0.8, 0.1, 0.8), p.stone())
    p.box("Boxes", (-0.8, -0.15, 0.8), (0.8, 0.1, 2.0), p.titanium())
    for r in range(4):
        for c in range(5):
            p.box(f"Door{r}{c}", (-0.76 + 0.31 * c, -0.16, 0.85 + 0.29 * r), (-0.5 + 0.31 * c, -0.15, 1.1 + 0.29 * r),
                  p.graphene(), small=True)
    return p


# ====================================================================== RES06 communal


@asset
def greenhouse():
    p = piece("city_greenhouse", "CityGreenhouse", "communal",
              "A shared rooftop greenhouse, 4 x 4 m: a titanium-ribbed glass gable over two raised "
              "beds of seedlings and a potting bench. Origin at its left front corner; door -Y.")
    for x in (0.1, 1.33, 2.66, 3.9):
        pts = [(x, 0.1, 0.0), (x, 0.1, 2.1), (x, 2.0, 3.1), (x, 3.9, 2.1), (x, 3.9, 0.0)]
        p.tube(f"Rib{x}", pts, 0.035, p.titanium(), sides=4)
    def skin(bm):
        prof = [(0.1, 0.0), (0.1, 2.1), (2.0, 3.1), (3.9, 2.1), (3.9, 0.0)]
        a = [bm.verts.new((0.1, y, z)) for y, z in prof]
        b = [bm.verts.new((3.9, y, z)) for y, z in prof]
        for i in range(4):
            bm.faces.new((a[i], a[i + 1], b[i + 1], b[i]))
        bm.faces.new(a)
        bm.faces.new(b[::-1])
    p.mesh("Glass", skin, p.glass())
    for k, y in enumerate((0.5, 2.6)):
        p.box(f"Bed{k}", (0.4, y, 0.0), (3.6, y + 0.9, 0.6), p.timber())
        p.box(f"Soil{k}", (0.45, y + 0.05, 0.6), (3.55, y + 0.85, 0.62), p.soil(), small=True)
        p.blades(f"Seedlings{k}", (2.0, y + 0.45, 0.62), 18, 0.25, 0.05, p.sprout(), spread=1.4, lean=0.4)
    p.socket("Gardener", (2.0, 1.9, 0.0), "where a Dancer tends the beds", accepts="performer")
    p.collider("box", (2, 2, 1.5), (3.8, 3.8, 3.0))
    return p


@asset
def raised_beds():
    p = piece("city_raised_beds", "CityRaisedBeds", "communal",
              "Two raised vegetable beds (3 x 0.9 m) in timber with crops at different stages and "
              "a water butt: the shared kitchen garden on a roof or in a courtyard. Origin at the "
              "centre of its 4 x 4 m cell.", phases=ALL)
    for k, y in enumerate((-0.9, 0.6)):
        p.box(f"Bed{k}", (-1.5, y, 0.0), (1.5, y + 0.9, 0.55), p.timber())
        p.box(f"Soil{k}", (-1.45, y + 0.05, 0.55), (1.45, y + 0.85, 0.57), p.soil(), small=True)
        for j in range(5):
            c = (-1.2 + 0.6 * j, y + 0.45, 0.57)
            if (j + k) % 2:
                p.canopy(f"Crop{k}{j}", (c[0], c[1], 0.75), 0.22, p.leaf_young(), lobes=3, flat=0.7, small=True)
            else:
                p.blades(f"Crop{k}{j}", c, 7, 0.35, 0.04, p.sprout(), spread=0.05, lean=0.3)
    p.lathe("Butt", [(0.0, 0.0), (0.3, 0.0), (0.32, 0.9), (0.0, 0.9)], p.titanium(), steps=12, centre=(1.75, 1.7, 0))
    p.collider("box", (0, -0.45, 0.27), (3.0, 0.9, 0.55))
    p.collider("box", (0, 1.05, 0.27), (3.0, 0.9, 0.55))
    return p


@asset
def communal_table():
    p = piece("city_communal_table", "CityCommunalTable", "communal",
              "A long shared table (3 x 0.9 m) with two benches, for roof-garden suppers and "
              "courtyard meals. Origin at its centre.", phases=ALL)
    p.box("Top", (-1.5, -0.45, 0.72), (1.5, 0.45, 0.77), p.timber())
    for x in (-1.2, 1.2):
        p.box(f"Trestle{x}", (x - 0.06, -0.35, 0.0), (x + 0.06, 0.35, 0.72), p.grown(), small=True)
    for y in (-0.85, 0.85):
        p.box(f"Bench{y}", (-1.4, y - 0.18, 0.42), (1.4, y + 0.18, 0.47), p.timber())
        for x in (-1.2, 1.2):
            p.box(f"BenchLeg{y}{x}", (x - 0.05, y - 0.15, 0.0), (x + 0.05, y + 0.15, 0.42), p.grown(), small=True)
    for k in range(6):
        p.socket(f"Seat_{k}", (-1.0 + 1.0 * (k % 3), -0.85 if k < 3 else 0.85, 0.47), "a place at the table", accepts="seat")
    p.collider("box", (0, 0, 0.38), (3.0, 0.9, 0.77))
    return p
