#!/usr/bin/env python3
"""Import every model's levels of detail into its Jackdaw import settings, in one go.

Levels of detail belong to the model: Jackdaw keeps them in the model's import
settings (`<model>.glb.meta`). Its editor sets them one model at a time; this runs the
same operators for every model through Jackdaw's documented remote control
(`jd mcp`, which drives the open editor over BRP). The editor itself is not changed.

For each model under `assets/models/`:
- it has `<id>_LOD<n>` nodes inside it  -> `model.lod.import` (the hand-made levels)
- it is listed in GENERATE              -> `model.lod.generate` (simplified levels)
- otherwise it is left alone
then `model.lod.apply` writes its `.meta`. Commit the `.meta` files.

Use: open the project in Jackdaw (main checkout), then
    python3 tools/jackdaw_lods.py            # all models
    python3 tools/jackdaw_lods.py --dry-run  # list what it would do
"""
from pathlib import Path
import json
import struct
import subprocess
import sys

REPO = Path(__file__).resolve().parents[1]
PROJECT = REPO / "jackdaw"
MODELS = REPO / "assets/models"   # jackdaw/assets/models links here
# Models without hand-made levels that are heavy enough to want generated ones.
GENERATE = {
    "machines/mudpit_burbler.glb": 3,
    "props/desert/desert_brush_0.glb": 3,
    "props/desert/desert_brush_1.glb": 3,
    "props/desert/desert_brush_2.glb": 3,
    "props/desert/desert_stump_0.glb": 2,
    "props/desert/desert_stump_1.glb": 2,
    "props/desert/desert_stump_2.glb": 2,
}


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
        path = f"models/{rel}"
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


def main():
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
        meta = MODELS / (path.removeprefix("models/") + ".meta")
        print(f"{'ok' if meta.exists() else 'NO META':8} {path}")
    mcp.close()


if __name__ == "__main__":
    main()
