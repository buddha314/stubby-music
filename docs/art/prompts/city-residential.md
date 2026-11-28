# City residential expansion prompt

The second City brief: mixed-use, inhabited buildings (shops, offices, homes, roof gardens)
added to the City kit (`docs/art/prompts/city-kit.md`). An agent brief, like the first.
Brief: `docs/art/city-residential.md` · References: `docs/art/city-kit/residential/references/`
(CC0, the user's) · Change: `openspec/changes/add-city-residential/`.

## Prompt

Kept verbatim. Repository conventions override its proposals; the mapping and conflicts are
in `docs/art/city-residential.md`.

## Mission
Act as a senior environment artist and Blender Python technical artist. Augment the existing modular city kit, especially STR07 residential blocks, STR08 terraces and STR02 facades. Do not replace the civic kit or rebuild the city from scratch. The city must read as a place where people actually live, work and shop, not a futuristic resort or monumental park. Produce editable Blender assets, repeatable scripts, GLBs, Jackdaw prefabs and a mixed-use reference district integrated into the existing city scene where available.

## 1. Discover the existing project first
Read AGENTS.md, docs/art/style-guide.md, docs/design/design-guide.md, docs/design/roadmap.md, openspec/changes/add-settlement-phase-reviews/design.md, and existing docs/art/prompts/ including the original city production prompt. Inspect the current asset manifest, Blender files, scripts, materials, snapping conventions, collision, acoustic classes and prefab/export tooling. Save this brief under docs/art/prompts/ without overwriting prior prompts. Existing conventions take precedence. Report missing files or conflicting contracts; do not invent runtime schemas. If the original kit is not yet built, implement the extension against its documented interfaces and clearly report integration as pending.

## 2. Reference direction
Inspect all four PNGs in references/. They are complementary examples, not four unrelated cities. 01 shows a clear storefront/office/apartment stack with a strong street wall; 02 shows rounded balconies, sofas, domestic terraces and stepped upper floors; 03 shows dusk, curtains, warm domestic lighting and individualized balconies; 04 shows curved corner units, vertical circulation and communal rooftop gardens. The defining requirement is that upper floors visibly contain homes, not more restaurants, offices or hotel rooms. Use the images as visual direction, not as textures or literal geometry. Reduce excessive visual noise and keep the structural grid legible.

## 3. Mixed-use architectural grammar
Use the existing 4 m structural bay and floor-to-floor convention if confirmed by the repository. Proposed building stack: ground floor retail (one level), office/business (one or two levels), residential (three to eight levels), rooftop shared gardens. Adapt heights to existing modules; do not introduce incompatible grids. Ground level: shops, cafes, grocers, repair/instrument stores, shaded arcades, display windows and a separate residential entrance. Offices: wider continuous glazing, desks, meeting tables, studios and workstations. Residential: repeated but individualized apartment bays with smaller window groupings, curtains, sliding doors, private balconies, sofas, bookshelves, dining tables, kitchens, beds where appropriate, laundry, planters and shade fabrics. Include opaque walls and privacy screens; avoid turning every home into a glass showroom. Residential circulation must be distinct from shops: lobby, stair/lift core, landings, access corridors and roof access. Public bridges should connect public or communal levels, not private living rooms. Use warm mineral masonry, rounded corners, deep reveals, abstract Southwestern geometric relief, titanium and graphene accents. Preserve lush greenery while leaving enough exposed facade to read the occupancy and floor hierarchy.

## 4. Modular backlog
Reuse existing foundations, slabs, columns, arches, rails, stairs, vegetation and materials before creating variants. IDs below are proposed; map them to the actual manifest. Build independent pieces with compatible snap origins and connectors, not a unique mesh for each building.

| ID | Family | Required variants |
|---|---|---|
| MIX01 | Retail frontage | open arcade, glazed shop, corner shop, cafe, awning, display insert |
| MIX02 | Office frontage | glazed bay, corner bay, opaque service bay, sunshade, workstation insert |
| RES01 | Residential facade | window pair, sliding balcony door, recessed loggia, solid/privacy bay, corner bay |
| RES02 | Balcony kit | straight, curved corner, recessed, end cap, rail, planter, shade sail, privacy screen |
| RES03 | Dwelling shell | studio, one-bedroom, family-size assembly using shared wall/floor/door modules |
| RES04 | Domestic interior | sofa, table/chairs, bed, kitchen counter, bookshelf, curtains, lamp, rug, laundry |
| RES05 | Access core | street lobby, entry door, stair/landing, lift placeholder, shared corridor, roof access |
| RES06 | Communal terrace | garden deck, pergola, seating, greenhouse insert, shared courtyard |
| RES07 | Building assemblies | straight street-wall block, rounded corner block, stepped terrace block, taller tower |
| RES08 | Distant facade | simplified opaque/window variants and low-cost interior proxies |

Separate furniture, vegetation, fabric and facade geometry so they can be reused independently. Provide at least two residential facade variants and multiple combinations of balcony inserts. Keep furniture at human scale. Make domestic modules useful in Village and Town as well as City. Avoid baking furnishings into structural assets. Provide a small set of fully modeled demonstration apartments; use simplified interior proxies for distant or inaccessible units. Do not imply all apartments are playable unless the existing design requires it.

## 5. Reference scene
Extend the existing city with a dense, ordinary mixed-use neighborhood along the canal and at least one inland pedestrian street or courtyard. Assemble at least three distinct building types from shared pieces: continuous street wall, rounded corner, stepped garden tower. Suggested heights 5–12 storeys, subject to existing scale and performance budgets. Include a retail promenade, office floors, clearly visible upper-level homes, residential entrance, shared roof garden and believable circulation. Preserve canal continuity, floating five-line staff, rail alignment and civic sightlines. Keep private/public boundaries legible. Add review cameras: street eye-level looking upward, opposite canal bank showing full vertical stack, upper-level balcony looking toward neighbors, and a rooftop overview. Render day and dusk views plus a night view using the same geometry. Include a simple assembly test showing interchangeable retail, office and residential bays and corner/stack connections.

## 6. Art and runtime constraints
Retain the original palette: day teal sky, ground #29361c, leaf #4f6a2a, sprout #6f9a34, titanium #b8bcc2, graphene #1a1c1f; dusk zenith #4d388f, horizon #ff8f73 and violet shadows; night sky #421a5c, organic cyan #33ffd9, magenta #ff4dd9, green #8cd933. Machine signal lights exclusively amber #ffb350. Organic glow uses separate emissive cores and translucent halo shells (alpha 0.2–0.4), authored at peak; runtime audio clock controls breathing, no independent blinking. Keep domestic lighting warm and readable without excessive emissive windows. Dancers retain four silhouette shapes and solar amber, clay coral, leaf teal or dusk violet tints. Every machine is an instrument; preserve interaction/audio anchors. Every exported material must have a verified glTF material extras `acoustic` value from the repository Steam Audio class definitions, including curtains, glass, rugs and furniture. Never invent enums. Use portable PBR, shared materials, reusable meshes, simple collision and documented LOD/proxy strategies. Verify export behavior rather than assuming Blender procedural materials or lighting transfer. Only CC0 external assets may be committed; record provenance. Do not commit without authorization.

## 7. Execution and acceptance
1. Audit existing kit and propose a delta manifest identifying reused and new assets.
2. Build one complete mixed-use vertical bay and validate snap alignment, material extras, GLB and prefab import before multiplying variants.
3. Build domestic props, balcony and access modules; assemble the three building types and demonstrate reuse across them.
4. Extend the reference scene non-destructively, render review views and inspect the result. If upper levels still look commercial, revise windows, domestic furniture, curtains and privacy boundaries.
5. Deliver editable .blend files, marked Blender assets/catalog entries, deterministic bpy scripts, GLBs, Jackdaw prefabs, manifest delta, regeneration commands, review renders and validation report. Record dimensions, connectors, materials/acoustic classes, triangle counts, dependencies and license. Verify actual imports where tooling permits. Report blockers honestly; do not label unbuilt or untested items complete.

Acceptance: retail/office/residential uses distinguishable without labels; homes visibly inhabited and varied; three assemblies share structural modules; private and public circulation coherent; no broken seams or collisions; original city assets preserved; day/dusk/night coherent; acoustic metadata verified; performance costs measured rather than guessed. Prioritize a complete reusable residential kit over a single elaborate building.

## Log

| Date | Run | Notes |
| --- | --- | --- |
| 2026-11-28 | Claude Code (Opus 5.5), on `breq` | Validated one mixed-use bay first, then the residential kit, four building assemblies and the neighbourhood in the reference district. |
