//! Named reservations against a workforce capacity supplied by the host.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const SNAPSHOT_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabourLedger {
    capacity: u32,
    reservations: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LabourSnapshot {
    version: u32,
    capacity: u32,
    reservations: BTreeMap<String, u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum LedgerError {
    #[error("reservation names must not be empty")]
    EmptyName,
    #[error("{missing} additional workers needed")]
    Shortfall { missing: u32 },
    #[error("unsupported labour snapshot version {0}")]
    SnapshotVersion(u32),
    #[error("saved reservation sizes must be positive")]
    InvalidReservation,
}

impl LabourLedger {
    pub fn new(capacity: u32) -> Self {
        Self {
            capacity,
            reservations: BTreeMap::new(),
        }
    }

    /// Existing reservations survive a drop in available workforce.
    pub fn set_capacity(&mut self, capacity: u32) {
        self.capacity = capacity;
    }

    pub fn reserved(&self) -> u64 {
        self.reservations
            .values()
            .map(|&workers| u64::from(workers))
            .sum()
    }

    pub fn available(&self) -> u32 {
        u64::from(self.capacity).saturating_sub(self.reserved()) as u32
    }

    pub fn reservation(&self, job: &str) -> u32 {
        self.reservations.get(job).copied().unwrap_or_default()
    }

    /// Set this job's total reservation. Failed growth leaves the old
    /// reservation intact. During overcommit, releases and reductions still work.
    pub fn reserve(&mut self, job: &str, workers: u32) -> Result<(), LedgerError> {
        if job.is_empty() {
            return Err(LedgerError::EmptyName);
        }
        let existing = self.reservation(job);
        let additional = workers.saturating_sub(existing);
        if additional > 0 {
            let missing = additional.saturating_sub(self.available());
            if missing > 0 {
                return Err(LedgerError::Shortfall { missing });
            }
        }
        if workers == 0 {
            self.release(job);
        } else if workers != existing {
            self.reservations.insert(job.into(), workers);
        }
        Ok(())
    }

    pub fn release(&mut self, job: &str) -> u32 {
        self.reservations.remove(job).unwrap_or_default()
    }

    pub fn snapshot(&self) -> LabourSnapshot {
        LabourSnapshot {
            version: SNAPSHOT_VERSION,
            capacity: self.capacity,
            reservations: self.reservations.clone(),
        }
    }

    pub fn restore(snapshot: LabourSnapshot) -> Result<Self, LedgerError> {
        if snapshot.version != SNAPSHOT_VERSION {
            return Err(LedgerError::SnapshotVersion(snapshot.version));
        }
        if snapshot.reservations.keys().any(String::is_empty) {
            return Err(LedgerError::EmptyName);
        }
        if snapshot.reservations.values().any(|&workers| workers == 0) {
            return Err(LedgerError::InvalidReservation);
        }
        Ok(Self {
            capacity: snapshot.capacity,
            reservations: snapshot.reservations,
        })
    }
}
