## Approach
Keep fixed hovering positions with distinct heights relative to terrain. An unlit alpha-blended soft disk sits just above the mudplate rims below each bug. Each disk uses the same PulseFollower and sample-clock envelope as its bug. Shadows are dark ground cues rather than physically simulated shadows from the bug's own light. Show a centered E prompt below the reticle whenever any uncollected bug is within the same horizontal 3m ground radius used by harvesting, including between flashes.

The ground accent is 2m across with a stronger soft core (95% peak opacity). Collection and all reach prompts ignore vertical height and select the nearest resource by horizontal distance.
