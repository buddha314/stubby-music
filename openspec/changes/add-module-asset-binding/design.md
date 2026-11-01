## Context

The engine (`add-audio-engine`) runs modules on its own thread behind the `Synth`
resource and reports clock ticks with predicted instants. The module assets
(`assets/modules/`) carry named nodes and a generated `asset.json` profile. The
lightning-bug work added a pulse bus that turns engine ticks into flash envelopes.
PR #17 joined these together.

## Goals / Non-Goals

**Goals:** a placed module asset is its engine module; what the player sees (lights,
display, knobs) follows what the engine does, and keeps time with it.

**Non-Goals:** cables between modules (patching gameplay), placing modules in the
generated world (`add-desert-opening` 2.5), and module UI interaction (turning knobs
with the mouse).

## Decisions

**D1: Profiles generated with the model, embedded in the game.** The Blender generator
writes `asset.json` beside the model, so names and ranges can't drift apart. The game
embeds the profiles with `include_str!`, as the lightning bug does, so the game and
the Jackdaw adapter (which compiles the game's library against Bevy `main`) read the
same data.

**D2: Logic separate from visuals.** `ModuleLights` holds light levels (0..1)
computed from the engine each frame; a separate pass applies them to materials,
segment visibility, knob rotation and the lamp's point light. That keeps the timing
logic testable headlessly, with no renderer or assets.

**D3: Flashes come from the shared pulse bus.** Engine ticks already reach
`PulseBus` (shared through `PulseBusPlugin` with the lightning bugs). A clock module's
lamp reads `PulseBus::level` for its module ID with the profile's envelope, so flashes
are scheduled from each tick's predicted instant, not from the frame the tick arrived
in.

**D4: Per-instance materials.** When a module's scene spawns, its glowing materials
(found by `GltfMaterialName`, or under `Light_*` nodes) are cloned for that instance.
Materials are exported at peak strength and scaled between rest and peak.

**D5: One shared module ID allocator (open).** Today four systems pick IDs from
hand-picked ranges (recorded in `AGENTS.md`). Replace them with a single `Resource`
handing out IDs, used by module assets, lightning bugs, the audio demo and collection
cues.

## Risks / Trade-offs

- [VCO LEDs show only "live" until the rack publishes meters] → task 2.2, after
  `add-rack-and-spatial-audio` 3.2.
- [Saw and triangle make no sound] → task 2.3, when the engine gains those oscillators.
- [Hand-picked IDs can collide as content grows] → task 2.1, the allocator.

## Live listen verification (task 2.5 / add-audio-engine 5.5)

`--module-demo` is an opt-in audition. Placed clock 8001 drives both its lamp
and an AD envelope (2ms attack, 120ms decay); placed square VCO 8002 feeds a
VCA at gain 0.15, controlled by that envelope. Auxiliary IDs 8003–8004 remain
in the reserved showcase range until the shared allocator lands. Patch only
once both source modules have been added; remove the auxiliary nodes if either
source is removed. Normal startup stays unpatched. Do not combine the showcase
with the independent F8/--audio-demo patch when checking timing.

The Jackdaw run config is `Live listen: clock + square`. The same generator
and graph code are compiled into the standalone game and Play adapter. Machine
checks render bursts and verify exact sample tick spacing; the on-screen 10ms
measurement remains task 2.4 rather than being inferred from a listening check.
