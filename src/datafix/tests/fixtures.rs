//! One test per oracle fixture file.

use super::run_oracle_cases;

/// One test per oracle fixture. The fixture path is spelled out in full at each call site so
/// the tracked-file guard in `project_foundation_tests` can resolve every `include_str!`.
macro_rules! oracle_fixture {
    ($test:ident, $file:literal, $cases:expr) => {
        #[test]
        fn $test() {
            run_oracle_cases($file, $cases);
        }
    };
}
oracle_fixture!(fixes_v100_to_v113, "v100_to_113", include_str!("../fixtures/v100_to_113.cases"));
oracle_fixture!(fixes_v135_to_v505, "v135_to_505", include_str!("../fixtures/v135_to_505.cases"));
oracle_fixture!(fixes_v700_to_v1446, "v700_to_1446", include_str!("../fixtures/v700_to_1446.cases"));
oracle_fixture!(fixes_v1450_to_v1470, "v1450_to_1470", include_str!("../fixtures/v1450_to_1470.cases"));
oracle_fixture!(fixes_v1474_to_v1488, "v1474_to_1488", include_str!("../fixtures/v1474_to_1488.cases"));
oracle_fixture!(json_lenient_text_components, "json_lenient", include_str!("../fixtures/json_lenient.cases"));
oracle_fixture!(chunk_paletted_storage_random_chunks, "chunk_paletted_random", include_str!("../fixtures/chunk_paletted_random.cases"));
oracle_fixture!(fixes_v1490_to_v1624, "v1490_to_1624", include_str!("../fixtures/v1490_to_1624.cases"));
