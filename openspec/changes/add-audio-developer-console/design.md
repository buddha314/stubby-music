# Design: audio developer console

## Approach

Reuse the existing Rules Designer page, which is served from `127.0.0.1:7878`, and its
existing connection field for the running dev game at `127.0.0.1:15702`. Add a compact
Spatial Audio panel beside the existing live-game panel. The page reads and writes one
registered reflected resource through Bevy Remote Protocol's `world.get_resources` and
`world.insert_resources` methods. No new web server or frontend dependency is needed.

The game's `dev` feature registers an `AudioDeveloperSettings` resource with reflection.
The resource contains a session-only `reflection_wet_mix` value, defaulting to the
listening-review mix of 0.75. A game system forwards changes as an audio `Command`. The audio
engine stores the value and updates existing and newly-created spatial emitters. The
processor reads it from an atomic and applies it to the preallocated wet buffers.

## Behavior

- The slider range is 0.0 to 1.0. Zero is a dry control; 0.75 is the current listening
  default; one is full wet contribution. It is a linear wet gain, not a percentage of a
  normalized equal-power crossfade.
- An explicit Apply action sends the value. Polling the game's state must not overwrite
  an in-progress slider edit.
- Disconnected, non-dev, or unregistered-resource states are shown in the panel rather
  than treated as a successful change.
- The panel labels the control as per-emitter reflections and explains that listener
  reverb is not routed yet.
- The control is ephemeral; restarting the game restores the default.

## Realtime and architecture constraints

- BRP and game systems update state on the Bevy thread. They do not access the Firewheel
  context or AudioNimbus native objects.
- A validated command crosses the existing audio thread request boundary.
- The callback reads one atomic scalar and uses existing buffers; it takes no locks and
  does not allocate.
- Reflection simulation settings and geometry remain worker-owned and unchanged.
- The HTML stays embedded in the existing Rules Designer binary using `include_str!`.

## Validation

- Unit-test command validation/clamping and that the engine applies the current mix to
  existing and future emitters.
- Exercise the developer resource read/write request shape and verify release builds do
  not register developer controls.
- Run the local game and page together, set wet mix to zero and back to 0.75, and confirm
  the page reports the running value. Listening remains a player review.
