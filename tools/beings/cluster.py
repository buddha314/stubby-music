# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy==2.5.5", "scipy==1.18.1", "scikit-learn==1.9.1", "matplotlib==3.11.2"]
# ///
"""Cluster the being atlas by its traits and compare the clusters with biomes.

    uv run tools/beings/cluster.py            # write the report, figures and layout
    uv run tools/beings/cluster.py --check    # validate beings.json only

Reads docs/research/beings/beings.json and writes, beside it:
  report.md     clusters, how they line up with biomes, each biome's signature traits
  layout.json   a seeded 2D layout (metric MDS) that keeps similar beings close: the
                amphitheatre's seating plan, once there is one
  dendrogram.png, map.png
Biome is never a clustering input: the point is to see whether traits alone sort beings
into their landscapes.
"""

import argparse
import json
import re
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ATLAS = ROOT / "docs/research/beings"
SEED = 314


def load(path):
    data = json.loads(path.read_text())
    errors, ids = [], set()
    facets = data["facets"]
    for b in data["beings"]:
        bid = b.get("id", "?")
        if not re.fullmatch(r"[a-z0-9_]+", bid) or bid in ids:
            errors.append(f"{bid}: id must be unique snake_case")
        ids.add(bid)
        for key, allowed in (("kind", data["kinds"]), ("status", data["statuses"])):
            if b.get(key) not in allowed:
                errors.append(f"{bid}: {key} {b.get(key)!r} not in {allowed}")
        if not b.get("biomes") or any(x not in data["biomes"] for x in b["biomes"]):
            errors.append(f"{bid}: biomes must be from {data['biomes']}")
        known = {"id", "essence", "kind", "biomes", "status", "found_in", *facets}
        for key in set(b) - known:
            errors.append(f"{bid}: unknown field {key!r}")
        for facet, allowed in facets.items():
            for v in b.get(facet, []):
                if v not in allowed:
                    errors.append(f"{bid}: {facet} {v!r} not in vocabulary")
    if errors:
        sys.exit("beings.json has errors:\n  " + "\n  ".join(errors))
    return data


def distances(data):
    """Mean over facets of the Jaccard distance; a facet empty on both sides is skipped."""
    import numpy as np

    beings, facets = data["beings"], list(data["facets"])
    n = len(beings)
    d = np.zeros((n, n))
    for i in range(n):
        for j in range(i + 1, n):
            parts = []
            for f in facets:
                a, b = set(beings[i].get(f, [])), set(beings[j].get(f, []))
                if a or b:
                    parts.append(1 - len(a & b) / len(a | b))
            d[i, j] = d[j, i] = sum(parts) / len(parts)
    return d


def run(data):
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    import numpy as np
    from scipy.cluster.hierarchy import dendrogram, fcluster, linkage
    from scipy.spatial.distance import squareform
    from sklearn.manifold import MDS
    from sklearn.metrics import adjusted_rand_score, silhouette_score

    beings = data["beings"]
    ids = [b["id"] for b in beings]
    primary = [b["biomes"][0] for b in beings]
    n = len(beings)
    d = distances(data)
    tree = linkage(squareform(d, checks=False), method="average")

    # k by silhouette, over every cut that leaves at least two beings per cluster on average
    scores = {}
    for k in range(2, max(3, n // 2) + 1):
        labels = fcluster(tree, k, criterion="maxclust")
        if len(set(labels)) > 1:
            scores[k] = silhouette_score(d, labels, metric="precomputed")
    k = max(scores, key=scores.get)
    labels = fcluster(tree, k, criterion="maxclust")
    ari = adjusted_rand_score(primary, labels)

    xy = MDS(n_components=2, metric="precomputed", init="classical_mds", random_state=SEED).fit_transform(d)
    xy = (xy - xy.mean(0)) / np.abs(xy).max()

    colours = dict(zip(data["biomes"], plt.get_cmap("tab10").colors))

    fig, ax = plt.subplots(figsize=(10, 0.35 * n + 2))
    dendrogram(tree, labels=ids, orientation="right", ax=ax, color_threshold=tree[-k + 1, 2] if k > 1 else None)
    for t in ax.get_yticklabels():
        t.set_color(colours[primary[ids.index(t.get_text())]])
    ax.set_title(f"Beings by traits (average linkage, mean per-facet Jaccard); label colour = biome; k={k}")
    fig.tight_layout()
    fig.savefig(ATLAS / "dendrogram.png", dpi=120)

    fig, ax = plt.subplots(figsize=(9, 9))
    for i, b in enumerate(beings):
        ax.scatter(*xy[i], color=colours[primary[i]], s=120, marker="o" if b["kind"] != "monster" else "X")
        ax.annotate(b["id"], xy[i], textcoords="offset points", xytext=(6, 4), fontsize=8)
    for biome, c in colours.items():
        ax.scatter([], [], color=c, label=biome)
    ax.legend(loc="lower right", fontsize=8)
    ax.set_title("Being map (MDS of trait distance): the amphitheatre's seating")
    ax.set_aspect("equal")
    fig.tight_layout()
    fig.savefig(ATLAS / "map.png", dpi=120)

    layout = {
        "about": "Generated by tools/beings/cluster.py; do not edit. x, y in [-1, 1].",
        "seed": SEED,
        "beings": [
            {"id": ids[i], "x": round(float(xy[i, 0]), 4), "y": round(float(xy[i, 1]), 4),
             "cluster": int(labels[i]), "biome": primary[i]}
            for i in range(n)
        ],
    }
    (ATLAS / "layout.json").write_text(json.dumps(layout, indent=2) + "\n")

    # report
    out = ["# Being atlas: cluster report", "",
           "Generated by `uv run tools/beings/cluster.py` from `beings.json`; do not edit.", "",
           f"{n} beings ({dict(Counter(b['kind'] for b in beings))}). Distance: mean over facets of the "
           "Jaccard distance between trait sets. Clustering: average linkage, k chosen by silhouette. "
           "Biome is not an input.", "",
           f"- **k = {k}** (silhouette {scores[k]:.2f}; tried " + ", ".join(f"{kk}: {s:.2f}" for kk, s in scores.items()) + ")",
           f"- **Agreement with biome** (adjusted Rand index, primary biome): **{ari:.2f}** "
           "(1 = clusters are exactly the biomes, 0 = no better than chance)", "",
           "![dendrogram](dendrogram.png)", "", "![map](map.png)", "", "## Clusters", ""]
    facets = list(data["facets"])
    for c in sorted(set(labels)):
        members = [i for i in range(n) if labels[i] == c]
        traits = Counter(f"{f}:{v}" for i in members for f in facets for v in beings[i].get(f, []))
        shared = [t for t, m in traits.most_common() if m >= max(2, (len(members) + 1) // 2)][:8]
        out.append(f"### Cluster {c} ({len(members)})")
        out.append("")
        out.append("- " + ", ".join(f"`{ids[i]}` ({primary[i]})" for i in members))
        out.append("- shared traits: " + (", ".join(f"`{t}`" for t in shared) or "none"))
        out.append("")
    out += ["## Clusters × biomes", "", "| biome | " + " | ".join(f"c{c}" for c in sorted(set(labels))) + " |",
            "| --- |" + " --- |" * len(set(labels))]
    for biome in data["biomes"]:
        row = [sum(1 for i in range(n) if primary[i] == biome and labels[i] == c) for c in sorted(set(labels))]
        if any(row):
            out.append(f"| {biome} | " + " | ".join(str(x) or "" for x in row) + " |")
    out += ["", "## Biome signatures", "",
            "Traits most over-represented in each biome (share in biome minus share overall).", ""]
    for biome in data["biomes"]:
        members = [i for i in range(n) if biome in beings[i]["biomes"]]
        if not members:
            out.append(f"- **{biome}**: no beings yet")
            continue
        lift = []
        for f in facets:
            for v in data["facets"][f]:
                inb = sum(v in beings[i].get(f, []) for i in members) / len(members)
                allb = sum(v in b.get(f, []) for b in beings) / n
                if inb > allb:
                    lift.append((inb - allb, f"{f}:{v}"))
        lift.sort(reverse=True)
        out.append(f"- **{biome}** ({len(members)}): " + ", ".join(f"`{t}`" for _, t in lift[:6]))
    out += ["", "## Nearest neighbour", "", "| being | nearest | distance | same biome |", "| --- | --- | --- | --- |"]
    for i in range(n):
        j = min((j for j in range(n) if j != i), key=lambda j: d[i, j])
        same = bool(set(beings[i]["biomes"]) & set(beings[j]["biomes"]))
        out.append(f"| `{ids[i]}` | `{ids[j]}` | {d[i, j]:.2f} | {'yes' if same else 'no'} |")
    (ATLAS / "report.md").write_text("\n".join(out) + "\n")
    print(f"{n} beings, k={k}, silhouette {scores[k]:.2f}, ARI vs biome {ari:.2f}; wrote {ATLAS.relative_to(ROOT)}/")


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--check", action="store_true", help="validate beings.json only")
    data = load(ATLAS / "beings.json")
    if not p.parse_args().check:
        run(data)
    else:
        print(f"beings.json ok: {len(data['beings'])} beings")
