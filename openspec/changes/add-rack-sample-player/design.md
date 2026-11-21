## Rack interaction

Use a Sampler tile with a trigger jack, an output jack and a selector that always
shows the assigned clip or “Choose sound”. Open a picker from the selector.
List collected clips by name with their source and duration. Include search,
keyboard navigation, a Preview action, and a clear selection action. Audition
one clip at a time on the interface bus. Selecting a clip assigns it and
closes the picker. Selecting, clearing or auditioning leaves patch cables in
place. Escape closes the picker and returns focus to the module. Keep current
cable and module gestures working outside the picker.

“Multi-sample” means a library of selectable clips. Each module plays one
selected sound. Several sampler modules can select and play different sounds in
the same rack. The NYSTHI sampler family is a reference for clip playback in a
modular rack. [VCV Library](https://library.vcvrack.com/NYSTHI)

## Collection and saved patches

A sound collectible can unlock an authored clip by stable catalog ID. Store
discovery in the existing recipe book. The catalog item ID identifies the clip,
and an optional `sample` on the item row gives its WAV path and source label. Reuse the existing field recorder feedback.
A rack configuration saves the clip ID per module. If a saved clip is
unavailable, keep the ID, show its unavailable state and leave the output
silent. Older rack configurations start with an empty selection.

## Playback

Reuse Firewheel's graph, sample loading and off-thread resampling.
Firewheel's stock sampler has no signal inputs. A small Firewheel node reads
the patched gate and resets fundsp's `WavePlayer` at the matching sample
offset. Voice data shares a prepared mono wave. Selection changes are
exchanged through Firewheel's deferred-drop events. Auditions use the
one-shot command path to the shared device output, bypassing the world meter
as other interface cues do.

A rising edge above the shared gate threshold plays a one-shot. A new rising
edge restarts an active clip with a short declick transition. Selection changes
apply to the next trigger, and can be staged without rebuilding cables. Prepare
clip data off the callback, share it across module instances, and retire data
off the callback. Live recording, file import, clip slicing and CV clip
selection can be considered separately.

## Offline playback baseline

Four Freesound previews listed as CC0 were decoded to stereo PCM at 44.1 kHz
and played at 48 kHz through the existing one-shot engine path at gain 0.8.
Each rendered without callback allocations or clipping, returned to silence
and left the world meter unchanged.

| Source | Rendered onset | Review note |
| --- | ---: | --- |
| [Single Rock Hitting Wood 3](https://freesound.org/people/worthahep88/sounds/319223/) | 38.4 ms | Trim lead-in before sequencing. |
| [Quick Metal Click](https://freesound.org/people/Yarmonics/sounds/441853/) | Under 0.1 ms | Candidate for tight rhythmic accents. |
| [Small Metal Clasp](https://freesound.org/people/_sinny_/sounds/822397/) | 0.1 ms | Audition the full clasp motion and the final snap separately. |
| [Hammer Falls to Wood Floor](https://freesound.org/people/odditonic/sounds/388833/) | 9.4 ms | Use the full drop for world effects and audition the first attack for sequencing. |

Onset is the first output frame above 0.01 absolute amplitude on either channel.
This baseline exercises command-driven one-shot playback. Patch-cable triggering
and clip selection are covered by the remaining implementation tasks. The
previews and generated renders are local review material.


## Review rack

The bounded `--standalone --sampler-review` run opens the nearest burbler
with two sampler places and a mixer. Both samplers have clock and mixer
cables prepatched. Three CC0 impact clips are available in this review bank.
The normal recipe book loads samples from authored item rows after collection.
A live collection source and a sampler unlock are still open.

From this worktree, build `stubby_jackdaw`, then run
`python3 tools/jackdaw_timed_test.py --standalone --sampler-review --name sampler`.
In the left rack column, choose a sound for each sampler. Type to filter, use
the arrow keys to move the selection, and press Enter to assign the selected
clip. Preview auditions a clip and Escape dismisses the picker.

The 20-second standalone review opened both wired sampler panels and showed
their selectors beside the rack. Headless checks exercised independent
assignments, clearing, serializing the two assignments, loading a real WAV
after discovery, and running a triggered voice through a filter. Live
pointer and keyboard navigation and reopening the rack with live selections
still need playtest coverage.
