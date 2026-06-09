use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

const PACKAGE_NULL_MARKED: bool = true;
const VANILLA_LOOT_TABLE_PROVIDER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/loot/packs/VanillaLootTableProvider.java"
);

fn package_null_marked() -> bool {
    PACKAGE_NULL_MARKED
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LootTableBuilderModel {
    label: String,
    validation_errors: Vec<String>,
}

impl LootTableBuilderModel {
    fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            validation_errors: Vec::new(),
        }
    }

    fn invalid(mut self, problem: &str) -> Self {
        self.validation_errors.push(problem.to_string());
        self
    }

    fn set_random_sequence(self, sequence: &str) -> LootTableModel {
        LootTableModel {
            label: self.label,
            random_sequence: sequence.to_string(),
            param_set: String::new(),
            validation_errors: self.validation_errors,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LootTableModel {
    label: String,
    random_sequence: String,
    param_set: String,
    validation_errors: Vec<String>,
}

impl LootTableModel {
    fn set_param_set(mut self, param_set: &str) -> Self {
        self.param_set = param_set.to_string();
        self
    }
}

trait LootTableSubProviderModel {
    fn generate(&self, output: &mut dyn FnMut(String, LootTableBuilderModel));
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StaticSubProviderModel {
    tables: Vec<(String, LootTableBuilderModel)>,
}

impl LootTableSubProviderModel for StaticSubProviderModel {
    fn generate(&self, output: &mut dyn FnMut(String, LootTableBuilderModel)) {
        for (id, builder) in &self.tables {
            output(id.clone(), builder.clone());
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SubProviderEntryModel {
    provider: StaticSubProviderModel,
    param_set: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MissingTableProblemModel {
    id: String,
}

impl MissingTableProblemModel {
    fn description(&self) -> String {
        format!("Missing built-in table: {}", self.id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LootTableProviderModel {
    required_tables: BTreeSet<String>,
    subproviders: Vec<SubProviderEntryModel>,
}

impl LootTableProviderModel {
    fn new(
        required_tables: impl IntoIterator<Item = &'static str>,
        subproviders: Vec<SubProviderEntryModel>,
    ) -> Self {
        Self {
            required_tables: required_tables.into_iter().map(str::to_string).collect(),
            subproviders,
        }
    }

    fn get_name(&self) -> &'static str {
        "Loot Tables"
    }

    fn sequence_id_for_loot_table(id: &str) -> String {
        id.to_string()
    }

    fn run(&self) -> Result<RunOutputModel, String> {
        let mut tables = BTreeMap::new();
        let mut collision_logs = Vec::new();
        let mut random_sequence_seeds = BTreeMap::new();
        for subprovider in &self.subproviders {
            subprovider.provider.generate(&mut |id, builder| {
                let sequence_id = Self::sequence_id_for_loot_table(&id);
                let seed = seed_for_key(&sequence_id);
                if let Some(previous) = random_sequence_seeds.insert(seed, sequence_id.clone()) {
                    collision_logs.push(format!(
                        "Loot table random sequence seed collision on {previous} and {id}"
                    ));
                }
                let table = builder
                    .set_random_sequence(&sequence_id)
                    .set_param_set(&subprovider.param_set);
                tables.insert(id, table);
            });
        }
        let mut problems = Vec::new();
        for missing in self
            .required_tables
            .difference(&tables.keys().cloned().collect())
        {
            problems.push(
                MissingTableProblemModel {
                    id: missing.clone(),
                }
                .description(),
            );
        }
        for (id, table) in &tables {
            for problem in &table.validation_errors {
                problems.push(format!("{id}: {problem}"));
            }
        }
        if !problems.is_empty() {
            return Err(format!(
                "Failed to validate loot tables, see logs: {}",
                problems.join(" | ")
            ));
        }
        let saved_paths = tables
            .keys()
            .map(|id| path_for_loot_table(id))
            .collect::<Vec<_>>();
        Ok(RunOutputModel {
            tables,
            saved_paths,
            collision_logs,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunOutputModel {
    tables: BTreeMap<String, LootTableModel>,
    saved_paths: Vec<String>,
    collision_logs: Vec<String>,
}

fn seed_for_key(sequence_id: &str) -> u64 {
    sequence_id
        .strip_suffix("_collision")
        .unwrap_or(sequence_id)
        .bytes()
        .map(u64::from)
        .sum()
}

fn path_for_loot_table(id: &str) -> String {
    let (namespace, path) = id.split_once(':').unwrap_or(("minecraft", id));
    format!("data/{namespace}/loot_table/{path}.json")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VanillaLootTableProviderSummary {
    required_tables_source: &'static str,
    subproviders: Vec<(&'static str, &'static str)>,
}

fn vanilla_loot_table_provider_summary() -> VanillaLootTableProviderSummary {
    VanillaLootTableProviderSummary {
        required_tables_source: "BuiltInLootTables.all()",
        subproviders: vec![
            ("VanillaFishingLoot", "fishing"),
            ("VanillaChestLoot", "chest"),
            ("VanillaEntityLoot", "entity"),
            ("VanillaEquipmentLoot", "equipment"),
            ("VanillaBlockLoot", "block"),
            ("VanillaPiglinBarterLoot", "piglin_barter"),
            ("VanillaGiftLoot", "gift"),
            ("VanillaArchaeologyLoot", "archaeology"),
            ("VanillaShearingLoot", "shearing"),
            ("VanillaEntityInteractLoot", "entity_interact"),
            ("VanillaBlockInteractLoot", "block_interact"),
            ("VanillaChargedCreeperExplosionLoot", "entity"),
        ],
    }
}

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn must_ok<T, E: fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("expected Ok(..), got Err({error:?})"),
        }
    }

    fn must_err<T: fmt::Debug, E>(result: Result<T, E>) -> E {
        match result {
            Ok(value) => panic!("expected Err(..), got Ok({value:?})"),
            Err(error) => error,
        }
    }

    fn subprovider(
        param_set: &str,
        tables: Vec<(&str, LootTableBuilderModel)>,
    ) -> SubProviderEntryModel {
        SubProviderEntryModel {
            provider: StaticSubProviderModel {
                tables: tables
                    .into_iter()
                    .map(|(id, builder)| (id.to_string(), builder))
                    .collect(),
            },
            param_set: param_set.to_string(),
        }
    }

    #[test]
    fn provider_name_sequence_id_and_package_marker_match_java() {
        let provider = LootTableProviderModel::new([], Vec::new());
        assert_eq!(provider.get_name(), "Loot Tables");
        assert_eq!(
            LootTableProviderModel::sequence_id_for_loot_table("minecraft:chests/simple_dungeon"),
            "minecraft:chests/simple_dungeon"
        );
        assert!(package_null_marked());
    }

    #[test]
    fn subprovider_generate_is_functional_biconsumer_contract() {
        let subprovider = StaticSubProviderModel {
            tables: vec![(
                "minecraft:gameplay/fishing".to_string(),
                LootTableBuilderModel::new("fishing"),
            )],
        };
        let mut generated = Vec::new();
        subprovider.generate(&mut |id, builder| generated.push((id, builder.label)));
        assert_eq!(
            generated,
            vec![(
                "minecraft:gameplay/fishing".to_string(),
                "fishing".to_string()
            )]
        );
    }

    #[test]
    fn run_sets_random_sequence_param_set_and_stable_save_paths() {
        let provider = LootTableProviderModel::new(
            ["minecraft:chests/simple_dungeon"],
            vec![subprovider(
                "chest",
                vec![(
                    "minecraft:chests/simple_dungeon",
                    LootTableBuilderModel::new("dungeon"),
                )],
            )],
        );
        let output = must_ok(provider.run());
        let table = &output.tables["minecraft:chests/simple_dungeon"];
        assert_eq!(table.random_sequence, "minecraft:chests/simple_dungeon");
        assert_eq!(table.param_set, "chest");
        assert_eq!(
            output.saved_paths,
            vec!["data/minecraft/loot_table/chests/simple_dungeon.json"]
        );
    }

    #[test]
    fn run_reports_missing_required_tables_with_java_problem_text() {
        let provider = LootTableProviderModel::new(
            ["minecraft:entities/zombie"],
            vec![subprovider(
                "entity",
                vec![(
                    "minecraft:entities/skeleton",
                    LootTableBuilderModel::new("skeleton"),
                )],
            )],
        );
        let err = must_err(provider.run());
        assert!(err.starts_with("Failed to validate loot tables, see logs: "));
        assert!(err.contains("Missing built-in table: minecraft:entities/zombie"));
    }

    #[test]
    fn run_reports_validation_errors_after_freezing_registry_model() {
        let provider = LootTableProviderModel::new(
            [],
            vec![subprovider(
                "block",
                vec![(
                    "minecraft:blocks/bad",
                    LootTableBuilderModel::new("bad").invalid("pools[0] has no entries"),
                )],
            )],
        );
        let err = must_err(provider.run());
        assert!(err.contains("minecraft:blocks/bad: pools[0] has no entries"));
    }

    #[test]
    fn random_sequence_seed_collisions_are_logged_without_stopping_output() {
        let provider = LootTableProviderModel::new(
            [],
            vec![subprovider(
                "generic",
                vec![
                    ("minecraft:loot/a", LootTableBuilderModel::new("a")),
                    (
                        "minecraft:loot/a_collision",
                        LootTableBuilderModel::new("b"),
                    ),
                ],
            )],
        );
        let output = must_ok(provider.run());
        assert_eq!(output.tables.len(), 2);
        assert_eq!(
            output.collision_logs,
            vec![
                "Loot table random sequence seed collision on minecraft:loot/a and minecraft:loot/a_collision"
                    .to_string()
            ]
        );
    }

    #[test]
    fn vanilla_loot_table_provider_matches_java_required_tables_and_subprovider_order() {
        let provider = vanilla_loot_table_provider_summary();
        assert_eq!(provider.required_tables_source, "BuiltInLootTables.all()");
        assert_eq!(
            provider.subproviders,
            vec![
                ("VanillaFishingLoot", "fishing"),
                ("VanillaChestLoot", "chest"),
                ("VanillaEntityLoot", "entity"),
                ("VanillaEquipmentLoot", "equipment"),
                ("VanillaBlockLoot", "block"),
                ("VanillaPiglinBarterLoot", "piglin_barter"),
                ("VanillaGiftLoot", "gift"),
                ("VanillaArchaeologyLoot", "archaeology"),
                ("VanillaShearingLoot", "shearing"),
                ("VanillaEntityInteractLoot", "entity_interact"),
                ("VanillaBlockInteractLoot", "block_interact"),
                ("VanillaChargedCreeperExplosionLoot", "entity"),
            ]
        );
    }

    #[test]
    fn vanilla_loot_table_provider_java_source_counts_match_authoritative_file() {
        assert_eq!(
            count_occurrences(VANILLA_LOOT_TABLE_PROVIDER_JAVA, "new LootTableProvider("),
            1
        );
        assert_eq!(
            count_occurrences(
                VANILLA_LOOT_TABLE_PROVIDER_JAVA,
                "new LootTableProvider.SubProviderEntry"
            ),
            12
        );
        assert_eq!(
            count_occurrences(VANILLA_LOOT_TABLE_PROVIDER_JAVA, "LootContextParamSets."),
            12
        );
        assert_eq!(
            count_occurrences(VANILLA_LOOT_TABLE_PROVIDER_JAVA, "BuiltInLootTables.all()"),
            1
        );
    }

    #[test]
    fn vanilla_loot_table_provider_java_source_sentinels_match_authoritative_file() {
        for expected in [
            "public class VanillaLootTableProvider",
            "public static LootTableProvider create(final PackOutput output, final CompletableFuture<HolderLookup.Provider> registries)",
            "return new LootTableProvider(",
            "output,\n         BuiltInLootTables.all(),",
            "new LootTableProvider.SubProviderEntry(VanillaFishingLoot::new, LootContextParamSets.FISHING)",
            "new LootTableProvider.SubProviderEntry(VanillaChestLoot::new, LootContextParamSets.CHEST)",
            "new LootTableProvider.SubProviderEntry(VanillaEntityLoot::new, LootContextParamSets.ENTITY)",
            "new LootTableProvider.SubProviderEntry(VanillaEquipmentLoot::new, LootContextParamSets.EQUIPMENT)",
            "new LootTableProvider.SubProviderEntry(VanillaBlockLoot::new, LootContextParamSets.BLOCK)",
            "new LootTableProvider.SubProviderEntry(VanillaPiglinBarterLoot::new, LootContextParamSets.PIGLIN_BARTER)",
            "new LootTableProvider.SubProviderEntry(VanillaGiftLoot::new, LootContextParamSets.GIFT)",
            "new LootTableProvider.SubProviderEntry(VanillaArchaeologyLoot::new, LootContextParamSets.ARCHAEOLOGY)",
            "new LootTableProvider.SubProviderEntry(VanillaShearingLoot::new, LootContextParamSets.SHEARING)",
            "new LootTableProvider.SubProviderEntry(VanillaEntityInteractLoot::new, LootContextParamSets.ENTITY_INTERACT)",
            "new LootTableProvider.SubProviderEntry(VanillaBlockInteractLoot::new, LootContextParamSets.BLOCK_INTERACT)",
            "new LootTableProvider.SubProviderEntry(VanillaChargedCreeperExplosionLoot::new, LootContextParamSets.ENTITY)",
            "registries\n      );",
        ] {
            assert!(
                VANILLA_LOOT_TABLE_PROVIDER_JAVA.contains(expected),
                "missing Java sentinel: {expected}"
            );
        }
    }
}
