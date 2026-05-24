use super::*;

pub const VANILLA_DIMENSION_Y_SIZE: i32 = 384;
pub const FLAT_GENERATOR_MIN_Y: i32 = 0;
pub const FLAT_GENERATOR_GEN_DEPTH: i32 = 384;
pub const FLAT_GENERATOR_SEA_LEVEL: i32 = -63;

pub fn flat_generator_settings(
    preset: &FlatGeneratorPreset,
) -> Result<FlatGeneratorSettingsModel, String> {
    let expanded_raw = expand_flat_layers(preset.layers)?;
    let mut expanded_layers = expanded_raw.clone();
    let mut top_layer_modifications = Vec::new();
    let void_generation = expanded_layers
        .iter()
        .all(|state| state.map(|block| block == "minecraft:air").unwrap_or(true));

    for (y, state) in expanded_layers.iter_mut().enumerate() {
        let Some(block) = *state else {
            continue;
        };
        if !block_blocks_motion(block) {
            *state = None;
            top_layer_modifications.push((y, block));
        }
    }

    Ok(FlatGeneratorSettingsModel {
        biome: preset.biome,
        structure_overrides: preset.structures.to_vec(),
        add_lakes: preset.add_lakes,
        decoration: preset.decoration,
        layers: preset.layers.to_vec(),
        expanded_layers,
        top_layer_modifications,
        void_generation,
    })
}

pub fn default_flat_generator_settings() -> Result<FlatGeneratorSettingsModel, String> {
    let expanded_layers = expand_flat_layers(FLAT_DEFAULT_LAYERS)?;
    Ok(FlatGeneratorSettingsModel {
        biome: "minecraft:plains",
        structure_overrides: vec!["minecraft:strongholds", "minecraft:villages"],
        add_lakes: false,
        decoration: false,
        layers: FLAT_DEFAULT_LAYERS.to_vec(),
        expanded_layers,
        top_layer_modifications: Vec::new(),
        void_generation: false,
    })
}

pub fn expand_flat_layers(layers: &[FlatLayerInfo]) -> Result<Vec<Option<&'static str>>, String> {
    let total_height = layers.iter().try_fold(0_i32, |sum, layer| {
        if layer.height < 0 {
            Err(format!(
                "flat layer {} has negative height {}",
                layer.block, layer.height
            ))
        } else {
            Ok(sum + layer.height)
        }
    })?;
    if total_height > VANILLA_DIMENSION_Y_SIZE {
        return Err(format!(
            "Sum of layer heights is > {}",
            VANILLA_DIMENSION_Y_SIZE
        ));
    }

    let mut expanded = Vec::with_capacity(total_height as usize);
    for layer in layers {
        for _ in 0..layer.height {
            expanded.push(Some(layer.block));
        }
    }
    Ok(expanded)
}

pub fn flat_base_height(
    layers: &[Option<&'static str>],
    min_y: i32,
    height: i32,
    heightmap: HeightmapKind,
) -> i32 {
    let max_layer = layers.len().min(height.max(0) as usize);
    for layer_index in (0..max_layer).rev() {
        if layers[layer_index].is_some_and(|block| heightmap_opaque(heightmap, block)) {
            return min_y + layer_index as i32 + 1;
        }
    }
    min_y
}

pub fn flat_base_column(
    layers: &[Option<&'static str>],
    min_y: i32,
    height: i32,
) -> FlatNoiseColumn {
    let states = (0..height.max(0) as usize)
        .map(|index| {
            layers
                .get(index)
                .and_then(|state| *state)
                .unwrap_or("minecraft:air")
        })
        .collect();
    FlatNoiseColumn { min_y, states }
}

pub fn materialize_flat_chunk(pos: ChunkPos, settings: &FlatGeneratorSettingsModel) -> LevelChunk {
    let mut chunk = LevelChunk::empty(pos);
    chunk.status = "minecraft:full".to_string();
    chunk.min_section_y = FLAT_GENERATOR_MIN_Y.div_euclid(16);

    let layers = &settings.expanded_layers;
    let max_layer = layers
        .iter()
        .rposition(Option::is_some)
        .map(|index| index + 1)
        .unwrap_or(0);
    let section_count = max_layer.div_ceil(16);
    chunk.sections = (0..section_count)
        .map(|section_index| {
            let section_min_y = FLAT_GENERATOR_MIN_Y + section_index as i32 * 16;
            let block_states = flat_section_block_states(layers, section_index);
            ChunkSection {
                y: (section_min_y / 16) as i8,
                block_states: block_states.to_nbt(),
                biomes: PalettedContainer::single(
                    Tag::String(settings.biome.to_string()),
                    BIOME_SECTION_VOLUME,
                )
                .to_nbt(),
                block_light: None,
                sky_light: Some(vec![-1; 2048]),
            }
        })
        .collect();

    let world_surface = flat_base_height(
        layers,
        FLAT_GENERATOR_MIN_Y,
        FLAT_GENERATOR_GEN_DEPTH,
        HeightmapKind::WorldSurfaceWg,
    );
    let ocean_floor = flat_base_height(
        layers,
        FLAT_GENERATOR_MIN_Y,
        FLAT_GENERATOR_GEN_DEPTH,
        HeightmapKind::OceanFloorWg,
    );
    chunk.heightmaps = BTreeMap::from([
        (
            HeightmapKind::WorldSurfaceWg.storage_name().to_string(),
            Tag::LongArray(pack_heightmap([world_surface; 16 * 16])),
        ),
        (
            HeightmapKind::OceanFloorWg.storage_name().to_string(),
            Tag::LongArray(pack_heightmap([ocean_floor; 16 * 16])),
        ),
    ]);
    chunk
}

pub fn flat_adjusted_generation_feature_steps(
    settings: &FlatGeneratorSettingsModel,
    source_biome_feature_steps: &[&[&'static str]],
) -> Vec<Vec<&'static str>> {
    let mut steps = vec![Vec::new(); GenerationDecorationStep::VALUES.len()];
    if settings.add_lakes {
        steps[GenerationDecorationStep::Lakes as usize].extend([
            "minecraft:lake_lava_underground",
            "minecraft:lake_lava_surface",
        ]);
    }

    let biome_decoration = (!settings.void_generation || settings.biome == "minecraft:the_void")
        && settings.decoration;
    if biome_decoration {
        for (step_index, features) in source_biome_feature_steps.iter().enumerate() {
            if step_index == GenerationDecorationStep::UndergroundStructures as usize
                || step_index == GenerationDecorationStep::SurfaceStructures as usize
                || (settings.add_lakes && step_index == GenerationDecorationStep::Lakes as usize)
            {
                continue;
            }
            if let Some(target_step) = steps.get_mut(step_index) {
                target_step.extend(features.iter().copied());
            }
        }
    }

    steps[GenerationDecorationStep::TopLayerModification as usize].extend(
        settings
            .top_layer_modifications
            .iter()
            .map(|_| "minecraft:fill_layer"),
    );
    steps
}

pub const FLAT_DEFAULT_LAYERS: &[FlatLayerInfo] = &[
    FlatLayerInfo {
        height: 1,
        block: "minecraft:bedrock",
    },
    FlatLayerInfo {
        height: 2,
        block: "minecraft:dirt",
    },
    FlatLayerInfo {
        height: 1,
        block: "minecraft:grass_block",
    },
];

pub const FLAT_GENERATOR_PRESETS: &[FlatGeneratorPreset] = &[
    FlatGeneratorPreset {
        id: "minecraft:classic_flat",
        display: "minecraft:grass_block",
        biome: "minecraft:plains",
        structures: &["minecraft:villages"],
        add_lakes: false,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 2,
                block: "minecraft:dirt",
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:grass_block",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:tunnelers_dream",
        display: "minecraft:stone",
        biome: "minecraft:windswept_hills",
        structures: &["minecraft:mineshafts", "minecraft:strongholds"],
        add_lakes: true,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 230,
                block: "minecraft:stone",
            },
            FlatLayerInfo {
                height: 5,
                block: "minecraft:dirt",
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:grass_block",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:water_world",
        display: "minecraft:water_bucket",
        biome: "minecraft:deep_ocean",
        structures: &[
            "minecraft:ocean_ruins",
            "minecraft:shipwrecks",
            "minecraft:ocean_monuments",
        ],
        add_lakes: false,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 64,
                block: "minecraft:deepslate",
            },
            FlatLayerInfo {
                height: 5,
                block: "minecraft:stone",
            },
            FlatLayerInfo {
                height: 5,
                block: "minecraft:dirt",
            },
            FlatLayerInfo {
                height: 5,
                block: "minecraft:gravel",
            },
            FlatLayerInfo {
                height: 90,
                block: "minecraft:water",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:overworld",
        display: "minecraft:short_grass",
        biome: "minecraft:plains",
        structures: &[
            "minecraft:villages",
            "minecraft:mineshafts",
            "minecraft:pillager_outposts",
            "minecraft:ruined_portals",
            "minecraft:strongholds",
        ],
        add_lakes: true,
        decoration: true,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 59,
                block: "minecraft:stone",
            },
            FlatLayerInfo {
                height: 3,
                block: "minecraft:dirt",
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:grass_block",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:snowy_kingdom",
        display: "minecraft:snow",
        biome: "minecraft:snowy_plains",
        structures: &["minecraft:villages", "minecraft:igloos"],
        add_lakes: false,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 59,
                block: "minecraft:stone",
            },
            FlatLayerInfo {
                height: 3,
                block: "minecraft:dirt",
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:grass_block",
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:snow",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:bottomless_pit",
        display: "minecraft:feather",
        biome: "minecraft:plains",
        structures: &["minecraft:villages"],
        add_lakes: false,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 2,
                block: "minecraft:cobblestone",
            },
            FlatLayerInfo {
                height: 3,
                block: "minecraft:dirt",
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:grass_block",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:desert",
        display: "minecraft:sand",
        biome: "minecraft:desert",
        structures: &[
            "minecraft:villages",
            "minecraft:desert_pyramids",
            "minecraft:mineshafts",
            "minecraft:strongholds",
        ],
        add_lakes: true,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 3,
                block: "minecraft:stone",
            },
            FlatLayerInfo {
                height: 52,
                block: "minecraft:sandstone",
            },
            FlatLayerInfo {
                height: 8,
                block: "minecraft:sand",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:redstone_ready",
        display: "minecraft:redstone",
        biome: "minecraft:desert",
        structures: &[],
        add_lakes: false,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 3,
                block: "minecraft:stone",
            },
            FlatLayerInfo {
                height: 116,
                block: "minecraft:sandstone",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:the_void",
        display: "minecraft:barrier",
        biome: "minecraft:the_void",
        structures: &[],
        add_lakes: true,
        decoration: false,
        layers: &[FlatLayerInfo {
            height: 1,
            block: "minecraft:air",
        }],
    },
];

fn flat_section_block_states(
    layers: &[Option<&'static str>],
    section_index: usize,
) -> PalettedContainer {
    let section_start = section_index * 16;
    let mut palette: Vec<&'static str> = Vec::new();
    let mut indices = vec![0_u64; SECTION_VOLUME];

    for local_y in 0..16 {
        let block = layers
            .get(section_start + local_y)
            .and_then(|state| *state)
            .unwrap_or("minecraft:air");
        let palette_index = match palette.iter().position(|entry| *entry == block) {
            Some(index) => index as u64,
            None => {
                palette.push(block);
                (palette.len() - 1) as u64
            }
        };
        for z in 0..16 {
            for x in 0..16 {
                indices[(local_y << 8) | (z << 4) | x] = palette_index;
            }
        }
    }

    if palette.len() == 1 {
        return PalettedContainer::single(block_state_tag(palette[0]), SECTION_VOLUME);
    }

    PalettedContainer {
        palette: palette.into_iter().map(block_state_tag).collect(),
        data: Some(pack_palette_indices(
            &indices,
            bits_for_palette(indices.iter().copied().max().unwrap_or(0) + 1),
        )),
        expected_entries: SECTION_VOLUME,
    }
}
