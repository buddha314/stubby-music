//! The scripted stream review (a review driver, so it lives in the game).
/// `--stream-review=<dir>`: a scripted pass over the scene's burbler that plans, lays
/// and fails to afford a stream, saving screenshots and `events.jsonl` to `<dir>`. Run
/// it under the graphical watchdog (`tools/jackdaw_timed_test.py`).
use bevy::prelude::*;
use stubby_core::{AssetPart, MachineVoice};
use stubby_streams::*;

mod review {
    use super::*;
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    use std::path::PathBuf;

    #[derive(Resource)]
    pub(super) struct StreamReview {
        dir: PathBuf,
        step: usize,
    }
    pub(super) fn install(app: &mut App) {
        if let Some(dir) =
            std::env::args().find_map(|arg| arg.strip_prefix("--stream-review=").map(PathBuf::from))
        {
            let _ = std::fs::create_dir_all(&dir);
            app.insert_resource(StreamReview { dir, step: 0 });
        }
    }
    fn log(review: &StreamReview, event: serde_json::Value) {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(review.dir.join("events.jsonl"))
        {
            let _ = writeln!(file, "{event}");
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn drive(
        mut commands: Commands,
        time: Res<Time<Real>>,
        mut review: ResMut<StreamReview>,
        mut build: ResMut<StreamBuild>,
        mut cameras: Query<&mut Transform, (With<Camera3d>, Without<stubby_core::HudCamera>)>,
        outlets: Query<(&AssetPart, &GlobalTransform)>,
        mut inventory: Option<ResMut<bevy_stubby_recipes::PlayerInventory>>,
        streams: Query<&LaidStream>,
        bars: Query<&BarLine>,
        mut racks: Query<&mut MachineVoice>,
        mut calendar: ResMut<stubby_world::calendar::Calendar>,
    ) {
        let Some((_, outlet)) = outlets.iter().find(|(part, _)| part.name == "Outlet_0") else {
            return;
        };
        let at = outlet.translation() - Vec3::Y * PORT_HEIGHT;
        let mut dir = outlet.right().as_vec3();
        dir.y = 0.;
        let dir = dir.normalize_or(Vec3::X);
        let side = dir.cross(Vec3::Y);
        let times = [4., 5., 5.5, 7., 7.5, 8.5, 9.5, 11., 12., 13.];
        let t = time.elapsed_secs();
        let Some(&due) = times.get(review.step) else {
            return;
        };
        if t < due {
            return;
        }
        let shoot = |commands: &mut Commands, review: &StreamReview, name: &str| {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(review.dir.join(name)));
        };
        let Ok(mut camera) = cameras.single_mut() else {
            return;
        };
        let look = |camera: &mut Transform, from: Vec3, to: Vec3| {
            *camera = Transform::from_translation(from).looking_at(to, Vec3::Y);
        };
        match review.step {
            // Stand by the mouth with 18 reeds, enter build mode, aim at the mouth.
            0 => {
                // Midday, so the screenshots show the stream (the game opens at dawn).
                calendar.elapsed = std::time::Duration::from_secs(1350);
                if let Some(inventory) = inventory.as_mut() {
                    let _ = inventory.items.grant(&[stubby_recipes::Stack {
                        item: "reed".into(),
                        quantity: 18,
                    }]);
                }
                build.active = true;
                look(&mut camera, at - dir * 4. + side * 3. + Vec3::Y * 1.7, at);
            }
            1 => {
                shoot(&mut commands, &review, "1-port-hover.png");
                build.click = true;
            }
            // Aim off to the side: a curving two-segment plan, affordable.
            2 => look(
                &mut camera,
                at - dir * 3. + side * 4. + Vec3::Y * 2.2,
                at + dir * 7. + side * 4.,
            ),
            3 => shoot(&mut commands, &review, "2-plan-valid.png"),
            4 => {
                build.click = true;
                // Show bar lines without patching anything: their gate stays silent.
                for mut rack in &mut racks {
                    rack.sounding = true;
                }
            }
            // Plan on from the new end: 6 reeds left, too few.
            5 => {
                if let Some(end) = streams.iter().next().map(|s| s.end) {
                    look(
                        &mut camera,
                        end.at - end.dir * 3. + Vec3::Y * 2.2,
                        end.at + end.dir * 9.,
                    );
                }
            }
            6 => shoot(&mut commands, &review, "3-plan-unaffordable.png"),
            // Leave build mode and look over what was laid, then the join at the mouth.
            7 => {
                build.active = false;
                // From the side at eye height (the controller keeps the player grounded).
                look(
                    &mut camera,
                    at + dir * 3. - side * 7. + Vec3::Y * 1.7,
                    at + dir * 4. + side * 2.,
                );
            }
            8 => shoot(&mut commands, &review, "4-laid.png"),
            9 => {
                // From behind the mouth, over the pool, down the stream.
                look(&mut camera, at - dir * 2.5 + Vec3::Y * 2.2, at + dir * 4.);
                shoot(&mut commands, &review, "5-mouth.png");
            }
            _ => {}
        }
        log(
            &review,
            serde_json::json!({
                "step": review.step,
                "seconds": t,
                "active": build.active,
                "hover": build.hover.is_some(),
                "anchor": build.anchor.is_some(),
                "plan": build.plan.as_ref().map(|p| serde_json::json!({
                    "segments": p.segments, "length": p.path.length(),
                    "valid": p.valid, "affordable": p.affordable,
                })),
                "streams": streams.iter().count(),
                "bars": bars.iter().count(),
                "reeds": inventory.as_ref().map(|i| i.items.quantity("reed")),
            }),
        );
        review.step += 1;
    }
}

/// The stream review, when `--stream-review=<dir>` asks for it.
pub struct StreamReviewPlugin;
impl Plugin for StreamReviewPlugin {
    fn build(&self, app: &mut App) {
        review::install(app);
        app.add_systems(
            Update,
            review::drive
                .after(StreamSystems::Input)
                .before(StreamSystems::Plan)
                .run_if(resource_exists::<review::StreamReview>),
        );
    }
}
