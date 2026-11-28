"""City kit, P1: terraces, light rail, gallery, concert hall, monument, luminous flora and
the water-music staff. Same contract as P0 (city_kit.py)."""
import math

import bmesh
from mathutils import Vector

from city_kit import BAY, SLAB, WATER_Z, Piece, arch_opening, circle, rounded_rect

P1 = {}


def asset(fn):
    P1["city_" + fn.__name__] = fn
    return fn


def piece(aid, kind, catalog, about, phases=("town", "city")):
    return Piece(aid, kind, catalog=catalog, priority="P1", phases=phases, about=about)


# ------------------------------------------------------------------ STR08 terraces


@asset
def balcony():
    p = piece("city_balcony", "CityBalcony", "terrace",
              "A 4 m balcony cantilevered 1.5 m off a facade at floor level, carried on two curved "
              "grown-timber brackets, with a clay planter trough as its front guard and titanium "
              "side rails. Origin at the facade's front base corner; it projects towards -Y.")
    p.box("Slab", (0, -1.5, -0.25), (BAY, 0.0, 0.0), p.paving())
    p.box("Edge", (0, -1.55, -0.3), (BAY, -1.5, 0.05), p.relief(), small=True)
    for x in (0.4, 3.6):
        pts = [(x, 0.0, -1.3), (x, -0.35, -0.7), (x, -0.9, -0.35), (x, -1.4, -0.27)]
        p.tube(f"Bracket{x}", pts, [0.13, 0.11, 0.09, 0.07], p.grown(), sides=7)
    p.box("Trough", (0.05, -1.5, 0.0), (BAY - 0.05, -1.1, 0.9), p.clay())
    p.box("TroughSoil", (0.12, -1.44, 0.9), (BAY - 0.12, -1.16, 0.92), p.soil(), small=True)
    for x in (0.03, BAY - 0.03):
        p.tube(f"Rail{x}", [(x, -1.1, 1.0), (x, -0.05, 1.0)], 0.03, p.titanium(), sides=6, small=True)
        p.tube(f"RailPost{x}", [(x, -0.6, 0.0), (x, -0.6, 1.0)], 0.025, p.titanium(), sides=5, small=True)
    p.socket("Plant_0", (1.0, -1.3, 0.92), "balcony trough", accepts="small")
    p.socket("Plant_1", (3.0, -1.3, 0.92), "balcony trough", accepts="small")
    p.socket("Vine_front", (2.0, -1.5, 0.9), "vines trailing from the trough", accepts="vine")
    p.connector("facade", (0, 0, 0), (1, 0, 0))
    p.collider("box", (2, -0.75, -0.12), (BAY, 1.5, 0.25), walkable=True)
    p.collider("box", (2, -1.3, 0.45), (BAY, 0.4, 0.9))
    return p


@asset
def garden_deck():
    p = piece("city_garden_deck", "CityGardenDeck", "terrace",
              "A 4 x 4 m garden terrace for a setback roof: timber decking on the roof slab and a "
              "0.7 m planted edge along its front (-Y), where the storey above steps back.")
    p.box("Slab", (0, 0, -SLAB), (BAY, BAY, 0), p.plaster())
    for k in range(10):
        y = 0.75 + 0.325 * k
        p.box(f"Board{k}", (0.05, y, 0.0), (BAY - 0.05, y + 0.3, 0.08), p.timber(), small=k % 3 != 0)
    p.box("Edge", (0, 0, 0), (BAY, 0.7, 0.7), p.stone())
    p.box("EdgeSoil", (0.08, 0.08, 0.7), (BAY - 0.08, 0.62, 0.72), p.soil(), small=True)
    for k in range(3):
        p.socket(f"Plant_{k}", (0.7 + 1.3 * k, 0.35, 0.72), "terrace edge planting", accepts="shrub")
    p.socket("Vine_front", (2.0, 0.0, 0.7), "vines over the terrace edge", accepts="vine")
    p.socket("Seat", (2.0, 2.5, 0.08), "a place for Dancers to sit", accepts="seat")
    p.collider("box", (2, 2, -0.11), (BAY, BAY, 0.38), walkable=True)
    p.collider("box", (2, 0.35, 0.35), (BAY, 0.7, 0.7))
    return p


@asset
def pergola():
    p = piece("city_pergola", "CityPergola", "terrace",
              "A 4 x 4 m grown-timber pergola, 3 m: four living posts, two arched beams and a lath "
              "roof for vines; for terraces, roofs and plazas. Origin at its left front corner.",
              phases=("village", "town", "city"))
    for x in (0.3, 3.7):
        for y in (0.3, 3.7):
            p.tube(f"Post{x}{y}", [(x, y, 0.0), (x + 0.03, y, 1.5), (x, y, 2.8)], [0.12, 0.1, 0.1], p.grown(), sides=7)
    for y in (0.3, 3.7):
        pts = [(0.1, y, 2.7), (1.0, y, 3.05), (2.0, y, 3.15), (3.0, y, 3.05), (3.9, y, 2.7)]
        p.tube(f"Beam{y}", pts, 0.09, p.grown(), sides=6)
    for k in range(7):
        x = 0.35 + 0.55 * k
        z = 2.75 + 0.4 * math.sin(math.pi * (x - 0.1) / 3.8)
        p.box(f"Lath{k}", (x - 0.03, 0.1, z), (x + 0.03, 3.9, z + 0.06), p.grown(), small=True)
    p.socket("Vine_top", (2.0, 2.0, 3.1), "vines over the lath", accepts="vine")
    for k, (x, y) in enumerate(((0.3, 0.3), (3.7, 3.7))):
        p.socket(f"Plant_{k}", (x + 0.3, y, 0.0), "a climber at the post's foot", accepts="small")
    for x in (0.3, 3.7):
        for y in (0.3, 3.7):
            p.collider("cylinder", (x, y, 1.4), (0.12, 2.8))
    return p


@asset
def skybridge_8():
    p = piece("city_skybridge_8", "CitySkybridge8", "terrace",
              "An 8 m skybridge between two buildings at floor level, 3 m wide: a titanium truss "
              "under a timber deck, grown-timber ribs arching over it holding a glass canopy, "
              "and planted lips. Origin at its start's left edge, deck top at z 0; runs along +X.")
    p.box("Deck", (0, 0, -0.2), (8, 3, 0), p.timber())
    for y in (0.0, 3.0):
        p.box(f"Chord{y}", (0, y - 0.08, -1.1), (8, y + 0.08, -0.95), p.titanium())
        p.box(f"TopChord{y}", (0, y - 0.08, -0.35), (8, y + 0.08, -0.2), p.titanium())
        for k in range(8):
            a, b = (k, -1.0), (k + 1, -0.28)
            if k % 2:
                a, b = (k, -0.28), (k + 1, -1.0)
            p.tube(f"Web{y}_{k}", [(a[0], y, a[1]), (b[0], y, b[1])], 0.05, p.titanium(), sides=5, small=True)
    for k in range(5):
        x = 0.2 + 1.9 * k
        pts = [(x, 0.05, 0.0), (x, -0.05, 1.6), (x, 0.5, 2.8), (x, 1.5, 3.15), (x, 2.5, 2.8), (x, 3.05, 1.6), (x, 2.95, 0.0)]
        p.tube(f"Rib{k}", pts, 0.08, p.grown(), sides=6)
    def canopy(bm):
        rows = []
        for i in range(9):
            t = i / 8
            a = math.pi * (0.15 + 0.7 * t)
            y = 1.5 - 1.5 * math.cos(a) * 1.02
            z = 1.6 + 1.6 * math.sin(a)
            rows.append((bm.verts.new((0.2, y, z)), bm.verts.new((7.8, y, z))))
        for (a0, a1), (b0, b1) in zip(rows, rows[1:]):
            bm.faces.new((a0, a1, b1, b0))
    p.mesh("Canopy", canopy, p.glass())
    for y0, y1 in ((-0.35, 0.0), (3.0, 3.35)):
        p.box(f"Lip{y0}", (0, y0, -0.35), (8, y1, 0.4), p.clay(), small=True)
        p.box(f"LipSoil{y0}", (0.05, y0 + 0.04, 0.4), (7.95, y1 - 0.04, 0.42), p.soil(), small=True)
        side = "L" if y0 < 0 else "R"
        p.socket(f"Vine_{side}", (4.0, y0 if y0 < 0 else y1, 0.4), "vines over the skybridge's side", accepts="vine",
                 rotation=(0, 0, 0 if y0 < 0 else math.pi))
    p.connector("start", (0, 0, 0), (-1, 0, 0))
    p.connector("end", (8, 0, 0), (1, 0, 0))
    p.collider("box", (4, 1.5, -0.1), (8, 3, 0.2), walkable=True)
    return p


# ------------------------------------------------------------------ TRN01 light rail

GAUGE = 1.435


def guideway(p, path, name="Beam"):
    """An elevated guideway along `path` [(x, y, heading)]: a pale beam with a planted track
    bed, two titanium rails and a low parapet each side. Track top at z 0."""
    def loft(profile, mat, nm, small=False):
        def fill(bm):
            rings = []
            for x, y, h in path:
                d = Vector((math.cos(h), math.sin(h), 0))
                s = Vector((-d.y, d.x, 0))
                rings.append([bm.verts.new(Vector((x, y, 0)) + s * u + Vector((0, 0, v))) for u, v in profile])
            for ra, rb in zip(rings, rings[1:]):
                n = len(profile)
                for k in range(n):
                    bm.faces.new((ra[k], ra[(k + 1) % n], rb[(k + 1) % n], rb[k]))
            bm.faces.new(rings[0][::-1])
            bm.faces.new(rings[-1])
        return p.mesh(nm, fill, mat, small=small)
    beam = [(-1.6, -0.2), (-1.6, 0.35), (-1.45, 0.35), (-1.45, -0.05), (1.45, -0.05), (1.45, 0.35), (1.6, 0.35),
            (1.6, -0.2), (1.2, -0.9), (-1.2, -0.9)]
    loft(beam, p.stone(), name)
    loft([(-1.4, -0.06), (1.4, -0.06), (1.4, -0.03), (-1.4, -0.03)], p.moss(), name + "Bed", small=True)
    for side in (-1, 1):
        u = side * GAUGE / 2
        loft([(u - 0.04, -0.03), (u + 0.04, -0.03), (u + 0.04, 0.0), (u - 0.04, 0.0)], p.titanium(),
             f"{name}Rail{side}", small=True)


@asset
def rail_track_8():
    p = piece("city_rail_track_8", "CityRailTrack8", "rail",
              "An 8 m straight of elevated light-rail guideway: a pale beam with a moss-planted track "
              "bed and two titanium rails. Origin at the start of its centreline, track top at z 0 "
              "(place at z 7.65 on city_rail_pier). Runs along +X.")
    guideway(p, [(x, 0.0, 0.0) for x in (0.0, 8.0)])
    p.connector("start", (0, 0, 0), (-1, 0, 0))
    p.connector("end", (8, 0, 0), (1, 0, 0))
    p.collider("box", (4, 0, -0.4), (8, 3.2, 1.0))
    return p


@asset
def rail_curve():
    p = piece("city_rail_curve", "CityRailCurve", "rail",
              "A curve of guideway: 22.5 degrees to the left on a 20 m radius (7.85 m along the "
              "centreline). Origin at its start's centreline; starts heading +X.")
    R, sweep = 20.0, math.radians(22.5)
    path = []
    for k in range(9):
        a = sweep * k / 8
        path.append((R * math.sin(a), R - R * math.cos(a), a))
    guideway(p, path)
    end = path[-1]
    p.connector("start", (0, 0, 0), (-1, 0, 0))
    p.connector("end", (end[0], end[1], 0), (math.cos(sweep), math.sin(sweep), 0))
    p.notes.append("four curves make a 90-degree turn; mirror by placing it rotated 180 degrees and run backwards")
    return p


@asset
def rail_pier():
    p = piece("city_rail_pier", "CityRailPier", "rail",
              "A guideway pier, 6.75 m: a stone column swelling to a forked, rounded capital that "
              "carries the beam's soffit, so the track top stands at 7.65 m and a platform deck at "
              "8 m (two storeys, reached by the stair kit). Origin at its base's centre.")
    prof = [(0.75, 0.0), (0.75, 0.3), (0.6, 0.45), (0.5, 4.0), (0.62, 5.2), (0.0, 5.2)]
    p.lathe("Column", prof, p.stone(), steps=18)
    cap = rounded_rect(-0.9, 0.9, -1.4, 1.4, 0.5, 4)
    p.plan("Capital", cap, 5.2, 6.6, p.stone())
    p.plan("Bearing", rounded_rect(-0.7, 0.7, -1.2, 1.2, 0.3, 3), 6.6, 6.75, p.stone_dark(), small=True)
    p.strata("Strata", -0.92, 0.92, 5.8, (0.06, 0.04), -1.4, p.relief())
    p.socket("Plant_0", (0.9, 0.0, 0.0), "a climber at the pier's foot", accepts="vine_climber")
    p.connector("guideway", (0, 0, 7.65), (1, 0, 0))
    p.collider("cylinder", (0, 0, 3.4), (0.75, 6.75))
    return p


@asset
def rail_platform_8():
    p = piece("city_rail_platform_8", "CityRailPlatform8", "rail",
              "An 8 m side platform beside the guideway: deck 0.35 m above the rails, 3 m wide, "
              "a pale edge strip, planters at the back and a seat. Same origin as the track it "
              "serves (track centreline start, track top z 0); it lies on the +Y side.")
    p.box("Deck", (0, 1.75, 0.0), (8, 4.75, 0.35), p.paving())
    p.box("Edge", (0, 1.7, 0.32), (8, 2.1, 0.36), p.stone(), small=True)
    p.box("Fascia", (0, 1.7, -0.9), (8, 4.75, 0.0), p.stone())
    for k in range(2):
        x0 = 0.6 + 4.0 * k
        p.box(f"Planter{k}", (x0, 4.15, 0.35), (x0 + 2.8, 4.7, 0.85), p.stone())
        p.box(f"PlanterSoil{k}", (x0 + 0.08, 4.23, 0.85), (x0 + 2.72, 4.62, 0.87), p.soil(), small=True)
        p.socket(f"Plant_{k}", (x0 + 1.4, 4.42, 0.87), "platform planter", accepts="shrub")
    p.box("Bench", (3.5, 3.4, 0.35), (4.5, 3.9, 0.8), p.timber())
    p.socket("Wait", (2.0, 3.0, 0.35), "where Dancers wait for the train", accepts="seat")
    p.socket("Audio_station", (4.0, 3.2, 2.0), "station ambience and arrivals", accepts="audio")
    p.connector("start", (0, 0, 0), (-1, 0, 0))
    p.connector("end", (8, 0, 0), (1, 0, 0))
    p.collider("box", (4, 3.25, 0.17), (8, 3.0, 0.35), walkable=True)
    return p


@asset
def rail_canopy_8():
    p = piece("city_rail_canopy_8", "CityRailCanopy8", "rail",
              "An 8 m platform canopy: grown-timber ribs rising from the platform's back edge and "
              "arching over it, carrying graphene solar leaves. Same origin as city_rail_platform_8.")
    for k, x in enumerate((0.6, 4.6)):
        pts = [(x, 4.45, 0.35), (x, 4.55, 2.4), (x, 4.0, 3.6), (x, 2.9, 4.0), (x, 1.9, 3.85)]
        p.tube(f"Rib{k}", pts, [0.16, 0.13, 0.11, 0.09, 0.07], p.grown(), sides=7)
    for k in range(4):
        x0 = 0.1 + 2.0 * k
        def leaf(bm, x0=x0):
            pts = [(x0, 4.3, 3.65), (x0 + 1.9, 4.3, 3.65), (x0 + 1.9, 1.8, 3.95), (x0, 1.8, 3.95)]
            v = [bm.verts.new(c) for c in pts]
            bm.faces.new(v)
            bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.05)
        p.mesh(f"Solar{k}", leaf, p.graphene())
        p.box(f"Frame{k}", (x0, 3.0, 3.7), (x0 + 1.9, 3.06, 3.82), p.titanium(), small=True)
    p.collider("box", (0.6, 4.45, 1.9), (0.3, 0.3, 3.1))
    p.collider("box", (4.6, 4.45, 1.9), (0.3, 0.3, 3.1))
    return p


@asset
def rail_car():
    p = piece("city_rail_car", "CityRailCar", "rail",
              "A light-rail car placeholder, 12 m: a rounded bone-white body with a graphene window "
              "band, titanium skirts, rooftop solar and pale head lamps. Origin at its centre on the "
              "rail top; runs along +X. Motion and doors are runtime work.")
    def body(bm):
        L, W, H, r = 12.0, 2.65, 3.3, 0.9
        prof = rounded_rect(-W / 2, W / 2, 0.45, H, 0.55, 4)   # cross-section (Y, Z)
        n = 10
        rings = []
        for i in range(n + 1):
            t = i / n
            x = -L / 2 + L * t
            taper = 1.0
            edge = min(t, 1 - t) * L
            if edge < r:
                taper = 0.55 + 0.45 * math.sqrt(max(0.0, 1 - ((r - edge) / r) ** 2))
            rings.append([bm.verts.new((x, y * taper, 0.45 + (z - 0.45) * (0.8 + 0.2 * taper))) for y, z in prof])
        m = len(prof)
        for ra, rb in zip(rings, rings[1:]):
            for k in range(m):
                bm.faces.new((ra[k], ra[(k + 1) % m], rb[(k + 1) % m], rb[k]))
        bm.faces.new(rings[0][::-1])
        bm.faces.new(rings[-1])
    p.mesh("Body", body, p.canvas(), smooth=True)
    for sy in (-1, 1):
        p.box(f"Windows{sy}", (-5.0, sy * 1.33 - 0.02, 1.5), (5.0, sy * 1.33 + 0.02, 2.6), p.graphene())
        p.box(f"Skirt{sy}", (-5.6, sy * 1.25 - 0.05, 0.3), (5.6, sy * 1.25 + 0.05, 0.6), p.titanium(), small=True)
    for x in (-4.0, 4.0):
        p.box(f"Bogie{x}", (x - 1.1, -0.9, 0.05), (x + 1.1, 0.9, 0.5), p.graphene())
    p.box("Solar", (-4.5, -0.9, 3.28), (4.5, 0.9, 3.34), p.graphene(), small=True)
    core, halo = p.lantern()
    for x in (-5.95, 5.95):
        for y in (-0.7, 0.7):
            p.glow_point(f"Lamp{x}{y}", (x, y, 1.0), 0.06, 0.15, core, halo)
    for sy, side in ((-1, "R"), (1, "L")):
        p.socket(f"Door_{side}", (0.0, sy * 1.35, 0.45), "a door for boarding (runtime opens it)", accepts="door")
    p.socket("Audio_car", (0, 0, 1.5), "the car's hum and chime", accepts="audio")
    p.collider("box", (0, 0, 1.85), (12, 2.65, 2.9))
    return p


# ------------------------------------------------------------------ CIV01 gallery


@asset
def gallery_entrance():
    p = piece("city_gallery_entrance", "CityGalleryEntrance", "civic",
              "The art gallery's entrance front, 8 m wide and 12 m tall: stepped plaster towers "
              "flank a 4 m round-headed glass portal whose fanlight radiates grown-timber ribs like "
              "sun rays; a stone stair-plinth and an empty name panel. Origin at its left front "
              "corner; front -Y. Displays player patches inside (see city_gallery_display_wall).")
    # Stepped flanking towers.
    for x0, x1 in ((0.0, 2.0), (6.0, 8.0)):
        p.box(f"Tower{x0}", (x0, 0.0, 0.0), (x1, 2.0, 9.0), p.plaster())
        p.box(f"TowerStep{x0}", (x0 + 0.3, 0.3, 9.0), (x1 - 0.3, 2.0, 10.6), p.plaster())
        p.box(f"TowerCap{x0}", (x0 + 0.6, 0.6, 10.6), (x1 - 0.6, 2.0, 11.6), p.plaster())
        p.strata(f"Strata{x0}", x0, x1, 8.2, (0.12, 0.06, 0.2), 0.0, p.relief())
    # The central block with the portal.
    portal = arch_opening(2.4, 5.6, 0.6, 5.2, 16)
    p.pierced("Front", [(2.0, 0.0), (6.0, 0.0), (6.0, 8.0), (2.0, 8.0)], [portal], 0.3, 1.0, p.plaster())
    p.box("Crown", (2.0, 0.3, 8.0), (6.0, 1.6, 8.6), p.relief())
    p.elevation("Glazing", arch_opening(2.42, 5.58, 0.6, 5.2, 16), 0.78, 0.8, p.glass())
    for k in range(9):   # sun-ray ribs across the fanlight
        a = math.pi * (k + 0.5) / 9
        p.tube(f"Ray{k}", [(4.0, 0.74, 5.2), (4.0 + 1.55 * math.cos(a), 0.74, 5.2 + 1.55 * math.sin(a))],
               0.04, p.grown(), sides=5, small=True)
    p.box("Transom", (2.42, 0.7, 5.15), (5.58, 0.78, 5.25), p.grown(), small=True)
    p.box("MullionL", (3.45, 0.7, 0.6), (3.53, 0.78, 5.2), p.titanium(), small=True)
    p.box("MullionR", (4.47, 0.7, 0.6), (4.55, 0.78, 5.2), p.titanium(), small=True)
    # A stone plinth with three steps up to the threshold.
    for k in range(3):
        p.box(f"Step{k}", (1.6 + 0.01 * k, -1.2 + 0.4 * k, 0.0), (6.4 - 0.01 * k, 1.0 - 0.01 * k, 0.2 * (k + 1)), p.stone())
    p.box("NamePanel", (2.8, 0.25, 6.9), (5.2, 0.3, 7.6), p.graphene(), small=True)
    p.socket("Door", (4.0, 0.6, 0.6), "the gallery's way in", accepts="door")
    p.socket("Name_panel", (4.0, 0.24, 7.25), "the gallery's name, written at runtime (empty here)", accepts="sign")
    p.socket("Plant_L", (1.0, -0.3, 0.0), "planting beside the stair", accepts="tree")
    p.socket("Plant_R", (7.0, -0.3, 0.0), "planting beside the stair", accepts="tree")
    p.collider("box", (1, 1, 4.5), (2, 2, 9))
    p.collider("box", (7, 1, 4.5), (2, 2, 9))
    p.collider("box", (4, -0.1, 0.3), (5.2, 2.2, 0.6), slope=0.0, walkable=True)
    return p


@asset
def gallery_display_wall():
    p = piece("city_gallery_display_wall", "CityGalleryDisplayWall", "civic",
              "A 4 m gallery display wall, 4 m tall: a plaster bay with a recessed titanium frame "
              "around an empty graphene display (3 x 2 m) where the game shows a player's saved "
              "patch, and a soft lantern over it. Front -Y; same line as a facade bay.")
    p.pierced("Wall", [(0, 0), (BAY, 0), (BAY, 3.7), (0, 3.7)], [rounded_rect(0.45, 3.55, 0.75, 2.95, 0.12, 2)],
              0.0, 0.4, p.plaster())
    p.box("Back", (0.45, 0.3, 0.75), (3.55, 0.4, 2.95), p.plaster())
    p.pierced("Frame", rounded_rect(0.45, 3.55, 0.75, 2.95, 0.12, 2), [rounded_rect(0.55, 3.45, 0.85, 2.85, 0.08, 2)],
              0.22, 0.3, p.titanium(), small=True)
    p.elevation("Display", rounded_rect(0.55, 3.45, 0.85, 2.85, 0.08, 2), 0.27, 0.3, p.graphene())
    core, halo = p.lantern()
    p.box("LampArm", (1.9, -0.3, 3.25), (2.1, 0.0, 3.32), p.titanium(), small=True)
    p.glow_point("Lamp", (2.0, -0.35, 3.2), 0.07, 0.2, core, halo, shape=(1.6, 1, 0.8))
    p.box("Band", (0, -0.08, 3.7), (BAY, 0.4, 4.0), p.relief())
    p.socket("Display_0", (2.0, 0.26, 1.85), "a player's saved patch is shown here (anchor only; the "
             "display faces -Y, 2.9 x 2.0 m)", accepts="display", rotation=(math.pi / 2, 0, 0))
    p.connector("start", (0, 0, 0), (1, 0, 0))
    p.connector("end", (BAY, 0, 0), (1, 0, 0))
    p.collider("box", (2, 0.2, 2), (BAY, 0.4, 4))
    return p


@asset
def gallery_plinth():
    p = piece("city_gallery_plinth", "CityGalleryPlinth", "civic",
              "An exhibit plinth, 1.2 x 1.2 x 0.9 m: pale stone with a titanium top plate and an "
              "anchor for a sculpture, an instrument or a recorded object. Origin at its base's centre.")
    p.box("Body", (-0.6, -0.6, 0.0), (0.6, 0.6, 0.85), p.monument(), round=0.08)
    p.box("Plate", (-0.55, -0.55, 0.85), (0.55, 0.55, 0.9), p.titanium(), round=0.06)
    p.strata("Strata", -0.6, 0.6, 0.15, (0.04, 0.03), -0.6, p.relief(), depth=0.02)
    p.socket("Exhibit", (0, 0, 0.9), "what the plinth shows (anchor only)", accepts="exhibit")
    p.collider("box", (0, 0, 0.45), (1.2, 1.2, 0.9))
    return p


# ------------------------------------------------------------------ CIV02 concert hall


def shell_surface(p, name, width, height, depth, mat, y_shift=0.0, thickness=0.25, inner_mat=None):
    """A half-dome shell (open to -Y): `width` across, `height` tall, `depth` deep."""
    def fill(bm, off=0.0):
        n_a, n_e = 16, 8
        rows = []
        for j in range(n_e + 1):
            e = (math.pi / 2) * j / n_e   # elevation from ground to crown
            row = []
            for i in range(n_a + 1):
                a = math.pi * i / n_a     # 0 at +X, pi at -X, through +Y (the back)
                x = (width / 2 - off) * math.cos(a) * math.cos(e)
                y = (depth - off) * math.sin(a) * math.cos(e) + y_shift
                z = (height - off) * math.sin(e)
                row.append(bm.verts.new((x, y, z)))
            rows.append(row)
        for r0, r1 in zip(rows, rows[1:]):
            for i in range(n_a):
                bm.faces.new((r0[i], r0[i + 1], r1[i + 1], r1[i]))
    outer = p.mesh(name, lambda bm: fill(bm), mat, smooth=True)
    inner = p.mesh(name + "Inner", lambda bm: fill(bm, thickness), inner_mat or mat, smooth=True)
    # The rim: the open front edge, a flat band joining outer and inner.
    def rim(bm):
        n_e = 8
        pts_o, pts_i = [], []
        for sgn in (1, -1):
            seq = range(n_e + 1) if sgn == 1 else range(n_e, -1, -1)
            for j in seq:
                e = (math.pi / 2) * j / n_e
                pts_o.append(Vector((sgn * (width / 2) * math.cos(e), y_shift, height * math.sin(e))))
                pts_i.append(Vector((sgn * (width / 2 - thickness) * math.cos(e), y_shift, (height - thickness) * math.sin(e))))
        for k in range(len(pts_o) - 1):
            v = [bm.verts.new(c) for c in (pts_o[k], pts_o[k + 1], pts_i[k + 1], pts_i[k])]
            bm.faces.new(v)
    p.mesh(name + "Rim", rim, p.mat("ShellRim", "rock", roughness=0.85, color="#c9b9a3", double=True))
    return outer, inner


@asset
def hall_shell():
    p = piece("city_hall_shell", "CityHallShell", "civic",
              "The concert hall's bandshell, 22 m wide and 14 m tall: three nested half-dome shells "
              "stepping forward, pale plaster outside and timber inside (acoustic wood), stitched "
              "with grown-timber ribs. Open to -Y towards the seating. Origin at the centre of its "
              "front line, ground level. Pair with city_hall_stage and city_hall_seating.")
    specs = ((22.0, 14.0, 10.0, 0.0), (17.0, 11.5, 8.0, 1.6), (12.5, 9.0, 6.0, 3.0))
    # Shells are open surfaces seen from both sides: their materials are double-sided.
    outer = p.mat("ShellPlaster", "brick", roughness=0.9, color="#d6c2a8", double=True)
    inner = p.mat("ShellTimber", "wood", roughness=0.7, color="#8a7a62", double=True)
    for k, (w, h, d, ys) in enumerate(specs):
        shell_surface(p, f"Shell{k}", w, h, d, outer, y_shift=ys, thickness=0.3, inner_mat=inner)
    w, h, d = specs[0][:3]
    for i in range(1, 8):   # meridian ribs on the outer shell
        a = math.pi * i / 8
        pts = []
        for j in range(9):
            e = (math.pi / 2) * j / 8
            pts.append(((w / 2 + 0.15) * math.cos(a) * math.cos(e), (d + 0.15) * math.sin(a) * math.cos(e),
                        (h + 0.15) * math.sin(e)))
        p.tube(f"Rib{i}", pts, 0.18, p.grown(), sides=6)
    core, halo = p.lantern()
    w3, h3, d3, y3 = specs[2]
    for i in range(1, 12):   # a string of pale lanterns under the innermost shell's rim
        e = math.pi * i / 12
        p.glow_point(f"Cove{i}", ((w3 / 2 - 0.6) * math.cos(e), y3 + 0.4, (h3 - 0.6) * math.sin(e)),
                     0.07, 0.2, core, halo)
    p.socket("Audio_hall", (0.0, 4.0, 3.0), "where the hall's music is emitted (the player's "
             "recordings play from here)", accepts="audio")
    p.notes.append("each shell is a double surface (plaster outside, timber inside) with a stone rim; "
                   "about 3 k triangles each")
    p.collider("box", (0, 9.5, 6), (22, 1.0, 12))
    p.collider("box", (-10.5, 4.5, 4), (1.0, 9, 8))
    p.collider("box", (10.5, 4.5, 4), (1.0, 9, 8))
    return p


@asset
def hall_rib():
    p = piece("city_hall_rib", "CityHallRib", "civic",
              "A single grown-timber shell rib, 12 m tall on a 9 m span: for extending the hall, "
              "framing a plaza or a gateway. Origin at the centre of its span, ground level.")
    pts = []
    for j in range(13):
        a = math.pi * j / 12
        pts.append((4.5 * math.cos(a), 0.0, 12.0 * math.sin(a)))
    p.tube("Rib", pts, [0.35, 0.3, 0.26, 0.23, 0.21, 0.2, 0.2, 0.2, 0.21, 0.23, 0.26, 0.3, 0.35], p.grown(), sides=8)
    for sx in (-1, 1):
        p.lathe(f"Foot{sx}", [(0.6, 0.0), (0.55, 0.35), (0.0, 0.4)], p.stone(), steps=12, centre=(sx * 4.5, 0, 0))
    p.collider("cylinder", (-4.5, 0, 1.5), (0.4, 3.0))
    p.collider("cylinder", (4.5, 0, 1.5), (0.4, 3.0))
    return p


@asset
def hall_stage():
    p = piece("city_hall_stage", "CityHallStage", "civic",
              "The concert hall's stage, 16 x 8 m and 1 m high: a carved stone front with ripple "
              "relief, a timber floor, side steps, and anchors for performers, instruments and the "
              "player's recordings. Origin at the centre of its front edge (front -Y), ground level.")
    p.box("Body", (-8.0, 0.0, 0.0), (8.0, 8.0, 0.9), p.stone())
    p.box("Floor", (-7.95, 0.05, 0.9), (7.95, 7.95, 1.0), p.timber())
    for x in (-4.0, 0.0, 4.0):
        p.ripples(f"Ripples{x}", x, 0.05, (0.3, 0.5, 0.7), 0.0, p.relief())
    for sx in (-1, 1):
        for k in range(4):
            x0 = sx * 8.0
            p.box(f"Step{sx}{k}", (min(x0, x0 + sx * (1.2 - 0.3 * k)), 0.6 + 0.01 * k, 0.0),
                  (max(x0, x0 + sx * (1.2 - 0.3 * k)), 2.6 - 0.01 * k, 0.25 * (k + 1)), p.stone())
    core, halo = p.lantern()
    for k in range(9):   # footlights: pale lantern bulbs along the stage's front lip
        p.glow_point(f"Footlight{k}", (-7.0 + 1.75 * k, 0.25, 1.06), 0.05, 0.14, core, halo, shape=(1.4, 1, 0.8))
    p.socket("Stage_centre", (0.0, 3.5, 1.0), "the performer's mark", accepts="performer")
    p.socket("Recording_source", (0.0, 5.0, 2.2), "the player's recordings play from here "
             "(anchor only; no content is baked)", accepts="audio")
    for k, x in enumerate((-5.0, -2.0, 2.0, 5.0)):
        p.socket(f"Instrument_{k}", (x, 4.5, 1.0), "an instrument or machine on stage", accepts="instrument")
    p.collider("box", (0, 4, 0.5), (16, 8, 1.0), walkable=True)
    return p


@asset
def hall_seating():
    p = piece("city_hall_seating", "CityHallSeating", "civic",
              "An 8 m segment of hall seating: four stone tiers (0.45 m rise, 0.9 m tread) with "
              "timber seat boards, climbing towards +Y. Lay segments edge to edge, each turned a "
              "little, for a fan. Origin at its front left corner, ground level.")
    sec = [(0.0, 0.0)]
    for k in range(4):
        sec += [(0.9 * k, 0.45 * (k + 1)), (0.9 * (k + 1), 0.45 * (k + 1))]
    sec += [(3.6, 0.0)]
    p.section("Tiers", [(y, z) for y, z in sec], 0.0, 8.0, p.stone())
    for k in range(4):
        p.box(f"Seat{k}", (0.1, 0.9 * k + 0.05, 0.45 * (k + 1)), (7.9, 0.9 * k + 0.5, 0.45 * (k + 1) + 0.06),
              p.timber(), small=True)
        for s in range(4):
            p.socket(f"Seat_{k}_{s}", (1.0 + 2.0 * s, 0.9 * k + 0.3, 0.45 * (k + 1) + 0.06),
                     "a seat for a Dancer in the audience", accepts="seat")
    p.connector("left", (0, 0, 0), (-1, 0, 0))
    p.connector("right", (8, 0, 0), (1, 0, 0))
    p.collider("box", (4, 1.8, 0.9), (8, 4.0, 0.3), slope=math.atan2(1.8, 3.6), walkable=True)
    return p


# ------------------------------------------------------------------ CIV03 monument


@asset
def monument():
    p = piece("city_monument", "CityMonument", "civic",
              "The pale monument, 28 m: three tapering stone fins rise from a triangular footing "
              "and lean together, holding a 7 m ring between them at 19 m: an abstract sun gate in the "
              "pale mineral that catches rosy light. Seen from everywhere in the district. Origin "
              "at the centre of its base.")
    def fin(bm, a):
        d = Vector((math.cos(a), math.sin(a), 0))
        s = Vector((-d.y, d.x, 0))
        rows = []
        n = 12
        for i in range(n + 1):
            t = i / n
            z = 28.0 * t
            r = 3.8 * (1 - t) ** 1.2 + 0.35      # distance from the axis: leaning inwards
            w = 2.1 * (1 - t) ** 0.8 + 0.15       # fin half-width
            th = 1.3 * (1 - t) + 0.18             # fin half-thickness
            c = d * r + Vector((0, 0, z))
            rows.append([bm.verts.new(c + s * w), bm.verts.new(c + d * th), bm.verts.new(c - s * w),
                         bm.verts.new(c - d * th * 0.6)])
        for ra, rb in zip(rows, rows[1:]):
            for k in range(4):
                bm.faces.new((ra[k], ra[(k + 1) % 4], rb[(k + 1) % 4], rb[k]))
        bm.faces.new(rows[0][::-1])
        bm.faces.new(rows[-1])
    for k in range(3):
        a = math.tau * k / 3 + math.pi / 2
        p.mesh(f"Fin{k}", lambda bm, a=a: fin(bm, a), p.monument(), smooth=False)
    def ring(bm):
        R, r, n, m = 3.4, 0.38, 48, 8
        rings = []
        for i in range(n):
            a = math.tau * i / n
            c = Vector((math.cos(a) * R, 0, 19.0 + math.sin(a) * R))
            out = Vector((math.cos(a), 0, math.sin(a)))
            rings.append([bm.verts.new(c + r * (out * math.cos(math.tau * j / m) + Vector((0, 1, 0)) * math.sin(math.tau * j / m))) for j in range(m)])
        for i in range(n):
            for j in range(m):
                bm.faces.new((rings[i][j], rings[i][(j + 1) % m], rings[(i + 1) % n][(j + 1) % m], rings[(i + 1) % n][j]))
    p.mesh("Ring", ring, p.monument(), smooth=True)
    p.plan("Footing", circle(0, 0, 6.2, 9), 0.0, 0.6, p.stone())
    # Far level: three plain fins.
    for k in range(3):
        a = math.tau * k / 3 + math.pi / 2
        d = Vector((math.cos(a), math.sin(a), 0))
        p.tube(f"FarFin{k}", [tuple(d * 3.2), tuple(d * 1.5 + Vector((0, 0, 14))), tuple(d * 0.4 + Vector((0, 0, 28)))],
               [1.0, 0.6, 0.15], p.monument(), sides=4, far=True, smooth=False)
    p.notes.append("a landmark silhouette for navigation; its rosy cast comes from the dusk keys, "
                   "the material is the palest pale stone (Monument #eadbd2)")
    p.collider("cylinder", (0, 0, 3.0), (3.6, 6.0))
    return p


@asset
def monument_base():
    p = piece("city_monument_base", "CityMonumentBase", "civic",
              "A modular 8 x 8 m stepped plinth (three 0.3 m steps to 0.9 m) for the monument, a "
              "statue, or a plaza stage. Origin at the centre of its base.")
    for k in range(3):
        h = 4.0 - 0.6 * k
        p.box(f"Step{k}", (-h, -h, 0.3 * k), (h, h, 0.3 * (k + 1)), p.stone(), round=0.3)
    p.strata("Strata", -4.0, 4.0, 0.08, (0.05,), -4.0, p.relief(), depth=0.02)
    p.socket("Top", (0, 0, 0.9), "what stands on the plinth", accepts="monument")
    p.collider("box", (0, 0, 0.45), (8, 8, 0.9), walkable=True)
    return p


@asset
def fountain():
    p = piece("city_fountain", "CityFountain", "civic",
              "A round basin, 6 m across: a stone rim at seat height, still water that holds the "
              "sky, and a small stepped centre stone. Origin at its centre, ground level.")
    ring_o, ring_i = circle(0, 0, 3.0, 36), circle(0, 0, 2.55, 36)
    p.plan_holes("Rim", ring_o, [ring_i], 0.0, 0.5, p.stone())
    p.plan_holes("RimCap", circle(0, 0, 3.08, 36), [circle(0, 0, 2.5, 36)], 0.5, 0.56, p.stone_dark(), small=True)
    p.plan("Floor", circle(0, 0, 2.56, 36), -0.1, 0.0, p.bed())
    p.plan("Water", circle(0, 0, 2.56, 36), 0.34, 0.35, p.water())
    for k, r in enumerate((0.9, 0.6, 0.35)):
        p.plan(f"Centre{k}", circle(0, 0, r, 16), 0.0 if k == 0 else 0.35 + 0.15 * (k - 1), 0.45 + 0.15 * k, p.stone())
    p.socket("Audio_water", (0, 0, 0.5), "the fountain's water sound", accepts="audio")
    p.socket("Seat", (0.0, -3.0, 0.56), "the rim is a seat", accepts="seat")
    p.collider("cylinder", (0, 0, 0.28), (3.0, 0.56))
    return p


# ------------------------------------------------------------------ VEG02 luminous flora


@asset
def glow_bell():
    p = piece("city_glow_bell", "CityGlowBell", "glow",
              "A bell flower, 1.7 m: three arching grown stems each hanging a translucent cyan "
              "jellyfish bell over a bright core with two tendrils. Separate core and halo meshes; "
              "authored at peak, breathed by the audio clock. Origin at the base's centre.",
              phases=("town", "city"))
    core, halo = p.glow("Cyan")
    p.blades("Leaves", (0, 0, 0), 7, 0.6, 0.12, p.leaf(), spread=0.05, lean=0.5)
    for k in range(3):
        a = math.tau * k / 3 + 0.3
        d = Vector((math.cos(a), math.sin(a), 0))
        h = 1.3 + 0.2 * k
        tip = d * 0.55 + Vector((0, 0, h))
        p.tube(f"Stem{k}", [(0, 0, 0), tuple(d * 0.1 + Vector((0, 0, h * 0.7))), tuple(d * 0.45 + Vector((0, 0, h + 0.15))), tuple(tip)],
               [0.03, 0.022, 0.016, 0.012], p.sprout(), sides=5, small=True)
        p.bell(f"Bell{k}", tuple(tip), 0.18, 0.2, core, halo)
    return p


@asset
def glow_vein_leaf():
    p = piece("city_glow_vein_leaf", "CityGlowVeinLeaf", "glow",
              "A great vein leaf, 1.5 m: a broad leaf on a curving stalk whose veins glow magenta "
              "(bright threads inside soft halo threads). Origin at the stalk's foot.",
              phases=("town", "city"))
    core, halo = p.glow("Magenta")
    base = Vector((0, 0, 0))
    stalk = [base, Vector((0.05, 0, 0.5)), Vector((0.25, 0, 0.9))]
    p.tube("Stalk", [tuple(v) for v in stalk], [0.035, 0.03, 0.025], p.sprout(), sides=5, small=True)
    origin, L, W = Vector((0.25, 0, 0.9)), 1.1, 0.45
    def blade(bm):
        rows = []
        for i in range(9):
            t = i / 8
            w = W * math.sin(math.pi * t) ** 0.8
            c = origin + Vector((L * t, 0, 0.35 * t - 0.45 * t * t))
            rows.append((bm.verts.new(c + Vector((0, -w, -0.05 * w))), bm.verts.new(c), bm.verts.new(c + Vector((0, w, -0.05 * w)))))
        for a, b in zip(rows, rows[1:]):
            bm.faces.new((a[0], a[1], b[1], b[0]))
            bm.faces.new((a[1], a[2], b[2], b[1]))
    p.mesh("Blade", blade, p.leaf(), smooth=True, small=True)
    def vein_points(t0, side):
        c0 = origin + Vector((L * t0, 0, 0.35 * t0 - 0.45 * t0 * t0 + 0.01))
        w = W * math.sin(math.pi * t0) ** 0.8 * 0.85
        return [tuple(c0), tuple(c0 + Vector((0.18, side * w * 0.6, -0.01))), tuple(c0 + Vector((0.3, side * w, -0.04)))]
    mid = [tuple(origin + Vector((L * t, 0, 0.35 * t - 0.45 * t * t + 0.012))) for t in (0, 0.25, 0.5, 0.75, 0.97)]
    p.tube("Vein_Core", mid, 0.008, core, sides=4, small=True)
    p.tube("Vein_Halo", mid, 0.03, halo, sides=6, small=True)
    for k, t0 in enumerate((0.2, 0.42, 0.64)):
        for side in (-1, 1):
            pts = vein_points(t0, side)
            p.tube(f"Vein{k}{side}_Core", pts, 0.006, core, sides=4, small=True)
            p.tube(f"Vein{k}{side}_Halo", pts, 0.024, halo, sides=6, small=True)
    return p


@asset
def glow_tendril():
    p = piece("city_glow_tendril", "CityGlowTendril", "glow",
              "Hanging glow tendrils, 1.6 m: seven green threads with bead motes, for the undersides "
              "of bridges, balconies and arches. Origin at the attachment point; hangs down (-Z).",
              phases=("town", "city"))
    core, halo = p.glow("Green")
    for k in range(7):
        x = -0.5 + k / 6
        L = 0.9 + 0.7 * p.rng.random()
        pts = [(x + 0.05 * math.sin(i + k), 0.05 * math.cos(i * 1.3 + k), -L * i / 5) for i in range(6)]
        p.tube(f"Thread{k}_Core", pts, 0.006, core, sides=4, small=True)
        p.tube(f"Thread{k}_Halo", pts, 0.022, halo, sides=5, small=True)
        p.glow_point(f"Bead{k}", pts[-1], 0.025, 0.07, core, halo)
    p.box("Holdfast", (-0.55, -0.04, -0.03), (0.55, 0.04, 0.0), p.grown(), small=True)
    return p


@asset
def glow_motes():
    p = piece("city_glow_motes", "CityGlowMotes", "glow",
              "A drift of fairy motes: nine soft points of cyan, magenta and green between 0.6 and "
              "2.4 m over a 3 m patch, each a core in its halo. Their drift and breathing are "
              "runtime work (anchors Mote_k). Origin at the patch's ground centre.",
              phases=("town", "city"))
    colours = ("Cyan", "Magenta", "Green")
    mats = {c: p.glow(c) for c in colours}
    for k in range(9):
        c = colours[k % 3]
        pos = (p.rng.uniform(-1.4, 1.4), p.rng.uniform(-1.4, 1.4), p.rng.uniform(0.6, 2.4))
        p.glow_point(f"Mote{k}", pos, 0.03, 0.11, *mats[c])
        p.socket(f"Mote_{k}", pos, f"a {c.lower()} mote (the game drifts it)", accepts="mote")
    return p


# ------------------------------------------------------------------ MUS01 water music staff

STAFF_SPACING = 0.18


def staff_lines(p, paths, n_notes):
    """Five luminous lines (amber, the stream staff's signal colour), core and halo."""
    core, halo = p.signal()
    p.lights[core.name] = ("the staff lines (amber, the stream staff's colour), emissive at peak; "
                           "the game brightens them as the water clears")
    for k, pts in enumerate(paths):
        p.tube(f"Line{k}_Core", pts, 0.012, core, sides=4, small=True)
        p.tube(f"Line{k}_Halo", pts, 0.04, halo, sides=6, small=True)


@asset
def staff_4():
    p = piece("city_staff_4", "CityStaff4", "music",
              "A 4 m straight of the floating music staff: five luminous lines 0.18 m apart, 0.15 m "
              "above the water, on the canal's centreline. Origin at its centreline start on the "
              "water surface (place at the canal's water level). Notes are runtime; anchors only.",
              phases=("village", "town", "city"))
    paths = [[(x, (k - 2) * STAFF_SPACING, 0.15) for x in (0.0, 1.0, 2.0, 3.0, 4.0)] for k in range(5)]
    staff_lines(p, paths, 4)
    for i in range(4):
        p.socket(f"Note_{i}", (0.5 + i, 0.0, 0.15), "a beat's note rides here (runtime)", accepts="note")
    p.socket("Bar_0", (0.0, 0.0, 0.15), "a bar line may stand here (runtime)", accepts="bar")
    p.connector("upstream", (0, 0, 0), (-1, 0, 0))
    p.connector("downstream", (4, 0, 0), (1, 0, 0))
    return p


@asset
def staff_curve():
    p = piece("city_staff_curve", "CityStaffCurve", "music",
              "A quarter turn of the floating staff: 90 degrees to the left on a 4 m centreline "
              "radius, from heading +X to heading +Y. Same origin rule as city_staff_4.",
              phases=("village", "town", "city"))
    R = 4.0
    paths = []
    for k in range(5):
        r = R - (k - 2) * STAFF_SPACING
        paths.append([(r * math.sin(a), R - r * math.cos(a), 0.15)
                      for a in (math.radians(90 * i / 12) for i in range(13))])
    staff_lines(p, paths, 6)
    for i in range(6):
        a = math.radians(90 * (i + 0.5) / 6)
        p.socket(f"Note_{i}", (R * math.sin(a), R - R * math.cos(a), 0.15), "a beat's note rides here", accepts="note")
    p.connector("upstream", (0, 0, 0), (-1, 0, 0))
    p.connector("downstream", (R, R, 0), (0, 1, 0))
    return p
