# Concept prompts

Image-generator prompts for concept art, kept so they can be rerun, varied and compared.
This is part of the art workflow (user, 2026-11-20):

1. **Prompt.** Write the asset's prompt here as `<asset>.md`, from the template below: the
   shared style and negative blocks, plus what is particular to the asset. Keep scale,
   materials and palette from `docs/art/style-guide.md`.
2. **Generate.** Run it in any image generator. Record each run in the prompt file's log:
   date, generator, any changes to the prompt, and which image was chosen.
3. **Select.** Save the chosen images under `docs/art/<asset>/` (a hero view, the
   elevations, and a `contact-sheet.jpg` of the rest). Images are concept art, not
   engineering drawings.
4. **Brief.** Write or update `docs/art/<asset>.md`: the continuity rules the model must
   keep where the images disagree with each other or with the brief (as
   `docs/art/vardo-and-cyber-cob.md` does). The written rules win over the images.
5. **Model.** Build in Blender from the brief (`assets-src/blender/<category>/`, exported by
   script), and check it against the brief's checklist.

Generators drift between views (window counts, door sides, wheel sizes). Ask for a
multi-view sheet, or generate the hero view first and use it as the reference for each
elevation, then check the views agree before choosing.

## Shared blocks

Paste these into every prompt; change them here, not in one asset's file.

### Style

> Stylised and readable, Satisfactory meets Inside, softened toward solarpunk. Clean bold
> shapes with a strong silhouette, simple PBR materials with flat base colours, moderate
> detail suitable for a low-to-mid-poly game model. Crude materials (sand, rusted iron,
> weathered wood) carry the weight; high-tech parts (titanium, graphene) do the precise
> work, lashed on with visible joins. Light is soft and organic, like jellyfish and
> fairies: no LED strips, no hard light points.

### Design-sheet layout

> Layout: a large three-quarter hero view at eye height (1.65 m), with orthographic front,
> side and back elevations and a top view in a row beneath, all at the same scale. Neutral,
> even, overcast lighting on a plain light-grey background.

### Negative prompt

> people, animals, text, labels, watermark, logo, photorealistic clutter, tiny noisy
> detail, LED strips, neon signs, glossy sci-fi chrome, dramatic lighting, heavy fog,
> perspective distortion in the elevations, inconsistent views between panels

Add asset-specific negatives after these.

## Template

```markdown
# <Asset> prompt

Brief: `docs/art/<asset>.md` · Images: `docs/art/<asset>/`

## Prompt
<what it is, its scale> <materials> <technology> <light> <palette with hexes>
<Style block> <Design-sheet layout block>

## Negative prompt
<shared negative>, <asset-specific>

## Log
| Date | Generator | Change | Chosen |
| --- | --- | --- | --- |
```

## Prompts

| Asset | Phase | Prompt | Brief |
| --- | --- | --- | --- |
| Cottage | Village | [cottage.md](cottage.md) | [../cottage.md](../cottage.md) |
