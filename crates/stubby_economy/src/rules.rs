//! Stock rules for one calendar checkpoint.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type Stock = BTreeMap<String, f64>;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleKind {
    Producer,
    Converter,
    Consumer,
    Upkeep,
}

const APPLY_ORDER: [RuleKind; 4] = [
    RuleKind::Producer,
    RuleKind::Converter,
    RuleKind::Consumer,
    RuleKind::Upkeep,
];

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BoundRule {
    pub id: String,
    pub kind: RuleKind,
    pub units: u32,
    /// Stock consumed per unit by converters, consumers and upkeep.
    pub inputs: Stock,
    /// Stock produced per unit by producers and converters.
    pub outputs: Stock,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Resolution {
    pub stock: Stock,
    pub deficits: Stock,
    pub applied: Vec<AppliedRule>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedRule {
    pub id: String,
    pub units: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum RuleError {
    #[error("stock or rule quantity for {resource} must be finite and nonnegative")]
    InvalidQuantity { resource: String },
}

/// Resolve the host's rules for this checkpoint. Within each kind, use the
/// order supplied by the host. A failed resolution leaves the inputs intact.
pub fn resolve(stock: &Stock, rules: &[BoundRule]) -> Result<Resolution, RuleError> {
    validate(stock)?;
    let mut result = Resolution {
        stock: stock.clone(),
        deficits: Stock::new(),
        applied: Vec::new(),
    };
    for kind in APPLY_ORDER {
        for rule in rules
            .iter()
            .filter(|rule| rule.kind == kind && rule.units > 0)
        {
            result.apply(rule)?;
        }
    }
    Ok(result)
}

impl Resolution {
    fn apply(&mut self, rule: &BoundRule) -> Result<(), RuleError> {
        let units = match rule.kind {
            RuleKind::Producer => {
                self.produce(&rule.outputs, rule.units)?;
                rule.units
            }
            RuleKind::Converter => {
                validate(&rule.inputs)?;
                let units = affordable(&self.stock, &rule.inputs, rule.units);
                self.consume(&rule.inputs, units)?;
                self.produce(&rule.outputs, units)?;
                units
            }
            RuleKind::Consumer | RuleKind::Upkeep => {
                validate(&rule.inputs)?;
                self.consume(&rule.inputs, rule.units)?;
                rule.units
            }
        };
        self.applied.push(AppliedRule {
            id: rule.id.clone(),
            units,
        });
        Ok(())
    }

    fn produce(&mut self, outputs: &Stock, units: u32) -> Result<(), RuleError> {
        validate(outputs)?;
        for (resource, amount) in outputs {
            add(&mut self.stock, resource, amount * f64::from(units))?;
        }
        Ok(())
    }

    fn consume(&mut self, inputs: &Stock, units: u32) -> Result<(), RuleError> {
        for (resource, amount) in inputs {
            let demand = amount * f64::from(units);
            if demand == 0.0 {
                continue;
            }
            checked(resource, demand)?;
            let available = self.stock.entry(resource.clone()).or_default();
            if demand > *available {
                add(&mut self.deficits, resource, demand - *available)?;
            }
            *available = (*available - demand).max(0.0);
        }
        Ok(())
    }
}

fn affordable(stock: &Stock, inputs: &Stock, limit: u32) -> u32 {
    inputs
        .iter()
        .filter(|(_, amount)| **amount > 0.0)
        .fold(limit, |units, (resource, amount)| {
            let available = stock.get(resource).copied().unwrap_or_default();
            let estimate = units.min((available / amount).floor() as u32);
            // The rounded quotient and product can disagree at an integer boundary.
            if f64::from(estimate) * amount > available {
                estimate - 1
            } else if estimate < units && f64::from(estimate + 1) * amount <= available {
                estimate + 1
            } else {
                estimate
            }
        })
}

fn validate(amounts: &Stock) -> Result<(), RuleError> {
    for (resource, &quantity) in amounts {
        checked(resource, quantity)?;
    }
    Ok(())
}

fn checked(resource: &str, quantity: f64) -> Result<f64, RuleError> {
    if quantity.is_finite() && quantity >= 0.0 {
        Ok(quantity)
    } else {
        Err(RuleError::InvalidQuantity {
            resource: resource.into(),
        })
    }
}

fn add(stock: &mut Stock, resource: &str, amount: f64) -> Result<(), RuleError> {
    if amount != 0.0 {
        let amount = checked(
            resource,
            stock.get(resource).copied().unwrap_or_default() + amount,
        )?;
        stock.insert(resource.into(), amount);
    }
    Ok(())
}
