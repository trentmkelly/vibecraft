use super::{GameTestInfoStateModel, GameTestInfoTickOutcome};

pub const GAMETEST_TICKER_SINGLETON: &str = "GameTestTicker.SINGLETON";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameTestTickerStateModel {
    Idle,
    Running,
    Halting,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestTickerModel {
    pub test_infos: Vec<GameTestInfoStateModel>,
    pub runner: Option<String>,
    pub state: GameTestTickerStateModel,
    pub warnings: Vec<&'static str>,
    pub runner_stop_count: usize,
}

impl Default for GameTestTickerModel {
    fn default() -> Self {
        Self {
            test_infos: Vec::new(),
            runner: None,
            state: GameTestTickerStateModel::Idle,
            warnings: Vec::new(),
            runner_stop_count: 0,
        }
    }
}

impl GameTestTickerModel {
    pub fn add(&mut self, test_info: GameTestInfoStateModel) {
        self.test_infos.push(test_info);
    }

    pub fn clear(&mut self) {
        if self.state != GameTestTickerStateModel::Idle {
            self.state = GameTestTickerStateModel::Halting;
        } else {
            self.test_infos.clear();
            if self.runner.take().is_some() {
                self.runner_stop_count += 1;
            }
        }
    }

    pub fn set_runner(&mut self, runner: impl Into<String>) {
        if self.runner.is_some() {
            self.warnings
                .push("The runner was already set in GameTestTicker");
        }
        self.runner = Some(runner.into());
    }

    pub fn tick(&mut self) -> Vec<GameTestInfoTickOutcome> {
        self.tick_with_clear_after(None)
    }

    pub(crate) fn tick_with_clear_after(
        &mut self,
        clear_after_test_index: Option<usize>,
    ) -> Vec<GameTestInfoTickOutcome> {
        if self.runner.is_none() {
            return Vec::new();
        }

        self.state = GameTestTickerStateModel::Running;
        let outcomes = self.tick_infos(clear_after_test_index);
        self.test_infos.retain(|info| !info.done);
        let finishing_state = self.state;
        self.state = GameTestTickerStateModel::Idle;
        if finishing_state == GameTestTickerStateModel::Halting {
            self.clear();
        }
        outcomes
    }

    fn tick_infos(
        &mut self,
        clear_after_test_index: Option<usize>,
    ) -> Vec<GameTestInfoTickOutcome> {
        let mut outcomes = Vec::with_capacity(self.test_infos.len());
        for index in 0..self.test_infos.len() {
            outcomes.push(self.test_infos[index].tick_internal());
            if clear_after_test_index == Some(index) {
                self.clear();
            }
        }
        outcomes
    }
}
