use super::{GameTestExceptionBase, GameTestExceptionModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestTimeoutExceptionModel {
    pub base: GameTestExceptionBase,
    pub message: String,
}

impl GameTestTimeoutExceptionModel {
    pub fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            base: GameTestExceptionBase::new(message.clone()),
            message,
        }
    }

    pub fn runtime_message(&self) -> &str {
        self.base.runtime_message()
    }
}

impl GameTestExceptionModel for GameTestTimeoutExceptionModel {
    type Description = String;

    fn base_exception(&self) -> &GameTestExceptionBase {
        &self.base
    }

    fn get_description(&self) -> Self::Description {
        self.message.clone()
    }
}
