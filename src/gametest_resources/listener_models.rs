use super::GameTestInfoStateModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameTestListenerEvent {
    StructureLoaded {
        test_id: String,
    },
    Passed {
        test_id: String,
        runner: String,
    },
    Failed {
        test_id: String,
        runner: String,
    },
    AddedForRerun {
        original_id: String,
        copy_id: String,
        runner: String,
    },
}

pub trait GameTestListenerModel {
    fn test_structure_loaded(&mut self, test_info: &GameTestInfoStateModel);

    fn test_passed(&mut self, test_info: &GameTestInfoStateModel, runner: impl Into<String>);

    fn test_failed(&mut self, test_info: &GameTestInfoStateModel, runner: impl Into<String>);

    fn test_added_for_rerun(
        &mut self,
        original: &GameTestInfoStateModel,
        copy: &GameTestInfoStateModel,
        runner: impl Into<String>,
    );
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecordingGameTestListener {
    pub events: Vec<GameTestListenerEvent>,
}

impl GameTestListenerModel for RecordingGameTestListener {
    fn test_structure_loaded(&mut self, test_info: &GameTestInfoStateModel) {
        self.events.push(GameTestListenerEvent::StructureLoaded {
            test_id: test_info.id.clone(),
        });
    }

    fn test_passed(&mut self, test_info: &GameTestInfoStateModel, runner: impl Into<String>) {
        self.events.push(GameTestListenerEvent::Passed {
            test_id: test_info.id.clone(),
            runner: runner.into(),
        });
    }

    fn test_failed(&mut self, test_info: &GameTestInfoStateModel, runner: impl Into<String>) {
        self.events.push(GameTestListenerEvent::Failed {
            test_id: test_info.id.clone(),
            runner: runner.into(),
        });
    }

    fn test_added_for_rerun(
        &mut self,
        original: &GameTestInfoStateModel,
        copy: &GameTestInfoStateModel,
        runner: impl Into<String>,
    ) {
        self.events.push(GameTestListenerEvent::AddedForRerun {
            original_id: original.id.clone(),
            copy_id: copy.id.clone(),
            runner: runner.into(),
        });
    }
}
