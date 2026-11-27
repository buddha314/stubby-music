#!/usr/bin/env bash
# Copy the shared engine tools from the template into this game, unchanged.
#
#   tools/update_engine_tools.sh [TEMPLATE_DIR]   default: $ENGINE_TEMPLATE or ../us-outpost-31
#
# These files are identical in every game made from VerdantAI/us-outpost-31; improve
# them there, then run this in each game. Game-specific steps belong in
# tools/game_hooks.sh, which this never touches. Review the diff and commit it.
set -euo pipefail
repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
template="$(cd -- "${1:-${ENGINE_TEMPLATE:-${repo_dir}/../us-outpost-31}}" && pwd)"
shared=(
    tools/jackdaw.sh
    tools/engine_sync.py
    tools/check_engine_pins.py
    tools/test_engine_pins.py
    tools/bevy_main_forks.py
    tools/timed_play.py
    tools/update_engine_tools.sh
    tools/check_skills.py
    docs/ENGINE-UPDATES.md
    # Stack-level agent skills (true of every game on this stack; AGENTS.md *Agents*).
    .agents/skills/updating-the-engine/SKILL.md
    .agents/skills/adding-a-dependency/SKILL.md
    .agents/skills/updating-from-the-template/SKILL.md
    .agents/skills/sharing-tool-changes/SKILL.md
)
if [[ "$template" == "$repo_dir" ]]; then
    echo "This is the template itself; nothing to copy." >&2
    exit 0
fi
git -C "$template" diff --quiet HEAD -- "${shared[@]}" \
    || echo "warning: $template has uncommitted changes to the shared tools" >&2
for file in "${shared[@]}"; do
    install -D -m "$(stat -c %a "${template}/${file}")" "${template}/${file}" "${repo_dir}/${file}"
done
echo "Copied ${#shared[@]} shared files from ${template} ($(git -C "$template" rev-parse --short HEAD))."
git -C "$repo_dir" status --short -- "${shared[@]}"
python3 "${repo_dir}/tools/test_engine_pins.py" >/dev/null 2>&1 && echo "Pin check tests pass here." \
    || { echo "Pin check tests fail in this game; see python3 tools/test_engine_pins.py" >&2; exit 1; }
