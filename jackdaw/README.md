# Desert opening in Jackdaw

Use the shared source checkout `/home/buddha/src/jackdaw`. This folder is a game
project, not an editor fork. Both editor and CLI come from that shared checkout.
The installed `~/.cargo/bin/jd` is older and must not be used for this project.

The shared checkout uses `https://github.com/buddha314/jackdaw.git` as development
`origin`, with `https://github.com/jbuehler23/jackdaw.git` retained as `upstream`.
Publish and use fixes from our fork without waiting for upstream PRs to merge.

From the repository root:

```sh
tools/jackdaw.sh prepare   # regenerate floor preview; build the matching shared CLI
tools/jackdaw.sh doctor
tools/jackdaw.sh build     # build the game and extract component schema
tools/jackdaw.sh open      # open the shared editor on assets/scene.bsn
```

The shared editor executable is
`/home/buddha/src/jackdaw/target/debug/jackdaw`. Its SDK/toolchain is Rust 1.99.0,
and it targets Bevy main at `c9c397ba1a73e6cfffe957318e0359330b701865`.
This project's lockfile must retain that exact commit. The main game workspace
continues to use Bevy 0.20.1; the isolated adapter compiles its generator source
against the editor's Bevy, with `JackdawPlugin` and `maybe_windowless` for Play.
The runtime dependency points directly to the shared Jackdaw checkout.

The edit viewport loads `generated/mudflat.glb` through `GltfSource` (a raw
`SceneRoot` is not registered in this editor). Re-run `prepare` after changing the
floor generator. These files are reproducible and ignored by Git. The preview
uses editor lighting; Play generates the full desert sky, haze and mountains.
The preview floor and light carry `HiddenInGame`, so they do not duplicate the
procedural scene in Play. Other entities authored in `scene.bsn` load in Play.
Moving the preview proxy does not change the procedural generator's parameters.

The shared Jackdaw Play fix excludes process-local light-probe components from
PIE snapshots and projection. Without it, a streamed generated environment map
is rebuilt with a null image handle in the editor, causing storage-format and
texture-usage GPU validation errors. Rebuild the shared editor after changing
its source; no game-specific editor binary is used.

Verified on 2026-10-31: `jd doctor` passes; `jd build` extracts 486 registered
components; the floor loads in the edit viewport; embedded Play connects and
streams the desert view; Stop restores authoring. The shared Jackdaw regression
and all 23 PIE projection tests pass. The game workspace tests and both game
and adapter Clippy checks pass.

The lightning bug models are shared through `assets/models`, a symlink into the
main runtime asset folder. Play compiles the shared pulse code and uses the
standalone `stubby_audio` crate through its own public API. The edit viewport's
floor proxy does not animate the creatures; Play does.
Audio: Play runs the game's audio engine too. The run configuration
"Desert opening + audio demo" starts a test patch (the opening's square wave plus an
audible clock); in either configuration F8 toggles it. Click into the Play panel
first, so keys reach the game rather than the editor.

## Live listen

Choose **Live listen: clock + square** in the editor's run configurations, then
Play. It places a clock and square VCO near spawn and patches that exact clock
into the VCO's envelope/VCA, so the lamp and audible low square bursts share
one source of timing. The normal Desert opening configuration remains silent
until discovery. Leave F8's independent demo off during this check.

Standalone equivalent: `cargo run -p stubby_game -- --seed=42 --module-demo`.
At the default 120 BPM, listen for two bursts per second and watch the clock lamp.
Stop Play to stop its audio; play again to verify it starts cleanly. Audio uses the
game child's default output device, not an editor audio bridge. This check does
not establish the separate 10ms on-screen timing requirement.

## Bounded dusk and replenishment review

Build the adapter, then run:

```sh
python3 tools/jackdaw_timed_test.py --review --seconds 20 --refresh 60 --name replenishment-review
```

The watchdog stops the editor and game within 20 seconds. The opt-in review starts
five game minutes before sunset at 64×, collects one bug, waits for the next night,
collects it again, and exercises slower/faster/reset controls. It writes screenshots
and `events.jsonl` under `/tmp/stubby-replenishment-review/`. Look for a successful
`complete` event and inventory 0 → 2 → 4, with no inventory change on replenishment.
Use the watchdog when testing the review run configuration.

Normal play uses `[`/`]` for slower/faster simulation and `\` for 1×.
Music pitch, clock tempo and flash envelopes retain their audio timing.
