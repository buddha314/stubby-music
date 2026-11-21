//! Per-quarter-note demand. The host owns the spatial air field and clock events.

use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, num::NonZeroU64};

use crate::{
    roster::DancerId,
    rules::{RuleError, checked, draw},
    state::Settlement,
};

pub const STANDARD_BPM: u64 = 120;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rations {
    pub food: f64,
    pub clean_water: f64,
    pub fresh_air: f64,
}

impl Rations {
    /// Convert daily budgets using the quarter notes in the host's 120 BPM
    /// reference day at normal simulation speed.
    pub fn per_note(self, reference_notes: NonZeroU64) -> Self {
        let notes = reference_notes.get() as f64;
        Self {
            food: self.food / notes,
            clean_water: self.clean_water / notes,
            fresh_air: self.fresh_air / notes,
        }
    }
}

/// Stock IDs bound from the host's ration rows and validated against its item
/// catalog before provisioning starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RationStock<'a> {
    food: &'a str,
    clean_water: &'a str,
}

impl<'a> RationStock<'a> {
    pub fn bind(
        food: &'a str,
        clean_water: &'a str,
        mut item_exists: impl FnMut(&str) -> bool,
    ) -> Result<Self, RationStockError> {
        for id in [food, clean_water] {
            if id.is_empty() || !item_exists(id) {
                return Err(RationStockError::UnknownItem(id.into()));
            }
        }
        if food == clean_water {
            return Err(RationStockError::DuplicateItem(food.into()));
        }
        Ok(Self { food, clean_water })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RationStockError {
    #[error("unknown ration stock item {0:?}")]
    UnknownItem(String),
    #[error("food and clean water both bind to stock item {0:?}")]
    DuplicateItem(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuarterNote {
    pub clock_id: u64,
    /// Monotonic for this clock, including across save and reload.
    pub index: u64,
    pub sample: u64,
}

/// Supplied once per draw by the host, using the Dancer's position at the note.
/// Return the amount delivered, between zero and the request inclusive.
pub trait AirSupply {
    fn draw(&mut self, dancer: DancerId, note: QuarterNote, demand: f64) -> f64;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Supply {
    pub demand: f64,
    pub met: f64,
}

impl Supply {
    pub fn shortfall(self) -> f64 {
        let unmet = self.demand - self.met;
        // Thousands of fractional withdrawals can accumulate rounding error.
        // Use a fraction of this request so small rations still report shortages.
        const ROUNDING_TOLERANCE: f64 = 1e-8;
        if unmet <= self.demand * ROUNDING_TOLERANCE {
            0.0
        } else {
            unmet
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DancerRations {
    pub dancer: DancerId,
    pub food: Supply,
    pub clean_water: Supply,
    pub fresh_air: Supply,
}

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum ProvisionError {
    #[error(transparent)]
    Quantity(#[from] RuleError),
    #[error("clock {clock_id} expected note {expected}, received {received}")]
    NoteGap {
        clock_id: u64,
        expected: u64,
        received: u64,
    },
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Provision {
    last_note: BTreeMap<u64, u64>,
}

impl Settlement {
    /// Supply notes in sample order, breaking ties by clock ID. The host must
    /// deliver every note from its checkpoint/replay stream before later notes.
    /// The first note for a clock establishes its starting index. Returns None for
    /// a note already resolved and errors on a gap without charging any resources.
    pub fn provision_note(
        &mut self,
        note: QuarterNote,
        stock_items: RationStock<'_>,
        rates: Rations,
        air: &mut impl AirSupply,
    ) -> Result<Option<Vec<DancerRations>>, ProvisionError> {
        if let Some(&last) = self.provision.last_note.get(&note.clock_id) {
            if note.index <= last {
                return Ok(None);
            }
            if note.index - last != 1 {
                return Err(ProvisionError::NoteGap {
                    clock_id: note.clock_id,
                    expected: last + 1,
                    received: note.index,
                });
            }
        }

        let RationStock {
            food: food_id,
            clean_water: water_id,
        } = stock_items;
        checked(food_id, rates.food)?;
        checked(water_id, rates.clean_water)?;
        checked("fresh_air", rates.fresh_air)?;
        let quantity = |key| self.stock.get(key).copied().unwrap_or_default();
        let mut food = checked(food_id, quantity(food_id))?;
        let mut water = checked(water_id, quantity(water_id))?;

        let mut report = Vec::new();
        for dancer in self
            .roster()
            .dancers()
            .filter(|dancer| dancer.clock_id == note.clock_id)
        {
            let met = if rates.fresh_air > 0.0 {
                air.draw(dancer.id, note, rates.fresh_air)
                    .max(0.0)
                    .min(rates.fresh_air)
            } else {
                0.0
            };
            report.push(DancerRations {
                dancer: dancer.id,
                food: Supply {
                    demand: rates.food,
                    met: draw(&mut food, rates.food),
                },
                clean_water: Supply {
                    demand: rates.clean_water,
                    met: draw(&mut water, rates.clean_water),
                },
                fresh_air: Supply {
                    demand: rates.fresh_air,
                    met,
                },
            });
        }
        if !report.is_empty() {
            self.stock.insert(food_id.into(), food);
            self.stock.insert(water_id.into(), water);
        }
        self.provision.last_note.insert(note.clock_id, note.index);
        Ok(Some(report))
    }
}
