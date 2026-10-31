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

**Streams are the sequence in space.** A mud stream from the outlet pulses at the
burbler's BPM; following a stream is following the music. Streams run along reeds,
the conveyor-belt segments (a later change). Raw mud is muddy by design; cleaning is
a later constructor's job, which gives the bleak-to-bloomed arc a production chain.

**The asset exposes, the game animates.** The model ships at rest. Named nodes carry
glTF extras for what the game drives:

| Node | Driven by the game |
| ---- | ------------------ |
| `Bubble_<n>` (extras: `division`) | swell and burst on every beat ÷ `division` |
| `Bubble_<n>_Ripple` | spread outward and fade after each burst |
| `Wisp_<n>` (extras: `rise_m`, `life_s`) | rise from a burst and fade, like a jellyfish drifting up |
| `Steam_<n>` (extras: `rise_m`, `life_s`) | drift up slowly |
| `Mote_<n>` | wander slowly; brightness breathes with the beat |
| `Outlet_0` (extras: `stream`, `pulse`) | where a mud stream starts; +X points along the flow |
| `Outlet_0_Halo` (extras: `connected_only`) | lit only while something is connected; breathes at the BPM |
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
