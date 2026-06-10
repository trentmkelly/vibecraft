use super::*;

pub fn flat_generator_preset(id: &str) -> Option<&'static FlatGeneratorPreset> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    FLAT_GENERATOR_PRESETS
        .iter()
        .find(|preset| preset.id.strip_prefix("minecraft:").unwrap_or(preset.id) == name)
}

pub fn flat_layers_total_height(layers: &[FlatLayerInfo]) -> i32 {
    layers.iter().map(|layer| layer.height).sum()
}

pub fn validate_flat_layers(layers: &[FlatLayerInfo]) -> Result<(), String> {
    let total_height = flat_layers_total_height(layers);
    if total_height > 384 {
        Err("Sum of layer heights is > 384".to_string())
    } else {
        Ok(())
    }
}

pub fn flat_block_at_y(layers: &[FlatLayerInfo], y: i32) -> Option<&'static str> {
    if y < 0 {
        return None;
    }
    let mut cursor = 0;
    for layer in layers {
        let next = cursor + layer.height;
        if y < next {
            return Some(layer.block);
        }
        cursor = next;
    }
    None
}

pub fn flat_layers_are_void(layers: &[FlatLayerInfo]) -> bool {
    layers.iter().all(|layer| layer.block == "minecraft:air")
}

pub fn world_preset(id: &str) -> Option<&'static WorldPresetEntry> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    WORLD_PRESETS
        .iter()
        .find(|preset| preset.id.strip_prefix("minecraft:").unwrap_or(preset.id) == name)
}

pub fn chunk_generator_kind(id: &str) -> Option<ChunkGeneratorKind> {
    match id.strip_prefix("minecraft:").unwrap_or(id) {
        "noise" => Some(ChunkGeneratorKind::Noise),
        "flat" => Some(ChunkGeneratorKind::Flat),
        "debug" => Some(ChunkGeneratorKind::Debug),
        _ => None,
    }
}

pub fn resolve_level_stem(stem: &LevelStemPreset) -> Result<ResolvedLevelStem, String> {
    let generator = match chunk_generator_kind(stem.generator)
        .ok_or_else(|| format!("Unknown chunk generator {}", stem.generator))?
    {
        ChunkGeneratorKind::Noise => {
            let settings_id = stem
                .noise_settings
                .ok_or_else(|| format!("Noise generator {} has no settings", stem.dimension))?;
            let settings = builtin_noise_generator_settings(settings_id)
                .ok_or_else(|| format!("Unknown noise settings {settings_id}"))?;
            let biome_source_model = biome_source_from_stem_id(stem.biome_source)
                .ok_or_else(|| format!("Unknown biome source {}", stem.biome_source))?;
            ResolvedChunkGenerator::Noise {
                biome_source: stem.biome_source,
                biome_source_model,
                noise_settings: settings,
            }
        }
        ChunkGeneratorKind::Flat => {
            if stem.noise_settings.is_some() {
                return Err(format!(
                    "Flat generator {} must not carry noise settings",
                    stem.dimension
                ));
            }
            let biome_source_model = biome_source_from_stem_id(stem.biome_source)
                .ok_or_else(|| format!("Unknown biome source {}", stem.biome_source))?;
            ResolvedChunkGenerator::Flat {
                biome_source_model,
                settings: default_flat_generator_settings()?,
            }
        }
        ChunkGeneratorKind::Debug => {
            if stem.noise_settings.is_some() {
                return Err(format!(
                    "Debug generator {} must not carry noise settings",
                    stem.dimension
                ));
            }
            let biome_source_model = biome_source_from_stem_id(stem.biome_source)
                .ok_or_else(|| format!("Unknown biome source {}", stem.biome_source))?;
            ResolvedChunkGenerator::Debug {
                biome: stem.biome_source,
                biome_source_model,
            }
        }
    };
    Ok(ResolvedLevelStem {
        dimension: stem.dimension,
        generator,
    })
}

pub fn resolve_world_preset(id: &str) -> Result<ResolvedWorldPreset, String> {
    let preset = world_preset(id).ok_or_else(|| format!("Unknown world preset {id}"))?;
    Ok(ResolvedWorldPreset {
        id: preset.id,
        overworld: resolve_level_stem(&preset.overworld)?,
        nether: resolve_level_stem(&preset.nether)?,
        end: resolve_level_stem(&preset.end)?,
    })
}

pub fn resolve_world_preset_from_registry(
    registry: &ParsedWorldgenPresetRegistry,
    id: &str,
) -> Result<ResolvedWorldPreset, String> {
    let id = static_world_preset_id(id).ok_or_else(|| format!("Unknown world preset {id}"))?;
    let parsed = registry
        .world_presets
        .get(id)
        .ok_or_else(|| format!("Missing world preset registry entry {id}"))?;
    let overworld = parsed_level_stem(parsed, "minecraft:overworld")?;
    let nether = parsed_level_stem(parsed, "minecraft:the_nether")?;
    let end = parsed_level_stem(parsed, "minecraft:the_end")?;
    Ok(ResolvedWorldPreset {
        id,
        overworld: resolve_parsed_level_stem("minecraft:overworld", overworld)?,
        nether: resolve_parsed_level_stem("minecraft:the_nether", nether)?,
        end: resolve_parsed_level_stem("minecraft:the_end", end)?,
    })
}

fn parsed_level_stem<'a>(
    preset: &'a ParsedWorldPreset,
    id: &str,
) -> Result<&'a ParsedLevelStem, String> {
    preset
        .dimensions
        .stems
        .iter()
        .find(|(stem_id, _)| stem_id == id)
        .map(|(_, stem)| stem)
        .ok_or_else(|| format!("Missing dimension {id}"))
}

fn resolve_parsed_level_stem(
    dimension: &'static str,
    stem: &ParsedLevelStem,
) -> Result<ResolvedLevelStem, String> {
    let parsed_dimension = static_dimension_type(&stem.dimension_type)
        .ok_or_else(|| format!("Unknown dimension type {}", stem.dimension_type))?;
    if parsed_dimension != dimension {
        return Err(format!(
            "Dimension {dimension} has mismatched type {}",
            stem.dimension_type
        ));
    }

    let generator = match &stem.generator {
        ParsedChunkGenerator::Noise {
            biome_source,
            settings,
        } => {
            let settings = builtin_noise_generator_settings(settings)
                .ok_or_else(|| format!("Unknown noise settings {settings}"))?;
            let (biome_source, biome_source_model) = resolve_parsed_biome_source(biome_source)?;
            ResolvedChunkGenerator::Noise {
                biome_source,
                biome_source_model,
                noise_settings: settings,
            }
        }
        ParsedChunkGenerator::Flat { settings } => {
            let biome_source_model = BiomeSourceModel::Fixed {
                biome: static_biome_id(&settings.biome)
                    .ok_or_else(|| format!("Unknown biome {}", settings.biome))?,
            };
            ResolvedChunkGenerator::Flat {
                biome_source_model,
                settings: flat_generator_settings_from_parsed(settings)?,
            }
        }
        ParsedChunkGenerator::Debug => ResolvedChunkGenerator::Debug {
            biome: "minecraft:plains",
            biome_source_model: BiomeSourceModel::Fixed {
                biome: "minecraft:plains",
            },
        },
    };

    Ok(ResolvedLevelStem {
        dimension,
        generator,
    })
}

fn resolve_parsed_biome_source(
    biome_source: &ParsedBiomeSource,
) -> Result<(&'static str, BiomeSourceModel), String> {
    match biome_source {
        ParsedBiomeSource::MultiNoisePreset { preset } => match strip_minecraft(preset) {
            "overworld" => Ok((
                "minecraft:multi_noise/overworld",
                BiomeSourceModel::MultiNoisePreset {
                    preset: "minecraft:overworld",
                },
            )),
            "nether" => Ok((
                "minecraft:multi_noise/nether",
                BiomeSourceModel::MultiNoisePreset {
                    preset: "minecraft:nether",
                },
            )),
            _ => Err(format!("Unknown multi-noise biome source preset {preset}")),
        },
        ParsedBiomeSource::TheEnd => Ok(("minecraft:the_end", BiomeSourceModel::TheEnd)),
        ParsedBiomeSource::Fixed { biome } => {
            let biome = static_biome_id(biome).ok_or_else(|| format!("Unknown biome {biome}"))?;
            Ok((biome, BiomeSourceModel::Fixed { biome }))
        }
        ParsedBiomeSource::Checkerboard { biomes, scale } => {
            let scale = i32::try_from(*scale)
                .map_err(|_| format!("checkerboard scale {scale} overflows i32"))?;
            let biomes = biomes
                .iter()
                .map(|biome| static_biome_id(biome).ok_or_else(|| format!("Unknown biome {biome}")))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((
                "minecraft:checkerboard",
                BiomeSourceModel::Checkerboard { biomes, scale },
            ))
        }
    }
}

fn flat_generator_settings_from_parsed(
    parsed: &ParsedFlatGeneratorSettings,
) -> Result<FlatGeneratorSettingsModel, String> {
    let layers = parsed
        .layers
        .iter()
        .map(|(height, block)| {
            Ok(FlatLayerInfo {
                height: *height,
                block: static_block_id(block).ok_or_else(|| format!("Unknown block {block}"))?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let expanded_raw = expand_flat_layers(&layers)?;
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

    let structure_overrides = parsed
        .structure_overrides
        .iter()
        .map(|structure| {
            static_structure_set_id(structure)
                .ok_or_else(|| format!("Unknown structure override {structure}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(FlatGeneratorSettingsModel {
        biome: static_biome_id(&parsed.biome)
            .ok_or_else(|| format!("Unknown biome {}", parsed.biome))?,
        structure_overrides,
        add_lakes: parsed.add_lakes,
        decoration: parsed.decoration,
        layers,
        expanded_layers,
        top_layer_modifications,
        void_generation,
    })
}

fn static_world_preset_id(id: &str) -> Option<&'static str> {
    match strip_minecraft(id) {
        "normal" => Some("minecraft:normal"),
        "flat" => Some("minecraft:flat"),
        "large_biomes" => Some("minecraft:large_biomes"),
        "amplified" => Some("minecraft:amplified"),
        "single_biome_surface" => Some("minecraft:single_biome_surface"),
        "debug_all_block_states" => Some("minecraft:debug_all_block_states"),
        _ => None,
    }
}

fn static_dimension_type(id: &str) -> Option<&'static str> {
    match strip_minecraft(id) {
        "overworld" => Some("minecraft:overworld"),
        "the_nether" => Some("minecraft:the_nether"),
        "the_end" => Some("minecraft:the_end"),
        _ => None,
    }
}

pub(super) fn static_biome_id(id: &str) -> Option<&'static str> {
    // Look the biome up in the full vanilla registry (`BUILTIN_BIOMES`) so
    // every biome string we ever produce (forest, badlands, jungle, …) can
    // be round-tripped through the chunk's biome storage. The previous
    // hardcoded six-biome match silently coerced every other biome back to
    // `minecraft:plains` when chunks were re-read via
    // `ChunkNoiseBiomeCache::from_chunk`, which produced the visible
    // chunk-border outline in badlands/beach biomes — the surface
    // generator's fallback path correctly resolved the depth-aware biome
    // for the chunk-border 1-block strip, while the cache path returned
    // the coerced plains for the chunk interior.
    let needle = id.strip_prefix("minecraft:").unwrap_or(id);
    crate::biome::BUILTIN_BIOMES
        .iter()
        .map(|entry| entry.id)
        .find(|registered| registered.strip_prefix("minecraft:").unwrap_or(registered) == needle)
}

fn static_block_id(id: &str) -> Option<&'static str> {
    match strip_minecraft(id) {
        "air" => Some("minecraft:air"),
        "barrier" => Some("minecraft:barrier"),
        "bedrock" => Some("minecraft:bedrock"),
        "cobblestone" => Some("minecraft:cobblestone"),
        "deepslate" => Some("minecraft:deepslate"),
        "dirt" => Some("minecraft:dirt"),
        "grass_block" => Some("minecraft:grass_block"),
        "gravel" => Some("minecraft:gravel"),
        "sand" => Some("minecraft:sand"),
        "sandstone" => Some("minecraft:sandstone"),
        "short_grass" => Some("minecraft:short_grass"),
        "snow" => Some("minecraft:snow"),
        "stone" => Some("minecraft:stone"),
        "water" => Some("minecraft:water"),
        _ => None,
    }
}

fn static_structure_set_id(id: &str) -> Option<&'static str> {
    match strip_minecraft(id) {
        "desert_pyramids" => Some("minecraft:desert_pyramids"),
        "igloos" => Some("minecraft:igloos"),
        "mineshafts" => Some("minecraft:mineshafts"),
        "ocean_monuments" => Some("minecraft:ocean_monuments"),
        "ocean_ruins" => Some("minecraft:ocean_ruins"),
        "pillager_outposts" => Some("minecraft:pillager_outposts"),
        "ruined_portals" => Some("minecraft:ruined_portals"),
        "shipwrecks" => Some("minecraft:shipwrecks"),
        "strongholds" => Some("minecraft:strongholds"),
        "villages" => Some("minecraft:villages"),
        _ => None,
    }
}
