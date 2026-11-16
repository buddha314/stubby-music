//! The contracts several stubby-music systems share, and nothing else: which screen is
//! open, the key bindings, the HUD camera marker, what E would act on, and the system
//! sets others order against. No behaviour lives here (split-game-into-systems).
use bevy::prelude::*;

/// Key bindings, following Satisfactory's conventions where they apply (AGENTS.md).
pub mod keys {
    use bevy::prelude::KeyCode;
    pub const INTERACT: KeyCode = KeyCode::KeyE;
    pub const RECIPE_BOOK: KeyCode = KeyCode::KeyO;
    pub const INVENTORY: KeyCode = KeyCode::Tab;
    pub const RECORDER: KeyCode = KeyCode::KeyH;
    pub const JUMP: KeyCode = KeyCode::Space;
    pub const CROUCH: KeyCode = KeyCode::KeyC;
}
pub use keys::*;

/// Full-screen HUD panels. One is open at a time; while one is, the cursor is free and
/// the player neither moves nor looks (add-inventory-and-goals-hud D7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Inventory,
    MachineConfiguration(Entity),
}

#[derive(Resource, Default)]
pub struct ScreenMode {
    pub open: Option<Screen>,
    /// A screen closed this frame, so the click that closed it does not also act.
    pub closed_this_frame: bool,
}
impl ScreenMode {
    pub fn close(&mut self) {
        self.open = None;
        self.closed_this_frame = true;
    }
}

/// The camera that draws HUD screens (the rack). World-facing systems skip it.
#[derive(Component)]
pub struct HudCamera;

/// The one thing E would act on now.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum InteractTarget {
    #[default]
    None,
    /// A finite pickup (its `ResourceNode` entity).
    Pickup(Entity),
    /// A placed machine (its `PlacedAsset` entity), reached through its `Interact` point.
    Machine(Entity),
}
impl InteractTarget {
    pub fn pickup(self) -> Option<Entity> {
        match self {
            Self::Pickup(e) => Some(e),
            _ => None,
        }
    }
    pub fn machine(self) -> Option<Entity> {
        match self {
            Self::Machine(e) => Some(e),
            _ => None,
        }
    }
}

/// Placed prefabs have been bound to their runtime components this frame.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PrefabBinding;

/// Machine interaction (opening a rack with E) has run this frame.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MachineInteractionSet;

/// The ground's height at a horizontal position. The world provides it; the player and
/// anything that sits on the ground read it. Flat until a world installs its own.
#[derive(Resource, Clone, Copy)]
pub struct GroundHeight(pub fn(Vec2) -> f32);
impl Default for GroundHeight {
    fn default() -> Self {
        Self(|_| 0.)
    }
}
impl GroundHeight {
    pub fn at(&self, x: f32, z: f32) -> f32 {
        (self.0)(Vec2::new(x, z))
    }
}

/// Something E can act on, marked by whichever system owns it while it is available: a
/// finite pickup (the entity itself), or a machine's interaction point (the machine).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interactable {
    Pickup,
    Machine(Entity),
}

/// The level was authored in Jackdaw (placed prefabs), not the built-in opening layout.
#[derive(Resource, Default)]
pub struct AuthoredLevel;

/// A source that has been harvested and not yet returned. Whichever system owns the
/// harvest marks it; anything that shows the source (a bug that stops flashing, reeds cut
/// to stubble) reads it.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Harvested;

/// A placed prefab's identity: which asset it is, its model, and its unique instance ID.
/// The prefab lifecycle sets it; every system that acts on placed things reads it.
#[derive(Component, Reflect, Clone, Debug, Default, PartialEq, Eq)]
#[reflect(Component, Default)]
pub struct PlacedAsset {
    pub asset: String,
    pub model: String,
    pub instance: String,
}

/// A placed prefab that could not be bound, and why. It is hidden and inert.
#[derive(Component, Clone, Debug)]
pub struct PrefabError(pub String);

/// A placed machine's own engine clock and its tempo, which everything in time with the
/// machine follows (its rack, its bubbles, its stream's bar lines).
#[derive(Component, Clone, Copy)]
pub struct AssetClock {
    pub engine: stubby_audio::ModuleId,
    pub clock: u64,
    pub bpm: f32,
}

/// The items the player holds, as systems other than crafting see them: counts, the
/// inventory's boxes, taking and giving back, and a one-line message to the player.
/// The crafting system implements it for its inventory.
pub trait Stock: Resource<Mutability = bevy::ecs::component::Mutable> {
    fn quantity(&self, item: &str) -> u32;
    /// The filled boxes in order: item and count.
    fn boxes(&self) -> Vec<(String, u32)>;
    /// How many boxes the inventory has.
    fn box_count(&self) -> usize;
    /// Take `count` of `item`; `false` (and nothing taken) when there are not enough.
    fn take(&mut self, item: &str, count: u32) -> bool;
    /// Give back `count` of `item`; `false` (and nothing given) when it does not fit.
    fn give(&mut self, item: &str, count: u32) -> bool;
    /// Tell the player something, in one line.
    fn say(&mut self, message: String);
}

/// Item IDs to the names players see.
#[derive(Resource, Default, Clone)]
pub struct ItemNames(pub std::collections::HashMap<String, String>);
impl ItemNames {
    pub fn name(&self, item: &str) -> String {
        self.0
            .get(item)
            .cloned()
            .unwrap_or_else(|| item.replace('_', " "))
    }
}

/// Screens open and close (Tab, Escape) in this set.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScreenInput;

/// The cursor is captured or released for mouse look in this set.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CursorCapture;

/// What E reaches is decided in this set.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionTargeting;

/// The player used a placed asset (E on it). Its presentation may react; its owner acts.
#[derive(Message, Clone, Debug)]
pub struct AssetUsed {
    pub entity: Entity,
    pub instance: String,
    pub asset: String,
}

/// A named node of a placed asset's model: its root, its name, its rest pose and glTF
/// extras. The world's presenter poses parts; other systems find ports and outlets by name.
#[derive(Component)]
pub struct AssetPart {
    pub root: Entity,
    pub name: String,
    pub rest: Transform,
    pub extras: serde_json::Value,
    pub flattened: bool,
    pub sway: Option<(Vec3, f32)>,
}

/// A placed asset's living state, which its presentation follows: how grown it is, the
/// water and light around it, and whether a stream reaches it or a port is highlighted.
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component, Default)]
pub struct AssetState {
    pub growth: f32,
    pub water_quality: f32,
    pub light_quality: f32,
    pub stream_connected: bool,
    pub port_highlight: bool,
}

/// Whether the mouse is captured for looking around. The player owns it; anything that
/// clicks in the world (building) reads it.
#[derive(Resource)]
pub struct LookCapture(pub bool);
impl LookCapture {
    pub fn captured(&self) -> bool {
        self.0
    }
}

/// A machine's voice in the world: the engine output it plays, whether it sounds, and
/// how far it carries. The rack writes it; streams carry it and place it in the world.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct MachineVoice {
    pub output: Option<stubby_audio::PortRef>,
    pub sounding: bool,
    pub hearing_radius_metres: f32,
}
