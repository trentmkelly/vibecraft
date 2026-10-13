use super::*;

fn decoded(raw: &str) -> LootTable {
    decode_loot_table(raw).expect("loot table JSON decodes")
}

fn evaluate_with_tables(
    table: &LootTable,
    tables: &[(&str, LootTable)],
) -> (Vec<LootStack>, Vec<String>) {
    let mut context = LootContext::new(LootParamSet::AllParams, 17);
    context.tables = tables
        .iter()
        .map(|(id, table)| ((*id).to_string(), table.clone()))
        .collect();
    let stacks = table.evaluate(&mut context);
    (stacks, context.warnings)
}

#[test]
fn separate_tables_with_the_same_random_sequence_do_not_look_recursive() {
    let top = decoded(
        r#"{"random_sequence":"minecraft:chests/test","pools":[{"rolls":1,"entries":[{"type":"minecraft:loot_table","value":"minecraft:test/nested"}]}]}"#,
    );
    let nested = decoded(
        r#"{"random_sequence":"minecraft:chests/test","pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:apple"}]}]}"#,
    );

    let (stacks, warnings) = evaluate_with_tables(&top, &[("minecraft:test/nested", nested)]);

    assert_eq!(stacks, vec![LootStack::new("minecraft:apple", 1)]);
    assert!(warnings.is_empty());
}

#[test]
fn cloned_table_identity_detects_direct_recursion() {
    let table = decoded(
        r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:loot_table","value":"minecraft:test/self"}]}]}"#,
    );

    let (stacks, warnings) =
        evaluate_with_tables(&table, &[("minecraft:test/self", table.clone())]);

    assert!(stacks.is_empty());
    assert_eq!(warnings, vec!["Detected infinite loop in loot tables"]);
}

#[test]
fn cloned_table_identities_detect_indirect_recursion() {
    let first = decoded(
        r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:loot_table","value":"minecraft:test/second"}]}]}"#,
    );
    let second = decoded(
        r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:loot_table","value":"minecraft:test/first"}]}]}"#,
    );

    let (stacks, warnings) = evaluate_with_tables(
        &first,
        &[
            ("minecraft:test/first", first.clone()),
            ("minecraft:test/second", second.clone()),
        ],
    );

    assert!(stacks.is_empty());
    assert_eq!(warnings, vec!["Detected infinite loop in loot tables"]);
}

#[test]
fn sequential_references_to_the_same_table_are_allowed() {
    let top = decoded(
        r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:loot_table","value":"minecraft:test/nested"}]},{"rolls":1,"entries":[{"type":"minecraft:loot_table","value":"minecraft:test/nested"}]}]}"#,
    );
    let nested = decoded(
        r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:apple"}]}]}"#,
    );

    let (stacks, warnings) = evaluate_with_tables(&top, &[("minecraft:test/nested", nested)]);

    assert_eq!(stacks, vec![LootStack::new("minecraft:apple", 1); 2]);
    assert!(warnings.is_empty());
}
