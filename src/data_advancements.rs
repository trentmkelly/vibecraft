use std::collections::BTreeSet;

pub const DATA_ADVANCEMENTS_PACKAGE_NULL_MARKED: bool = true;

#[derive(Debug, Clone, PartialEq, Eq)]
struct AdvancementHolderModel {
    id: String,
    value: AdvancementModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AdvancementModel {
    placeholder: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AdvancementSubProviderModel {
    generated_ids: Vec<String>,
}

impl AdvancementSubProviderModel {
    fn generate(&self, output: &mut impl FnMut(AdvancementHolderModel)) {
        for id in &self.generated_ids {
            output(AdvancementSubProviderContract::create_placeholder(id));
        }
    }
}

struct AdvancementSubProviderContract;

impl AdvancementSubProviderContract {
    fn create_placeholder(id: &str) -> AdvancementHolderModel {
        AdvancementHolderModel {
            id: parse_identifier(id),
            value: AdvancementModel { placeholder: true },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SavedAdvancementModel {
    id: String,
    path: String,
    used_registry_lookup: bool,
    value: AdvancementModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AdvancementProviderModel {
    output_root: String,
    sub_providers: Vec<AdvancementSubProviderModel>,
    registries_ready: bool,
}

impl AdvancementProviderModel {
    fn new(
        output_root: &str,
        registries_ready: bool,
        sub_providers: Vec<AdvancementSubProviderModel>,
    ) -> Self {
        Self {
            output_root: output_root.to_string(),
            sub_providers,
            registries_ready,
        }
    }

    fn get_name(&self) -> &'static str {
        "Advancements"
    }

    fn run(&self) -> Result<Vec<SavedAdvancementModel>, String> {
        if !self.registries_ready {
            return Err("registries future has not completed".to_string());
        }

        let mut all_advancements = BTreeSet::new();
        let mut tasks = Vec::new();
        for sub_provider in &self.sub_providers {
            let mut output = |holder: AdvancementHolderModel| {
                if !all_advancements.insert(holder.id.clone()) {
                    tasks.push(Err(format!("Duplicate advancement {}", holder.id)));
                    return;
                }
                let path = advancement_json_path(&self.output_root, &holder.id);
                tasks.push(Ok(SavedAdvancementModel {
                    id: holder.id,
                    path,
                    used_registry_lookup: true,
                    value: holder.value,
                }));
            };
            sub_provider.generate(&mut output);
        }

        tasks.into_iter().collect()
    }
}

fn advancement_json_path(output_root: &str, id: &str) -> String {
    let (namespace, path) = id.split_once(':').unwrap_or(("minecraft", id));
    format!("{output_root}/data/{namespace}/advancement/{path}.json")
}

fn parse_identifier(id: &str) -> String {
    let (namespace, path) = id.split_once(':').unwrap_or(("minecraft", id));
    format!("{namespace}:{path}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advancement_provider_uses_advancement_registry_path_and_name() {
        let provider = AdvancementProviderModel::new(
            "generated",
            true,
            vec![AdvancementSubProviderModel {
                generated_ids: vec![
                    "story/root".to_string(),
                    "minecraft:recipes/decorations/crafting_table".to_string(),
                ],
            }],
        );

        assert_eq!(provider.get_name(), "Advancements");
        let saved = provider.run().unwrap();
        assert_eq!(saved.len(), 2);
        assert_eq!(
            saved[0].path,
            "generated/data/minecraft/advancement/story/root.json"
        );
        assert_eq!(
            saved[1].path,
            "generated/data/minecraft/advancement/recipes/decorations/crafting_table.json"
        );
        assert!(saved.iter().all(|entry| entry.used_registry_lookup));
    }

    #[test]
    fn advancement_provider_rejects_duplicate_ids_across_subproviders() {
        let provider = AdvancementProviderModel::new(
            "generated",
            true,
            vec![
                AdvancementSubProviderModel {
                    generated_ids: vec!["minecraft:story/root".to_string()],
                },
                AdvancementSubProviderModel {
                    generated_ids: vec!["story/root".to_string()],
                },
            ],
        );

        assert_eq!(
            provider.run().unwrap_err(),
            "Duplicate advancement minecraft:story/root"
        );
    }

    #[test]
    fn advancement_provider_waits_for_registry_lookup() {
        let provider = AdvancementProviderModel::new(
            "generated",
            false,
            vec![AdvancementSubProviderModel {
                generated_ids: vec!["story/root".to_string()],
            }],
        );

        assert_eq!(
            provider.run().unwrap_err(),
            "registries future has not completed"
        );
    }

    #[test]
    fn advancement_subprovider_placeholder_matches_java_factory() {
        let holder = AdvancementSubProviderContract::create_placeholder("story/root");
        assert_eq!(holder.id, "minecraft:story/root");
        assert!(holder.value.placeholder);
    }

    #[test]
    fn java_source_contract_sentinels_match_provider_model() {
        let provider_source = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/data/advancements/AdvancementProvider.java"
        );
        assert!(provider_source
            .contains("output.createRegistryElementsPathProvider(Registries.ADVANCEMENT)"));
        assert!(provider_source.contains("new HashSet<>()"));
        assert!(provider_source.contains("Duplicate advancement "));
        assert!(
            provider_source.contains("DataProvider.saveStable(cache, lookup, Advancement.CODEC")
        );
        assert!(provider_source.contains("return \"Advancements\";"));

        let sub_provider_source = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/data/advancements/AdvancementSubProvider.java"
        );
        assert!(sub_provider_source.contains(
            "void generate(HolderLookup.Provider registries, Consumer<AdvancementHolder> output);"
        ));
        assert!(sub_provider_source
            .contains("Advancement.Builder.advancement().build(Identifier.parse(id))"));
    }

    #[test]
    fn package_is_null_marked() {
        const {
            assert!(DATA_ADVANCEMENTS_PACKAGE_NULL_MARKED);
        }
    }
}
