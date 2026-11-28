#!/usr/bin/env python3
"""Validate the City kit's exported models from their glTF JSON (no Blender needed).

  python3 tools/blender/check_city_kit.py        # every assets/structures/city/city_*/ model

Checks, per model: every material carries extras.acoustic (a known preset or "none");
mesh nodes have no negative scale and no rotation or scale beyond identity (transforms
applied); every primitive has POSITION, NORMAL and TEXCOORD_0; no images, cameras or
lights; an `<id>_LOD0` node; the sockets the profile names exist as nodes; large-mesh
node extras use known presets; the prefab names the model. Writes the result into
each profile's `validation` field and prints a summary.
"""
from pathlib import Path
import json
import struct
import sys

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "assets/structures/city"
PRESETS = {"generic", "rock", "concrete", "brick", "wood", "glass", "metal", "carpet", "gravel"}


def gltf(path):
    data = path.read_bytes()
    n = struct.unpack("<I", data[12:16])[0]
    return json.loads(data[20:20 + n])


def check(folder):
    aid = folder.name
    errors = []
    doc = gltf(folder / f"{aid}.glb")
    prof = json.loads((folder / "asset.json").read_text())
    for m in doc.get("materials", []):
        cls = m.get("extras", {}).get("acoustic")
        if cls is None:
            errors.append(f"material {m['name']} has no acoustic extras")
        elif cls != "none" and cls not in PRESETS:
            errors.append(f"material {m['name']} has unknown acoustic {cls!r}")
    names = {n.get("name") for n in doc["nodes"]}
    for node in doc["nodes"]:
        s = node.get("scale", [1, 1, 1])
        if any(c < 0 for c in s):
            errors.append(f"node {node.get('name')} has negative scale")
        if "mesh" in node and (node.get("rotation", [0, 0, 0, 1]) != [0, 0, 0, 1] or s != [1, 1, 1]):
            errors.append(f"mesh node {node.get('name')} is not identity-transformed")
        cls = node.get("extras", {}).get("acoustic")
        if cls is not None and cls not in PRESETS:
            errors.append(f"node {node.get('name')} has unknown acoustic {cls!r}")
    for mesh in doc.get("meshes", []):
        for prim in mesh["primitives"]:
            for attr in ("POSITION", "NORMAL", "TEXCOORD_0"):
                if attr not in prim["attributes"]:
                    errors.append(f"mesh {mesh['name']} lacks {attr}")
    for key in ("images", "cameras"):
        if doc.get(key):
            errors.append(f"model carries {key}")
    if doc.get("extensions", {}).get("KHR_lights_punctual"):
        errors.append("model carries lights")
    if f"{aid}_LOD0" not in names:
        errors.append("no LOD0 node")
    if prof["triangles"].get("far") is not None and f"{aid}_LOD1" not in names:
        errors.append("profile has a far level but the model has no LOD1 node")
    for s in prof.get("sockets", {}):
        if s not in names:
            errors.append(f"socket {s} missing from the model")
    bsn = (folder / f"{aid}.bsn").read_text()
    if f"structures/city/{aid}/{aid}.glb" not in bsn:
        errors.append("prefab does not name the model")
    tris = prof["triangles"]["near"]
    return errors, tris, len(doc.get("materials", []))


def main():
    folders = sorted(p for p in OUT.glob("city_*") if p.is_dir())
    bad = 0
    for f in folders:
        errors, tris, mats = check(f)
        prof_path = f / "asset.json"
        prof = json.loads(prof_path.read_text())
        prof["validation"] = "ok: glTF checks passed (check_city_kit.py)" if not errors else "FAILED: " + "; ".join(errors)
        prof_path.write_text(json.dumps(prof, indent=2) + "\n")
        if errors:
            bad += 1
            print(f"FAIL {f.name}: " + "; ".join(errors))
    print(f"{len(folders) - bad}/{len(folders)} models passed")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
