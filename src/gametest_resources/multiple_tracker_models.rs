use super::GameTestInfoStateModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackedGameTestInfoModel {
    pub info: GameTestInfoStateModel,
    pub listener_count: usize,
}

impl TrackedGameTestInfoModel {
    pub fn new(info: GameTestInfoStateModel, listener_count: usize) -> Self {
        Self {
            info,
            listener_count,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MultipleTestTrackerModel {
    pub tests: Vec<TrackedGameTestInfoModel>,
    pub listener_count: usize,
    pub failure_listener_count: usize,
    pub failure_notifications: Vec<String>,
}

impl MultipleTestTrackerModel {
    pub fn new(tests: Vec<GameTestInfoStateModel>) -> Self {
        Self {
            tests: tests
                .into_iter()
                .map(|info| TrackedGameTestInfoModel::new(info, 0))
                .collect(),
            listener_count: 0,
            failure_listener_count: 0,
            failure_notifications: Vec::new(),
        }
    }

    pub fn add_test_to_track(&mut self, test_info: GameTestInfoStateModel) {
        self.tests.push(TrackedGameTestInfoModel::new(
            test_info,
            self.listener_count,
        ));
    }

    pub fn add_listener(&mut self) {
        self.listener_count += 1;
        for test in &mut self.tests {
            test.listener_count += 1;
        }
    }

    pub fn add_failure_listener(&mut self) {
        self.failure_listener_count += 1;
        self.add_listener();
    }

    pub fn notify_test_failed(&mut self, test_id: &str) {
        for _ in 0..self.failure_listener_count {
            self.failure_notifications.push(test_id.to_string());
        }
    }

    pub fn get_failed_required_count(&self) -> usize {
        self.tests
            .iter()
            .filter(|test| test.info.has_failed() && test.info.is_required())
            .count()
    }

    pub fn get_failed_optional_count(&self) -> usize {
        self.tests
            .iter()
            .filter(|test| test.info.has_failed() && test.info.is_optional())
            .count()
    }

    pub fn get_done_count(&self) -> usize {
        self.tests.iter().filter(|test| test.info.done).count()
    }

    pub fn has_failed_required(&self) -> bool {
        self.get_failed_required_count() > 0
    }

    pub fn has_failed_optional(&self) -> bool {
        self.get_failed_optional_count() > 0
    }

    pub fn get_failed_required(&self) -> Vec<String> {
        self.tests
            .iter()
            .filter(|test| test.info.has_failed() && test.info.is_required())
            .map(|test| test.info.id.clone())
            .collect()
    }

    pub fn get_failed_optional(&self) -> Vec<String> {
        self.tests
            .iter()
            .filter(|test| test.info.has_failed() && test.info.is_optional())
            .map(|test| test.info.id.clone())
            .collect()
    }

    pub fn get_total_count(&self) -> usize {
        self.tests.len()
    }

    pub fn is_done(&self) -> bool {
        self.get_done_count() == self.get_total_count()
    }

    pub fn get_progress_bar(&self) -> String {
        let mut progress = String::with_capacity(self.tests.len() + 2);
        progress.push('[');
        for test in &self.tests {
            if !test.info.started {
                progress.push(' ');
            } else if test.info.has_succeeded() {
                progress.push('+');
            } else if test.info.has_failed() {
                progress.push(if test.info.is_required() { 'X' } else { 'x' });
            } else {
                progress.push('_');
            }
        }
        progress.push(']');
        progress
    }

    pub fn remove(&mut self, test_id: &str) {
        self.tests.retain(|test| test.info.id != test_id);
    }
}

impl std::fmt::Display for MultipleTestTrackerModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.get_progress_bar())
    }
}
