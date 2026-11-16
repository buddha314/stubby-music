## 1. Research and agreement

- [x] 1.1 Compare Godot layout and pinned Bevy/Jackdaw behavior (claimed: Codex).
- [x] 1.2 Record sources, proposed paths and import requirements (claimed: Codex).
- [x] 1.3 Reconcile overlapping native-path requirements and AGENTS.md layout with art ownership before implementation. (claimed: Codex).

## 2. Asset migration

- [x] 2.1 Inventory all model, metadata, prefab and profile references; prepare old-to-new path manifest. (claimed: Codex).
- [x] 2.2 Pilot co-located oak variants and burbler with existing metadata and their existing variant master prefabs. (claimed: Codex).
- [x] 2.3 Adapt exporters, LOD discovery/generation policies and reference-camera threshold handling. (claimed: Codex).
- [x] 2.4 Move remaining families and update code, profiles, scenes, tests and package references atomically; preserve IDs and scene selectors. (claimed: Codex).
- [x] 2.5 Publish Bevy asset authoring guide and update repository layout documentation. (claimed: Codex).

## 3. Verification

- [x] 3.1 Validate references and metadata, with missing-level negative coverage and existing prefab tests. (claimed: Codex).
- [x] 3.2 Verify clean-cache standalone import plus state/animation bindings across levels. (claimed: Codex).
- [x] 3.3 Build stubby_jackdaw and review near/far switching and return through tools/jackdaw_timed_test.py; verify scale/FOV policy and final-culling metadata are unchanged. (claimed: Codex).
- [x] 3.4 Run strict OpenSpec validation and relevant local CI checks for the local implementation. (claimed: Codex).

Validation evidence and limits: [migration validation](../../../docs/research/lod-asset-layout-validation.md).
