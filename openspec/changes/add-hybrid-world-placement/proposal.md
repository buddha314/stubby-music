## Why
The opening is authored in Jackdaw, while the mudflat and skyline are seeded.
We need procedural variation without losing exact designer placements or bringing
back objects a designer removed. Existing Jackdaw scatter tools must be evaluated
and reused before writing another placement system.

## What Changes
- Preserve the authored opening and add optional seeded scatter around it.
- Record the seed, resolved settings, generator revision and authored content revision.
- Support protected areas and persistent move/remove overrides for generated objects.
- Use native prefab placement for interactive resources; keep decorative batching
  where appropriate. Preview and Play consume the same resolved layout.
- Use **seed 17 only** for prototype validation. Replace the obsolete 1,000-seed task.
- Begin with configuration and one decorative scatter rule; resource scattering is
  opt-in after its existing gameplay binding and identity requirements are satisfied.

## Capabilities
### New Capabilities
- `hybrid-world-placement`: deterministic scatter composed with authored placements,
  exclusion masks, designer overrides and versioned placement identity.

## Impact
Game-side configuration/generation glue, the Jackdaw adapter, optional scene metadata
and export/diagnostic tooling. No shared-editor feature changes, engine upgrade,
replacement RNG, custom LOD system, whole-world save framework or new resource design.
Coordinates with `add-opening-region`; retains its authored layout, resource catalog,
starting conditions and deferred ecology. Research and runbook:
`docs/research/seeded-world-generation.md`.
