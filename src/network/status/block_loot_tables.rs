use super::*;

/// Builds the block loot table for `block_name`, matching the JSON loot tables
/// from data/minecraft/loot_table/blocks/ in the Java source.
pub fn block_loot_table(block_name: &str) -> Option<LootTable> {
    let key = block_name.strip_prefix("minecraft:").unwrap_or(block_name);
    let random_sequence = format!("minecraft:blocks/{key}");

    terrain_block_loot_table(key, &random_sequence)
        .or_else(|| ore_block_loot_table(key, &random_sequence))
        .or_else(|| wood_block_loot_table(key, &random_sequence))
        .or_else(|| leaves_block_loot_table(key, &random_sequence))
        .or_else(|| plant_block_loot_table(key, &random_sequence))
        .or_else(|| crop_block_loot_table(key, &random_sequence))
        .or_else(|| special_block_loot_table(key, &random_sequence))
}

fn terrain_block_loot_table(key: &str, random_sequence: &str) -> Option<LootTable> {
    Some(match key {
        // These drop cobblestone/dirt instead of themselves (silk touch not implemented).
        "stone" => self_drop_table("cobblestone", random_sequence),
        "grass_block" | "mycelium" | "podzol" | "dirt_path" | "farmland" => {
            self_drop_table("dirt", random_sequence)
        }
        "clay" => clay_table(random_sequence),
        "gravel" => gravel_table(random_sequence),
        "granite" | "polished_granite" | "diorite" | "polished_diorite" | "andesite"
        | "polished_andesite" | "deepslate" | "cobbled_deepslate" | "cobblestone" | "dirt"
        | "coarse_dirt" | "rooted_dirt" | "mud" | "sand" | "red_sand" | "sandstone"
        | "chiseled_sandstone" | "cut_sandstone" | "smooth_sandstone" => {
            self_drop_table(key, random_sequence)
        }
        _ => return None,
    })
}

fn ore_block_loot_table(key: &str, random_sequence: &str) -> Option<LootTable> {
    Some(match key {
        // Silk touch (ore block self-drop) not yet implemented; always drops raw product.
        // Fortune bonuses not yet implemented; counts are base values.
        "coal_ore" | "deepslate_coal_ore" => ore_drop_1("coal", random_sequence),
        "iron_ore" | "deepslate_iron_ore" => ore_drop_1("raw_iron", random_sequence),
        "gold_ore" | "deepslate_gold_ore" => ore_drop_1("raw_gold", random_sequence),
        "copper_ore" | "deepslate_copper_ore" => {
            ore_drop_count("raw_copper", 2.0, 5.0, random_sequence)
        }
        "redstone_ore"
        | "lit_redstone_ore"
        | "deepslate_redstone_ore"
        | "lit_deepslate_redstone_ore" => ore_drop_count("redstone", 4.0, 5.0, random_sequence),
        "emerald_ore" | "deepslate_emerald_ore" => ore_drop_1("emerald", random_sequence),
        "lapis_ore" | "deepslate_lapis_ore" => {
            ore_drop_count("lapis_lazuli", 4.0, 9.0, random_sequence)
        }
        "diamond_ore" | "deepslate_diamond_ore" => ore_drop_1("diamond", random_sequence),
        "nether_quartz_ore" => ore_drop_1("quartz", random_sequence),
        "nether_gold_ore" => ore_drop_count("gold_nugget", 2.0, 6.0, random_sequence),
        _ => return None,
    })
}

fn wood_block_loot_table(key: &str, random_sequence: &str) -> Option<LootTable> {
    Some(match key {
        "oak_log"
        | "spruce_log"
        | "birch_log"
        | "jungle_log"
        | "acacia_log"
        | "dark_oak_log"
        | "stripped_oak_log"
        | "stripped_spruce_log"
        | "stripped_birch_log"
        | "stripped_jungle_log"
        | "stripped_acacia_log"
        | "stripped_dark_oak_log"
        | "oak_wood"
        | "spruce_wood"
        | "birch_wood"
        | "jungle_wood"
        | "acacia_wood"
        | "dark_oak_wood"
        | "stripped_oak_wood"
        | "stripped_spruce_wood"
        | "stripped_birch_wood"
        | "stripped_jungle_wood"
        | "stripped_acacia_wood"
        | "stripped_dark_oak_wood"
        | "oak_planks"
        | "spruce_planks"
        | "birch_planks"
        | "jungle_planks"
        | "acacia_planks"
        | "dark_oak_planks" => self_drop_table(key, random_sequence),
        _ => return None,
    })
}

fn leaves_block_loot_table(key: &str, random_sequence: &str) -> Option<LootTable> {
    Some(match key {
        // Silk touch and fortune bonuses not yet implemented. Sapling chance is fortune-0 base.
        "oak_leaves" => leaves_table("oak_leaves", "oak_sapling", 0.05, true, random_sequence),
        "spruce_leaves" => leaves_table(
            "spruce_leaves",
            "spruce_sapling",
            0.05,
            false,
            random_sequence,
        ),
        "birch_leaves" => leaves_table(
            "birch_leaves",
            "birch_sapling",
            0.05,
            false,
            random_sequence,
        ),
        "jungle_leaves" => leaves_table(
            "jungle_leaves",
            "jungle_sapling",
            0.025,
            false,
            random_sequence,
        ),
        "acacia_leaves" => leaves_table(
            "acacia_leaves",
            "acacia_sapling",
            0.05,
            false,
            random_sequence,
        ),
        "dark_oak_leaves" => leaves_table(
            "dark_oak_leaves",
            "dark_oak_sapling",
            0.05,
            false,
            random_sequence,
        ),
        "cherry_leaves" => leaves_table(
            "cherry_leaves",
            "cherry_sapling",
            0.05,
            false,
            random_sequence,
        ),
        "pale_oak_leaves" => leaves_table(
            "pale_oak_leaves",
            "pale_oak_sapling",
            0.05,
            false,
            random_sequence,
        ),
        "azalea_leaves" => leaves_table("azalea_leaves", "azalea", 0.05, false, random_sequence),
        "flowering_azalea_leaves" => leaves_table(
            "flowering_azalea_leaves",
            "flowering_azalea",
            0.05,
            false,
            random_sequence,
        ),
        "mangrove_leaves" => mangrove_leaves_table(random_sequence),
        _ => return None,
    })
}

fn plant_block_loot_table(key: &str, random_sequence: &str) -> Option<LootTable> {
    Some(match key {
        "short_grass" => grass_table("short_grass", random_sequence),
        "fern" => grass_table("fern", random_sequence),
        "tall_grass" => tall_grass_table("short_grass", random_sequence),
        "large_fern" => tall_grass_table("fern", random_sequence),
        "dead_bush" => dead_bush_table(random_sequence),
        "dandelion" | "golden_dandelion" | "torchflower" | "poppy" | "blue_orchid" | "allium"
        | "azure_bluet" | "red_tulip" | "orange_tulip" | "white_tulip" | "pink_tulip"
        | "oxeye_daisy" | "cornflower" | "wither_rose" | "lily_of_the_valley"
        | "brown_mushroom" | "red_mushroom" | "wildflowers" | "firefly_bush" => {
            self_drop_table(key, random_sequence)
        }
        "sunflower" | "lilac" | "rose_bush" | "peony" => {
            self_drop_survives_explosion(key, random_sequence)
        }
        _ => return None,
    })
}

fn crop_block_loot_table(key: &str, random_sequence: &str) -> Option<LootTable> {
    Some(match key {
        // Block-state age checks not yet implemented; always treats crop as fully grown.
        "wheat" => mature_crop_table("wheat", "wheat_seeds", random_sequence),
        "carrots" => mature_crop_table("carrot", "carrot", random_sequence),
        "beetroots" => mature_crop_table("beetroot", "beetroot_seeds", random_sequence),
        "potatoes" => potatoes_table(random_sequence),
        _ => return None,
    })
}

fn special_block_loot_table(key: &str, random_sequence: &str) -> Option<LootTable> {
    Some(match key {
        "glowstone" => count_drop_table(
            "glowstone_dust",
            vec![
                LootFunction::SetCount(NumberProvider::Uniform { min: 2.0, max: 4.0 }),
                LootFunction::LimitCount { min: 1, max: 4 },
                LootFunction::ApplyExplosionDecay,
            ],
            random_sequence,
        ),
        "sea_lantern" => count_drop_table(
            "prismarine_crystals",
            vec![
                LootFunction::SetCount(NumberProvider::Uniform { min: 2.0, max: 3.0 }),
                LootFunction::LimitCount { min: 1, max: 5 },
                LootFunction::ApplyExplosionDecay,
            ],
            random_sequence,
        ),
        "bookshelf" => count_drop_table(
            "book",
            vec![
                LootFunction::SetCount(NumberProvider::Constant(3.0)),
                LootFunction::ApplyExplosionDecay,
            ],
            random_sequence,
        ),
        "snow_block" => count_drop_table(
            "snowball",
            vec![
                LootFunction::SetCount(NumberProvider::Constant(4.0)),
                LootFunction::ApplyExplosionDecay,
            ],
            random_sequence,
        ),
        "snow" => count_drop_table(
            "snowball",
            vec![LootFunction::SetCount(NumberProvider::Constant(1.0))],
            random_sequence,
        ),
        "melon" => count_drop_table(
            "melon_slice",
            vec![
                LootFunction::SetCount(NumberProvider::Uniform { min: 3.0, max: 7.0 }),
                LootFunction::LimitCount { min: 0, max: 9 },
                LootFunction::ApplyExplosionDecay,
            ],
            random_sequence,
        ),
        "pumpkin" | "carved_pumpkin" => self_drop_survives_explosion("pumpkin", random_sequence),
        "sugar_cane" | "cactus" | "bamboo" => self_drop_survives_explosion(key, random_sequence),
        "air" | "cave_air" | "void_air" | "bedrock" | "water" | "flowing_water" | "lava"
        | "flowing_lava" | "fire" | "soul_fire" | "ice" | "packed_ice" | "blue_ice" | "glass"
        | "glass_pane" | "nether_portal" => return None,
        _ => return None,
    })
}

fn self_drop_table(item: &str, sequence: &str) -> LootTable {
    LootTable {
        param_set: LootParamSet::Block,
        random_sequence: Some(sequence.to_string()),
        pools: vec![LootPool::single(LootEntry::item(
            format!("minecraft:{item}"),
            1,
        ))],
        functions: Vec::new(),
    }
}

fn self_drop_survives_explosion(item: &str, sequence: &str) -> LootTable {
    LootTable {
        param_set: LootParamSet::Block,
        random_sequence: Some(sequence.to_string()),
        pools: vec![LootPool {
            entries: vec![LootEntry::item(format!("minecraft:{item}"), 1)],
            conditions: vec![LootCondition::SurvivesExplosion],
            functions: Vec::new(),
            rolls: NumberProvider::Constant(1.0),
            bonus_rolls: NumberProvider::Constant(0.0),
        }],
        functions: Vec::new(),
    }
}

fn ore_drop_1(drop_item: &str, sequence: &str) -> LootTable {
    LootTable {
        param_set: LootParamSet::Block,
        random_sequence: Some(sequence.to_string()),
        pools: vec![LootPool::single(LootEntry::Item {
            item: format!("minecraft:{drop_item}"),
            weight: 1,
            quality: 0,
            conditions: Vec::new(),
            functions: vec![LootFunction::ApplyExplosionDecay],
        })],
        functions: Vec::new(),
    }
}

fn ore_drop_count(drop_item: &str, min: f32, max: f32, sequence: &str) -> LootTable {
    LootTable {
        param_set: LootParamSet::Block,
        random_sequence: Some(sequence.to_string()),
        pools: vec![LootPool::single(LootEntry::Item {
            item: format!("minecraft:{drop_item}"),
            weight: 1,
            quality: 0,
            conditions: Vec::new(),
            functions: vec![
                LootFunction::SetCount(NumberProvider::Uniform { min, max }),
                LootFunction::ApplyExplosionDecay,
            ],
        })],
        functions: Vec::new(),
    }
}

fn count_drop_table(item: &str, functions: Vec<LootFunction>, sequence: &str) -> LootTable {
    LootTable {
        param_set: LootParamSet::Block,
        random_sequence: Some(sequence.to_string()),
        pools: vec![LootPool::single(LootEntry::Item {
            item: format!("minecraft:{item}"),
            weight: 1,
            quality: 0,
            conditions: Vec::new(),
            functions,
        })],
        functions: Vec::new(),
    }
}

fn clay_table(sequence: &str) -> LootTable {
    count_drop_table(
        "clay_ball",
        vec![
            LootFunction::SetCount(NumberProvider::Constant(4.0)),
            LootFunction::ApplyExplosionDecay,
        ],
        sequence,
    )
}

fn gravel_table(sequence: &str) -> LootTable {
    LootTable {
        param_set: LootParamSet::Block,
        random_sequence: Some(sequence.to_string()),
        pools: vec![LootPool::single(LootEntry::Alternatives(vec![
            LootEntry::Item {
                item: "minecraft:flint".to_string(),
                weight: 1,
                quality: 0,
                conditions: vec![
                    LootCondition::SurvivesExplosion,
                    LootCondition::RandomChance(0.1),
                ],
                functions: Vec::new(),
            },
            LootEntry::Item {
                item: "minecraft:gravel".to_string(),
                weight: 1,
                quality: 0,
                conditions: vec![LootCondition::SurvivesExplosion],
                functions: Vec::new(),
            },
        ]))],
        functions: Vec::new(),
    }
}

fn grass_table(self_item: &str, sequence: &str) -> LootTable {
    LootTable {
        param_set: LootParamSet::Block,
        random_sequence: Some(sequence.to_string()),
        pools: vec![LootPool::single(LootEntry::Alternatives(vec![
            shears_drop_entry(self_item, Vec::new()),
            LootEntry::Item {
                item: "minecraft:wheat_seeds".to_string(),
                weight: 1,
                quality: 0,
                conditions: vec![LootCondition::RandomChance(0.125)],
                functions: Vec::new(),
            },
        ]))],
        functions: Vec::new(),
    }
}

fn leaves_table(
    leaves_item: &str,
    sapling_item: &str,
    sapling_chance: f32,
    has_apple_pool: bool,
    sequence: &str,
) -> LootTable {
    let not_shears = LootCondition::Inverted(Box::new(LootCondition::MatchTool {
        item: "minecraft:shears".to_string(),
    }));
    let mut pools = vec![
        LootPool::single(LootEntry::Alternatives(vec![
            shears_drop_entry(leaves_item, Vec::new()),
            LootEntry::Item {
                item: format!("minecraft:{sapling_item}"),
                weight: 1,
                quality: 0,
                conditions: vec![
                    LootCondition::SurvivesExplosion,
                    LootCondition::RandomChance(sapling_chance),
                ],
                functions: Vec::new(),
            },
        ])),
        stick_pool(not_shears.clone()),
    ];
    if has_apple_pool {
        pools.push(apple_pool(not_shears));
    }
    LootTable {
        param_set: LootParamSet::Block,
        random_sequence: Some(sequence.to_string()),
        pools,
        functions: Vec::new(),
    }
}

fn mangrove_leaves_table(sequence: &str) -> LootTable {
    LootTable {
        param_set: LootParamSet::Block,
        random_sequence: Some(sequence.to_string()),
        pools: vec![LootPool::single(LootEntry::Alternatives(vec![
            shears_drop_entry("mangrove_leaves", Vec::new()),
            LootEntry::Item {
                item: "minecraft:stick".to_string(),
                weight: 1,
                quality: 0,
                conditions: vec![LootCondition::RandomChance(0.02)],
                functions: vec![
                    LootFunction::SetCount(NumberProvider::Uniform { min: 1.0, max: 2.0 }),
                    LootFunction::ApplyExplosionDecay,
                ],
            },
        ]))],
        functions: Vec::new(),
    }
}

fn tall_grass_table(shears_item: &str, sequence: &str) -> LootTable {
    LootTable {
        param_set: LootParamSet::Block,
        random_sequence: Some(sequence.to_string()),
        pools: vec![LootPool::single(LootEntry::Alternatives(vec![
            shears_drop_entry(
                shears_item,
                vec![LootFunction::SetCount(NumberProvider::Constant(2.0))],
            ),
            LootEntry::Item {
                item: "minecraft:wheat_seeds".to_string(),
                weight: 1,
                quality: 0,
                conditions: vec![
                    LootCondition::SurvivesExplosion,
                    LootCondition::RandomChance(0.125),
                ],
                functions: Vec::new(),
            },
        ]))],
        functions: Vec::new(),
    }
}

fn dead_bush_table(sequence: &str) -> LootTable {
    LootTable {
        param_set: LootParamSet::Block,
        random_sequence: Some(sequence.to_string()),
        pools: vec![LootPool::single(LootEntry::Alternatives(vec![
            shears_drop_entry("dead_bush", Vec::new()),
            LootEntry::Item {
                item: "minecraft:stick".to_string(),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![
                    LootFunction::SetCount(NumberProvider::Uniform { min: 0.0, max: 2.0 }),
                    LootFunction::ApplyExplosionDecay,
                ],
            },
        ]))],
        functions: Vec::new(),
    }
}

fn mature_crop_table(food_item: &str, bonus_item: &str, sequence: &str) -> LootTable {
    LootTable {
        param_set: LootParamSet::Block,
        random_sequence: Some(sequence.to_string()),
        pools: vec![
            LootPool::single(LootEntry::item(format!("minecraft:{food_item}"), 1)),
            LootPool::single(LootEntry::Item {
                item: format!("minecraft:{bonus_item}"),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![LootFunction::SetCount(NumberProvider::Binomial {
                    n: 3,
                    p: 0.5714286,
                })],
            }),
        ],
        functions: vec![LootFunction::ApplyExplosionDecay],
    }
}

fn potatoes_table(sequence: &str) -> LootTable {
    LootTable {
        param_set: LootParamSet::Block,
        random_sequence: Some(sequence.to_string()),
        pools: vec![
            LootPool::single(LootEntry::item("minecraft:potato".to_string(), 1)),
            LootPool::single(LootEntry::Item {
                item: "minecraft:potato".to_string(),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![LootFunction::SetCount(NumberProvider::Binomial {
                    n: 3,
                    p: 0.5714286,
                })],
            }),
            LootPool::single(LootEntry::Item {
                item: "minecraft:poisonous_potato".to_string(),
                weight: 1,
                quality: 0,
                conditions: vec![LootCondition::RandomChance(0.02)],
                functions: Vec::new(),
            }),
        ],
        functions: vec![LootFunction::ApplyExplosionDecay],
    }
}

fn shears_drop_entry(item: &str, functions: Vec<LootFunction>) -> LootEntry {
    LootEntry::Item {
        item: format!("minecraft:{item}"),
        weight: 1,
        quality: 0,
        conditions: vec![LootCondition::MatchTool {
            item: "minecraft:shears".to_string(),
        }],
        functions,
    }
}

fn stick_pool(not_shears: LootCondition) -> LootPool {
    LootPool {
        entries: vec![LootEntry::Item {
            item: "minecraft:stick".to_string(),
            weight: 1,
            quality: 0,
            conditions: vec![LootCondition::RandomChance(0.02)],
            functions: vec![
                LootFunction::SetCount(NumberProvider::Uniform { min: 1.0, max: 2.0 }),
                LootFunction::ApplyExplosionDecay,
            ],
        }],
        conditions: vec![not_shears],
        functions: Vec::new(),
        rolls: NumberProvider::Constant(1.0),
        bonus_rolls: NumberProvider::Constant(0.0),
    }
}

fn apple_pool(not_shears: LootCondition) -> LootPool {
    LootPool {
        entries: vec![LootEntry::Item {
            item: "minecraft:apple".to_string(),
            weight: 1,
            quality: 0,
            conditions: vec![
                LootCondition::SurvivesExplosion,
                LootCondition::RandomChance(0.005),
            ],
            functions: Vec::new(),
        }],
        conditions: vec![not_shears],
        functions: Vec::new(),
        rolls: NumberProvider::Constant(1.0),
        bonus_rolls: NumberProvider::Constant(0.0),
    }
}
