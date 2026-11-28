//! A rack's configuration as data another peer can apply (add-multiplayer-and-menus 3.5):
//! what is mounted, the player's own cables, the knobs and the chosen samples. Applying one
//! sets `RackState`, `RackKnobs` and `RackSamples`; the engine follows them as it follows
//! the player's own edits. What a change costs is priced from the machine's profile: an
//! item it accepts for each new mount, a patch cable for each new cable (one back for each
//! unplugged). Nothing here knows about networks.
use std::collections::BTreeMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::machine_config::{JackRef, RackConfiguration};
use crate::rack::{RackKnobs, RackState};
use crate::samples::RackSamples;

/// Every patch cable is one of the player's (simplify-rack-cables).
pub const PATCH_CABLE: &str = "patch_cable";

/// One rack's configuration, in a stable order (so equal racks compare equal).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RackRecord {
    /// Place -> module, for places the player filled.
    pub mounted: BTreeMap<String, String>,
    /// The player's own cables (output, input), sorted.
    pub own: Vec<(JackRef, JackRef)>,
    /// (place, param, value), sorted.
    pub knobs: Vec<(String, String, f32)>,
    /// Place -> sample, for sample players.
    pub samples: BTreeMap<String, String>,
}

impl RackRecord {
    pub fn of(state: &RackState, knobs: &RackKnobs, samples: Option<&RackSamples>) -> Self {
        let mut own = state.own.clone();
        own.sort();
        let mut knobs: Vec<(String, String, f32)> = knobs
            .0
            .iter()
            .map(|((place, param), value)| (place.clone(), param.clone(), *value))
            .collect();
        knobs.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
        Self {
            mounted: state
                .mounted
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            own,
            knobs,
            samples: samples.map(|s| s.0.clone()).unwrap_or_default(),
        }
    }

    /// Make a rack this configuration, touching only what differs (so the engine rebuilds
    /// only what changed).
    pub fn apply(
        &self,
        state: &mut Mut<RackState>,
        knobs: &mut Mut<RackKnobs>,
        samples: Option<&mut Mut<RackSamples>>,
    ) {
        let current = RackRecord::of(state, knobs, None);
        if current.mounted != self.mounted || current.own != self.own {
            state.mounted = self.mounted.clone().into_iter().collect();
            state.own = self.own.clone();
            state.version += 1;
        }
        if current.knobs != self.knobs {
            knobs.0 = self
                .knobs
                .iter()
                .map(|(place, param, value)| ((place.clone(), param.clone()), *value))
                .collect();
        }
        if let Some(samples) = samples
            && samples.0 != self.samples
        {
            samples.0 = self.samples.clone();
        }
    }
}

/// What changing a rack from `was` to `now` takes from the player and gives back, as
/// (item, count) lists; or why it cannot be done (a mount the profile does not accept, a
/// module taken out: racks only ever gain modules).
pub type Price = (Vec<(String, u32)>, Vec<(String, u32)>);

pub fn price(
    config: &RackConfiguration,
    was: &RackRecord,
    now: &RackRecord,
) -> Result<Price, String> {
    let mut take: BTreeMap<String, u32> = BTreeMap::new();
    let mut give: BTreeMap<String, u32> = BTreeMap::new();
    for (place, module) in &was.mounted {
        if now.mounted.get(place) != Some(module) {
            return Err(format!("The {place} module cannot be taken out."));
        }
    }
    for (place, module) in &now.mounted {
        if was.mounted.contains_key(place) {
            continue;
        }
        let item = config
            .places
            .iter()
            .find(|p| p.place == *place)
            .and_then(|p| p.accepts.iter().find(|a| a.module == *module))
            .map(|a| a.item.clone())
            .ok_or_else(|| format!("A {module} does not go in {place}."))?;
        *take.entry(item).or_default() += 1;
    }
    let (before, after) = (was.own.len() as u32, now.own.len() as u32);
    if after > before {
        take.insert(PATCH_CABLE.into(), after - before);
    } else if before > after {
        give.insert(PATCH_CABLE.into(), before - after);
    }
    Ok((take.into_iter().collect(), give.into_iter().collect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn campfire() -> RackConfiguration {
        let asset: serde_json::Value = serde_json::from_str(include_str!(
            "../../../assets/structures/campfire/asset.json"
        ))
        .unwrap();
        serde_json::from_value(asset["configure"].clone()).unwrap()
    }

    fn jack(place: &str, port: &str) -> JackRef {
        JackRef {
            place: place.into(),
            port: port.into(),
        }
    }

    /// A mount costs the item the place accepts; cables cost a patch cable each and give
    /// one back unplugged; a module never comes out, and a place takes only what it
    /// accepts.
    #[test]
    fn a_change_is_priced_from_the_profile() {
        let config = campfire();
        let empty = RackRecord::default();
        let mut shell = empty.clone();
        shell.mounted.insert("seq".into(), "snail_sequencer".into());
        shell.own = vec![(jack("clock", "gate"), jack("seq", "clock"))];
        assert_eq!(
            price(&config, &empty, &shell),
            Ok((
                vec![(PATCH_CABLE.into(), 1), ("snail_shell".into(), 1)],
                vec![]
            ))
        );
        assert_eq!(
            price(
                &config,
                &shell,
                &RackRecord {
                    own: vec![],
                    ..shell.clone()
                }
            ),
            Ok((vec![], vec![(PATCH_CABLE.into(), 1)])),
            "unplugging gives the cable back"
        );
        assert!(
            price(&config, &shell, &empty).is_err(),
            "a module never comes out"
        );
        let mut wrong = empty.clone();
        wrong.mounted.insert("seq".into(), "square_vco".into());
        assert!(
            price(&config, &empty, &wrong).is_err(),
            "only what the place accepts"
        );
    }

    /// A record read from a rack and applied to another makes them the same.
    #[test]
    fn a_record_applied_makes_the_rack_the_same() {
        let mut world = World::new();
        let mut knobs = RackKnobs::default();
        knobs.0.insert(("seq".into(), "gate_2".into()), 1.);
        let from = world
            .spawn((
                RackState {
                    mounted: [("seq".to_owned(), "snail_sequencer".to_owned())].into(),
                    own: vec![(jack("clock", "gate"), jack("seq", "clock"))],
                    ..default()
                },
                knobs,
                RackSamples::default(),
            ))
            .id();
        let to = world
            .spawn((
                RackState::default(),
                RackKnobs::default(),
                RackSamples::default(),
            ))
            .id();
        let record = {
            let e = world.entity(from);
            RackRecord::of(
                e.get::<RackState>().unwrap(),
                e.get::<RackKnobs>().unwrap(),
                e.get::<RackSamples>(),
            )
        };
        let mut query = world.query::<(&mut RackState, &mut RackKnobs, &mut RackSamples)>();
        let (mut state, mut knobs, mut samples) = query.get_mut(&mut world, to).unwrap();
        record.apply(&mut state, &mut knobs, Some(&mut samples));
        assert_eq!(RackRecord::of(&state, &knobs, Some(&samples)), record);
        assert_eq!(state.version, 1, "the engine is told the rack changed");
    }
}
