//! The first-person player as a system (split-game-into-systems): Satisfactory-style
//! movement and look, the hand, and what E reaches. It reads the ground through
//! `stubby_core::GroundHeight` and reaches whatever another system marks
//! `stubby_core::Interactable`; it never names pickups, machines or the world.
pub mod compass;
pub mod controls;
pub mod flashlight;
pub mod hand;
pub mod interaction;
pub mod outline;
