"""The City kit (Phase 4): build every modular piece as its own placeable asset.

Brief: docs/art/city-kit.md. Contract and helpers: tools/blender/city_kit.py.

  blender -b --factory-startup -P tools/blender/create_city_kit.py            # every piece
  blender -b --factory-startup -P tools/blender/create_city_kit.py -- city_paving_4 city_tree
  python3 tools/blender/create_city_kit.py index                              # index only

Each piece is built in a clean Blender process and writes:
  assets-src/blender/structures/city/<id>.blend   source, its near level marked as an asset
  assets/structures/city/<id>/<id>.glb            the model (LOD nodes, sockets, extras)
  assets/structures/city/<id>/<id>.bsn            the Jackdaw prefab
  assets/structures/city/<id>/asset.json          the profile
The index step (run after) writes the family's asset.json, colliders.json, the manifest
(docs/art/city-kit/manifest.csv and .json) and the Blender asset catalog file.
"""
from pathlib import Path
import csv
import json
import subprocess
import sys

HERE = Path(__file__).resolve().parent
PROJECT = HERE.parents[1]
sys.path.insert(0, str(HERE))
OUT = PROJECT / "assets/structures/city"
SOURCES = PROJECT / "assets-src/blender/structures/city"
DOCS = PROJECT / "docs/art/city-kit"


def builders():
    from city_build_p0 import P0
    found = dict(P0)
    for module in ("city_build_p1", "city_build_p2"):
        try:
            found.update(__import__(module).__dict__[module.split("_")[-1].upper()])
        except ModuleNotFoundError:
            pass
    return found


def bevy(c):
    """Blender (x, y, z), Z up, to Bevy/glTF (x, y, z), Y up, forward -Z."""
    return [round(c[0], 3), round(c[2], 3), round(-c[1], 3)]


META = """(
    meta_format_version: "1.0",
    asset: Load(
        loader: "jackdaw_scene_types::model_import::ModelLoader",
        settings: (
            gltf: (
                load_meshes: ("MAIN_WORLD | RENDER_WORLD"),
                load_materials: ("RENDER_WORLD"),
                load_cameras: true,
                load_lights: true,
                load_animations: true,
                include_source: false,
                default_sampler: None,
                override_sampler: false,
                validate: true,
                convert_coordinates: None,
                skinned_mesh_bounds_policy: None,
                mesh_compression: None,
            ),
            lod: Some((
                version: 1,
                source: NodeSuffixes,
                size: {size:.4f},
                fade: CrossFade(width: {fade}),
                levels: [
                    (
                        show: Nodes([
                            "{aid}_LOD0",
                        ]),
                        screen_height: {h0},
                    ),
                    (
                        show: Nodes([
                            "{aid}_LOD1",
                        ]),
                        screen_height: {h1},
                    ),
                ],
            )),
        ),
    ),
)"""


def write_lod_meta(aid, prof):
    """The model's LOD import settings, as Jackdaw's Import LOD levels + Apply would write
    them (size: the near level's largest extent), switched by tools/assets/lod_policy.json
    exactly as `tools/jackdaw_lods.py --switch` computes it."""
    import math
    policy = json.loads((PROJECT / "tools/assets/lod_policy.json").read_text())
    rel = f"structures/city/{aid}/{aid}.glb"
    fam = [f for f in policy["families"] if Path(rel).match(f["models"])]
    if len(fam) != 1:
        raise SystemExit(f"{rel}: needs exactly one family in tools/assets/lod_policy.json, found {len(fam)}")
    distances = [*fam[0]["handover_metres"], fam[0]["cull_metres"]]
    size = max(prof["size_metres"].values())
    fov = math.radians(policy["reference_vertical_fov_degrees"])
    h0, h1 = (round(size / (2 * d * math.tan(fov / 2)), 6) for d in distances)
    (OUT / aid / f"{aid}.glb.meta").write_text(META.format(size=size, fade=policy["crossfade_fraction"],
                                                           aid=aid, h0=h0, h1=h1))


def index():
    from city_catalog import CATALOG_ID, CATALOG_PATH
    profiles = {}
    for f in sorted(OUT.glob("city_*/asset.json")):
        profiles[f.parent.name] = json.loads(f.read_text())
    OUT.mkdir(parents=True, exist_ok=True)
    rows, colliders = [], {}
    for aid, prof in profiles.items():
        if prof["triangles"].get("far") is not None:
            write_lod_meta(aid, prof)
        kit = prof.get("kit", {})
        shapes = []
        for c in kit.get("colliders", []):
            entry = {"translation": bevy(c["centre"])}
            if c["shape"] == "box":
                entry.update(shape="Cuboid", x_length=c["size"][0], y_length=c["size"][2], z_length=c["size"][1])
            else:
                entry.update(shape="Cylinder", radius=c["size"][0], height=c["size"][1])
            if c.get("slope"):
                entry["tilt_about_x_radians"] = round(c["slope"], 4)
            if c.get("walkable"):
                entry["walkable"] = True
            shapes.append(entry)
        if shapes:
            colliders[f"structures/city/{aid}/{aid}.glb"] = {"colliders": shapes, "rigid_body": "none (static)"}
        size = prof["size_metres"]
        rows.append({
            "id": aid, "priority": prof["priority"], "catalog": prof["catalog"],
            "model": prof["model"].split("#")[0], "prefab": f"structures/city/{aid}/{aid}.bsn",
            "source": f"assets-src/blender/structures/city/{aid}.blend",
            "width_m": size["width"], "depth_m": size["depth"], "height_m": size["height"],
            "origin": ("centre of base" if prof["bounds_blender"]["min"][0] < -0.01 and prof["bounds_blender"]["max"][0] > 0.01
                       and abs(prof["bounds_blender"]["min"][0] + prof["bounds_blender"]["max"][0]) < 0.6
                       else "left front snap corner, ground level"),
            "connectors": ";".join(f"{n}@{v['position']}->{v['direction']}" for n, v in prof.get("connectors", {}).items()),
            "sockets": ";".join(prof.get("sockets", {}).keys()),
            "materials_acoustic": ";".join(f"{m}={c or 'none'}" for m, c in prof["acoustics"]["materials"].items()),
            "triangles_near": prof["triangles"]["near"], "triangles_far": prof["triangles"].get("far", ""),
            "phases": ";".join(prof["phases"]), "licence": "CC0-1.0 (generated)",
            "validation": prof.get("validation", "not validated"),
        })
    (OUT / "colliders.json").write_text(json.dumps({
        "about": "Static collider proxies for the City kit, written by create_city_kit.py index from "
                 "each piece's simple shapes. Same form as the other families' colliders.json "
                 "(model coordinates, Bevy axes, Y up, metres; AvianCollider(ColliderConstructor) on "
                 "a child entity at `translation`, no RigidBody). Two additions, not yet read by any "
                 "code: `tilt_about_x_radians` (stairs and ramps collide as a tilted slab, rotate the "
                 "child about its local X) and `walkable` (a surface the player should stand on; the "
                 "controller does not support standing on colliders yet).",
        "models": colliders}, indent=2) + "\n")
    (OUT / "asset.json").write_text(json.dumps({
        "id": "city_kit", "kind": "structure", "family": "city",
        "about": "The City kit (Phase 4): modular construction pieces, canal, civic landmarks, "
                 "planting and luminous flora on a 4 m grid. Brief: docs/art/city-kit.md. "
                 "Manifest: docs/art/city-kit/manifest.csv.",
        "pieces": {aid: p["model"] for aid, p in profiles.items()},
    }, indent=2) + "\n")
    DOCS.mkdir(parents=True, exist_ok=True)
    with open(DOCS / "manifest.csv", "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0]))
        w.writeheader()
        w.writerows(rows)
    (DOCS / "manifest.json").write_text(json.dumps(
        {aid: {"model": p["model"], "size_metres": p["size_metres"], "bounds_blender": p["bounds_blender"],
               "connectors": p.get("connectors", {}), "sockets": p.get("kit", {}).get("sockets", {}),
               "triangles": p["triangles"], "catalog": p["catalog"], "priority": p["priority"],
               "phases": p["phases"]} for aid, p in profiles.items()}, indent=1) + "\n")
    lines = ["# Blender asset catalogs for the City kit (create_city_kit.py)", "VERSION 1", ""]
    lines += [f"{CATALOG_ID[k]}:{CATALOG_PATH[k]}:{CATALOG_PATH[k].replace('/', '-')}" for k in CATALOG_ID]
    SOURCES.mkdir(parents=True, exist_ok=True)
    (SOURCES / "blender_assets.cats.txt").write_text("\n".join(lines) + "\n")
    print(f"indexed {len(rows)} pieces")


def main():
    try:
        import bpy
    except ModuleNotFoundError:
        bpy = None
    if "--" in sys.argv:
        args = sys.argv[sys.argv.index("--") + 1:]
    else:
        args = [] if bpy else sys.argv[1:]
    if args == ["index"] or bpy is None:
        index()
        return
    table = builders()
    names = args or list(table)
    unknown = [n for n in names if n not in table]
    if unknown:
        raise SystemExit(f"unknown piece(s) {unknown}")
    if len(names) > 1:
        failed = []
        for n in names:
            r = subprocess.run([bpy.app.binary_path, "-b", "--factory-startup", "-P", __file__, "--", n],
                               capture_output=True, text=True)
            ok = r.returncode == 0 and "CITY_BUILT" in r.stdout
            print(("ok   " if ok else "FAIL ") + n, flush=True)
            if not ok:
                failed.append(n)
                print("\n".join((r.stdout + r.stderr).splitlines()[-25:]))
        index()
        if failed:
            raise SystemExit(f"failed: {failed}")
        return
    piece = table[names[0]]()
    row = piece.finish()
    print("CITY_BUILT", json.dumps({"id": row["id"], "triangles": row["profile"]["triangles"]}))


if __name__ == "__main__":
    main()
