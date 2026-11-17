//! F dismantles the aimed stream segment (sync-stream-growth-and-dismantling 2.1, the
//! Godot game's `LaidStream.dismantle`, design D8 of `add-stream-building`).
//!
//! Outside build mode, F removes the segment in the crosshair within 8 m and refunds its
//! cost (`SegmentCost`) all or nothing: with no room for the refund, nothing changes.
//! The stream splits around the gap. The part upstream keeps its source and gets a free
//! end at the gap; the part downstream is cut off, and it and every stream laid on from
//! it lie quiet (`resolve_sources`). Removing a stream's first segment frees the port it
//! was laid from.
use super::*;

/// How far F reaches, metres (the Godot game's `DISMANTLE_REACH`).
const DISMANTLE_REACH: f32 = 8.0;
/// Aim this close to a stream's centre line hits it: its bed and banks.
const HALF_WIDTH: f32 = 1.5;

/// Remove segment `segment` of `stream` and refund it.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dismantle {
    pub stream: Entity,
    pub segment: u32,
}

/// The nearest segment along a ray from `origin` (direction `dir`, normalised) within
/// `reach`: the stream and the segment's index.
pub fn pick_segment<'a>(
    streams: impl IntoIterator<Item = (Entity, &'a LaidStream)>,
    origin: Vec3,
    dir: Vec3,
    reach: f32,
) -> Option<(Entity, u32)> {
    let mut best: Option<(f32, Entity, u32)> = None;
    for (entity, stream) in streams {
        for k in stream.first..=stream.last {
            let from = k as f32 * stream.span;
            let steps = (stream.span / 0.25).ceil().max(1.) as u32;
            for i in 0..=steps {
                let (point, _) = stream.path.at(from + stream.span * i as f32 / steps as f32);
                let to = point - origin;
                let t = to.dot(dir);
                if t <= 0. || t > reach || (to - dir * t).length() > HALF_WIDTH {
                    continue;
                }
                if best.is_none_or(|(nearest, ..)| t < nearest) {
                    best = Some((t, entity, k));
                }
            }
        }
    }
    best.map(|(_, entity, k)| (entity, k))
}

pub(crate) fn dismantle_input(
    keys: Res<ButtonInput<KeyCode>>,
    capture: Res<LookCapture>,
    screens: Res<ScreenMode>,
    build: Res<StreamBuild>,
    camera: Query<&GlobalTransform, (With<Camera3d>, Without<HudCamera>)>,
    streams: Query<(Entity, &LaidStream)>,
    mut dismantles: MessageWriter<Dismantle>,
) {
    if !keys.just_pressed(stubby_core::keys::DISMANTLE)
        || build.active
        || screens.open.is_some()
        || !capture.captured()
    {
        return;
    }
    let Ok(camera) = camera.single() else { return };
    if let Some((stream, segment)) = pick_segment(
        streams,
        camera.translation(),
        camera.forward().as_vec3(),
        DISMANTLE_REACH,
    ) {
        dismantles.write(Dismantle { stream, segment });
    }
}

/// Give the cost back, all or nothing.
fn refund<S: Stock>(stock: &mut S, cost: &SegmentCost) -> bool {
    let mut given: Vec<(&str, u32)> = Vec::new();
    for (item, count) in &cost.0 {
        if stock.give(item, *count) {
            given.push((item, *count));
        } else {
            for (item, count) in given {
                stock.take(item, count);
            }
            stock.say(format!("No room for the {item} from this segment."));
            return false;
        }
    }
    true
}

#[allow(clippy::too_many_arguments)] // The split touches streams, burblers, meshes and stock.
pub(crate) fn apply_dismantles<S: Stock>(
    mut commands: Commands,
    mut dismantles: MessageReader<Dismantle>,
    templates: Res<Templates>,
    mut meshes: ResMut<Assets<Mesh>>,
    cost: Res<SegmentCost>,
    mut stock: Option<ResMut<S>>,
    mut burblers: Query<&mut AssetState>,
    mut streams: Query<(Entity, &mut LaidStream)>,
) {
    let mut gone = Vec::new();
    for &Dismantle { stream, segment: k } in dismantles.read() {
        let Some(stock) = stock.as_mut() else { return };
        let Ok((_, old)) = streams.get(stream) else {
            continue;
        };
        if gone.contains(&stream) || k < old.first || k > old.last {
            continue;
        }
        if !refund(stock.as_mut(), &cost) {
            continue;
        }
        let (path, span, first, last) = (old.path.clone(), old.span, old.first, old.last);
        let (upstream, start, end_used) = (old.upstream, old.start, old.end_used);
        let fed: Vec<Entity> = streams
            .iter()
            .filter(|(_, other)| other.upstream == Some(PortOwner::Stream(stream)))
            .map(|(entity, _)| entity)
            .collect();
        let downstream = (k < last).then(|| {
            spawn_stream(
                &mut commands,
                &templates,
                &mut meshes,
                StreamPart {
                    path: path.clone(),
                    span,
                    first: k + 1,
                    last,
                    upstream: None,
                    start: start + (k + 1 - first) as f32 * span,
                    end_used,
                },
            )
            .0
        });
        // Streams laid on from the old end now hang off the cut-off part (or nothing).
        for entity in fed {
            if let Ok((_, mut child)) = streams.get_mut(entity) {
                child.upstream = downstream.map(PortOwner::Stream);
            }
        }
        if k > first {
            spawn_stream(
                &mut commands,
                &templates,
                &mut meshes,
                StreamPart {
                    path,
                    span,
                    first,
                    last: k - 1,
                    upstream,
                    start,
                    end_used: false,
                },
            );
        } else {
            match upstream {
                Some(PortOwner::Burbler(burbler)) => {
                    if let Ok(mut state) = burblers.get_mut(burbler) {
                        state.stream_connected = false;
                    }
                }
                Some(PortOwner::Stream(entity)) => {
                    if let Ok((_, mut up)) = streams.get_mut(entity) {
                        up.end_used = false;
                    }
                }
                None => {}
            }
        }
        commands.entity(stream).despawn();
        gone.push(stream);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct Reeds {
        count: u32,
        room: bool,
        said: Vec<String>,
    }
    impl Stock for Reeds {
        fn quantity(&self, _: &str) -> u32 {
            self.count
        }
        fn boxes(&self) -> Vec<(String, u32)> {
            vec![]
        }
        fn box_count(&self) -> usize {
            16
        }
        fn take(&mut self, _: &str, count: u32) -> bool {
            let Some(left) = self.count.checked_sub(count) else {
                return false;
            };
            self.count = left;
            true
        }
        fn give(&mut self, _: &str, count: u32) -> bool {
            if self.room {
                self.count += count;
            }
            self.room
        }
        fn say(&mut self, message: String) {
            self.said.push(message);
        }
    }

    /// A burbler and, from it, a straight 4-segment stream along +x.
    fn app(room: bool) -> (App, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Templates>()
            .init_resource::<Assets<Mesh>>()
            .insert_resource(SegmentCost(vec![("reed".into(), 6)]))
            .insert_resource(Reeds { room, ..default() })
            .add_message::<Dismantle>()
            .add_systems(Update, (apply_dismantles::<Reeds>, resolve_sources).chain());
        let burbler = app
            .world_mut()
            .spawn(AssetState {
                stream_connected: true,
                ..default()
            })
            .id();
        let stream = app
            .world_mut()
            .spawn(LaidStream::straight_segments(
                Some(burbler),
                Vec3::ZERO,
                Vec3::X * 16.,
                4,
            ))
            .id();
        app.update();
        (app, burbler, stream)
    }

    /// Every stream as (first, last, flowing from the burbler), by first segment.
    fn streams(app: &mut App) -> Vec<(u32, u32, bool)> {
        let mut found: Vec<_> = app
            .world_mut()
            .query::<&LaidStream>()
            .iter(app.world())
            .map(|s| (s.first, s.last, s.source.is_some()))
            .collect();
        found.sort();
        found
    }

    fn dismantle(app: &mut App, stream: Entity, segment: u32) {
        app.world_mut().write_message(Dismantle { stream, segment });
        app.update();
        app.update(); // sources resolve once the new parts exist
    }

    #[test]
    fn cutting_the_middle_leaves_upstream_flowing_and_downstream_quiet() {
        let (mut app, _, stream) = app(true);
        dismantle(&mut app, stream, 1);
        assert_eq!(streams(&mut app), [(0, 0, true), (2, 3, false)]);
        assert_eq!(app.world().resource::<Reeds>().count, 6, "refunded once");
    }

    #[test]
    fn cutting_the_first_segment_frees_the_burblers_mouth() {
        let (mut app, burbler, stream) = app(true);
        dismantle(&mut app, stream, 0);
        assert_eq!(streams(&mut app), [(1, 3, false)]);
        assert!(
            !app.world()
                .get::<AssetState>(burbler)
                .unwrap()
                .stream_connected
        );
    }

    #[test]
    fn cutting_the_last_segment_shortens_the_stream_and_frees_its_end() {
        let (mut app, _, stream) = app(true);
        dismantle(&mut app, stream, 3);
        assert_eq!(streams(&mut app), [(0, 2, true)]);
        let end_used = app
            .world_mut()
            .query::<&LaidStream>()
            .single(app.world())
            .unwrap()
            .end_used;
        assert!(!end_used);
    }

    #[test]
    fn a_stream_laid_on_from_a_cut_off_part_goes_quiet() {
        let (mut app, _, stream) = app(true);
        app.world_mut()
            .get_mut::<LaidStream>(stream)
            .unwrap()
            .end_used = true;
        let mut next = LaidStream::straight(None, Vec3::X * 16., Vec3::X * 24.);
        next.upstream = Some(PortOwner::Stream(stream));
        let next = app.world_mut().spawn(next).id();
        app.update();
        assert!(
            app.world()
                .get::<LaidStream>(next)
                .unwrap()
                .source
                .is_some()
        );
        dismantle(&mut app, stream, 2);
        let next = app.world().get::<LaidStream>(next).unwrap();
        assert!(next.source.is_none(), "cut off with the downstream part");
        assert!(matches!(next.upstream, Some(PortOwner::Stream(_))));
    }

    #[test]
    fn with_no_room_for_the_refund_nothing_changes() {
        let (mut app, _, stream) = app(false);
        dismantle(&mut app, stream, 1);
        assert_eq!(streams(&mut app), [(0, 3, true)]);
        assert_eq!(app.world().resource::<Reeds>().said.len(), 1);
    }

    #[test]
    fn f_picks_the_segment_in_the_crosshair_within_reach() {
        let stream = LaidStream::straight_segments(None, Vec3::ZERO, Vec3::X * 16., 4);
        let entity = Entity::PLACEHOLDER;
        // Standing beside segment 2 (x 8..12), looking down at it.
        let eye = Vec3::new(10., 1.7, -3.);
        let at = (Vec3::new(10., 0., 0.) - eye).normalize();
        assert_eq!(
            pick_segment([(entity, &stream)], eye, at, DISMANTLE_REACH),
            Some((entity, 2))
        );
        // Looking away, or from too far, picks nothing.
        assert_eq!(
            pick_segment([(entity, &stream)], eye, -at, DISMANTLE_REACH),
            None
        );
        let far = Vec3::new(10., 1.7, -20.);
        let toward = (Vec3::new(10., 0., 0.) - far).normalize();
        assert_eq!(
            pick_segment([(entity, &stream)], far, toward, DISMANTLE_REACH),
            None
        );
    }
}
