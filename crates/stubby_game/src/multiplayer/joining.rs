//! Joining behind the loading screen (task 3.7, open question 5: a short wait is fine): a
//! client that joins sees the loading screen until the host's world so far has arrived
//! (its world, clock, depleted sources and racks) and its own body is here, then plays. A
//! slow link gives up after a while and plays with what has come.
use bevy::prelude::*;
use stubby_core::{Screen, ScreenMode};
use stubby_net::{JoinedHost, lightyear::prelude::client};

use super::{HostWorld, collecting::DepletedSources, racks::Racks, time::HostClock};
use stubby_player::movement::MoveBody;

/// How long a join may wait for the host's world before playing anyway.
pub const CATCH_UP_TIMEOUT_S: f32 = 20.;

/// This client joined and is waiting for the host's world: the loading screen shows.
#[derive(Resource, Default)]
pub struct CatchingUp {
    waited: f32,
}

pub struct JoiningPlugin;

impl Plugin for JoiningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                start_catching_up.run_if(resource_added::<JoinedHost>),
                caught_up.run_if(resource_exists::<CatchingUp>),
            )
                .chain(),
        );
    }
}

fn start_catching_up(mut screens: ResMut<ScreenMode>, mut commands: Commands) {
    commands.insert_resource(CatchingUp::default());
    screens.open = Some(Screen::Loading);
}

/// Everything the host shares with a joining player, and its own body.
#[allow(clippy::type_complexity)]
fn caught_up(
    tables: (
        Query<(), (With<HostWorld>, With<client::Remote>)>,
        Query<(), (With<HostClock>, With<client::Remote>)>,
        Query<(), (With<DepletedSources>, With<client::Remote>)>,
        Query<(), (With<Racks>, With<client::Remote>)>,
    ),
    bodies: Query<(), With<MoveBody>>,
    time: Res<Time<Real>>,
    mut catching_up: ResMut<CatchingUp>,
    mut screens: ResMut<ScreenMode>,
    mut commands: Commands,
) {
    catching_up.waited += time.delta_secs();

    let (world, clock, depleted, racks) = tables;
    let arrived = !world.is_empty()
        && !clock.is_empty()
        && !depleted.is_empty()
        && !racks.is_empty()
        && !bodies.is_empty();
    let timed_out = catching_up.waited > CATCH_UP_TIMEOUT_S;
    if timed_out && !arrived {
        warn!("The host's world has not all arrived after {CATCH_UP_TIMEOUT_S} s; playing");
    }
    if arrived || timed_out {
        info!("Caught up with the host after {:.1} s", catching_up.waited);
        commands.remove_resource::<CatchingUp>();
        if screens.open == Some(Screen::Loading) {
            screens.close();
        }
    }
}
