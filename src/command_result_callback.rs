#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandResultCallbackModel {
    Empty,
    Recorder(&'static str),
    Chain(
        Box<CommandResultCallbackModel>,
        Box<CommandResultCallbackModel>,
    ),
}

impl CommandResultCallbackModel {
    pub fn empty() -> Self {
        Self::Empty
    }

    pub fn recorder(name: &'static str) -> Self {
        Self::Recorder(name)
    }

    pub fn chain(first: Self, second: Self) -> Self {
        if first == Self::Empty {
            second
        } else if second == Self::Empty {
            first
        } else {
            Self::Chain(Box::new(first), Box::new(second))
        }
    }

    pub fn on_result(&self, success: bool, result: i32, log: &mut Vec<CallbackInvocation>) {
        match self {
            Self::Empty => {}
            Self::Recorder(name) => log.push(CallbackInvocation {
                callback: name,
                success,
                result,
            }),
            Self::Chain(first, second) => {
                first.on_result(success, result, log);
                second.on_result(success, result, log);
            }
        }
    }

    pub fn on_success(&self, result: i32, log: &mut Vec<CallbackInvocation>) {
        self.on_result(true, result, log);
    }

    pub fn on_failure(&self, log: &mut Vec<CallbackInvocation>) {
        self.on_result(false, 0, log);
    }

    pub fn java_to_string(&self) -> Option<&'static str> {
        match self {
            Self::Empty => Some("<empty>"),
            Self::Recorder(_) | Self::Chain(_, _) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallbackInvocation {
    pub callback: &'static str,
    pub success: bool,
    pub result: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn invocation(callback: &'static str, success: bool, result: i32) -> CallbackInvocation {
        CallbackInvocation {
            callback,
            success,
            result,
        }
    }

    #[test]
    fn empty_callback_ignores_results_and_prints_empty() {
        let callback = CommandResultCallbackModel::empty();
        let mut log = Vec::new();

        callback.on_result(true, 12, &mut log);

        assert!(log.is_empty());
        assert_eq!(callback.java_to_string(), Some("<empty>"));
    }

    #[test]
    fn on_success_forwards_true_with_supplied_result() {
        let callback = CommandResultCallbackModel::recorder("store_result");
        let mut log = Vec::new();

        callback.on_success(7, &mut log);

        assert_eq!(log, [invocation("store_result", true, 7)]);
    }

    #[test]
    fn on_failure_forwards_false_with_zero_result() {
        let callback = CommandResultCallbackModel::recorder("store_success");
        let mut log = Vec::new();

        callback.on_failure(&mut log);

        assert_eq!(log, [invocation("store_success", false, 0)]);
    }

    #[test]
    fn chain_returns_second_callback_when_first_is_empty() {
        let second = CommandResultCallbackModel::recorder("second");
        let chained =
            CommandResultCallbackModel::chain(CommandResultCallbackModel::empty(), second.clone());

        assert_eq!(chained, second);
    }

    #[test]
    fn chain_returns_first_callback_when_second_is_empty() {
        let first = CommandResultCallbackModel::recorder("first");
        let chained =
            CommandResultCallbackModel::chain(first.clone(), CommandResultCallbackModel::empty());

        assert_eq!(chained, first);
    }

    #[test]
    fn chain_invokes_non_empty_callbacks_in_first_then_second_order() {
        let chained = CommandResultCallbackModel::chain(
            CommandResultCallbackModel::recorder("first"),
            CommandResultCallbackModel::recorder("second"),
        );
        let mut log = Vec::new();

        chained.on_result(true, 3, &mut log);

        assert_eq!(
            log,
            [invocation("first", true, 3), invocation("second", true, 3),]
        );
    }
}
