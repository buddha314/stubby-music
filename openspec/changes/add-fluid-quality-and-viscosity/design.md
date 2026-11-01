## Status
Design intent and implementation guidance only. The source model, streams and
cleaning simulation must be implemented as a later claimed task; do not silently
add balancing constants or a new machine recipe in this documentation change.

## Gameplay decisions
- Muddy water is an initial raw resource, analogous to unprocessed ore.
- Every fluid has viscosity as authoritative gameplay data.
- Cleaning water improves quality and nutrient provision, reduces its effective
  viscosity and lets it move faster through an otherwise identical stream.
- Better nutrient delivery and water quality feed production such as scrub-oak
  lightning-bug mines. The mine stays infinite; delivery and processing limit rate.
- Existing streams carry musical nutrients and pulse at the burbler's BPM, per
  `docs/art/style-guide.md` and `add-mudpit-burbler`. Cleaning has mechanical and
  visual stages; it is more than changing muddy-water opacity.

## Suggested data boundary
Keep pure domain structs independent of Bevy and the realtime audio callback.
Extend the existing resource boundary when implementing it; do not add a separate
fluid engine crate until its ownership is useful.

- `FluidDefinition`: stable ID, validated viscosity and nutrient-quality curves,
  and presentation-stage references. Reserve `muddy_water` for the initial resource;
  decide whether cleaned water is a state or a distinct catalog ID before coding.
- `FluidState`: definition ID, nonnegative quantity, water quality, and explicit
  nutrient availability. Track effective viscosity or derive it from the same
  authored curve. Avoid independent mutable derived values that can disagree.
- `StreamSegment`: inlet/outlet IDs, length, cross-section/capacity and transported
  volume. Derive flow and travel time from segment state and driving conditions.
- `CleaningProcessDefinition`: declared input/output state changes, processing
  limits and eventual costs. Machines and exact recipes remain unauthored.

Choose consistent units at the boundary: metres, seconds, cubic metres, m/s for
velocity, m3/s for volumetric flow, and a documented positive viscosity unit (Pa·s
if using dynamic viscosity). Do not confuse quantity with flow rate or speed.
Numerical curves, initial quality, nutrient scale and base viscosity are undecided.

## Transport implementation guidance
Start with a deterministic directed graph and a simple game-oriented resistance
model rather than a full Navier–Stokes solver. For fixed geometry, driving force
and fluid supply, decreasing viscosity must increase potential flow/velocity.
Finite supply, downstream storage and processing limits can still cap actual flow.
Use a stable simulation timestep, explicit seed where randomness is needed, stable
edge traversal, and a staged flow budget so junction updates do not depend on ECS
iteration order. Clamp transfers to available input and downstream capacity, and
apply the same transferred quantity to both ends; avoid frame-dependent leaks.

For mixing, conserve volume and use explicit volume-weighted quality/nutrient
rules; recompute viscosity from the chosen mixture model instead of averaging
arbitrary material settings. Cleaning's nutrient benefit is a gameplay rule:
represent increased *usable nutrient availability* or an authored nutrient yield.
Do not accidentally generate nutrient units every frame. Decide whether nutrients
are unlocked from muddy content or supplied by another process before introducing
nutrient mass accounting. Exact chemistry and balancing are not implied here.

## Presentation and audio integration
Let Bevy materials read the simulation's quality/viscosity/flow snapshot for
muddy-to-clear stages, colour, opacity, visible flow speed and later staff-like
motifs. Coordinate with Claude's stream art ownership; do not rewrite source models
or the style guide as a substitute for implementing transport.
Fluid transport and cleaning stay off the realtime audio thread. Musical pulses
remain sample-clock driven; do not invent Bevy frame clocks to generate stream
rhythm. Feed quality/nutrient delivery into the future mine-production interface
alongside light quality; keep those inputs separate from palette progression.

## Verification when implementation begins
- Same seed, topology and inputs produce identical state across repeated runs.
- Finite stocks exhaust; infinite source capacity still has a bounded output rate.
- Cleaning increases useful nutrients and lowers effective viscosity.
- Lower viscosity increases speed/flow with other driving conditions held constant.
- Junctions, empty streams, full outputs and mixing conserve transferred volume.
- Fixed simulated duration gives equivalent transfers at different render FPS.
- Better delivered water improves scrub-oak production with light held constant.
