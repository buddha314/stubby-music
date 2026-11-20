//! Things that sit on a laid stream (ripple pools, `add-ripple-pool`) keep its time: a
//! placement marked [`OnStream`] carries its stream's clock while the stream sounds, so
//! its one-beat clips play on that clock and its beat-only parts show
//! (`stubby_world::prefab_presentation`); a silent stream leaves it still. The stream and
//! the placement are two systems, so the rule that joins them lives here, in the game.
use bevy::prelude::*;
use bevy_stubby_recipes::{PlayerInventory, RecipeCatalog};
use stubby_core::{AssetClock, MachineVoice, PlacedAsset};
use stubby_streams::LaidStream;

/// Placed on a laid stream: the stream entity it belongs to.
#[derive(Component, Clone, Copy, Debug)]
pub struct OnStream(pub Entity);

/// Built across one laid stream segment (a dam): the stream holding it, the segment's
/// stable key (`LaidStream::segment_key`) and where it stands along the chain. Dismantling
/// elsewhere respawns a stream as pieces; the placement finds its segment again by key.
/// When its own segment is dismantled it goes too, and what it cost to build comes back,
/// as dismantling anything does.
#[derive(Component, Clone, Copy, Debug)]
pub struct OnSegment {
    pub stream: Entity,
    pub segment: u64,
    pub along: f32,
}

impl OnSegment {
    /// The segment of `stream` nearest `at`.
    pub fn of(stream: Entity, laid: &LaidStream, at: Vec3) -> Self {
        let (k, along) = laid.segment_under(at);
        Self {
            stream,
            segment: laid.segment_key(k),
            along,
        }
    }
}

pub struct StreamFeaturesPlugin;
impl Plugin for StreamFeaturesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (keep_stream_time, follow_segments).after(stubby_core::PrefabBinding),
        );
    }
}

#[allow(clippy::type_complexity)] // Features, their streams, and the machines voicing them.
fn keep_stream_time(
    mut commands: Commands,
    features: Query<(Entity, &OnStream, Option<&AssetClock>)>,
    streams: Query<&LaidStream>,
    machines: Query<(&AssetClock, &MachineVoice)>,
) {
    for (entity, on, held) in &features {
        let clock = streams
            .get(on.0)
            .ok()
            .and_then(|stream| stream.source)
            .and_then(|source| machines.get(source).ok())
            .filter(|(_, voice)| voice.sounding)
            .map(|(clock, _)| *clock);
        match (clock, held) {
            (Some(clock), Some(held)) if held.clock == clock.clock && held.bpm == clock.bpm => {}
            (Some(clock), _) => {
                commands.entity(entity).insert(clock);
            }
            (None, Some(_)) => {
                commands.entity(entity).remove::<AssetClock>();
            }
            (None, None) => {}
        }
    }
}

#[allow(clippy::type_complexity)] // Placements on segments, the streams, and the inventory.
fn follow_segments(
    mut commands: Commands,
    mut placed: Query<(
        Entity,
        &mut OnSegment,
        &GlobalTransform,
        Option<&PlacedAsset>,
    )>,
    streams: Query<(Entity, &LaidStream)>,
    catalog: Option<Res<RecipeCatalog>>,
    mut player: Option<ResMut<PlayerInventory>>,
) {
    for (entity, mut on, place, asset) in &mut placed {
        if streams.contains(on.stream) {
            continue;
        }
        let key = on.segment;
        if let Some((stream, laid)) = streams
            .iter()
            .find(|(_, laid)| (laid.first..=laid.last).any(|k| laid.segment_key(k) == key))
        {
            *on = OnSegment {
                stream,
                segment: key,
                along: laid.nearest_along(place.translation()).0,
            };
            continue;
        }
        // Its segment is gone: what it cost to build comes back, as dismantling gives.
        if let (Some(asset), Some(catalog), Some(player)) = (asset, &catalog, player.as_mut())
            && let Ok(cost) = catalog.0.build_cost(&asset.asset, 1)
        {
            let _ = player.items.grant(&cost);
        }
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.insert_resource(RecipeCatalog(stubby_recipes::shipped_catalog().unwrap()))
            .insert_resource(PlayerInventory::default())
            .add_systems(Update, follow_segments);
        app
    }

    fn dam(app: &mut App, stream: Entity, at: Vec3) -> Entity {
        let on = OnSegment::of(stream, app.world().get::<LaidStream>(stream).unwrap(), at);
        app.world_mut()
            .spawn((
                on,
                GlobalTransform::from_translation(at),
                PlacedAsset {
                    asset: "reed_water_purifier".into(),
                    model: "machines/reed_water_purifier/reed_water_purifier.glb".into(),
                    instance: "reed_water_purifier-built-1".into(),
                },
            ))
            .id()
    }

    /// A dam keeps its segment when dismantling elsewhere respawns its stream as pieces.
    #[test]
    fn a_dam_follows_its_segment_into_a_respawned_piece() {
        let mut app = app();
        let (from, to) = (Vec3::ZERO, Vec3::new(12., 0., 0.));
        let stream = app
            .world_mut()
            .spawn(LaidStream::straight_segments(None, from, to, 3))
            .id();
        let dam = dam(&mut app, stream, Vec3::new(6., 0., 0.)); // segment 1
        // Segment 0 is dismantled: segments 1 and 2 respawn as a piece on the same path.
        let mut piece = LaidStream::straight_segments(None, from, to, 3);
        piece.first = 1;
        app.world_mut().entity_mut(stream).despawn();
        let piece = app.world_mut().spawn(piece).id();
        app.update();
        let on = app.world().get::<OnSegment>(dam).expect("the dam stays");
        assert_eq!(on.stream, piece);
    }

    /// A dam whose own segment is dismantled goes, and its build cost comes back.
    #[test]
    fn a_dam_on_a_dismantled_segment_returns_its_cost() {
        let mut app = app();
        let stream = app
            .world_mut()
            .spawn(LaidStream::straight_segments(
                None,
                Vec3::ZERO,
                Vec3::new(12., 0., 0.),
                3,
            ))
            .id();
        let dam = dam(&mut app, stream, Vec3::new(6., 0., 0.));
        app.world_mut().entity_mut(stream).despawn();
        app.update();
        assert!(
            app.world().get_entity(dam).is_err(),
            "the dam went with its segment"
        );
        let player = app.world().resource::<PlayerInventory>();
        assert_eq!(player.items.quantity("reed_water_purifier_kit"), 1);
    }
}
