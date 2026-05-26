#![allow(dead_code)]

use super::*;

pub fn vanilla_worldgen_tree_density_diagnostic_for_normal_overworld(
    vanilla_fixture_json: &str,
) -> Result<VanillaTreeDensityDiagnostic, String> {
    let fixture = parse_tree_density_fixture(vanilla_fixture_json)?;
    let seed = tree_density_fixture_seed(&fixture)?;
    let chunks = fixture
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing chunks".to_string())?;

    let mut diagnostic = empty_tree_density_diagnostic(chunks.len());
    for expected_chunk in chunks {
        let chunk_summary = tree_density_chunk_summary(expected_chunk, seed)?;
        accumulate_tree_density_chunk_summary(&mut diagnostic, &chunk_summary);
        diagnostic.chunks.push(chunk_summary);
    }

    Ok(diagnostic)
}

fn parse_tree_density_fixture(vanilla_fixture_json: &str) -> Result<Value, String> {
    serde_json::from_str(vanilla_fixture_json)
        .map_err(|err| format!("failed to parse vanilla worldgen block-array fixture: {err}"))
}

fn tree_density_fixture_seed(fixture: &Value) -> Result<i64, String> {
    fixture
        .get("seed")
        .and_then(Value::as_str)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing seed".to_string())?
        .parse::<i64>()
        .map_err(|err| format!("vanilla worldgen block-array fixture has invalid seed: {err}"))
}

fn empty_tree_density_diagnostic(chunk_count: usize) -> VanillaTreeDensityDiagnostic {
    VanillaTreeDensityDiagnostic {
        chunks: Vec::with_capacity(chunk_count),
        expected_logs: 0,
        actual_logs: 0,
        expected_leaves: 0,
        actual_leaves: 0,
        expected_tree_blocks: 0,
        actual_tree_blocks: 0,
        leaf_matches: 0,
        log_matches: 0,
        extra_actual_leaves: 0,
        missing_expected_leaves: 0,
        extra_actual_logs: 0,
        missing_expected_logs: 0,
        expected_log_columns: 0,
        actual_log_columns: 0,
        matching_log_columns: 0,
    }
}

fn tree_density_chunk_summary(
    expected_chunk: &Value,
    seed: i64,
) -> Result<VanillaTreeDensityChunk, String> {
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
        .ok_or_else(|| format!("vanilla fixture chunk ({chunk_x},{chunk_z}) is missing blocks"))?;
    let actual_chunk = crate::worldgen::generate_overworld_chunk_for_preset_with_mode(
        crate::storage::region::ChunkPos {
            x: chunk_x,
            z: chunk_z,
        },
        "normal",
        crate::worldgen::LiveChunkGenerationMode::RealSurface,
        seed,
    )?;

    let mut chunk_summary = VanillaTreeDensityChunk {
        chunk: ChunkCoord {
            x: chunk_x,
            z: chunk_z,
        },
        expected_logs: 0,
        actual_logs: 0,
        expected_leaves: 0,
        actual_leaves: 0,
        expected_tree_blocks: 0,
        actual_tree_blocks: 0,
        leaf_matches: 0,
        log_matches: 0,
        extra_actual_leaves: 0,
        missing_expected_leaves: 0,
        extra_actual_logs: 0,
        missing_expected_logs: 0,
        expected_log_columns: Vec::new(),
        actual_log_columns: Vec::new(),
        matching_log_columns: 0,
        expected_only_log_columns: Vec::new(),
        actual_only_log_columns: Vec::new(),
    };
    let mut expected_log_columns = BTreeMap::new();
    let mut actual_log_columns = BTreeMap::new();
    let block_scan = TreeDensityBlockScan {
        blocks,
        y_min,
        chunk_x,
        chunk_z,
        actual_chunk: &actual_chunk,
    };
    accumulate_tree_density_blocks(
        &mut chunk_summary,
        block_scan,
        &mut expected_log_columns,
        &mut actual_log_columns,
    )?;
    finish_tree_density_chunk_columns(&mut chunk_summary, expected_log_columns, actual_log_columns);
    Ok(chunk_summary)
}

struct TreeDensityBlockScan<'a> {
    blocks: &'a [Value],
    y_min: i32,
    chunk_x: i32,
    chunk_z: i32,
    actual_chunk: &'a LevelChunk,
}

fn accumulate_tree_density_blocks(
    chunk_summary: &mut VanillaTreeDensityChunk,
    scan: TreeDensityBlockScan<'_>,
    expected_log_columns: &mut BTreeMap<(i32, i32), VanillaTreeLogColumn>,
    actual_log_columns: &mut BTreeMap<(i32, i32), VanillaTreeLogColumn>,
) -> Result<(), String> {
    for (local_x, y_column) in scan.blocks.iter().enumerate() {
        let y_column = y_column.as_array().ok_or_else(|| {
            format!(
                "vanilla fixture chunk ({},{}) x={local_x} is not an array",
                scan.chunk_x, scan.chunk_z
            )
        })?;
        for (y_offset, z_column) in y_column.iter().enumerate() {
            let z_column = z_column.as_array().ok_or_else(|| {
                format!(
                    "vanilla fixture chunk ({},{}) x={local_x} y_offset={y_offset} is not an array",
                    scan.chunk_x, scan.chunk_z
                )
            })?;
            let world_y = scan.y_min + y_offset as i32;
            for (local_z, expected) in z_column.iter().enumerate() {
                let expected = expected
                    .as_str()
                    .ok_or_else(|| {
                        format!(
                            "vanilla fixture chunk ({},{}) x={local_x} y_offset={y_offset} z={local_z} is not a block string",
                            scan.chunk_x, scan.chunk_z
                        )
                    })
                    .map(vanilla_block_type)?;
                let actual = scan
                    .actual_chunk
                    .get_block_state(
                        scan.chunk_x * 16 + local_x as i32,
                        world_y,
                        scan.chunk_z * 16 + local_z as i32,
                    )
                    .unwrap_or_else(|| "minecraft:air".to_string());
                accumulate_tree_density_cell(chunk_summary, &expected, &actual);
                let world_x = scan.chunk_x * 16 + local_x as i32;
                let world_z = scan.chunk_z * 16 + local_z as i32;
                if is_tree_log_block(&expected) {
                    record_tree_log_column(
                        expected_log_columns,
                        world_x,
                        world_y,
                        world_z,
                        &expected,
                    );
                }
                if is_tree_log_block(&actual) {
                    record_tree_log_column(actual_log_columns, world_x, world_y, world_z, &actual);
                }
            }
        }
    }
    Ok(())
}

fn finish_tree_density_chunk_columns(
    chunk_summary: &mut VanillaTreeDensityChunk,
    expected_log_columns: BTreeMap<(i32, i32), VanillaTreeLogColumn>,
    actual_log_columns: BTreeMap<(i32, i32), VanillaTreeLogColumn>,
) {
    chunk_summary.expected_log_columns = finish_tree_log_columns(expected_log_columns);
    chunk_summary.actual_log_columns = finish_tree_log_columns(actual_log_columns);
    let expected_column_keys: BTreeSet<(i32, i32)> = chunk_summary
        .expected_log_columns
        .iter()
        .map(|column| (column.world_x, column.world_z))
        .collect();
    let actual_column_keys: BTreeSet<(i32, i32)> = chunk_summary
        .actual_log_columns
        .iter()
        .map(|column| (column.world_x, column.world_z))
        .collect();
    chunk_summary.matching_log_columns = expected_column_keys
        .intersection(&actual_column_keys)
        .count();
    chunk_summary.expected_only_log_columns = chunk_summary
        .expected_log_columns
        .iter()
        .filter(|column| !actual_column_keys.contains(&(column.world_x, column.world_z)))
        .cloned()
        .collect();
    chunk_summary.actual_only_log_columns = chunk_summary
        .actual_log_columns
        .iter()
        .filter(|column| !expected_column_keys.contains(&(column.world_x, column.world_z)))
        .cloned()
        .collect();
}

fn accumulate_tree_density_chunk_summary(
    diagnostic: &mut VanillaTreeDensityDiagnostic,
    chunk_summary: &VanillaTreeDensityChunk,
) {
    diagnostic.expected_logs += chunk_summary.expected_logs;
    diagnostic.actual_logs += chunk_summary.actual_logs;
    diagnostic.expected_leaves += chunk_summary.expected_leaves;
    diagnostic.actual_leaves += chunk_summary.actual_leaves;
    diagnostic.expected_tree_blocks += chunk_summary.expected_tree_blocks;
    diagnostic.actual_tree_blocks += chunk_summary.actual_tree_blocks;
    diagnostic.leaf_matches += chunk_summary.leaf_matches;
    diagnostic.log_matches += chunk_summary.log_matches;
    diagnostic.extra_actual_leaves += chunk_summary.extra_actual_leaves;
    diagnostic.missing_expected_leaves += chunk_summary.missing_expected_leaves;
    diagnostic.extra_actual_logs += chunk_summary.extra_actual_logs;
    diagnostic.missing_expected_logs += chunk_summary.missing_expected_logs;
    diagnostic.expected_log_columns += chunk_summary.expected_log_columns.len();
    diagnostic.actual_log_columns += chunk_summary.actual_log_columns.len();
    diagnostic.matching_log_columns += chunk_summary.matching_log_columns;
}

fn record_tree_log_column(
    columns: &mut BTreeMap<(i32, i32), VanillaTreeLogColumn>,
    world_x: i32,
    world_y: i32,
    world_z: i32,
    block: &str,
) {
    columns
        .entry((world_x, world_z))
        .and_modify(|column| {
            column.min_y = column.min_y.min(world_y);
            column.max_y = column.max_y.max(world_y);
            column.logs += 1;
        })
        .or_insert_with(|| VanillaTreeLogColumn {
            world_x,
            world_z,
            min_y: world_y,
            max_y: world_y,
            logs: 1,
            block: block.to_string(),
        });
}

fn finish_tree_log_columns(
    columns: BTreeMap<(i32, i32), VanillaTreeLogColumn>,
) -> Vec<VanillaTreeLogColumn> {
    columns.into_values().collect()
}

fn accumulate_tree_density_cell(
    summary: &mut VanillaTreeDensityChunk,
    expected: &str,
    actual: &str,
) {
    let expected_leaf = is_tree_leaf_block(expected);
    let actual_leaf = is_tree_leaf_block(actual);
    let expected_log = is_tree_log_block(expected);
    let actual_log = is_tree_log_block(actual);

    if expected_leaf {
        summary.expected_leaves += 1;
    }
    if actual_leaf {
        summary.actual_leaves += 1;
    }
    if expected_log {
        summary.expected_logs += 1;
    }
    if actual_log {
        summary.actual_logs += 1;
    }
    if expected_leaf || expected_log {
        summary.expected_tree_blocks += 1;
    }
    if actual_leaf || actual_log {
        summary.actual_tree_blocks += 1;
    }
    if expected_leaf && actual_leaf {
        summary.leaf_matches += 1;
    } else if !expected_leaf && actual_leaf {
        summary.extra_actual_leaves += 1;
    } else if expected_leaf && !actual_leaf {
        summary.missing_expected_leaves += 1;
    }
    if expected_log && actual_log {
        summary.log_matches += 1;
    } else if !expected_log && actual_log {
        summary.extra_actual_logs += 1;
    } else if expected_log && !actual_log {
        summary.missing_expected_logs += 1;
    }
}

fn is_tree_leaf_block(block: &str) -> bool {
    block
        .strip_prefix("minecraft:")
        .unwrap_or(block)
        .ends_with("_leaves")
}

fn is_tree_log_block(block: &str) -> bool {
    let block = block.strip_prefix("minecraft:").unwrap_or(block);
    block.ends_with("_log") || block.ends_with("_wood") || block.ends_with("_stem")
}

pub(super) fn json_i32_field(value: &Value, field: &str) -> Result<i32, String> {
    let number = value.get(field).and_then(Value::as_i64).ok_or_else(|| {
        format!("vanilla worldgen block-array fixture is missing integer {field}")
    })?;
    i32::try_from(number).map_err(|_| {
        format!("vanilla worldgen block-array fixture field {field} is out of i32 range")
    })
}

pub(super) fn vanilla_block_type(block_state: &str) -> String {
    block_state
        .split_once('[')
        .map(|(name, _)| name)
        .unwrap_or(block_state)
        .to_string()
}

pub(super) fn expected_world_surface_height(
    y_column: &[Value],
    y_min: i32,
    local_z: usize,
    chunk_x: i32,
    chunk_z: i32,
) -> Result<i32, String> {
    for (y_offset, z_column) in y_column.iter().enumerate().rev() {
        let z_column = z_column.as_array().ok_or_else(|| {
            format!(
                "vanilla fixture chunk ({chunk_x},{chunk_z}) y_offset={y_offset} is not an array"
            )
        })?;
        let block = z_column
            .get(local_z)
            .and_then(Value::as_str)
            .ok_or_else(|| {
                format!(
                    "vanilla fixture chunk ({chunk_x},{chunk_z}) y_offset={y_offset} z={local_z} is not a block string"
                )
            })
            .map(vanilla_block_type)?;
        if block != "minecraft:air" {
            return Ok(y_min + y_offset as i32 + 1);
        }
    }
    Ok(y_min)
}

pub(super) fn block_at_world_surface_height(
    y_column: &[Value],
    y_min: i32,
    local_z: usize,
    height: i32,
    chunk_x: i32,
    chunk_z: i32,
) -> Result<String, String> {
    if height <= y_min {
        return Ok("minecraft:air".to_string());
    }
    let y_offset = (height - 1 - y_min) as usize;
    let z_column = y_column
        .get(y_offset)
        .and_then(Value::as_array)
        .ok_or_else(|| {
            format!(
                "vanilla fixture chunk ({chunk_x},{chunk_z}) y_offset={y_offset} is not an array"
            )
        })?;
    z_column
        .get(local_z)
        .and_then(Value::as_str)
        .ok_or_else(|| {
            format!(
                "vanilla fixture chunk ({chunk_x},{chunk_z}) y_offset={y_offset} z={local_z} is not a block string"
            )
        })
        .map(vanilla_block_type)
}

pub(super) fn expected_block_at_world_y(
    y_column: &[Value],
    y_min: i32,
    local_z: usize,
    world_y: i32,
    chunk_x: i32,
    chunk_z: i32,
) -> Result<String, String> {
    let y_offset = (world_y - y_min) as usize;
    let z_column = y_column
        .get(y_offset)
        .and_then(Value::as_array)
        .ok_or_else(|| {
            format!(
                "vanilla fixture chunk ({chunk_x},{chunk_z}) y_offset={y_offset} is not an array"
            )
        })?;
    z_column
        .get(local_z)
        .and_then(Value::as_str)
        .ok_or_else(|| {
            format!(
                "vanilla fixture chunk ({chunk_x},{chunk_z}) y_offset={y_offset} z={local_z} is not a block string"
            )
        })
        .map(vanilla_block_type)
}

pub(super) fn expected_surface_stack_signature(
    y_column: &[Value],
    y_min: i32,
    local_z: usize,
    surface_height: i32,
    chunk_x: i32,
    chunk_z: i32,
) -> Result<String, String> {
    let mut entries = Vec::new();
    for relative_y in -4..=3 {
        let world_y = surface_height + relative_y;
        let block = if world_y < y_min || world_y >= y_min + y_column.len() as i32 {
            "minecraft:air".to_string()
        } else {
            expected_block_at_world_y(y_column, y_min, local_z, world_y, chunk_x, chunk_z)?
        };
        entries.push(format!("{relative_y:+}:{block}"));
    }
    Ok(entries.join("|"))
}

pub(super) fn actual_surface_stack_signature(
    chunk: &LevelChunk,
    chunk_x: i32,
    chunk_z: i32,
    local_x: i32,
    local_z: usize,
    surface_height: i32,
) -> String {
    let world_x = chunk_x * 16 + local_x;
    let world_z = chunk_z * 16 + local_z as i32;
    (-4..=3)
        .map(|relative_y| {
            let world_y = surface_height + relative_y;
            let block = chunk
                .get_block_state(world_x, world_y, world_z)
                .unwrap_or_else(|| "minecraft:air".to_string());
            format!("{relative_y:+}:{block}")
        })
        .collect::<Vec<_>>()
        .join("|")
}

pub(super) fn actual_world_surface_height(
    chunk: &LevelChunk,
    chunk_x: i32,
    chunk_z: i32,
    local_x: i32,
    local_z: usize,
    y_min: i32,
    y_max_exclusive: i32,
) -> i32 {
    let world_x = chunk_x * 16 + local_x;
    let world_z = chunk_z * 16 + local_z as i32;
    for world_y in (y_min..y_max_exclusive).rev() {
        let block = chunk
            .get_block_state(world_x, world_y, world_z)
            .unwrap_or_else(|| "minecraft:air".to_string());
        if block != "minecraft:air" {
            return world_y + 1;
        }
    }
    y_min
}

pub(super) fn actual_block_at_world_surface_height(
    chunk: &LevelChunk,
    chunk_x: i32,
    chunk_z: i32,
    local_x: i32,
    local_z: usize,
    height: i32,
) -> String {
    chunk
        .get_block_state(
            chunk_x * 16 + local_x,
            height - 1,
            chunk_z * 16 + local_z as i32,
        )
        .unwrap_or_else(|| "minecraft:air".to_string())
}

pub(super) fn actual_chunk_biome(
    chunk: &LevelChunk,
    quart_x: usize,
    quart_y: i32,
    quart_z: usize,
) -> Option<String> {
    if quart_x >= 4 || quart_z >= 4 {
        return None;
    }
    let section_y = quart_y.div_euclid(4) as i8;
    let local_y = quart_y.rem_euclid(4) as usize;
    let index = local_y * 16 + quart_z * 4 + quart_x;
    let section = chunk
        .sections
        .iter()
        .find(|section| section.y == section_y)?;
    let container = PalettedContainer::from_nbt(&section.biomes, BIOME_SECTION_VOLUME).ok()?;
    match container.get_entry(index)? {
        Tag::String(name) => Some(name.clone()),
        _ => None,
    }
}
