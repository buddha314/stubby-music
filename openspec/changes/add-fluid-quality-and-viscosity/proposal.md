## Why
Muddy water is one of the initial raw resources. Streams transport it, and cleaning
water is a central progression mechanic: cleaner water provides more nutrients
and moves faster. Fluids need explicit viscosity so this affects throughput and
resource production, rather than only their appearance.

## What Changes
- Record muddy water as an initial fluid resource and require viscosity for fluids.
- Require cleaning to improve water quality, nutrient delivery and flow speed.
- Guide a deterministic stream transport model separate from rendering and audio DSP.
- Connect improved water delivery to future scrub-oak bug productivity.
- Requirements and implementation guidance only; no fluid simulation or recipes yet.

## Capabilities
### New Capabilities
- `fluid-resources`: fluid state, viscosity, water cleaning and nutrient transport.

## Impact
Future resource catalog, stream transport, cleaning processes, plant/mine production,
and their Bevy presentation. Coordinate visual stages with the existing art guide
and mudpit-burbler assets; preserve the Bevy-independent audio boundary.
