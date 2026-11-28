"""The City reference district and the modular test strip, built from the City kit.

  blender -b --factory-startup -P tools/blender/create_city_district.py            # build, save, render
  blender -b --factory-startup -P tools/blender/create_city_district.py -- norender

Run `create_city_kit.py` first: this links each piece's asset collection from
assets-src/blender/structures/city/<id>.blend (library instances, never copies) and
reads socket positions from docs/art/city-kit/manifest.json to dress planting.

Writes
  assets-src/blender/structures/city/city_district.blend
      scenes Day, Dusk and Night share one District collection (identical geometry);
      collections Blockout (hidden masses, the first pass), District, Test-Strip, Context;
      cameras Approach-Reveal, Canal-Eye, Civic-Plaza, Rooftop-Overview, Night-View, Test-Strip
  previews/structures/city/district_<camera>_<light>.jpg
  assets/world/asset_reviews/city.bsn   the same layout as a Jackdaw review scene
  docs/art/city-kit/district_layout.json

The district is about 144 x 100 m: the old mud stream arrives from the camp in the west,
becomes the canal at its head and runs east past residential blocks, turns north at the
civic plaza (concert hall, gallery across a footbridge) and ends at a second head under the
gallery. An elevated light rail runs along the north edge with a station; the pale
monument stands beyond the plaza on the canal's axis.
"""
from pathlib import Path
import json
import math
import random
import sys

import bpy
from mathutils import Euler, Matrix, Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from city_layout import Frame, Layout  # noqa: E402
from city_mixed import furnished_home, mixed_block  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
SOURCES = ROOT / "assets-src/blender/structures/city"
MANIFEST = json.loads((ROOT / "docs/art/city-kit/manifest.json").read_text())
PREVIEWS = ROOT / "previews/structures/city"
WATER_Z = -0.9
TRACK_Z = 7.65
JACKDAW_LIFT = 2.3   # the review scene stands the district on the ground: canal bed above y 0

SKIN = {"solar_amber": "#D89B54", "clay_coral": "#C96D74", "leaf_teal": "#4F9D88", "dusk_violet": "#8878AB"}
SHAPES = ("full", "lanky", "broad", "compact")


def srgb(h):
    h = h.lstrip("#")
    c = [int(h[i:i + 2], 16) / 255 for i in (0, 2, 4)]
    return tuple(x / 12.92 if x <= 0.04045 else ((x + 0.055) / 1.055) ** 2.4 for x in c)


# ====================================================================== layout


# ---------------------------------------------------------------------- buildings

PATTERNS = {
    "plaster": ("city_facade_window", "city_facade_window", "city_facade_blank", "city_facade_window",
                "city_facade_green_b"),
    "mixed": ("city_facade_window", "city_facade_window_b", "city_facade_window", "city_facade_green_b",
              "city_facade_blank", "city_facade_window_b"),
    "glass": ("city_facade_window_b", "city_facade_window_b", "city_facade_green_b", "city_facade_window_b",
              "city_facade_window"),
}


def block(L, F, nx, ny, storeys, *, style="mixed", arcade=True, setback_at=None, walks=False,
          roof="planted", balconies=True, group="District", seed=0, front_override=None):
    """A residential block on the 4 m grid, in its own frame: footprint [0, 4nx] x [0, 4ny],
    front along y 0 facing -Y. `front_override(i, s)` may replace a front bay's module."""
    pat = PATTERNS[style]
    rng = random.Random(seed)

    def pick(i, s, side):
        k = (i * 3 + s * 2 + side + seed) % len(pat)
        return pat[k]

    for s in range(storeys):
        z = 4.0 * s
        y_front = 4.0 if (setback_at is not None and s >= setback_at) else 0.0
        j0 = 1 if y_front else 0
        # Front (south in the block's frame).
        for i in range(nx):
            mod = front_override(i, s) if front_override else None
            if mod == "skip":
                continue
            if s == 0 and arcade:
                L.put("city_facade_arcade", (4 * i, 0, z), 0, group, F)
                L.put("city_facade_door" if i % 2 == 0 else "city_facade_window", (4 * i, 4, z), 0, group, F)
                L.put("city_slab_4", (4 * i, 0, 3.98), 0, group, F)
            else:
                L.put(mod or pick(i, s, 0), (4 * i, y_front, z), 0, group, F)
                if balconies and s >= 1 and not walks and (i + s) % 2 == 1 and not y_front and not mod:
                    L.put("city_balcony", (4 * i, y_front, z), 0, group, F)
        if s == 0 and arcade:
            L.put("city_facade_end", (0, 4, z), 0, group, F)
            L.put("city_facade_end", (4 * nx, 4, z), 0, group, F)
        # Back (north in the block's frame), turned 180.
        for i in range(nx):
            mod = "city_facade_door" if (s == 0 and i == nx // 2) else pick(i, s, 1)
            L.put(mod, (4 * (i + 1), 4 * ny, z), 180, group, F)
        # Sides.
        for j in range(j0, ny):
            west = "city_facade_arcade" if (s == 0 and arcade and j == 0) else pick(j, s, 2)
            east = "city_facade_arcade" if (s == 0 and arcade and j == 0) else pick(j, s, 3)
            if west == "city_facade_window_b" and j == 0:
                west = "city_facade_window"
            L.put(west, (0, 4 * (j + 1), z), -90, group, F)
            L.put(east, (4 * nx, 4 * j, z), 90, group, F)
        # Corners.
        for (cx, cy, r) in ((0, y_front, 0), (4 * nx, y_front, 90), (4 * nx, 4 * ny, 180), (0, 4 * ny, -90)):
            L.put("city_facade_corner", (cx, cy, z), r, group, F)
        # Setback terrace on the storey where the front steps back.
        if setback_at is not None and s == setback_at:
            for i in range(nx):
                L.put("city_garden_deck", (4 * i, 0, z), 0, group, F)
        # Outdoor hallways in front of the block.
        if walks:
            for i in range(nx):
                L.put("city_walk_bay", (4 * i, -3, z), 0, group, F)
                if s >= 1:
                    L.put("city_guardrail_4", (4 * i, -2.95, z), 0, group, F)
            L.put("city_walk_end", (4 * nx, -3, z), 0, group, F)
    top = 4.0 * storeys
    y0 = 4.0 if setback_at is not None else 0.0
    j0 = 1 if setback_at is not None else 0
    if walks:
        for i in range(nx):
            L.put("city_walk_roof", (4 * i, -3, top - 4.0), 0, group, F)
    # Roof and parapets.
    for i in range(nx):
        for j in range(j0, ny):
            planted = roof == "planted" and (i + j) % 2 == 0
            L.put("city_roof_planted_4" if planted else "city_roof_cap_4", (4 * i, 4 * j, top), 0, group, F)
    for i in range(nx):
        step = (i == nx // 2)
        L.put("city_parapet_step_4" if step else "city_parapet_4", (4 * i, y0, top), 0, group, F)
        L.put("city_parapet_4", (4 * (i + 1), 4 * ny, top), 180, group, F)
    for j in range(j0, ny):
        L.put("city_parapet_4", (0, 4 * (j + 1), top), -90, group, F)
        L.put("city_parapet_4", (4 * nx, 4 * j, top), 90, group, F)
    for (cx, cy, r) in ((0, y0, 0), (4 * nx, y0, 90), (4 * nx, 4 * ny, 180), (0, 4 * ny, -90)):
        L.put("city_parapet_corner", (cx, cy, top), r, group, F)
    L.mass(f"Block{seed}", (0, 0, 0), (4 * nx, 4 * ny, top + 1.2), F)


def build_layout():
    L = Layout()
    occupied = set()

    def occupy(x0, y0, x1, y1):
        for cx in range(int(math.floor(x0 / 4)), int(math.ceil(x1 / 4))):
            for cy in range(int(math.floor(y0 / 4)), int(math.ceil(y1 / 4))):
                occupied.add((cx, cy))

    # ---------------------------------------------------------- the canal
    for x in range(0, 88, 8):
        L.put("city_canal_water_8", (x, -4, 0))
        L.put("city_canal_bank_8", (x, -4, 0))
    occupy(-4, -8, 92, 8)
    for x in range(0, 72, 8):
        if x == 40:
            L.put("city_canal_steps", (44, 4, 0), 180)
            L.put("city_canal_steps", (48, 4, 0), 180)
        else:
            L.put("city_canal_bank_8", (x + 8, 4, 0), 180)
    L.put("city_canal_bank_4", (76, 4, 0), 180)
    L.put("city_canal_corner_outer", (80, 8, 0), -90)
    L.put("city_canal_corner_inner", (88, -8, 0), 90)
    for y in range(4, 36, 8):
        L.put("city_canal_water_8", (80, y, 0))
    for y in (8, 16, 24):
        L.put("city_canal_bank_8", (80, y + 8, 0), -90)
    L.put("city_canal_bank_4", (80, 36, 0), -90)
    for y in range(-4, 36, 8):
        L.put("city_canal_bank_8", (88, y, 0), 90)
    occupy(76, 8, 92, 40)
    # The two heads: the west where the camp stream arrives, the north under the gallery.
    L.put("city_canal_end", (0, 4, 0), -90)
    L.put("city_canal_corner_inner", (-4, -8, 0), 0)
    L.put("city_canal_corner_inner", (0, 8, 0), -90)
    L.put("city_canal_end", (88, 36, 0), 180)
    L.put("city_canal_corner_inner", (80, 40, 0), -90)
    L.put("city_canal_corner_inner", (92, 36, 0), 180)
    occupy(-4, -8, 0, 8)
    # The old mud stream, kept as a relic on the approach from the camp.
    for ox in (-8, -12, -16, -20):
        L.put("city_mud_relic", (ox, 4, 0), -90)
    occupy(-20, -4, -4, 4)
    L.put("city_sign", (-10, -3.2, 0), 0)
    L.mass("Canal", (-4, -4, -2), (88, 4, -0.9))
    L.mass("CanalNorth", (80, 4, -2), (88, 36, -0.9))
    # The staff follows the canal's centreline.
    for x in range(0, 80, 4):
        L.put("city_staff_4", (x, 0, WATER_Z))
    L.put("city_staff_curve", (80, 0, WATER_Z))
    for y in range(4, 36, 4):
        L.put("city_staff_4", (84, y, WATER_Z), 90)
    L.put("city_water_gate", (80, 33.2, 0), -90)
    # Crossings: two over the east canal, one to the gallery over the north canal.
    for x, kind in ((24, "city_footbridge_8"), (56, "city_footbridge_planted_8")):
        L.put(kind, (x, -4, 0))
        L.put("city_bridge_abutment", (x, -4, 0))
        L.put("city_bridge_abutment", (x + 4, 4, 0), 180)
        L.put("city_glow_tendril", (x + 2, 0, 0.2))
    L.put("city_footbridge_planted_8", (80, 28, 0), -90)
    L.put("city_bridge_abutment", (80, 28, 0), -90)
    L.put("city_bridge_abutment", (88, 24, 0), 90)
    L.put("city_glow_tendril", (84, 26, 0.2), 90)

    # ---------------------------------------------------------- south blocks (face the canal)
    block(L, Frame((20, -12, 0), 180), 4, 4, 4, style="plaster", seed=1)
    block(L, Frame((52, -12, 0), 180), 5, 4, 6, style="mixed", setback_at=4, seed=2)
    block(L, Frame((76, -12, 0), 180), 3, 3, 2, style="plaster", arcade=False, roof="planted", seed=3,
          balconies=False)
    occupy(4, -28, 20, -12)
    occupy(32, -28, 52, -12)
    occupy(64, -24, 76, -12)
    # ---------------------------------------------------------- north of the canal
    block(L, Frame((4, 15, 0), 0), 4, 4, 5, style="glass", arcade=False, walks=True, seed=4)
    occupy(4, 12, 20, 31)
    block(L, Frame((28, 15, 0), 0), 3, 3, 3, style="mixed", arcade=True, seed=5)
    occupy(28, 15, 40, 27)

    # The gallery: faces east across the north canal; its entrance meets the footbridge.
    def gallery_front(i, s):
        if i in (1, 2):
            return "skip"
        if s == 0:
            return "city_gallery_display_wall"
        return "city_facade_window_b"
    gF = Frame((76, 18, 0), 90)
    block(L, gF, 5, 5, 3, style="glass", arcade=False, roof="planted", seed=6, balconies=False,
          front_override=gallery_front)
    L.put("city_gallery_entrance", (4, 0, 0), 0, frame=gF)
    occupy(56, 18, 77, 38)
    for k, (x, y) in enumerate(((66.0, 22.0), (66.0, 30.0))):
        L.put("city_gallery_plinth", (x, y, 0))

    # ---------------------------------------------------------- the civic plaza and hall
    L.put("city_hall_shell", (106, 16, 0), -90)
    L.put("city_hall_stage", (106, 16, 0), -90)
    for k, y in enumerate((4, 12, 20)):
        L.put("city_hall_seating", (100, y, 0), 90)
    L.mass("Hall", (106, 5, 0), (116, 27, 14))
    occupy(106, 4, 118, 28)
    L.put("city_fountain", (100, 38, 0))
    L.put("city_instrument_pedestal", (96, 30, 0), 90)
    L.put("city_instrument_pedestal", (46, 9.5, 0), 180)
    for k, (x, y) in enumerate(((98, -6), (110, -6), (120, 34))):
        L.put("city_solar_canopy", (x, y, 0))
    L.put("city_shade_sail", (112, 38, 0), 15)
    L.put("city_pump_house", (98, 46, 0), 180)
    L.put("city_pipe_4", (93.5, 46, 0), 0)
    for k, y in enumerate((0, 8, 28, 36)):
        L.put("city_planter_4", (120, y, 0), 90)
    # The monument: beyond the plaza, on the canal's axis.
    L.put("city_monument_base", (156, 10, 0))
    L.put("city_monument", (156, 10, 0.9))
    L.mass("Monument", (150, 4, 0), (162, 16, 28))

    # ---------------------------------------------------------- light rail along the north edge
    for x in range(-16, 128, 8):
        L.put("city_rail_track_8", (x, 52, TRACK_Z))
    for x in range(-12, 128, 16):
        L.put("city_rail_pier", (x, 52, 0))
    for x in (24, 32, 40):
        L.put("city_rail_platform_8", (x, 52, TRACK_Z))
        L.put("city_rail_canopy_8", (x, 52, TRACK_Z))
    L.put("city_rail_car", (36, 52, TRACK_Z))
    # The station stair: two switchbacks to the 8 m platform, and a lift beside it.
    L.put("city_stair_flight", (40, 56.75, 0))
    L.put("city_stair_landing", (44, 56.75, 0))
    L.put("city_stair_flight", (44, 60.75, 2), 180)
    L.put("city_stair_landing", (38, 56.75, 2))
    L.put("city_stair_flight", (40, 56.75, 4))
    L.put("city_stair_landing", (44, 56.75, 4))
    L.put("city_stair_flight", (44, 60.75, 6), 180)
    L.put("city_stair_landing", (38, 56.75, 6))
    L.put("city_lift", (49.5, 58.8, 0))
    L.put("city_lift", (49.5, 58.8, 4))
    L.mass("Rail", (-16, 50.4, 6.75), (128, 53.6, 8.0))
    occupy(36, 56, 52, 62)

    # ---------------------------------------------------------- streets, promenades and furniture
    for x in range(0, 88, 8):
        L.put("city_bollard", (x + 2, -7.4, 0))
        if x < 76:
            L.put("city_bollard", (x + 6, 7.4, 0))
    for x in (8, 40, 68):
        L.put("city_bench", (x, -10.5, 0), 180)
        L.put("city_bench", (x + 4, 10.5, 0))
    L.put("city_sign", (22, -9.5, 0), 180)
    for x in (24, 56):
        L.put("city_planter_4", (x, -26, 0), 90)
        L.put("city_planter_4", (x + 5, -26, 0), 90)
    for k, (x, y) in enumerate(((26, -18), (58, -18), (22, 34), (46, 30), (90, 50), (-6, -14), (-8, 14))):
        L.put("city_tree", (x, y, 0))
    for k, (x, y) in enumerate(((47, 9.0), (102, 34), (8, 9.6), (98, 12))):
        L.put("city_glow_motes", (x, y, 0))
    for k, (x, y) in enumerate(((12, -9.6), (36, -9.6), (64, -9.6), (16, 9.8), (60, 9.8), (93, 24), (93, 8))):
        L.put("city_glow_bell", (x, y, 0))
    for k, (x, y) in enumerate(((119, 2), (119, 10), (119, 30))):
        L.put("city_glow_vein_leaf", (x, y, 0.52), 180)
    L.put("city_pergola", (116, -8, 0))
    L.put("city_pergola", (120, 40, 0), 90)

    residential(L, occupy)

    # Keep the plaza's features clear of the greening below.
    occupy(96, 4, 100, 28)       # seating
    occupy(96, 34, 104, 42)      # fountain
    occupy(104, 32, 120, 44)     # shade sail, pergola
    occupy(92, -12, 100, -4)     # pergola, solar tree
    occupy(92, 44, 100, 48)      # pump house
    occupy(116, -4, 124, 40)     # planters and vein leaves
    occupy(-20, 50, 128, 62)     # rail piers, station stair
    keep_clear = set(occupied)

    # ---------------------------------------------------------- paving everything else
    X0, X1, Y0, Y1 = -20, 124, -36, 64
    done = set()
    for cx in range(X0 // 4, X1 // 4):
        for cy in range(Y0 // 4, Y1 // 4):
            if (cx, cy) in occupied or (cx, cy) in done:
                continue
            quad = {(cx, cy), (cx + 1, cy), (cx, cy + 1), (cx + 1, cy + 1)}
            if cx % 2 == 0 and cy % 2 == 0 and cx + 1 < X1 // 4 and cy + 1 < Y1 // 4 and not (quad & (occupied | done)):
                L.put("city_paving_8", (cx * 4, cy * 4, 0))
                done |= quad
                # Every suitable surface supports life: some squares become planted.
                promenade = -12 <= cy * 4 < -4 or 4 <= cy * 4 < 12
                r = L.rng.random()
                if not promenade and r < 0.22:
                    L.put("city_planter_4", (cx * 4 + 2, cy * 4 + 3.5, 0))
                    L.put("city_bench", (cx * 4 + 4, cy * 4 + 2.6, 0))
                elif not promenade and r < 0.42:
                    L.put("city_tree", (cx * 4 + 4, cy * 4 + 4, 0), L.rng.uniform(0, 360))
                elif r < 0.5:
                    L.put("city_glow_bell", (cx * 4 + 4, cy * 4 + 4, 0), L.rng.uniform(0, 360))
            else:
                L.put("city_paving_4", (cx * 4, cy * 4, 0))
                done.add((cx, cy))

    # ---------------------------------------------------------- Dancers
    spots = [((101, 30), 0, "Dance"), ((103, 31), 40, "Dance"), ((100, 33), 200, "Dance"), ((104, 28), 120, "Dance"),
             ((47.5, 2.5), 180, "Sit"), ((45.5, 2.5), 180, "Sit"), ((26, -1.0), 90, "Walk"), ((58, 2.0), 270, "Walk"),
             ((30, -9.0), 0, "Talk"), ((31, -9.2), 180, "Talk"), ((84, 26), 90, "Walk"), ((96, 18), 90, "Idle"),
             ((97, 14), 80, "Idle"), ((97, 22), 100, "Idle"), ((-10, 0.0), 90, "Walk"), ((70, 10), 0, "Idle"),
             ((34, 55), 0, "Idle"), ((112, 40), 220, "Talk"), ((12, 11), 180, "Idle"), ((66, 26), 270, "Idle")]
    tones = list(SKIN)
    for k, ((x, y), r, clip) in enumerate(spots):
        z = TRACK_Z + 0.35 if y > 53 else (-0.75 if clip == "Sit" else 0.0)
        L.dancers.append((SHAPES[k % 4], tones[(k // 4 + k) % 4], (x, y, z), r, clip))

    # ---------------------------------------------------------- planting at sockets
    dress(L)
    return L


def residential(L, occupy):
    """The mixed-use neighbourhood (docs/art/city-residential.md): shops on the street,
    offices above, homes above those, shared roof gardens. Four building types from one
    kit, on ground the first district left as open paving."""
    g = "Residential"
    # 1. The street wall on the canal: north bank, between Block E and the gallery.
    F = Frame((40, 13, 0), 0)
    demo = [(3, 0, 2), (4, 2, 1), (5, 0, 3)]
    mixed_block(L, F, 4, 3, ["shop", "office", "office", "home", "home", "home", "home", "home"],
                seed=21, group=g, lobby=3, core=("back", 1), demo=demo)
    for s, i0, n in demo:
        furnished_home(L, F, s, i0, n, 3, group=g, seed=s)
    occupy(40, 13, 56, 29)
    # 2. The rounded-corner block facing the plaza and the canal's bend, a courtyard behind.
    F = Frame((100, -12, 0), 180)
    mixed_block(L, F, 5, 4, ["shop", "office", "home", "home", "home", "home", "home", "home"],
                seed=22, group=g, corners="round", lobby=2, core="east")
    occupy(76, -28, 100, -12)
    for k, (x, y) in enumerate(((84, -32), (92, -32))):
        L.put("city_raised_beds", (x, y, 0), 0, g)
    L.put("city_communal_table", (88, -29.5, 0), 0, g)
    for x in (80.5, 97.5):
        L.put("city_tree", (x, -32.5, 0), x * 7, g)
    L.put("city_pergola", (78, -36, 0), 0, g)
    # 3. The stepped garden tower and 4. the taller tower, on a new inland street (y 31-36)
    # with a shared courtyard between them.
    F = Frame((4, 36, 0), 0)
    mixed_block(L, F, 3, 3, ["shop", "office"] + ["home"] * 8, seed=23, group=g, lobby=0,
                setbacks={4: 1, 7: 2}, core="east")
    F = Frame((24, 36, 0), 0)
    mixed_block(L, F, 3, 3, ["shop", "office", "office"] + ["home"] * 9, seed=24, group=g, lobby=2,
                core="west", balcony_rate=0.7)
    occupy(4, 36, 36, 48)
    for x, y in ((18, 38.5), (21.5, 38.5)):
        L.put("city_raised_beds", (x, y, 0), 90, g)
    L.put("city_communal_table", (20, 41.5, 0), 0, g)
    for x in (2, 14, 30, 38):
        L.put("city_tree", (x, 33.6, 0), x * 13, g)
    for x in (8, 26, 34):
        L.put("city_bench", (x, 32.2, 0), 0, g)
    for x in (11, 37):
        L.put("city_glow_bell", (x, 33.4, 0), x, g)
    L.put("city_sign", (1.5, 33.5, 0), 90, g)
    # A distant skyline of ordinary towers (cheap far storeys), beyond the district.
    rng = random.Random(9)
    for x, y in ((-70, 130), (10, 160), (95, 140), (180, 95), (215, -30), (170, -125), (60, -150),
                 (-40, -140), (-130, 30), (-120, -60)):
        for s in range(rng.randint(10, 20)):
            L.put("city_far_tower_storey", (x, y, 4.0 * s), 0, "Skyline")
        L.put("city_roof_planted_4", (x - 2, y - 2, 4.0 * (s + 1)), 0, "Skyline")


ACCEPT = {"small": ("city_grass", "city_agave", "city_grass", "city_shrub"),
          "shrub": ("city_shrub", "city_agave", "city_shrub"),
          "tree": ("city_tree", "city_shrub"),
          "vine": ("city_vine_strip",),
          "reeds": ("city_grass",)}
CHANCE = {"small": 0.8, "shrub": 0.9, "tree": 0.75, "vine": 0.45, "reeds": 0.7}


def dress(L):
    """Plant what each placed piece's sockets accept (deterministic)."""
    rng = random.Random(11)
    for item in list(L.items):
        for name, s in MANIFEST[item["id"]].get("sockets", {}).items():
            acc = s.get("accepts")
            if acc not in ACCEPT or rng.random() > CHANCE[acc]:
                continue
            choice = rng.choice(ACCEPT[acc])
            F = Frame(item["pos"], item["rot"])
            pos, srot = list(s["position"]), math.degrees(s.get("rotation_z", 0.0))
            if choice == "city_vine_strip":
                a = math.radians(srot)
                pos = [pos[0] - 2 * math.cos(a), pos[1] - 2 * math.sin(a), pos[2]]
            else:
                srot += rng.uniform(0, 360)
            L.put(choice, pos, srot, "Planting", F)


def test_strip(L):
    """Straight joins, corners, stacked floors, bridge connections: one of each, in a row."""
    F0 = (0, -100, 0)
    g = "Test-Strip"
    block(L, Frame((F0[0] + 0, F0[1], 0), 0), 2, 2, 3, style="mixed", arcade=True, seed=9, group=g)
    # Stacked walks with rails, end and roof.
    Fw = Frame((F0[0] + 12, F0[1], 0), 0)
    for s in range(2):
        for i in range(2):
            L.put("city_walk_bay", (4 * i, 0, 4 * s), 0, g, Fw)
            L.put("city_facade_window" if i else "city_facade_door", (4 * i, 3, 4 * s), 0, g, Fw)
            if s:
                L.put("city_guardrail_4", (4 * i, 0.05, 4 * s), 0, g, Fw)
        L.put("city_walk_end", (8, 0, 4 * s), 0, g, Fw)
    for i in range(2):
        L.put("city_walk_roof", (4 * i, 0, 4), 0, g, Fw)
    # A canal stretch with both banks, a bridge, steps and the staff.
    Fc = Frame((F0[0] + 28, F0[1] + 4, 0), 0)
    for x in (0, 8):
        L.put("city_canal_water_8", (x, 0, 0), 0, g, Fc)
        L.put("city_canal_bank_8", (x, 0, 0), 0, g, Fc)
    L.put("city_canal_bank_8", (8, 8, 0), 180, g, Fc)
    L.put("city_canal_steps", (12, 8, 0), 180, g, Fc)
    L.put("city_canal_steps", (16, 8, 0), 180, g, Fc)
    L.put("city_footbridge_8", (4, 0, 0), 0, g, Fc)
    L.put("city_bridge_abutment", (4, 0, 0), 0, g, Fc)
    L.put("city_bridge_abutment", (8, 8, 0), 180, g, Fc)
    for x in range(0, 16, 4):
        L.put("city_staff_4", (x, 4, WATER_Z), 0, g, Fc)
    # Stairs to a 4 m landing, a ramp, kerbs and plaza steps.
    Fs = Frame((F0[0] + 50, F0[1], 0), 0)
    L.put("city_stair_flight", (0, 0, 0), 0, g, Fs)
    L.put("city_stair_landing", (4, 0, 0), 0, g, Fs)
    L.put("city_stair_flight", (4, 4, 2), 180, g, Fs)
    L.put("city_stair_landing", (-2, 0, 0), 0, g, Fs)
    L.put("city_stair_landing", (-2, 0, 2), 0, g, Fs)
    L.put("city_ramp_8", (8, 0, 0), 0, g, Fs)
    for x in (0, 4, 8):
        L.put("city_curb_4", (x, -4, 0), 0, g, Fs)
        L.put("city_paving_4", (x, -4, 0), 0, g, Fs)
    L.put("city_curb_corner", (0, -4, 0), 0, g, Fs)
    L.put("city_plaza_steps_4", (12, -4, -1), 0, g, Fs)
    # Mixed use: every ground and upper bay type in one row (interchangeable on the same grid),
    # then a small block with a pier corner and a rounded corner, stacked shop/office/homes.
    Fm = Frame((F0[0] + 70, F0[1] - 6, 0), 0)
    row = ("city_shop_glazed", "city_shop_cafe", "city_res_entry", "city_office_glazed", "city_office_service",
           "city_res_slider", "city_res_slider_b", "city_res_loggia", "city_res_window_pair",
           "city_res_window_pair_b", "city_res_privacy", "city_res_door")
    for i, aid in enumerate(row):
        L.put("city_slab_4", (4 * i, 0, 0), 0, g, Fm)
        L.put(aid, (4 * i, 0, 0), 0, g, Fm)
    L.put("city_facade_end", (0, 0, 0), 0, g, Fm)
    L.put("city_facade_end", (4 * len(row), 0, 0), 0, g, Fm)
    mixed_block(L, Frame((F0[0] + 70, F0[1] + 4, 0), 0), 3, 2, ["shop", "office", "home", "home"],
                seed=5, group=g, corners="round", lobby=1, core="east")


# ====================================================================== Blender scene


def link_kit(ids):
    found = {}
    for aid in sorted(ids):
        path = SOURCES / f"{aid}.blend"
        with bpy.data.libraries.load(str(path), link=True, relative=True) as (src, dst):
            dst.collections = [aid]
        found[aid] = dst.collections[0]
    return found


def instance(aid, coll, pos, rot, parent_coll, name=None):
    e = bpy.data.objects.new(name or aid, None)
    e.instance_type = "COLLECTION"
    e.instance_collection = coll
    e.location = pos
    e.rotation_euler = (0, 0, math.radians(rot))
    e.empty_display_size = 0.5
    parent_coll.objects.link(e)
    return e


def dancer_collections(root):
    """Each Dancer shape imported once; four collections per shape share its armature and
    mesh, and differ only in an object-level skin material (16 instanceable Dancers)."""
    out = {}
    holder = bpy.data.collections.new("Dancer-Sources")
    root.children.link(holder)
    mats = {}
    for tone, hexc in SKIN.items():
        mat = bpy.data.materials.new(f"Dancer_Skin_{tone}")
        mat.use_nodes = True
        mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (*srgb(hexc), 1)
        mat.node_tree.nodes["Principled BSDF"].inputs["Roughness"].default_value = 0.8
        mats[tone] = mat
    for shape in SHAPES:
        path = ROOT / f"assets/characters/dancers/dancer_{shape}/dancer_{shape}.glb"
        before = set(bpy.data.objects)
        bpy.ops.import_scene.gltf(filepath=str(path))
        new = [o for o in bpy.data.objects if o not in before]
        for o in [o for o in new if o.name.startswith("Icosphere")]:   # the importer's bone shape
            new.remove(o)
            bpy.data.objects.remove(o, do_unlink=True)
        for o in new:
            for c in list(o.users_collection):
                c.objects.unlink(o)
        body = next(o for o in new if o.type == "MESH")
        rig = [o for o in new if o is not body]
        for k, (tone, mat) in enumerate(mats.items()):
            coll = bpy.data.collections.new(f"Dancer-{shape}-{tone}")
            holder.children.link(coll)
            for o in rig:
                coll.objects.link(o)
            b = body if k == 0 else body.copy()   # shares the mesh and the armature
            coll.objects.link(b)
            for slot in b.material_slots:
                slot.link = "OBJECT"
                slot.material = mat
            out[(shape, tone)] = coll
    holder.hide_render = True    # sources for the Dancer instances, not shown themselves
    holder.hide_viewport = True
    return out


def world_gradient(name, horizon, zenith, strength):
    w = bpy.data.worlds.new(name)
    w.use_nodes = True
    nt = w.node_tree
    nt.nodes.clear()
    tc = nt.nodes.new("ShaderNodeTexCoord")
    sep = nt.nodes.new("ShaderNodeSeparateXYZ")
    ramp = nt.nodes.new("ShaderNodeValToRGB")
    bg = nt.nodes.new("ShaderNodeBackground")
    out = nt.nodes.new("ShaderNodeOutputWorld")
    nt.links.new(tc.outputs["Generated"], sep.inputs[0])
    nt.links.new(sep.outputs["Z"], ramp.inputs["Fac"])
    nt.links.new(ramp.outputs["Color"], bg.inputs["Color"])
    nt.links.new(bg.outputs["Background"], out.inputs["Surface"])
    ramp.color_ramp.elements[0].position = 0.0
    ramp.color_ramp.elements[0].color = (*srgb(horizon), 1)
    ramp.color_ramp.elements[1].position = 0.55
    ramp.color_ramp.elements[1].color = (*srgb(zenith), 1)
    bg.inputs["Strength"].default_value = strength
    return w


LIGHTING = {
    # name: (horizon, zenith, world strength, sun colour, sun energy, sun elevation, sun azimuth)
    "Day": ("#bfe3d8", "#2f9fae", 1.0, "#fff3e3", 4.5, 58, 140),
    "Dusk": ("#ff8f73", "#4d388f", 0.9, "#ff9c6e", 3.2, 7, 255),
    "Night": ("#2a1440", "#421a5c", 0.8, "#9b8be0", 0.6, 42, 200),
}


def make_scenes(collections):
    scenes = {}
    base = bpy.context.scene
    for k, (name, spec) in enumerate(LIGHTING.items()):
        sc = base if k == 0 else bpy.data.scenes.new(name)
        sc.name = name
        for c in collections:
            if c.name not in sc.collection.children:
                sc.collection.children.link(c)
        horizon, zenith, ws, sun_c, sun_e, elev, az = spec
        sc.world = world_gradient(f"World-{name}", horizon, zenith, ws)
        sun = bpy.data.objects.new(f"Sun-{name}", bpy.data.lights.new(f"Sun-{name}", "SUN"))
        sun.data.color = srgb(sun_c)
        sun.data.energy = sun_e
        sun.data.angle = math.radians(1.5)
        sun.rotation_euler = Euler((math.radians(90 - elev), 0, math.radians(az)), "XYZ")
        sc.collection.objects.link(sun)
        sc.render.engine = "BLENDER_EEVEE"
        sc.render.resolution_x, sc.render.resolution_y = 1280, 720
        sc.view_settings.view_transform = "AgX"
        sc.view_settings.look = "AgX - Medium High Contrast" if name != "Night" else "AgX - Base Contrast"
        try:
            sc.eevee.use_shadows = True
            sc.eevee.shadow_ray_count = 1
        except AttributeError:
            pass
        scenes[name] = sc
    return scenes


CAMERAS = {
    # name: (location, aim, lens)
    "Approach-Reveal": ((-19.0, -1.4, 1.65), (60.0, 4.0, 6.0), 30),
    "Canal-Eye": ((14.0, -6.6, 1.65), (60.0, 2.0, 4.0), 28),
    "Civic-Plaza": ((121.0, -9.0, 1.65), (98.0, 24.0, 6.0), 26),
    "Rooftop-Overview": ((125.0, -85.0, 62.0), (45.0, 8.0, 0.0), 30),
    "Night-View": ((49.0, 6.2, 1.65), (90.0, 14.0, 5.0), 28),
    "Test-Strip": ((22.0, -136.0, 28.0), (27.0, -95.0, 1.0), 28),
    "Res-Street-Up": ((9.0, 31.4, 1.65), (11.0, 48.0, 13.0), 16),
    "Res-Canal-Stack": ((47.0, -10.5, 1.65), (48.0, 16.0, 15.0), 18),
    "Res-Balcony": ((25.0, 34.6, 25.6), (10.0, 38.0, 21.0), 24),
    "Res-Overview": ((-34.0, -30.0, 62.0), (44.0, 20.0, 8.0), 28),
    "Res-Corner": ((104.0, 2.0, 1.65), (90.0, -14.0, 12.0), 22),
    "Res-Test-Strip": ((86.0, -130.0, 14.0), (100.0, -100.0, 5.0), 28),
}


def make_cameras(scenes):
    cams = {}
    for name, (loc, aim, lens) in CAMERAS.items():
        cam = bpy.data.objects.new(name, bpy.data.cameras.new(name))
        cam.location = loc
        cam.rotation_euler = (Vector(aim) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
        cam.data.lens = lens
        cam.data.clip_end = 1500
        cam.data.sensor_width = 36
        for sc in scenes.values():
            sc.collection.objects.link(cam)
        cams[name] = cam
    return cams


def context_ground(root):
    """A bloomed ground plane round the district and the camp's mudflat to the west
    (context only, not part of the kit)."""
    coll = bpy.data.collections.new("Context")
    root.children.link(coll)
    import bmesh

    def ground(name, outer, holes, z, hexc):
        """A flat context plane with rectangular holes (triangle-filled)."""
        bm = bmesh.new()
        def loop(x0, y0, x1, y1):
            vs = [bm.verts.new(c) for c in ((x0, y0, z), (x1, y0, z), (x1, y1, z), (x0, y1, z))]
            return [bm.edges.new((vs[i], vs[(i + 1) % 4])) for i in range(4)]
        edges = loop(*outer)
        for h in holes:
            edges += loop(*h)
        bmesh.ops.triangle_fill(bm, use_beauty=True, use_dissolve=True, edges=edges, normal=(0, 0, 1))
        me = bpy.data.meshes.new(name)
        bm.to_mesh(me)
        bm.free()
        m = bpy.data.materials.new(name)
        m.use_nodes = True
        m.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (*srgb(hexc), 1)
        m.node_tree.nodes["Principled BSDF"].inputs["Roughness"].default_value = 1.0
        me.materials.append(m)
        coll.objects.link(bpy.data.objects.new(name, me))
    # The district and the test strip's canal sit below their own paving: the context ground
    # surrounds them and never covers them. The camp's mudflat lies west of the approach.
    district, strip_canal, camp = (-20, -36, 124, 64), (28, -100, 44, -84), (-380, -60, -20, 60)
    ground("Ground-Bloomed", (-400, -400, 600, 400), [district, strip_canal, camp], -0.32, "#29361c")
    ground("Ground-Camp-Mudflat", camp, [], -0.32, "#5a4433")
    return coll


def blockout(L, root):
    coll = bpy.data.collections.new("Blockout")
    root.children.link(coll)
    m = bpy.data.materials.new("Blockout")
    m.diffuse_color = (0.8, 0.8, 0.8, 1)
    for name, mn, mx, _ in L.masses:
        me = bpy.data.meshes.new(name)
        x0, y0, z0 = mn
        x1, y1, z1 = mx
        v = [(x0, y0, z0), (x1, y0, z0), (x1, y1, z0), (x0, y1, z0), (x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)]
        f = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4), (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)]
        me.from_pydata(v, [], f)
        me.materials.append(m)
        coll.objects.link(bpy.data.objects.new(f"Mass-{name}", me))
    return coll


def glare(scene):
    """Bloom for the luminous flora (Eevee has none of its own since 4.2): a compositor
    glare. Best effort: the compositor's Python API changed in 5.x."""
    try:
        tree = bpy.data.node_groups.new(f"Glow-{scene.name}", "CompositorNodeTree")
        tree.interface.new_socket("Image", in_out="OUTPUT", socket_type="NodeSocketColor")
        rl = tree.nodes.new("CompositorNodeRLayers")
        rl.scene = scene
        g = tree.nodes.new("CompositorNodeGlare")
        out = tree.nodes.new("NodeGroupOutput")
        for key, val in (("glare_type", "BLOOM"), ("quality", "MEDIUM")):
            if hasattr(g, key):
                setattr(g, key, val)
        if "Type" in g.inputs:
            g.inputs["Type"].default_value = "Bloom"
        for key, val in (("Threshold", 1.0), ("Strength", 0.6), ("Size", 0.5)):
            if key in g.inputs:
                g.inputs[key].default_value = val
        tree.links.new(rl.outputs["Image"], g.inputs["Image"])
        tree.links.new(g.outputs["Image"], out.inputs[0])
        scene.compositing_node_group = tree
        return True
    except Exception as exc:   # noqa: BLE001 (preview nicety only)
        print("glare: skipped:", exc)
        return False


def write_bsn(L, name="city", lift=JACKDAW_LIFT, header=None):
    """The layout as a Jackdaw review scene, every piece an IsA of its prefab."""
    lines = (header or ["// Asset review: the City kit's reference district (create_city_district.py). Not a level.",
                        f"// Lifted {lift} m so the canal's bed sits on the ground; the canal is below the",
                        "// promenade, and the game's ground has no cut for it yet."]) + ["bevy_ecs::hierarchy::Children ["]
    entries = []
    for k, it in enumerate(L.items):
        if it["group"] == "Test-Strip":
            continue
        x, y, z = it["pos"]
        a = math.radians(it["rot"])
        entries.append([
            f'    #"{it["id"]} {k}"',
            f'    jackdaw::prefab::components::IsA {{ source: "structures/city/{it["id"]}/{it["id"]}.bsn", deleted: [] }}',
            "    jackdaw::prefab::components::PrefabEntityId(0)",
            f"    bevy_transform::components::transform::Transform {{ translation: glam::Vec3 {{ x: {x:.6f}, y: {z + lift:.6f}, z: {-y:.6f} }}, "
            f"rotation: glam::Quat {{ x: 0.000000, y: {math.sin(a / 2):.6f}, z: 0.000000, w: {math.cos(a / 2):.6f} }} }}",
        ])
    for k, (shape, tone, (x, y, z), r, clip) in enumerate(L.dancers):
        a = math.radians(r)
        rgb = ", ".join(f"{int(SKIN[tone][i:i + 2], 16) / 255:.4f}" for i in (1, 3, 5))
        entries.append([
            f'    #"Dancer {shape} {tone} {k}"',
            f'    jackdaw::prefab::components::IsA {{ source: "characters/dancers/dancer_{shape}/dancer_{shape}.bsn", deleted: [] }}',
            "    jackdaw::prefab::components::PrefabEntityId(0)",
            f"    bevy_transform::components::transform::Transform {{ translation: glam::Vec3 {{ x: {x:.6f}, y: {z + lift:.6f}, z: {-y:.6f} }}, "
            f"rotation: glam::Quat {{ x: 0.000000, y: {math.sin(a / 2):.6f}, z: 0.000000, w: {math.cos(a / 2):.6f} }} }}",
            f'    stubby_world::prefab_presentation::MaterialTint {{ material: "Dancer_Skin", srgb: [{rgb}] }}',
        ])
    body = []
    for i, e in enumerate(entries):
        body += e
        if i < len(entries) - 1:
            body.append("    --")
    (ROOT / f"assets/world/asset_reviews/{name}.bsn").write_text("\n".join(lines + body + ["]"]) + "\n")


def main():
    args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    L = build_layout()
    test_strip(L)
    (ROOT / "docs/art/city-kit").mkdir(parents=True, exist_ok=True)
    counts = {}
    for it in L.items:
        counts[it["id"]] = counts.get(it["id"], 0) + 1
    (ROOT / "docs/art/city-kit/district_layout.json").write_text(json.dumps(
        {"about": "Placements of the reference district and test strip (Blender coordinates, Z up, "
                  "rotation about Z in degrees). Generated by create_city_district.py.",
         "instances": len(L.items), "unique_pieces": len(counts), "pieces_used": dict(sorted(counts.items())),
         "dancers": [{"shape": s, "tone": t, "pos": p, "rot": r, "clip": c} for s, t, p, r, c in L.dancers],
         "placements": L.items}, indent=0) + "\n")
    write_bsn(L)

    # Fresh file: drop the startup objects.
    for o in list(bpy.data.objects):
        bpy.data.objects.remove(o, do_unlink=True)
    scene = bpy.context.scene
    root = scene.collection
    kit = link_kit({it["id"] for it in L.items})
    groups = {}
    for g in ("District", "Planting", "Test-Strip", "Residential", "Skyline"):
        c = bpy.data.collections.new(g)
        root.children.link(c)
        groups[g] = c
    for it in L.items:
        instance(it["id"], kit[it["id"]], it["pos"], it["rot"], groups[it["group"]])
    dancers = dancer_collections(root)
    dcoll = bpy.data.collections.new("Dancers")
    root.children.link(dcoll)
    for shape, tone, pos, r, clip in L.dancers:
        instance(f"Dancer-{shape}-{tone}", dancers[(shape, tone)], pos, r, dcoll)
    ctx = context_ground(root)
    blk = blockout(L, root)
    shared = [groups[g] for g in ("District", "Planting", "Test-Strip", "Residential", "Skyline")] + [dcoll, ctx, blk]
    for c in list(root.children):
        if c in shared:
            root.children.unlink(c)
    scenes = make_scenes(shared)
    blk.hide_render = True     # the first pass, kept for reference: show it in the outliner
    blk.hide_viewport = True
    cams = make_cameras(scenes)
    out = SOURCES / "city_district.blend"
    if bpy.context.window:
        bpy.context.window.scene = scenes["Day"]
    bpy.context.preferences.filepaths.save_version = 0   # no .blend1 beside the source
    bpy.ops.wm.save_as_mainfile(filepath=str(out), check_existing=False, relative_remap=True)
    print("saved", out, "instances", len(L.items), "unique", len(counts))
    if "norender" in args:
        return
    PREVIEWS.mkdir(parents=True, exist_ok=True)
    plan = [("Approach-Reveal", "Day"), ("Approach-Reveal", "Dusk"), ("Canal-Eye", "Day"), ("Canal-Eye", "Dusk"),
            ("Civic-Plaza", "Day"), ("Civic-Plaza", "Night"), ("Rooftop-Overview", "Day"),
            ("Night-View", "Night"), ("Test-Strip", "Day"),
            ("Res-Canal-Stack", "Day"), ("Res-Canal-Stack", "Dusk"), ("Res-Canal-Stack", "Night"),
            ("Res-Street-Up", "Day"), ("Res-Street-Up", "Dusk"), ("Res-Balcony", "Day"), ("Res-Balcony", "Night"),
            ("Res-Overview", "Day"), ("Res-Overview", "Dusk"), ("Res-Corner", "Day"), ("Res-Test-Strip", "Day")]
    only = [a for a in args if a != "norender"]
    for cam, light in plan:
        if only and cam not in only:
            continue
        sc = scenes[light]
        if light == "Night":
            glare(sc)
        sc.camera = cams[cam]
        sc.render.image_settings.file_format = "JPEG"
        sc.render.image_settings.quality = 90
        sc.render.filepath = str(PREVIEWS / f"district_{cam.lower()}_{light.lower()}.jpg")
        bpy.ops.render.render(write_still=True, scene=sc.name)
        print("rendered", cam, light, flush=True)


if __name__ == "__main__":
    main()
