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
