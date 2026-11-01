## The abstract rack

Rust has no abstract classes; the established equivalent is a shared type plus a
trait with default methods, and in Bevy the shared type is a component.

```rust
/// Everything every machine's rack has. One per machine instance.
#[derive(Component, Reflect)]
pub struct Rack {
    pub title: String,               // "MUDPIT BURBLER"
    pub slots: Vec<RackSlot>,        // from the machine's configuration recipe
    pub cables: Vec<Cable>,          // patched by the player
    pub interact_range_m: f32,       // E opens the screen within this reach (3 m)
}

pub struct RackSlot {
    pub place: String,               // "vco"
    pub kind: ComponentKind,         // Clock, Vco, Vcf, Vca, ... (the outline's label)
    pub hp: u32,                     // width; picks slot_outline_<hp>hp
    pub fixed: bool,                 // built in, not removable (the burbler's clock)
    pub mounted: Option<ModuleId>,   // empty → show the labelled outline
}

pub struct Cable { pub from: (String, String), pub to: (String, String), pub color: CableColor }

/// What a particular machine does with its rack. Defaults cover the common case.
pub trait RackMachine {
    fn configuration(&self) -> &RackConfiguration;          // the recipe's slots
    fn header(&self) -> String { String::new() }             // e.g. "90 BPM · configure"
    fn normalled_output(&self) -> Option<PortRef> { None }   // e.g. the burbler's outlet
    fn on_patch_changed(&mut self, _rack: &Rack) {}          // rebuild the engine patch
}
```

Common features, written once for every rack:
- **Open and close:** E within `interact_range_m` of the machine's `Interact` node
  (nearer target wins against pickups), with a prompt; Escape closes; cursor freed,
  movement stopped.
- **Screen:** a separate HUD over the game, drawn by its own camera on a dedicated
  render layer; the world behind dims and blurs (`assets/hud/rack_frame/asset.json`,
  `screen`). The frame stretches to the slots' total width.
- **Slots:** empty slots show `slot_outline_<hp>hp` with `Label_<kind>` from
  `slot_labels`; a component mounts only in a slot of its kind; fixed slots cannot be
  emptied.
- **Patching:** drag between jacks to connect; cables are the general patch cable,
  fully elastic as in VCV Rack (`assets/cables/`); unpatched inputs are silent.
- **Engine:** the rack's modules and cables become the machine's engine patch,
  rebuilt whenever a module or cable changes.

## Configuration recipes are data

Each machine's configurator is the `configure` block in its profile, so a new
machine needs data and its own `RackMachine`, not a new screen:

```json
"configure": {
  "title": "MUDPIT BURBLER",
  "interact": { "node": "Interact", "key": "E", "range_m": 3.0, "prompt": "E | Configure burbler" },
  "places": [
    { "place": "clock",  "kind": "clock", "hp": 8, "fixed": true, "module": "burbler_clock" },
    { "place": "vco",    "kind": "vco",   "hp": 8, "fixed": false },
    { "place": "volume", "kind": "vca",   "hp": 4, "fixed": false }
  ]
}
```

Machines may differ in more than slots (the water filter rack's separated channels,
later): that is what the trait is for. A machine may also get its own frame later;
the profile's `hud` names it.

## Art

`tools/blender/create_hud_rack.py`: `rack_frame` (posts, straps, rails 1 m on X to
stretch), `slot_outline_<hp>hp` for 4–16 HP in the build-ghost colour, `slot_labels`
(CLOCK, VCO, VCF, VCA, each a word and a glyph). New component kinds are one entry in
its `KINDS`. Mocks: `docs/art/hud_mock_empty.png` (construction), `hud_mock.png`
(built and patched), rendered from these assets and the burbler's configuration.

## The rack pattern as built (2026-11-01)
Every machine is configured the same way, so the rack is general and machines differ
only in data:
- **Machine data** (`assets/machines/<id>/asset.json`, `configure`): `places` (kind, HP,
  fixed module or what an open place `accepts`: an inventory item that becomes a module),
  `cables` (each spare cable's colour, the two jacks it joins, its step hint),
  `normalled_output`, `patched_hint`.
- **Module data** (`rack.rs`, `parts` / `internal` / `jack`): the engine modules a rack
  module is built from, its internal connections, and which engine port each panel jack
  is. A new module adds one entry to each.
- **State** (`RackState` on the placed machine): what the player mounted, which cables
  are patched. The engine patch is rebuilt from it on every change, and is exactly the
  cables, so a half-patched voice is silent without special cases. Completing every
  cable sends `MachineConfigured` once (the "Build a heartbeat" chore listens for it).
- **Screen** (`machine_config.rs`): the rack's models drawn by a camera straight onto the
  window (so mesh picking reaches the jacks), parts on its layer by `Propagate`, guidance
  from the data: ghost cables, colour-tinted pulsing pods, the step hint.

Interim choices, to revisit: patching is click a jack, then the other (drag comes with
direct cable dragging); mounting is a button under the rack; the `RackMachine` trait of
D-section tasks is replaced by data, since no machine has needed behaviour of its own.

## Open questions
- Which component kinds come next, and their widths (VCF first, for the water rack).
- Whether slots can accept more than one kind (for example any oscillator).
