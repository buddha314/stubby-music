//! One dynamic ID pool for runtime assets and collection voices.
use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use stubby_audio::ModuleId;
#[derive(Resource)]
pub struct ModuleIds {
    next: u64,
    claims: BTreeMap<u64, String>,
    logical_history: BTreeSet<u64>,
}
impl Default for ModuleIds {
    fn default() -> Self {
        Self {
            next: 10_000,
            claims: BTreeMap::new(),
            logical_history: BTreeSet::new(),
        }
    }
}
impl ModuleIds {
    pub fn allocate(&mut self) -> Option<ModuleId> {
        loop {
            let id = self.next;
            self.next = self.next.checked_add(1)?;
            if !self.logical_history.contains(&id) {
                return Some(ModuleId(id));
            }
        }
    }
    pub fn claim_clock(&mut self, id: u64, owner: &str) -> Result<(), String> {
        if id == 0
            || (100..=108).contains(&id)
            || (8001..=8004).contains(&id)
            || (9001..=9007).contains(&id)
            || id >= 10_000 && id < self.next && !self.logical_history.contains(&id)
        {
            return Err(format!("clock {id} conflicts with an engine module"));
        }
        if let Some(previous) = self.claims.get(&id) {
            return Err(format!(
                "clock {id} belongs to {previous}, also requested by {owner}"
            ));
        }
        self.logical_history.insert(id);
        self.claims.insert(id, owner.into());
        Ok(())
    }
    pub fn release_clock(&mut self, id: u64) {
        self.claims.remove(&id);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_clocks_and_dynamic_voices_never_collide() {
        let mut ids = ModuleIds::default();
        ids.claim_clock(10001, "bug").unwrap();
        assert_eq!(ids.allocate(), Some(ModuleId(10000)));
        assert_eq!(ids.allocate(), Some(ModuleId(10002)));
        assert!(ids.claim_clock(10001, "copy").is_err());
        assert!(ids.claim_clock(9004, "demo conflict").is_err());
        assert!(ids.claim_clock(10000, "allocated").is_err());
        ids.release_clock(10001);
        ids.claim_clock(10001, "reloaded").unwrap();
    }
}
