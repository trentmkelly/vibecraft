#![allow(dead_code)]

use std::collections::VecDeque;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

pub const TARGET_TPS: u32 = 20;
pub const TICK_DURATION: Duration = Duration::from_millis(50);
pub const MIN_TICK_RATE: f32 = 1.0;
pub const MAX_TICK_RATE: f32 = 10_000.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickMetrics {
    pub tick: u64,
    pub duration: Duration,
    pub over_budget_by: Duration,
    pub drift_from_schedule: Duration,
}

#[derive(Debug)]
pub struct ServerRuntime {
    access_guard: ServerThreadAccess,
    tick: u64,
    next_tick_at: Instant,
    pause_when_empty_after: Duration,
    empty_since: Option<Instant>,
    scheduled: VecDeque<ScheduledTask>,
    metrics: Vec<TickMetrics>,
    samples: RuntimeSamples,
    saves: SaveController,
    tick_rate: TickRateController,
    profiler: RuntimeProfiler,
    status_heartbeat: StatusHeartbeatScheduler,
}

struct ScheduledTask {
    run_at_tick: u64,
    task: Box<dyn FnOnce() + Send>,
}

impl std::fmt::Debug for ScheduledTask {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ScheduledTask")
            .field("run_at_tick", &self.run_at_tick)
            .finish_non_exhaustive()
    }
}

impl ServerRuntime {
    pub fn new(now: Instant, pause_when_empty_after: Duration) -> Self {
        Self {
            access_guard: ServerThreadAccess::current_thread(),
            tick: 0,
            next_tick_at: now,
            pause_when_empty_after,
            empty_since: None,
            scheduled: VecDeque::new(),
            metrics: Vec::new(),
            samples: RuntimeSamples::default(),
            saves: SaveController::default(),
            tick_rate: TickRateController::default(),
            profiler: RuntimeProfiler::default(),
            status_heartbeat: StatusHeartbeatScheduler::disabled(),
        }
    }

    pub fn tick(&self) -> u64 {
        self.access_guard.assert_server_thread();
        self.tick
    }

    pub fn schedule<F>(&mut self, delay_ticks: u64, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.access_guard.assert_server_thread();
        self.scheduled.push_back(ScheduledTask {
            run_at_tick: self.tick + delay_ticks,
            task: Box::new(task),
        });
    }

    pub fn should_pause(&mut self, now: Instant, player_count: usize) -> bool {
        self.access_guard.assert_server_thread();
        if player_count > 0 {
            self.empty_since = None;
            return false;
        }

        let empty_since = *self.empty_since.get_or_insert(now);
        now.duration_since(empty_since) >= self.pause_when_empty_after
    }

    pub fn run_one_tick(&mut self, now: Instant) -> TickMetrics {
        self.access_guard.assert_server_thread();
        let drift = now.saturating_duration_since(self.next_tick_at);
        let started = Instant::now();
        self.tick += 1;

        let mut remaining = VecDeque::new();
        while let Some(task) = self.scheduled.pop_front() {
            if task.run_at_tick <= self.tick {
                (task.task)();
            } else {
                remaining.push_back(task);
            }
        }
        self.scheduled = remaining;

        let duration = started.elapsed();
        let over_budget_by = duration.saturating_sub(TICK_DURATION);
        self.next_tick_at += TICK_DURATION;
        let metrics = TickMetrics {
            tick: self.tick,
            duration,
            over_budget_by,
            drift_from_schedule: drift,
        };
        self.metrics.push(metrics.clone());
        metrics
    }

    pub fn metrics(&self) -> &[TickMetrics] {
        self.access_guard.assert_server_thread();
        &self.metrics
    }

    pub fn samples(&self) -> &RuntimeSamples {
        self.access_guard.assert_server_thread();
        &self.samples
    }

    pub fn record_packet_in(&mut self, bytes: u64) {
        self.access_guard.assert_server_thread();
        self.samples.packets_in += 1;
        self.samples.bytes_in += bytes;
    }

    pub fn record_packet_out(&mut self, bytes: u64) {
        self.access_guard.assert_server_thread();
        self.samples.packets_out += 1;
        self.samples.bytes_out += bytes;
    }

    pub fn record_debug_sample(&mut self, kind: impl Into<String>, value: f64) {
        self.access_guard.assert_server_thread();
        self.samples.debug_samples.push(DebugSample {
            tick: self.tick,
            kind: kind.into(),
            value,
        });
    }

    pub fn saves(&self) -> &SaveController {
        self.access_guard.assert_server_thread();
        &self.saves
    }

    pub fn saves_mut(&mut self) -> &mut SaveController {
        self.access_guard.assert_server_thread();
        &mut self.saves
    }

    pub fn tick_rate(&self) -> &TickRateController {
        self.access_guard.assert_server_thread();
        &self.tick_rate
    }

    pub fn tick_rate_mut(&mut self) -> &mut TickRateController {
        self.access_guard.assert_server_thread();
        &mut self.tick_rate
    }

    pub fn profiler(&self) -> &RuntimeProfiler {
        self.access_guard.assert_server_thread();
        &self.profiler
    }

    pub fn profiler_mut(&mut self) -> &mut RuntimeProfiler {
        self.access_guard.assert_server_thread();
        &mut self.profiler
    }

    pub fn access_guard(&self) -> ServerThreadAccess {
        self.access_guard
    }

    pub fn status_heartbeat(&self) -> &StatusHeartbeatScheduler {
        self.access_guard.assert_server_thread();
        &self.status_heartbeat
    }

    pub fn status_heartbeat_mut(&mut self) -> &mut StatusHeartbeatScheduler {
        self.access_guard.assert_server_thread();
        &mut self.status_heartbeat
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusHeartbeatScheduler {
    interval: Option<Duration>,
    last_heartbeat: Option<Instant>,
    sent: u64,
}

impl StatusHeartbeatScheduler {
    pub fn disabled() -> Self {
        Self {
            interval: None,
            last_heartbeat: None,
            sent: 0,
        }
    }

    pub fn from_seconds(seconds: u32) -> Self {
        if seconds == 0 {
            Self::disabled()
        } else {
            Self {
                interval: Some(Duration::from_secs(u64::from(seconds))),
                last_heartbeat: None,
                sent: 0,
            }
        }
    }

    pub fn interval(&self) -> Option<Duration> {
        self.interval
    }

    pub fn sent_count(&self) -> u64 {
        self.sent
    }

    pub fn poll_due(&mut self, now: Instant) -> bool {
        let Some(interval) = self.interval else {
            return false;
        };
        let Some(last_heartbeat) = self.last_heartbeat else {
            self.last_heartbeat = Some(now);
            self.sent += 1;
            return true;
        };
        if now.duration_since(last_heartbeat) >= interval {
            self.last_heartbeat = Some(now);
            self.sent += 1;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerThreadAccess {
    server_thread: ThreadId,
}

impl ServerThreadAccess {
    pub fn current_thread() -> Self {
        Self {
            server_thread: thread::current().id(),
        }
    }

    pub fn is_server_thread(self) -> bool {
        thread::current().id() == self.server_thread
    }

    pub fn check(self) -> Result<(), ServerThreadViolation> {
        if self.is_server_thread() {
            Ok(())
        } else {
            Err(ServerThreadViolation)
        }
    }

    pub fn assert_server_thread(self) {
        self.check()
            .expect("server state accessed from a non-server thread");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerThreadViolation;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RuntimeSamples {
    pub packets_in: u64,
    pub packets_out: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub debug_samples: Vec<DebugSample>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DebugSample {
    pub tick: u64,
    pub kind: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveController {
    enabled: bool,
    interval_ticks: u64,
    last_save_tick: u64,
    forced_saves: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownSource {
    Console,
    Signal,
    StopCommand,
    ShutdownHook,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShutdownStep {
    AnnounceStopping(ShutdownSource),
    DisconnectPlayers,
    SavePlayers,
    SaveWorlds,
    SaveServerState,
    StopConnections,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShutdownPlan {
    source: ShutdownSource,
    steps: Vec<ShutdownStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Watchdog {
    max_tick_time: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchdogDecision {
    Healthy,
    Disabled,
    Crash { exceeded_by: Duration },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TickRateController {
    tick_rate: f32,
    nanos_per_tick: u64,
    frozen_ticks_to_run: u32,
    run_game_elements: bool,
    frozen: bool,
    remaining_sprint_ticks: u64,
    scheduled_sprint_ticks: u64,
    previous_frozen: bool,
    sprint_time_spent: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SprintReport {
    pub completed_ticks: u64,
    pub ticks_per_second: u64,
    pub milliseconds_per_tick: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeProfiler {
    stack: Vec<OpenProfileSection>,
    samples: Vec<ProfileSample>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OpenProfileSection {
    name: String,
    started_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileSample {
    pub path: String,
    pub duration: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileReport {
    pub samples: Vec<ProfileSample>,
    pub sections: Vec<ProfileSectionSummary>,
    pub open_sections: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileSectionSummary {
    pub path: String,
    pub calls: u64,
    pub total: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AsyncPoolKind {
    Io,
    ChunkGeneration,
    ResourceReload,
    ProfileSession,
}

#[derive(Debug)]
pub struct AsyncTaskPools {
    io: ThreadPool,
    chunk_generation: ThreadPool,
    resource_reload: ThreadPool,
    profile_session: ThreadPool,
}

struct ThreadPool {
    kind: AsyncPoolKind,
    sender: mpsc::Sender<ThreadPoolMessage>,
    workers: Vec<thread::JoinHandle<()>>,
}

enum ThreadPoolMessage {
    Run(Box<dyn FnOnce() + Send + 'static>),
    Shutdown,
}

impl std::fmt::Debug for ThreadPool {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ThreadPool")
            .field("kind", &self.kind)
            .field("workers", &self.workers.len())
            .finish()
    }
}

impl Default for SaveController {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_ticks: 6000,
            last_save_tick: 0,
            forced_saves: 0,
        }
    }
}

impl ShutdownPlan {
    pub fn graceful(source: ShutdownSource) -> Self {
        Self {
            source,
            steps: vec![
                ShutdownStep::AnnounceStopping(source),
                ShutdownStep::DisconnectPlayers,
                ShutdownStep::SavePlayers,
                ShutdownStep::SaveWorlds,
                ShutdownStep::SaveServerState,
                ShutdownStep::StopConnections,
                ShutdownStep::Complete,
            ],
        }
    }

    pub fn source(&self) -> ShutdownSource {
        self.source
    }

    pub fn steps(&self) -> &[ShutdownStep] {
        &self.steps
    }
}

impl Watchdog {
    pub fn new(max_tick_time: Duration) -> Self {
        Self { max_tick_time }
    }

    pub fn from_max_tick_time_millis(max_tick_time_millis: u64) -> Self {
        Self::new(Duration::from_millis(max_tick_time_millis))
    }

    pub fn max_tick_time(&self) -> Duration {
        self.max_tick_time
    }

    pub fn max_tick_time_millis(&self) -> u64 {
        self.max_tick_time.as_millis().min(u128::from(u64::MAX)) as u64
    }

    pub fn check_tick(&self, tick_duration: Duration) -> WatchdogDecision {
        if self.max_tick_time.is_zero() {
            WatchdogDecision::Disabled
        } else if tick_duration > self.max_tick_time {
            WatchdogDecision::Crash {
                exceeded_by: tick_duration - self.max_tick_time,
            }
        } else {
            WatchdogDecision::Healthy
        }
    }
}

impl Default for TickRateController {
    fn default() -> Self {
        Self {
            tick_rate: TARGET_TPS as f32,
            nanos_per_tick: TICK_DURATION.as_nanos() as u64,
            frozen_ticks_to_run: 0,
            run_game_elements: true,
            frozen: false,
            remaining_sprint_ticks: 0,
            scheduled_sprint_ticks: 0,
            previous_frozen: false,
            sprint_time_spent: Duration::ZERO,
        }
    }
}

impl TickRateController {
    pub fn set_tick_rate(&mut self, rate: f32) {
        self.tick_rate = rate.clamp(MIN_TICK_RATE, MAX_TICK_RATE);
        self.nanos_per_tick = (1_000_000_000.0 / self.tick_rate) as u64;
    }

    pub fn tick_rate(&self) -> f32 {
        self.tick_rate
    }

    pub fn tick_duration(&self) -> Duration {
        Duration::from_nanos(self.nanos_per_tick)
    }

    pub fn milliseconds_per_tick(&self) -> f32 {
        self.nanos_per_tick as f32 / 1_000_000.0
    }

    pub fn runs_normally(&self) -> bool {
        self.run_game_elements
    }

    pub fn is_frozen(&self) -> bool {
        self.frozen
    }

    pub fn set_frozen(&mut self, frozen: bool) {
        if frozen {
            self.stop_sprinting();
            self.stop_stepping();
        }
        self.frozen = frozen;
    }

    pub fn is_stepping_forward(&self) -> bool {
        self.frozen_ticks_to_run > 0
    }

    pub fn frozen_ticks_to_run(&self) -> u32 {
        self.frozen_ticks_to_run
    }

    pub fn step_game_if_paused(&mut self, ticks: u32) -> bool {
        if !self.frozen {
            return false;
        }
        self.frozen_ticks_to_run = ticks;
        true
    }

    pub fn stop_stepping(&mut self) -> bool {
        if self.frozen_ticks_to_run == 0 {
            false
        } else {
            self.frozen_ticks_to_run = 0;
            true
        }
    }

    pub fn tick(&mut self) {
        self.run_game_elements = !self.frozen || self.frozen_ticks_to_run > 0;
        if self.frozen_ticks_to_run > 0 {
            self.frozen_ticks_to_run -= 1;
        }
    }

    pub fn request_game_to_sprint(&mut self, ticks: u64) -> bool {
        let interrupted = self.remaining_sprint_ticks > 0;
        self.sprint_time_spent = Duration::ZERO;
        self.scheduled_sprint_ticks = ticks;
        self.remaining_sprint_ticks = ticks;
        self.previous_frozen = self.frozen;
        self.frozen = false;
        interrupted
    }

    pub fn is_sprinting(&self) -> bool {
        self.scheduled_sprint_ticks > 0
    }

    pub fn check_should_sprint_this_tick(&mut self) -> bool {
        if !self.run_game_elements {
            return false;
        }
        if self.remaining_sprint_ticks > 0 {
            self.remaining_sprint_ticks -= 1;
            true
        } else if self.scheduled_sprint_ticks > 0 {
            let _ = self.finish_tick_sprint();
            false
        } else {
            false
        }
    }

    pub fn record_sprint_tick_time(&mut self, duration: Duration) {
        if self.is_sprinting() {
            self.sprint_time_spent += duration;
        }
    }

    pub fn stop_sprinting(&mut self) -> bool {
        self.finish_tick_sprint().is_some()
    }

    pub fn finish_tick_sprint(&mut self) -> Option<SprintReport> {
        if self.scheduled_sprint_ticks == 0 {
            return None;
        }

        let completed_ticks = self
            .scheduled_sprint_ticks
            .saturating_sub(self.remaining_sprint_ticks);
        let millis = self.sprint_time_spent.as_secs_f64().max(0.001) * 1000.0;
        let ticks_per_second = ((1000.0 * completed_ticks as f64) / millis) as u64;
        let milliseconds_per_tick = if completed_ticks == 0 {
            self.milliseconds_per_tick() as f64
        } else {
            millis / completed_ticks as f64
        };

        self.scheduled_sprint_ticks = 0;
        self.remaining_sprint_ticks = 0;
        self.sprint_time_spent = Duration::ZERO;
        self.frozen = self.previous_frozen;

        Some(SprintReport {
            completed_ticks,
            ticks_per_second,
            milliseconds_per_tick,
        })
    }
}

impl RuntimeProfiler {
    pub fn begin_section(&mut self, name: impl Into<String>, now: Instant) {
        self.stack.push(OpenProfileSection {
            name: name.into(),
            started_at: now,
        });
    }

    pub fn end_section(&mut self, now: Instant) -> Option<ProfileSample> {
        let section = self.stack.pop()?;
        let mut path = self
            .stack
            .iter()
            .map(|section| section.name.as_str())
            .collect::<Vec<_>>();
        path.push(section.name.as_str());
        let sample = ProfileSample {
            path: path.join("/"),
            duration: now.saturating_duration_since(section.started_at),
        };
        self.samples.push(sample.clone());
        Some(sample)
    }

    pub fn record_sample(&mut self, path: impl Into<String>, duration: Duration) {
        self.samples.push(ProfileSample {
            path: path.into(),
            duration,
        });
    }

    pub fn clear(&mut self) {
        self.stack.clear();
        self.samples.clear();
    }

    pub fn report(&self) -> ProfileReport {
        let mut sections = Vec::<ProfileSectionSummary>::new();
        for sample in &self.samples {
            if let Some(summary) = sections
                .iter_mut()
                .find(|summary| summary.path == sample.path)
            {
                summary.calls += 1;
                summary.total += sample.duration;
            } else {
                sections.push(ProfileSectionSummary {
                    path: sample.path.clone(),
                    calls: 1,
                    total: sample.duration,
                });
            }
        }
        sections.sort_by(|left, right| {
            right
                .total
                .cmp(&left.total)
                .then_with(|| left.path.cmp(&right.path))
        });

        ProfileReport {
            samples: self.samples.clone(),
            sections,
            open_sections: self
                .stack
                .iter()
                .map(|section| section.name.clone())
                .collect(),
        }
    }
}

impl AsyncTaskPools {
    pub fn vanilla_layout() -> Self {
        Self::with_worker_counts(2, 2, 1, 2)
    }

    pub fn with_worker_counts(
        io_workers: usize,
        chunk_workers: usize,
        reload_workers: usize,
        profile_session_workers: usize,
    ) -> Self {
        Self {
            io: ThreadPool::new(AsyncPoolKind::Io, io_workers.max(1)),
            chunk_generation: ThreadPool::new(AsyncPoolKind::ChunkGeneration, chunk_workers.max(1)),
            resource_reload: ThreadPool::new(AsyncPoolKind::ResourceReload, reload_workers.max(1)),
            profile_session: ThreadPool::new(
                AsyncPoolKind::ProfileSession,
                profile_session_workers.max(1),
            ),
        }
    }

    pub fn execute<F, R>(&self, kind: AsyncPoolKind, task: F) -> mpsc::Receiver<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        match kind {
            AsyncPoolKind::Io => self.io.execute(task),
            AsyncPoolKind::ChunkGeneration => self.chunk_generation.execute(task),
            AsyncPoolKind::ResourceReload => self.resource_reload.execute(task),
            AsyncPoolKind::ProfileSession => self.profile_session.execute(task),
        }
    }
}

impl ThreadPool {
    fn new(kind: AsyncPoolKind, worker_count: usize) -> Self {
        let (sender, receiver) = mpsc::channel::<ThreadPoolMessage>();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(worker_count);

        for worker_index in 0..worker_count {
            let receiver = Arc::clone(&receiver);
            let name = format!("rustcraft-{kind:?}-{worker_index}");
            workers.push(
                thread::Builder::new()
                    .name(name)
                    .spawn(move || loop {
                        let message = receiver.lock().unwrap().recv();
                        match message {
                            Ok(ThreadPoolMessage::Run(task)) => task(),
                            Ok(ThreadPoolMessage::Shutdown) | Err(_) => break,
                        }
                    })
                    .expect("failed to spawn async runtime worker"),
            );
        }

        Self {
            kind,
            sender,
            workers,
        }
    }

    fn execute<F, R>(&self, task: F) -> mpsc::Receiver<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let (result_sender, result_receiver) = mpsc::channel();
        self.sender
            .send(ThreadPoolMessage::Run(Box::new(move || {
                let _ = result_sender.send(task());
            })))
            .expect("async runtime worker pool has stopped");
        result_receiver
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            let _ = self.sender.send(ThreadPoolMessage::Shutdown);
        }
        while let Some(worker) = self.workers.pop() {
            let _ = worker.join();
        }
    }
}

impl SaveController {
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn save_on(&mut self) {
        self.enabled = true;
    }

    pub fn save_off(&mut self) {
        self.enabled = false;
    }

    pub fn force_save(&mut self, tick: u64) {
        self.last_save_tick = tick;
        self.forced_saves += 1;
    }

    pub fn should_autosave(&self, tick: u64) -> bool {
        self.enabled && tick.saturating_sub(self.last_save_tick) >= self.interval_ticks
    }

    pub fn mark_autosaved(&mut self, tick: u64) {
        self.last_save_tick = tick;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AsyncPoolKind, AsyncTaskPools, ServerRuntime, ServerThreadAccess, ShutdownPlan,
        ShutdownSource, ShutdownStep, StatusHeartbeatScheduler, TickRateController, Watchdog,
        WatchdogDecision, MAX_TICK_RATE, MIN_TICK_RATE, TARGET_TPS, TICK_DURATION,
    };
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    #[test]
    fn exposes_twenty_tps_target() {
        assert_eq!(TARGET_TPS, 20);
        assert_eq!(TICK_DURATION, Duration::from_millis(50));
    }

    #[test]
    fn executes_scheduled_tasks_on_target_tick() {
        let now = Instant::now();
        let mut runtime = ServerRuntime::new(now, Duration::from_secs(60));
        let value = Arc::new(Mutex::new(0));
        let task_value = Arc::clone(&value);
        runtime.schedule(2, move || {
            *task_value.lock().unwrap() = 10;
        });

        runtime.run_one_tick(now);
        assert_eq!(*value.lock().unwrap(), 0);
        runtime.run_one_tick(now + TICK_DURATION);
        assert_eq!(*value.lock().unwrap(), 10);
    }

    #[test]
    fn tracks_pause_when_empty_after_threshold() {
        let now = Instant::now();
        let mut runtime = ServerRuntime::new(now, Duration::from_secs(60));
        assert!(!runtime.should_pause(now, 0));
        assert!(!runtime.should_pause(now + Duration::from_secs(59), 0));
        assert!(runtime.should_pause(now + Duration::from_secs(60), 0));
        assert!(!runtime.should_pause(now + Duration::from_secs(61), 1));
    }

    #[test]
    fn records_tick_drift_and_budget_metrics() {
        let now = Instant::now();
        let mut runtime = ServerRuntime::new(now, Duration::from_secs(60));
        let metrics = runtime.run_one_tick(now + Duration::from_millis(5));
        assert_eq!(metrics.tick, 1);
        assert_eq!(metrics.drift_from_schedule, Duration::from_millis(5));
        assert_eq!(runtime.metrics().len(), 1);
    }

    #[test]
    fn server_thread_access_rejects_foreign_threads() {
        let guard = ServerThreadAccess::current_thread();
        assert!(guard.check().is_ok());

        let result = std::thread::spawn(move || guard.check()).join().unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn runtime_carries_main_thread_access_guard() {
        let runtime = ServerRuntime::new(Instant::now(), Duration::from_secs(60));
        let guard = runtime.access_guard();

        assert!(guard.is_server_thread());
        assert!(!std::thread::spawn(move || guard.is_server_thread())
            .join()
            .unwrap());
    }

    #[test]
    fn records_bandwidth_packet_and_debug_samples() {
        let now = Instant::now();
        let mut runtime = ServerRuntime::new(now, Duration::from_secs(60));
        runtime.record_packet_in(10);
        runtime.record_packet_out(15);
        runtime.record_debug_sample("tick_time", 1.5);

        assert_eq!(runtime.samples().packets_in, 1);
        assert_eq!(runtime.samples().packets_out, 1);
        assert_eq!(runtime.samples().bytes_in, 10);
        assert_eq!(runtime.samples().bytes_out, 15);
        assert_eq!(runtime.samples().debug_samples[0].kind, "tick_time");
    }

    #[test]
    fn status_heartbeat_scheduler_matches_dedicated_server_interval_rules() {
        let now = Instant::now();
        let mut disabled = StatusHeartbeatScheduler::from_seconds(0);
        assert_eq!(disabled.interval(), None);
        assert!(!disabled.poll_due(now));
        assert_eq!(disabled.sent_count(), 0);

        let mut scheduler = StatusHeartbeatScheduler::from_seconds(2);
        assert_eq!(scheduler.interval(), Some(Duration::from_secs(2)));
        assert!(scheduler.poll_due(now));
        assert_eq!(scheduler.sent_count(), 1);
        assert!(!scheduler.poll_due(now + Duration::from_millis(1999)));
        assert_eq!(scheduler.sent_count(), 1);
        assert!(scheduler.poll_due(now + Duration::from_secs(2)));
        assert_eq!(scheduler.sent_count(), 2);
        assert!(scheduler.poll_due(now + Duration::from_secs(4)));
        assert_eq!(scheduler.sent_count(), 3);
    }

    #[test]
    fn runtime_profiler_tracks_nested_sections_and_aggregates_report() {
        let now = Instant::now();
        let mut runtime = ServerRuntime::new(now, Duration::from_secs(60));

        runtime.profiler_mut().begin_section("root", now);
        runtime
            .profiler_mut()
            .begin_section("world_tick", now + Duration::from_millis(1));
        let sample = runtime
            .profiler_mut()
            .end_section(now + Duration::from_millis(6))
            .unwrap();
        runtime
            .profiler_mut()
            .record_sample("root/world_tick", Duration::from_millis(3));
        runtime
            .profiler_mut()
            .end_section(now + Duration::from_millis(10))
            .unwrap();

        assert_eq!(sample.path, "root/world_tick");
        assert_eq!(sample.duration, Duration::from_millis(5));

        let report = runtime.profiler().report();
        let world = report
            .sections
            .iter()
            .find(|section| section.path == "root/world_tick")
            .unwrap();
        assert_eq!(world.calls, 2);
        assert_eq!(world.total, Duration::from_millis(8));
        assert!(report.open_sections.is_empty());
    }

    #[test]
    fn runtime_profiler_reports_open_sections_for_crash_and_perf_output() {
        let now = Instant::now();
        let mut runtime = ServerRuntime::new(now, Duration::from_secs(60));

        runtime.profiler_mut().begin_section("tick", now);
        runtime
            .profiler_mut()
            .begin_section("scheduled_tasks", now + Duration::from_millis(1));

        let report = runtime.profiler().report();
        assert_eq!(report.open_sections, vec!["tick", "scheduled_tasks"]);

        runtime.profiler_mut().clear();
        assert!(runtime.profiler().report().open_sections.is_empty());
    }

    #[test]
    fn controls_save_on_save_off_force_save_and_autosave() {
        let now = Instant::now();
        let mut runtime = ServerRuntime::new(now, Duration::from_secs(60));
        assert!(runtime.saves().enabled());
        assert!(!runtime.saves().should_autosave(5999));
        assert!(runtime.saves().should_autosave(6000));

        runtime.saves_mut().save_off();
        assert!(!runtime.saves().enabled());
        assert!(!runtime.saves().should_autosave(12000));

        runtime.saves_mut().save_on();
        runtime.saves_mut().force_save(12000);
        assert!(!runtime.saves().should_autosave(17999));
        assert!(runtime.saves().should_autosave(18000));
    }

    #[test]
    fn tick_rate_controller_clamps_rate_and_updates_target_duration() {
        let mut controller = TickRateController::default();

        controller.set_tick_rate(40.0);
        assert_eq!(controller.tick_rate(), 40.0);
        assert_eq!(controller.tick_duration(), Duration::from_millis(25));

        controller.set_tick_rate(0.25);
        assert_eq!(controller.tick_rate(), MIN_TICK_RATE);

        controller.set_tick_rate(20_000.0);
        assert_eq!(controller.tick_rate(), MAX_TICK_RATE);
    }

    #[test]
    fn tick_rate_controller_freezes_steps_and_resumes_game_elements() {
        let mut controller = TickRateController::default();

        controller.set_frozen(true);
        controller.tick();
        assert!(!controller.runs_normally());
        assert!(controller.step_game_if_paused(2));

        controller.tick();
        assert!(controller.runs_normally());
        assert_eq!(controller.frozen_ticks_to_run(), 1);

        controller.tick();
        assert!(controller.runs_normally());
        assert_eq!(controller.frozen_ticks_to_run(), 0);

        controller.tick();
        assert!(!controller.runs_normally());

        controller.set_frozen(false);
        controller.tick();
        assert!(controller.runs_normally());
    }

    #[test]
    fn tick_rate_controller_sprints_and_restores_previous_freeze_state() {
        let mut controller = TickRateController::default();
        controller.set_frozen(true);

        assert!(!controller.request_game_to_sprint(2));
        assert!(controller.is_sprinting());
        assert!(!controller.is_frozen());

        controller.tick();
        assert!(controller.check_should_sprint_this_tick());
        controller.record_sprint_tick_time(Duration::from_millis(2));
        controller.tick();
        assert!(controller.check_should_sprint_this_tick());
        controller.record_sprint_tick_time(Duration::from_millis(2));

        let report = controller.finish_tick_sprint().unwrap();
        assert_eq!(report.completed_ticks, 2);
        assert_eq!(report.milliseconds_per_tick, 2.0);
        assert!(controller.is_frozen());
        assert!(!controller.is_sprinting());
    }

    #[test]
    fn freeze_interrupts_step_and_sprint_modes() {
        let mut controller = TickRateController::default();
        controller.set_frozen(true);
        assert!(controller.step_game_if_paused(5));
        controller.set_frozen(false);
        controller.request_game_to_sprint(10);

        controller.set_frozen(true);

        assert_eq!(controller.frozen_ticks_to_run(), 0);
        assert!(!controller.is_sprinting());
        assert!(controller.is_frozen());
    }

    #[test]
    fn graceful_shutdown_plan_matches_vanilla_ordering_sources() {
        for source in [
            ShutdownSource::Console,
            ShutdownSource::Signal,
            ShutdownSource::StopCommand,
            ShutdownSource::ShutdownHook,
        ] {
            let plan = ShutdownPlan::graceful(source);
            assert_eq!(plan.source(), source);
            assert_eq!(
                plan.steps(),
                &[
                    ShutdownStep::AnnounceStopping(source),
                    ShutdownStep::DisconnectPlayers,
                    ShutdownStep::SavePlayers,
                    ShutdownStep::SaveWorlds,
                    ShutdownStep::SaveServerState,
                    ShutdownStep::StopConnections,
                    ShutdownStep::Complete,
                ]
            );
        }
    }

    #[test]
    fn watchdog_crashes_when_tick_exceeds_max_tick_time() {
        let watchdog = Watchdog::new(Duration::from_secs(60));
        assert_eq!(
            watchdog.check_tick(Duration::from_secs(1)),
            WatchdogDecision::Healthy
        );
        assert_eq!(
            watchdog.check_tick(Duration::from_secs(61)),
            WatchdogDecision::Crash {
                exceeded_by: Duration::from_secs(1)
            }
        );
        assert_eq!(
            Watchdog::new(Duration::ZERO).check_tick(Duration::from_secs(999)),
            WatchdogDecision::Disabled
        );
    }

    #[test]
    fn watchdog_uses_server_property_milliseconds() {
        let watchdog = Watchdog::from_max_tick_time_millis(12_345);
        assert_eq!(watchdog.max_tick_time(), Duration::from_millis(12_345));
        assert_eq!(watchdog.max_tick_time_millis(), 12_345);
        assert_eq!(
            watchdog.check_tick(Duration::from_millis(12_346)),
            WatchdogDecision::Crash {
                exceeded_by: Duration::from_millis(1)
            }
        );
    }

    #[test]
    fn async_task_pools_route_work_to_dedicated_services() {
        let pools = AsyncTaskPools::with_worker_counts(1, 1, 1, 1);

        let io = pools.execute(AsyncPoolKind::Io, || "io");
        let chunk = pools.execute(AsyncPoolKind::ChunkGeneration, || "chunk");
        let reload = pools.execute(AsyncPoolKind::ResourceReload, || "reload");
        let profile = pools.execute(AsyncPoolKind::ProfileSession, || "profile");

        assert_eq!(io.recv_timeout(Duration::from_secs(1)).unwrap(), "io");
        assert_eq!(chunk.recv_timeout(Duration::from_secs(1)).unwrap(), "chunk");
        assert_eq!(
            reload.recv_timeout(Duration::from_secs(1)).unwrap(),
            "reload"
        );
        assert_eq!(
            profile.recv_timeout(Duration::from_secs(1)).unwrap(),
            "profile"
        );
    }

    #[test]
    fn async_task_pools_clamp_empty_worker_counts() {
        let pools = AsyncTaskPools::with_worker_counts(0, 0, 0, 0);
        let result = pools.execute(AsyncPoolKind::ResourceReload, || 26);

        assert_eq!(result.recv_timeout(Duration::from_secs(1)).unwrap(), 26);
    }
}
