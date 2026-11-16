## ADDED Requirements

### Requirement: A forensic flashlight on B
B SHALL bring the flashlight into the hand and switch it on, and B again SHALL put it away
and switch it off. Bringing up another hand item SHALL put the flashlight away. B SHALL do
nothing while a screen has the controls.

#### Scenario: Switch it on and off
- **WHEN** the player presses B with an empty hand, then B again
- **THEN** the flashlight is held and casting its beam, then put away and dark

### Requirement: The light is a spinning disco ball
While the flashlight is held, its light SHALL appear as a mirror ball that spins and throws
a ring of narrow violet and cyan beams, without shadows, whose spots sweep the ground as it
turns. When switched on, the ball SHALL appear above the player's head to the left, about
30 cm above the head, where a comfortable glance up and left finds it, and SHALL stay at that
place in the world while on, not following the player's view (user, 2026-11-16). Its size, place, spin and beams SHALL come from
`assets/ui/flashlight.json`.

#### Scenario: The spots sweep
- **WHEN** the player holds the flashlight still for a few seconds
- **THEN** the ball turns and its spots move across the ground

#### Scenario: The ball stays put
- **WHEN** the player switches the flashlight on and then turns to look the other way
- **THEN** the ball stays where it appeared, above and to the left of where they stood

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
