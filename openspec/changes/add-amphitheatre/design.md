## Context

The being atlas (`add-being-atlas`) codes 103 mythic beings by trait and writes a seeded 2D
layout that keeps alike beings close. The user decided (2026-11-18, design guide *Mythic
regions*) that a natural amphitheatre in the chaparral shows that layout as a map: found
perhaps early, understood later; its seats give modules back; it rhymes with gathering
places for rain dances and other ceremonies in form only; it echoes Arches National Park
and Greek landscapes. It now drives the story, so its assets come first.

## Goals / Non-Goals

**Goals:** the amphitheatre's model, states and seats as an editable Jackdaw prefab in the
opening scene; seats laid out from the atlas; the game behaviour specified so it can be
built against the asset.

**Non-Goals:** the beings' carvings themselves (each is designed with its shrine,
`add-deity-shrines`); how a being is woken; the final location on the biome map; which
module each seat gives (data, chosen as beings are chosen).

## Decisions

### D1. One rock, two ages
A bowl of red and salmon sandstone, smooth like the slickrock bowl below Delicate Arch,
framed on one side by a natural arch. Its tiers are worn into the bowl's side as a Greek
theatre's follow its hillside (Epidaurus), in paler stone, so it reads as one place shaped
twice: by weather, then by hands long gone. Chaparral on the rim: shrubs, rocky sand,
cypress and olive shapes in the bleak palette until the air clears. Plain PBR materials;
the stone is acoustic rock (glTF extras `"acoustic": "rock"`), so the bowl carries sound
as a theatre should.

### D2. Landforms only
No petroglyphs or rock art (the park's Ute panels included), no copy of a real theatre, no
ceremony or sacred structure. The rhyme with ceremonial grounds comes from forms many
peoples share: a round dancing floor, something living at the centre, seating rising around
it. The Greek *orchestra* was exactly that, the round dancing place of the chorus.

### D3. The arch frames the centre
From the bowl's open side the arch frames the dancing floor and its spring, so the first
view lines up arch, bowl and pool: memorable before it makes sense.

### D4. Seats come from the layout, generated
The export script reads `layout.json` and maps each seated being's (x, y) onto the bowl's
plan: angle around the centre from the layout's angle, tier from its distance, snapped to
the nearest free seat on that tier. Each seat is a socket (`Seat_<being_id>`, a childless
empty outside the LOD nodes, so the placed asset keeps it) holding a seat stone with three
states by node scale: `Empty`, `Hint`, `Filled`. Which beings get seats is the atlas's
`status` (`chosen`); until beings are chosen, the review build seats every open being.
Rerunning the layout and export moves seats; seats are never placed by hand.

### D5. States of the whole
`Spring` at the centre has `Dry`, `Rising` and `Full` levels (node scale, as other shipped
states), driven by the share of seats filled. The empty bowl has no prompt, label or
ring: it is a landform until the first being is seated (show, don't tell).

### D6. Seating gives a module, by rule
What a seat gives is a rule, a row per being in `assets/recipes/` (new rule type: a Rule PR
in the owning stream), never profile data. Filling a seat gives the module once, adds its
neighbours' hints (nearest seats in the layout), and updates the spring.

### D7. Provisional place
Until the biome map exists, the amphitheatre stands in a pocket of chaparral (scrub and
rocky sand) in the opening scene, beyond the echo grotto from spawn and out of every
hearing radius, so the opening stays silent and a wandering player can find it early.

## Risks / Trade-offs

- The seat count is unknown (about a dozen figures ship, the atlas has 103 beings). The
  bowl must read well with a dozen filled seats and with many: model tiers for about 120
  seats, show only the seated ones.
- The layout changes whenever the atlas does; seats move with it. Fine before release,
  a save-format question after.
- A large concave rock form needs good colliders: generated per tier, not one trimesh.

## Open Questions

1. How a being is woken and brought to its seat (`add-deity-shrines`).
2. Which module each being gives.
3. Its place on the biome map, and how far from the start.
4. Whether hinted seats show a silhouette, a biome mark, a kind mark, or all three.
