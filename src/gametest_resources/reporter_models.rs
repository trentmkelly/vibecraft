use crate::core_block_pos::BlockPosModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReporterTestInfoModel {
    pub id: String,
    pub structure: String,
    pub runtime_ms: i64,
    pub required: bool,
    pub test_block_pos: BlockPosModel,
    pub error: Option<String>,
}

impl ReporterTestInfoModel {
    pub fn new(
        id: impl Into<String>,
        structure: impl Into<String>,
        runtime_ms: i64,
        required: bool,
        test_block_pos: BlockPosModel,
        error: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            structure: structure.into(),
            runtime_ms,
            required,
            test_block_pos,
            error,
        }
    }

    pub fn short_pos(&self) -> String {
        format!(
            "{}, {}, {}",
            self.test_block_pos.x(),
            self.test_block_pos.y(),
            self.test_block_pos.z()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestReporterCall {
    Failed(String),
    Success(String),
    Finish,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecordingTestReporterModel {
    pub calls: Vec<TestReporterCall>,
}

impl RecordingTestReporterModel {
    pub fn on_test_failed(&mut self, test_info: &ReporterTestInfoModel) {
        self.calls
            .push(TestReporterCall::Failed(test_info.id.clone()));
    }

    pub fn on_test_success(&mut self, test_info: &ReporterTestInfoModel) {
        self.calls
            .push(TestReporterCall::Success(test_info.id.clone()));
    }

    pub fn finish(&mut self) {
        self.calls.push(TestReporterCall::Finish);
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LogTestReporterModel {
    pub logs: Vec<String>,
}

impl LogTestReporterModel {
    pub fn on_test_failed(&mut self, test_info: &ReporterTestInfoModel) {
        let error = test_info.error.as_deref().unwrap_or("");
        if test_info.required {
            self.logs.push(format!(
                "ERROR {} failed at {}! {}",
                test_info.id,
                test_info.short_pos(),
                error
            ));
        } else {
            self.logs.push(format!(
                "WARN (optional) {} failed at {}. {}",
                test_info.id,
                test_info.short_pos(),
                error
            ));
        }
    }

    pub fn on_test_success(&mut self, _test_info: &ReporterTestInfoModel) {}

    pub fn finish(&mut self) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JUnitTestResultModel {
    Failure { message: String },
    Skipped { message: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct JUnitTestCaseModel {
    pub name: String,
    pub classname: String,
    pub time_seconds: f64,
    pub result: Option<JUnitTestResultModel>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JUnitLikeTestReporterModel {
    pub destination: String,
    pub timestamp_set: bool,
    pub test_cases: Vec<JUnitTestCaseModel>,
    pub suite_time_seconds: Option<f64>,
    pub saved_destination: Option<String>,
}

impl JUnitLikeTestReporterModel {
    pub fn new(destination: impl Into<String>) -> Self {
        Self {
            destination: destination.into(),
            timestamp_set: true,
            test_cases: Vec::new(),
            suite_time_seconds: None,
            saved_destination: None,
        }
    }

    pub fn on_test_failed(&mut self, test_info: &ReporterTestInfoModel) {
        let tag = if test_info.required {
            JUnitTestResultModel::Failure {
                message: self.failure_message(test_info),
            }
        } else {
            JUnitTestResultModel::Skipped {
                message: self.failure_message(test_info),
            }
        };
        self.create_test_case(test_info, Some(tag));
    }

    pub fn on_test_success(&mut self, test_info: &ReporterTestInfoModel) {
        self.create_test_case(test_info, None);
    }

    pub fn finish(&mut self, elapsed_ms: i64) {
        self.suite_time_seconds = Some(elapsed_ms as f64 / 1000.0);
        self.saved_destination = Some(self.destination.clone());
    }

    fn create_test_case(
        &mut self,
        test_info: &ReporterTestInfoModel,
        result: Option<JUnitTestResultModel>,
    ) {
        self.test_cases.push(JUnitTestCaseModel {
            name: test_info.id.clone(),
            classname: test_info.structure.clone(),
            time_seconds: test_info.runtime_ms as f64 / 1000.0,
            result,
        });
    }

    fn failure_message(&self, test_info: &ReporterTestInfoModel) -> String {
        format!(
            "({}) {}",
            test_info.short_pos(),
            test_info.error.as_deref().unwrap_or("")
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestReporterDelegateModel {
    Log(LogTestReporterModel),
    Recording(RecordingTestReporterModel),
    JUnit(JUnitLikeTestReporterModel),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTestReporterModel {
    pub delegate: TestReporterDelegateModel,
}

impl Default for GlobalTestReporterModel {
    fn default() -> Self {
        Self {
            delegate: TestReporterDelegateModel::Log(LogTestReporterModel::default()),
        }
    }
}

impl GlobalTestReporterModel {
    pub fn replace_with(&mut self, delegate: TestReporterDelegateModel) {
        self.delegate = delegate;
    }

    pub fn on_test_failed(&mut self, test_info: &ReporterTestInfoModel) {
        match &mut self.delegate {
            TestReporterDelegateModel::Log(reporter) => reporter.on_test_failed(test_info),
            TestReporterDelegateModel::Recording(reporter) => reporter.on_test_failed(test_info),
            TestReporterDelegateModel::JUnit(reporter) => reporter.on_test_failed(test_info),
        }
    }

    pub fn on_test_success(&mut self, test_info: &ReporterTestInfoModel) {
        match &mut self.delegate {
            TestReporterDelegateModel::Log(reporter) => reporter.on_test_success(test_info),
            TestReporterDelegateModel::Recording(reporter) => reporter.on_test_success(test_info),
            TestReporterDelegateModel::JUnit(reporter) => reporter.on_test_success(test_info),
        }
    }

    pub fn finish(&mut self, elapsed_ms: i64) {
        match &mut self.delegate {
            TestReporterDelegateModel::Log(reporter) => reporter.finish(),
            TestReporterDelegateModel::Recording(reporter) => reporter.finish(),
            TestReporterDelegateModel::JUnit(reporter) => reporter.finish(elapsed_ms),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryOptionsModel {
    pub number_of_tries: i32,
    pub halt_on_failure: bool,
}

impl RetryOptionsModel {
    pub const fn no_retries() -> Self {
        Self {
            number_of_tries: 1,
            halt_on_failure: true,
        }
    }

    pub fn unlimited_tries(self) -> bool {
        self.number_of_tries < 1
    }

    pub fn has_tries_left(self, attempts: i32, successes: i32) -> bool {
        let has_failures = attempts != successes;
        let has_more_attempts = self.unlimited_tries() || attempts < self.number_of_tries;
        has_more_attempts && (!has_failures || !self.halt_on_failure)
    }

    pub fn has_retries(self) -> bool {
        self.number_of_tries != 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportGameListenerBlockEntityEvent {
    Success,
    ErrorMessage(String),
    MarkError {
        absolute_pos: BlockPosModel,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportGameListenerChatFormat {
    Green,
    Red,
    Yellow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportGameListenerChatEvent {
    pub format: ReportGameListenerChatFormat,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportGameListenerErrorModel {
    pub message: String,
    pub cause: Option<String>,
    pub assert_description: Option<String>,
    pub assert_pos: Option<BlockPosModel>,
    pub assert_block_message: Option<String>,
}

impl ReportGameListenerErrorModel {
    pub fn plain(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            cause: None,
            assert_description: None,
            assert_pos: None,
            assert_block_message: None,
        }
    }

    pub fn assertion(message: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            cause: None,
            assert_description: Some(description.into()),
            assert_pos: None,
            assert_block_message: None,
        }
    }

    pub fn positional_assertion(
        message: impl Into<String>,
        description: impl Into<String>,
        absolute_pos: BlockPosModel,
        block_message: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            cause: None,
            assert_description: Some(description.into()),
            assert_pos: Some(absolute_pos),
            assert_block_message: Some(block_message.into()),
        }
    }

    fn described(&self) -> String {
        self.message.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportGameListenerTestInfoModel {
    pub id: String,
    pub structure: String,
    pub runtime_ms: i64,
    pub tick: i32,
    pub required: bool,
    pub retry_options: RetryOptionsModel,
    pub max_attempts: i32,
    pub required_successes: i32,
    pub test_block_pos: BlockPosModel,
    pub block_entity_present: bool,
    pub error: Option<ReportGameListenerErrorModel>,
}

impl ReportGameListenerTestInfoModel {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            structure: "minecraft:structure".to_string(),
            runtime_ms: 0,
            tick: 0,
            required: true,
            retry_options: RetryOptionsModel::no_retries(),
            max_attempts: 1,
            required_successes: 1,
            test_block_pos: BlockPosModel::ZERO,
            block_entity_present: true,
            error: None,
        }
    }

    pub fn flaky(mut self, max_attempts: i32, required_successes: i32) -> Self {
        self.max_attempts = max_attempts;
        self.required_successes = required_successes;
        self
    }

    pub fn with_retry_options(mut self, retry_options: RetryOptionsModel) -> Self {
        self.retry_options = retry_options;
        self
    }

    pub fn with_runtime(mut self, runtime_ms: i64, tick: i32) -> Self {
        self.runtime_ms = runtime_ms;
        self.tick = tick;
        self
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn with_error(mut self, error: ReportGameListenerErrorModel) -> Self {
        self.error = Some(error);
        self
    }

    pub fn is_flaky(&self) -> bool {
        self.max_attempts > 1
    }

    fn reporter_info(&self) -> ReporterTestInfoModel {
        ReporterTestInfoModel::new(
            self.id.clone(),
            self.structure.clone(),
            self.runtime_ms,
            self.required,
            self.test_block_pos,
            self.error.as_ref().map(|error| error.message.clone()),
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReportGameListenerModel {
    pub attempts: i32,
    pub successes: i32,
    pub chats: Vec<ReportGameListenerChatEvent>,
    pub block_entity_events: Vec<ReportGameListenerBlockEntityEvent>,
    pub reruns: Vec<String>,
    pub global_calls: Vec<TestReporterCall>,
    pub attached_rerun_ids: Vec<String>,
}

impl ReportGameListenerModel {
    pub fn test_structure_loaded(&mut self) {
        self.attempts += 1;
    }

    pub fn test_passed(&mut self, test_info: &ReportGameListenerTestInfoModel) {
        self.successes += 1;
        if test_info.retry_options.has_retries() {
            self.handle_retry(test_info, true);
        } else if !test_info.is_flaky() {
            self.report_passed(
                test_info,
                format!(
                    "{} passed! ({}ms / {}gameticks)",
                    test_info.id, test_info.runtime_ms, test_info.tick
                ),
            );
        } else if self.successes >= test_info.required_successes {
            self.report_passed(
                test_info,
                format!(
                    "{} passed {} times of {} attempts.",
                    test_info.id, self.successes, self.attempts
                ),
            );
        } else {
            self.say(
                ReportGameListenerChatFormat::Green,
                format!(
                    "Flaky test {} succeeded, attempt: {} successes: {}",
                    test_info.id, self.attempts, self.successes
                ),
            );
            self.rerun(test_info);
        }
    }

    pub fn test_failed(&mut self, test_info: &ReportGameListenerTestInfoModel) {
        if !test_info.is_flaky() {
            let fallback_error;
            let error = if let Some(error) = test_info.error.as_ref() {
                error
            } else {
                fallback_error = ReportGameListenerErrorModel::plain("");
                &fallback_error
            };
            self.report_failure(test_info, error);
            if test_info.retry_options.has_retries() {
                self.handle_retry(test_info, false);
            }
        } else {
            let mut text = format!(
                "Flaky test {} failed, attempt: {}/{}",
                test_info.id, self.attempts, test_info.max_attempts
            );
            if test_info.required_successes > 1 {
                text.push_str(&format!(
                    ", successes: {} ({} required)",
                    self.successes, test_info.required_successes
                ));
            }

            self.say(ReportGameListenerChatFormat::Yellow, text);
            if test_info.max_attempts - self.attempts + self.successes
                >= test_info.required_successes
            {
                self.rerun(test_info);
            } else {
                let exhausted = ReportGameListenerErrorModel {
                    message: format!(
                        "Not enough successes: {} out of {} attempts. Required successes: {}. max attempts: {}.",
                        self.successes,
                        self.attempts,
                        test_info.required_successes,
                        test_info.max_attempts
                    ),
                    cause: test_info.error.as_ref().map(|error| error.message.clone()),
                    assert_description: None,
                    assert_pos: None,
                    assert_block_message: None,
                };
                self.report_failure(test_info, &exhausted);
            }
        }
    }

    pub fn test_added_for_rerun(&mut self, copy: &ReportGameListenerTestInfoModel) {
        self.attached_rerun_ids.push(copy.id.clone());
    }

    fn handle_retry(&mut self, test_info: &ReportGameListenerTestInfoModel, passed: bool) {
        let mut report_as = format!(
            "[Run: {:4}, Ok: {:4}, Fail: {:4}",
            self.attempts,
            self.successes,
            self.attempts - self.successes
        );
        if !test_info.retry_options.unlimited_tries() {
            report_as.push_str(&format!(
                ", Left: {:4}",
                test_info.retry_options.number_of_tries - self.attempts
            ));
        }
        report_as.push(']');

        let name_part = format!(
            "{} {}! {}ms",
            test_info.id,
            if passed { "passed" } else { "failed" },
            test_info.runtime_ms
        );
        let text = format!("{report_as:<53}{name_part}");
        if passed {
            self.report_passed(test_info, text);
        } else {
            self.say(ReportGameListenerChatFormat::Red, text);
        }

        if test_info
            .retry_options
            .has_tries_left(self.attempts, self.successes)
        {
            self.rerun(test_info);
        }
    }

    fn report_passed(&mut self, test_info: &ReportGameListenerTestInfoModel, text: String) {
        if test_info.block_entity_present {
            self.block_entity_events
                .push(ReportGameListenerBlockEntityEvent::Success);
        }
        self.say(ReportGameListenerChatFormat::Green, text);
        self.global_calls
            .push(TestReporterCall::Success(test_info.id.clone()));
    }

    fn report_failure(
        &mut self,
        test_info: &ReportGameListenerTestInfoModel,
        error: &ReportGameListenerErrorModel,
    ) {
        if test_info.block_entity_present {
            self.block_entity_events
                .push(ReportGameListenerBlockEntityEvent::ErrorMessage(
                    error
                        .assert_description
                        .clone()
                        .unwrap_or_else(|| error.described()),
                ));
        }
        self.visualize_failed_test(test_info, error);
    }

    fn visualize_failed_test(
        &mut self,
        test_info: &ReportGameListenerTestInfoModel,
        error: &ReportGameListenerErrorModel,
    ) {
        let cause = error
            .cause
            .as_ref()
            .map(|cause| format!(" cause: {cause}"))
            .unwrap_or_default();
        let error_message = format!("{}{}", error.message, cause);
        let optional_prefix = if test_info.required {
            ""
        } else {
            "(optional) "
        };
        self.say(
            if test_info.required {
                ReportGameListenerChatFormat::Red
            } else {
                ReportGameListenerChatFormat::Yellow
            },
            format!("{optional_prefix}{} failed! {error_message}", test_info.id),
        );

        if let (Some(absolute_pos), Some(block_message)) =
            (error.assert_pos, error.assert_block_message.as_ref())
        {
            self.block_entity_events
                .push(ReportGameListenerBlockEntityEvent::MarkError {
                    absolute_pos,
                    message: block_message.clone(),
                });
        }

        self.global_calls
            .push(TestReporterCall::Failed(test_info.id.clone()));
    }

    fn say(&mut self, format: ReportGameListenerChatFormat, text: String) {
        self.chats
            .push(ReportGameListenerChatEvent { format, text });
    }

    fn rerun(&mut self, test_info: &ReportGameListenerTestInfoModel) {
        self.reruns.push(test_info.id.clone());
    }

    pub fn global_reporter_info(
        test_info: &ReportGameListenerTestInfoModel,
    ) -> ReporterTestInfoModel {
        test_info.reporter_info()
    }
}
