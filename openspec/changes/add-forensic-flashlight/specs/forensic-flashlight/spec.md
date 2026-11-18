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
turns. When switched on, the ball SHALL appear in view so the player knows it is there: at
about eyebrow level, about 1.1 m away and 35 degrees to the left, at the left edge of the view
with about a third of it out of frame (user, 2026-11-17, replacing "above the head" of
2026-11-16). It SHALL stay at that place in the world while on, not following the player's view. Its size, place, spin and beams SHALL come from
`assets/ui/flashlight.json`.

#### Scenario: It appears in view
- **WHEN** the player, looking level, switches the disco light on
- **THEN** the ball is at the left edge of the view at eyebrow level, about a third of it out of frame

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

### Requirement: Collectibles the beams touch show as holograms
Every available collectible within range that a beam passes over SHALL show a holographic
shell over its body and a holographic beacon above it, visible through grass and reeds,
flaring as the beam crosses it and fading after. Collectibles SHALL be found through the
pickup contract, without naming asset kinds.

#### Scenario: A shell in a beam
- **WHEN** a beam crosses a snail shell 10 m away
- **THEN** a holographic beacon stands over it and its shape glows holographically

#### Scenario: A bug between flashes
- **WHEN** a beam crosses a lightning bug while it is dark
- **THEN** its beacon still shows where it is

#### Scenario: After the beam passes
- **WHEN** no beam has crossed a collectible for a few seconds
- **THEN** its hologram has faded out and its overlay is removed
