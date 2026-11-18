## Decisions

### D1. A ring, not fixed ports
A tree has no front: the player approaches from anywhere. The inlet is a ring round the
trunk, and the snap point is the ring point nearest the aim, so the player chooses which
side the stream passes by aiming left or right of the trunk. The stream arrives along the
ring's tangent, taking the tangent that leads on away from where it was laid from.

### D2. Aiming at the trunk snaps
Aiming anywhere inside the ring (at the trunk itself) snaps, as aiming at a Satisfactory
machine snaps a conveyor to its input. Outside, the aim must be within `snap_m` of the
ring.

### D3. The ring is the tree's data
Its size is art data on the oak's profile (trunk spread plus the stream's half width with
banks, 1.58 m), and a test keeps it inside the oak's watering reach. The contract names
no tree, so other owners (a garden, a pond) can carry inlets later.

### D4. The stream's end stays free
A snapped stream's end is still a port: the player can lay on past the tree. Watering is
still by proximity (`watering.rs`).

## Risks

- A stream laid from behind the tree toward its far side may need a tight bend at the end;
  the existing minimum radius check marks such a plan invalid (red), as for any bend.
