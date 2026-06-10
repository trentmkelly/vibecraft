use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
struct LootTableBuilderModel(&'static str);

#[derive(Debug, Clone, PartialEq, Eq)]
struct LootPoolModel {
    entries: Vec<LootEntryModel>,
}

impl LootPoolModel {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn add(mut self, entry: LootEntryModel) -> Self {
        self.entries.push(entry);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LootEntryModel {
    Alternatives(Vec<LootEntryModel>),
    NestedTable {
        table: String,
        conditions: Vec<LootConditionModel>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LootConditionModel {
    AnyOf(Vec<LootConditionModel>),
    EntityOnFire { target: LootEntityTargetModel },
    DirectAttackerMainhandEnchantmentTag(&'static str),
    SheepColorAndHasWool { color: DyeColorModel },
    DamageSourceEntityType { entity_type: &'static str },
    DamageSourceFrogVariant { variant: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LootEntityTargetModel {
    This,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DyeColorModel {
    White,
    Orange,
    Magenta,
    LightBlue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EntityTypeModel {
    registry_key: String,
    default_loot_table: Option<String>,
    allowed_enabled: bool,
    required_enabled: bool,
}

impl EntityTypeModel {
    fn living(key: &str) -> Self {
        Self {
            registry_key: key.to_string(),
            default_loot_table: Some(format!(
                "minecraft:entities/{}",
                key.replace("minecraft:", "")
            )),
            allowed_enabled: true,
            required_enabled: true,
        }
    }

    fn non_living(key: &str) -> Self {
        Self {
            registry_key: key.to_string(),
            default_loot_table: None,
            allowed_enabled: true,
            required_enabled: true,
        }
    }

    fn allowed_disabled(mut self) -> Self {
        self.allowed_enabled = false;
        self
    }

    fn required_disabled(mut self) -> Self {
        self.required_enabled = false;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EntityLootSubProviderModel {
    map: BTreeMap<String, BTreeMap<String, LootTableBuilderModel>>,
}

impl EntityLootSubProviderModel {
    fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    fn should_smelt_loot(&self) -> LootConditionModel {
        LootConditionModel::AnyOf(vec![
            LootConditionModel::EntityOnFire {
                target: LootEntityTargetModel::This,
            },
            LootConditionModel::DirectAttackerMainhandEnchantmentTag("minecraft:smelts_loot"),
        ])
    }

    fn create_sheep_dispatch_pool(table_names: BTreeMap<DyeColorModel, String>) -> LootPoolModel {
        let variants = table_names
            .into_iter()
            .map(|(color, table)| LootEntryModel::NestedTable {
                table,
                conditions: vec![LootConditionModel::SheepColorAndHasWool { color }],
            })
            .collect();
        LootPoolModel::new().add(LootEntryModel::Alternatives(variants))
    }

    fn killed_by_frog(&self) -> LootConditionModel {
        LootConditionModel::DamageSourceEntityType {
            entity_type: "minecraft:frog",
        }
    }

    fn killed_by_frog_variant(&self, variant: &str) -> LootConditionModel {
        LootConditionModel::DamageSourceFrogVariant {
            variant: variant.to_string(),
        }
    }

    fn add_default(
        &mut self,
        entity_type: &EntityTypeModel,
        builder: LootTableBuilderModel,
    ) -> Result<(), String> {
        let Some(default) = &entity_type.default_loot_table else {
            return Err(format!(
                "Entity {} has no loot table",
                entity_type.registry_key
            ));
        };
        self.add(entity_type, default, builder);
        Ok(())
    }

    fn add(
        &mut self,
        entity_type: &EntityTypeModel,
        loot_table: &str,
        builder: LootTableBuilderModel,
    ) {
        self.map
            .entry(entity_type.registry_key.clone())
            .or_default()
            .insert(loot_table.to_string(), builder);
    }

    fn generate(
        &mut self,
        entity_types: &[EntityTypeModel],
        generated_by_subclass: impl FnOnce(&mut Self) -> Result<(), String>,
    ) -> Result<Vec<(String, LootTableBuilderModel)>, String> {
        generated_by_subclass(self)?;
        let mut seen = BTreeSet::new();
        let mut output = Vec::new();
        for entity_type in entity_types
            .iter()
            .filter(|entity_type| entity_type.allowed_enabled)
        {
            if let Some(default_loot_table) = &entity_type.default_loot_table {
                let builders = self.map.remove(&entity_type.registry_key);
                if entity_type.required_enabled
                    && builders
                        .as_ref()
                        .is_none_or(|tables| !tables.contains_key(default_loot_table))
                {
                    return Err(format!(
                        "Missing loottable '{default_loot_table}' for '{}'",
                        entity_type.registry_key
                    ));
                }
                if let Some(builders) = builders {
                    for (id, builder) in builders {
                        if !seen.insert(id.clone()) {
                            return Err(format!(
                                "Duplicate loottable '{id}' for '{}'",
                                entity_type.registry_key
                            ));
                        }
                        output.push((id, builder));
                    }
                }
            } else if let Some(builders) = self.map.remove(&entity_type.registry_key) {
                return Err(format!(
                    "Weird loottables '{}' for '{}', not a LivingEntity so should not have loot",
                    builders.keys().cloned().collect::<Vec<_>>().join(","),
                    entity_type.registry_key
                ));
            }
        }
        if !self.map.is_empty() {
            return Err(format!(
                "Created loot tables for entities not supported by datapack: {:?}",
                self.map.keys().collect::<Vec<_>>()
            ));
        }
        Ok(output)
    }
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

    #[test]
    fn should_smelt_loot_matches_java_any_of_sources() {
        assert_eq!(
            EntityLootSubProviderModel::new().should_smelt_loot(),
            LootConditionModel::AnyOf(vec![
                LootConditionModel::EntityOnFire {
                    target: LootEntityTargetModel::This,
                },
                LootConditionModel::DirectAttackerMainhandEnchantmentTag("minecraft:smelts_loot"),
            ])
        );
    }

    #[test]
    fn sheep_dispatch_pool_builds_color_specific_nested_tables() {
        let pool = EntityLootSubProviderModel::create_sheep_dispatch_pool(BTreeMap::from([
            (
                DyeColorModel::White,
                "minecraft:entities/sheep/white".to_string(),
            ),
            (
                DyeColorModel::Orange,
                "minecraft:entities/sheep/orange".to_string(),
            ),
            (
                DyeColorModel::Magenta,
                "minecraft:entities/sheep/magenta".to_string(),
            ),
            (
                DyeColorModel::LightBlue,
                "minecraft:entities/sheep/light_blue".to_string(),
            ),
        ]));
        let entries = match &pool.entries[0] {
            LootEntryModel::Alternatives(entries) => entries,
            other => panic!("expected alternatives entry, got {other:?}"),
        };
        assert_eq!(entries.len(), 4);
        assert_eq!(
            entries[0],
            LootEntryModel::NestedTable {
                table: "minecraft:entities/sheep/white".to_string(),
                conditions: vec![LootConditionModel::SheepColorAndHasWool {
                    color: DyeColorModel::White,
                }],
            }
        );
    }

    #[test]
    fn frog_damage_source_helpers_capture_type_and_variant_predicates() {
        let provider = EntityLootSubProviderModel::new();
        assert_eq!(
            provider.killed_by_frog(),
            LootConditionModel::DamageSourceEntityType {
                entity_type: "minecraft:frog",
            }
        );
        assert_eq!(
            provider.killed_by_frog_variant("minecraft:warm"),
            LootConditionModel::DamageSourceFrogVariant {
                variant: "minecraft:warm".to_string(),
            }
        );
    }

    #[test]
    fn add_default_uses_entity_default_table_and_rejects_missing_defaults() {
        let zombie = EntityTypeModel::living("minecraft:zombie");
        let marker = EntityTypeModel::non_living("minecraft:marker");
        let mut provider = EntityLootSubProviderModel::new();
        must_ok(provider.add_default(&zombie, LootTableBuilderModel("zombie")));
        assert!(provider.map["minecraft:zombie"].contains_key("minecraft:entities/zombie"));
        assert_eq!(
            must_err(provider.add_default(&marker, LootTableBuilderModel("marker"))),
            "Entity minecraft:marker has no loot table"
        );
    }

    #[test]
    fn generate_outputs_supported_tables_and_checks_required_defaults() {
        let zombie = EntityTypeModel::living("minecraft:zombie");
        let optional = EntityTypeModel::living("minecraft:creaking").required_disabled();
        let disabled = EntityTypeModel::living("minecraft:unused").allowed_disabled();
        let mut provider = EntityLootSubProviderModel::new();
        let output = must_ok(provider.generate(
            &[zombie.clone(), optional.clone(), disabled],
            |provider| {
                provider.add_default(&zombie, LootTableBuilderModel("zombie"))?;
                provider.add(
                    &optional,
                    "minecraft:entities/creaking/special",
                    LootTableBuilderModel("creaking"),
                );
                Ok(())
            },
        ));
        assert_eq!(
            output.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>(),
            vec![
                "minecraft:entities/zombie",
                "minecraft:entities/creaking/special",
            ]
        );

        let mut missing = EntityLootSubProviderModel::new();
        assert_eq!(
            must_err(missing.generate(std::slice::from_ref(&zombie), |_| Ok(()))),
            "Missing loottable 'minecraft:entities/zombie' for 'minecraft:zombie'"
        );
    }

    #[test]
    fn generate_rejects_weird_duplicate_and_unsupported_tables() {
        let zombie = EntityTypeModel::living("minecraft:zombie");
        let skeleton = EntityTypeModel::living("minecraft:skeleton").required_disabled();
        let marker = EntityTypeModel::non_living("minecraft:marker");
        let mut weird = EntityLootSubProviderModel::new();
        let weird_err = must_err(weird.generate(std::slice::from_ref(&marker), |provider| {
            provider.add(
                &marker,
                "minecraft:entities/marker",
                LootTableBuilderModel("marker"),
            );
            Ok(())
        }));
        assert_eq!(
            weird_err,
            "Weird loottables 'minecraft:entities/marker' for 'minecraft:marker', not a LivingEntity so should not have loot"
        );

        let mut duplicate = EntityLootSubProviderModel::new();
        let duplicate_err = must_err(duplicate.generate(&[zombie.clone(), skeleton], |provider| {
            provider.add_default(&zombie, LootTableBuilderModel("zombie"))?;
            provider.add(
                &EntityTypeModel::living("minecraft:skeleton"),
                "minecraft:entities/zombie",
                LootTableBuilderModel("duplicate"),
            );
            Ok(())
        }));
        assert_eq!(
            duplicate_err,
            "Duplicate loottable 'minecraft:entities/zombie' for 'minecraft:skeleton'"
        );

        let mut unsupported = EntityLootSubProviderModel::new();
        let unsupported_err = must_err(unsupported.generate(&[zombie], |provider| {
            provider.add(
                &EntityTypeModel::living("minecraft:zombie"),
                "minecraft:entities/zombie",
                LootTableBuilderModel("zombie"),
            );
            provider.add(
                &EntityTypeModel::living("minecraft:not_in_pack"),
                "minecraft:entities/not_in_pack",
                LootTableBuilderModel("extra"),
            );
            Ok(())
        }));
        assert!(unsupported_err
            .starts_with("Created loot tables for entities not supported by datapack: "));
    }
}
