#!/usr/bin/env python3
"""Import every model's levels of detail into its Jackdaw import settings, in one go.

Levels of detail belong to the model: Jackdaw keeps them in the model's import
settings (`<model>.glb.meta`). Its editor sets them one model at a time; this runs the
same operators for every model through Jackdaw's documented remote control
(`jd mcp`, which drives the open editor over BRP). The editor itself is not changed.

For each model under `assets/`:
- it has `<id>_LOD<n>` nodes inside it  -> `model.lod.import` (the hand-made levels)
- it is listed in GENERATE              -> `model.lod.generate` (simplified levels)
- otherwise it is left alone
then `model.lod.apply` writes its `.meta`, and finally each level's switch point is set
from its reference handover/final-cull distances in tools/assets/lod_policy.json.
The policy also sets the crossfade; runtime selection remains Jackdaw's.
Commit the `.meta` files.

Use: open the project in Jackdaw (main checkout), then
    python3 tools/jackdaw_lods.py            # all models
    python3 tools/jackdaw_lods.py --dry-run  # list what it would do
    python3 tools/jackdaw_lods.py --switch   # apply handovers, fade and final cull
    python3 tools/jackdaw_lods.py --check    # verify committed metadata matches policy
"""
from pathlib import Path
import json
import math
import re
import struct
import subprocess
import sys

REPO = Path(__file__).resolve().parents[1]
PROJECT = REPO
MODELS = REPO / "assets"
# Models without hand-made levels that are heavy enough to want generated ones.
GENERATE = {
    "machines/mudpit_burbler/mudpit_burbler.glb": 3,
    "props/desert/desert_brush_0/desert_brush_0.glb": 3,
    "props/desert/desert_brush_1/desert_brush_1.glb": 3,
    "props/desert/desert_brush_2/desert_brush_2.glb": 3,
    "props/desert/desert_stump_0/desert_stump_0.glb": 2,
    "props/desert/desert_stump_1/desert_stump_1.glb": 2,
    "props/desert/desert_stump_2/desert_stump_2.glb": 2,
}


# Model metadata is the runtime authority. This file is an explicit authoring input:
# reference metres at unit placement scale; placed scale and camera FOV remain
# Jackdaw's responsibility. Shells are placed at 3x scale in the game.
POLICY_PATH = REPO / "tools/assets/lod_policy.json"
NUMBER = r"[-+0-9.eE]+"


def tuning_policy():
    policy = json.loads(POLICY_PATH.read_text())
    if not 0 < policy["reference_vertical_fov_degrees"] < 180:
        raise ValueError("LOD policy: reference FOV must be between 0 and 180 degrees")
    if not 0 < policy["crossfade_fraction"] < 1:
        raise ValueError("LOD policy: crossfade fraction must be between 0 and 1")
    return policy


def tuned_meta(meta, policy):
    """Return proposed metadata and thresholds without writing or contacting Jackdaw."""
    text = meta.read_text()
    rel = meta.relative_to(MODELS).as_posix().removesuffix(".meta")
    matches = [family for family in policy["families"] if Path(rel).match(family["models"])]
    if len(matches) != 1:
        raise ValueError(f"{rel}: expected one tuning policy, found {len(matches)}")
    family = matches[0]
    if "handover_profile" in family:
        profile = json.loads((MODELS / family["handover_profile"]).read_text())
        handovers = [profile["far_lod"]["distance_m"]]
    else:
        handovers = family["handover_metres"]
    distances = [*handovers, family["cull_metres"]]
    if not distances or any(not math.isfinite(d) or d <= 0 for d in distances):
        raise ValueError(f"{rel}: distances must be finite and positive")
    if any(a >= b for a, b in zip(distances, distances[1:])):
        raise ValueError(f"{rel}: handovers and final cull must increase strictly")
    size = re.search(rf"size: ({NUMBER})", text)
    if size is None or not math.isfinite(float(size[1])) or float(size[1]) <= 0:
        raise ValueError(f"{rel}: missing positive imported model size; apply its import first")
    size = float(size[1])
    fov = math.radians(policy["reference_vertical_fov_degrees"])
    heights = [round(size / (2 * d * math.tan(fov / 2)), 6) for d in distances]
    if any(height <= 0 for height in heights):
        raise ValueError(f"{rel}: a threshold rounded to zero; reduce the cull distance")
    found = re.findall(rf"screen_height: {NUMBER}", text)
    if len(found) != len(heights):
        raise ValueError(f"{rel}: {len(found)} levels, {len(heights)} configured thresholds")
    values = iter(heights)
    text = re.sub(rf"screen_height: {NUMBER}", lambda _: f"screen_height: {next(values)}", text)
    fade = f"fade: CrossFade(width: {policy['crossfade_fraction']}),"
    text, count = re.subn(r"fade: (?:Snap|CrossFade\s*\([^)]*\)),", fade, text)
    if count != 1:
        raise ValueError(f"{rel}: expected one model fade setting, found {count}")
    return text, heights


def set_switch_points(meta, policy=None):
    text, heights = tuned_meta(meta, policy or tuning_policy())
    meta.write_text(text)
    return heights


def has_lod_nodes(glb):
    data = glb.read_bytes()
    length = struct.unpack("<I", data[12:16])[0]
    nodes = json.loads(data[20:20 + length]).get("nodes", [])
    for node in nodes:
        name = node.get("name", "").lower()
        at = name.rfind("_lod")
        if at >= 0 and name[at + 4:].isdigit():
            return True
    return False


def plan():
    jobs = []
    for glb in sorted(MODELS.rglob("*.glb")):
        rel = glb.relative_to(MODELS).as_posix()
        path = rel
        if has_lod_nodes(glb):
            jobs.append((path, "model.lod.import", {"path": path}))
        elif rel in GENERATE:
            jobs.append((path, "model.lod.generate", {"path": path, "count": GENERATE[rel]}))
    return jobs


class Mcp:
    """A minimal MCP client over `jd mcp`'s stdio: newline-delimited JSON-RPC."""

    def __init__(self):
        self.proc = subprocess.Popen(["jd", "mcp", "--project", str(PROJECT)], stdin=subprocess.PIPE,
                                     stdout=subprocess.PIPE, text=True)
        self.next_id = 0
        self.request("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                                    "clientInfo": {"name": "stubby-jackdaw-lods", "version": "1"}})
        self.send({"jsonrpc": "2.0", "method": "notifications/initialized"})

    def send(self, message):
        self.proc.stdin.write(json.dumps(message) + "\n")
        self.proc.stdin.flush()

    def request(self, method, params):
        self.next_id += 1
        self.send({"jsonrpc": "2.0", "id": self.next_id, "method": method, "params": params})
        while True:
            line = self.proc.stdout.readline()
            if not line:
                raise RuntimeError("jd mcp closed")
            reply = json.loads(line)
            if reply.get("id") == self.next_id:
                if "error" in reply:
                    raise RuntimeError(reply["error"])
                return reply["result"]

    def tool(self, name, arguments):
        result = self.request("tools/call", {"name": name, "arguments": arguments})
        text = " ".join(c.get("text", "") for c in result.get("content", []))
        if result.get("isError"):
            raise RuntimeError(text)
        return text

    def close(self):
        self.proc.stdin.close()
        self.proc.wait(timeout=10)


def switch_points(check=False):
    policy = tuning_policy()
    # Validate every file before changing any of them. A new unconfigured model
    # must fail visibly rather than silently inheriting a blanket cutoff.
    planned = [(meta, *tuned_meta(meta, policy)) for meta in sorted(MODELS.rglob("*.glb.meta"))
               if re.search(r"\blod:\s*Some\(", meta.read_text())]
    for meta, text, heights in planned:
        if check:
            if text != meta.read_text():
                raise SystemExit(f"{meta.relative_to(MODELS)}: metadata differs from LOD policy")
        else:
            meta.write_text(text)
    print(f"{len(planned)} model import settings {'match' if check else 'updated from'} the LOD policy")



def main():
    if "--check" in sys.argv:
        switch_points(check=True)
        return
    if "--switch" in sys.argv:
        switch_points()
        return
    jobs = plan()
    for path, op, _ in jobs:
        print(f"{op:20} {path}")
    if "--dry-run" in sys.argv:
        return
    mcp = Mcp()
    print(mcp.tool("status", {}))
    for path, op, params in jobs:
        mcp.tool("call_operator", {"id": op, "params": params, "history": False})
        mcp.tool("wait", {"frames": 5})
        mcp.tool("call_operator", {"id": "model.lod.apply", "params": {"path": path}, "history": False})
        mcp.tool("wait", {"frames": 2})
        meta = MODELS / (path + ".meta")
        print(f"{'ok' if meta.exists() else 'NO META':8} {path}")
    mcp.close()
    switch_points()


if __name__ == "__main__":
    main()
