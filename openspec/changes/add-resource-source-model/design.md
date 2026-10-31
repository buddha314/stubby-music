## Approach
Keep catalog resources as exhaustible pickups yielding item stacks. Introduce a separate
mine definition with a produced-resource ID: scrub_oak produces lightning_bug, which
in turn yields square_wave items. A mine produces resource objects, not finished
modules or an implicit direct square-wave payout.

SourceObject is a serializable tagged enum. Exhaustible objects have a collected
flag; infinite mine objects have no depletion counter or collected flag. SourceKind
exposes the distinction without parsing IDs. Catalog factories validate source IDs;
object harvesting atomically consumes exhaustible instances and explicitly rejects
mine collection. Mines are not placed in the finite-pickup query.

Mine definitions name future productivity factors (water_quality, light_quality).
They contain no rates, environmental simulation, timers or growth formulas yet.
MineNode is a reflected Bevy component suitable for authoring a mine instance, with
conversion to the Bevy-independent source object. Do not spawn an invisible mine or
invent scrub-oak geometry. Production is tracked in add-scrub-oak-production.
