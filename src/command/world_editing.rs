use super::*;

pub(super) fn clone_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let parsed = parse_clone_command(state, parts)?;
    if state.debug_world {
        return Err(CommandError::CloneFailed);
    }
    let source_box = BoundingBox::from_corners(parsed.begin, parsed.end);
    let area = source_box.volume();
    if area > i64::from(state.max_block_modifications) {
        return Err(CommandError::CloneTooBig);
    }
    let target_end = parsed.destination.offset(source_box.size_minus_one());
    let target_box = BoundingBox::from_corners(parsed.destination, target_end);
    if parsed.mode == CloneMode::Normal
        && parsed.source_dimension == parsed.target_dimension
        && source_box.intersects(&target_box)
    {
        return Err(CommandError::CloneOverlap);
    }

    let offset = BlockPos {
        x: parsed.destination.x - source_box.min.x,
        y: parsed.destination.y - source_box.min.y,
        z: parsed.destination.z - source_box.min.z,
    };
    let mut copies = Vec::new();
    for source_pos in source_box.positions() {
        let source_block = block_at(state, &parsed.source_dimension, source_pos);
        if !clone_filter_matches(
            parsed.filter,
            parsed.filtered_block.as_deref(),
            &source_block,
        ) {
            continue;
        }
        copies.push((
            source_pos,
            BlockPos {
                x: source_pos.x + offset.x,
                y: source_pos.y + offset.y,
                z: source_pos.z + offset.z,
            },
            source_block,
        ));
    }

    if copies.is_empty() {
        return Err(CommandError::CloneFailed);
    }

    if parsed.mode == CloneMode::Move {
        for (source_pos, _, _) in &copies {
            set_block_in_dimension(
                state,
                &parsed.source_dimension,
                *source_pos,
                "minecraft:air".to_string(),
            );
        }
    }
    for (_, destination, block) in &copies {
        set_block_in_dimension(state, &parsed.target_dimension, *destination, block.clone());
    }

    let count = copies.len() as i32;
    state.clone_events.push(CloneEvent {
        source_dimension: parsed.source_dimension,
        target_dimension: parsed.target_dimension,
        begin: parsed.begin,
        end: parsed.end,
        destination: parsed.destination,
        filter: parsed.filter,
        mode: parsed.mode,
        strict: parsed.strict,
        count,
    });
    Ok(CommandResult {
        success_count: count,
        feedback_key: "commands.clone.success",
        broadcast_to_admins: true,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParsedCloneCommand {
    source_dimension: String,
    target_dimension: String,
    begin: BlockPos,
    end: BlockPos,
    destination: BlockPos,
    filter: CloneFilter,
    filtered_block: Option<String>,
    mode: CloneMode,
    strict: bool,
}

pub(super) fn parse_clone_command(
    state: &ServerCommandState,
    parts: &[&str],
) -> Result<ParsedCloneCommand, CommandError> {
    let mut index = 1;
    let source_dimension = if parts.get(index) == Some(&"from") {
        index += 1;
        let dimension =
            parse_resource_identifier(parts.get(index).ok_or(CommandError::InvalidSyntax)?)?;
        index += 1;
        dimension
    } else {
        state.command_source_dimension.clone()
    };
    let begin = parse_block_pos(
        parts.get(index).ok_or(CommandError::InvalidSyntax)?,
        parts.get(index + 1).ok_or(CommandError::InvalidSyntax)?,
        parts.get(index + 2).ok_or(CommandError::InvalidSyntax)?,
    )?;
    index += 3;
    let end = parse_block_pos(
        parts.get(index).ok_or(CommandError::InvalidSyntax)?,
        parts.get(index + 1).ok_or(CommandError::InvalidSyntax)?,
        parts.get(index + 2).ok_or(CommandError::InvalidSyntax)?,
    )?;
    index += 3;
    let target_dimension = if parts.get(index) == Some(&"to") {
        index += 1;
        let dimension =
            parse_resource_identifier(parts.get(index).ok_or(CommandError::InvalidSyntax)?)?;
        index += 1;
        dimension
    } else {
        state.command_source_dimension.clone()
    };
    let strict = if parts.get(index) == Some(&"strict") {
        index += 1;
        true
    } else {
        false
    };
    let destination = parse_block_pos(
        parts.get(index).ok_or(CommandError::InvalidSyntax)?,
        parts.get(index + 1).ok_or(CommandError::InvalidSyntax)?,
        parts.get(index + 2).ok_or(CommandError::InvalidSyntax)?,
    )?;
    index += 3;
    let mut filter = CloneFilter::Replace;
    let mut filtered_block = None;
    let mut mode = CloneMode::Normal;
    if let Some(next) = parts.get(index) {
        match *next {
            "replace" => {
                filter = CloneFilter::Replace;
                index += 1;
            }
            "masked" => {
                filter = CloneFilter::Masked;
                index += 1;
            }
            "filtered" => {
                filter = CloneFilter::Filtered;
                filtered_block = Some(parse_resource_identifier(
                    parts.get(index + 1).ok_or(CommandError::InvalidSyntax)?,
                )?);
                index += 2;
            }
            "force" | "move" | "normal" => {}
            _ => return Err(CommandError::InvalidSyntax),
        }
    }
    if let Some(next) = parts.get(index) {
        mode = match *next {
            "force" => CloneMode::Force,
            "move" => CloneMode::Move,
            "normal" => CloneMode::Normal,
            _ => return Err(CommandError::InvalidSyntax),
        };
        index += 1;
    }
    if index != parts.len() {
        return Err(CommandError::InvalidSyntax);
    }
    Ok(ParsedCloneCommand {
        source_dimension,
        target_dimension,
        begin,
        end,
        destination,
        filter,
        filtered_block,
        mode,
        strict,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct BoundingBox {
    min: BlockPos,
    max: BlockPos,
}

impl BoundingBox {
    pub(super) fn from_corners(a: BlockPos, b: BlockPos) -> Self {
        Self {
            min: BlockPos {
                x: a.x.min(b.x),
                y: a.y.min(b.y),
                z: a.z.min(b.z),
            },
            max: BlockPos {
                x: a.x.max(b.x),
                y: a.y.max(b.y),
                z: a.z.max(b.z),
            },
        }
    }

    pub(super) fn size_minus_one(self) -> BlockPos {
        BlockPos {
            x: self.max.x - self.min.x,
            y: self.max.y - self.min.y,
            z: self.max.z - self.min.z,
        }
    }

    pub(super) fn volume(self) -> i64 {
        i64::from(self.max.x - self.min.x + 1)
            * i64::from(self.max.y - self.min.y + 1)
            * i64::from(self.max.z - self.min.z + 1)
    }

    pub(super) fn intersects(&self, other: &Self) -> bool {
        self.max.x >= other.min.x
            && self.min.x <= other.max.x
            && self.max.y >= other.min.y
            && self.min.y <= other.max.y
            && self.max.z >= other.min.z
            && self.min.z <= other.max.z
    }

    pub(super) fn positions(self) -> Vec<BlockPos> {
        let mut output = Vec::new();
        for z in self.min.z..=self.max.z {
            for y in self.min.y..=self.max.y {
                for x in self.min.x..=self.max.x {
                    output.push(BlockPos { x, y, z });
                }
            }
        }
        output
    }
}

impl BlockPos {
    pub(super) fn offset(self, offset: BlockPos) -> Self {
        Self {
            x: self.x + offset.x,
            y: self.y + offset.y,
            z: self.z + offset.z,
        }
    }
}

pub(super) fn block_at(state: &ServerCommandState, dimension: &str, position: BlockPos) -> String {
    state
        .blocks
        .iter()
        .find(|entry| entry.dimension == dimension && entry.position == position)
        .map(|entry| entry.block.clone())
        .unwrap_or_else(|| "minecraft:air".to_string())
}

pub(super) fn set_block_in_dimension(
    state: &mut ServerCommandState,
    dimension: &str,
    position: BlockPos,
    block: String,
) {
    if let Some(entry) = state
        .blocks
        .iter_mut()
        .find(|entry| entry.dimension == dimension && entry.position == position)
    {
        entry.block = block;
    } else {
        state.blocks.push(BlockStateEntry {
            dimension: dimension.to_string(),
            position,
            block,
        });
    }
}

pub(super) fn clone_filter_matches(
    filter: CloneFilter,
    filtered_block: Option<&str>,
    block: &str,
) -> bool {
    match filter {
        CloneFilter::Replace => true,
        CloneFilter::Masked => block != "minecraft:air",
        CloneFilter::Filtered => filtered_block.is_some_and(|filtered| filtered == block),
    }
}

pub(super) fn fill_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if parts.len() < 8 || parts[0] != "fill" {
        return Err(CommandError::InvalidSyntax);
    }
    if state.debug_world {
        return Err(CommandError::FillFailed);
    }
    let begin = parse_block_pos(parts[1], parts[2], parts[3])?;
    let end = parse_block_pos(parts[4], parts[5], parts[6])?;
    let block = parse_resource_identifier(parts[7])?;
    let options = parse_fill_options(&parts[8..])?;
    let region = BoundingBox::from_corners(begin, end);
    if region.volume() > i64::from(state.max_block_modifications) {
        return Err(CommandError::FillTooBig);
    }

    let dimension = state.command_source_dimension.clone();
    let count = apply_fill_region(state, &dimension, region, &block, &options)?;
    state.fill_events.push(FillEvent {
        dimension,
        begin,
        end,
        block,
        mode: options.mode,
        filter: options.filter,
        strict: options.strict,
        count,
    });
    Ok(CommandResult {
        success_count: count,
        feedback_key: "commands.fill.success",
        broadcast_to_admins: true,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FillOptions {
    mode: FillMode,
    filter: Option<String>,
    strict: bool,
}

fn parse_fill_options(parts: &[&str]) -> Result<FillOptions, CommandError> {
    let mut mode = FillMode::Replace;
    let mut strict = false;
    let mut filter = None;
    let mut index = 0;
    match parts.first().copied() {
        None => {}
        Some("replace") => {
            if let Some(predicate) = parts.get(1) {
                filter = Some(parse_resource_identifier(predicate)?);
                index = 2;
            } else {
                index = 1;
            }
        }
        Some("outline") => {
            mode = FillMode::Outline;
            index = 1;
        }
        Some("hollow") => {
            mode = FillMode::Hollow;
            index = 1;
        }
        Some("destroy") => {
            mode = FillMode::Destroy;
            index = 1;
        }
        Some("keep") => {
            mode = FillMode::Keep;
            index = 1;
        }
        Some("strict") => {
            strict = true;
            index = 1;
        }
        Some(_) => return Err(CommandError::InvalidSyntax),
    }
    if index < parts.len() {
        if filter.is_none() || mode != FillMode::Replace || strict {
            return Err(CommandError::InvalidSyntax);
        }
        match parts[index] {
            "outline" => mode = FillMode::Outline,
            "hollow" => mode = FillMode::Hollow,
            "destroy" => mode = FillMode::Destroy,
            "strict" => strict = true,
            _ => return Err(CommandError::InvalidSyntax),
        }
        index += 1;
    }
    if index != parts.len() {
        return Err(CommandError::InvalidSyntax);
    }
    Ok(FillOptions {
        mode,
        filter,
        strict,
    })
}

fn apply_fill_region(
    state: &mut ServerCommandState,
    dimension: &str,
    region: BoundingBox,
    block: &str,
    options: &FillOptions,
) -> Result<i32, CommandError> {
    let mut count = 0;
    for position in region.positions() {
        let old_block = block_at(state, dimension, position);
        if options
            .filter
            .as_deref()
            .is_some_and(|predicate| predicate != old_block)
        {
            continue;
        }
        let replacement = match options.mode {
            FillMode::Replace => Some(block.to_string()),
            FillMode::Keep if old_block == "minecraft:air" => Some(block.to_string()),
            FillMode::Keep => None,
            FillMode::Destroy => Some(block.to_string()),
            FillMode::Outline if is_boundary(region, position) => Some(block.to_string()),
            FillMode::Outline => None,
            FillMode::Hollow if is_boundary(region, position) => Some(block.to_string()),
            FillMode::Hollow => Some("minecraft:air".to_string()),
        };
        if let Some(replacement) = replacement {
            if replacement != old_block || options.mode == FillMode::Destroy {
                set_block_in_dimension(state, dimension, position, replacement);
                count += 1;
            }
        }
    }
    if count == 0 {
        return Err(CommandError::FillFailed);
    }
    Ok(count)
}

pub(super) fn fill_biome_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if parts.len() != 8 && parts.len() != 10 {
        return Err(CommandError::InvalidSyntax);
    }
    let begin = quantize_biome_pos(parse_block_pos(parts[1], parts[2], parts[3])?);
    let end = quantize_biome_pos(parse_block_pos(parts[4], parts[5], parts[6])?);
    let biome = parse_resource_identifier(parts[7])?;
    let filter = match parts.get(8).copied() {
        None => None,
        Some("replace") => Some(parse_resource_identifier(parts[9])?),
        Some(_) => return Err(CommandError::InvalidSyntax),
    };
    let region = BoundingBox::from_corners(begin, end);
    if region.volume() > i64::from(state.max_block_modifications) {
        return Err(CommandError::FillBiomeTooBig);
    }

    let dimension = state.command_source_dimension.clone();
    let mut count = 0;
    for position in region.positions() {
        if position.x % 4 != 0 || position.y % 4 != 0 || position.z % 4 != 0 {
            continue;
        }
        let current = biome_at(state, &dimension, position);
        if filter
            .as_deref()
            .is_some_and(|predicate| predicate != current)
        {
            continue;
        }
        count += 1;
        if current != biome {
            set_biome_in_dimension(state, &dimension, position, biome.clone());
        }
    }
    state.fill_biome_events.push(FillBiomeEvent {
        dimension,
        begin,
        end,
        biome,
        filter,
        count,
    });
    Ok(CommandResult {
        success_count: count,
        feedback_key: "commands.fillbiome.success.count",
        broadcast_to_admins: true,
    })
}

/// VibeCraft-only debugging command.
///
/// Intentional Java parity divergence: vanilla 26.1.2 exposes `/fillbiome` but not
/// a root `/biome` command. This reports the source player's current biome so we
/// can debug generated terrain and client state without leaving the game.
pub(super) fn biome_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if parts.len() != 1 {
        return Err(CommandError::InvalidSyntax);
    }
    let _ = debug_biome_at_command_source(state);
    Ok(CommandResult {
        success_count: 1,
        feedback_key: "commands.vibecraft.debug.biome",
        broadcast_to_admins: false,
    })
}

/// Returns the command source's current biome for VibeCraft's `/biome` debug command.
///
/// Intentional Java parity divergence: this function supports a non-vanilla command,
/// but it still uses Java's biome quart-coordinate convention for the lookup.
pub fn debug_biome_at_command_source(state: &ServerCommandState) -> String {
    let block_pos = block_pos_containing(state.command_source_position);
    let biome_pos = quantize_biome_pos(block_pos);
    biome_at_explicit_or_generated(state, &state.command_source_dimension, biome_pos)
}

pub(super) fn is_boundary(region: BoundingBox, position: BlockPos) -> bool {
    position.x == region.min.x
        || position.x == region.max.x
        || position.y == region.min.y
        || position.y == region.max.y
        || position.z == region.min.z
        || position.z == region.max.z
}

pub(super) fn quantize_biome_pos(position: BlockPos) -> BlockPos {
    BlockPos {
        x: position.x.div_euclid(4) * 4,
        y: position.y.div_euclid(4) * 4,
        z: position.z.div_euclid(4) * 4,
    }
}

pub(super) fn biome_at(state: &ServerCommandState, dimension: &str, position: BlockPos) -> String {
    state
        .biomes
        .iter()
        .find(|entry| entry.dimension == dimension && entry.position == position)
        .map(|entry| entry.biome.clone())
        .unwrap_or_else(|| "minecraft:plains".to_string())
}

fn biome_at_explicit_or_generated(
    state: &ServerCommandState,
    dimension: &str,
    position: BlockPos,
) -> String {
    if let Some(entry) = state
        .biomes
        .iter()
        .find(|entry| entry.dimension == dimension && entry.position == position)
    {
        return entry.biome.clone();
    }

    generated_biome_at(state, dimension, position).unwrap_or_else(|| "minecraft:plains".to_string())
}

fn generated_biome_at(
    state: &ServerCommandState,
    dimension: &str,
    position: BlockPos,
) -> Option<String> {
    let preset = resolve_world_preset(&state.world_preset)
        .or_else(|_| resolve_world_preset(state.world_preset.trim_start_matches("minecraft:")))
        .ok()?;
    let stem = match dimension {
        "minecraft:the_nether" => &preset.nether,
        "minecraft:the_end" => &preset.end,
        _ => &preset.overworld,
    };
    let quart_x = position.x.div_euclid(4);
    let quart_y = position.y.div_euclid(4);
    let quart_z = position.z.div_euclid(4);
    match &stem.generator {
        ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } => {
            let router = builtin_noise_router(noise_router_id_for_settings(**noise_settings))?;
            let sampler = ClimateSampler::from_noise_router(
                &router.router,
                state.world_seed,
                **noise_settings,
            );
            get_biome(biome_source_model, quart_x, quart_y, quart_z, &sampler).map(str::to_string)
        }
        ResolvedChunkGenerator::Flat {
            biome_source_model, ..
        } => crate::biome::select_biome_from_source(
            biome_source_model,
            quart_x,
            quart_y,
            quart_z,
            crate::biome::climate_target(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            0.0,
        )
        .map(str::to_string),
        ResolvedChunkGenerator::Debug { biome, .. } => Some((*biome).to_string()),
    }
}

pub(super) fn set_biome_in_dimension(
    state: &mut ServerCommandState,
    dimension: &str,
    position: BlockPos,
    biome: String,
) {
    if let Some(entry) = state
        .biomes
        .iter_mut()
        .find(|entry| entry.dimension == dimension && entry.position == position)
    {
        entry.biome = biome;
    } else {
        state.biomes.push(BiomeEntry {
            dimension: dimension.to_string(),
            position,
            biome,
        });
    }
}

pub(super) fn forceload_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["forceload", "add", x, z] => change_forceload(
            state,
            parse_column_pos(x, z)?,
            parse_column_pos(x, z)?,
            true,
        ),
        ["forceload", "add", from_x, from_z, to_x, to_z] => change_forceload(
            state,
            parse_column_pos(from_x, from_z)?,
            parse_column_pos(to_x, to_z)?,
            true,
        ),
        ["forceload", "remove", x, z] => change_forceload(
            state,
            parse_column_pos(x, z)?,
            parse_column_pos(x, z)?,
            false,
        ),
        ["forceload", "remove", from_x, from_z, to_x, to_z] => change_forceload(
            state,
            parse_column_pos(from_x, from_z)?,
            parse_column_pos(to_x, to_z)?,
            false,
        ),
        ["forceload", "remove", "all"] => {
            let dimension = state.command_source_dimension.clone();
            state
                .forced_chunks
                .retain(|chunk| chunk.dimension != dimension);
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.forceload.removed.all",
                broadcast_to_admins: true,
            })
        }
        ["forceload", "query"] => {
            let count = state
                .forced_chunks
                .iter()
                .filter(|chunk| chunk.dimension == state.command_source_dimension)
                .count() as i32;
            Ok(CommandResult {
                success_count: count,
                feedback_key: if count == 1 {
                    "commands.forceload.list.single"
                } else if count > 1 {
                    "commands.forceload.list.multiple"
                } else {
                    "commands.forceload.added.none"
                },
                broadcast_to_admins: false,
            })
        }
        ["forceload", "query", x, z] => {
            let chunk = block_column_to_chunk(parse_column_pos(x, z)?);
            if is_forced_chunk(state, &state.command_source_dimension, chunk) {
                Ok(CommandResult {
                    success_count: 1,
                    feedback_key: "commands.forceload.query.success",
                    broadcast_to_admins: false,
                })
            } else {
                Err(CommandError::ForceLoadNotForced)
            }
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn change_forceload(
    state: &mut ServerCommandState,
    from: ChunkPos,
    to: ChunkPos,
    add: bool,
) -> Result<CommandResult, CommandError> {
    let min_block_x = from.x.min(to.x);
    let min_block_z = from.z.min(to.z);
    let max_block_x = from.x.max(to.x);
    let max_block_z = from.z.max(to.z);
    if min_block_x < -30_000_000
        || min_block_z < -30_000_000
        || max_block_x >= 30_000_000
        || max_block_z >= 30_000_000
    {
        return Err(CommandError::ForceLoadOutOfWorld);
    }
    let min = block_column_to_chunk(ChunkPos {
        x: min_block_x,
        z: min_block_z,
    });
    let max = block_column_to_chunk(ChunkPos {
        x: max_block_x,
        z: max_block_z,
    });
    let chunk_count = i64::from(max.x - min.x + 1) * i64::from(max.z - min.z + 1);
    if chunk_count > 256 {
        return Err(CommandError::ForceLoadTooBig);
    }

    let dimension = state.command_source_dimension.clone();
    let mut changed = 0;
    for x in min.x..=max.x {
        for z in min.z..=max.z {
            let chunk = ChunkPos { x, z };
            let forced = is_forced_chunk(state, &dimension, chunk);
            if add && !forced {
                state.forced_chunks.push(ForcedChunk {
                    dimension: dimension.clone(),
                    chunk,
                });
                changed += 1;
            } else if !add && forced {
                state
                    .forced_chunks
                    .retain(|entry| !(entry.dimension == dimension && entry.chunk == chunk));
                changed += 1;
            }
        }
    }
    if changed == 0 {
        return Err(if add {
            CommandError::ForceLoadAlreadyAdded
        } else {
            CommandError::ForceLoadNotForced
        });
    }
    Ok(CommandResult {
        success_count: changed,
        feedback_key: match (add, changed) {
            (true, 1) => "commands.forceload.added.single",
            (true, _) => "commands.forceload.added.multiple",
            (false, 1) => "commands.forceload.removed.single",
            (false, _) => "commands.forceload.removed.multiple",
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn parse_column_pos(x: &str, z: &str) -> Result<ChunkPos, CommandError> {
    Ok(ChunkPos {
        x: parse_i32(x)?,
        z: parse_i32(z)?,
    })
}

pub(super) fn block_column_to_chunk(pos: ChunkPos) -> ChunkPos {
    ChunkPos {
        x: pos.x.div_euclid(16),
        z: pos.z.div_euclid(16),
    }
}

pub(super) fn is_forced_chunk(
    state: &ServerCommandState,
    dimension: &str,
    chunk: ChunkPos,
) -> bool {
    state
        .forced_chunks
        .iter()
        .any(|entry| entry.dimension == dimension && entry.chunk == chunk)
}
