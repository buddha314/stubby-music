"""The validation bay: one complete mixed-use building slice, built before the residential
kit is multiplied (docs/art/city-residential.md).

Two bays wide and two deep: a shop and the residents' own street door on the ground floor,
an office floor, three storeys of homes (one a fully furnished one-bedroom home), the glass
stair tower and lift, and a shared roof garden. It proves the stack snaps (bands, slabs,
corners, balconies), exports and loads, before the neighbourhood is built from it.

  blender -b --factory-startup -P tools/blender/create_city_mixed_bay.py

Writes assets-src/blender/structures/city/city_mixed_bay.blend (Day, Dusk, Night),
previews/structures/city/mixed_bay_<view>_<light>.jpg and
assets/world/asset_reviews/city_mixed_bay.bsn (placed ahead of the game's spawn).
"""
from pathlib import Path
import math
import sys

import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
import create_city_district as cd  # noqa: E402
from city_layout import Frame, Layout  # noqa: E402
from city_mixed import furnished_home, mixed_block  # noqa: E402

ROOT = cd.ROOT
OUT = cd.SOURCES / "city_mixed_bay.blend"
# Ahead of the game's spawn (Blender 0, -6, facing about +X): the bay's front faces -X.
ORIGIN, ROT = (16.0, 8.0, 0.0), -90


def layout():
    L = Layout()
    F = Frame(ORIGIN, ROT)
    stack = ["shop", "office", "home", "home", "home"]
    mixed_block(L, F, 2, 2, stack, seed=7, group="Bay", lobby=0, core="east", demo=[(2, 0, 2)],
                balcony_rate=0.9)
    furnished_home(L, F, 2, 0, 2, 2, group="Bay", seed=7)
    for x in range(-8, 40, 8):          # paving round it
        for y in range(-24, 24, 8):
            if 16 <= x < 24 and 0 <= y < 8:
                continue
            L.put("city_paving_8", (x, y, 0), 0, "Ground")
    L.put("city_tree", (10.0, -6.0, 0), 30, "Ground")
    L.put("city_bench", (11.0, 4.0, 0), 90, "Ground")
    L.put("city_glow_bell", (12.5, 9.5, 0), 0, "Ground")
    return L


VIEWS = {   # name: (location, aim, lens)
    "Street": ((3.0, -7.0, 1.65), (16.0, 3.0, 9.0), 24),
    "Stack": ((-4.0, 6.0, 9.0), (16.0, 3.0, 9.0), 30),
    "Home": ((19.6, 7.2, 9.65), (15.0, 1.5, 9.2), 20),
}


def main():
    L = layout()
    cd.write_bsn(L, name="city_mixed_bay", lift=0.0, header=[
        "// Asset review: the City residential kit's validation bay (create_city_mixed_bay.py):",
        "// shop and residents' door, an office floor, three home storeys (one furnished), the stair",
        "// tower and a roof garden, standing ahead of the spawn. Not a level."])
    for o in list(bpy.data.objects):
        bpy.data.objects.remove(o, do_unlink=True)
    root = bpy.context.scene.collection
    kit = cd.link_kit({it["id"] for it in L.items})
    groups = {}
    for g in ("Bay", "Ground"):
        c = bpy.data.collections.new(g)
        groups[g] = c
    for it in L.items:
        cd.instance(it["id"], kit[it["id"]], it["pos"], it["rot"], groups[it["group"]])
    ground = bpy.data.collections.new("Context")
    me = bpy.data.meshes.new("Ground")
    me.from_pydata([(-200, -200, -0.32), (200, -200, -0.32), (200, 200, -0.32), (-200, 200, -0.32)], [], [(0, 1, 2, 3)])
    m = bpy.data.materials.new("Ground")
    m.diffuse_color = (*cd.srgb("#29361c"), 1)
    m.use_nodes = True
    m.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (*cd.srgb("#29361c"), 1)
    me.materials.append(m)
    ground.objects.link(bpy.data.objects.new("Ground", me))
    scenes = cd.make_scenes([groups["Bay"], groups["Ground"], ground])
    cams = {}
    for name, (loc, aim, lens) in VIEWS.items():
        cam = bpy.data.objects.new(f"Bay-{name}", bpy.data.cameras.new(f"Bay-{name}"))
        cam.location = loc
        cam.rotation_euler = (Vector(aim) - Vector(loc)).to_track_quat("-Z", "Y").to_euler()
        cam.data.lens = lens
        for sc in scenes.values():
            sc.collection.objects.link(cam)
        cams[name] = cam
    bpy.context.preferences.filepaths.save_version = 0
    if bpy.context.window:
        bpy.context.window.scene = scenes["Day"]
    bpy.ops.wm.save_as_mainfile(filepath=str(OUT), check_existing=False, relative_remap=True)
    print("saved", OUT, "instances", len(L.items))
    for view, light in (("Street", "Day"), ("Stack", "Day"), ("Stack", "Dusk"), ("Home", "Day"), ("Stack", "Night")):
        sc = scenes[light]
        if light == "Night":
            cd.glare(sc)
        sc.camera = cams[view]
        sc.render.image_settings.file_format = "JPEG"
        sc.render.image_settings.quality = 90
        sc.render.filepath = str(cd.PREVIEWS / f"mixed_bay_{view.lower()}_{light.lower()}.jpg")
        bpy.ops.render.render(write_still=True, scene=sc.name)
        print("rendered", view, light, flush=True)


if __name__ == "__main__":
    main()
