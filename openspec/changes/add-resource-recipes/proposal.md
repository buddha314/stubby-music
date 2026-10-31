## Why
Lightning bugs are a game resource: they provide square-wave items that players can use as recipe ingredients. Establish a small general recipe model rather than embedding crafting rules in the creature code.

## What Changes
- Add data-defined item, resource yield and recipe definitions.
- Add engine-independent inventory and atomic crafting operations with clear errors for missing ingredients and invalid recipes.
- Make lightning bugs harvestable once per instance and add their square-wave yield to player inventory.
- Keep recipe data independent of DSP nodes; consuming a waveform item does not automatically connect an audible oscillator.

## Capabilities
### New Capabilities
- `resource-recipes`: resource yields, inventory, recipe definitions and crafting checks.

## Impact
New pure Rust recipe crate and asset data, plus game interaction and documentation. Waveforms are raw factory resources, analogous to coal. The recipe book records the square-wave primitive and its VCO/LFO/gate roles; production combinations can be added later without inventing a finished-module recipe now.
