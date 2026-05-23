use super::*;

pub(super) fn locate_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let (kind, query, include_y, feedback_key) = match parts {
        ["locate", "structure", query] => (
            LocateKind::Structure,
            parse_locate_query(query)?,
            false,
            "commands.locate.structure.success",
        ),
        ["locate", "biome", query] => (
            LocateKind::Biome,
            parse_locate_query(query)?,
            true,
            "commands.locate.biome.success",
        ),
        ["locate", "poi", query] => (
            LocateKind::Poi,
            parse_locate_query(query)?,
            false,
            "commands.locate.poi.success",
        ),
        _ => return Err(CommandError::InvalidSyntax),
    };

    if kind == LocateKind::Structure
        && !query.is_tag
        && !known_locate_structure_ids().contains(&query.id.as_str())
        && !state
            .locatable_entries
            .iter()
            .any(|entry| entry.kind == LocateKind::Structure && entry.id == query.id)
    {
        return Err(CommandError::LocateStructureInvalid);
    }

    let source_pos = BlockPos {
        x: state.command_source_position.x.floor() as i32,
        y: state.command_source_position.y.floor() as i32,
        z: state.command_source_position.z.floor() as i32,
    };
    let nearest = state
        .locatable_entries
        .iter()
        .filter(|entry| entry.kind == kind && locate_entry_matches(entry, &query))
        .min_by_key(|entry| locate_distance(source_pos, entry.position, include_y));

    let Some(found) = nearest else {
        return Err(match kind {
            LocateKind::Structure => CommandError::LocateStructureNotFound,
            LocateKind::Biome => CommandError::LocateBiomeNotFound,
            LocateKind::Poi => CommandError::LocatePoiNotFound,
        });
    };

    let distance = locate_distance(source_pos, found.position, include_y);
    state.locate_results.push(CommandLocateResult {
        kind,
        query: query.printable(),
        found_id: found.id.clone(),
        position: found.position,
        distance,
        include_y,
    });
    Ok(CommandResult {
        success_count: distance,
        feedback_key,
        broadcast_to_admins: false,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LocateQuery {
    id: String,
    is_tag: bool,
}

impl LocateQuery {
    pub(super) fn printable(&self) -> String {
        if self.is_tag {
            format!("#{}", self.id)
        } else {
            self.id.clone()
        }
    }
}

pub(super) fn parse_locate_query(input: &str) -> Result<LocateQuery, CommandError> {
    if let Some(tag) = input.strip_prefix('#') {
        Ok(LocateQuery {
            id: parse_resource_identifier(tag)?,
            is_tag: true,
        })
    } else {
        Ok(LocateQuery {
            id: parse_resource_identifier(input)?,
            is_tag: false,
        })
    }
}

pub(super) fn locate_entry_matches(entry: &CommandLocatableEntry, query: &LocateQuery) -> bool {
    if query.is_tag {
        entry.tags.iter().any(|tag| tag == &query.id)
    } else {
        entry.id == query.id
    }
}

pub(super) fn locate_distance(source: BlockPos, found: BlockPos, include_y: bool) -> i32 {
    let dx = i64::from(found.x) - i64::from(source.x);
    let dz = i64::from(found.z) - i64::from(source.z);
    let dy = if include_y {
        i64::from(found.y) - i64::from(source.y)
    } else {
        0
    };
    ((dx * dx + dy * dy + dz * dz) as f64).sqrt().floor() as i32
}

pub(super) fn known_locate_structure_ids() -> &'static [&'static str] {
    &[
        "minecraft:ancient_city",
        "minecraft:bastion_remnant",
        "minecraft:buried_treasure",
        "minecraft:desert_pyramid",
        "minecraft:end_city",
        "minecraft:fortress",
        "minecraft:igloo",
        "minecraft:jungle_pyramid",
        "minecraft:mansion",
        "minecraft:mineshaft",
        "minecraft:monument",
        "minecraft:nether_fossil",
        "minecraft:ocean_ruin_cold",
        "minecraft:ocean_ruin_warm",
        "minecraft:pillager_outpost",
        "minecraft:ruined_portal",
        "minecraft:shipwreck",
        "minecraft:stronghold",
        "minecraft:swamp_hut",
        "minecraft:trail_ruins",
        "minecraft:trial_chambers",
        "minecraft:village_desert",
        "minecraft:village_plains",
        "minecraft:village_savanna",
        "minecraft:village_snowy",
        "minecraft:village_taiga",
    ]
}

pub(super) fn loot_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let (target, source_start) = parse_loot_target(parts)?;
    let (source, drops) = parse_loot_source(state, &parts[source_start..])?;
    let used_drops = apply_loot_target(state, &target, &drops)?;
    state.loot_events.push(CommandLootEvent {
        target,
        source,
        drops: used_drops.clone(),
    });
    Ok(CommandResult {
        success_count: used_drops.len() as i32,
        feedback_key: if used_drops.len() == 1 {
            "commands.drop.success.single"
        } else {
            "commands.drop.success.multiple"
        },
        broadcast_to_admins: false,
    })
}

pub(super) fn parse_loot_target(
    parts: &[&str],
) -> Result<(CommandLootTarget, usize), CommandError> {
    match parts {
        ["loot", "give", players, ..] => Ok((
            CommandLootTarget::Give {
                players: parse_name_list(players),
            },
            3,
        )),
        ["loot", "spawn", x, y, z, ..] => Ok((
            CommandLootTarget::Spawn {
                position: Vec3 {
                    x: parse_f64(x)?,
                    y: parse_f64(y)?,
                    z: parse_f64(z)?,
                },
            },
            5,
        )),
        ["loot", "insert", x, y, z, ..] => Ok((
            CommandLootTarget::Insert {
                pos: parse_block_pos(x, y, z)?,
            },
            5,
        )),
        ["loot", "replace", "entity", entities, slot, rest @ ..] => {
            let (count, source_start) = parse_optional_loot_count(rest, 5)?;
            Ok((
                CommandLootTarget::ReplaceEntity {
                    entities: parse_entity_list(entities),
                    slot: parse_item_slot(slot)?,
                    count,
                },
                source_start,
            ))
        }
        ["loot", "replace", "block", x, y, z, slot, rest @ ..] => {
            let (count, source_start) = parse_optional_loot_count(rest, 7)?;
            Ok((
                CommandLootTarget::ReplaceBlock {
                    pos: parse_block_pos(x, y, z)?,
                    slot: parse_item_slot(slot)?,
                    count,
                },
                source_start,
            ))
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn parse_optional_loot_count(
    rest: &[&str],
    source_start_without_count: usize,
) -> Result<(usize, usize), CommandError> {
    if rest.is_empty() {
        return Err(CommandError::InvalidSyntax);
    }
    if matches!(rest[0], "fish" | "loot" | "kill" | "mine") {
        Ok((usize::MAX, source_start_without_count))
    } else {
        let count = parse_i32(rest[0])?;
        if count < 0 {
            Err(CommandError::InvalidSyntax)
        } else {
            Ok((count as usize, source_start_without_count + 1))
        }
    }
}

pub(super) fn parse_loot_source(
    state: &ServerCommandState,
    parts: &[&str],
) -> Result<(CommandLootSource, Vec<CommandItemStack>), CommandError> {
    match parts {
        ["loot", table] => {
            let table = parse_resource_identifier(table)?;
            Ok((
                CommandLootSource::LootTable {
                    table: table.clone(),
                },
                loot_table_drops(state, &table),
            ))
        }
        ["fish", table, x, y, z] => {
            let table = parse_resource_identifier(table)?;
            Ok((
                CommandLootSource::Fish {
                    table: table.clone(),
                    pos: parse_block_pos(x, y, z)?,
                    tool: None,
                },
                loot_table_drops(state, &table),
            ))
        }
        ["fish", table, x, y, z, tool] => {
            let table = parse_resource_identifier(table)?;
            let tool = parse_loot_tool(state, tool)?;
            Ok((
                CommandLootSource::Fish {
                    table: table.clone(),
                    pos: parse_block_pos(x, y, z)?,
                    tool,
                },
                loot_table_drops(state, &table),
            ))
        }
        ["kill", target] => {
            let entity = entity_ref(target);
            let loot = state
                .entity_loot_tables
                .iter()
                .find(|entry| entry.entity.id == entity.id)
                .ok_or(CommandError::LootNoEntityLootTable)?;
            Ok((
                CommandLootSource::Kill {
                    entity,
                    table: loot.table.clone(),
                },
                loot.drops.clone(),
            ))
        }
        ["mine", x, y, z] => mine_loot_source(state, x, y, z, None),
        ["mine", x, y, z, tool] => {
            let tool = parse_loot_tool(state, tool)?;
            mine_loot_source(state, x, y, z, tool)
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn mine_loot_source(
    state: &ServerCommandState,
    x: &str,
    y: &str,
    z: &str,
    tool: Option<String>,
) -> Result<(CommandLootSource, Vec<CommandItemStack>), CommandError> {
    let pos = parse_block_pos(x, y, z)?;
    let block = state
        .blocks
        .iter()
        .find(|entry| entry.position == pos && entry.dimension == state.command_source_dimension)
        .map(|entry| entry.block.clone())
        .ok_or(CommandError::LootNoBlockLootTable)?;
    Ok((
        CommandLootSource::Mine {
            pos,
            block: block.clone(),
            tool,
        },
        vec![CommandItemStack {
            item: block,
            count: 1,
        }],
    ))
}

pub(super) fn parse_loot_tool(
    state: &ServerCommandState,
    input: &str,
) -> Result<Option<String>, CommandError> {
    match input {
        "mainhand" | "offhand" => state
            .command_source_entity
            .as_ref()
            .map(|_| Some(input.to_string()))
            .ok_or(CommandError::LootNoHeldItems),
        item => Ok(Some(parse_resource_identifier(item)?)),
    }
}

pub(super) fn loot_table_drops(state: &ServerCommandState, table: &str) -> Vec<CommandItemStack> {
    state
        .command_loot_tables
        .iter()
        .find(|entry| entry.id == table)
        .map(|entry| entry.drops.clone())
        .unwrap_or_else(|| {
            vec![CommandItemStack {
                item: "minecraft:air".to_string(),
                count: 0,
            }]
        })
        .into_iter()
        .filter(|stack| stack.count > 0)
        .collect()
}

pub(super) fn apply_loot_target(
    state: &mut ServerCommandState,
    target: &CommandLootTarget,
    drops: &[CommandItemStack],
) -> Result<Vec<CommandItemStack>, CommandError> {
    match target {
        CommandLootTarget::Give { players } => {
            for player in players {
                for drop in drops {
                    command_inventory_mut(state, player).add_item_stacks(
                        &drop.item,
                        drop.count,
                        item_max_stack_size(&drop.item),
                    );
                }
            }
            Ok(drops.to_vec())
        }
        CommandLootTarget::Spawn { .. } => Ok(drops.to_vec()),
        CommandLootTarget::Insert { pos } => {
            for (index, drop) in drops.iter().enumerate() {
                upsert_block_item(
                    state,
                    *pos,
                    &format!("container.{index}"),
                    Some(drop.clone()),
                );
            }
            Ok(drops.to_vec())
        }
        CommandLootTarget::ReplaceEntity {
            entities,
            slot,
            count,
        } => {
            let count = if *count == usize::MAX {
                drops.len()
            } else {
                *count
            };
            let mut used = Vec::new();
            for entity in entities {
                for index in 0..count {
                    let item = drops.get(index).cloned();
                    if let Some(stack) = item.clone() {
                        used.push(stack);
                    }
                    upsert_entity_item(state, entity.clone(), &offset_slot(slot, index), item);
                }
            }
            Ok(used)
        }
        CommandLootTarget::ReplaceBlock { pos, slot, count } => {
            let count = if *count == usize::MAX {
                drops.len()
            } else {
                *count
            };
            let mut used = Vec::new();
            for index in 0..count {
                let item = drops.get(index).cloned();
                if let Some(stack) = item.clone() {
                    used.push(stack);
                }
                upsert_block_item(state, *pos, &offset_slot(slot, index), item);
            }
            Ok(used)
        }
    }
}

pub(super) fn offset_slot(slot: &str, offset: usize) -> String {
    if offset == 0 {
        return slot.to_string();
    }
    if let Some((prefix, number)) = slot.rsplit_once('.') {
        if let Ok(base) = number.parse::<usize>() {
            return format!("{prefix}.{}", base + offset);
        }
    }
    if let Ok(base) = slot.parse::<usize>() {
        return (base + offset).to_string();
    }
    format!("{slot}+{offset}")
}

pub(super) fn place_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["place", "feature", feature] => {
            place_feature_command(state, feature, command_source_block_pos(state))
        }
        ["place", "feature", feature, x, y, z] => {
            place_feature_command(state, feature, parse_block_pos(x, y, z)?)
        }
        ["place", "jigsaw", pool, target, max_depth] => place_jigsaw_command(
            state,
            pool,
            target,
            max_depth,
            command_source_block_pos(state),
        ),
        ["place", "jigsaw", pool, target, max_depth, x, y, z] => {
            place_jigsaw_command(state, pool, target, max_depth, parse_block_pos(x, y, z)?)
        }
        ["place", "structure", structure] => {
            place_structure_command(state, structure, command_source_block_pos(state))
        }
        ["place", "structure", structure, x, y, z] => {
            place_structure_command(state, structure, parse_block_pos(x, y, z)?)
        }
        ["place", "template", template] => place_template_command(
            state,
            template,
            command_source_block_pos(state),
            "none",
            "none",
            1.0,
            0,
            false,
        ),
        ["place", "template", template, x, y, z] => place_template_command(
            state,
            template,
            parse_block_pos(x, y, z)?,
            "none",
            "none",
            1.0,
            0,
            false,
        ),
        ["place", "template", template, x, y, z, rotation] => place_template_command(
            state,
            template,
            parse_block_pos(x, y, z)?,
            rotation,
            "none",
            1.0,
            0,
            false,
        ),
        ["place", "template", template, x, y, z, rotation, mirror] => place_template_command(
            state,
            template,
            parse_block_pos(x, y, z)?,
            rotation,
            mirror,
            1.0,
            0,
            false,
        ),
        ["place", "template", template, x, y, z, rotation, mirror, integrity] => {
            place_template_command(
                state,
                template,
                parse_block_pos(x, y, z)?,
                rotation,
                mirror,
                parse_integrity(integrity)?,
                0,
                false,
            )
        }
        ["place", "template", template, x, y, z, rotation, mirror, integrity, seed] => {
            place_template_command(
                state,
                template,
                parse_block_pos(x, y, z)?,
                rotation,
                mirror,
                parse_integrity(integrity)?,
                parse_i32(seed)?,
                false,
            )
        }
        ["place", "template", template, x, y, z, rotation, mirror, integrity, seed, "strict"] => {
            place_template_command(
                state,
                template,
                parse_block_pos(x, y, z)?,
                rotation,
                mirror,
                parse_integrity(integrity)?,
                parse_i32(seed)?,
                true,
            )
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn place_feature_command(
    state: &mut ServerCommandState,
    feature: &str,
    position: BlockPos,
) -> Result<CommandResult, CommandError> {
    let feature = parse_resource_identifier(feature)?;
    if configured_feature(&feature).is_none() {
        return Err(CommandError::PlaceFeatureFailed);
    }
    state.place_events.push(CommandPlaceEvent {
        kind: PlaceKind::Feature,
        id: feature,
        position,
        rotation: None,
        mirror: None,
        integrity: None,
        seed: None,
        strict: false,
        target: None,
        max_depth: None,
    });
    Ok(place_result("commands.place.feature.success"))
}

pub(super) fn place_jigsaw_command(
    state: &mut ServerCommandState,
    pool: &str,
    target: &str,
    max_depth: &str,
    position: BlockPos,
) -> Result<CommandResult, CommandError> {
    let pool = parse_resource_identifier(pool)?;
    let target = parse_resource_identifier(target)?;
    let max_depth = parse_i32(max_depth)?;
    if !(1..=20).contains(&max_depth) {
        return Err(CommandError::InvalidSyntax);
    }
    state.place_events.push(CommandPlaceEvent {
        kind: PlaceKind::Jigsaw,
        id: pool,
        position,
        rotation: None,
        mirror: None,
        integrity: None,
        seed: None,
        strict: false,
        target: Some(target),
        max_depth: Some(max_depth),
    });
    Ok(place_result("commands.place.jigsaw.success"))
}

pub(super) fn place_structure_command(
    state: &mut ServerCommandState,
    structure: &str,
    position: BlockPos,
) -> Result<CommandResult, CommandError> {
    let structure = parse_resource_identifier(structure)?;
    if !known_locate_structure_ids().contains(&structure.as_str()) {
        return Err(CommandError::PlaceStructureFailed);
    }
    state.place_events.push(CommandPlaceEvent {
        kind: PlaceKind::Structure,
        id: structure,
        position,
        rotation: None,
        mirror: None,
        integrity: None,
        seed: None,
        strict: false,
        target: None,
        max_depth: None,
    });
    Ok(place_result("commands.place.structure.success"))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn place_template_command(
    state: &mut ServerCommandState,
    template: &str,
    position: BlockPos,
    rotation: &str,
    mirror: &str,
    integrity: f32,
    seed: i32,
    strict: bool,
) -> Result<CommandResult, CommandError> {
    let template = parse_resource_identifier(template)?;
    if !state.available_templates.contains(&template) {
        return Err(CommandError::PlaceTemplateInvalid);
    }
    let rotation = parse_template_rotation(rotation)?;
    let mirror = parse_template_mirror(mirror)?;
    state.place_events.push(CommandPlaceEvent {
        kind: PlaceKind::Template,
        id: template,
        position,
        rotation: Some(rotation),
        mirror: Some(mirror),
        integrity: Some(integrity),
        seed: Some(seed),
        strict,
        target: None,
        max_depth: None,
    });
    Ok(place_result("commands.place.template.success"))
}

pub(super) fn place_result(feedback_key: &'static str) -> CommandResult {
    CommandResult {
        success_count: 1,
        feedback_key,
        broadcast_to_admins: true,
    }
}

pub(super) fn command_source_block_pos(state: &ServerCommandState) -> BlockPos {
    BlockPos {
        x: state.command_source_position.x.floor() as i32,
        y: state.command_source_position.y.floor() as i32,
        z: state.command_source_position.z.floor() as i32,
    }
}

pub(super) fn parse_integrity(input: &str) -> Result<f32, CommandError> {
    let integrity = input
        .parse::<f32>()
        .map_err(|_| CommandError::InvalidSyntax)?;
    if (0.0..=1.0).contains(&integrity) {
        Ok(integrity)
    } else {
        Err(CommandError::InvalidSyntax)
    }
}

pub(super) fn parse_template_rotation(input: &str) -> Result<String, CommandError> {
    match input {
        "none" | "clockwise_90" | "180" | "counterclockwise_90" => Ok(input.to_string()),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn parse_template_mirror(input: &str) -> Result<String, CommandError> {
    match input {
        "none" | "left_right" | "front_back" => Ok(input.to_string()),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn raid_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let pos = raid_source_pos(state)?;
    match parts {
        ["raid", "start", omen] => {
            let omen = parse_i32(omen)?;
            if omen < 0 {
                return Err(CommandError::InvalidSyntax);
            }
            if active_raid_index(state, pos).is_some() {
                return Ok(raid_result(-1, "commands.raid.already_started"));
            }
            state.raids.push(CommandRaidState {
                center: pos,
                omen_level: omen,
                groups_spawned: 0,
                raiders_alive: 0,
                health: 0,
                total_health: 0,
                stopped: false,
                glowing: false,
            });
            Ok(raid_result(1, "commands.raid.start.success"))
        }
        ["raid", "stop"] => {
            if let Some(index) = active_raid_index(state, pos) {
                state.raids[index].stopped = true;
                Ok(raid_result(1, "commands.raid.stop.success"))
            } else {
                Ok(raid_result(-1, "commands.raid.stop.none"))
            }
        }
        ["raid", "check"] => {
            if active_raid_index(state, pos).is_some() {
                Ok(raid_result(1, "commands.raid.check.success"))
            } else {
                Ok(raid_result(0, "commands.raid.check.none"))
            }
        }
        ["raid", "sound", sound_type] => {
            let local = *sound_type == "local";
            if local {
                state.raid_events.push(CommandRaidEvent::Sound {
                    local,
                    position: Vec3 {
                        x: state.command_source_position.x + 5.0,
                        y: state.command_source_position.y,
                        z: state.command_source_position.z,
                    },
                });
            }
            Ok(raid_result(1, "commands.raid.sound"))
        }
        ["raid", "spawnleader"] => {
            state.raid_events.push(CommandRaidEvent::SpawnLeader {
                position: state.command_source_position,
            });
            Ok(raid_result(1, "commands.raid.spawnleader.success"))
        }
        ["raid", "setomen", level] => {
            let level = parse_i32(level)?;
            if level < 0 {
                return Err(CommandError::InvalidSyntax);
            }
            if let Some(index) = active_raid_index(state, pos) {
                let max = 5;
                if level > max {
                    Ok(raid_result(1, "commands.raid.omen.too_high"))
                } else {
                    state.raids[index].omen_level = level;
                    Ok(raid_result(1, "commands.raid.omen.changed"))
                }
            } else {
                Ok(raid_result(1, "commands.raid.omen.none"))
            }
        }
        ["raid", "glow"] => {
            if let Some(index) = active_raid_index(state, pos) {
                state.raids[index].glowing = true;
            }
            Ok(raid_result(1, "commands.raid.glow"))
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn raid_source_pos(state: &ServerCommandState) -> Result<BlockPos, CommandError> {
    state
        .command_source_player
        .as_ref()
        .ok_or(CommandError::InvalidSyntax)?;
    Ok(command_source_block_pos(state))
}

pub(super) fn active_raid_index(state: &ServerCommandState, pos: BlockPos) -> Option<usize> {
    state
        .raids
        .iter()
        .position(|raid| !raid.stopped && raid.center == pos)
}

pub(super) fn raid_result(success_count: i32, feedback_key: &'static str) -> CommandResult {
    CommandResult {
        success_count,
        feedback_key,
        broadcast_to_admins: false,
    }
}
