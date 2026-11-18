## ADDED Requirements

### Requirement: A disco light on B
B SHALL switch the disco light on, and B again SHALL switch it off. It SHALL NOT be a hand
item: no flashlight model is shown, and whatever the hand holds (the field recorder) stays
in it. B SHALL do nothing while a screen has the controls.

#### Scenario: Switch it on and off
- **WHEN** the player, holding the field recorder, presses B, then B again
- **THEN** the disco ball appears casting its beams with the recorder still in hand, then it is gone and dark

#### Scenario: Not while a screen is open
- **WHEN** the player presses B with the inventory open
- **THEN** the light does not change

### Requirement: The light is a spinning disco ball
While the disco light is on, it SHALL appear as a mirror ball that spins and throws
a ring of narrow violet and cyan beams, without shadows, whose spots sweep the ground as it
turns. When switched on, the ball SHALL appear in view so the player knows it is there: about
0.95 m away, 33 degrees to the left and 17 degrees up, so that in a level view it sits in the
upper-left corner, cut by the left and top edges (the user's reference screenshot,
2026-11-17, replacing "above the head" of 2026-11-16). It SHALL stay at that place in the world while on, not following the player's view. Its size, place, spin and beams SHALL come from
`assets/ui/flashlight.json`.

#### Scenario: It appears in view
- **WHEN** the player, looking level, switches the disco light on
- **THEN** the ball is in the upper-left corner of the view, partly out of frame at the left and top

#### Scenario: The spots sweep
- **WHEN** the player holds still with the disco light on for a few seconds
- **THEN** the ball turns and its spots move across the ground

#### Scenario: The ball stays put
- **WHEN** the player switches the disco light on and then turns to look the other way
- **THEN** the ball stays where it appeared, above and to the left of where they stood

### Requirement: Near a clock the ball keeps time
Within reach of a clock (the reach is data in `assets/ui/flashlight.json`), the ball SHALL
pulse slightly in time with the nearest clock's BPM: it swells a little on each beat at
the instant the beat is heard (the audio clock's predicted instant, never frame time) and
settles before the next. Away from every clock it SHALL hold still.

#### Scenario: Beside a burbler
- **WHEN** the disco light is on within reach of a burbler at 90 BPM
- **THEN** the ball swells 90 times a minute, each swell on a heard beat

#### Scenario: Out of reach
- **WHEN** no clock is within reach
- **THEN** the ball does not pulse

### Requirement: Collectibles in reach show as holograms
While the disco light is on, every available collectible within reach of the player SHALL
show a holographic shell over its body and a holographic beacon above it, steadily and
visible through grass and reeds; the beams are its look, not what reveals (user,
2026-11-18: revealing only what a sweeping beam crossed made pickups blink). A revealed
collectible SHALL be marked `stubby_core::Spotlit`, and its owner SHALL keep it in view (a
lightning bug holds its body lit between flashes). Collectibles SHALL be found through the
pickup contract, without naming asset kinds.

#### Scenario: A shell in reach
- **WHEN** the light is on and a snail shell lies 10 m from the player
- **THEN** a holographic beacon stands over it and its shape glows holographically, without blinking

#### Scenario: A bug between flashes
- **WHEN** the light is on and a lightning bug in reach is between flashes
- **THEN** its body stays in view, with its hologram and beacon

#### Scenario: Walking away or switching off
- **WHEN** the player walks out of reach of a collectible, or switches the light off
- **THEN** its hologram fades out within a few seconds and its overlay is removed
