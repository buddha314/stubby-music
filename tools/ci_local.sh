#!/usr/bin/env bash
# Run the same checks as .github/workflows/stubby-audio.yml, locally, before pushing.
# CI minutes cost money: a push should only ever confirm what this already passed.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
step() { printf '\n== %s\n' "$*"; }
step "LOD authoring policy"; python3 tools/jackdaw_lods.py --check
python3 tools/test_jackdaw_lods.py
step "Runtime asset references"; python3 tools/check_asset_layout.py
step "Version consistency";        python3 tools/check_engine_pins.py
step "Version-check regression";   python3 tools/test_engine_pins.py
step "cargo-deny: layering, licenses, advisories, sources (deny.toml)"
command -v cargo-deny >/dev/null || { echo "install it: cargo install cargo-deny --locked --version 0.20.2"; exit 1; }
cargo deny --log-level error check
step "Active Bevy graph"
tree="$(mktemp)"; trap 'rm -f "$tree"' EXIT
cargo tree --color never --workspace --locked -e normal,build,dev --prefix none --format '{p}' > "$tree"
python3 tools/check_engine_pins.py --tree "$tree"
step "Format";  cargo fmt -p stubby_audio --check
step "Clippy";  cargo clippy -p stubby_audio --all-targets --locked -- -D warnings
step "Test";    cargo test -p stubby_audio -p stubby_economy --locked
step "stubby_audio stands alone"
deps=$(cargo tree -p stubby_audio -e normal,build --prefix none --format '{p}' --locked \
  | grep -F "$PWD" | grep -v '^stubby_audio ' || true)
if [ -n "$deps" ]; then echo "stubby_audio depends on workspace crates:"; echo "$deps"; exit 1; fi
step "Cores never reach the Bevy engine"
for core in stubby_audio stubby_rack stubby_recipes stubby_economy; do
  edges=normal
  forbidden='^bevy( |_app |_ecs |_render |_asset )'
  if [ "$core" = stubby_economy ]; then
    edges=normal,build,dev
    forbidden='^bevy([_ ]|$)'
  fi
  if cargo tree -p "$core" -e "$edges" --prefix none --locked | grep -E "$forbidden"; then
    echo "$core must not depend on the Bevy engine"; exit 1
  fi
done
# Not in GitHub CI (no editor there): the project as Jackdaw sees it. This is what
# finds the game plugin the editor opens; a compiling build can still fail it.
if command -v jd >/dev/null; then
  step "Jackdaw doctor";  jd doctor --project .
else
  echo "(jd not found: skipped the Jackdaw doctor check)"
fi
printf '\nAll CI checks passed locally.\n'
