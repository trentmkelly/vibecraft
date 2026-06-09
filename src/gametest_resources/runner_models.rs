use super::{create_gametest_batch, GameTestBatchModel, GameTestInfoStateModel};

pub const GAMETEST_RUNNER_SERVER_RUNTIME_TODO: &str = "gametest-runner-server-runtime";
pub const DEFAULT_GAMETESTS_PER_ROW: i32 = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameTestRunnerEvent {
    BatchStarting {
        index: i32,
        environment: String,
    },
    BatchFinished {
        index: i32,
        environment: String,
    },
    EnvironmentActivated {
        environment: String,
    },
    EnvironmentTornDown {
        environment: String,
    },
    TestAddedForRerun {
        original_id: String,
        copy_id: String,
    },
    TickerCleared,
    ForcedChunksCleared,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureSpawnerModel {
    InPlace,
    NotSet,
}

impl StructureSpawnerModel {
    pub fn spawn_structure(self, test_info: &mut GameTestInfoStateModel) -> Option<String> {
        match self {
            Self::InPlace => {
                test_info.start_execution(1);
                Some(test_info.id.clone())
            }
            Self::NotSet => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestRunnerBuilderModel {
    pub batches: Vec<GameTestBatchModel>,
    pub batcher: &'static str,
    pub existing_structure_spawner: StructureSpawnerModel,
    pub new_structure_spawner: StructureSpawnerModel,
    pub halt_on_error: bool,
    pub clear_between_batches: bool,
}

impl GameTestRunnerBuilderModel {
    pub fn from_batches(batches: Vec<GameTestBatchModel>) -> Self {
        Self {
            batches,
            batcher: "GameTestBatchFactory.fromGameTestInfo",
            existing_structure_spawner: StructureSpawnerModel::InPlace,
            new_structure_spawner: StructureSpawnerModel::NotSet,
            halt_on_error: false,
            clear_between_batches: false,
        }
    }

    pub fn halt_on_error(mut self) -> Self {
        self.halt_on_error = true;
        self
    }

    pub fn clear_between_batches(mut self) -> Self {
        self.clear_between_batches = true;
        self
    }

    pub fn new_structure_spawner(mut self, spawner: StructureSpawnerModel) -> Self {
        self.new_structure_spawner = spawner;
        self
    }

    pub fn existing_structure_spawner(mut self, spawner: StructureSpawnerModel) -> Self {
        self.existing_structure_spawner = spawner;
        self
    }

    pub fn build(self) -> GameTestRunnerModel {
        GameTestRunnerModel::new(
            self.batches,
            self.existing_structure_spawner,
            self.new_structure_spawner,
            self.halt_on_error,
            self.clear_between_batches,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestRunnerModel {
    pub batches: Vec<GameTestBatchModel>,
    pub all_test_infos: Vec<String>,
    pub scheduled_for_rerun: Vec<String>,
    pub stopped: bool,
    pub current_environment: Option<String>,
    pub existing_structure_spawner: StructureSpawnerModel,
    pub new_structure_spawner: StructureSpawnerModel,
    pub halt_on_error: bool,
    pub clear_between_batches: bool,
    pub events: Vec<GameTestRunnerEvent>,
}

impl GameTestRunnerModel {
    pub fn new(
        batches: Vec<GameTestBatchModel>,
        existing_structure_spawner: StructureSpawnerModel,
        new_structure_spawner: StructureSpawnerModel,
        halt_on_error: bool,
        clear_between_batches: bool,
    ) -> Self {
        let all_test_infos = batches
            .iter()
            .flat_map(|batch| batch.game_test_infos.clone())
            .collect();
        Self {
            batches,
            all_test_infos,
            scheduled_for_rerun: Vec::new(),
            stopped: true,
            current_environment: None,
            existing_structure_spawner,
            new_structure_spawner,
            halt_on_error,
            clear_between_batches,
            events: Vec::new(),
        }
    }

    pub fn start(&mut self) {
        self.stopped = false;
        self.run_batch(0);
    }

    pub fn stop(&mut self) {
        self.stopped = true;
        self.end_current_environment();
    }

    pub fn rerun_test(&mut self, original: &GameTestInfoStateModel) {
        let copy = original.copy_reset();
        self.events.push(GameTestRunnerEvent::TestAddedForRerun {
            original_id: original.id.clone(),
            copy_id: copy.id.clone(),
        });
        self.all_test_infos.push(copy.id.clone());
        self.scheduled_for_rerun.push(copy.id);
        if self.stopped {
            self.run_scheduled_rerun_tests();
        }
    }

    pub fn run_batch(&mut self, batch_index: usize) {
        if batch_index >= self.batches.len() {
            self.end_current_environment();
            self.run_scheduled_rerun_tests();
            return;
        }

        if batch_index > 0 && self.clear_between_batches {
            self.events.push(GameTestRunnerEvent::ForcedChunksCleared);
        }

        let batch = self.batches[batch_index].clone();
        self.end_current_environment();
        self.current_environment = Some(batch.environment.clone());
        self.events.push(GameTestRunnerEvent::EnvironmentActivated {
            environment: batch.environment.clone(),
        });
        self.events.push(GameTestRunnerEvent::BatchStarting {
            index: batch.index,
            environment: batch.environment,
        });
    }

    pub fn complete_batch(&mut self, batch_index: usize, failed: bool) {
        if let Some(batch) = self.batches.get(batch_index) {
            if failed && self.halt_on_error {
                self.end_current_environment();
                self.events.push(GameTestRunnerEvent::ForcedChunksCleared);
                self.events.push(GameTestRunnerEvent::TickerCleared);
                self.stopped = true;
            } else {
                self.events.push(GameTestRunnerEvent::BatchFinished {
                    index: batch.index,
                    environment: batch.environment.clone(),
                });
                self.events.push(GameTestRunnerEvent::ForcedChunksCleared);
                self.run_batch(batch_index + 1);
            }
        }
    }

    fn end_current_environment(&mut self) {
        if let Some(environment) = self.current_environment.take() {
            self.events
                .push(GameTestRunnerEvent::EnvironmentTornDown { environment });
        }
    }

    fn run_scheduled_rerun_tests(&mut self) {
        if self.scheduled_for_rerun.is_empty() {
            self.batches.clear();
            self.stopped = true;
        } else {
            let tests = std::mem::take(&mut self.scheduled_for_rerun);
            self.batches = vec![create_gametest_batch(0, tests, "minecraft:default")
                .expect("rerun batch must include copied tests")];
            self.stopped = false;
            self.run_batch(0);
        }
    }
}
