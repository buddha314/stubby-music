## Context
Art ships `stream_segment` (4 m, +X flow, 24 subdivisions, `Port_in`/`Port_out` with
halos) and `stream_hologram` (same frame, `Flow` chevrons every 0.5 m, `snap_ghost`
rings, valid/invalid colours). The burbler's `Outlet_0` is the mouth; its +X is the flow.

## Decisions
**D1 Build mode on Q, one buildable.** Q is reserved for build. With one buildable there
is no menu yet: Q goes straight to streams. A build menu comes with the second buildable.

**D2 Ports.** A port is a position on the ground plus a horizontal flow direction. The
burbler's outlet and each laid stream's free end are ports; a port feeds one stream.
The candidate port is the free port nearest the aim point within 3 m.

**D3 Curve.** A cubic Bézier: P0 the port, P1 = P0 + d0·L/3 along the port's direction,
P3 the aim point (clamped to 32 m from the port), P2 = P3 − d1·L/3 with d1 the
direction from P1 to P3. The plan has N = max(1, round(length / 4)) segments; each
covers 1/N of the arc, so segments stretch by at most ±12.5% and the end meets the aim.
Plans with a bend radius under 2 m (the banks would fold) are invalid.

**D4 Bending.** Segment and hologram meshes are read from their glTFs in the asset's
root space and their vertices mapped once per segment: x to the arc position, y up,
z along the side vector (tangent × up); normals and tangents rotate with the frame.
`Pulse_0` and `Bar_0` (notes, out of scope) are not spawned, so a laid stream lies
quiet. Laid streams follow the start port's height (the mudflat is near flat).

**D5 Cost is a rule.** `buildables.json`: `{ "id", "name", "cost": [Stack] }`.
`Catalog::build(id, count, inventory)` takes `cost × count` all or nothing;
`Catalog::build_shortfall` reports what is missing. The segment profile's
`reeds.built_from` stays art documentation; code reads the rule.

**D6 Show, don't tell.** The cost is the reed icon and `need / have`, red when short; no
hint text. Valid/invalid is the hologram's tint.

## Risks / Trade-offs
- Streams are not saved and cannot be removed yet; a misplaced stream stays for the
  session. Dismantle (F) and saves are follow-ups.
- No collision checks: a stream can cross props or another stream.
