# Streams

Streams carry the music: mud flows from a burbler's mouth and pulses at its BPM, so a
stream is the played sequence laid out in space. As cleaning processes are identified
the water goes from muddy to clear, and a clear stream shows the staff lines on its
bed: a clear stream is staff music. Requirements: `openspec/changes/add-mudpit-burbler/`.

## Stream segment (`tools/blender/create_stream.py`)

![muddy, silty, clear](stream_segment/preview_stages.png)

A straight 4 m length of the burbler's own channel: 1 m of water between banked
sides, built up from the ground. Its bank profile and water height are imported from
`create_burbler.py`, so a segment meets the burbler's mouth exactly.

- **Chaining:** flow runs along +X; the origin is the inlet's centre on the ground.
  `Port_in` (x = 0) and `Port_out` (x = 4) each carry an upright halo arch
  (`Port_<end>_Halo`) for port highlighting while a stream is being extended. The ends
  are square and noise fades out toward them, so segments tile.
- **Bending:** 24 subdivisions along the length, for the game to bend it along a path.
- **Clarity:** one water material, `Stream_Water`, over a pebbled bed. `asset.json`
  lists its look at each stage (`muddy` 0, `silty` 0.5, `clear` 1: sRGB colour,
  roughness, alpha) for the game to blend. It ships muddy (alpha 0.98, so it exports
  alpha-blended). The `Staff` node's five luminous lines lie on the bed and show from
  clarity 0.6.
- **Pulse:** `Pulse_0` slides downstream (+X) on each beat. It shows a band of light
  across the surface while muddy (`Pulse_0_Band`) and a note head riding the staff once
  clear (`Pulse_0_Note`, from clarity 0.5).

```sh
~/.local/bin/blender -b --factory-startup -P tools/blender/create_stream.py
```

Still to come: the stream hologram (the planned stream, shown while extending) and
reeds, the conveyor-belt segments streams run along.
