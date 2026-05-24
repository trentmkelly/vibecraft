#![allow(dead_code)]

use super::*;

impl VanillaBlockArrayParityScore {
    pub fn from_counts(matching_blocks: usize, total_blocks: usize) -> Result<Self, String> {
        Self::from_counts_and_mismatches(
            matching_blocks,
            total_blocks,
            BTreeMap::new(),
            Vec::new(),
            BTreeMap::new(),
            BTreeMap::new(),
        )
    }

    pub fn from_counts_and_mismatches(
        matching_blocks: usize,
        total_blocks: usize,
        mismatches: BTreeMap<(String, String), usize>,
        samples: Vec<VanillaBlockArrayMismatchSample>,
        chunk_mismatches: BTreeMap<(i32, i32), usize>,
        y_band_mismatches: BTreeMap<i32, usize>,
    ) -> Result<Self, String> {
        if total_blocks == 0 {
            return Err("cannot score worldgen parity without block samples".to_string());
        }
        let mut mismatches = mismatches
            .into_iter()
            .map(
                |((expected, actual), count)| VanillaBlockArrayMismatchCount {
                    expected,
                    actual,
                    count,
                },
            )
            .collect::<Vec<_>>();
        mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.expected.cmp(&right.expected))
                .then_with(|| left.actual.cmp(&right.actual))
        });
        let mut chunk_mismatches = chunk_mismatches
            .into_iter()
            .map(|((x, z), count)| VanillaChunkMismatchCount {
                chunk: ChunkCoord { x, z },
                count,
            })
            .collect::<Vec<_>>();
        chunk_mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.chunk.x.cmp(&right.chunk.x))
                .then_with(|| left.chunk.z.cmp(&right.chunk.z))
        });
        let mut y_band_mismatches = y_band_mismatches
            .into_iter()
            .map(|(y_min, count)| VanillaYBandMismatchCount {
                y_min,
                y_max_exclusive: y_min + 16,
                count,
            })
            .collect::<Vec<_>>();
        y_band_mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.y_min.cmp(&right.y_min))
        });
        Ok(Self {
            matching_blocks,
            total_blocks,
            score: matching_blocks as f64 / total_blocks as f64,
            mismatches,
            samples,
            chunk_mismatches,
            y_band_mismatches,
        })
    }
}

impl VanillaHeightmapParityScore {
    pub fn from_counts_and_mismatches(
        matching_columns: usize,
        total_columns: usize,
        mismatches: BTreeMap<(i32, i32), usize>,
        samples: Vec<VanillaHeightmapMismatchSample>,
        chunk_mismatches: BTreeMap<(i32, i32), usize>,
        delta_mismatches: BTreeMap<i32, usize>,
    ) -> Result<Self, String> {
        if total_columns == 0 {
            return Err("cannot score worldgen heightmap parity without columns".to_string());
        }
        let mut mismatches = mismatches
            .into_iter()
            .map(
                |((expected_height, actual_height), count)| VanillaHeightmapMismatchCount {
                    expected_height,
                    actual_height,
                    count,
                },
            )
            .collect::<Vec<_>>();
        mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.expected_height.cmp(&right.expected_height))
                .then_with(|| left.actual_height.cmp(&right.actual_height))
        });
        let mut chunk_mismatches = chunk_mismatches
            .into_iter()
            .map(|((x, z), count)| VanillaChunkMismatchCount {
                chunk: ChunkCoord { x, z },
                count,
            })
            .collect::<Vec<_>>();
        chunk_mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.chunk.x.cmp(&right.chunk.x))
                .then_with(|| left.chunk.z.cmp(&right.chunk.z))
        });
        let mut delta_mismatches = delta_mismatches
            .into_iter()
            .map(|(delta, count)| VanillaHeightmapDeltaMismatchCount { delta, count })
            .collect::<Vec<_>>();
        delta_mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.delta.cmp(&right.delta))
        });
        Ok(Self {
            matching_columns,
            total_columns,
            score: matching_columns as f64 / total_columns as f64,
            mismatches,
            samples,
            chunk_mismatches,
            delta_mismatches,
        })
    }
}

impl VanillaBiomeGridParityScore {
    pub fn from_counts_and_mismatches(
        matching_biomes: usize,
        total_biomes: usize,
        mismatches: BTreeMap<(String, String), usize>,
    ) -> Result<Self, String> {
        if total_biomes == 0 {
            return Err(
                "cannot score worldgen biome-grid parity without biome samples".to_string(),
            );
        }
        let mut mismatches = mismatches
            .into_iter()
            .map(
                |((expected, actual), count)| VanillaBiomeGridMismatchCount {
                    expected,
                    actual,
                    count,
                },
            )
            .collect::<Vec<_>>();
        mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.expected.cmp(&right.expected))
                .then_with(|| left.actual.cmp(&right.actual))
        });
        Ok(Self {
            matching_biomes,
            total_biomes,
            score: matching_biomes as f64 / total_biomes as f64,
            mismatches,
        })
    }
}

impl VanillaColumnProfileParityScore {
    pub fn from_counts_and_mismatches(
        matching_columns: usize,
        total_columns: usize,
        first_diff_y_mismatches: BTreeMap<i32, usize>,
        prefix_len_mismatches: BTreeMap<usize, usize>,
        surface_stack_mismatches: BTreeMap<(String, String), usize>,
        samples: Vec<VanillaColumnProfileMismatchSample>,
        chunk_mismatches: BTreeMap<(i32, i32), usize>,
    ) -> Result<Self, String> {
        if total_columns == 0 {
            return Err("cannot score worldgen column-profile parity without columns".to_string());
        }
        let mut first_diff_y_mismatches = first_diff_y_mismatches
            .into_iter()
            .map(|(y, count)| VanillaColumnFirstDiffYMismatchCount { y, count })
            .collect::<Vec<_>>();
        first_diff_y_mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.y.cmp(&right.y))
        });
        let mut prefix_len_mismatches = prefix_len_mismatches
            .into_iter()
            .map(
                |(matching_prefix_blocks, count)| VanillaColumnPrefixLenMismatchCount {
                    matching_prefix_blocks,
                    count,
                },
            )
            .collect::<Vec<_>>();
        prefix_len_mismatches.sort_by(|left, right| {
            right.count.cmp(&left.count).then_with(|| {
                left.matching_prefix_blocks
                    .cmp(&right.matching_prefix_blocks)
            })
        });
        let mut surface_stack_mismatches = surface_stack_mismatches
            .into_iter()
            .map(
                |((expected_stack, actual_stack), count)| VanillaColumnSurfaceStackMismatchCount {
                    expected_stack,
                    actual_stack,
                    count,
                },
            )
            .collect::<Vec<_>>();
        surface_stack_mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.expected_stack.cmp(&right.expected_stack))
                .then_with(|| left.actual_stack.cmp(&right.actual_stack))
        });
        let mut chunk_mismatches = chunk_mismatches
            .into_iter()
            .map(|((x, z), count)| VanillaChunkMismatchCount {
                chunk: ChunkCoord { x, z },
                count,
            })
            .collect::<Vec<_>>();
        chunk_mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.chunk.x.cmp(&right.chunk.x))
                .then_with(|| left.chunk.z.cmp(&right.chunk.z))
        });
        Ok(Self {
            matching_columns,
            total_columns,
            score: matching_columns as f64 / total_columns as f64,
            first_diff_y_mismatches,
            prefix_len_mismatches,
            surface_stack_mismatches,
            samples,
            chunk_mismatches,
        })
    }
}

pub fn vanilla_worldgen_block_array_parity_score(
    vanilla_fixture_json: &str,
    actual_chunks: &[LevelChunk],
) -> Result<VanillaBlockArrayParityScore, String> {
    let fixture: Value = serde_json::from_str(vanilla_fixture_json)
        .map_err(|err| format!("failed to parse vanilla worldgen block-array fixture: {err}"))?;
    let format = fixture
        .get("format")
        .and_then(Value::as_str)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing format".to_string())?;
    if format != "rustcraft-vanilla-worldgen-block-array-target-v1" {
        return Err(format!(
            "unsupported vanilla worldgen block-array fixture format: {format}"
        ));
    }

    let chunks = fixture
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing chunks".to_string())?;
    let mut matching_blocks = 0usize;
    let mut total_blocks = 0usize;
    let mut mismatches = BTreeMap::<(String, String), usize>::new();
    let mut samples = Vec::new();
    let mut chunk_mismatches = BTreeMap::<(i32, i32), usize>::new();
    let mut y_band_mismatches = BTreeMap::<i32, usize>::new();

    for expected_chunk in chunks {
        let dimension = expected_chunk
            .get("dimension")
            .and_then(Value::as_str)
            .unwrap_or("overworld");
        if dimension != "overworld" {
            return Err(format!(
                "unsupported vanilla worldgen block-array dimension: {dimension}"
            ));
        }

        let chunk_x = json_i32_field(expected_chunk, "chunkX")?;
        let chunk_z = json_i32_field(expected_chunk, "chunkZ")?;
        let y_min = json_i32_field(expected_chunk, "yMin")?;
        let blocks = expected_chunk
            .get("blocks")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                format!("vanilla fixture chunk ({chunk_x},{chunk_z}) is missing blocks")
            })?;
        if blocks.len() != 16 {
            return Err(format!(
                "vanilla fixture chunk ({chunk_x},{chunk_z}) has {} x columns, expected 16",
                blocks.len()
            ));
        }

        let actual = actual_chunks
            .iter()
            .find(|chunk| chunk.pos.x == chunk_x && chunk.pos.z == chunk_z)
            .ok_or_else(|| format!("missing actual generated chunk ({chunk_x},{chunk_z})"))?;

        for (local_x, y_column) in blocks.iter().enumerate() {
            let y_column = y_column.as_array().ok_or_else(|| {
                format!("vanilla fixture chunk ({chunk_x},{chunk_z}) x={local_x} is not an array")
            })?;
            for (y_offset, z_column) in y_column.iter().enumerate() {
                let z_column = z_column.as_array().ok_or_else(|| {
                    format!(
                        "vanilla fixture chunk ({chunk_x},{chunk_z}) x={local_x} y_offset={y_offset} is not an array"
                    )
                })?;
                if z_column.len() != 16 {
                    return Err(format!(
                        "vanilla fixture chunk ({chunk_x},{chunk_z}) x={local_x} y_offset={y_offset} has {} z entries, expected 16",
                        z_column.len()
                    ));
                }
                let world_y = y_min + y_offset as i32;
                for (local_z, expected) in z_column.iter().enumerate() {
                    let expected = expected
                        .as_str()
                        .ok_or_else(|| {
                            format!(
                                "vanilla fixture chunk ({chunk_x},{chunk_z}) x={local_x} y_offset={y_offset} z={local_z} is not a block string"
                            )
                        })
                        .map(vanilla_block_type)?;
                    let actual = actual
                        .get_block_state(
                            chunk_x * 16 + local_x as i32,
                            world_y,
                            chunk_z * 16 + local_z as i32,
                        )
                        .map(|block| vanilla_block_type(&block).to_string())
                        .unwrap_or_else(|| "minecraft:air".to_string());
                    if actual == expected {
                        matching_blocks += 1;
                    } else {
                        let chunk = ChunkCoord {
                            x: chunk_x,
                            z: chunk_z,
                        };
                        let expected_sample = expected.clone();
                        let actual_sample = actual.clone();
                        *mismatches.entry((expected, actual)).or_default() += 1;
                        *chunk_mismatches.entry((chunk.x, chunk.z)).or_default() += 1;
                        *y_band_mismatches
                            .entry(world_y.div_euclid(16) * 16)
                            .or_default() += 1;
                        if samples.len() < 128 {
                            samples.push(VanillaBlockArrayMismatchSample {
                                chunk,
                                local_x,
                                y: world_y,
                                local_z,
                                expected: expected_sample,
                                actual: actual_sample,
                            });
                        }
                    }
                    total_blocks += 1;
                }
            }
        }
    }

    VanillaBlockArrayParityScore::from_counts_and_mismatches(
        matching_blocks,
        total_blocks,
        mismatches,
        samples,
        chunk_mismatches,
        y_band_mismatches,
    )
}

pub fn vanilla_worldgen_heightmap_parity_score(
    vanilla_fixture_json: &str,
    actual_chunks: &[LevelChunk],
) -> Result<VanillaHeightmapParityScore, String> {
    let fixture: Value = serde_json::from_str(vanilla_fixture_json)
        .map_err(|err| format!("failed to parse vanilla worldgen block-array fixture: {err}"))?;
    let format = fixture
        .get("format")
        .and_then(Value::as_str)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing format".to_string())?;
    if format != "rustcraft-vanilla-worldgen-block-array-target-v1" {
        return Err(format!(
            "unsupported vanilla worldgen block-array fixture format: {format}"
        ));
    }

    let chunks = fixture
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing chunks".to_string())?;
    let mut matching_columns = 0usize;
    let mut total_columns = 0usize;
    let mut mismatches = BTreeMap::<(i32, i32), usize>::new();
    let mut samples = Vec::new();
    let mut chunk_mismatches = BTreeMap::<(i32, i32), usize>::new();
    let mut delta_mismatches = BTreeMap::<i32, usize>::new();

    for expected_chunk in chunks {
        let dimension = expected_chunk
            .get("dimension")
            .and_then(Value::as_str)
            .unwrap_or("overworld");
        if dimension != "overworld" {
            return Err(format!(
                "unsupported vanilla worldgen block-array dimension: {dimension}"
            ));
        }
        let chunk_x = json_i32_field(expected_chunk, "chunkX")?;
        let chunk_z = json_i32_field(expected_chunk, "chunkZ")?;
        let y_min = json_i32_field(expected_chunk, "yMin")?;
        let y_max_exclusive = json_i32_field(expected_chunk, "yMaxExclusive")?;
        let blocks = expected_chunk
            .get("blocks")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                format!("vanilla fixture chunk ({chunk_x},{chunk_z}) is missing blocks")
            })?;
        if blocks.len() != 16 {
            return Err(format!(
                "vanilla fixture chunk ({chunk_x},{chunk_z}) has {} x columns, expected 16",
                blocks.len()
            ));
        }

        let actual = actual_chunks
            .iter()
            .find(|chunk| chunk.pos.x == chunk_x && chunk.pos.z == chunk_z)
            .ok_or_else(|| format!("missing actual generated chunk ({chunk_x},{chunk_z})"))?;

        for local_x in 0..16 {
            let y_column = blocks
                .get(local_x)
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    format!(
                        "vanilla fixture chunk ({chunk_x},{chunk_z}) x={local_x} is not an array"
                    )
                })?;
            for local_z in 0..16 {
                let expected_height =
                    expected_world_surface_height(y_column, y_min, local_z, chunk_x, chunk_z)?;
                let actual_height = actual_world_surface_height(
                    actual,
                    chunk_x,
                    chunk_z,
                    local_x as i32,
                    local_z,
                    y_min,
                    y_max_exclusive,
                );
                if actual_height == expected_height {
                    matching_columns += 1;
                } else {
                    *mismatches
                        .entry((expected_height, actual_height))
                        .or_default() += 1;
                    let chunk = ChunkCoord {
                        x: chunk_x,
                        z: chunk_z,
                    };
                    *chunk_mismatches.entry((chunk.x, chunk.z)).or_default() += 1;
                    *delta_mismatches
                        .entry(actual_height - expected_height)
                        .or_default() += 1;
                    if samples.len() < 128 {
                        let expected_top_block = block_at_world_surface_height(
                            y_column,
                            y_min,
                            local_z,
                            expected_height,
                            chunk_x,
                            chunk_z,
                        )?;
                        let actual_top_block = actual_block_at_world_surface_height(
                            actual,
                            chunk_x,
                            chunk_z,
                            local_x as i32,
                            local_z,
                            actual_height,
                        );
                        samples.push(VanillaHeightmapMismatchSample {
                            chunk,
                            local_x,
                            local_z,
                            expected_height,
                            actual_height,
                            expected_top_block,
                            actual_top_block,
                        });
                    }
                }
                total_columns += 1;
            }
        }
    }

    VanillaHeightmapParityScore::from_counts_and_mismatches(
        matching_columns,
        total_columns,
        mismatches,
        samples,
        chunk_mismatches,
        delta_mismatches,
    )
}

pub fn vanilla_worldgen_biome_grid_parity_score(
    vanilla_fixture_json: &str,
    actual_chunks: &[LevelChunk],
) -> Result<VanillaBiomeGridParityScore, String> {
    let fixture: Value = serde_json::from_str(vanilla_fixture_json)
        .map_err(|err| format!("failed to parse vanilla worldgen block-array fixture: {err}"))?;
    let format = fixture
        .get("format")
        .and_then(Value::as_str)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing format".to_string())?;
    if format != "rustcraft-vanilla-worldgen-block-array-target-v1" {
        return Err(format!(
            "unsupported vanilla worldgen block-array fixture format: {format}"
        ));
    }

    let chunks = fixture
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing chunks".to_string())?;
    let mut matching_biomes = 0usize;
    let mut total_biomes = 0usize;
    let mut mismatches = BTreeMap::<(String, String), usize>::new();

    for expected_chunk in chunks {
        let dimension = expected_chunk
            .get("dimension")
            .and_then(Value::as_str)
            .unwrap_or("overworld");
        if dimension != "overworld" {
            return Err(format!(
                "unsupported vanilla worldgen block-array dimension: {dimension}"
            ));
        }
        let chunk_x = json_i32_field(expected_chunk, "chunkX")?;
        let chunk_z = json_i32_field(expected_chunk, "chunkZ")?;
        let quart_y_min = json_i32_field(expected_chunk, "quartYMin")?;
        let biomes = expected_chunk
            .get("biomes")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                format!("vanilla fixture chunk ({chunk_x},{chunk_z}) is missing biomes")
            })?;
        if biomes.len() != 4 {
            return Err(format!(
                "vanilla fixture chunk ({chunk_x},{chunk_z}) has {} quart-x columns, expected 4",
                biomes.len()
            ));
        }

        let actual = actual_chunks
            .iter()
            .find(|chunk| chunk.pos.x == chunk_x && chunk.pos.z == chunk_z)
            .ok_or_else(|| format!("missing actual generated chunk ({chunk_x},{chunk_z})"))?;

        for (quart_x, y_column) in biomes.iter().enumerate() {
            let y_column = y_column.as_array().ok_or_else(|| {
                format!(
                    "vanilla fixture chunk ({chunk_x},{chunk_z}) quart_x={quart_x} is not an array"
                )
            })?;
            for (quart_y_offset, z_column) in y_column.iter().enumerate() {
                let z_column = z_column.as_array().ok_or_else(|| {
                    format!(
                        "vanilla fixture chunk ({chunk_x},{chunk_z}) quart_x={quart_x} quart_y_offset={quart_y_offset} is not an array"
                    )
                })?;
                if z_column.len() != 4 {
                    return Err(format!(
                        "vanilla fixture chunk ({chunk_x},{chunk_z}) quart_x={quart_x} quart_y_offset={quart_y_offset} has {} quart-z entries, expected 4",
                        z_column.len()
                    ));
                }
                let quart_y = quart_y_min + quart_y_offset as i32;
                for (quart_z, expected) in z_column.iter().enumerate() {
                    let expected = expected
                        .as_str()
                        .ok_or_else(|| {
                            format!(
                                "vanilla fixture chunk ({chunk_x},{chunk_z}) quart_x={quart_x} quart_y_offset={quart_y_offset} quart_z={quart_z} is not a biome string"
                            )
                        })?
                        .to_string();
                    let actual = actual_chunk_biome(actual, quart_x, quart_y, quart_z)
                        .unwrap_or_else(|| "<missing>".to_string());
                    if actual == expected {
                        matching_biomes += 1;
                    } else {
                        *mismatches.entry((expected, actual)).or_default() += 1;
                    }
                    total_biomes += 1;
                }
            }
        }
    }

    VanillaBiomeGridParityScore::from_counts_and_mismatches(
        matching_biomes,
        total_biomes,
        mismatches,
    )
}

pub fn vanilla_worldgen_column_profile_parity_score(
    vanilla_fixture_json: &str,
    actual_chunks: &[LevelChunk],
) -> Result<VanillaColumnProfileParityScore, String> {
    let fixture: Value = serde_json::from_str(vanilla_fixture_json)
        .map_err(|err| format!("failed to parse vanilla worldgen block-array fixture: {err}"))?;
    let format = fixture
        .get("format")
        .and_then(Value::as_str)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing format".to_string())?;
    if format != "rustcraft-vanilla-worldgen-block-array-target-v1" {
        return Err(format!(
            "unsupported vanilla worldgen block-array fixture format: {format}"
        ));
    }

    let chunks = fixture
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing chunks".to_string())?;
    let mut matching_columns = 0usize;
    let mut total_columns = 0usize;
    let mut first_diff_y_mismatches = BTreeMap::<i32, usize>::new();
    let mut prefix_len_mismatches = BTreeMap::<usize, usize>::new();
    let mut surface_stack_mismatches = BTreeMap::<(String, String), usize>::new();
    let mut chunk_mismatches = BTreeMap::<(i32, i32), usize>::new();
    let mut samples = Vec::new();

    for expected_chunk in chunks {
        let dimension = expected_chunk
            .get("dimension")
            .and_then(Value::as_str)
            .unwrap_or("overworld");
        if dimension != "overworld" {
            return Err(format!(
                "unsupported vanilla worldgen block-array dimension: {dimension}"
            ));
        }
        let chunk_x = json_i32_field(expected_chunk, "chunkX")?;
        let chunk_z = json_i32_field(expected_chunk, "chunkZ")?;
        let y_min = json_i32_field(expected_chunk, "yMin")?;
        let y_max_exclusive = json_i32_field(expected_chunk, "yMaxExclusive")?;
        let blocks = expected_chunk
            .get("blocks")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                format!("vanilla fixture chunk ({chunk_x},{chunk_z}) is missing blocks")
            })?;
        if blocks.len() != 16 {
            return Err(format!(
                "vanilla fixture chunk ({chunk_x},{chunk_z}) has {} x columns, expected 16",
                blocks.len()
            ));
        }

        let actual = actual_chunks
            .iter()
            .find(|chunk| chunk.pos.x == chunk_x && chunk.pos.z == chunk_z)
            .ok_or_else(|| format!("missing actual generated chunk ({chunk_x},{chunk_z})"))?;

        for local_x in 0..16 {
            let y_column = blocks
                .get(local_x)
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    format!(
                        "vanilla fixture chunk ({chunk_x},{chunk_z}) x={local_x} is not an array"
                    )
                })?;
            for local_z in 0..16 {
                let mut first_diff = None;
                for world_y in y_min..y_max_exclusive {
                    let expected = expected_block_at_world_y(
                        y_column, y_min, local_z, world_y, chunk_x, chunk_z,
                    )?;
                    let actual_block = actual
                        .get_block_state(
                            chunk_x * 16 + local_x as i32,
                            world_y,
                            chunk_z * 16 + local_z as i32,
                        )
                        .unwrap_or_else(|| "minecraft:air".to_string());
                    if expected != actual_block {
                        first_diff = Some((world_y, expected, actual_block));
                        break;
                    }
                }

                match first_diff {
                    None => matching_columns += 1,
                    Some((first_diff_y, expected_at_first_diff, actual_at_first_diff)) => {
                        let matching_prefix_blocks = (first_diff_y - y_min) as usize;
                        let expected_surface_height = expected_world_surface_height(
                            y_column, y_min, local_z, chunk_x, chunk_z,
                        )?;
                        let actual_surface_height = actual_world_surface_height(
                            actual,
                            chunk_x,
                            chunk_z,
                            local_x as i32,
                            local_z,
                            y_min,
                            y_max_exclusive,
                        );
                        let expected_surface_stack = expected_surface_stack_signature(
                            y_column,
                            y_min,
                            local_z,
                            expected_surface_height,
                            chunk_x,
                            chunk_z,
                        )?;
                        let actual_surface_stack = actual_surface_stack_signature(
                            actual,
                            chunk_x,
                            chunk_z,
                            local_x as i32,
                            local_z,
                            actual_surface_height,
                        );
                        *first_diff_y_mismatches.entry(first_diff_y).or_default() += 1;
                        *prefix_len_mismatches
                            .entry(matching_prefix_blocks)
                            .or_default() += 1;
                        *surface_stack_mismatches
                            .entry((expected_surface_stack.clone(), actual_surface_stack.clone()))
                            .or_default() += 1;
                        *chunk_mismatches.entry((chunk_x, chunk_z)).or_default() += 1;
                        if samples.len() < 128 {
                            samples.push(VanillaColumnProfileMismatchSample {
                                chunk: ChunkCoord {
                                    x: chunk_x,
                                    z: chunk_z,
                                },
                                local_x,
                                local_z,
                                first_diff_y,
                                matching_prefix_blocks,
                                expected_at_first_diff,
                                actual_at_first_diff,
                                expected_surface_height,
                                actual_surface_height,
                                expected_surface_stack,
                                actual_surface_stack,
                            });
                        }
                    }
                }
                total_columns += 1;
            }
        }
    }

    VanillaColumnProfileParityScore::from_counts_and_mismatches(
        matching_columns,
        total_columns,
        first_diff_y_mismatches,
        prefix_len_mismatches,
        surface_stack_mismatches,
        samples,
        chunk_mismatches,
    )
}

pub fn vanilla_worldgen_block_array_parity_score_for_normal_overworld(
    vanilla_fixture_json: &str,
) -> Result<VanillaBlockArrayParityScore, String> {
    let actual_chunks = normal_overworld_actual_chunks_for_fixture(vanilla_fixture_json)?;

    vanilla_worldgen_block_array_parity_score(vanilla_fixture_json, &actual_chunks)
}

pub fn vanilla_worldgen_heightmap_parity_score_for_normal_overworld(
    vanilla_fixture_json: &str,
) -> Result<VanillaHeightmapParityScore, String> {
    let actual_chunks = normal_overworld_actual_chunks_for_fixture(vanilla_fixture_json)?;

    vanilla_worldgen_heightmap_parity_score(vanilla_fixture_json, &actual_chunks)
}

pub fn vanilla_worldgen_biome_grid_parity_score_for_normal_overworld(
    vanilla_fixture_json: &str,
) -> Result<VanillaBiomeGridParityScore, String> {
    let actual_chunks = normal_overworld_actual_chunks_for_fixture(vanilla_fixture_json)?;

    vanilla_worldgen_biome_grid_parity_score(vanilla_fixture_json, &actual_chunks)
}

pub fn vanilla_worldgen_column_profile_parity_score_for_normal_overworld(
    vanilla_fixture_json: &str,
) -> Result<VanillaColumnProfileParityScore, String> {
    let actual_chunks = normal_overworld_actual_chunks_for_fixture(vanilla_fixture_json)?;

    vanilla_worldgen_column_profile_parity_score(vanilla_fixture_json, &actual_chunks)
}

fn normal_overworld_actual_chunks_for_fixture(
    vanilla_fixture_json: &str,
) -> Result<Vec<LevelChunk>, String> {
    let fixture: Value = serde_json::from_str(vanilla_fixture_json)
        .map_err(|err| format!("failed to parse vanilla worldgen block-array fixture: {err}"))?;
    let seed = fixture
        .get("seed")
        .and_then(Value::as_str)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing seed".to_string())?
        .parse::<i64>()
        .map_err(|err| format!("vanilla worldgen block-array fixture has invalid seed: {err}"))?;
    let chunks = fixture
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing chunks".to_string())?;

    let mut actual_chunks = Vec::with_capacity(chunks.len());
    for chunk in chunks {
        let dimension = chunk
            .get("dimension")
            .and_then(Value::as_str)
            .unwrap_or("overworld");
        if dimension != "overworld" {
            return Err(format!(
                "unsupported vanilla worldgen block-array dimension: {dimension}"
            ));
        }
        let pos = crate::storage::region::ChunkPos {
            x: json_i32_field(chunk, "chunkX")?,
            z: json_i32_field(chunk, "chunkZ")?,
        };
        actual_chunks.push(
            crate::worldgen::generate_overworld_chunk_for_preset_with_mode(
                pos,
                "normal",
                crate::worldgen::LiveChunkGenerationMode::RealSurface,
                seed,
            )?,
        );
    }
    Ok(actual_chunks)
}
