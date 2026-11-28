"""City kit, P2: street furniture, infrastructure (machines are instruments: amber signal,
interaction and audio anchors) and growth overlays. Same contract as P0 (city_kit.py)."""
import math

import bmesh
from mathutils import Vector

from city_kit import BAY, WATER_Z, BED_Z, Piece, circle, rounded_rect

P2 = {}


def asset(fn):
    P2["city_" + fn.__name__] = fn
    return fn


def piece(aid, kind, catalog, about, phases=("town", "city")):
    return Piece(aid, kind, catalog=catalog, priority="P2", phases=phases, about=about)


def machine(p, signal_at, interact_at, audio_at):
    """What every machine carries: one amber signal light, an interaction point and an
    audio source. Nothing animates on its own; the rack's state drives the light."""
    core, halo = p.signal()
    p.glow_point("Light_signal", signal_at, 0.035, 0.09, core, halo)
    p.socket("Interact", interact_at, "E opens this machine's rack", accepts="interact")
    p.socket("Audio_source", audio_at, "where this machine's voice is heard from", accepts="audio")
    p.root["kind"] = "machine"


# ------------------------------------------------------------------ PRP01 street furniture


@asset
def bench():
    p = piece("city_bench", "CityBench", "street",
              "A 2 m bench: a timber seat on two rounded stone blocks. Origin at its base's centre; "
              "sitters face -Y.", phases=("village", "town", "city"))
    for x in (-0.75, 0.75):
        p.box(f"Block{x}", (x - 0.2, -0.22, 0.0), (x + 0.2, 0.22, 0.4), p.stone(), round=0.08)
    for k in range(4):
        y = -0.24 + 0.125 * k
        p.box(f"Slat{k}", (-1.0, y, 0.4), (1.0, y + 0.11, 0.46), p.timber(), small=True)
    p.socket("Seat", (0.0, 0.0, 0.46), "a seat for two", accepts="seat")
    p.collider("box", (0, 0, 0.23), (2.0, 0.5, 0.46))
    return p


@asset
def shade_sail():
    p = piece("city_shade_sail", "CityShadeSail", "street",
              "A triangular shade sail, about 6 m across: bone canvas tensioned between three "
              "leaning titanium masts (3.2-4.2 m). Origin at the centre of its footprint.",
              phases=("village", "town", "city"))
    corners = [(math.cos(a) * 3.2, math.sin(a) * 3.2, h) for a, h in
               ((math.radians(90), 4.2), (math.radians(210), 3.3), (math.radians(330), 3.6))]
    for k, (x, y, h) in enumerate(corners):
        foot = (x * 1.12, y * 1.12, 0.0)
        p.tube(f"Mast{k}", [foot, (x, y, h + 0.25)], [0.07, 0.05], p.titanium(), sides=7)
    def sail(bm):
        n = 6
        c = [Vector(v) for v in corners]
        rows = []
        for i in range(n + 1):
            row = []
            for j in range(n + 1 - i):
                a, b = i / n, j / n
                pnt = c[0] * (1 - a - b) + c[1] * a + c[2] * b
                sag = 0.45 * (1 - a - b) * a * b * 27 / 1.0 * 0.15
                row.append(bm.verts.new(pnt - Vector((0, 0, sag))))
            rows.append(row)
        for i in range(n):
            for j in range(n - i):
                bm.faces.new((rows[i][j], rows[i + 1][j], rows[i][j + 1]))
                if j < n - i - 1:
                    bm.faces.new((rows[i + 1][j], rows[i + 1][j + 1], rows[i][j + 1]))
    p.mesh("Sail", sail, p.canvas(), smooth=True)
    for k, (x, y, h) in enumerate(corners):
        p.collider("cylinder", (x * 1.06, y * 1.06, h / 2), (0.08, h))
    return p


@asset
def bollard():
    p = piece("city_bollard", "CityBollard", "street",
              "A lantern bollard, 1 m: a rounded stone post whose top holds a pale lantern bulb in "
              "its halo. Origin at its base's centre.", phases=("village", "town", "city"))
    p.lathe("Post", [(0.16, 0.0), (0.16, 0.7), (0.12, 0.78), (0.12, 0.82), (0.0, 0.82)], p.stone(), steps=12)
    for k in range(3):
        a = math.tau * k / 3
        p.box(f"Cage{k}", (0.09 * math.cos(a) - 0.015, 0.09 * math.sin(a) - 0.015, 0.82),
              (0.09 * math.cos(a) + 0.015, 0.09 * math.sin(a) + 0.015, 1.02), p.titanium(), small=True)
    p.lathe("Cap", [(0.13, 1.02), (0.1, 1.07), (0.0, 1.09)], p.titanium(), steps=12, small=True)
    core, halo = p.lantern()
    p.glow_point("Lamp", (0, 0, 0.92), 0.05, 0.14, core, halo, shape=(1, 1, 1.2))
    p.collider("cylinder", (0, 0, 0.5), (0.16, 1.0))
    return p


@asset
def sign():
    p = piece("city_sign", "CitySign", "street",
              "A wayfinding stele, 2.2 m: a rounded stone slab carved with strata and a mesa step, "
              "holding an empty graphene panel where the game writes a place name or arrow. "
              "Origin at its base's centre; reads towards -Y.", phases=("village", "town", "city"))
    p.box("Slab", (-0.4, -0.12, 0.0), (0.4, 0.12, 2.2), p.stone(), round=0.1)
    p.box("Panel", (-0.32, -0.14, 1.1), (0.32, -0.12, 1.9), p.graphene(), small=True)
    p.mesa_steps("Mesa", 0.0, 0.3, 0.6, 3, 0.12, -0.12, p.relief(), depth=0.03)
    p.strata("Strata", -0.38, 0.38, 0.85, (0.04, 0.03), -0.12, p.relief(), depth=0.02)
    p.socket("Sign_face", (0.0, -0.15, 1.5), "the panel's text or symbol, written at runtime", accepts="sign",
             rotation=(math.pi / 2, 0, 0))
    p.collider("box", (0, 0, 1.1), (0.8, 0.24, 2.2))
    return p


@asset
def instrument_pedestal():
    p = piece("city_instrument_pedestal", "CityInstrumentPedestal", "street",
              "A street instrument: a stone pedestal, 1.1 m, carrying a tilted graphene face where "
              "a small rack mounts, with its amber signal light, an interaction point and an audio "
              "source. A machine, so an instrument (the city's busker spot). Origin at its base's centre.")
    p.box("Pedestal", (-0.35, -0.3, 0.0), (0.35, 0.3, 0.95), p.stone(), round=0.1)
    def face(bm):
        pts = [(-0.4, -0.42, 0.95), (0.4, -0.42, 0.95), (0.4, 0.3, 1.2), (-0.4, 0.3, 1.2)]
        v = [bm.verts.new(c) for c in pts]
        bm.faces.new(v)
        bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.04)
    p.mesh("Face", face, p.graphene())
    p.box("Trim", (-0.42, -0.45, 0.92), (0.42, -0.4, 0.98), p.titanium(), small=True)
    machine(p, (0.3, -0.38, 1.0), (0.0, -0.6, 1.0), (0.0, 0.0, 1.2))
    p.socket("Rack", (0.0, -0.06, 1.1), "where the instrument's small rack mounts", accepts="rack")
    p.collider("box", (0, 0, 0.55), (0.8, 0.8, 1.1))
    return p


# ------------------------------------------------------------------ TEC01 infrastructure


@asset
def solar_canopy():
    p = piece("city_solar_canopy", "CitySolarCanopy", "infrastructure",
              "A solar tree, 5.5 m: a titanium trunk branching into five graphene leaf panels that "
              "shade the street. Origin at its base's centre.")
    p.tube("Trunk", [(0, 0, 0), (0, 0, 3.6)], [0.16, 0.11], p.titanium(), sides=8)
    for k in range(5):
        a = math.tau * k / 5
        d = Vector((math.cos(a), math.sin(a), 0))
        tip = d * 1.6 + Vector((0, 0, 4.9 + 0.3 * (k % 2)))
        p.tube(f"Branch{k}", [(0, 0, 3.4), tuple(d * 0.8 + Vector((0, 0, 4.3))), tuple(tip)], [0.07, 0.05, 0.035],
               p.titanium(), sides=6, small=True)
        def leaf(bm, d=d, tip=tip):
            s = Vector((-d.y, d.x, 0))
            pts = [tip - d * 1.1, tip - d * 0.3 + s * 0.75, tip + d * 0.9, tip - d * 0.3 - s * 0.75]
            v = [bm.verts.new(c + Vector((0, 0, 0.12 * (i % 2)))) for i, c in enumerate(pts)]
            bm.faces.new(v)
            bmesh.ops.solidify(bm, geom=bm.faces[:], thickness=0.04)
        p.mesh(f"Panel{k}", leaf, p.graphene())
    p.lathe("Foot", [(0.4, 0.0), (0.35, 0.15), (0.16, 0.25), (0.0, 0.25)], p.stone(), steps=12, small=True)
    p.collider("cylinder", (0, 0, 1.8), (0.2, 3.6))
    return p


@asset
def pump_house():
    p = piece("city_pump_house", "CityPumpHouse", "infrastructure",
              "A pump house, 3 x 3 m and 3.6 m tall: plaster with a rounded roof lip, titanium "
              "louvres and a pipe stub to the canal. A machine and so an instrument: amber signal, "
              "interaction and audio anchors. Origin at its base's centre; door -Y.")
    p.box("Body", (-1.5, -1.5, 0.0), (1.5, 1.5, 3.0), p.plaster(), round=0.35)
    p.box("Roof", (-1.7, -1.7, 3.0), (1.7, 1.7, 3.25), p.stone(), round=0.45)
    p.box("RoofSoil", (-1.45, -1.45, 3.25), (1.45, 1.45, 3.3), p.moss(), round=0.3)
    for k in range(5):
        z = 1.4 + 0.22 * k
        p.box(f"Louvre{k}", (1.5, -0.7, z), (1.62, 0.7, z + 0.08), p.titanium(), small=True)
    p.box("Door", (-0.55, -1.53, 0.0), (0.55, -1.48, 2.1), p.timber())
    p.tube("Pipe", [(-1.5, 0.6, 0.5), (-2.4, 0.6, 0.5), (-2.4, 0.6, -0.3)], 0.15, p.titanium(), sides=8)
    machine(p, (0.8, -1.56, 1.8), (0.0, -1.9, 1.0), (0.0, 0.0, 1.5))
    p.socket("Plant_roof", (0.0, 0.0, 3.3), "the roof's planting", accepts="shrub")
    p.collider("box", (0, 0, 1.6), (3.4, 3.4, 3.2))
    return p


@asset
def water_gate():
    p = piece("city_water_gate", "CityWaterGate", "infrastructure",
              "A sluice gate across the 8 m canal: two stone piers on the copings, a titanium frame "
              "and a raised gate leaf, with a hand wheel. A machine and so an instrument (it sets "
              "the canal's flow): amber signal, interaction and audio anchors. Same origin as a "
              "canal tile (near wall line, y 0); spans to y 8; 1.2 m along +X.")
    for y0, y1 in ((-0.65, 0.35), (7.65, 8.65)):
        p.box(f"Pier{y0}", (0.0, y0, 0.0), (1.2, y1, 3.6), p.stone(), round=0.2)
        p.box(f"Cap{y0}", (-0.05, y0 - 0.05, 3.6), (1.25, y1 + 0.05, 3.75), p.stone_dark(), small=True)
    p.box("Lintel", (0.3, 0.35, 3.2), (0.9, 7.65, 3.55), p.titanium())
    for y in (0.35, 7.65):
        p.box(f"Guide{y}", (0.45, y - 0.12, BED_Z), (0.75, y + 0.12, 3.2), p.titanium(), small=True)
    p.box("Leaf", (0.5, 0.3, WATER_Z + 1.4), (0.7, 7.7, WATER_Z + 3.1), p.graphene())
    p.tube("Wheel", [(1.3, 4.0 + 0.5 * math.cos(a), 3.4 + 0.5 * math.sin(a)) for a in
                     (math.tau * k / 16 for k in range(17))], 0.04, p.grown(), sides=5, small=True)
    machine(p, (1.25, -0.3, 2.6), (1.6, -0.2, 1.2), (0.6, 4.0, WATER_Z))
    p.collider("box", (0.6, -0.15, 1.8), (1.2, 1.0, 3.6))
    p.collider("box", (0.6, 8.15, 1.8), (1.2, 1.0, 3.6))
    return p


@asset
def pipe_4():
    p = piece("city_pipe_4", "CityPipe4", "infrastructure",
              "A 4 m run of titanium pipe 0.3 m up on two grown-timber cradles: water and power "
              "between machines. Linear along +X from its origin.")
    p.tube("Pipe", [(0, 0, 0.3), (BAY, 0, 0.3)], 0.12, p.titanium(), sides=10)
    for x in (0.6, 3.4):
        p.box(f"Cradle{x}", (x - 0.08, -0.16, 0.0), (x + 0.08, 0.16, 0.2), p.grown(), small=True)
        p.tube(f"Clamp{x}", [(x, 0.13 * math.cos(a), 0.3 + 0.13 * math.sin(a)) for a in
                             (math.pi * k / 8 for k in range(9))], 0.02, p.titanium(), sides=4, small=True)
    p.connector("start", (0, 0, 0.3), (-1, 0, 0))
    p.connector("end", (BAY, 0, 0.3), (1, 0, 0))
    return p


# ------------------------------------------------------------------ VEG03 growth overlays


def creepers(p, density, bloom):
    """Climbing stems up a facade's face (y 0, front -Y), with leaf cards; more and higher
    with `density`."""
    stems = int(3 + 6 * density)
    height = 1.2 + 2.4 * density

    def fill(bm, leaves):
        for k in range(stems):
            x = BAY * (k + 0.5) / stems + p.rng.uniform(-0.2, 0.2)
            h = height * p.rng.uniform(0.6, 1.0)
            n = int(6 + 10 * density)
            for i in range(n):
                t = i / max(n - 1, 1)
                c = Vector((x + 0.18 * math.sin(t * 7 + k), -0.04, h * t))
                if not leaves:
                    continue
                s = 1 if i % 2 else -1
                v = [bm.verts.new(c + Vector(d)) for d in ((0, 0, 0), (s * 0.16, -0.05, 0.06),
                                                         (s * 0.12, -0.07, 0.18), (0, -0.02, 0.12))]
                bm.faces.new(v)
    p.mesh("Leaves", lambda bm: fill(bm, True), p.leaf() if density < 0.9 else p.leaf_young(), small=True)
    for k in range(stems):
        x = BAY * (k + 0.5) / stems
        p.tube(f"Stem{k}", [(x, -0.03, 0.0), (x + 0.1, -0.03, height * 0.5), (x - 0.05, -0.03, height * 0.9)],
               0.015, p.grown(), sides=4, small=True)
    if bloom:
        def blooms(bm):
            for k in range(int(10 * density)):
                c = Vector((p.rng.uniform(0.2, BAY - 0.2), -0.12, p.rng.uniform(0.6, height)))
                g = bmesh.ops.create_icosphere(bm, subdivisions=1, radius=0.05)
                for v in g["verts"]:
                    v.co += c
        p.mesh("Blooms", blooms, p.bloom(), small=True)


def overlay(stage, density, bloom, about):
    def build():
        p = piece(f"city_greenery_{stage}", f"CityGreenery{stage.capitalize()}", "growth", about,
                  phases=("village", "town", "city"))
        creepers(p, density, bloom)
        p.connector("facade", (0, 0, 0), (1, 0, 0))
        p.notes.append("an overlay: place at a facade bay's origin; it adds plants, never structure")
        return p
    build.__name__ = f"greenery_{stage}"
    return asset(build)


overlay("sprout", 0.15, False, "Growth overlay, sprout: a few young creepers starting up a 4 m facade bay.")
overlay("established", 0.55, False, "Growth overlay, established: creepers covering the lower half of a 4 m bay.")
overlay("lush", 1.0, True, "Growth overlay, lush: a facade bay climbed to the band, in flower.")


@asset
def tree_sapling():
    p = piece("city_tree_sapling", "CityTreeSapling", "growth",
              "A young street tree, 2.6 m, in a staked timber guard: the city_tree before it grows "
              "(the bare-to-lush stages of a planting). Origin at its base's centre.",
              phases=("village", "town", "city"))
    p.tube("Trunk", [(0, 0, 0), (0.02, 0, 1.4), (0, 0.03, 2.0)], [0.07, 0.05, 0.03], p.grown(), sides=6)
    p.canopy("Crown", (0, 0.03, 2.2), 0.6, p.leaf_young(), lobes=4, flat=0.8)
    for k in range(3):
        a = math.tau * k / 3
        x, y = 0.3 * math.cos(a), 0.3 * math.sin(a)
        p.box(f"Stake{k}", (x - 0.025, y - 0.025, 0.0), (x + 0.025, y + 0.025, 1.3), p.timber(), small=True)
    p.tube("Tie", [(0.3 * math.cos(a), 0.3 * math.sin(a), 1.1) for a in (math.tau * k / 3 for k in range(4))],
           0.012, p.grass(), sides=3, small=True)
    p.plan("Grate", circle(0, 0, 0.5, 12), 0.0, 0.02, p.graphene())
    return p
