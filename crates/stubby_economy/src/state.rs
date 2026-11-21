//! Serializable settlement state and operations that keep homes and residents in sync.

use serde::{Deserialize, Serialize};

use crate::housing::{Dwelling, DwellingId, Housing};
use crate::roster::{AppearanceCounts, Dancer, DancerId, Roster};
use crate::rules::Stock;
use crate::time::CheckpointCursor;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Settlement {
    world_seed: u64,
    roster: Roster,
    housing: Housing,
    pub cursor: CheckpointCursor,
    pub stock: Stock,
}

impl Settlement {
    pub fn new(world_seed: u64) -> Self {
        Self {
            world_seed,
            roster: Roster::default(),
            housing: Housing::default(),
            cursor: CheckpointCursor::default(),
            stock: Stock::new(),
        }
    }

    pub fn roster(&self) -> &Roster {
        &self.roster
    }

    /// Admit a resident once the caller has checked provisioning. A full
    /// settlement cannot admit a new resident, and failed admissions use no ID.
    pub fn admit(&mut self, appearance: AppearanceCounts, clock_id: u64) -> Option<DancerId> {
        let id = self.roster.next_id()?;
        let home = self
            .housing
            .choose_home(&self.roster, self.world_seed, id)?;
        self.roster.add(self.world_seed, appearance, clock_id, home)
    }

    pub fn add_dwelling(&mut self, id: DwellingId, dwelling: Dwelling) -> bool {
        if !self.housing.add(id, dwelling) {
            return false;
        }
        self.rehouse();
        true
    }

    pub fn remove_dwelling(&mut self, id: DwellingId) -> Option<Dwelling> {
        let dwelling = self.housing.remove(id)?;
        let displaced: Vec<_> = self
            .roster
            .dancers()
            .filter(|dancer| dancer.home == Some(id))
            .map(|dancer| dancer.id)
            .collect();
        for dancer in displaced {
            self.roster.set_home(dancer, None);
        }
        self.rehouse();
        Some(dwelling)
    }

    pub fn remove_dancer(&mut self, id: DancerId) -> Option<Dancer> {
        let dancer = self.roster.remove(id)?;
        self.rehouse();
        Some(dancer)
    }

    /// Residents already here have first claim on shelter, in arrival order.
    fn rehouse(&mut self) {
        let homeless: Vec<_> = self
            .roster
            .dancers()
            .filter(|dancer| dancer.home.is_none())
            .map(|dancer| dancer.id)
            .collect();
        for id in homeless {
            let Some(home) = self.housing.choose_home(&self.roster, self.world_seed, id) else {
                break;
            };
            self.roster.set_home(id, Some(home));
        }
    }
}
