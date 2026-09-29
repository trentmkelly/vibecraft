//! One test per oracle fixture file.

use super::run_oracle_cases;

macro_rules! oracle_fixture {
    ($test:ident, $file:literal) => {
        #[test]
        fn $test() {
            run_oracle_cases(
                $file,
                include_str!(concat!("../fixtures/", $file, ".cases")),
            );
        }
    };
}
oracle_fixture!(fixes_v100_to_v113, "v100_to_113");
oracle_fixture!(fixes_v135_to_v505, "v135_to_505");
oracle_fixture!(fixes_v700_to_v1446, "v700_to_1446");
oracle_fixture!(fixes_v1450_to_v1470, "v1450_to_1470");
oracle_fixture!(fixes_v1474_to_v1488, "v1474_to_1488");
oracle_fixture!(json_lenient_text_components, "json_lenient");
oracle_fixture!(
    chunk_paletted_storage_random_chunks,
    "chunk_paletted_random"
);
oracle_fixture!(fixes_v1490_to_v1624, "v1490_to_1624");
