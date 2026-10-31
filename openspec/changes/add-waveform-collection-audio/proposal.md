## Why
The 4ms click is too short to recognize the square-wave resource, and successful waveform collection needs audible confirmation. Bugs can yield multiple raw waveform items.

## What Changes
- Use a researched 0.3s flash and square burst.
- Play one listener-level waveform cue per successful collection, independent of removed world voice.
- Set catalog yield to 2 square_wave items per bug; quantities remain configurable.
- Document square-wave asset as in development.

## Capabilities
### New Capabilities
- `waveform-audio`: reusable waveform voice composition and collection feedback.
### Modified Capabilities
- `resource-recipes`: configurable multi-item yield.
- `lightning-bug`: longer square burst on flash.

## Impact
Bevy presentation uses existing square VCO/envelope/VCA/constant modules; no new DSP.
