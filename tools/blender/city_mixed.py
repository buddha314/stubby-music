"""Mixed-use buildings assembled from the City kit: shops on the street, offices above, homes
above those, a shared roof garden on top, and the residents' own way in and up.

One function, `mixed_block`, makes every building type: a street wall (pier corners), a
rounded corner block, a stepped garden tower (setbacks) and a taller tower differ only in
their arguments. No Blender needed: it writes placements into a `Layout`.

Uses: "shop", "office", "home". Each home storey is individualised from a seed: which bays
have balconies, sliding doors, loggias or privacy walls; curtains; and what is on each
balcony (chairs, laundry, plants, shade cloth, screens between neighbours).
"""
import math
import random

from city_layout import Frame

BAY = 4.0
# Fronts are where people live outdoors: mostly balcony doors and loggias.
HOME_FRONT = ("city_res_slider", "city_res_loggia", "city_res_slider_b", "city_res_slider",
              "city_res_window_pair", "city_res_slider_b", "city_res_loggia", "city_res_window_pair_b",
              "city_res_slider", "city_res_privacy")
HOME_BACK = ("city_res_window_pair", "city_res_privacy", "city_res_window_pair_b", "city_res_door")
HOME_SIDE = ("city_res_window_pair", "city_res_privacy", "city_res_window_pair_b")
PROXY_HOME = ("city_proxy_home_living", "city_proxy_home_bedroom", "city_proxy_home_kitchen")
CURTAINS = ("city_curtain_linen", "city_curtain_coral", "city_curtain_teal", "city_curtain_linen")
AWNINGS = ("city_awning_coral", "city_awning_teal", "city_awning_ochre")
CURTAIN_X = {"city_res_window_pair": (1.1, 2.9), "city_res_window_pair_b": (1.1, 2.9),
             "city_res_slider": (1.5,), "city_res_slider_b": (1.5,), "city_res_privacy": ()}


def mixed_block(L, F, nx, ny, stack, *, seed=0, group="Residential", corners="pier", lobby=0,
                setbacks=(), core="east", demo=(), roof="garden", balcony_rate=0.6):
    """A mixed-use building in frame F: footprint [0, 4nx] x [0, 4ny], front along y 0
    facing -Y. `stack` lists each storey's use, bottom up. `setbacks` maps a storey index to
    how many rows the front steps back from that storey up (a garden terrace forms on the
    step). `demo` lists (storey, first bay, bays) for fully furnished homes; elsewhere homes
    and offices show interior proxies. Returns the roof height."""
    rng = random.Random(seed)
    put = lambda aid, pos, rot=0.0: L.put(aid, pos, rot, group, F)  # noqa: E731
    setback = dict(setbacks)
    rows = 0
    round_corners = corners == "round"
    demo_cells = {}
    for s, i0, n in demo:
        for i in range(i0, i0 + n):
            demo_cells[(s, i)] = (i0, n)
    for s, use in enumerate(stack):
        z = BAY * s
        rows = max([rows] + [r for k, r in setback.items() if s >= k])
        y0 = BAY * rows
        depth = ny - rows
        first, last = (1, nx - 1) if round_corners else (0, nx)
        # ---- floor slabs
        for i in range(nx):
            for j in range(rows, ny):
                rc = round_corners and j == rows and i in (0, nx - 1)
                if rc:
                    put("city_slab_round", (0, y0, z), 0) if i == 0 else put("city_slab_round", (4 * nx, y0, z), 90)
                else:
                    put("city_slab_4", (4 * i, 4 * j, z))
        # ---- the terrace where the front steps back
        if s > 0 and rows > 0 and s in setback:
            prev = max([0] + [r for k, r in setback.items() if k < s])
            for i in range(nx):
                for j in range(prev, rows):
                    put("city_garden_deck" if j == prev else "city_roof_planted_4", (4 * i, 4 * j, z))
                    if j == prev and rng.random() < 0.5:
                        put("city_communal_table" if i % 2 else "city_raised_beds", (4 * i + 2, 4 * j + 2.6, z), 90)
        # ---- front
        for i in range(first, last):
            x = 4 * i
            if use == "shop":
                if i == lobby:
                    put("city_res_entry", (x, y0, z))
                    put("city_mailboxes", (x + 2.0, y0 + 3.6, z))
                    continue
                kind = "city_shop_cafe" if i % 3 == 1 else "city_shop_glazed"
                put(kind, (x, y0, z))
                put(AWNINGS[(i + seed) % 3], (x, y0, z))
                put("city_shop_display" if i % 2 == 0 else "city_proxy_shop", (x, y0, z))
                if kind == "city_shop_cafe":
                    for k, dx in enumerate((1.0, 3.0)):
                        put("city_bistro_set", (x + dx, y0 - 2.2, z), rng.uniform(0, 90))
            elif use == "office":
                put("city_office_glazed", (x, y0, z))
                put("city_sunshade_4", (x, y0, z))
                if (s, i) in demo_cells or rng.random() < 0.35:
                    put("city_workstation", (x + 1.1, y0 + 1.6, z), 180)
                    put("city_workstation", (x + 2.9, y0 + 1.6, z), 180)
                    put("city_pendant_lamp", (x + 2.0, y0 + 2.2, z + 3.7))
                    if (s, i) in demo_cells:
                        put("city_meeting_table", (x + 2.0, y0 + 5.5, z))
                else:
                    put("city_proxy_office", (x, y0, z))
            else:   # home
                home_bay(L, put, rng, x, y0, z, s, i, demo_cells, balcony_rate, nx, depth)
        # ---- rounded corners
        if round_corners:
            for i, cx, rot in ((0, 0, 0), (nx - 1, 4 * nx, 90)):
                kind = {"shop": "city_shop_corner_round", "office": "city_office_corner_round",
                        "home": "city_res_corner_round"}[use]
                put(kind, (cx, y0, z), rot)
                if use == "home" and rng.random() < 0.85:
                    put("city_balcony_round", (cx, y0, z), rot)
                    if rng.random() < 0.6:   # the curved balcony's furniture socket, in the corner's frame
                        off = 4 + 4.8 * math.cos(math.radians(225))
                        put("city_balcony_chairs", (cx + off if i == 0 else cx - off, y0 + off, z), rng.uniform(0, 360))
        # ---- back, turned 180
        for i in range(nx):
            x = 4 * (i + 1)
            if isinstance(core, tuple) and i == core[1]:   # the door to the stair tower behind
                put("city_res_door" if use == "home" else "city_facade_door", (x, 4 * ny, z), 180)
                continue
            if use == "shop":
                put("city_facade_door" if i == lobby else "city_office_service", (x, 4 * ny, z), 180)
            elif use == "office":
                put("city_office_glazed" if i % 2 else "city_office_service", (x, 4 * ny, z), 180)
                if i % 2:
                    put("city_proxy_office", (x, 4 * ny, z), 180)
            else:
                kind = HOME_BACK[(i * 3 + s + seed) % len(HOME_BACK)]
                put(kind, (x, 4 * ny, z), 180)
                put(PROXY_HOME[(i + s * 2 + seed) % 3], (x, 4 * ny, z), 180)
                curtains(put, rng, kind, x, 4 * ny, z, 180)
        # ---- sides
        j0 = rows + (1 if round_corners else 0)
        core_side = {"east": "e", "west": "w"}.get(core, core) if isinstance(core, str) else None
        for j in range(j0, ny):
            for side, (px, py, rot) in (("w", (0, 4 * (j + 1), -90)), ("e", (4 * nx, 4 * j, 90))):
                if core_side == side and j == ny - 1:   # the door to the stair tower's landing
                    put("city_res_door" if use == "home" else "city_facade_door", (px, py, z), rot)
                    continue
                if use == "shop":
                    kind = "city_shop_glazed" if j == j0 else "city_facade_blank"
                elif use == "office":
                    kind = "city_office_glazed" if (j + s) % 2 else "city_office_service"
                else:
                    kind = HOME_SIDE[(j * 2 + s + seed + (side == "e")) % len(HOME_SIDE)]
                put(kind, (px, py, z), rot)
                inner = j not in (rows, ny - 1)
                if use == "home" and inner:
                    put(PROXY_HOME[(j + s + seed) % 3], (px, py, z), rot)
                    curtains(put, rng, kind, px, py, z, rot)
        # ---- corners
        corner_list = [(4 * nx, 4 * ny, 180), (0, 4 * ny, -90)]
        if not round_corners:
            corner_list += [(0, y0, 0), (4 * nx, y0, 90)]
        for cx, cy, r in corner_list:
            put("city_facade_corner", (cx, cy, z), r)
        # ---- the residents' stair tower and lift, outside the side wall
        if isinstance(core, tuple):           # ("back", bay): the tower stands behind that bay
            put("city_core_spiral", (4 * core[1], 4 * ny, z))
            put("city_lift", (4 * core[1] + 5.4, 4 * ny + 1.6, z))
        elif core:
            cx = 4 * nx if core_side == "e" else -4
            put("city_core_spiral", (cx, 4 * (ny - 1), z))
            put("city_lift", (cx + 2.0, 4 * (ny - 1) - 1.3, z))
    top = BAY * len(stack)
    roof_garden(L, put, rng, nx, ny, rows, top, round_corners, core, roof)
    return top


def home_bay(L, put, rng, x, y0, z, s, i, demo_cells, balcony_rate, nx, depth):
    furnished = (s, i) in demo_cells
    kind = HOME_FRONT[rng.randrange(len(HOME_FRONT))]
    if furnished and kind == "city_res_privacy":
        kind = "city_res_slider"
    put(kind, (x, y0, z))
    if kind in ("city_res_slider", "city_res_slider_b") and rng.random() < balcony_rate + 0.35:
        put("city_balcony_res_4", (x, y0, z))
        if rng.random() < 0.55:
            put("city_balcony_chairs", (x + 1.2, y0 - 0.8, z), rng.uniform(0, 360))
        if rng.random() < 0.4:
            put("city_laundry", (x + 3.0, y0 - 0.8, z), rng.choice((0, 90)))
        if rng.random() < 0.5:
            put("city_balcony_planter", (x + 2.0, y0 - 1.55, z + 1.05))
        if rng.random() < 0.3:
            put("city_shade_fabric", (x, y0, z))
        if rng.random() < 0.35:
            put("city_privacy_screen", (x + 3.95, y0 - 0.8, z))
        else:
            put("city_balcony_end", (x + 3.94, y0, z))
    elif kind == "city_res_loggia":
        if rng.random() < 0.7:
            put("city_balcony_chairs", (x + 2.6, y0 + 0.9, z), rng.uniform(0, 360))
    curtains(put, rng, kind, x, y0, z, 0)
    if furnished:
        return
    put(PROXY_HOME[rng.randrange(3)], (x, y0 + (1.6 if kind == "city_res_loggia" else 0.0), z))


def curtains(put, rng, kind, x, y, z, rot):
    """Curtains inside a home bay's windows, drawn in each window's own colour."""
    a = math.radians(rot)
    for cx in CURTAIN_X.get(kind, ()):
        if rng.random() < 0.8:
            lx, ly = cx, 0.36
            put(rng.choice(CURTAINS), (x + lx * math.cos(a) - ly * math.sin(a), y + lx * math.sin(a) + ly * math.cos(a), z), rot)


def furnished_home(L, F, s, i0, n, ny, group="Residential", seed=0):
    """A fully furnished home on storey s across bays i0..i0+n-1 (the full depth): a studio
    (1 bay), a one-bedroom (2) or a family home (3). Front rooms are for living; back rooms
    for kitchens and beds; partitions with doors between."""
    put = lambda aid, pos, rot=0.0: L.put(aid, pos, rot, group, F)  # noqa: E731
    rng = random.Random(seed)
    z = BAY * s
    for i in range(i0, i0 + n):
        for j in range(ny):
            put("city_room_floor_4", (4 * i, 4 * j, z))
    # walls between this home and its neighbours, and between front and back rooms
    for xw in (4 * i0, 4 * (i0 + n)):
        for j in range(ny):
            put("city_partition_4", (xw + 0.075, 4 * j + 4, z), -90)
    for i in range(i0, i0 + n):
        put("city_partition_door" if i == i0 else "city_partition_4", (4 * i, 4.0, z))
    x0 = 4 * i0
    # front: living
    put("city_sofa", (x0 + 2.0, 3.2, z), 180)
    put("city_coffee_table", (x0 + 2.0, 2.0, z))
    put("city_rug", (x0 + 2.0, 2.2, z))
    put("city_floor_lamp", (x0 + 3.5, 3.4, z))
    put("city_bookshelf", (x0 + 0.5, 3.6, z), -90)
    if n == 1:
        put("city_bed", (x0 + 2.0, 6.6, z))
        put("city_kitchen", (x0 + 0.4, 6.0, z), 90)
        return
    xk = x0 + 4
    put("city_kitchen", (xk + 2.0, 7.4, z))
    put("city_dining_set", (xk + 2.0, 5.6, z))
    put("city_pendant_lamp", (xk + 2.0, 5.6, z + 3.7))
    put("city_armchair", (xk + 1.0, 2.6, z), 160)
    put("city_bookshelf", (xk + 3.5, 3.6, z), 90)
    put("city_bed", (x0 + 2.0, 6.6, z))
    if n >= 3:
        xb = x0 + 8
        put("city_bed", (xb + 1.2, 6.6, z), 90)
        put("city_bed", (xb + 2.9, 6.6, z), 90)
        put("city_rug", (xb + 2.0, 2.0, z), 90)
        put("city_armchair", (xb + 2.0, 2.4, z), 200)


def roof_garden(L, put, rng, nx, ny, rows, top, round_corners, core, roof):
    """The shared roof: planted and decked tiles, a greenhouse, beds and a table, parapets
    round the edge (curved on rounded corners), and the stair head over the tower."""
    y0 = 4 * rows
    choices = ("city_roof_planted_4", "city_garden_deck", "city_roof_planted_4", "city_roof_cap_4")
    placed_greenhouse = False
    for i in range(nx):
        for j in range(rows, ny):
            if round_corners and j == rows and i in (0, nx - 1):
                put("city_roof_round", (0 if i == 0 else 4 * nx, y0, top), 0 if i == 0 else 90)
                continue
            if roof == "garden" and not placed_greenhouse and i == nx // 2 and j == ny - 1:
                put("city_roof_cap_4", (4 * i, 4 * j, top))
                put("city_greenhouse", (4 * i, 4 * j, top))
                placed_greenhouse = True
                continue
            kind = choices[(i + 2 * j + rows) % len(choices)] if roof == "garden" else "city_roof_planted_4"
            put(kind, (4 * i, 4 * j, top))
            if kind == "city_garden_deck" and rng.random() < 0.6:
                extra = rng.choice(("city_communal_table", "city_raised_beds", "city_pergola"))
                put(extra, (4 * i, 4 * j, top) if extra == "city_pergola" else (4 * i + 2, 4 * j + 2.2, top), 0)
    first, last = (1, nx - 1) if round_corners else (0, nx)
    for i in range(first, last):
        put("city_parapet_4", (4 * i, y0, top))
    for i in range(nx):
        put("city_parapet_4", (4 * (i + 1), 4 * ny, top), 180)
    j0 = rows + (1 if round_corners else 0)
    for j in range(j0, ny):
        put("city_parapet_4", (0, 4 * (j + 1), top), -90)
        put("city_parapet_4", (4 * nx, 4 * j, top), 90)
    corners = [(4 * nx, 4 * ny, 180), (0, 4 * ny, -90)]
    if not round_corners:
        corners += [(0, y0, 0), (4 * nx, y0, 90)]
    for cx, cy, r in corners:
        put("city_parapet_corner", (cx, cy, top), r)
    if isinstance(core, tuple):
        put("city_roof_access", (4 * core[1], 4 * ny, top))
    elif core:
        cx = 4 * nx if core in ("e", "east") else -4
        put("city_roof_access", (cx, 4 * (ny - 1), top))
