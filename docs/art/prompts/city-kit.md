# City kit prompt

The Phase 4 City (Camp, Village, Town, **City**): a modular construction kit, its reference
district and test strip. Unlike the other files here this is an **agent brief** (a production
prompt for Claude Code), not an image-generator prompt; it arrived with four generated concept
images (see *References* below).
Brief: `docs/art/city-kit.md` · Kit notes and manifest: `docs/art/city-kit/` · Change: `openspec/changes/add-city-kit/`.

## Prompt

The user's brief, kept verbatim. Repository conventions override its proposed defaults; the
conflicts found are listed in `docs/art/city-kit.md` (*Where the repository overrides the prompt*).

## Mission
Act as a senior environment artist, technical artist and Blender Python developer working locally in the Stubby Music repository. Build actual editable Blender assets, a reusable asset library, exported GLBs and a reference city scene, not merely a plan or a concept illustration. The city is the reward for restoring a silent Mojave mudflat through music, water and plant growth. Prioritize reusable geometry and coherent gameplay scale over a single beautiful but monolithic scene.

## 0. Repository discovery (mandatory)
Before modeling, read AGENTS.md, docs/art/style-guide.md, docs/design/design-guide.md, docs/design/roadmap.md, openspec/changes/add-settlement-phase-reviews/design.md and existing docs/art/prompts/. Inspect existing Blender scripts, assets, material naming, acoustic classes, coordinate conventions, Jackdaw prefab examples, exporter and validation commands. Identify installed Blender version and run a small headless export test. Save this prompt in docs/art/prompts/ under a nonconflicting name. Repository conventions override proposed defaults below. Record conflicts and missing files; do not invent acoustic enums, prefab schemas or runtime features. If integration is blocked, still produce editable Blender assets and report the blocker. Do not change unrelated files or commit without authorization. Only CC0 external assets may be committed; record source and license. Generated reference images are art direction, not texture sources or geometry to trace literally.

## 1. Visual direction
Read the four PNGs in references/ (use local image viewing). 01: canal at dusk, layered pedestrian circulation and planted architecture. 02: night, organic bioluminescence and civic performance. 03: monumental plaza, gallery and hall. 04: daytime, clear water, teal sky and vertical gardens. Treat them as alternative views of one coherent design language, not four unrelated cities. Reduce visual noise and excessive neon in the illustrations; do not reproduce giant moons, impractical waterfalls or glowing machine trim. Preserve the brief over the images.

The world begins hazy, silent and brown-grey (Satisfactory meets Inside). Phase 4 is a dense vertical solarpunk city, Tomorrowland and San Fransokyo in spirit, lush rather than chrome. Use stepped Southwestern massing, deep shaded arcades, rounded corners, carved geometric relief, pale mineral monuments, grown timber ribs and modern titanium/graphene structures. Use original abstract motifs inspired by regional geology, water and sunlight; avoid copying sacred or culturally specific designs. Keep visible construction logic and human-scale doors. Every suitable surface supports life, but preserve walking space and readable silhouettes. Water follows the original camp stream; retain a modest old mud-stream edge or relic at the approach. The five-line floating music staff follows the canal, crisp and glowing over clear water. Civic landmarks: light rail, art gallery displaying player patches, concert hall playing player recordings. Provide empty display/stage anchors rather than baking player content. Dancers are mostly silhouettes in four body shapes tinted solar amber, clay coral, leaf teal and dusk violet; use placeholders if existing character assets are unavailable. They must read against both bright sky and luminous night.

### Palette and lighting
Day: teal sky, fog density 0.0008–0.002 (interpret in the actual runtime fog model; not necessarily a Blender volume density), ground #29361c, leaf #4f6a2a, sprout #6f9a34, titanium #b8bcc2, graphene #1a1c1f. One pale monument catches rosy light. Dusk: zenith #4d388f, horizon #ff8f73, violet shadows, reflective water. Night: sky #421a5c, dark never black, flora/architecture cyan #33ffd9, magenta #ff4dd9, green #8cd933. Machine/module signal lights exclusively amber #ffb350. No independent blinking timers. Organic glow: bright core plus translucent soft halo shells (alpha 0.2–0.4), bells, veins, threads and motes, authored at peak intensity. Separate core and halo meshes/materials; runtime audio clock controls breathing. Avoid hard neon strips. Do not assume Blender bloom, procedural nodes, volumetrics or water shaders export to GLB. Use portable PBR materials, bake where necessary, and document runtime effects. Use sRGB hex inputs converted appropriately for Blender color values.

## 2. Modular construction contract
Proposed defaults pending repository inspection: meters, Z-up in Blender, 1 m grid, 0.25 m detail increments, 4 m structural bay, 4 m floor-to-floor, 2.1 m doors, 2–4 m sidewalks, 8 m canal clear width. Use 4/8 m repeatable segments. Place origins at ground-level snap corners for linear modules, center base for standalone props; document local forward direction and connectors. Apply scale and rotation before export, avoid negative scales. Use consistent end profiles and avoid coplanar overlaps. Create straight, corner, end-cap and junction variants where applicable. Keep collision meshes separate and simple, with continuous walkable surfaces; stairs may use ramp collision. Use separate sockets/anchors for vegetation, instruments, rail, doors and displays following existing conventions. Avoid baking decorative objects into structural meshes. Use linked collections or instances in Blender; verify export/runtime instancing behavior instead of assuming it. Create reusable material families and consistent UV density; use shared trim sheets or atlases only if useful. No unique 4K texture per wall. Preserve editable source modifiers; export evaluated geometry. Validate normals, transforms, UVs, texture paths, material slots and seam alignment.

## 3. Asset backlog (build in order)
Start with the P0 kit, prove assembly, then expand P1 and P2. Dimensions are proposed, not fixed if repository conventions differ.

| ID | Asset family | Required modules / variants | Priority |
|---|---|---|---|
| STR01 | Foundation/plaza | 4x4 and 8x8 paving, curb, edge, corner, steps | P0 |
| STR02 | Building facade | 4x4 blank, window, doorway, arcade, corner, end cap | P0 |
| STR03 | Floor/roof | 4x4 slab, parapet, roof cap, planted roof insert | P0 |
| STR04 | Outdoor hallway | 4 m covered bay, open bay, support column, end | P0 |
| STR05 | Vertical circulation | stair flight, landing, ramp, guardrail, lift placeholder | P0 |
| WAT01 | Canal | 4/8 m straight bank, inner/outer corner, end, water plane, access steps | P0 |
| STR06 | Footbridge | 8 m clear-span deck, approaches, railings, optional planted variant | P0 |
| VEG01 | Planting kit | planter, agave, desert grass, shrub, tree, vine strip | P0 |
| STR07 | Residential block | assemble facade/slab kit into 2-, 4-, 6-storey examples | P1 |
| STR08 | Terrace kit | balcony, setback, garden deck, pergola, skybridge | P1 |
| TRN01 | Light rail | straight/curve track, pier, platform, canopy, vehicle placeholder | P1 |
| CIV01 | Gallery | entrance, display wall, exhibit plinth, patch display anchor | P1 |
| CIV02 | Concert hall | shell/rib segments, stage, seating tiers, recording anchor | P1 |
| CIV03 | Monument | pale sculptural landmark, modular base, water feature | P1 |
| VEG02 | Luminous flora | bell, vein leaf, tendril, fairy mote placeholder, core/halo variants | P1 |
| MUS01 | Water music staff | five-line curved/straight guide geometry and note anchor placeholders | P1 |
| PRP01 | Street furniture | bench, shade sail, bollard, signage, instrument pedestal | P2 |
| TEC01 | Infrastructure | solar canopy, pump housing, water gate, cable/pipe, amber signal | P2 |
| VEG03 | Growth variants | bare, sprout, established, lush; independent overlays | P2 |

Use the same structural bays and materials across settlement phases: Camp uses simple foundations, water edges and instrument props; Village adds low arcades and planting; Town adds stacked floors and bridges; City adds vertical terraces, rail and civic shells. Provide at least two compatible facade variants and avoid making a unique mesh for every building. Landmark silhouettes may be unique but should reuse common structural components.

## 4. Reference city scene
Create a compact, walkable demonstration district (suggested 120 x 160 m, adjust to repository scale). Build from library instances, not duplicated bespoke geometry. Include a continuous canal-side route from the old camp approach to a civic plaza; at least two crossings, a shaded arcade, residential blocks, planted terraces, rail station and track, gallery, concert hall and one distant pale monument. Use a simple loop with no inaccessible dead ends on the primary route. Keep clear pedestrian widths and unobstructed Dancer silhouettes. Place landmarks so the player can navigate by sight. Add a separate modular test strip showing straight joins, corners, stacked floors and bridge connections. Provide cameras for approach reveal, canal eye-level, civic plaza, rooftop overview and night view. Eye-level camera about 1.65 m, avoid extreme wide-angle distortion. Create day/dusk/night lighting presets sharing identical geometry. Render at least three review views; use a lightweight viewport or Eevee if available. Include a simple blockout first, then replace with assets. Do not simulate the entire city or invent gameplay mechanics.

## 5. Runtime and performance
Every machine is an instrument: separate interaction, signal and moving parts; provide stable anchors for audio sources and interaction. Every exported material declares the repository's Steam Audio acoustic class in glTF material extras key `acoustic`. Inspect existing values and exporter behavior. Visual material and acoustic class are separate concerns; use actual supported classes for stone, glass, metal, wood, vegetation, etc. Do not silently invent classes. Validate the exported GLB JSON chunk, including extras on every material; if Blender exporter does not preserve them, use a documented deterministic postprocessor that preserves other chunks and metadata. Check that Jackdaw consumes the expected fields. Audio-reactive objects expose stable identifiers; no autonomous animation timers. Do not implement audio-clock behavior in Blender as a substitute for runtime integration.

Favor low draw-call counts, shared materials, reusable meshes and modest transparency. Suggested starting triangle budgets: small prop 100–1500, facade bay 500–3000, tree 1000–5000, bridge 2000–8000, landmark shell 10000–30000. These are targets, not engine limits; measure and revise. Add LOD variants for repeated expensive vegetation and distant buildings where useful and supported. Keep halo shells sparse to reduce overdraw, separate collision from render geometry, and do not export preview cameras/lights in individual assets unless required. Document triangle counts, material slots and texture sizes. Test actual runtime import when tooling is available; do not claim FPS without measuring. Keep desktop and future VR scalability in mind.

## 6. Execution and deliverables
Use Blender Python (`bpy`) scripts that can be rerun headlessly with the installed Blender executable. Prefer deterministic generation, named collections and parameters; no destructive changes to existing files. Save actual .blend files, mark reusable collections/objects as Blender assets where appropriate, and create asset catalog metadata. Export each module or assembly as appropriate through the repository's Blender → GLB → Jackdaw-prefab pipeline. Use the existing prefab schema and tooling rather than inventing one. Keep runtime outputs separate from source and preview scenes. Suggested output layout if none exists: `art/city/source/`, `art/city/scripts/`, `art/city/exports/`, `art/city/review/`; actual repo conventions take precedence. Create an asset manifest (CSV or JSON) with ID, file, dimensions, origin, connectors, materials/acoustic classes, triangle count, phase applicability, dependencies, license and validation status. Include a short README with exact regeneration commands, asset browser setup, snapping instructions and known limitations.

Work in milestones: (1) inspect and report conventions; (2) create P0 geometry and export a representative asset; (3) validate GLB and prefab import; (4) assemble reference blockout and test strip; (5) expand P1/P2, materials and lighting; (6) render, inspect and fix obvious issues; (7) deliver manifest and validation report. Continue autonomously on reversible art decisions, but flag incompatible repository conventions or missing runtime contracts. If time or tools limit completion, prioritize a complete P0 kit and a coherent reference scene over many unfinished assets. Never label unbuilt items as complete.

## Acceptance checklist
- Editable Blender source and repeatable scripts exist; asset library is browsable.
- Modules snap without gaps, overlaps or broken collision; common pieces reused across phases.
- Reference district assembled from the kit with continuous walkable canal route and readable civic landmarks.
- Day/dusk/night share geometry; night glow is organic; machine signals remain amber.
- Every exported material has verified acoustic extras or an explicitly reported blocker.
- GLBs load; prefab generation and runtime import verified where tooling permits.
- No missing textures, unapproved external assets, unrelated changes or invented runtime compatibility claims.
- Review renders, manifest, commands, measured geometry statistics and outstanding issues are delivered.

## References

Four generated concept images came with the prompt, kept as `docs/art/city-kit/references/`
(CC0, the user's): 01 canal at dusk, 02 lunarpunk night, 03 civic plaza, 04 solarpunk day. They are
art direction only. The brief records what was taken from them and what was deliberately left
out (giant moons, waterfalls off buildings, neon trim).

## Log

| Date | Run | Notes |
| --- | --- | --- |
| 2026-11-28 | Claude Code (Opus 5.5), on `breq` | First pass: 86 pieces (P0 47, P1 26, P2 13), reference district and test strip. See `docs/art/city-kit/README.md`. |
