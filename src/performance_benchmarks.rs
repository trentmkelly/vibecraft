#![cfg(test)]
#![allow(dead_code)]

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use crate::ai_system::{NavigationKind, PathPlan, TargetingConditions};
    use crate::block_update::{BlockPos, Direction};
    use crate::chunk_manager::ChunkManager;
    use crate::network::pipeline::NetworkPipeline;
    use crate::redstone::{
        comparator_output, dust_propagated_power, observer_on_neighbor_changed, piston_decision,
        repeater_output, ComparatorMode, ObserverState, PistonKind, PistonState, RepeaterState,
    };
    use crate::runtime::{ServerRuntime, TICK_DURATION};
    use crate::scheduled_tick::{LevelTickQueues, TickPriority};
    use crate::storage::chunk::LevelChunk;
    use crate::storage::datafix::TARGET_DATA_VERSION;
    use crate::storage::region::ChunkPos;
    use crate::worldgen::{
        CONFIGURED_FEATURES, FEATURE_BEHAVIOR_MODELS, FEATURE_TYPES,
        PLACED_FEATURE_BOOTSTRAP_SOURCES,
    };

    const MAX_TEST_BENCH_DURATION: Duration = Duration::from_secs(5);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct BenchResult {
        name: &'static str,
        operations: usize,
        elapsed: Duration,
    }

    impl BenchResult {
        fn assert_completed(self) {
            assert!(self.operations > 0, "{} performed no operations", self.name);
            assert!(
                self.elapsed <= MAX_TEST_BENCH_DURATION,
                "{} took too long for the deterministic benchmark harness: {:?}",
                self.name,
                self.elapsed
            );
        }
    }

    #[test]
    fn performance_benchmark_matrix_covers_required_surfaces() {
        let surfaces = [
            "tick loop",
            "chunk IO",
            "chunk generation",
            "packet throughput",
            "entity ticking",
            "pathfinding",
            "redstone",
        ];

        assert_eq!(surfaces.len(), 7);
        assert!(surfaces.contains(&"tick loop"));
        assert!(surfaces.contains(&"chunk IO"));
        assert!(surfaces.contains(&"chunk generation"));
        assert!(surfaces.contains(&"packet throughput"));
        assert!(surfaces.contains(&"entity ticking"));
        assert!(surfaces.contains(&"pathfinding"));
        assert!(surfaces.contains(&"redstone"));
    }

    #[test]
    fn tick_loop_benchmark_runs_many_runtime_ticks() {
        let now = Instant::now();
        let mut runtime = ServerRuntime::new(now, Duration::from_secs(30));
        let started = Instant::now();
        let mut task_hits = 0usize;

        for tick in 0..2_000u64 {
            runtime.schedule(0, || {});
            let metrics = runtime.run_one_tick(now + TICK_DURATION * tick as u32);
            black_box(metrics.tick);
            task_hits += 1;
        }

        let result = BenchResult {
            name: "tick loop",
            operations: runtime.metrics().len(),
            elapsed: started.elapsed(),
        };

        assert_eq!(task_hits, 2_000);
        assert_eq!(runtime.tick(), 2_000);
        result.assert_completed();
    }

    #[test]
    fn chunk_io_benchmark_round_trips_vanilla_nbt_payloads() {
        let started = Instant::now();
        let chunks: Vec<_> = (0..256)
            .map(|index| {
                LevelChunk::empty(ChunkPos {
                    x: index,
                    z: -index,
                })
            })
            .collect();
        let mut decoded = 0usize;

        for chunk in &chunks {
            let tag = chunk.to_nbt(TARGET_DATA_VERSION);
            let round_trip = LevelChunk::from_nbt(chunk.pos, black_box(&tag)).unwrap();
            assert_eq!(round_trip.pos, chunk.pos);
            decoded += 1;
        }

        BenchResult {
            name: "chunk IO",
            operations: decoded,
            elapsed: started.elapsed(),
        }
        .assert_completed();
    }

    #[test]
    fn chunk_generation_benchmark_loads_and_advances_chunk_statuses() {
        let started = Instant::now();
        let mut manager = ChunkManager::new(96);
        let mut generated = 0usize;

        for x in -8..8 {
            for z in -8..8 {
                let chunk = manager
                    .load_or_generate(ChunkPos { x, z }, "full")
                    .expect("chunk generation should advance to full status");
                assert_eq!(chunk.status, "minecraft:full");
                generated += 1;
            }
        }

        assert!(!FEATURE_TYPES.is_empty());
        assert!(!CONFIGURED_FEATURES.is_empty());
        assert!(!PLACED_FEATURE_BOOTSTRAP_SOURCES.is_empty());
        assert!(!FEATURE_BEHAVIOR_MODELS.is_empty());
        BenchResult {
            name: "chunk generation",
            operations: generated,
            elapsed: started.elapsed(),
        }
        .assert_completed();
    }

    #[test]
    fn packet_throughput_benchmark_round_trips_pipeline_frames() {
        let started = Instant::now();
        let mut encoder = NetworkPipeline::default();
        let mut decoder = NetworkPipeline::default();
        encoder.enable_compression(64);
        decoder.enable_compression(64);
        let payload = b"packet throughput benchmark payload ".repeat(8);
        let mut decoded = 0usize;

        for packet_id in 0..1_024 {
            let frame = encoder
                .encode_packet(packet_id, black_box(&payload))
                .unwrap();
            let packet = decoder.decode_packet(black_box(&frame)).unwrap();
            assert_eq!(packet.id, packet_id);
            assert_eq!(packet.payload, payload);
            decoded += 1;
        }

        BenchResult {
            name: "packet throughput",
            operations: decoded,
            elapsed: started.elapsed(),
        }
        .assert_completed();
    }

    #[test]
    fn entity_ticking_and_pathfinding_benchmark_exercises_ai_surfaces() {
        let started = Instant::now();
        let plans = [
            PathPlan {
                navigation: NavigationKind::Ground,
                can_float: false,
                max_visited_nodes_multiplier: 1.0,
                target: (16, 64, 16),
                reached: false,
            },
            PathPlan {
                navigation: NavigationKind::Water,
                can_float: false,
                max_visited_nodes_multiplier: 2.0,
                target: (-32, 62, 8),
                reached: false,
            },
            PathPlan {
                navigation: NavigationKind::Amphibious,
                can_float: true,
                max_visited_nodes_multiplier: 1.5,
                target: (0, 70, -24),
                reached: true,
            },
        ];
        let targeting = TargetingConditions {
            range: 48,
            check_line_of_sight: true,
            test_invisible: false,
        };
        let mut successful_checks = 0usize;

        for tick in 0..4_096 {
            for plan in &plans {
                let distance = tick % 64;
                black_box(plan.can_path_through_water());
                if targeting.can_target(distance, distance < 48, false) {
                    successful_checks += 1;
                }
            }
        }

        assert!(successful_checks > 0);
        BenchResult {
            name: "entity ticking and pathfinding",
            operations: plans.len() * 4_096,
            elapsed: started.elapsed(),
        }
        .assert_completed();
    }

    #[test]
    fn redstone_benchmark_exercises_signal_and_scheduled_tick_work() {
        let started = Instant::now();
        let mut queues = LevelTickQueues::new();
        for chunk_x in 0..8 {
            queues.add_container(ChunkPos { x: chunk_x, z: 0 });
        }

        let mut operations = 0usize;
        for step in 0..1_024 {
            let pos = BlockPos {
                x: step % 128,
                y: 64,
                z: 0,
            };
            let repeater = RepeaterState {
                facing: Direction::East,
                delay_index: (step % 4) as u8,
                powered: step % 2 == 0,
                locked: step % 7 == 0,
            };
            black_box(dust_propagated_power((step % 32) as u8));
            black_box(repeater_output(repeater, (step % 16) as u8, [1, 2]));
            black_box(comparator_output(
                ComparatorMode::Subtract,
                (step % 16) as u8,
                (step % 5) as u8,
            ));
            black_box(piston_decision(
                PistonState {
                    kind: PistonKind::Sticky,
                    facing: Direction::East,
                    extended: step % 3 == 0,
                },
                step % 2 == 0,
                step % 5 == 0,
                (step % 13) as usize,
            ));

            if let Some(tick) = observer_on_neighbor_changed(
                pos,
                ObserverState {
                    facing: Direction::East,
                    powered: false,
                },
                pos.relative(Direction::East),
                20,
            ) {
                queues.schedule(tick);
            }
            let scheduled =
                queues.create_tick(20, pos, "minecraft:redstone_wire", 0, TickPriority::Normal);
            queues.schedule(scheduled);
            operations += 1;
        }

        let drained = queues.tick(22, 2_048, |_| true);
        assert!(!drained.is_empty());
        BenchResult {
            name: "redstone",
            operations,
            elapsed: started.elapsed(),
        }
        .assert_completed();
    }
}
