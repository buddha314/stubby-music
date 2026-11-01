## ADDED Requirements

### Requirement: Burbler is a clock mine
A mudpit burbler SHALL be an infinite mine whose output is a clock. Each burbler SHALL
have its own tempo, derived deterministically from the world seed and its instance, and
SHALL NOT be collectable as a finite pickup.

#### Scenario: Same seed, same tempo
- **WHEN** the same world seed is generated twice
- **THEN** each burbler has the same tempo both times

#### Scenario: Not a pickup
- **WHEN** the player presses E at a burbler
- **THEN** no items are granted and the burbler remains

### Requirement: Configuration screen
Pressing E (interact) while within range of a burbler SHALL open its configuration
screen, a separate HUD over the game view in the manner of a Satisfactory machine's
screen. The range SHALL be the same 3 m reach as E pickups, measured to the burbler's
`Interact` node, and a prompt SHALL show while the player is in range. Escape SHALL
close the screen. While it is open the cursor SHALL be free and player movement SHALL
stop.

#### Scenario: Open and close
- **WHEN** the player is within 3 m of a burbler's `Interact` node and presses E
- **THEN** its configuration screen opens with the cursor released

#### Scenario: Out of range
- **WHEN** the player is farther than 3 m from every burbler and presses E
- **THEN** no configuration screen opens

#### Scenario: Prompt
- **WHEN** the player walks within range of a burbler
- **THEN** the prompt "[E] Configure burbler" shows, in the same style as "[E] Collect lightning bug", and hides again out of range

#### Scenario: Burbler and pickup both in reach
- **WHEN** a lightning bug and a burbler are both within reach
- **THEN** E acts on the nearer of the two
- **WHEN** the player presses Escape
- **THEN** the screen closes and mouse look resumes

### Requirement: Three-place rack
The configuration screen SHALL be a HUD over the game view, like a Satisfactory machine
screen, showing a basic Eurorack with three places: the burbler's clock, built in and
not removable; one VCO slot; and a volume module (a VCA opened directly by TRIG while the gate is high, with no envelope, so the voice stays square), also
built in. The VCO slot SHALL accept a VCO made from a square wave, the lightning bug's
yield: the player inserts it, and one square wave leaves the inventory.

#### Scenario: Square VCO from a bug
- **WHEN** the player holds square-wave items and inserts one into the VCO slot
- **THEN** a Square VCO occupies the slot and one square wave leaves the inventory

#### Scenario: Empty slot
- **WHEN** no VCO is placed
- **THEN** the burbler still keeps time visibly and makes no tone

### Requirement: Settable tempo
The clock SHALL have a BPM knob setting the burbler's BPM, starting at the burbler's
own seeded tempo, and its display SHALL show the BPM the knob sets. The burbler's
bubbles, its stream's notes and the clock's lamp SHALL all follow that BPM.

#### Scenario: Changing tempo
- **WHEN** the player turns the BPM knob from 90 to 120 BPM
- **THEN** the display reads 120, and the bubbles, lamp and stream notes follow 120 BPM

### Requirement: Patched with cables
The voice SHALL be patched by the player with patch cables, taken from the two spare
cables hanging on the rack's cable comb and dragged from jack to jack:
the clock's GATE to the volume's TRIG, and the VCO's OUT to the volume's IN. The
volume's OUT SHALL be normalled to the burbler's outlet. Cables SHALL be fully elastic,
as in VCV Rack: no fixed length, drooping by one tension setting.

#### Scenario: First voice
- **WHEN** a Square VCO is in the slot, GATE is patched to TRIG and OUT to IN
- **THEN** the VCO sounds on every beat at the volume's level, out of the burbler

#### Scenario: Unpatched
- **WHEN** the VCO is in the slot but no cable reaches the volume's IN
- **THEN** the burbler keeps time and makes no tone

#### Scenario: Half patched
- **WHEN** GATE is patched to TRIG but the VCO's OUT is not patched to IN
- **THEN** the burbler makes no tone and its stream shows no notes

### Requirement: The screen guides the patch
Sound SHALL begin only when every cable in the machine's configuration is patched,
and the screen SHALL make that easy to understand: each patch still to make SHALL show
as a ghost cable between its two jacks in the build-ghost colour; the two jacks a
hanging cable is for SHALL pulse in that cable's colour, while other jacks rest; the
hint line SHALL name the next step ("1 of 2 · take the green cable · patch GATE to
TRIG"). Completing the last patch SHALL start the sound and the stream's notes on the
next beat, settle the pods, and show the configuration's `patched_hint`.

#### Scenario: First step
- **WHEN** the VCO and volume are mounted and nothing is patched
- **THEN** ghost cables show GATE to TRIG and OUT to IN, GATE and TRIG pulse green, OUT and IN pulse red, and the hint reads step 1 of 2

#### Scenario: Second step
- **WHEN** the green cable joins GATE to TRIG
- **THEN** its ghost and pods go out, the red cable's remain, and the hint reads step 2 of 2

#### Scenario: Patch complete
- **WHEN** the red cable joins the VCO's OUT to the volume's IN
- **THEN** on the next beat the burbler sounds and its stream's notes begin, the pods rest, and the hint reads that the burbler plays on every beat

#### Scenario: Stretching a cable
- **WHEN** the player drags a cable end anywhere on the screen
- **THEN** the cable stretches to follow it with no length limit

### Requirement: Mud stream outlet
A burbler SHALL have an outlet from which a stream of raw mud can be connected. A
connected stream SHALL pulse at the burbler's BPM once the burbler's voice sounds (before
the patch is complete the stream lies quiet), so the stream presents the played
sequence along its length in space. Raw mud streams SHALL look muddy; cleaning them is
the job of other constructors, not the burbler. As cleaning processes are added, a
stream's water SHALL go from muddy to clear in stages.

The water SHALL resemble sheet music: a five-line staff SHALL run along the stream,
waving gently from bank to bank, at every stage of clarity. Each clock tick SHALL emit
a note at the stream's source that rides the staff downstream at the stream's flow
speed, so the spacing between notes shows how fast the water moves.

#### Scenario: Stream pulses at the burbler's tempo
- **WHEN** a stream is connected to a 90 BPM burbler's outlet
- **THEN** notes enter the stream 1.5 times a second, in time with the clock ticks

#### Scenario: Velocity reads as spacing
- **WHEN** a 90 BPM stream flows at 1.5 m/s
- **THEN** its notes ride the staff 1 m apart, and faster water spreads them further

#### Scenario: Muddy water is still sheet music
- **WHEN** a stream is raw mud
- **THEN** its staff and notes still show on the surface, dimmer than on clear water

### Requirement: Port highlighting
A burbler's stream port SHALL be marked by a soft, upright halo arch over its mouth.
The arch SHALL light while the player is extending a stream from that port, to show
the connection point, in the manner of Satisfactory's port highlighting when a conveyor
is dragged from a generator. Once a stream is connected, the arch SHALL stay softly lit
and breathe at the BPM; otherwise it SHALL be dark.

#### Scenario: Extending a stream
- **WHEN** the player starts extending a stream near a burbler
- **THEN** the burbler's port arch lights as the snap point

#### Scenario: Connected and free
- **WHEN** a stream is connected to the port
- **THEN** the arch stays softly lit, breathing at the BPM
- **WHEN** the stream is removed and the player is not building
- **THEN** the arch is dark

### Requirement: Stream hologram
While the player extends a stream, the planned stream SHALL show as a holographic
preview before it is laid, like a Satisfactory conveyor. The hologram SHALL be its own
asset, separate from the burbler.

#### Scenario: Planning a stream
- **WHEN** the player drags a stream out from the port
- **THEN** a holographic stream follows the planned path and snaps to the port arch

### Requirement: The beat is visible
The burbler's bubbles SHALL burst on its beat and subdivisions, and luminous wisps
SHALL rise from the bursts, driven by the audio clock's ticks, so the tempo can be
seen from a distance. Each bubble SHALL ship a baked one-beat animation
(`Bubble_<n>_Burble`: swell, burst at time 0, ripple) that the game plays looping at
BPM / 60 / division, phase-locked to the clock ticks.

#### Scenario: Bubble clips present
- **WHEN** the burbler glb is loaded
- **THEN** it has one animation per bubble, each one second long with the burst at 0

#### Scenario: Seeing the tempo
- **WHEN** a burbler at 90 BPM is in view
- **THEN** its main bubble bursts 1.5 times a second, in time with its clock ticks

### Requirement: Burbler asset
The burbler model SHALL be generated by `tools/blender/create_burbler.py` and expose
the game-driven nodes and materials listed in the design, with a profile at
`assets/machines/mudpit_burbler/asset.json`. It SHALL be an organically round pool
of mud with a stream about 1 m wide leaving it between banked sides, with no built
structure, lit with soft jellyfish- and fairy-like
luminescence, not LED-like points.

#### Scenario: Asset contract
- **WHEN** the burbler glb is loaded
- **THEN** it contains the bubbles, ripples, wisps, steam, port node, port arch and interact node with their extras
