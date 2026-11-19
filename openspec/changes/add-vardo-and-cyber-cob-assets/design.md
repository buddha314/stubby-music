## Context

The archive provides an art direction text file and many exploratory images. The text is internally explicit about the intended wagon orientation, proportions, and prosthesis count; generated images vary and occasionally contradict those requirements. The written decisions therefore govern continuity. Existing art conventions require reproducible headless Blender generators, glTF runtime models, asset-family organization, and authored LODs in the model.

## Decisions

- Treat the pair as two separate asset families with related visual language: `structures/vardo` and `creatures/cyber_cob`.
- Preserve the vardo's towing/front and entrance/rear orientation in model naming, views, and prefabs. The axle/wheel ratio is a visual target, not a claim about every historic vardo.
- Make the cyber-cob prosthesis an explicit, inspectable near-side front limb. Exactly three natural legs remain. Do not make the alternative hind-leg exploration a selectable variant.
- Model the wagon's organic roof as structural timber integrated with the barrel form. Keep foliage and emissive accents secondary to its silhouette and practical construction.
- Use the generated images as visual inspiration only. They are not historical documentation, orthographic plans, runtime textures, or evidence of material/license provenance for anything beyond the user-supplied references.
- Keep the image footprint limited to the contact sheet and three useful studies; document the source archive and the limits of the image set.
- Keep gameplay out of this art change: no spawn placement, animal AI, harness physics, wagon movement, or save behavior is implied.

## Asset and review approach

Generate reproducible Blender sources with scripts under `tools/blender/`. Export the wagon to `assets/structures/vardo/vardo.glb` and the horse to `assets/creatures/cyber_cob/cyber_cob.glb`, with matching family profiles, Jackdaw metadata/prefabs as required by the current pipeline, and model-owned LODs. Use separate previews for consistent side, front, rear, and three-quarter review. Check scale and orientation against the game conventions and visually inspect both standalone assets together to verify the harness/towing relationship.

## Risks / Trade-offs

- Generated concept art can imply false history or inconsistent mechanics → the written guide and continuity checklist take precedence; historical claims require separate sourced research.
- Fine mane, feathers, wheel spokes, and organic roof details can exceed real-time budgets → keep silhouette and topology readable at distance and author model-owned LODs.
- An animated horse may be valuable later, but this art task does not define skeletal animation or gameplay locomotion → keep hierarchy workable for future rigging without inventing runtime behavior here.
