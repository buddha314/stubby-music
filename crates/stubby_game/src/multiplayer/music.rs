//! Music together (task 4.2, design D7 and open question 4: music is tied to locations,
//! so it is universally synced). Every machine clock derives its beat from its engine's
//! master clock (`lock-clocks-to-master`), so peers hear the same bar when their master
//! clocks agree. The host publishes its master beat against lightyear's tick; a client
//! projects it to now on its estimate of the host's timeline, and shifts its own master
//! clock (`MasterOffset`) when they drift apart. How far apart they are is kept
//! (`MusicSync`) and logged, so the drift is measured, not assumed.
use bevy::prelude::*;
use bevy_stubby_audio::{MasterClock, MasterOffset};
use serde::{Deserialize, Serialize};
use stubby_net::{
    JoinedHost,
    lightyear::prelude::{server::Started, *},
};

/// The host republishes its beat this often.
const PUBLISH_EVERY_S: f32 = 0.5;
/// A client corrects its clock once its (smoothed) error passes this many seconds: well
/// under what an ear hears as two players out of time.
pub const CORRECT_AT_S: f64 = 0.015;
/// How much each new measurement moves the smoothed error (the timeline estimate jitters).
const SMOOTHING: f64 = 0.3;

/// The host's master clock: `beats` at its tick instant `tick` (whole ticks and the
/// fraction towards the next).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HostBeat {
    pub tick: f64,
    pub beats: f64,
}

/// How far this client's music is from the host's: what the drift report reads.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct MusicSync {
    /// The last measured error, seconds (positive: the host is ahead).
    pub error_s: f64,
    /// The smoothed error the corrections act on.
    pub smoothed_s: f64,
    /// The largest error measured after the first correction.
    pub worst_s: f64,
    pub measurements: u32,
    pub corrections: u32,
}

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.component::<HostBeat>().replicate();
        app.init_resource::<MasterOffset>()
            .init_resource::<MusicSync>()
            .add_observer(publish_beat)
            // After the fixed steps: the timelines have taken this frame's time, as the
            // master clock (read in PreUpdate) has.
            .add_systems(
                Update,
                (
                    keep_beat.run_if(not(resource_exists::<JoinedHost>)),
                    (follow_beat, report_drift)
                        .chain()
                        .run_if(resource_exists::<JoinedHost>),
                ),
            );
    }
}

fn publish_beat(_: On<Add<Started>>, mut commands: Commands) {
    commands.spawn((
        Name::new("Host beat"),
        HostBeat::default(),
        Replicate::to_clients(NetworkTarget::All),
    ));
}

fn keep_beat(
    clock: Option<Res<MasterClock>>,
    timeline: Option<Res<LocalTimeline>>,
    fixed: Res<Time<Fixed>>,
    time: Res<Time<Real>>,
    mut beats: Query<&mut HostBeat>,
    mut since: Local<f32>,
) {
    *since += time.delta_secs();
    let (Some(clock), Some(timeline)) = (clock, timeline) else {
        return;
    };
    let Some(now) = clock.beats() else {
        return;
    };
    if *since < PUBLISH_EVERY_S {
        return;
    }
    *since = 0.;
    let tick = timeline.instant(&fixed).value.to_num::<f64>();
    for mut beat in &mut beats {
        *beat = HostBeat { tick, beats: now };
    }
}

/// The host's beat now: its published beat, carried forward by the ticks since, on this
/// client's estimate of the host's timeline (`remote_tick`). Master beats run at 1 BPM.
pub fn host_beats_now(beat: HostBeat, remote_tick: f64, tick_s: f64) -> f64 {
    beat.beats + (remote_tick - beat.tick) * tick_s / 60.
}

fn follow_beat(
    beats: Query<Ref<HostBeat>, With<client::Remote>>,
    timelines: Query<&client::RemoteTimeline, (With<client::Client>, Without<LinkOf>)>,
    clock: Option<Res<MasterClock>>,
    mut offset: ResMut<MasterOffset>,
    mut sync: ResMut<MusicSync>,
) {
    let (Ok(beat), Ok(timeline), Some(local)) = (
        beats.single(),
        timelines.single(),
        clock.and_then(|c| c.beats()),
    ) else {
        return;
    };
    if !beat.is_changed() || *beat == HostBeat::default() {
        return;
    }
    let remote_tick = timeline.now.value.to_num::<f64>();
    let tick_s = stubby_net::tick_duration().as_secs_f64();
    let error_s = (host_beats_now(*beat, remote_tick, tick_s) - local) * 60.;
    sync.error_s = error_s;
    sync.measurements += 1;
    if sync.corrections == 0 {
        // The first measurement sets the clock: the two engines started anywhere apart.
        offset.0 += error_s;
        sync.corrections = 1;
        sync.smoothed_s = 0.;
        info!("Music follows the host: master clock moved {error_s:+.3} s");
        return;
    }
    sync.worst_s = sync.worst_s.max(error_s.abs());
    sync.smoothed_s += (error_s - sync.smoothed_s) * SMOOTHING;
    if sync.smoothed_s.abs() > CORRECT_AT_S {
        offset.0 += sync.smoothed_s;
        sync.corrections += 1;
        info!(
            "Music drifted {:+.1} ms from the host; corrected ({} corrections, worst {:.1} ms)",
            sync.smoothed_s * 1000.,
            sync.corrections,
            sync.worst_s * 1000.
        );
        sync.smoothed_s = 0.;
    }
}

/// Every half minute, how far this client's music is from the host's: the drift report.
fn report_drift(sync: Res<MusicSync>, time: Res<Time<Real>>, mut since: Local<f32>) {
    *since += time.delta_secs();
    if *since < 30. || sync.measurements == 0 {
        return;
    }
    *since = 0.;
    info!(
        "Music is {:+.1} ms from the host ({} measurements, {} corrections, worst {:.1} ms)",
        sync.error_s * 1000.,
        sync.measurements,
        sync.corrections,
        sync.worst_s * 1000.
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hosts_beat_is_carried_forward_by_the_ticks_since() {
        let beat = HostBeat {
            tick: 1000.,
            beats: 10.,
        };
        // 64 ticks of 1/64 s is a second: a sixtieth of a 1 BPM beat.
        let now = host_beats_now(beat, 1064., 1. / 64.);
        assert!((now - (10. + 1. / 60.)).abs() < 1e-9);
    }
}
