## Context

Mines already exist as a domain type (`add-resource-source-model`); the scrub oak is
the first. The burbler is the second, and the first a player configures. Art and game
logic are split between agents, so the asset must expose everything the game animates
or binds as named nodes and profile data.

## Decisions

**A mine that produces a signal, not pickups.** The scrub oak emits finite bugs to
collect. The burbler's output is a clock that is used in place: configuring it is the
interaction, not harvesting. Its `MineDefinition` output is `clock`. (Whether a
burbler can also yield collectable clock items is open.)

**The voice is fixed and minimal.** Clock → envelope → VCA ← VCO, where the
envelope is fixed and hidden and the volume knob is the VCA's level. The player sees
three places, not five. A full VCA and envelope come later as modules in their own
right.

**Tempo belongs to the burbler.** Each burbler has its own BPM, seeded from the world
(so the same seed gives the same burbler), shown and possibly adjustable on its clock
panel. Bubbles burst on the beat and on subdivisions, and wisps rise from the bursts,
so the beat can be seen from a distance.

**Configuration is a HUD, not a structure.** As in Satisfactory, configuring opens a
screen over the game; nothing appears beside the pit. The HUD shows a basic Eurorack
built from the same module assets as everywhere else.

**A round pool with its stream.** The burbler is an organically round pool of mud,
and the stream it feeds leaves it as a channel about 1 m wide between raised, banked
sides; pool and channel are one landform (a smooth union of the two), built up from
the ground so it sits on any terrain. The port is at the stream's mouth.

**Bubbles carry baked clips.** Each bubble has one glTF animation,
`Bubble_<n>_Burble`, a single beat (1 s at speed 1) with the burst at time 0: the
dome swells, bursts, and its ripple and glow spread. The game loops it at speed
BPM / 60 / division and phase-locks it to the clock ticks, so the art owns the shape of
a burble and the game owns its timing.

**Streams are the sequence in space.** A mud stream from the outlet pulses at the
burbler's BPM; following a stream is following the music. Its water is sheet music: a
wavy five-line staff floats on it, and each beat sends a note riding the staff
downstream at the water's speed, so note spacing shows velocity. Each 4 m stream segment is built from 6 reeds (`add-reeds`) and outlined by reeds on its banks,
the conveyor-belt segments (a later change). Extending one works like dragging a
conveyor from a generator: the port arch highlights and a hologram shows the plan. Raw mud is muddy by design; cleaning is
a later constructor's job, taking the water from muddy to clear in stages as processes
are identified, which gives the bleak-to-bloomed arc a production chain.

**The asset exposes, the game animates.** The model ships at rest. Named nodes carry
glTF extras for what the game drives:

| Node | Driven by the game |
| ---- | ------------------ |
| `Bubble_<n>` (extras: `division`) | burst on every beat ÷ `division` |
| `Bubble_<n>_Burble` (animation) | play looping at BPM / 60 / division, burst (time 0) on the tick |
| `Wisp_<n>` (extras: `rise_m`, `life_s`) | rise from a burst and fade, like a jellyfish drifting up |
| `Steam_<n>` (extras: `rise_m`, `life_s`) | drift up slowly |
| `Mote_<n>` | wander slowly; brightness breathes with the beat |
| `Outlet_0` (extras: `stream`, `pulse`) | the stream port at the mouth; +X points along the flow |
| `Outlet_0_Halo` | the upright port arch: peak while a stream is being extended from here (port highlighting), `connected` level and breathing at the BPM once attached, dark otherwise |
| `Interact` (extras: `action: configure`) | where E opens the configuration HUD |

Luminous materials are separate and stably named (`Burbler_Glow`, `Burbler_GlowHalo`,
`Burbler_WispBell`, `Burbler_WispCore`, `Burbler_Mote`, `Burbler_OutletHalo`) so the
game can clone and drive them per instance, as with modules. Rest and peak levels are
in the profile.

**The rig is a kept concept.** `mudpit_burbler_rig` (the pit with a crude/high-tech
rack, probe and conduit) is generated from the same script, `-- rig`, for later use
(for example a built-up burbler). The game does not load it.

## Risks / Trade-offs

- [Alpha-blended halos sort badly in Bevy when many overlap] → keep the halo count
  per burbler small; the bright cores are opaque.
- [Players may expect to patch freely] → the three-place HUD is the first, guided
  step; the general rack (`add-rack-and-spatial-audio`) comes later.
