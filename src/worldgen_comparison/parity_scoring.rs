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
    let fixture = parse_vanilla_worldgen_block_array_fixture(vanilla_fixture_json)?;
    let mut tally = BlockArrayParityTally::default();

    for expected_chunk in vanilla_fixture_chunks(&fixture)? {
        score_block_array_chunk(expected_chunk, actual_chunks, &mut tally)?;
    }

    tally.into_score()
}

pub fn vanilla_worldgen_heightmap_parity_score(
    vanilla_fixture_json: &str,
    actual_chunks: &[LevelChunk],
) -> Result<VanillaHeightmapParityScore, String> {
    let fixture = parse_vanilla_worldgen_block_array_fixture(vanilla_fixture_json)?;
    let mut tally = HeightmapParityTally::default();

    for expected_chunk in vanilla_fixture_chunks(&fixture)? {
        score_heightmap_chunk(expected_chunk, actual_chunks, &mut tally)?;
    }

    tally.into_score()
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
    let fixture = parse_vanilla_worldgen_block_array_fixture(vanilla_fixture_json)?;
    let mut tally = ColumnProfileParityTally::default();

    for expected_chunk in vanilla_fixture_chunks(&fixture)? {
        score_column_profile_chunk(expected_chunk, actual_chunks, &mut tally)?;
    }

    tally.into_score()
}

fn parse_vanilla_worldgen_block_array_fixture(raw: &str) -> Result<Value, String> {
    let fixture: Value = serde_json::from_str(raw)
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
    Ok(fixture)
}

fn vanilla_fixture_chunks(fixture: &Value) -> Result<&[Value], String> {
    fixture
        .get("chunks")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing chunks".to_string())
}

fn validate_overworld_fixture_chunk(expected_chunk: &Value) -> Result<(), String> {
    let dimension = expected_chunk
        .get("dimension")
        .and_then(Value::as_str)
        .unwrap_or("overworld");
    if dimension == "overworld" {
        Ok(())
    } else {
        Err(format!(
            "unsupported vanilla worldgen block-array dimension: {dimension}"
        ))
    }
}

struct BlockArrayFixtureChunk<'a> {
    chunk: ChunkCoord,
    y_min: i32,
    y_max_exclusive: Option<i32>,
    blocks: &'a [Value],
}

fn block_array_fixture_chunk(
    expected_chunk: &Value,
    needs_y_max: bool,
) -> Result<BlockArrayFixtureChunk<'_>, String> {
    validate_overworld_fixture_chunk(expected_chunk)?;
    let chunk = ChunkCoord {
        x: json_i32_field(expected_chunk, "chunkX")?,
        z: json_i32_field(expected_chunk, "chunkZ")?,
    };
    let y_min = json_i32_field(expected_chunk, "yMin")?;
    let y_max_exclusive = if needs_y_max {
        Some(json_i32_field(expected_chunk, "yMaxExclusive")?)
    } else {
        None
    };
    let blocks = expected_chunk
        .get("blocks")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| {
            format!(
                "vanilla fixture chunk ({},{}) is missing blocks",
                chunk.x, chunk.z
            )
        })?;
    if blocks.len() != 16 {
        return Err(format!(
            "vanilla fixture chunk ({},{}) has {} x columns, expected 16",
            chunk.x,
            chunk.z,
            blocks.len()
        ));
    }
    Ok(BlockArrayFixtureChunk {
        chunk,
        y_min,
        y_max_exclusive,
        blocks,
    })
}

fn actual_chunk_for_fixture<'a>(
    actual_chunks: &'a [LevelChunk],
    fixture_chunk: &BlockArrayFixtureChunk<'_>,
) -> Result<&'a LevelChunk, String> {
    actual_chunks
        .iter()
        .find(|chunk| chunk.pos.x == fixture_chunk.chunk.x && chunk.pos.z == fixture_chunk.chunk.z)
        .ok_or_else(|| {
            format!(
                "missing actual generated chunk ({},{})",
                fixture_chunk.chunk.x, fixture_chunk.chunk.z
            )
        })
}

fn fixture_y_column<'a>(
    fixture_chunk: &BlockArrayFixtureChunk<'a>,
    local_x: usize,
) -> Result<&'a [Value], String> {
    fixture_chunk
        .blocks
        .get(local_x)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| {
            format!(
                "vanilla fixture chunk ({},{}) x={local_x} is not an array",
                fixture_chunk.chunk.x, fixture_chunk.chunk.z
            )
        })
}

fn fixture_z_column<'a>(
    fixture_chunk: &BlockArrayFixtureChunk<'_>,
    y_column: &'a [Value],
    local_x: usize,
    y_offset: usize,
) -> Result<&'a [Value], String> {
    let z_column = y_column
        .get(y_offset)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| {
            format!(
                "vanilla fixture chunk ({},{}) x={local_x} y_offset={y_offset} is not an array",
                fixture_chunk.chunk.x, fixture_chunk.chunk.z
            )
        })?;
    if z_column.len() != 16 {
        return Err(format!(
            "vanilla fixture chunk ({},{}) x={local_x} y_offset={y_offset} has {} z entries, expected 16",
            fixture_chunk.chunk.x,
            fixture_chunk.chunk.z,
            z_column.len()
        ));
    }
    Ok(z_column)
}

#[derive(Default)]
struct BlockArrayParityTally {
    matching_blocks: usize,
    total_blocks: usize,
    mismatches: BTreeMap<(String, String), usize>,
    samples: Vec<VanillaBlockArrayMismatchSample>,
    chunk_mismatches: BTreeMap<(i32, i32), usize>,
    y_band_mismatches: BTreeMap<i32, usize>,
}

impl BlockArrayParityTally {
    fn into_score(self) -> Result<VanillaBlockArrayParityScore, String> {
        VanillaBlockArrayParityScore::from_counts_and_mismatches(
            self.matching_blocks,
            self.total_blocks,
            self.mismatches,
            self.samples,
            self.chunk_mismatches,
            self.y_band_mismatches,
        )
    }

    fn record(&mut self, location: BlockArrayCell, expected: String, actual: String) {
        if actual == expected {
            self.matching_blocks += 1;
        } else {
            *self
                .mismatches
                .entry((expected.clone(), actual.clone()))
                .or_default() += 1;
            *self
                .chunk_mismatches
                .entry((location.chunk.x, location.chunk.z))
                .or_default() += 1;
            *self
                .y_band_mismatches
                .entry(location.y.div_euclid(16) * 16)
                .or_default() += 1;
            if self.samples.len() < 128 {
                self.samples.push(VanillaBlockArrayMismatchSample {
                    chunk: location.chunk,
                    local_x: location.local_x,
                    y: location.y,
                    local_z: location.local_z,
                    expected,
                    actual,
                });
            }
        }
        self.total_blocks += 1;
    }
}

#[derive(Clone, Copy)]
struct BlockArrayCell {
    chunk: ChunkCoord,
    local_x: usize,
    y: i32,
    local_z: usize,
}

fn score_block_array_chunk(
    expected_chunk: &Value,
    actual_chunks: &[LevelChunk],
    tally: &mut BlockArrayParityTally,
) -> Result<(), String> {
    let fixture_chunk = block_array_fixture_chunk(expected_chunk, false)?;
    let actual_chunk = actual_chunk_for_fixture(actual_chunks, &fixture_chunk)?;
    for (local_x, y_column) in fixture_chunk.blocks.iter().enumerate() {
        let y_column = y_column.as_array().map(Vec::as_slice).ok_or_else(|| {
            format!(
                "vanilla fixture chunk ({},{}) x={local_x} is not an array",
                fixture_chunk.chunk.x, fixture_chunk.chunk.z
            )
        })?;
        for (y_offset, _) in y_column.iter().enumerate() {
            score_block_array_y_column(
                &fixture_chunk,
                actual_chunk,
                tally,
                local_x,
                y_offset,
                y_column,
            )?;
        }
    }
    Ok(())
}

fn score_block_array_y_column(
    fixture_chunk: &BlockArrayFixtureChunk<'_>,
    actual_chunk: &LevelChunk,
    tally: &mut BlockArrayParityTally,
    local_x: usize,
    y_offset: usize,
    y_column: &[Value],
) -> Result<(), String> {
    let z_column = fixture_z_column(fixture_chunk, y_column, local_x, y_offset)?;
    let world_y = fixture_chunk.y_min + y_offset as i32;
    for (local_z, expected) in z_column.iter().enumerate() {
        let expected = expected
            .as_str()
            .ok_or_else(|| {
                format!(
                    "vanilla fixture chunk ({},{}) x={local_x} y_offset={y_offset} z={local_z} is not a block string",
                    fixture_chunk.chunk.x, fixture_chunk.chunk.z
                )
            })
            .map(vanilla_block_type)?;
        let actual = actual_chunk
            .get_block_state(
                fixture_chunk.chunk.x * 16 + local_x as i32,
                world_y,
                fixture_chunk.chunk.z * 16 + local_z as i32,
            )
            .map(|block| vanilla_block_type(&block).to_string())
            .unwrap_or_else(|| "minecraft:air".to_string());
        tally.record(
            BlockArrayCell {
                chunk: fixture_chunk.chunk,
                local_x,
                y: world_y,
                local_z,
            },
            expected,
            actual,
        );
    }
    Ok(())
}

#[derive(Default)]
struct HeightmapParityTally {
    matching_columns: usize,
    total_columns: usize,
    mismatches: BTreeMap<(i32, i32), usize>,
    samples: Vec<VanillaHeightmapMismatchSample>,
    chunk_mismatches: BTreeMap<(i32, i32), usize>,
    delta_mismatches: BTreeMap<i32, usize>,
}

impl HeightmapParityTally {
    fn into_score(self) -> Result<VanillaHeightmapParityScore, String> {
        VanillaHeightmapParityScore::from_counts_and_mismatches(
            self.matching_columns,
            self.total_columns,
            self.mismatches,
            self.samples,
            self.chunk_mismatches,
            self.delta_mismatches,
        )
    }
}

fn score_heightmap_chunk(
    expected_chunk: &Value,
    actual_chunks: &[LevelChunk],
    tally: &mut HeightmapParityTally,
) -> Result<(), String> {
    let fixture_chunk = block_array_fixture_chunk(expected_chunk, true)?;
    let actual_chunk = actual_chunk_for_fixture(actual_chunks, &fixture_chunk)?;
    for local_x in 0..16 {
        let y_column = fixture_y_column(&fixture_chunk, local_x)?;
        for local_z in 0..16 {
            score_heightmap_column(
                &fixture_chunk,
                actual_chunk,
                tally,
                local_x,
                local_z,
                y_column,
            )?;
        }
    }
    Ok(())
}

fn score_heightmap_column(
    fixture_chunk: &BlockArrayFixtureChunk<'_>,
    actual_chunk: &LevelChunk,
    tally: &mut HeightmapParityTally,
    local_x: usize,
    local_z: usize,
    y_column: &[Value],
) -> Result<(), String> {
    let expected_height = expected_world_surface_height(
        y_column,
        fixture_chunk.y_min,
        local_z,
        fixture_chunk.chunk.x,
        fixture_chunk.chunk.z,
    )?;
    let actual_height = actual_world_surface_height(
        actual_chunk,
        fixture_chunk.chunk.x,
        fixture_chunk.chunk.z,
        local_x as i32,
        local_z,
        fixture_chunk.y_min,
        fixture_chunk.y_max_exclusive.unwrap_or(fixture_chunk.y_min),
    );
    if actual_height == expected_height {
        tally.matching_columns += 1;
    } else {
        record_heightmap_mismatch(
            fixture_chunk,
            actual_chunk,
            tally,
            HeightmapColumn {
                local_x,
                local_z,
                expected_height,
                actual_height,
            },
            y_column,
        )?;
    }
    tally.total_columns += 1;
    Ok(())
}

struct HeightmapColumn {
    local_x: usize,
    local_z: usize,
    expected_height: i32,
    actual_height: i32,
}

fn record_heightmap_mismatch(
    fixture_chunk: &BlockArrayFixtureChunk<'_>,
    actual_chunk: &LevelChunk,
    tally: &mut HeightmapParityTally,
    column: HeightmapColumn,
    y_column: &[Value],
) -> Result<(), String> {
    *tally
        .mismatches
        .entry((column.expected_height, column.actual_height))
        .or_default() += 1;
    *tally
        .chunk_mismatches
        .entry((fixture_chunk.chunk.x, fixture_chunk.chunk.z))
        .or_default() += 1;
    *tally
        .delta_mismatches
        .entry(column.actual_height - column.expected_height)
        .or_default() += 1;
    if tally.samples.len() < 128 {
        tally.samples.push(VanillaHeightmapMismatchSample {
            chunk: fixture_chunk.chunk,
            local_x: column.local_x,
            local_z: column.local_z,
            expected_height: column.expected_height,
            actual_height: column.actual_height,
            expected_top_block: block_at_world_surface_height(
                y_column,
                fixture_chunk.y_min,
                column.local_z,
                column.expected_height,
                fixture_chunk.chunk.x,
                fixture_chunk.chunk.z,
            )?,
            actual_top_block: actual_block_at_world_surface_height(
                actual_chunk,
                fixture_chunk.chunk.x,
                fixture_chunk.chunk.z,
                column.local_x as i32,
                column.local_z,
                column.actual_height,
            ),
        });
    }
    Ok(())
}

#[derive(Default)]
struct ColumnProfileParityTally {
    matching_columns: usize,
    total_columns: usize,
    first_diff_y_mismatches: BTreeMap<i32, usize>,
    prefix_len_mismatches: BTreeMap<usize, usize>,
    surface_stack_mismatches: BTreeMap<(String, String), usize>,
    samples: Vec<VanillaColumnProfileMismatchSample>,
    chunk_mismatches: BTreeMap<(i32, i32), usize>,
}

impl ColumnProfileParityTally {
    fn into_score(self) -> Result<VanillaColumnProfileParityScore, String> {
        VanillaColumnProfileParityScore::from_counts_and_mismatches(
            self.matching_columns,
            self.total_columns,
            self.first_diff_y_mismatches,
            self.prefix_len_mismatches,
            self.surface_stack_mismatches,
            self.samples,
            self.chunk_mismatches,
        )
    }
}

fn score_column_profile_chunk(
    expected_chunk: &Value,
    actual_chunks: &[LevelChunk],
    tally: &mut ColumnProfileParityTally,
) -> Result<(), String> {
    let fixture_chunk = block_array_fixture_chunk(expected_chunk, true)?;
    let actual_chunk = actual_chunk_for_fixture(actual_chunks, &fixture_chunk)?;
    for local_x in 0..16 {
        let y_column = fixture_y_column(&fixture_chunk, local_x)?;
        for local_z in 0..16 {
            score_column_profile_column(
                &fixture_chunk,
                actual_chunk,
                tally,
                local_x,
                local_z,
                y_column,
            )?;
        }
    }
    Ok(())
}

fn score_column_profile_column(
    fixture_chunk: &BlockArrayFixtureChunk<'_>,
    actual_chunk: &LevelChunk,
    tally: &mut ColumnProfileParityTally,
    local_x: usize,
    local_z: usize,
    y_column: &[Value],
) -> Result<(), String> {
    let y_max_exclusive = fixture_chunk.y_max_exclusive.unwrap_or(fixture_chunk.y_min);
    let first_diff = first_column_profile_diff(
        fixture_chunk,
        actual_chunk,
        local_x,
        local_z,
        y_column,
        y_max_exclusive,
    )?;
    match first_diff {
        None => tally.matching_columns += 1,
        Some(diff) => record_column_profile_mismatch(
            fixture_chunk,
            actual_chunk,
            tally,
            ColumnProfileLocation { local_x, local_z },
            y_column,
            diff,
        )?,
    }
    tally.total_columns += 1;
    Ok(())
}

struct ColumnProfileDiff {
    first_diff_y: i32,
    expected_at_first_diff: String,
    actual_at_first_diff: String,
}

struct ColumnProfileLocation {
    local_x: usize,
    local_z: usize,
}

fn first_column_profile_diff(
    fixture_chunk: &BlockArrayFixtureChunk<'_>,
    actual_chunk: &LevelChunk,
    local_x: usize,
    local_z: usize,
    y_column: &[Value],
    y_max_exclusive: i32,
) -> Result<Option<ColumnProfileDiff>, String> {
    for world_y in fixture_chunk.y_min..y_max_exclusive {
        let expected = expected_block_at_world_y(
            y_column,
            fixture_chunk.y_min,
            local_z,
            world_y,
            fixture_chunk.chunk.x,
            fixture_chunk.chunk.z,
        )?;
        let actual_block = actual_chunk
            .get_block_state(
                fixture_chunk.chunk.x * 16 + local_x as i32,
                world_y,
                fixture_chunk.chunk.z * 16 + local_z as i32,
            )
            .unwrap_or_else(|| "minecraft:air".to_string());
        if expected != actual_block {
            return Ok(Some(ColumnProfileDiff {
                first_diff_y: world_y,
                expected_at_first_diff: expected,
                actual_at_first_diff: actual_block,
            }));
        }
    }
    Ok(None)
}

fn record_column_profile_mismatch(
    fixture_chunk: &BlockArrayFixtureChunk<'_>,
    actual_chunk: &LevelChunk,
    tally: &mut ColumnProfileParityTally,
    location: ColumnProfileLocation,
    y_column: &[Value],
    diff: ColumnProfileDiff,
) -> Result<(), String> {
    let matching_prefix_blocks = (diff.first_diff_y - fixture_chunk.y_min) as usize;
    let expected_surface_height = expected_world_surface_height(
        y_column,
        fixture_chunk.y_min,
        location.local_z,
        fixture_chunk.chunk.x,
        fixture_chunk.chunk.z,
    )?;
    let actual_surface_height = actual_world_surface_height(
        actual_chunk,
        fixture_chunk.chunk.x,
        fixture_chunk.chunk.z,
        location.local_x as i32,
        location.local_z,
        fixture_chunk.y_min,
        fixture_chunk.y_max_exclusive.unwrap_or(fixture_chunk.y_min),
    );
    let expected_surface_stack = expected_surface_stack_signature(
        y_column,
        fixture_chunk.y_min,
        location.local_z,
        expected_surface_height,
        fixture_chunk.chunk.x,
        fixture_chunk.chunk.z,
    )?;
    let actual_surface_stack = actual_surface_stack_signature(
        actual_chunk,
        fixture_chunk.chunk.x,
        fixture_chunk.chunk.z,
        location.local_x as i32,
        location.local_z,
        actual_surface_height,
    );

    *tally
        .first_diff_y_mismatches
        .entry(diff.first_diff_y)
        .or_default() += 1;
    *tally
        .prefix_len_mismatches
        .entry(matching_prefix_blocks)
        .or_default() += 1;
    *tally
        .surface_stack_mismatches
        .entry((expected_surface_stack.clone(), actual_surface_stack.clone()))
        .or_default() += 1;
    *tally
        .chunk_mismatches
        .entry((fixture_chunk.chunk.x, fixture_chunk.chunk.z))
        .or_default() += 1;
    if tally.samples.len() < 128 {
        tally.samples.push(VanillaColumnProfileMismatchSample {
            chunk: fixture_chunk.chunk,
            local_x: location.local_x,
            local_z: location.local_z,
            first_diff_y: diff.first_diff_y,
            matching_prefix_blocks,
            expected_at_first_diff: diff.expected_at_first_diff,
            actual_at_first_diff: diff.actual_at_first_diff,
            expected_surface_height,
            actual_surface_height,
            expected_surface_stack,
            actual_surface_stack,
        });
    }
    Ok(())
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
