## What kind of change

- [ ] **Data**: rows or values in `assets/recipes/*.json` only (no code, no OpenSpec change)
- [ ] **Rule**: a new rule type; OpenSpec change: `openspec/changes/<name>/`
- [ ] **Other**: code, art, docs

## Summary

## Checks

- [ ] `cargo test -p stubby_recipes` (data and rule PRs)
- [ ] `openspec validate --strict <change>` (rule PRs and spec edits)
