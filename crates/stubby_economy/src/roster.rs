//! Stable resident identities and the data saved for each resident.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::num::NonZeroU32;

use crate::housing::DwellingId;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct DancerId(pub u64);

/// Variant counts supplied by the appearance rules. Indices start at zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AppearanceCounts {
    pub body_shapes: NonZeroU32,
    pub skin_tones: NonZeroU32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Appearance {
    pub body_shape: u32,
    pub skin_tone: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Task {
    Idle,
    Working(String),
    Building(String),
    Hauling(String),
    Crewing(String),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Dancer {
    pub id: DancerId,
    pub appearance: Appearance,
    pub home: Option<DwellingId>,
    pub profession: String,
    pub task: Task,
    pub clock_id: u64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Roster {
    next_id: u64,
    dancers: BTreeMap<DancerId, Dancer>,
}

impl Roster {
    pub fn dancers(&self) -> impl Iterator<Item = &Dancer> {
        self.dancers.values()
    }

    pub fn population(&self) -> usize {
        self.dancers.len()
    }

    pub fn profession_count(&self, profession: &str) -> usize {
        self.dancers()
            .filter(|dancer| dancer.profession == profession)
            .count()
    }

    pub fn idle_count(&self) -> usize {
        self.dancers()
            .filter(|dancer| dancer.task == Task::Idle)
            .count()
    }

    pub fn homeless_count(&self) -> usize {
        self.dancers()
            .filter(|dancer| dancer.home.is_none())
            .count()
    }

    pub(crate) fn add(
        &mut self,
        world_seed: u64,
        counts: AppearanceCounts,
        clock_id: u64,
        home: DwellingId,
    ) -> Option<DancerId> {
        let id = self.next_id()?;
        self.next_id = id.0 + 1;
        let mut rng = dancer_rng(world_seed, id, 0);
        let dancer = Dancer {
            id,
            appearance: Appearance {
                body_shape: rng.random_range(0..counts.body_shapes.get()),
                skin_tone: rng.random_range(0..counts.skin_tones.get()),
            },
            home: Some(home),
            profession: "laborer".into(),
            task: Task::Idle,
            clock_id,
        };
        self.dancers.insert(id, dancer);
        Some(id)
    }

    pub(crate) fn next_id(&self) -> Option<DancerId> {
        self.next_id.checked_add(1).map(|_| DancerId(self.next_id))
    }

    pub(crate) fn remove(&mut self, id: DancerId) -> Option<Dancer> {
        self.dancers.remove(&id)
    }

    pub(crate) fn set_home(&mut self, id: DancerId, home: Option<DwellingId>) {
        if let Some(dancer) = self.dancers.get_mut(&id) {
            dancer.home = home;
        }
    }
}

pub(crate) fn dancer_rng(world_seed: u64, id: DancerId, purpose: u64) -> ChaCha8Rng {
    let mut seed = [0; 32];
    seed[..8].copy_from_slice(&world_seed.to_le_bytes());
    seed[8..16].copy_from_slice(&id.0.to_le_bytes());
    seed[16..24].copy_from_slice(&purpose.to_le_bytes());
    ChaCha8Rng::from_seed(seed)
}
