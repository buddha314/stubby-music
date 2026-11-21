//! Process the settlement calendar's boundaries in chronological order.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Checkpoint {
    Dawn,
    Midday,
    Dusk,
    Night,
}

const CHECKPOINTS: [Checkpoint; 4] = [
    Checkpoint::Dawn,
    Checkpoint::Midday,
    Checkpoint::Dusk,
    Checkpoint::Night,
];

/// The host supplies whole simulation seconds since the world's epoch.
/// Dawn is the start of each cycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Schedule {
    cycle_seconds: u64,
    offsets: [u64; 4],
}

impl Schedule {
    pub fn new(cycle_seconds: u64, midday: u64, dusk: u64, night: u64) -> Option<Self> {
        (0 < midday && midday < dusk && dusk < night && night < cycle_seconds).then_some(Self {
            cycle_seconds,
            offsets: [0, midday, dusk, night],
        })
    }

    fn at(self, index: u64) -> Option<DueCheckpoint> {
        let count = CHECKPOINTS.len() as u64;
        let day = index / count;
        let slot = (index % count) as usize;
        let second = day
            .checked_mul(self.cycle_seconds)?
            .checked_add(self.offsets[slot])?;
        Some(DueCheckpoint {
            checkpoint: CHECKPOINTS[slot],
            index,
            second,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DueCheckpoint {
    pub checkpoint: Checkpoint,
    /// Zero-based, starting with the world's first dawn.
    pub index: u64,
    pub second: u64,
}

/// Save beside the settlement state. A fresh cursor includes dawn at time zero.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct CheckpointCursor {
    last_processed: Option<u64>,
}

impl CheckpointCursor {
    pub fn last_processed(self) -> Option<u64> {
        self.last_processed
    }

    /// Resolve each due boundary, including one exactly at `through_second`.
    /// A failed callback leaves that boundary pending. The callback should
    /// commit its state changes only on success.
    pub fn advance<E>(
        &mut self,
        schedule: Schedule,
        through_second: u64,
        mut resolve: impl FnMut(DueCheckpoint) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut next = self
            .last_processed
            .map_or(Some(0), |last| last.checked_add(1));
        while let Some(due) = next
            .and_then(|index| schedule.at(index))
            .filter(|due| due.second <= through_second)
        {
            resolve(due)?;
            self.last_processed = Some(due.index);
            next = due.index.checked_add(1);
        }
        Ok(())
    }
}
