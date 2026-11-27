#!/usr/bin/env python3
"""Check the agent skills: every .agents/skills/<name>/SKILL.md has frontmatter that
both Claude Code and Codex accept, and .claude/skills links to .agents/skills.

Rules (Agent Skills / Anthropic's authoring guide): `name` is lowercase letters,
digits and hyphens, at most 64 characters, the folder's name, and contains neither
"anthropic" nor "claude"; `description` is present, at most 1024 characters, and
quoted when it contains ": " (unquoted, YAML reads that as a new key).
"""
import json
import pathlib
import re
import sys

repo = pathlib.Path(__file__).resolve().parent.parent
skills = repo / ".agents" / "skills"
errors = []

link = repo / ".claude" / "skills"
if not link.is_symlink() or link.resolve() != skills.resolve():
    errors.append(".claude/skills must be a symlink to ../.agents/skills")

found = sorted(skills.glob("*/SKILL.md"))
if not found:
    errors.append("no skills under .agents/skills")
for path in found:
    where = path.relative_to(repo)
    text = path.read_text()
    match = re.match(r"---\n(.*?)\n---\n", text, re.S)
    if not match:
        errors.append(f"{where}: no frontmatter")
        continue
    fields = dict(
        line.split(": ", 1) for line in match.group(1).splitlines() if ": " in line and not line.startswith(" ")
    )
    name = fields.get("name", "")
    if name != path.parent.name:
        errors.append(f"{where}: name {name!r} is not the folder's name")
    if not re.fullmatch(r"[a-z0-9-]{1,64}", name) or "anthropic" in name or "claude" in name:
        errors.append(f"{where}: name {name!r} breaks the naming rules")
    raw = fields.get("description", "")
    if raw.startswith('"'):
        try:
            description = json.loads(raw)
        except json.JSONDecodeError:
            errors.append(f"{where}: description quoting is broken")
            continue
    else:
        description = raw
        if ": " in raw:
            errors.append(f"{where}: description contains ': ' unquoted (quote it)")
    if not description.strip():
        errors.append(f"{where}: empty description")
    if len(description) > 1024:
        errors.append(f"{where}: description is {len(description)} characters (max 1024)")
    if text.count("\n") > 500:
        errors.append(f"{where}: over 500 lines; move detail into reference files")

if errors:
    print("\n".join(errors))
    sys.exit(1)
print(f"{len(found)} skills OK; .claude/skills links to .agents/skills.")
