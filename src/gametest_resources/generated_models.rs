use std::collections::BTreeMap;

pub const GENERATED_TEST_FUNCTION_REGISTRY: &str = "test_function";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedTestFunctionKey {
    pub registry: &'static str,
    pub id: String,
}

impl GeneratedTestFunctionKey {
    pub fn create(function_id: impl Into<String>) -> Self {
        Self {
            registry: GENERATED_TEST_FUNCTION_REGISTRY,
            id: function_id.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedTestModel<TestData> {
    pub tests: BTreeMap<String, TestData>,
    pub function_key: GeneratedTestFunctionKey,
    pub function: String,
}

impl<TestData> GeneratedTestModel<TestData> {
    pub fn new(
        tests: BTreeMap<String, TestData>,
        function_key: GeneratedTestFunctionKey,
        function: impl Into<String>,
    ) -> Self {
        Self {
            tests,
            function_key,
            function: function.into(),
        }
    }

    pub fn from_function_id(
        tests: BTreeMap<String, TestData>,
        function_id: impl Into<String>,
        function: impl Into<String>,
    ) -> Self {
        Self::new(
            tests,
            GeneratedTestFunctionKey::create(function_id),
            function,
        )
    }

    pub fn single(id: impl Into<String>, test_data: TestData, function: impl Into<String>) -> Self {
        let id = id.into();
        let mut tests = BTreeMap::new();
        tests.insert(id.clone(), test_data);
        Self::from_function_id(tests, id, function)
    }
}
