#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionCommandSourceModel {
    callback: CommandResultCallbackModel,
    dispatcher: &'static str,
    silent: bool,
    handled_errors: Vec<HandledErrorModel>,
}

impl ExecutionCommandSourceModel {
    pub fn new(
        callback: CommandResultCallbackModel,
        dispatcher: &'static str,
        silent: bool,
    ) -> Self {
        Self {
            callback,
            dispatcher,
            silent,
            handled_errors: Vec::new(),
        }
    }

    pub fn with_callback(&self, callback: CommandResultCallbackModel) -> Self {
        Self {
            callback,
            ..self.clone()
        }
    }

    pub fn callback(&self) -> &CommandResultCallbackModel {
        &self.callback
    }

    pub fn clear_callbacks(&self) -> Self {
        self.with_callback(CommandResultCallbackModel::Empty)
    }

    pub fn dispatcher(&self) -> &'static str {
        self.dispatcher
    }

    pub fn handle_error(
        &mut self,
        error_type: &'static str,
        message: &'static str,
        forked: bool,
        tracer: Option<&mut TraceCallbacksModel>,
    ) {
        if let Some(tracer) = tracer {
            tracer.errors.push(message.to_string());
        }
        self.handled_errors.push(HandledErrorModel {
            error_type,
            message,
            forked,
        });
    }

    pub fn handle_syntax_error(
        &mut self,
        error: CommandSyntaxExceptionModel,
        forked: bool,
        tracer: Option<&mut TraceCallbacksModel>,
    ) {
        self.handle_error(error.error_type, error.raw_message, forked, tracer);
    }

    pub fn is_silent(&self) -> bool {
        self.silent
    }

    pub fn handled_errors(&self) -> &[HandledErrorModel] {
        &self.handled_errors
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandResultCallbackModel {
    Empty,
    Recorder(&'static str),
}

impl CommandResultCallbackModel {
    fn on_result(&self, success: bool, result: i32, log: &mut Vec<ResultInvocationModel>) {
        match self {
            Self::Empty => {}
            Self::Recorder(name) => log.push(ResultInvocationModel {
                callback: name,
                success,
                result,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandContextModel {
    pub source: ExecutionCommandSourceModel,
}

pub fn result_consumer(
    context: &CommandContextModel,
    success: bool,
    result: i32,
) -> Vec<ResultInvocationModel> {
    let mut log = Vec::new();
    context
        .source
        .callback()
        .on_result(success, result, &mut log);
    log
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSyntaxExceptionModel {
    pub error_type: &'static str,
    pub raw_message: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceCallbacksModel {
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandledErrorModel {
    pub error_type: &'static str,
    pub message: &'static str,
    pub forked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultInvocationModel {
    pub callback: &'static str,
    pub success: bool,
    pub result: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_callbacks_replaces_current_callback_with_empty() {
        let source = ExecutionCommandSourceModel::new(
            CommandResultCallbackModel::Recorder("store"),
            "dispatcher",
            false,
        );

        assert_eq!(
            source.clear_callbacks().callback(),
            &CommandResultCallbackModel::Empty
        );
    }

    #[test]
    fn dispatcher_and_silent_delegate_to_implementation_state() {
        let source =
            ExecutionCommandSourceModel::new(CommandResultCallbackModel::Empty, "root", true);

        assert_eq!(source.dispatcher(), "root");
        assert!(source.is_silent());
    }

    #[test]
    fn syntax_error_default_delegates_type_and_raw_message_to_handle_error() {
        let mut source =
            ExecutionCommandSourceModel::new(CommandResultCallbackModel::Empty, "root", false);
        let mut tracer = TraceCallbacksModel { errors: Vec::new() };

        source.handle_syntax_error(
            CommandSyntaxExceptionModel {
                error_type: "dispatcherUnknownCommand",
                raw_message: "Unknown command",
            },
            false,
            Some(&mut tracer),
        );

        assert_eq!(
            source.handled_errors(),
            [HandledErrorModel {
                error_type: "dispatcherUnknownCommand",
                message: "Unknown command",
                forked: false,
            }]
        );
        assert_eq!(tracer.errors, ["Unknown command"]);
    }

    #[test]
    fn result_consumer_invokes_source_callback_with_success_and_result() {
        let context = CommandContextModel {
            source: ExecutionCommandSourceModel::new(
                CommandResultCallbackModel::Recorder("callback"),
                "dispatcher",
                false,
            ),
        };

        assert_eq!(
            result_consumer(&context, true, 42),
            [ResultInvocationModel {
                callback: "callback",
                success: true,
                result: 42,
            }]
        );
    }

    #[test]
    fn result_consumer_uses_empty_callback_as_no_op() {
        let context = CommandContextModel {
            source: ExecutionCommandSourceModel::new(
                CommandResultCallbackModel::Empty,
                "dispatcher",
                false,
            ),
        };

        assert!(result_consumer(&context, false, 0).is_empty());
    }
}
