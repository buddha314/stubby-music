//! Time in a shared world (task 3.6, design D6 as revised 2026-11-27): the host's calendar
//! is everyone's, and in company time runs at 1x. The host publishes its calendar (about
//! once a second); a client keeps its own, set to the host's when they drift apart by more
//! than a second. The sky and everything timed follow the calendar.
//!
//! Why 1x in company: on a client lightyear sets Bevy's virtual-time speed itself every
//! frame (timeline sync), and a host ticking faster than real time would pull its clients'
//! timelines along. Speeding time (the speed keys) is for a world of one: they work alone,
//! and a player joining slows it to 1x (`stubby_core::InCompany`).
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use stubby_core::InCompany;
use stubby_net::{
    JoinedHost,
    lightyear::prelude::{server::Started, *},
};
use stubby_world::calendar::Calendar;

/// How far a client's calendar may wander from the host's before it is set to it.
pub const DRIFT_S: f64 = 1.0;
/// The host republishes its calendar this often.
const PUBLISH_EVERY_S: f64 = 1.0;

/// The host's calendar: seconds since the world began.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HostClock {
    pub elapsed: f64,
}

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.component::<HostClock>().replicate();
        app.add_observer(publish_clock).add_systems(
            Update,
            (
                (keep_company, keep_clock).run_if(not(resource_exists::<JoinedHost>)),
                (joined_company, follow_clock).run_if(resource_exists::<JoinedHost>),
            ),
        );
    }
}

fn publish_clock(_: On<Add<Started>>, mut commands: Commands) {
    commands.spawn((
        Name::new("Host clock"),
        HostClock::default(),
        Replicate::to_clients(NetworkTarget::All),
    ));
}

/// The host is in company while a remote player is connected.
fn keep_company(
    players: Query<&RemoteId, (With<server::ClientOf>, With<Connected>)>,
    company: Option<Res<InCompany>>,
    mut commands: Commands,
) {
    let others = players.iter().any(|id| !id.0.is_local());
    match (others, company.is_some()) {
        (true, false) => commands.insert_resource(InCompany),
        (false, true) => commands.remove_resource::<InCompany>(),
        _ => {}
    }
}

fn joined_company(company: Option<Res<InCompany>>, mut commands: Commands) {
    if company.is_none() {
        commands.insert_resource(InCompany);
    }
}

fn keep_clock(calendar: Option<Res<Calendar>>, mut clocks: Query<&mut HostClock>) {
    let Some(calendar) = calendar else {
        return;
    };
    let elapsed = calendar.elapsed().as_secs_f64();
    for mut clock in &mut clocks {
        if (elapsed - clock.elapsed).abs() >= PUBLISH_EVERY_S {
            clock.elapsed = elapsed;
        }
    }
}

fn follow_clock(
    clocks: Query<Ref<HostClock>, With<client::Remote>>,
    calendar: Option<ResMut<Calendar>>,
) {
    let (Ok(clock), Some(mut calendar)) = (clocks.single(), calendar) else {
        return;
    };
    if clock.is_changed() && (calendar.elapsed().as_secs_f64() - clock.elapsed).abs() > DRIFT_S {
        calendar.elapsed = std::time::Duration::from_secs_f64(clock.elapsed.max(0.));
    }
}
