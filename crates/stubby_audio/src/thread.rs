//! The engine on its own control thread, behind a handle the game can share.
//!
//! `Engine` is not `Send` (Firewheel's context isn't), so it lives its whole life on
//! one thread. `AudioThread` starts that thread and gives the game a `Send + Sync`
//! handle: send commands, `commit` once per frame, and drain notices. A slow or
//! stalled game frame never delays the engine's own housekeeping, and the game never
//! touches Firewheel types.

use crate::engine::{Engine, EngineConfig, OUTPUT_CHANNELS, OfflineRenderer};
use crate::protocol::{Command, EngineError, Event};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// How often the control thread wakes when the game sends nothing.
const TICK: Duration = Duration::from_millis(2);

/// Something the control thread reports back to the game.
#[derive(Debug, Clone, PartialEq)]
pub enum Notice {
    /// An event from the audio thread, such as a clock tick.
    Event(Event),
    /// A committed command the engine refused. The rest of the frame still applied.
    Rejected {
        command: Command,
        error: EngineError,
    },
    /// A flush failed (for example the queue was full); edits stay queued.
    Error(EngineError),
}

enum Request {
    Command(Command),
    LoadSample(crate::protocol::SampleId, u32, Vec<Vec<f32>>),
    Commit,
    Shutdown,
}

/// A reading of the audio clock and the wall-clock instant it was taken at.
#[derive(Debug, Clone, Copy)]
struct ClockSnapshot {
    samples: u64,
    at: Instant,
    sample_rate: u32,
}

/// Lock-free counters the control thread publishes for diagnostics and tests.
#[derive(Default)]
struct Stats {
    /// Frames rendered by the offline renderer (0 with a device).
    frames_rendered: AtomicU64,
    /// Peak absolute sample of the most recent offline block, as `f32` bits.
    last_block_peak: AtomicU32,
}

pub struct AudioThread {
    requests: mpsc::Sender<Request>,
    notices: Mutex<mpsc::Receiver<Notice>>,
    clock: Arc<Mutex<Option<ClockSnapshot>>>,
    stats: Arc<Stats>,
    join: Option<JoinHandle<()>>,
}

impl AudioThread {
    /// Start the engine on the default output device.
    pub fn start() -> Result<Self, EngineError> {
        Self::spawn(|| Engine::start().map(|engine| (engine, None)))
    }

    /// Start the engine with no device. The control thread renders in real time
    /// itself, standing in for the audio callback. For tests and headless runs.
    pub fn start_offline(config: EngineConfig) -> Result<Self, EngineError> {
        Self::spawn(move || {
            Engine::offline(config).map(|(engine, renderer)| (engine, Some(renderer)))
        })
    }

    fn spawn<F>(make: F) -> Result<Self, EngineError>
    where
        F: FnOnce() -> Result<(Engine, Option<OfflineRenderer>), EngineError> + Send + 'static,
    {
        let (requests, request_rx) = mpsc::channel();
        let (notice_tx, notices) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let clock = Arc::new(Mutex::new(None));
        let stats = Arc::new(Stats::default());
        let (thread_clock, thread_stats) = (clock.clone(), stats.clone());

        let join = std::thread::Builder::new()
            .name("stubby-audio-control".into())
            .spawn(move || match make() {
                Ok((engine, renderer)) => {
                    let _ = ready_tx.send(Ok(()));
                    run(
                        engine,
                        renderer,
                        request_rx,
                        notice_tx,
                        thread_clock,
                        thread_stats,
                    );
                }
                Err(e) => {
                    let _ = ready_tx.send(Err(e));
                }
            })
            .map_err(|e| EngineError::Device(e.to_string()))?;

        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Self {
                requests,
                notices: Mutex::new(notices),
                clock,
                stats,
                join: Some(join),
            }),
            Ok(Err(e)) => {
                let _ = join.join();
                Err(e)
            }
            Err(_) => Err(EngineError::Device(
                "audio control thread exited during startup".into(),
            )),
        }
    }

    /// Stage a command. It is applied, with the rest of the frame's commands, at the
    /// next [`commit`](Self::commit). Never blocks.
    /// Load a one-shot sample (one buffer per channel, recorded at `sample_rate`) for
    /// `Command::PlaySample`. Applied before any later command.
    pub fn load_sample(
        &self,
        id: crate::protocol::SampleId,
        sample_rate: u32,
        channels: Vec<Vec<f32>>,
    ) {
        let _ = self
            .requests
            .send(Request::LoadSample(id, sample_rate, channels));
    }

    pub fn send(&self, command: Command) {
        let _ = self.requests.send(Request::Command(command));
    }

    /// Apply everything sent since the last commit, as one batch. Call once per frame.
    pub fn commit(&self) {
        let _ = self.requests.send(Request::Commit);
    }

    /// Move every pending notice into `out`.
    pub fn drain(&self, out: &mut Vec<Notice>) {
        if let Ok(rx) = self.notices.lock() {
            out.extend(rx.try_iter());
        }
    }

    pub fn sample_rate(&self) -> Option<u32> {
        self.snapshot().map(|s| s.sample_rate)
    }

    /// Predict the wall-clock instant at which audio frame `sample_time` is processed.
    pub fn sample_to_instant(&self, sample_time: u64) -> Option<Instant> {
        let s = self.snapshot()?;
        let delta = (sample_time as f64 - s.samples as f64) / s.sample_rate as f64;
        if delta >= 0.0 {
            s.at.checked_add(Duration::from_secs_f64(delta))
        } else {
            s.at.checked_sub(Duration::from_secs_f64(-delta))
        }
    }

    /// Frames the offline renderer has produced (always 0 with a real device).
    pub fn offline_frames_rendered(&self) -> u64 {
        self.stats.frames_rendered.load(Ordering::Relaxed)
    }

    /// Peak absolute sample of the most recent offline block.
    pub fn offline_last_block_peak(&self) -> f32 {
        f32::from_bits(self.stats.last_block_peak.load(Ordering::Relaxed))
    }

    fn snapshot(&self) -> Option<ClockSnapshot> {
        self.clock.lock().ok().and_then(|c| *c)
    }
}

impl Drop for AudioThread {
    fn drop(&mut self) {
        let _ = self.requests.send(Request::Shutdown);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn run(
    mut engine: Engine,
    mut renderer: Option<OfflineRenderer>,
    requests: mpsc::Receiver<Request>,
    notices: mpsc::Sender<Notice>,
    clock: Arc<Mutex<Option<ClockSnapshot>>>,
    stats: Arc<Stats>,
) {
    let mut staged = Vec::new();
    let mut events = Vec::new();
    let mut block = vec![0.0f32; 256 * OUTPUT_CHANNELS];
    let started = Instant::now();
    loop {
        let mut next = match requests.recv_timeout(TICK) {
            Ok(request) => Some(request),
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => return,
        };
        while let Some(request) = next.take() {
            match request {
                Request::Command(command) => staged.push(command),
                Request::LoadSample(id, rate, channels) => engine.load_sample(id, rate, channels),
                Request::Commit => {
                    for command in staged.drain(..) {
                        if let Err(error) = engine.send(command) {
                            let _ = notices.send(Notice::Rejected { command, error });
                        }
                    }
                }
                Request::Shutdown => return,
            }
            next = requests.try_recv().ok();
        }

        // Publishes committed edits, and lets Firewheel free removed nodes.
        if let Err(error) = engine.flush() {
            let _ = notices.send(Notice::Error(error));
        }

        // Offline: render as many blocks as wall-clock time calls for, like a device
        // would, so the audio clock and real time agree.
        if let (Some(renderer), Some(sample_rate)) = (&mut renderer, engine.sample_rate()) {
            let due = (started.elapsed().as_secs_f64() * sample_rate as f64) as u64;
            while renderer.frames_rendered() < due {
                renderer.render(&mut block);
                let peak = block.iter().fold(0.0f32, |m, s| m.max(s.abs()));
                stats
                    .last_block_peak
                    .store(peak.to_bits(), Ordering::Relaxed);
            }
            stats
                .frames_rendered
                .store(renderer.frames_rendered(), Ordering::Relaxed);
        }

        engine.drain_events(&mut events);
        for event in events.drain(..) {
            let _ = notices.send(Notice::Event(event));
        }

        if let Some(sample_rate) = engine.sample_rate() {
            let snapshot = ClockSnapshot {
                samples: engine.audio_clock(),
                at: Instant::now(),
                sample_rate,
            };
            if let Ok(mut c) = clock.lock() {
                *c = Some(snapshot);
            }
        }
    }
}
