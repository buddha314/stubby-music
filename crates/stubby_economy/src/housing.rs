//! Placed dwellings and selection among dwellings with vacancies.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::num::NonZeroU32;

use crate::roster::{DancerId, Roster, dancer_rng};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct DwellingId(pub u64);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Dwelling {
    pub kind: String,
    pub capacity: NonZeroU32,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Housing(BTreeMap<DwellingId, Dwelling>);

impl Housing {
    pub(crate) fn add(&mut self, id: DwellingId, dwelling: Dwelling) -> bool {
        if let std::collections::btree_map::Entry::Vacant(slot) = self.0.entry(id) {
            slot.insert(dwelling);
            return true;
        }
        false
    }

    pub(crate) fn remove(&mut self, id: DwellingId) -> Option<Dwelling> {
        self.0.remove(&id)
    }

    pub(crate) fn choose_home(
        &self,
        roster: &Roster,
        seed: u64,
        dancer: DancerId,
    ) -> Option<DwellingId> {
        let mut occupancy = BTreeMap::<_, usize>::new();
        for home in roster.dancers().filter_map(|resident| resident.home) {
            *occupancy.entry(home).or_default() += 1;
        }
        let vacancies: Vec<_> = self
            .0
            .iter()
            .filter_map(|(&id, dwelling)| {
                (occupancy.get(&id).copied().unwrap_or_default() < dwelling.capacity.get() as usize)
                    .then_some(id)
            })
            .collect();
        if vacancies.is_empty() {
            return None;
        }

        let mut rng = dancer_rng(seed, dancer, 1);
        // Sample a fixed-width index so the choice is stable across host widths.
        let index = rng.random_range(0..vacancies.len() as u64);
        Some(vacancies[index as usize])
    }
}
