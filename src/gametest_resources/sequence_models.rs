use super::{
    create_gametest_event, create_gametest_event_at, create_gametest_event_with_minimum_delay,
    GameTestEventModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameTestSequenceOutcome {
    Continue,
    ParentSucceeded,
    ParentFailed { message: String, tick: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestSequenceModel {
    pub parent_tick: i32,
    pub last_tick: i32,
    pub events: Vec<GameTestEventModel>,
}

impl GameTestSequenceModel {
    pub fn new(parent_tick: i32) -> Self {
        Self {
            parent_tick,
            last_tick: parent_tick,
            events: Vec::new(),
        }
    }

    pub fn then_wait_until(&mut self, assertion: impl Into<String>) -> &mut Self {
        self.events.push(create_gametest_event(assertion));
        self
    }

    pub fn then_wait_until_delay(
        &mut self,
        expected_delay: i64,
        assertion: impl Into<String>,
    ) -> &mut Self {
        self.events
            .push(create_gametest_event_at(expected_delay, assertion));
        self
    }

    pub fn then_wait_at_least(
        &mut self,
        minimum_delay: i64,
        assertion: impl Into<String>,
    ) -> &mut Self {
        self.events.push(create_gametest_event_with_minimum_delay(
            minimum_delay,
            assertion,
        ));
        self
    }

    pub fn then_idle(&mut self, delta: i32) -> &mut Self {
        self.then_execute_after(delta, "idle")
    }

    pub fn then_execute(&mut self, assertion: impl Into<String>) -> &mut Self {
        self.events.push(create_gametest_event(format!(
            "execute:{}",
            assertion.into()
        )));
        self
    }

    pub fn then_execute_after(&mut self, delta: i32, action: impl Into<String>) -> &mut Self {
        self.events.push(create_gametest_event(format!(
            "execute_after:{delta}:{}",
            action.into()
        )));
        self
    }

    pub fn then_execute_for(&mut self, delta: i32, action: impl Into<String>) -> &mut Self {
        self.events.push(create_gametest_event(format!(
            "execute_for:{delta}:{}",
            action.into()
        )));
        self
    }

    pub fn then_succeed(&mut self) {
        self.events.push(create_gametest_event("parent.succeed"));
    }

    pub fn then_fail(&mut self, message: impl Into<String>) {
        self.events.push(create_gametest_event(format!(
            "parent.fail:{}",
            message.into()
        )));
    }

    pub fn tick(&mut self, tick: i32) -> GameTestSequenceOutcome {
        let Some(event) = self.events.first().cloned() else {
            return GameTestSequenceOutcome::Continue;
        };

        if let Some((delta, _)) = parse_delta_action(&event.assertion, "execute_after:") {
            if tick < self.last_tick + delta {
                return GameTestSequenceOutcome::ParentFailed {
                    message: "test.error.sequence.not_completed".to_string(),
                    tick,
                };
            }
        }
        if let Some((delta, _)) = parse_delta_action(&event.assertion, "execute_for:") {
            if tick < self.last_tick + delta {
                return GameTestSequenceOutcome::ParentFailed {
                    message: "test.error.sequence.not_completed".to_string(),
                    tick,
                };
            }
        }

        self.events.remove(0);
        let delay = tick - self.last_tick;
        let previous_tick = self.last_tick;
        self.last_tick = tick;

        if let Some(minimum_delay) = event.minimum_delay {
            if minimum_delay > i64::from(delay) {
                return GameTestSequenceOutcome::ParentFailed {
                    message: format!(
                        "test.error.sequence.minimum_tick:{}",
                        previous_tick + minimum_delay as i32
                    ),
                    tick,
                };
            }
        }

        if let Some(expected_delay) = event.expected_delay {
            if expected_delay != i64::from(delay) {
                return GameTestSequenceOutcome::ParentFailed {
                    message: format!(
                        "test.error.sequence.invalid_tick:{}",
                        previous_tick + expected_delay as i32
                    ),
                    tick,
                };
            }
        }

        if event.assertion == "parent.succeed" {
            GameTestSequenceOutcome::ParentSucceeded
        } else if let Some(message) = event.assertion.strip_prefix("parent.fail:") {
            GameTestSequenceOutcome::ParentFailed {
                message: message.to_string(),
                tick,
            }
        } else {
            GameTestSequenceOutcome::Continue
        }
    }

    pub fn tick_and_continue(&mut self, tick: i32) -> GameTestSequenceOutcome {
        match self.tick(tick) {
            GameTestSequenceOutcome::ParentFailed { .. } => GameTestSequenceOutcome::Continue,
            outcome => outcome,
        }
    }

    pub fn tick_and_fail_if_not_complete(&mut self, tick: i32) -> GameTestSequenceOutcome {
        self.tick(tick)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameTestSequenceConditionModel {
    trigger_time: i32,
}

impl Default for GameTestSequenceConditionModel {
    fn default() -> Self {
        Self { trigger_time: -1 }
    }
}

impl GameTestSequenceConditionModel {
    pub fn trigger(&mut self, time: i32) -> Result<(), String> {
        if self.trigger_time != -1 {
            return Err(format!(
                "Condition already triggered at {}",
                self.trigger_time
            ));
        }
        self.trigger_time = time;
        Ok(())
    }

    pub fn assert_triggered_this_tick(&self, tick: i32) -> Result<(), String> {
        if self.trigger_time == tick {
            Ok(())
        } else if self.trigger_time == -1 {
            Err("test.error.sequence.condition_not_triggered".to_string())
        } else {
            Err(format!(
                "test.error.sequence.condition_already_triggered:{}",
                self.trigger_time
            ))
        }
    }
}

fn parse_delta_action<'a>(value: &'a str, prefix: &str) -> Option<(i32, &'a str)> {
    let rest = value.strip_prefix(prefix)?;
    let (delta, action) = rest.split_once(':')?;
    Some((delta.parse().ok()?, action))
}
