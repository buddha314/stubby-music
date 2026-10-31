## Decisions
Use a seeded integer random stream inside a RandomClock audio processor. Each interval is selected once in sample frames; callback block size and video frames cannot move a tick. The first pulse follows a random resting interval, rather than firing during load. The gate is a 5 ms trigger. It drives the existing Envelope and VCA modules around the existing fundsp square VCO, producing a brief audio-rate click. The gate itself never goes directly to the master.

Publish existing ClockTick events through the engine event rings. The game keeps future tick instants and samples one shared flash envelope using Engine::sample_to_instant. All followers of a clock use the same tick instant. No Bevy timer generates musical pulses. A missing audio device leaves the assets hidden and reports the problem once.

The Blender asset has no visible resting body in-game. Its authored white_flash clip is for asset preview; runtime visibility, scale and a white point light follow the timestamped envelope instead of free-running AnimationPlayer time. The entire GLB is hidden and the light is zero between flashes. The initial interval range is 0.65–2.4 seconds and the visual flash lasts 0.3 seconds. Asset dimensions use metres.

Build into a dedicated scene/collection through the live Blender MCP bridge, preserving the user's open scene. Save only the asset scene and its dependencies to assets-src/blender/lightning_bug.blend. A headless script reproduces it and exports assets/models/lightning_bug.glb. Each bug owns a separate random-clock module, with a seed derived from the world seed and its stable instance ID. Other assets can explicitly follow one of those clock IDs; the bugs are independent by default. The click is distance-attenuated within 3 m; spawn is outside each bug’s hearing range, and sound does not become a sustained tone. Collecting its waveform does not alter its flash.

The clock abstraction is the public `PulseClock` trait, implemented by both `PeriodicClock` and `RandomClock`. Both audio processors step their clock through that common sample interface.

Reuse existing audio modules: square VCO at 2200 Hz → VCA → master, random gate → 0.5 ms attack / 4 ms decay envelope → VCA CV. Each bug has its own voice and clock; distance updates only the VCA gain. No additional DSP processor is introduced.
