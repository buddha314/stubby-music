#!/usr/bin/env bash
# Run the same checks as .github/workflows/stubby-audio.yml, locally, before pushing.
# CI minutes cost money: a push should only ever confirm what this already passed.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
step() { printf '\n== %s\n' "$*"; }
step "Version consistency";        python3 tools/check_jackdaw_versions.py
step "Version-check regression";   python3 tools/test_jackdaw_versions.py
step "Layer rules";                python3 tools/test_check_layers.py
step "Layer check";                python3 tools/check_layers.py
step "Active Bevy graph"
tree="$(mktemp)"; trap 'rm -f "$tree"' EXIT
cargo tree --color never --workspace --locked -e normal,build,dev --prefix none --format '{p}' > "$tree"
python3 tools/check_jackdaw_versions.py --tree "$tree"
step "Format";  cargo fmt -p stubby_audio --check
step "Clippy";  cargo clippy -p stubby_audio --all-targets --locked -- -D warnings
step "Test";    cargo test -p stubby_audio --locked
step "stubby_audio stands alone"
deps=$(cargo tree -p stubby_audio -e normal,build --prefix none --format '{p}' --locked \
  | grep -F "$PWD" | grep -v '^stubby_audio ' || true)
if [ -n "$deps" ]; then echo "stubby_audio depends on workspace crates:"; echo "$deps"; exit 1; fi
if cargo tree -p stubby_audio -e normal --prefix none --locked | grep -E '^bevy( |_app |_ecs |_render |_asset )'; then
  echo "stubby_audio must not depend on the Bevy engine"; exit 1
fi
# Not in GitHub CI (no editor there): the project as Jackdaw sees it. This is what
# finds the game plugin the editor opens; a compiling build can still fail it.
if command -v jd >/dev/null; then
  step "Jackdaw doctor";  jd doctor --project .
else
  echo "(jd not found: skipped the Jackdaw doctor check)"
fi
printf '\nAll CI checks passed locally.\n'
