const JAVA_SOURCE: &str = vibecraft_java_source!("/net/minecraft/data/loot/packs/VanillaShearingLoot.java");

fn wool_by_dye_order() -> &'static [(&'static str, &'static str)] {
    &[
        ("white", "white_wool"),
        ("orange", "orange_wool"),
        ("magenta", "magenta_wool"),
        ("light_blue", "light_blue_wool"),
        ("yellow", "yellow_wool"),
        ("lime", "lime_wool"),
        ("pink", "pink_wool"),
        ("gray", "gray_wool"),
        ("light_gray", "light_gray_wool"),
        ("cyan", "cyan_wool"),
        ("purple", "purple_wool"),
        ("blue", "blue_wool"),
        ("brown", "brown_wool"),
        ("green", "green_wool"),
        ("red", "red_wool"),
        ("black", "black_wool"),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShearingTableSummary {
    id: &'static str,
    roll_shape: &'static str,
    entries: Vec<&'static str>,
}

fn special_shearing_tables() -> Vec<ShearingTableSummary> {
    vec![
        ShearingTableSummary {
            id: "minecraft:gameplay/shearing/bogged",
            roll_shape: "constant:2",
            entries: vec!["brown_mushroom:count=1", "red_mushroom:count=1"],
        },
        ShearingTableSummary {
            id: "minecraft:gameplay/shearing/sheep",
            roll_shape: "dispatch:sheep_by_dye",
            entries: vec!["shear_sheep_by_dye"],
        },
        ShearingTableSummary {
            id: "minecraft:gameplay/shearing/mooshroom",
            roll_shape: "constant:1",
            entries: vec![
                "red_variant->shear_red_mooshroom",
                "brown_variant->shear_brown_mooshroom",
            ],
        },
        ShearingTableSummary {
            id: "minecraft:gameplay/shearing/red_mooshroom",
            roll_shape: "constant:5",
            entries: vec!["red_mushroom"],
        },
        ShearingTableSummary {
            id: "minecraft:gameplay/shearing/brown_mooshroom",
            roll_shape: "constant:5",
            entries: vec!["brown_mushroom"],
        },
        ShearingTableSummary {
            id: "minecraft:gameplay/shearing/snow_golem",
            roll_shape: "constant:1",
            entries: vec!["carved_pumpkin"],
        },
    ]
}

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(expected: &[&str]) {
    for sentinel in expected {
        assert!(
            JAVA_SOURCE.contains(sentinel),
            "missing Java sentinel: {sentinel}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shearing_wool_tables_follow_loot_data_dye_order() {
        assert_eq!(wool_by_dye_order().len(), 16);
        assert_eq!(
            wool_by_dye_order(),
            &[
                ("white", "white_wool"),
                ("orange", "orange_wool"),
                ("magenta", "magenta_wool"),
                ("light_blue", "light_blue_wool"),
                ("yellow", "yellow_wool"),
                ("lime", "lime_wool"),
                ("pink", "pink_wool"),
                ("gray", "gray_wool"),
                ("light_gray", "light_gray_wool"),
                ("cyan", "cyan_wool"),
                ("purple", "purple_wool"),
                ("blue", "blue_wool"),
                ("brown", "brown_wool"),
                ("green", "green_wool"),
                ("red", "red_wool"),
                ("black", "black_wool"),
            ]
        );
        assert_source_contains_all(&[
            "LootData.WOOL_ITEM_BY_DYE",
            "BuiltInLootTables.SHEAR_SHEEP_BY_DYE.get(dye)",
            "LootPool.lootPool().setRolls(UniformGenerator.between(1.0F, 3.0F)).add(LootItem.lootTableItem(wool))",
        ]);
    }

    #[test]
    fn special_shearing_tables_match_java_shapes() {
        let tables = special_shearing_tables();
        assert_eq!(
            tables.iter().map(|table| table.id).collect::<Vec<_>>(),
            vec![
                "minecraft:gameplay/shearing/bogged",
                "minecraft:gameplay/shearing/sheep",
                "minecraft:gameplay/shearing/mooshroom",
                "minecraft:gameplay/shearing/red_mooshroom",
                "minecraft:gameplay/shearing/brown_mooshroom",
                "minecraft:gameplay/shearing/snow_golem",
            ]
        );
        assert_eq!(
            tables
                .iter()
                .map(|table| (table.roll_shape, table.entries.clone()))
                .collect::<Vec<_>>(),
            vec![
                (
                    "constant:2",
                    vec!["brown_mushroom:count=1", "red_mushroom:count=1"]
                ),
                ("dispatch:sheep_by_dye", vec!["shear_sheep_by_dye"]),
                (
                    "constant:1",
                    vec![
                        "red_variant->shear_red_mooshroom",
                        "brown_variant->shear_brown_mooshroom"
                    ]
                ),
                ("constant:5", vec!["red_mushroom"]),
                ("constant:5", vec!["brown_mushroom"]),
                ("constant:1", vec!["carved_pumpkin"]),
            ]
        );
    }

    #[test]
    fn shearing_java_source_counts_match_authoritative_file() {
        assert_eq!(count_occurrences(JAVA_SOURCE, "output.accept"), 7);
        assert_eq!(count_occurrences(JAVA_SOURCE, "LootPool.lootPool("), 6);
        assert_eq!(count_occurrences(JAVA_SOURCE, "LootItem.lootTableItem"), 6);
        assert_eq!(
            count_occurrences(JAVA_SOURCE, "NestedLootTable.lootTableReference"),
            2
        );
        assert_eq!(
            count_occurrences(JAVA_SOURCE, "AlternativesEntry.alternatives"),
            1
        );
        assert_eq!(
            count_occurrences(JAVA_SOURCE, "SetItemCountFunction.setCount"),
            2
        );
        assert_eq!(
            count_occurrences(JAVA_SOURCE, "LootItemEntityPropertyCondition.hasProperties"),
            2
        );
        assert_eq!(
            count_occurrences(JAVA_SOURCE, "DataComponentExactPredicate.expect"),
            2
        );
        assert_eq!(count_occurrences(JAVA_SOURCE, "ConstantValue.exactly"), 6);
        assert_eq!(
            count_occurrences(JAVA_SOURCE, "UniformGenerator.between"),
            1
        );
    }

    #[test]
    fn shearing_java_source_sentinels_match_authoritative_file() {
        assert_source_contains_all(&[
            "public record VanillaShearingLoot(HolderLookup.Provider registries) implements LootTableSubProvider",
            "BuiltInLootTables.BOGGED_SHEAR",
            "LootPool.lootPool()\n                  .setRolls(ConstantValue.exactly(2.0F))",
            "LootItem.lootTableItem(Items.BROWN_MUSHROOM).apply(SetItemCountFunction.setCount(ConstantValue.exactly(1.0F)))",
            "LootItem.lootTableItem(Items.RED_MUSHROOM).apply(SetItemCountFunction.setCount(ConstantValue.exactly(1.0F)))",
            "BuiltInLootTables.SHEAR_SHEEP_BY_DYE.get(dye)",
            "BuiltInLootTables.SHEAR_SHEEP, LootTable.lootTable().withPool(EntityLootSubProvider.createSheepDispatchPool(BuiltInLootTables.SHEAR_SHEEP_BY_DYE))",
            "BuiltInLootTables.SHEAR_MOOSHROOM",
            "AlternativesEntry.alternatives(",
            "NestedLootTable.lootTableReference(BuiltInLootTables.SHEAR_RED_MOOSHROOM)",
            "DataComponentExactPredicate.expect(DataComponents.MOOSHROOM_VARIANT, MushroomCow.Variant.RED)",
            "NestedLootTable.lootTableReference(BuiltInLootTables.SHEAR_BROWN_MOOSHROOM)",
            "DataComponentExactPredicate.expect(DataComponents.MOOSHROOM_VARIANT, MushroomCow.Variant.BROWN)",
            "BuiltInLootTables.SHEAR_RED_MOOSHROOM",
            "LootPool.lootPool().setRolls(ConstantValue.exactly(5.0F)).add(LootItem.lootTableItem(Items.RED_MUSHROOM))",
            "BuiltInLootTables.SHEAR_BROWN_MOOSHROOM",
            "LootPool.lootPool().setRolls(ConstantValue.exactly(5.0F)).add(LootItem.lootTableItem(Items.BROWN_MUSHROOM))",
            "BuiltInLootTables.SHEAR_SNOW_GOLEM",
            "LootPool.lootPool().setRolls(ConstantValue.exactly(1.0F)).add(LootItem.lootTableItem(Items.CARVED_PUMPKIN))",
        ]);
    }
}
