use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
const NORMAL_LEAVES_SAPLING_CHANCES: [f32; 4] = [0.05, 0.0625, 0.083333336, 0.1];
const NORMAL_LEAVES_STICK_CHANCES: [f32; 5] = [0.02, 0.022222223, 0.025, 0.033333335, 0.1];
#[derive(Debug, Clone, PartialEq)]
struct LootTableBuilderModel {
    pools: Vec<LootPoolModel>,
}
impl LootTableBuilderModel {
    fn new() -> Self {
        Self { pools: Vec::new() }
    }
    fn pool(mut self, pool: LootPoolModel) -> Self {
        self.pools.push(pool);
        self
    }
}
#[derive(Debug, Clone, PartialEq)]
struct LootPoolModel {
    entries: Vec<LootEntryModel>,
    conditions: Vec<LootConditionModel>,
    functions: Vec<LootFunctionModel>,
}
impl LootPoolModel {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            conditions: Vec::new(),
            functions: Vec::new(),
        }
    }
    fn add(mut self, entry: LootEntryModel) -> Self {
        self.entries.push(entry);
        self
    }
    fn when(mut self, condition: LootConditionModel) -> Self {
        self.conditions.push(condition);
        self
    }
}
#[derive(Debug, Clone, PartialEq)]
struct LootEntryModel {
    item: String,
    conditions: Vec<LootConditionModel>,
    functions: Vec<LootFunctionModel>,
    otherwise: Option<Box<LootEntryModel>>,
}
impl LootEntryModel {
    fn item(item: impl Into<String>) -> Self {
        Self {
            item: item.into(),
            conditions: Vec::new(),
            functions: Vec::new(),
            otherwise: None,
        }
    }
    fn when(mut self, condition: LootConditionModel) -> Self {
        self.conditions.push(condition);
        self
    }
    fn apply(mut self, function: LootFunctionModel) -> Self {
        self.functions.push(function);
        self
    }
    fn otherwise(mut self, entry: LootEntryModel) -> Self {
        self.otherwise = Some(Box::new(entry));
        self
    }
}
#[derive(Debug, Clone, PartialEq)]
enum LootConditionModel {
    HasSilkTouch,
    HasShears,
    Inverted(Box<LootConditionModel>),
    AnyOf(Vec<LootConditionModel>),
    SurvivesExplosion,
    RandomChance(f32),
    BonusLevelFlatChance {
        enchantment: &'static str,
        chances: Vec<f32>,
    },
    BlockState {
        block: String,
        property: &'static str,
        value: String,
    },
    LocationBlock {
        block: String,
        property: &'static str,
        value: String,
        offset: (i32, i32, i32),
    },
}
#[derive(Debug, Clone, PartialEq)]
enum LootFunctionModel {
    ApplyExplosionDecay,
    SetCount(NumberProviderModel, bool),
    LimitCount {
        min: i32,
    },
    ApplyOreBonus {
        enchantment: &'static str,
    },
    ApplyUniformBonus {
        enchantment: &'static str,
        multiplier: i32,
    },
    ApplyBinomialBonus {
        enchantment: &'static str,
        probability: f32,
        extra: i32,
    },
    CopyComponents {
        source: &'static str,
        components: Vec<&'static str>,
    },
    CopyBlockState {
        block: String,
        properties: Vec<&'static str>,
    },
}
#[derive(Debug, Clone, PartialEq)]
enum NumberProviderModel {
    Constant(f32),
    Uniform(f32, f32),
    Binomial { n: i32, p: f32 },
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockModel {
    name: String,
    loot_table: Option<String>,
    enabled: bool,
    potted: Option<String>,
    segment_property: Option<&'static str>,
}
impl BlockModel {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            loot_table: Some(format!("minecraft:blocks/{name}")),
            enabled: true,
            potted: None,
            segment_property: None,
        }
    }
    fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
    fn without_loot_table(mut self) -> Self {
        self.loot_table = None;
        self
    }
    fn potted(mut self, item: &str) -> Self {
        self.potted = Some(item.to_string());
        self
    }
    fn segmentable(mut self, property: &'static str) -> Self {
        self.segment_property = Some(property);
        self
    }
}
#[derive(Debug, Clone, PartialEq)]
struct BlockLootSubProviderModel {
    explosion_resistant: BTreeSet<String>,
    generated: BTreeMap<String, LootTableBuilderModel>,
}
impl BlockLootSubProviderModel {
    fn new(explosion_resistant: impl IntoIterator<Item = &'static str>) -> Self {
        Self {
            explosion_resistant: explosion_resistant
                .into_iter()
                .map(str::to_string)
                .collect(),
            generated: BTreeMap::new(),
        }
    }
    fn has_silk_touch(&self) -> LootConditionModel {
        LootConditionModel::HasSilkTouch
    }
    fn does_not_have_silk_touch(&self) -> LootConditionModel {
        LootConditionModel::Inverted(Box::new(self.has_silk_touch()))
    }
    fn has_shears(&self) -> LootConditionModel {
        LootConditionModel::HasShears
    }
    fn has_shears_or_silk_touch(&self) -> LootConditionModel {
        LootConditionModel::AnyOf(vec![self.has_shears(), self.has_silk_touch()])
    }
    fn does_not_have_shears_or_silk_touch(&self) -> LootConditionModel {
        LootConditionModel::Inverted(Box::new(self.has_shears_or_silk_touch()))
    }
    fn apply_explosion_decay(&self, item: &str, entry: LootEntryModel) -> LootEntryModel {
        if self.explosion_resistant.contains(item) {
            entry
        } else {
            entry.apply(LootFunctionModel::ApplyExplosionDecay)
        }
    }
    fn apply_explosion_condition_pool(&self, item: &str, pool: LootPoolModel) -> LootPoolModel {
        if self.explosion_resistant.contains(item) {
            pool
        } else {
            pool.when(LootConditionModel::SurvivesExplosion)
        }
    }
    fn apply_explosion_condition_entry(&self, item: &str, entry: LootEntryModel) -> LootEntryModel {
        if self.explosion_resistant.contains(item) {
            entry
        } else {
            entry.when(LootConditionModel::SurvivesExplosion)
        }
    }
    fn create_single_item_table(&self, drop: &str) -> LootTableBuilderModel {
        LootTableBuilderModel::new().pool(self.apply_explosion_condition_pool(
            drop,
            LootPoolModel::new().add(LootEntryModel::item(drop)),
        ))
    }
    fn create_self_drop_dispatch_table(
        &self,
        original: &str,
        condition: LootConditionModel,
        entry: LootEntryModel,
    ) -> LootTableBuilderModel {
        LootTableBuilderModel::new().pool(
            LootPoolModel::new().add(
                LootEntryModel::item(original)
                    .when(condition)
                    .otherwise(entry),
            ),
        )
    }
    fn create_silk_touch_dispatch_table(
        &self,
        original: &str,
        entry: LootEntryModel,
    ) -> LootTableBuilderModel {
        self.create_self_drop_dispatch_table(original, self.has_silk_touch(), entry)
    }
    fn create_shears_dispatch_table(
        &self,
        original: &str,
        entry: LootEntryModel,
    ) -> LootTableBuilderModel {
        self.create_self_drop_dispatch_table(original, self.has_shears(), entry)
    }
    fn create_silk_touch_or_shears_dispatch_table(
        &self,
        original: &str,
        entry: LootEntryModel,
    ) -> LootTableBuilderModel {
        self.create_self_drop_dispatch_table(original, self.has_shears_or_silk_touch(), entry)
    }
    fn create_single_item_table_with_count(
        &self,
        drop: &str,
        count: NumberProviderModel,
    ) -> LootTableBuilderModel {
        LootTableBuilderModel::new().pool(LootPoolModel::new().add(self.apply_explosion_decay(
            drop,
            LootEntryModel::item(drop).apply(LootFunctionModel::SetCount(count, false)),
        )))
    }
    fn create_single_item_table_with_silk_touch(
        &self,
        original: &str,
        drop: &str,
        count: Option<NumberProviderModel>,
    ) -> LootTableBuilderModel {
        let mut entry = LootEntryModel::item(drop);
        if let Some(count) = count {
            entry = entry.apply(LootFunctionModel::SetCount(count, false));
            entry = self.apply_explosion_decay(original, entry);
        } else {
            entry = self.apply_explosion_condition_entry(original, entry);
        }
        self.create_silk_touch_dispatch_table(original, entry)
    }
    fn create_silk_touch_only_table(&self, drop: &str) -> LootTableBuilderModel {
        LootTableBuilderModel::new().pool(
            LootPoolModel::new()
                .when(self.has_silk_touch())
                .add(LootEntryModel::item(drop)),
        )
    }
    fn create_pot_flower_item_table(&self, flower: &str) -> LootTableBuilderModel {
        LootTableBuilderModel::new()
            .pool(self.apply_explosion_condition_pool(
                "minecraft:flower_pot",
                LootPoolModel::new().add(LootEntryModel::item("minecraft:flower_pot")),
            ))
            .pool(self.apply_explosion_condition_pool(
                flower,
                LootPoolModel::new().add(LootEntryModel::item(flower)),
            ))
    }
    fn create_slab_item_table(&self, slab: &str) -> LootTableBuilderModel {
        LootTableBuilderModel::new().pool(
            LootPoolModel::new().add(
                self.apply_explosion_decay(
                    slab,
                    LootEntryModel::item(slab)
                        .apply(LootFunctionModel::SetCount(
                            NumberProviderModel::Constant(2.0),
                            false,
                        ))
                        .when(block_state(slab, "type", "double")),
                ),
            ),
        )
    }
    fn create_single_prop_condition_table(
        &self,
        drop: &str,
        property: &'static str,
        value: &str,
    ) -> LootTableBuilderModel {
        LootTableBuilderModel::new().pool(
            self.apply_explosion_condition_pool(
                drop,
                LootPoolModel::new()
                    .add(LootEntryModel::item(drop).when(block_state(drop, property, value))),
            ),
        )
    }
    fn create_nameable_block_entity_table(&self, drop: &str) -> LootTableBuilderModel {
        self.copy_components_table(drop, vec!["minecraft:custom_name"])
    }
    fn create_shulker_box_drop(&self, block: &str) -> LootTableBuilderModel {
        self.copy_components_table(
            block,
            vec![
                "minecraft:custom_name",
                "minecraft:container",
                "minecraft:lock",
                "minecraft:container_loot",
            ],
        )
    }
    fn create_banner_drop(&self, block: &str) -> LootTableBuilderModel {
        self.copy_components_table(
            block,
            vec![
                "minecraft:custom_name",
                "minecraft:item_name",
                "minecraft:tooltip_display",
                "minecraft:banner_patterns",
                "minecraft:rarity",
            ],
        )
    }
    fn copy_components_table(
        &self,
        block: &str,
        components: Vec<&'static str>,
    ) -> LootTableBuilderModel {
        LootTableBuilderModel::new().pool(self.apply_explosion_condition_pool(
            block,
            LootPoolModel::new().add(LootEntryModel::item(block).apply(
                LootFunctionModel::CopyComponents {
                    source: "block_entity",
                    components,
                },
            )),
        ))
    }
    fn create_bee_nest_drop(&self, block: &str) -> LootTableBuilderModel {
        LootTableBuilderModel::new().pool(
            LootPoolModel::new().when(self.has_silk_touch()).add(
                LootEntryModel::item(block)
                    .apply(LootFunctionModel::CopyComponents {
                        source: "block_entity",
                        components: vec!["minecraft:bees"],
                    })
                    .apply(LootFunctionModel::CopyBlockState {
                        block: block.to_string(),
                        properties: vec!["honey_level"],
                    }),
            ),
        )
    }
    fn create_beehive_drop(&self, block: &str) -> LootTableBuilderModel {
        LootTableBuilderModel::new().pool(
            LootPoolModel::new().add(
                LootEntryModel::item(block)
                    .when(self.has_silk_touch())
                    .apply(LootFunctionModel::CopyComponents {
                        source: "block_entity",
                        components: vec!["minecraft:bees"],
                    })
                    .apply(LootFunctionModel::CopyBlockState {
                        block: block.to_string(),
                        properties: vec!["honey_level"],
                    })
                    .otherwise(LootEntryModel::item(block)),
            ),
        )
    }
    fn create_ore_drop(&self, original: &str, drop: &str) -> LootTableBuilderModel {
        self.create_silk_touch_dispatch_table(
            original,
            self.apply_explosion_decay(
                original,
                LootEntryModel::item(drop).apply(LootFunctionModel::ApplyOreBonus {
                    enchantment: "minecraft:fortune",
                }),
            ),
        )
    }
    fn create_copper_ore_drops(&self, block: &str) -> LootTableBuilderModel {
        self.create_silk_touch_dispatch_table(
            block,
            self.apply_explosion_decay(
                block,
                LootEntryModel::item("minecraft:raw_copper")
                    .apply(LootFunctionModel::SetCount(
                        NumberProviderModel::Uniform(2.0, 5.0),
                        false,
                    ))
                    .apply(LootFunctionModel::ApplyOreBonus {
                        enchantment: "minecraft:fortune",
                    }),
            ),
        )
    }
    fn create_redstone_ore_drops(&self, block: &str) -> LootTableBuilderModel {
        self.create_silk_touch_dispatch_table(
            block,
            self.apply_explosion_decay(
                block,
                LootEntryModel::item("minecraft:redstone")
                    .apply(LootFunctionModel::SetCount(
                        NumberProviderModel::Uniform(4.0, 5.0),
                        false,
                    ))
                    .apply(LootFunctionModel::ApplyUniformBonus {
                        enchantment: "minecraft:fortune",
                        multiplier: 1,
                    }),
            ),
        )
    }
    fn create_mushroom_block_drop(&self, original: &str, drop: &str) -> LootTableBuilderModel {
        self.create_silk_touch_dispatch_table(
            original,
            self.apply_explosion_decay(
                original,
                LootEntryModel::item(drop)
                    .apply(LootFunctionModel::SetCount(
                        NumberProviderModel::Uniform(-6.0, 2.0),
                        false,
                    ))
                    .apply(LootFunctionModel::LimitCount { min: 0 }),
            ),
        )
    }
    fn create_grass_drops(&self, original: &str) -> LootTableBuilderModel {
        self.create_shears_dispatch_table(
            original,
            self.apply_explosion_decay(
                original,
                LootEntryModel::item("minecraft:wheat_seeds")
                    .when(LootConditionModel::RandomChance(0.125))
                    .apply(LootFunctionModel::ApplyUniformBonus {
                        enchantment: "minecraft:fortune",
                        multiplier: 2,
                    }),
            ),
        )
    }
    fn create_stem_drops(&self, block: &str, drop: &str) -> LootTableBuilderModel {
        let entry = (0..=7).fold(LootEntryModel::item(drop), |entry, age| {
            entry
                .apply(LootFunctionModel::SetCount(
                    NumberProviderModel::Binomial {
                        n: 3,
                        p: (age + 1) as f32 / 15.0,
                    },
                    false,
                ))
                .when(block_state(block, "age", &age.to_string()))
        });
        LootTableBuilderModel::new()
            .pool(LootPoolModel::new().add(self.apply_explosion_decay(block, entry)))
    }
    fn create_attached_stem_drops(&self, block: &str, drop: &str) -> LootTableBuilderModel {
        LootTableBuilderModel::new().pool(LootPoolModel::new().add(self.apply_explosion_decay(
            block,
            LootEntryModel::item(drop).apply(LootFunctionModel::SetCount(
                NumberProviderModel::Binomial {
                    n: 3,
                    p: 0.53333336,
                },
                false,
            )),
        )))
    }
    fn create_leaves_drops(
        &self,
        original: &str,
        sapling: &str,
        sapling_chances: Vec<f32>,
    ) -> LootTableBuilderModel {
        self.create_silk_touch_or_shears_dispatch_table(
            original,
            self.apply_explosion_condition_entry(
                original,
                LootEntryModel::item(sapling).when(LootConditionModel::BonusLevelFlatChance {
                    enchantment: "minecraft:fortune",
                    chances: sapling_chances,
                }),
            ),
        )
        .pool(
            LootPoolModel::new()
                .when(self.does_not_have_shears_or_silk_touch())
                .add(
                    self.apply_explosion_decay(
                        original,
                        LootEntryModel::item("minecraft:stick")
                            .apply(LootFunctionModel::SetCount(
                                NumberProviderModel::Uniform(1.0, 2.0),
                                false,
                            ))
                            .when(LootConditionModel::BonusLevelFlatChance {
                                enchantment: "minecraft:fortune",
                                chances: NORMAL_LEAVES_STICK_CHANCES.to_vec(),
                            }),
                    ),
                ),
        )
    }
    fn create_oak_leaves_drops(
        &self,
        original: &str,
        sapling: &str,
        sapling_chances: Vec<f32>,
    ) -> LootTableBuilderModel {
        self.create_leaves_drops(original, sapling, sapling_chances)
            .pool(
                LootPoolModel::new()
                    .when(self.does_not_have_shears_or_silk_touch())
                    .add(LootEntryModel::item("minecraft:apple").when(
                        LootConditionModel::BonusLevelFlatChance {
                            enchantment: "minecraft:fortune",
                            chances: vec![0.005, 0.0055555557, 0.00625, 0.008333334, 0.025],
                        },
                    )),
            )
    }
    fn create_crop_drops(
        &self,
        crop_drop: &str,
        seed_drop: &str,
        is_max_age: LootConditionModel,
    ) -> LootTableBuilderModel {
        LootTableBuilderModel::new()
            .pool(
                LootPoolModel::new().add(
                    LootEntryModel::item(crop_drop)
                        .when(is_max_age.clone())
                        .otherwise(LootEntryModel::item(seed_drop)),
                ),
            )
            .pool(
                LootPoolModel::new()
                    .when(is_max_age)
                    .add(LootEntryModel::item(seed_drop).apply(
                        LootFunctionModel::ApplyBinomialBonus {
                            enchantment: "minecraft:fortune",
                            probability: 0.5714286,
                            extra: 3,
                        },
                    )),
            )
    }
    fn create_double_plant_with_seed_drops(
        &self,
        block: &str,
        drop: &str,
    ) -> LootTableBuilderModel {
        let drop_entry = LootEntryModel::item(drop)
            .apply(LootFunctionModel::SetCount(
                NumberProviderModel::Constant(2.0),
                false,
            ))
            .when(self.has_shears())
            .otherwise(
                LootEntryModel::item("minecraft:wheat_seeds")
                    .when(LootConditionModel::RandomChance(0.125)),
            );
        LootTableBuilderModel::new()
            .pool(
                LootPoolModel::new()
                    .add(drop_entry.clone())
                    .when(block_state(block, "half", "lower"))
                    .when(location_block(block, "half", "upper", (0, 1, 0))),
            )
            .pool(
                LootPoolModel::new()
                    .add(drop_entry)
                    .when(block_state(block, "half", "upper"))
                    .when(location_block(block, "half", "lower", (0, -1, 0))),
            )
    }
    fn create_candle_drops(&self, block: &str) -> LootTableBuilderModel {
        let entry = [2, 3, 4]
            .into_iter()
            .fold(LootEntryModel::item(block), |entry, count| {
                entry
                    .apply(LootFunctionModel::SetCount(
                        NumberProviderModel::Constant(count as f32),
                        false,
                    ))
                    .when(block_state(block, "candles", &count.to_string()))
            });
        LootTableBuilderModel::new()
            .pool(LootPoolModel::new().add(self.apply_explosion_decay(block, entry)))
    }
    fn create_segmented_block_drops(&self, block: &BlockModel) -> LootTableBuilderModel {
        let Some(property) = block.segment_property else {
            return Self::no_drop();
        };
        let entry = (1..=4).fold(LootEntryModel::item(&block.name), |entry, count| {
            entry
                .apply(LootFunctionModel::SetCount(
                    NumberProviderModel::Constant(count as f32),
                    false,
                ))
                .when(block_state(&block.name, property, &count.to_string()))
        });
        LootTableBuilderModel::new()
            .pool(LootPoolModel::new().add(self.apply_explosion_decay(&block.name, entry)))
    }
    fn create_multiface_block_drops(
        &self,
        block: &str,
        condition: Option<LootConditionModel>,
    ) -> LootTableBuilderModel {
        let mut entry = LootEntryModel::item(block);
        if let Some(condition) = condition {
            entry = entry.when(condition);
        }
        for dir in ["down", "up", "north", "south", "west", "east"] {
            entry = entry
                .apply(LootFunctionModel::SetCount(
                    NumberProviderModel::Constant(1.0),
                    true,
                ))
                .when(block_state(block, dir, "true"));
        }
        entry = entry.apply(LootFunctionModel::SetCount(
            NumberProviderModel::Constant(-1.0),
            true,
        ));
        LootTableBuilderModel::new()
            .pool(LootPoolModel::new().add(self.apply_explosion_decay(block, entry)))
    }
    fn create_door_table(&self, block: &str) -> LootTableBuilderModel {
        self.create_single_prop_condition_table(block, "half", "lower")
    }
    fn drop_potted_contents(&mut self, potted: &BlockModel) -> Result<(), String> {
        let Some(flower) = &potted.potted else {
            return Err(format!("Block {} is not a flower pot", potted.name));
        };
        self.add(potted, self.create_pot_flower_item_table(flower))
    }
    fn other_when_silk_touch(&mut self, block: &BlockModel, other: &str) -> Result<(), String> {
        self.add(block, self.create_silk_touch_only_table(other))
    }
    fn drop_other(&mut self, block: &BlockModel, drop: &str) -> Result<(), String> {
        self.add(block, self.create_single_item_table(drop))
    }
    fn drop_when_silk_touch(&mut self, block: &BlockModel) -> Result<(), String> {
        self.other_when_silk_touch(block, &block.name)
    }
    fn drop_self(&mut self, block: &BlockModel) -> Result<(), String> {
        self.drop_other(block, &block.name)
    }
    fn add(&mut self, block: &BlockModel, builder: LootTableBuilderModel) -> Result<(), String> {
        let Some(loot_table) = &block.loot_table else {
            return Err(format!("Block {} does not have loot table", block.name));
        };
        self.generated.insert(loot_table.clone(), builder);
        Ok(())
    }
    fn generate(
        &mut self,
        blocks: &[BlockModel],
        generated_by_subclass: impl FnOnce(&mut Self) -> Result<(), String>,
    ) -> Result<Vec<(String, LootTableBuilderModel)>, String> {
        generated_by_subclass(self)?;
        let mut seen = BTreeSet::new();
        let mut output = Vec::new();
        for block in blocks.iter().filter(|block| block.enabled) {
            if let Some(loot_table) = &block.loot_table {
                if seen.insert(loot_table.clone()) {
                    let Some(builder) = self.generated.remove(loot_table) else {
                        return Err(format!(
                            "Missing loottable '{loot_table}' for '{}'",
                            block.name
                        ));
                    };
                    output.push((loot_table.clone(), builder));
                }
            }
        }
        if !self.generated.is_empty() {
            return Err(format!(
                "Created block loot tables for non-blocks: {:?}",
                self.generated.keys().collect::<Vec<_>>()
            ));
        }
        Ok(output)
    }
    fn no_drop() -> LootTableBuilderModel {
        LootTableBuilderModel::new()
    }
}
fn block_state(block: &str, property: &'static str, value: &str) -> LootConditionModel {
    LootConditionModel::BlockState {
        block: block.to_string(),
        property,
        value: value.to_string(),
    }
}
fn location_block(
    block: &str,
    property: &'static str,
    value: &str,
    offset: (i32, i32, i32),
) -> LootConditionModel {
    LootConditionModel::LocationBlock {
        block: block.to_string(),
        property,
        value: value.to_string(),
        offset,
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
    fn explosion_helpers_match_resistant_and_non_resistant_java_branches() {
        let provider = BlockLootSubProviderModel::new(["minecraft:obsidian"]);
        let fragile = provider.create_single_item_table("minecraft:stone");
        let resistant = provider.create_single_item_table("minecraft:obsidian");
        assert_eq!(
            fragile.pools[0].conditions,
            vec![LootConditionModel::SurvivesExplosion]
        );
        assert!(resistant.pools[0].conditions.is_empty());
        assert_eq!(
            provider
                .create_single_item_table_with_count(
                    "minecraft:coal",
                    NumberProviderModel::Uniform(1.0, 3.0)
                )
                .pools[0]
                .entries[0]
                .functions,
            vec![
                LootFunctionModel::SetCount(NumberProviderModel::Uniform(1.0, 3.0), false),
                LootFunctionModel::ApplyExplosionDecay,
            ]
        );
    }
    #[test]
    fn dispatch_tables_use_silk_touch_shears_and_inverted_conditions() {
        let provider = BlockLootSubProviderModel::new([]);
        let silk = provider.create_silk_touch_dispatch_table(
            "minecraft:diamond_ore",
            LootEntryModel::item("minecraft:diamond"),
        );
        assert_eq!(
            silk.pools[0].entries[0].conditions,
            vec![provider.has_silk_touch()]
        );
        assert_eq!(
            silk.pools[0].entries[0]
                .otherwise
                .as_ref()
                .map(|entry| &entry.item),
            Some(&"minecraft:diamond".to_string())
        );
        assert_eq!(
            provider.does_not_have_silk_touch(),
            LootConditionModel::Inverted(Box::new(LootConditionModel::HasSilkTouch))
        );
        assert_eq!(
            provider
                .create_shears_dispatch_table(
                    "minecraft:vine",
                    LootEntryModel::item("minecraft:string")
                )
                .pools[0]
                .entries[0]
                .conditions,
            vec![LootConditionModel::HasShears]
        );
    }
    #[test]
    fn block_entity_helpers_copy_exact_java_components_and_state() {
        let provider = BlockLootSubProviderModel::new([]);
        assert_eq!(
            provider
                .create_nameable_block_entity_table("minecraft:chest")
                .pools[0]
                .entries[0]
                .functions,
            vec![LootFunctionModel::CopyComponents {
                source: "block_entity",
                components: vec!["minecraft:custom_name"],
            }]
        );
        assert_eq!(
            provider
                .create_shulker_box_drop("minecraft:shulker_box")
                .pools[0]
                .entries[0]
                .functions,
            vec![LootFunctionModel::CopyComponents {
                source: "block_entity",
                components: vec![
                    "minecraft:custom_name",
                    "minecraft:container",
                    "minecraft:lock",
                    "minecraft:container_loot"
                ],
            }]
        );
        assert_eq!(
            provider.create_banner_drop("minecraft:white_banner").pools[0].entries[0].functions[0],
            LootFunctionModel::CopyComponents {
                source: "block_entity",
                components: vec![
                    "minecraft:custom_name",
                    "minecraft:item_name",
                    "minecraft:tooltip_display",
                    "minecraft:banner_patterns",
                    "minecraft:rarity",
                ],
            }
        );
        assert_eq!(
            provider.create_bee_nest_drop("minecraft:bee_nest").pools[0].entries[0].functions,
            vec![
                LootFunctionModel::CopyComponents {
                    source: "block_entity",
                    components: vec!["minecraft:bees"],
                },
                LootFunctionModel::CopyBlockState {
                    block: "minecraft:bee_nest".to_string(),
                    properties: vec!["honey_level"],
                },
            ]
        );
        assert!(
            provider.create_beehive_drop("minecraft:beehive").pools[0].entries[0]
                .otherwise
                .is_some()
        );
    }
    #[test]
    fn ore_helpers_preserve_java_count_ranges_and_fortune_functions() {
        let provider = BlockLootSubProviderModel::new([]);
        assert_eq!(
            provider
                .create_copper_ore_drops("minecraft:copper_ore")
                .pools[0]
                .entries[0]
                .otherwise
                .as_ref()
                .map(|entry| entry.functions.clone()),
            Some(vec![
                LootFunctionModel::SetCount(NumberProviderModel::Uniform(2.0, 5.0), false),
                LootFunctionModel::ApplyOreBonus {
                    enchantment: "minecraft:fortune",
                },
                LootFunctionModel::ApplyExplosionDecay,
            ])
        );
        assert_eq!(
            provider
                .create_redstone_ore_drops("minecraft:redstone_ore")
                .pools[0]
                .entries[0]
                .otherwise
                .as_ref()
                .map(|entry| entry.functions.clone()),
            Some(vec![
                LootFunctionModel::SetCount(NumberProviderModel::Uniform(4.0, 5.0), false),
                LootFunctionModel::ApplyUniformBonus {
                    enchantment: "minecraft:fortune",
                    multiplier: 1,
                },
                LootFunctionModel::ApplyExplosionDecay,
            ])
        );
        assert_eq!(
            provider
                .create_ore_drop("minecraft:diamond_ore", "minecraft:diamond")
                .pools[0]
                .entries[0]
                .otherwise
                .as_ref()
                .map(|entry| entry.functions.clone()),
            Some(vec![
                LootFunctionModel::ApplyOreBonus {
                    enchantment: "minecraft:fortune",
                },
                LootFunctionModel::ApplyExplosionDecay,
            ])
        );
    }
    #[test]
    fn plant_leaf_and_crop_helpers_preserve_java_constants_and_pool_layouts() {
        let provider = BlockLootSubProviderModel::new([]);
        assert_eq!(
            NORMAL_LEAVES_SAPLING_CHANCES,
            [0.05, 0.0625, 0.083333336, 0.1]
        );
        assert_eq!(
            NORMAL_LEAVES_STICK_CHANCES,
            [0.02, 0.022222223, 0.025, 0.033333335, 0.1]
        );
        let oak = provider.create_oak_leaves_drops(
            "minecraft:oak_leaves",
            "minecraft:oak_sapling",
            NORMAL_LEAVES_SAPLING_CHANCES.to_vec(),
        );
        assert_eq!(oak.pools.len(), 3);
        assert_eq!(
            oak.pools[1].conditions,
            vec![provider.does_not_have_shears_or_silk_touch()]
        );
        assert_eq!(oak.pools[2].entries[0].item, "minecraft:apple");
        let crop = provider.create_crop_drops(
            "minecraft:wheat",
            "minecraft:wheat_seeds",
            block_state("minecraft:wheat", "age", "7"),
        );
        assert_eq!(crop.pools.len(), 2);
        assert_eq!(
            crop.pools[1].entries[0].functions,
            vec![LootFunctionModel::ApplyBinomialBonus {
                enchantment: "minecraft:fortune",
                probability: 0.5714286,
                extra: 3,
            }]
        );
        assert_eq!(
            provider
                .create_mushroom_block_drop(
                    "minecraft:brown_mushroom_block",
                    "minecraft:brown_mushroom"
                )
                .pools[0]
                .entries[0]
                .otherwise
                .as_ref()
                .map(|entry| entry.functions.clone()),
            Some(vec![
                LootFunctionModel::SetCount(NumberProviderModel::Uniform(-6.0, 2.0), false),
                LootFunctionModel::LimitCount { min: 0 },
                LootFunctionModel::ApplyExplosionDecay,
            ])
        );
        assert_eq!(
            provider.create_grass_drops("minecraft:grass").pools[0].entries[0]
                .otherwise
                .as_ref()
                .map(|entry| entry.conditions.clone()),
            Some(vec![LootConditionModel::RandomChance(0.125)])
        );
        assert_eq!(
            provider
                .create_stem_drops("minecraft:melon_stem", "minecraft:melon_seeds")
                .pools[0]
                .entries[0]
                .functions
                .first(),
            Some(&LootFunctionModel::SetCount(
                NumberProviderModel::Binomial {
                    n: 3,
                    p: 1.0 / 15.0,
                },
                false,
            ))
        );
        assert_eq!(
            provider
                .create_attached_stem_drops(
                    "minecraft:attached_melon_stem",
                    "minecraft:melon_seeds"
                )
                .pools[0]
                .entries[0]
                .functions[0],
            LootFunctionModel::SetCount(
                NumberProviderModel::Binomial {
                    n: 3,
                    p: 0.53333336,
                },
                false,
            )
        );
    }
    #[test]
    fn shape_helpers_capture_java_state_properties_and_counts() {
        let provider = BlockLootSubProviderModel::new([]);
        assert_eq!(
            provider
                .create_slab_item_table("minecraft:stone_slab")
                .pools[0]
                .entries[0]
                .conditions,
            vec![block_state("minecraft:stone_slab", "type", "double")]
        );
        assert_eq!(
            provider.create_door_table("minecraft:oak_door").pools[0].entries[0].conditions,
            vec![block_state("minecraft:oak_door", "half", "lower")]
        );
        let candle = provider.create_candle_drops("minecraft:candle");
        assert_eq!(candle.pools[0].entries[0].conditions.len(), 3);
        assert_eq!(
            provider
                .create_single_item_table_with_silk_touch(
                    "minecraft:amethyst_cluster",
                    "minecraft:amethyst_shard",
                    Some(NumberProviderModel::Constant(4.0))
                )
                .pools[0]
                .entries[0]
                .otherwise
                .as_ref()
                .map(|entry| entry.functions.clone()),
            Some(vec![
                LootFunctionModel::SetCount(NumberProviderModel::Constant(4.0), false),
                LootFunctionModel::ApplyExplosionDecay,
            ])
        );
        let segmented = provider.create_segmented_block_drops(
            &BlockModel::new("minecraft:pink_petals").segmentable("flower_amount"),
        );
        assert_eq!(segmented.pools[0].entries[0].conditions.len(), 4);
        assert!(BlockLootSubProviderModel::no_drop().pools.is_empty());
        assert!(provider
            .create_segmented_block_drops(&BlockModel::new("minecraft:stone"))
            .pools
            .is_empty());
    }
    #[test]
    fn multiface_and_double_plant_helpers_match_java_direction_and_neighbor_checks() {
        let provider = BlockLootSubProviderModel::new([]);
        let multiface = provider
            .create_multiface_block_drops("minecraft:glow_lichen", Some(provider.has_shears()));
        assert_eq!(multiface.pools[0].entries[0].conditions.len(), 7);
        assert_eq!(
            multiface.pools[0].entries[0]
                .functions
                .iter()
                .rev()
                .take(2)
                .cloned()
                .collect::<Vec<_>>(),
            vec![
                LootFunctionModel::ApplyExplosionDecay,
                LootFunctionModel::SetCount(NumberProviderModel::Constant(-1.0), true),
            ]
        );
        let tall = provider
            .create_double_plant_with_seed_drops("minecraft:tall_grass", "minecraft:tall_grass");
        assert_eq!(tall.pools.len(), 2);
        assert_eq!(
            tall.pools[0].conditions,
            vec![
                block_state("minecraft:tall_grass", "half", "lower"),
                location_block("minecraft:tall_grass", "half", "upper", (0, 1, 0)),
            ]
        );
        assert_eq!(
            tall.pools[1].conditions,
            vec![
                block_state("minecraft:tall_grass", "half", "upper"),
                location_block("minecraft:tall_grass", "half", "lower", (0, -1, 0)),
            ]
        );
    }
    #[test]
    fn add_and_generate_match_java_block_table_validation() {
        let stone = BlockModel::new("stone");
        let dirt = BlockModel::new("dirt");
        let disabled = BlockModel::new("disabled").disabled();
        let mut provider = BlockLootSubProviderModel::new([]);
        let output = must_ok(provider.generate(
            &[stone.clone(), dirt.clone(), disabled],
            |provider| {
                provider.drop_self(&stone)?;
                provider.drop_other(&dirt, "minecraft:coarse_dirt")?;
                Ok(())
            },
        ));
        assert_eq!(
            output.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>(),
            vec!["minecraft:blocks/stone", "minecraft:blocks/dirt"]
        );
        let mut missing = BlockLootSubProviderModel::new([]);
        let err = must_err(missing.generate(std::slice::from_ref(&stone), |_| Ok(())));
        assert_eq!(
            err,
            "Missing loottable 'minecraft:blocks/stone' for 'stone'"
        );
        let mut extra = BlockLootSubProviderModel::new([]);
        let err = must_err(extra.generate(std::slice::from_ref(&stone), |provider| {
            provider.drop_self(&stone)?;
            provider.drop_self(&BlockModel::new("not_in_registry"))?;
            Ok(())
        }));
        assert!(err.starts_with("Created block loot tables for non-blocks: "));
        let mut no_table = BlockLootSubProviderModel::new([]);
        assert_eq!(
            must_err(no_table.drop_self(&BlockModel::new("air").without_loot_table())),
            "Block air does not have loot table"
        );
    }
    #[test]
    fn potted_and_silk_touch_shortcuts_delegate_to_java_table_helpers() {
        let mut provider = BlockLootSubProviderModel::new([]);
        let potted = BlockModel::new("potted_dandelion").potted("minecraft:dandelion");
        must_ok(provider.drop_potted_contents(&potted));
        assert_eq!(
            provider.generated["minecraft:blocks/potted_dandelion"]
                .pools
                .len(),
            2
        );
        let mut silk = BlockLootSubProviderModel::new([]);
        let glass = BlockModel::new("glass");
        must_ok(silk.drop_when_silk_touch(&glass));
        assert_eq!(
            silk.generated["minecraft:blocks/glass"].pools[0].conditions,
            vec![LootConditionModel::HasSilkTouch]
        );
    }
}
