## Why

The first recipe at the campfire is a **reed water purifier** (user, 2026-11-18). Streams
leave the burbler as raw mud, and until something cleans them they stay mud
(`LaidStream.clarity` 0), so trees never grow past budding. The purifier is the first
machine that cleans water, through the rule the game already states: **high-pass
filters separate out pure liquid** (AGENTS.md, "Mud streams carry the music").

The user patched the purifier in VCV Rack (2026-11-18), saved as
`assets/rack-recipies/reed-water-filter.vcv`, which this change copies:

- **VCO 1**, saw out at **C2** (65.625 Hz in the patch, 65.41 Hz quantized), into
  **mixer** channel 1;
- **VCO 2**, triangle out at **C4** (261.63 Hz), into mixer channel 2;
- **mixer** MIX out into a **VCF**, cutoff **531.5 Hz**, its **HPF** out into the **VCA** IN;
- **the clock** into the VCA's CV: the sketch uses a clock module at 120 BPM, but in the
  game the clock comes from the stream or burbler (a `borrowed` place, as on the campfire);
- **VCA** OUT through the **scope** (IN 1, OUT 1 through) to the outlet.

Musically, a high-pass at about C5 removes the C2 saw's fundamental and its low
harmonics, and the C4 triangle's fundamental, and keeps the saw's bright upper harmonics
(its 8th, at C5, and above) with the triangle's faint overtones: long waves out, short
waves through. That is
the purifier's job in the world: mud in, clear water out.

The VCOs should be **quantized** to notes, "to make it easier" (user): a VCO's frequency
knob clicks through semitones and its display names the note, so the player can set C2
and C4 exactly. This adds new rules for setting a rack: knobs that click to a musical
grid, and a machine that works only when its knobs are at the settings it asks for.

## What Changes

- **A recipe**: `reed_water_purifier`, made at the campfire from **reeds, a high-pass
  filter and an oscilloscope**, at **96 to 144 BPM** (user, 2026-11-18): the first recipe
  with a BPM range, so the camp's clock must be in it. The purifier comes with its
  modules in place.
- **A placeable machine**: the purifier stands in a stream (a bed of reeds, art) and has
  a rack (`configure` block) shaped like the sketch, every place fixed: a saw VCO, a
  triangle VCO, a mixer, a VCF, a VCA, a scope, and a borrowed clock (the stream's or
  burbler's). The player patches the seven cables and tunes it.
- **Rack settings rules** (`bevy_stubby_rack`, `stubby_rack`, module data):
  - **Quantized knobs**: a knob may click to a pitch grid (semitones, equal temperament,
    A4 = 440 Hz), and its display shows the note name (`C2`, `C4`). The VCO frequency
    knobs and the VCF cutoff use it (the sketch's 531.5 Hz becomes C5, 523.25 Hz; user,
    2026-11-18).
  - **Target settings**: a machine profile may name the settings its work needs (each
    VCO's note, the cutoff, the patch), as the campfire names its `light_pattern`. The
    purifier cleans only while the rack sounds with those settings.
- **Engine modules** (`stubby_audio`, the audio work stream's crate): triangle and saw
  VCOs (fundsp oscillators), a four-channel mixer with per-channel levels, a state-variable
  filter with LPF and HPF outputs and a cutoff in Hz, and a scope that passes its input
  through and publishes a short waveform for its panel.
- **Panel art** (Claude): mixer, VCF and scope panels in the module kit; the purifier
  model, its hologram, icon and LODs. The triangle and saw VCO panels already exist.
- **Cleaning**: a sounding purifier with its targets met raises the clarity of the
  stream downstream of it (mud to silty, at least), which lets watered oaks grow further
  (`mines.json` growth caps). Rates live in `add-fluid-quality-and-viscosity`.

## Impact

- New capability `reed-water-purifier`; new capability `rack-settings` (quantized knobs,
  target settings) used by every rack.
- Crates: `stubby_audio` (new module kinds: coordinate with `add-rack-and-spatial-audio`),
  `stubby_rack` (knob quantization, targets), `bevy_stubby_rack` (note display, scope
  panel), `stubby_streams` (clarity downstream of a machine), `stubby_game` (the binder
  and the cleaning rule), `stubby_recipes` (the recipe row, new items).
- Rules data: the recipe row, the purifier item and buildable or placeable entry, and any
  new waveform or module items the decisions below add.
