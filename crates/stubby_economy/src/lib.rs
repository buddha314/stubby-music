//! Headless settlement accounting. The host supplies calendar offsets, rule
//! data and stock. The checkpoint dispatcher, stock rules and workforce
//! ledger follow automate-godot (MIT, see LICENSE-automate-godot).

pub mod housing;
pub mod ledger;
pub mod roster;
pub mod rules;
pub mod state;
pub mod time;
