use super::{
    AsyncPoolKind, AsyncTaskPools, ServerRuntime, ServerThreadAccess, ShutdownPlan,
    ShutdownSource, ShutdownStep, StatusHeartbeatScheduler, TickRateController, Watchdog,
    WatchdogDecision, MAX_TICK_RATE, MIN_TICK_RATE, TARGET_TPS, TICK_DURATION,
};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[cfg(vibecraft_has_decompiled_sources)]
const SERVER_WATCHDOG_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/server/dedicated/ServerWatchdog.java");

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn server_watchdog_source_matches_tick_limit_and_crash_report_surface() {
    for fragment in [
        "private static final long MAX_SHUTDOWN_TIME = 10000L",
        "private static final int SHUTDOWN_STATUS = 1",
        "this.maxTickTimeNanos = server.getMaxTickLength() * TimeUtil.NANOSECONDS_PER_MILLISECOND",
        "if (deltaNanos > this.maxTickTimeNanos)",
        "Considering it to be crashed, server will forcibly shutdown.",
        "createWatchdogCrashReport(\"Watching Server\"",
        "public static CrashReport createWatchdogCrashReport",
    ] {
        assert!(
            SERVER_WATCHDOG_JAVA.contains(fragment),
            "missing Java source fragment: {fragment}"
        );
    }
}

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
fn set_frozen_only_sets_base_freeze_flag_like_java_manager() {
    let mut controller = TickRateController::default();
    controller.set_frozen(true);
    assert!(controller.step_game_if_paused(5));
    controller.set_frozen(false);
    controller.request_game_to_sprint(10);

    controller.set_frozen(true);

    assert_eq!(controller.frozen_ticks_to_run(), 5);
    assert!(controller.is_sprinting());
    assert!(controller.is_frozen());
}

#[test]
fn tick_rate_controller_sprint_report_uses_java_nanosecond_clamp() {
    let mut controller = TickRateController::default();

    controller.request_game_to_sprint(1);
    controller.tick();
    assert!(controller.check_should_sprint_this_tick());

    let report = controller.finish_tick_sprint().unwrap();
    assert_eq!(report.completed_ticks, 1);
    assert_eq!(report.ticks_per_second, 1_000_000_000);
    assert_eq!(report.milliseconds_per_tick, 0.000001);
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
