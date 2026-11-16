## Decisions

Keep Jackdaw's existing screen-height LOD mechanism. The authoring tool converts reference metres at 45-degree vertical FOV and unit placement scale using each metadata file's model size. A placed shell is scaled 3x; its actual reference distances are therefore three times the unit-scale authoring distances. Explicit final-cull distances replace the blanket 1% cutoff. Short fractional crossfade bands use Jackdaw's LodFade::CrossFade rather than game alpha animation.

Put reference tuning inputs in tools/assets/lod_policy.json (authoring configuration, not runtime rules). Include all currently configured families and the shell. Ordinary loading does not regenerate metadata. Preserve hand-authored node names, existing generated ratios and animation bindings. Read flora handover distances from their existing family profiles to avoid an independent conflicting setting.

The adapter's --lod-review writes screenshots and JSONL evidence while moving one tripod camera. It samples actual LiveLevels/LodSwitches and visible render parts; it does not choose levels itself. Use the production opening assets, adding temporary representative brush/stump fixtures for those otherwise absent. Remove player movement from the review camera so tripod shots can inspect tiny shells without the character controller raising the camera. Review dormant/grown oak and standing/harvested reeds as separate states. Additional checks vary placement scale and vertical FOV.

Only tools/jackdaw_timed_test.py may launch graphical diagnostics, with its independent 20-second kill timer. Build before launching. Compare baseline and tuned shots, and record limitations: one bounded seed/view sequence is not a GPU performance benchmark or a guarantee across all materials, variants and cameras.

## Scope of this first pass

See [observed results and engine findings](../../../docs/research/lod-tuning-first-pass.md). The live-root-scale refresh and ineffective generated reduction findings are recorded follow-ups; this pass changes fade/cull settings and shell handover, not Jackdaw internals or authored geometry.
