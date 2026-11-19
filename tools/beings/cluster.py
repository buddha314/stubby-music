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
        known = {"id", "essence", "kind", "biomes", "status", "sharing", "found_in", "summary", "sources", *facets}
        if b.get("sharing") not in data["sharings"]:
            errors.append(f"{bid}: sharing must be one of {data['sharings']} (restricted beings stay out of the atlas)")
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


def run(data, out, title):
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
    for k in range(2, min(max(3, n // 3), 24) + 1):
        labels = fcluster(tree, k, criterion="maxclust")
        if len(set(labels)) > 1:
            scores[k] = silhouette_score(d, labels, metric="precomputed")
    k = max(scores, key=scores.get)
    labels = fcluster(tree, k, criterion="maxclust")
    ari = adjusted_rand_score(primary, labels)
    n_biomes = len(set(primary))
    ari_at_biomes = adjusted_rand_score(primary, fcluster(tree, n_biomes, criterion="maxclust"))
    # nearest neighbour shares a biome, against the chance of a random other being doing so
    def shares(i, j):
        return bool(set(beings[i]["biomes"]) & set(beings[j]["biomes"]))
    nn_same = sum(shares(i, min((j for j in range(n) if j != i), key=lambda j: d[i, j])) for i in range(n)) / n
    chance = sum(shares(i, j) for i in range(n) for j in range(n) if j != i) / (n * (n - 1))

    xy = MDS(n_components=2, metric="precomputed", init="classical_mds", random_state=SEED).fit_transform(d)
    xy = (xy - xy.mean(0)) / np.abs(xy).max()

    colours = dict(zip(data["biomes"], plt.get_cmap("tab10").colors))

    fig, ax = plt.subplots(figsize=(10, 0.35 * n + 2))
    dendrogram(tree, labels=ids, orientation="right", ax=ax, color_threshold=tree[-k + 1, 2] if k > 1 else None)
    for t in ax.get_yticklabels():
        t.set_color(colours[primary[ids.index(t.get_text())]])
    ax.set_title(f"Beings by traits (average linkage, mean per-facet Jaccard); label colour = biome; k={k}")
    fig.tight_layout()
    fig.savefig(out / "dendrogram.png", dpi=110)

    fig, ax = plt.subplots(figsize=(14, 14) if n > 40 else (9, 9))
    for i, b in enumerate(beings):
        ax.scatter(*xy[i], color=colours[primary[i]], s=120, marker="o" if b["kind"] != "monster" else "X")
        ax.annotate(b["id"], xy[i], textcoords="offset points", xytext=(5, 3), fontsize=6 if n > 40 else 8)
    for biome, c in colours.items():
        ax.scatter([], [], color=c, label=biome)
    ax.legend(loc="lower right", fontsize=8)
    ax.set_title("Being map (MDS of trait distance): the amphitheatre's seating")
    ax.set_aspect("equal")
    fig.tight_layout()
    fig.savefig(out / "map.png", dpi=110)

    layout = {
        "about": "Generated by tools/beings/cluster.py; do not edit. x, y in [-1, 1].",
        "seed": SEED,
        "beings": [
            {"id": ids[i], "x": round(float(xy[i, 0]), 4), "y": round(float(xy[i, 1]), 4),
             "cluster": int(labels[i]), "biome": primary[i]}
            for i in range(n)
        ],
    }
    (out / "layout.json").write_text(json.dumps(layout, indent=2) + "\n")

    # report
    lines = [f"# Being atlas: cluster report{title}", "",
           "Generated by `uv run tools/beings/cluster.py` from `beings.json`; do not edit.", "",
           f"{n} beings ({dict(Counter(b['kind'] for b in beings))}). Distance: mean over facets of the "
           "Jaccard distance between trait sets. Clustering: average linkage, k chosen by silhouette. "
           "Biome is not an input.", "",
           f"- **k = {k}** (silhouette {scores[k]:.2f}; tried " + ", ".join(f"{kk}: {s:.2f}" for kk, s in scores.items()) + ")",
           f"- **Agreement with biome** (adjusted Rand index, primary biome): **{ari:.2f}** "
           "(1 = clusters are exactly the biomes, 0 = no better than chance)",
           f"- **Cut into {n_biomes} clusters**, as many as biomes: adjusted Rand index **{ari_at_biomes:.2f}**",
           f"- **Nearest neighbour shares a biome**: **{nn_same:.0%}** of beings, against **{chance:.0%}** by chance", "",
           "![dendrogram](dendrogram.png)", "", "![map](map.png)", "", "## Clusters", ""]
    facets = list(data["facets"])
    for c in sorted(set(labels)):
        members = [i for i in range(n) if labels[i] == c]
        traits = Counter(f"{f}:{v}" for i in members for f in facets for v in beings[i].get(f, []))
        shared = [t for t, m in traits.most_common() if m >= max(2, (len(members) + 1) // 2)][:8]
        lines.append(f"### Cluster {c} ({len(members)})")
        lines.append("")
        lines.append("- " + ", ".join(f"`{ids[i]}` ({primary[i]})" for i in members))
        lines.append("- shared traits: " + (", ".join(f"`{t}`" for t in shared) or "none"))
        lines.append("")
    lines += ["## Clusters × biomes", "", "| biome | " + " | ".join(f"c{c}" for c in sorted(set(labels))) + " |",
            "| --- |" + " --- |" * len(set(labels))]
    for biome in data["biomes"]:
        row = [sum(1 for i in range(n) if primary[i] == biome and labels[i] == c) for c in sorted(set(labels))]
        if any(row):
            lines.append(f"| {biome} | " + " | ".join(str(x) or "" for x in row) + " |")
    lines += ["", "## Biome signatures", "",
            "Traits most over-represented in each biome (share in biome minus share overall).", ""]
    for biome in data["biomes"]:
        members = [i for i in range(n) if biome in beings[i]["biomes"]]
        if not members:
            lines.append(f"- **{biome}**: no beings yet")
            continue
        lift = []
        for f in facets:
            for v in data["facets"][f]:
                inb = sum(v in beings[i].get(f, []) for i in members) / len(members)
                allb = sum(v in b.get(f, []) for b in beings) / n
                if inb > allb:
                    lift.append((inb - allb, f"{f}:{v}"))
        lift.sort(reverse=True)
        lines.append(f"- **{biome}** ({len(members)}): " + ", ".join(f"`{t}`" for _, t in lift[:6]))
    lines += ["", "## Nearest neighbour", "", "| being | nearest | distance | same biome |", "| --- | --- | --- | --- |"]
    for i in range(n):
        j = min((j for j in range(n) if j != i), key=lambda j: d[i, j])
        same = bool(set(beings[i]["biomes"]) & set(beings[j]["biomes"]))
        lines.append(f"| `{ids[i]}` | `{ids[j]}` | {d[i, j]:.2f} | {'yes' if same else 'no'} |")
    (out / "report.md").write_text("\n".join(lines) + "\n")
    print(f"{n} beings, k={k}, silhouette {scores[k]:.2f}, ARI vs biome {ari:.2f} (at k={n_biomes}: {ari_at_biomes:.2f}), nearest neighbour same biome {nn_same:.0%} vs chance {chance:.0%}; wrote {out.relative_to(ROOT)}/")


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--check", action="store_true", help="validate beings.json only")
    p.add_argument("--where", action="append", default=[], metavar="FACET=VALUE",
                   help="keep only beings with this value (e.g. music=core); repeatable")
    p.add_argument("--drop", default="", metavar="FACETS", help="comma-separated facets to leave out (e.g. place,element)")
    args = p.parse_args()
    data = load(ATLAS / "beings.json")
    if args.check:
        print(f"beings.json ok: {len(data['beings'])} beings")
        sys.exit()
    out, title = ATLAS, ""
    if args.where or args.drop:
        for cond in args.where:
            f, v = cond.split("=", 1)
            if f not in data["facets"] and f not in ("kind", "status", "sharing", "biomes"):
                sys.exit(f"unknown facet {f!r}")
            data["beings"] = [b for b in data["beings"] if v in (b.get(f) if isinstance(b.get(f), list) else [b.get(f)])]
        dropped = [f for f in args.drop.split(",") if f]
        for f in dropped:
            if f not in data["facets"]:
                sys.exit(f"unknown facet {f!r}")
            del data["facets"][f]
        name = "_".join([c.replace("=", "-") for c in args.where] + [f"no-{f}" for f in dropped])
        out = ATLAS / "runs" / name
        out.mkdir(parents=True, exist_ok=True)
        title = f" ({', '.join(args.where + [f'without {f}' for f in dropped])})"
    run(data, out, title)
