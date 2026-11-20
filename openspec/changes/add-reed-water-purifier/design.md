## Context

The user's VCV Rack patch is the purifier's patch, saved for reference as
`assets/rack-recipies/reed-water-filter.vcv` (user, 2026-11-18; read from its
`patch.json`, which settles what the screenshot leaves ambiguous). Cable by cable:

| Cable | From | To |
| --- | --- | --- |
| purple | VCO 1 (C2) SAW | MIX IN 1 |
| blue | VCO 2 (C4) TRI | MIX IN 2 |
| yellow | MIX OUT | VCF IN |
| green | VCF HPF | VCA IN |
| blue | clock CLK 1 | VCA CV |
| purple | VCA OUT | SCOPE IN 1 |
| red | SCOPE OUT 1 | outlet (AUDIO L) |

Settings in the sketch:
- VCO 1 at 65.625 Hz, which is C2 (65.41 Hz) 3.6 cents sharp;
- VCO 2 at 261.63 Hz (C4);
- VCF cutoff 531.5 Hz, which is C5 (523.25 Hz) +27 cents, with resonance and drive near
  their defaults;
- mixer: every level at 1 (channels and mix);
- the clock at 120 BPM.

Rack rules already in force: a rack plays only what it shows (AGENTS.md); the rack is data
(`refactor-rack-modules-as-data`); a machine with no clock of its own borrows the nearest
running one (`add-campfire-craft-bench` D7, the `borrowed` place); tempos on the 24 grid.

## Decisions

### D1. The purifier is a machine with a rack, like the burbler
The recipe makes a placeable purifier. Its rack is the sketch, as places:

| Place | Module | Fixed? |
| --- | --- | --- |
| clock | `burbler_clock`, `borrowed` | fixed |
| vco_low | `saw_vco` at C2 | fixed (no saw source yet; user, 2026-11-18) |
| vco_high | `triangle_vco` at C4 | fixed (no triangle source yet) |
| mix | `mixer` | fixed |
| filter | `vcf` | fixed: the recipe's high-pass filter |
| vca | `volume` (its CV/TRIG as gain) | fixed |
| scope | `scope` | fixed: the recipe's oscilloscope |

The recipe takes **reeds (draft: 20), one `high_pass_filter` and one `oscilloscope`**, at
**96 to 144 BPM** (user, 2026-11-18), and the purifier comes with every module in place.
The player's work is the patch and the tuning. When triangle and saw sources are designed,
their places may open to items.

The normalled output is the scope's OUT 1. The comb carries the sketch's seven cables, so
the screen ghosts them like the burbler's.

### D2. Quantized pitch knobs
A knob may declare `quantize: "semitone"` in its module data. Turning it clicks through
equal-tempered semitones (A4 = 440 Hz), and the knob's readout (VCV Rack's knob tooltip,
beside the pointer) shows the note name. Knobs with a grid turn by whole clicks (one per
scroll notch, one per 10 px of drag), so a small move never snaps back to where it was.
Quantization is the rack's (data plus `stubby_rack::Knob::snap`), not the engine's: the
engine still takes Hz. The sketch's VCOs land on C2 and C4. The cutoff is quantized too: 531.5 Hz becomes C5,
523.25 Hz (user, 2026-11-18).

### D3. Target settings
A machine's targets are rules (`machines.json`, moved there by `add-machine-rules`; first
written as the profile's `configure.targets`): a knob in a place at a `note` (`"C2"`, for a
semitone knob) or a `value` (`1` for a gate on). Each rack publishes `RackTargets { met }`;
the patch itself is checked separately, by `RackEngine::sounding`. A target on an empty
place is missed, and an unturned knob counts at its starting value. Each time a rack's
state changes, the log names the targets it misses (`campfire-…: targets missed:
seq.gate_4 at 0`), for tuning. A test checks that every shipped profile's targets name a
real place and knob, and that their values are inside the knob's travel. The game's rule for the machine (here, cleaning) runs
only while the rack sounds and every target is met. Quantized knobs make a target an
exact match, so there is no tolerance to tune. The campfire's `light_pattern` is the same
idea for step gates. Both should move to `targets` once this lands.

### D4. Cleaning: four levels of clear (user, 2026-11-19)
Water has **four levels of clarity: muddy, silty, cloudy, clear** (`LaidStream.clarity`
0, 1/3, 2/3, 1). Every stream leaves its burbler muddy. **The first filter lifts the water
one level**: while the purifier works, the stream below it rises one level above the
water reaching the dam, easing back when it stops. Upstream water is unchanged.
**Filters in sequence lift it further, up to clear** (user, 2026-11-19): three working dams
in a row take muddy to silty, cloudy and clear. A dam fed clear water passes it clear.

Today's growth caps (`mines.json`) put `leafed` at clarity 0.5, which silty (1/3) does not
reach, so the first purifier would grow nothing new. Lower `leafed` to silty in the same
change (a Data PR: budding at muddy, leafed at silty, full bloom at clear), so the first
filter pays off visibly.

### D6. The purifier is a check dam in the stream (user, 2026-11-19)
The purifier **stands in the course of the stream**: a small beaver dam, or check dam, of
reeds and sticks across the bed, the water pooling behind it and spilling over. It is
placed **onto a laid stream segment**, as a stream joins a port: the hologram snaps across
the aimed segment's bed, square to its flow, and is red anywhere else. It splits nothing:
the segment keeps flowing through it, and the clarity changes at the dam. Dismantling the
dam (F) returns the stream to the clarity it had above it.

### D5. Where the modules come from
- Square VCO: from a lightning bug's square wave (exists).
- High-pass filter: one per opening in a rock formation, and one per hollow tree trunk
  (only some trunks are hollow) (user, 2026-11-19; `add-high-pass-sources`).
- Oscilloscope: from a new ripple pool (user, 2026-11-19; `add-ripple-pool`).
- Triangle and saw waves: **no source decided**. AGENTS.md says not to invent one, so
  their VCOs are fixed in the purifier's rack for now (user, 2026-11-18).
- The recipe needs a high-pass filter and an oscilloscope, whose collection does not exist
  yet. **The purifier cannot be made until both can be collected.** Their sources are
  their own changes.
- Mixer and VCA: fixed in the rack (no item).

## Risks / Trade-offs

- Five new engine modules land in `stubby_audio`, which the audio stream is changing
  (`add-rack-and-spatial-audio`). They must be scheduled with that work, not around it.
- The scope needs audio-thread data on the panel: a lock-free ring of recent samples,
  under the realtime rules (no allocation or locks on the callback).
- A seven-cable patch is a big jump from the burbler's two. The ghosts and pods guide it,
  but the first purifier may need fewer places pre-filled (fixed mixer, VCA, clock).

## Open Questions

1. **Reed count** (draft 20).
2. **Triangle and saw sources**, later; their VCOs are fixed until then.
3. ~~Collecting a high-pass filter and an oscilloscope~~: their own changes,
   `add-high-pass-sources` and `add-ripple-pool` (2026-11-19).
4. ~~What clean means~~: four levels; each filter lifts one, in sequence up to clear (D4).
5. ~~Placement~~: a check dam on a laid stream segment (D6).
