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
