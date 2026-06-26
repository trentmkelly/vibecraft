use super::*;
use crate::item_catalog::primary_item_static_name;

pub(super) fn clear_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let (targets, item, max_count) = match parts {
        ["clear"] => (
            vec![state
                .command_source_player
                .clone()
                .ok_or(CommandError::InvalidSyntax)?],
            None,
            -1,
        ),
        ["clear", targets] => (resolve_clear_targets(&state.online_players, targets)?, None, -1),
        ["clear", targets, item] => (
            resolve_clear_targets(&state.online_players, targets)?,
            Some(parse_item_identifier(item)?),
            -1,
        ),
        ["clear", targets, item, max_count] => {
            let max_count = parse_i32(max_count)?;
            if max_count < 0 {
                return Err(CommandError::InvalidSyntax);
            }
            (
                resolve_clear_targets(&state.online_players, targets)?,
                Some(parse_item_identifier(item)?),
                max_count,
            )
        }
        _ => return Err(CommandError::InvalidSyntax),
    };
    if targets.is_empty() {
        return Err(CommandError::InvalidSyntax);
    }

    let mut cleared = 0;
    for target in &targets {
        let inventory = command_inventory_mut(state, target);
        cleared += inventory.clear_or_count(item.as_deref(), max_count);
    }

    if cleared == 0 {
        return Err(if targets.len() == 1 {
            CommandError::ClearFailedSingle
        } else {
            CommandError::ClearFailedMultiple
        });
    }

    Ok(CommandResult {
        success_count: cleared,
        feedback_key: match (max_count == 0, targets.len() == 1) {
            (true, true) => "commands.clear.test.single",
            (true, false) => "commands.clear.test.multiple",
            (false, true) => "commands.clear.success.single",
            (false, false) => "commands.clear.success.multiple",
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn resolve_clear_targets(
    online_players: &[NameAndId],
    targets: &str,
) -> Result<Vec<NameAndId>, CommandError> {
    let target_names = parse_name_list(targets);
    if target_names.is_empty() {
        return Err(CommandError::NoPlayers);
    }
    let mut resolved = Vec::with_capacity(target_names.len());
    for target in target_names {
        let player = online_players
            .iter()
            .find(|online| online.uuid == target.uuid)
            .ok_or(CommandError::NoPlayers)?;
        resolved.push(player.clone());
    }
    Ok(resolved)
}

impl CommandPlayerInventory {
    pub(super) fn clear_or_count(&mut self, item: Option<&str>, max_count: i32) -> i32 {
        let counting_only = max_count == 0;
        let unlimited = max_count < 0;
        let mut changed = 0;
        for stack in &mut self.items {
            if stack.count <= 0 || item.is_some_and(|item| stack.item != item) {
                continue;
            }
            if counting_only {
                changed += stack.count;
                continue;
            }
            let removed = if unlimited {
                stack.count
            } else {
                (max_count - changed).min(stack.count)
            };
            stack.count -= removed;
            changed += removed;
            if !unlimited && changed >= max_count {
                break;
            }
        }
        if !counting_only {
            self.items.retain(|stack| stack.count > 0);
        }
        changed
    }

    pub(super) fn add_item_stacks(&mut self, item: &str, count: i32, max_stack_size: i32) {
        let mut remaining = count;
        while remaining > 0 {
            let size = remaining.min(max_stack_size);
            self.items.push(CommandItemStack {
                item: item.to_string(),
                count: size,
            });
            remaining -= size;
        }
    }
}

pub(super) fn command_inventory_mut<'a>(
    state: &'a mut ServerCommandState,
    player: &NameAndId,
) -> &'a mut CommandPlayerInventory {
    if let Some(index) = state
        .player_inventories
        .iter()
        .position(|inventory| inventory.player.uuid == player.uuid)
    {
        return &mut state.player_inventories[index];
    }
    state.player_inventories.push(CommandPlayerInventory {
        player: player.clone(),
        items: Vec::new(),
    });
    let index = state.player_inventories.len() - 1;
    &mut state.player_inventories[index]
}

pub(super) fn give_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let (targets, item, count) = match parts {
        ["give", targets, item] => (
            parse_name_list(targets),
            parse_item_identifier(item)?,
            1,
        ),
        ["give", targets, item, count] => (
            parse_name_list(targets),
            parse_item_identifier(item)?,
            parse_i32(count)?,
        ),
        _ => return Err(CommandError::InvalidSyntax),
    };
    if targets.is_empty() || count < 1 {
        return Err(CommandError::InvalidSyntax);
    }
    let max_stack_size = item_max_stack_size(&item);
    let max_allowed_count = max_stack_size * 100;
    if count > max_allowed_count {
        return Ok(CommandResult {
            success_count: 0,
            feedback_key: "commands.give.failed.toomanyitems",
            broadcast_to_admins: false,
        });
    }
    for target in &targets {
        command_inventory_mut(state, target).add_item_stacks(&item, count, max_stack_size);
    }
    Ok(CommandResult {
        success_count: targets.len() as i32,
        feedback_key: "commands.give.success.single",
        broadcast_to_admins: true,
    })
}

fn parse_item_identifier(input: &str) -> Result<String, CommandError> {
    let item = parse_resource_identifier(input)?;
    if primary_item_static_name(&item).is_some() {
        Ok(item)
    } else {
        Err(CommandError::InvalidSyntax)
    }
}

pub(super) fn item_max_stack_size(item: &str) -> i32 {
    if JAVA_STACKS_TO_1.contains(&item)
        || item.ends_with("_armor")
        || item.ends_with("_sword")
        || item.ends_with("_pickaxe")
        || item.ends_with("_axe")
        || item.ends_with("_shovel")
        || item.ends_with("_hoe")
        || item.ends_with("_helmet")
        || item.ends_with("_chestplate")
        || item.ends_with("_leggings")
        || item.ends_with("_boots")
    {
        1
    } else if JAVA_STACKS_TO_16.contains(&item) {
        16
    } else {
        64
    }
}

const JAVA_STACKS_TO_16: &[&str] = &[
    "minecraft:oak_sign",
    "minecraft:spruce_sign",
    "minecraft:birch_sign",
    "minecraft:jungle_sign",
    "minecraft:acacia_sign",
    "minecraft:cherry_sign",
    "minecraft:dark_oak_sign",
    "minecraft:pale_oak_sign",
    "minecraft:mangrove_sign",
    "minecraft:bamboo_sign",
    "minecraft:crimson_sign",
    "minecraft:warped_sign",
    "minecraft:oak_hanging_sign",
    "minecraft:spruce_hanging_sign",
    "minecraft:birch_hanging_sign",
    "minecraft:jungle_hanging_sign",
    "minecraft:acacia_hanging_sign",
    "minecraft:cherry_hanging_sign",
    "minecraft:dark_oak_hanging_sign",
    "minecraft:pale_oak_hanging_sign",
    "minecraft:mangrove_hanging_sign",
    "minecraft:bamboo_hanging_sign",
    "minecraft:crimson_hanging_sign",
    "minecraft:warped_hanging_sign",
    "minecraft:bucket",
    "minecraft:snowball",
    "minecraft:egg",
    "minecraft:blue_egg",
    "minecraft:brown_egg",
    "minecraft:ender_pearl",
    "minecraft:written_book",
    "minecraft:armor_stand",
    "minecraft:white_banner",
    "minecraft:orange_banner",
    "minecraft:magenta_banner",
    "minecraft:light_blue_banner",
    "minecraft:yellow_banner",
    "minecraft:lime_banner",
    "minecraft:pink_banner",
    "minecraft:gray_banner",
    "minecraft:light_gray_banner",
    "minecraft:cyan_banner",
    "minecraft:purple_banner",
    "minecraft:blue_banner",
    "minecraft:brown_banner",
    "minecraft:green_banner",
    "minecraft:red_banner",
    "minecraft:black_banner",
    "minecraft:honey_bottle",
];

const JAVA_STACKS_TO_1: &[&str] = &[
    "minecraft:shulker_box",
    "minecraft:white_shulker_box",
    "minecraft:orange_shulker_box",
    "minecraft:magenta_shulker_box",
    "minecraft:light_blue_shulker_box",
    "minecraft:yellow_shulker_box",
    "minecraft:lime_shulker_box",
    "minecraft:pink_shulker_box",
    "minecraft:gray_shulker_box",
    "minecraft:light_gray_shulker_box",
    "minecraft:cyan_shulker_box",
    "minecraft:purple_shulker_box",
    "minecraft:blue_shulker_box",
    "minecraft:brown_shulker_box",
    "minecraft:green_shulker_box",
    "minecraft:red_shulker_box",
    "minecraft:black_shulker_box",
    "minecraft:saddle",
    "minecraft:white_harness",
    "minecraft:orange_harness",
    "minecraft:magenta_harness",
    "minecraft:light_blue_harness",
    "minecraft:yellow_harness",
    "minecraft:lime_harness",
    "minecraft:pink_harness",
    "minecraft:gray_harness",
    "minecraft:light_gray_harness",
    "minecraft:cyan_harness",
    "minecraft:purple_harness",
    "minecraft:blue_harness",
    "minecraft:brown_harness",
    "minecraft:green_harness",
    "minecraft:red_harness",
    "minecraft:black_harness",
    "minecraft:minecart",
    "minecraft:chest_minecart",
    "minecraft:furnace_minecart",
    "minecraft:tnt_minecart",
    "minecraft:hopper_minecart",
    "minecraft:carrot_on_a_stick",
    "minecraft:warped_fungus_on_a_stick",
    "minecraft:elytra",
    "minecraft:oak_boat",
    "minecraft:oak_chest_boat",
    "minecraft:spruce_boat",
    "minecraft:spruce_chest_boat",
    "minecraft:birch_boat",
    "minecraft:birch_chest_boat",
    "minecraft:jungle_boat",
    "minecraft:jungle_chest_boat",
    "minecraft:acacia_boat",
    "minecraft:acacia_chest_boat",
    "minecraft:cherry_boat",
    "minecraft:cherry_chest_boat",
    "minecraft:dark_oak_boat",
    "minecraft:dark_oak_chest_boat",
    "minecraft:pale_oak_boat",
    "minecraft:pale_oak_chest_boat",
    "minecraft:mangrove_boat",
    "minecraft:mangrove_chest_boat",
    "minecraft:bamboo_raft",
    "minecraft:bamboo_chest_raft",
    "minecraft:flint_and_steel",
    "minecraft:bow",
    "minecraft:mushroom_stew",
    "minecraft:water_bucket",
    "minecraft:lava_bucket",
    "minecraft:powder_snow_bucket",
    "minecraft:milk_bucket",
    "minecraft:pufferfish_bucket",
    "minecraft:salmon_bucket",
    "minecraft:cod_bucket",
    "minecraft:tropical_fish_bucket",
    "minecraft:axolotl_bucket",
    "minecraft:tadpole_bucket",
    "minecraft:bundle",
    "minecraft:white_bundle",
    "minecraft:orange_bundle",
    "minecraft:magenta_bundle",
    "minecraft:light_blue_bundle",
    "minecraft:yellow_bundle",
    "minecraft:lime_bundle",
    "minecraft:pink_bundle",
    "minecraft:gray_bundle",
    "minecraft:light_gray_bundle",
    "minecraft:cyan_bundle",
    "minecraft:purple_bundle",
    "minecraft:blue_bundle",
    "minecraft:brown_bundle",
    "minecraft:green_bundle",
    "minecraft:red_bundle",
    "minecraft:black_bundle",
    "minecraft:fishing_rod",
    "minecraft:spyglass",
    "minecraft:cake",
    "minecraft:white_bed",
    "minecraft:orange_bed",
    "minecraft:magenta_bed",
    "minecraft:light_blue_bed",
    "minecraft:yellow_bed",
    "minecraft:lime_bed",
    "minecraft:pink_bed",
    "minecraft:gray_bed",
    "minecraft:light_gray_bed",
    "minecraft:cyan_bed",
    "minecraft:purple_bed",
    "minecraft:blue_bed",
    "minecraft:brown_bed",
    "minecraft:green_bed",
    "minecraft:red_bed",
    "minecraft:black_bed",
    "minecraft:shears",
    "minecraft:potion",
    "minecraft:writable_book",
    "minecraft:mace",
    "minecraft:enchanted_book",
    "minecraft:rabbit_stew",
    "minecraft:command_block_minecart",
    "minecraft:beetroot_soup",
    "minecraft:splash_potion",
    "minecraft:lingering_potion",
    "minecraft:shield",
    "minecraft:totem_of_undying",
    "minecraft:knowledge_book",
    "minecraft:debug_stick",
    "minecraft:music_disc_13",
    "minecraft:music_disc_cat",
    "minecraft:music_disc_blocks",
    "minecraft:music_disc_chirp",
    "minecraft:music_disc_creator",
    "minecraft:music_disc_creator_music_box",
    "minecraft:music_disc_far",
    "minecraft:music_disc_lava_chicken",
    "minecraft:music_disc_mall",
    "minecraft:music_disc_mellohi",
    "minecraft:music_disc_stal",
    "minecraft:music_disc_strad",
    "minecraft:music_disc_ward",
    "minecraft:music_disc_11",
    "minecraft:music_disc_wait",
    "minecraft:music_disc_otherside",
    "minecraft:music_disc_relic",
    "minecraft:music_disc_5",
    "minecraft:music_disc_pigstep",
    "minecraft:music_disc_precipice",
    "minecraft:music_disc_tears",
    "minecraft:trident",
    "minecraft:crossbow",
    "minecraft:suspicious_stew",
    "minecraft:flower_banner_pattern",
    "minecraft:creeper_banner_pattern",
    "minecraft:skull_banner_pattern",
    "minecraft:mojang_banner_pattern",
    "minecraft:globe_banner_pattern",
    "minecraft:piglin_banner_pattern",
    "minecraft:flow_banner_pattern",
    "minecraft:guster_banner_pattern",
    "minecraft:field_masoned_banner_pattern",
    "minecraft:bordure_indented_banner_pattern",
    "minecraft:goat_horn",
    "minecraft:brush",
];

pub(super) fn item_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts.get(1).copied() {
        Some("replace") => item_replace_command(state, parts),
        Some("modify") => item_modify_command(state, parts),
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn item_replace_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if parts.contains(&"with") {
        item_replace_with_command(state, parts)
    } else if parts.contains(&"from") {
        item_replace_from_command(state, parts)
    } else {
        Err(CommandError::InvalidSyntax)
    }
}

fn item_replace_with_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["item", "replace", "entity", targets, slot, "with", item] => {
            let stack = parse_command_item_stack(item, 1)?;
            set_entity_items(
                state,
                parse_entity_list(targets),
                &parse_item_slot(slot)?,
                Some(stack),
            )
        }
        ["item", "replace", "entity", targets, slot, "with", item, count] => {
            let count = parse_item_count(count)?;
            let stack = parse_command_item_stack(item, count)?;
            set_entity_items(
                state,
                parse_entity_list(targets),
                &parse_item_slot(slot)?,
                Some(stack),
            )
        }
        ["item", "replace", "block", x, y, z, slot, "with", item] => {
            let pos = parse_block_pos(x, y, z)?;
            let stack = parse_command_item_stack(item, 1)?;
            set_block_item(state, pos, &parse_item_slot(slot)?, Some(stack))
        }
        ["item", "replace", "block", x, y, z, slot, "with", item, count] => {
            let pos = parse_block_pos(x, y, z)?;
            let count = parse_item_count(count)?;
            let stack = parse_command_item_stack(item, count)?;
            set_block_item(state, pos, &parse_item_slot(slot)?, Some(stack))
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn item_replace_from_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["item", "replace", "entity", targets, target_slot, "from", "entity", source, source_slot] =>
        {
            let source = entity_ref(source);
            let stack = get_entity_item(state, &source, &parse_item_slot(source_slot)?)?;
            set_entity_items(
                state,
                parse_entity_list(targets),
                &parse_item_slot(target_slot)?,
                stack,
            )
        }
        ["item", "replace", "entity", targets, target_slot, "from", "entity", source, source_slot, modifier] =>
        {
            let source = entity_ref(source);
            let target_slot = parse_item_slot(target_slot)?;
            let stack = get_entity_item(state, &source, &parse_item_slot(source_slot)?)?;
            let stack = apply_item_modifier(state, None, modifier, stack)?;
            set_entity_items(state, parse_entity_list(targets), &target_slot, stack)
        }
        ["item", "replace", "entity", targets, target_slot, "from", "block", x, y, z, source_slot] =>
        {
            let source = parse_block_pos(x, y, z)?;
            let stack = get_block_item(state, &source, &parse_item_slot(source_slot)?)?;
            set_entity_items(
                state,
                parse_entity_list(targets),
                &parse_item_slot(target_slot)?,
                stack,
            )
        }
        ["item", "replace", "entity", targets, target_slot, "from", "block", x, y, z, source_slot, modifier] =>
        {
            let target_slot = parse_item_slot(target_slot)?;
            let source = parse_block_pos(x, y, z)?;
            let stack = get_block_item(state, &source, &parse_item_slot(source_slot)?)?;
            let stack = apply_item_modifier(state, None, modifier, stack)?;
            set_entity_items(state, parse_entity_list(targets), &target_slot, stack)
        }
        ["item", "replace", "block", x, y, z, target_slot, "from", "entity", source, source_slot] =>
        {
            let target = parse_block_pos(x, y, z)?;
            let source = entity_ref(source);
            let stack = get_entity_item(state, &source, &parse_item_slot(source_slot)?)?;
            set_block_item(state, target, &parse_item_slot(target_slot)?, stack)
        }
        ["item", "replace", "block", x, y, z, target_slot, "from", "entity", source, source_slot, modifier] =>
        {
            let target = parse_block_pos(x, y, z)?;
            let target_slot = parse_item_slot(target_slot)?;
            let source = entity_ref(source);
            let stack = get_entity_item(state, &source, &parse_item_slot(source_slot)?)?;
            let stack = apply_item_modifier(state, None, modifier, stack)?;
            set_block_item(state, target, &target_slot, stack)
        }
        ["item", "replace", "block", tx, ty, tz, target_slot, "from", "block", sx, sy, sz, source_slot] =>
        {
            let target = parse_block_pos(tx, ty, tz)?;
            let source = parse_block_pos(sx, sy, sz)?;
            let stack = get_block_item(state, &source, &parse_item_slot(source_slot)?)?;
            set_block_item(state, target, &parse_item_slot(target_slot)?, stack)
        }
        ["item", "replace", "block", tx, ty, tz, target_slot, "from", "block", sx, sy, sz, source_slot, modifier] =>
        {
            let target = parse_block_pos(tx, ty, tz)?;
            let target_slot = parse_item_slot(target_slot)?;
            let source = parse_block_pos(sx, sy, sz)?;
            let stack = get_block_item(state, &source, &parse_item_slot(source_slot)?)?;
            let stack = apply_item_modifier(state, None, modifier, stack)?;
            set_block_item(state, target, &target_slot, stack)
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn item_modify_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["item", "modify", "entity", targets, slot, modifier] => {
            let targets = parse_entity_list(targets);
            let slot = parse_item_slot(slot)?;
            let mut changed = 0;
            for target in targets {
                let stack = get_entity_item(state, &target, &slot)?;
                let modified = apply_item_modifier(
                    state,
                    Some(CommandItemTarget::Entity {
                        entity: target.clone(),
                        slot: slot.clone(),
                    }),
                    modifier,
                    stack,
                )?;
                upsert_entity_item(state, target, &slot, modified);
                changed += 1;
            }
            if changed == 0 {
                Err(CommandError::ItemTargetNoChanges)
            } else {
                Ok(CommandResult {
                    success_count: changed,
                    feedback_key: if changed == 1 {
                        "commands.item.entity.set.success.single"
                    } else {
                        "commands.item.entity.set.success.multiple"
                    },
                    broadcast_to_admins: true,
                })
            }
        }
        ["item", "modify", "block", x, y, z, slot, modifier] => {
            let pos = parse_block_pos(x, y, z)?;
            let slot = parse_item_slot(slot)?;
            let stack = get_block_item(state, &pos, &slot)?;
            let modified = apply_item_modifier(
                state,
                Some(CommandItemTarget::Block {
                    pos,
                    slot: slot.clone(),
                }),
                modifier,
                stack,
            )?;
            set_block_item(state, pos, &slot, modified)
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn parse_command_item_stack(item: &str, count: i32) -> Result<CommandItemStack, CommandError> {
    let item = parse_item_identifier(item)?;
    if count > item_max_stack_size(&item) {
        return Err(CommandError::InvalidSyntax);
    }
    Ok(CommandItemStack { item, count })
}

pub(super) fn parse_item_count(input: &str) -> Result<i32, CommandError> {
    let count = parse_i32(input)?;
    if (1..=99).contains(&count) {
        Ok(count)
    } else {
        Err(CommandError::InvalidSyntax)
    }
}

pub(super) fn parse_item_slot(input: &str) -> Result<String, CommandError> {
    let valid_named_slot = matches!(
        input,
        "contents"
            | "weapon"
            | "weapon.mainhand"
            | "weapon.offhand"
            | "armor.head"
            | "armor.chest"
            | "armor.legs"
            | "armor.feet"
            | "armor.body"
            | "saddle"
            | "horse.chest"
            | "player.cursor"
    ) || slot_index_in_range(input, "container.", 54)
        || slot_index_in_range(input, "hotbar.", 9)
        || slot_index_in_range(input, "inventory.", 27)
        || slot_index_in_range(input, "enderchest.", 27)
        || slot_index_in_range(input, "mob.inventory.", 8)
        || slot_index_in_range(input, "horse.", 15)
        || slot_index_in_range(input, "player.crafting.", 4);
    if valid_named_slot {
        Ok(input.to_string())
    } else {
        Err(CommandError::InvalidSyntax)
    }
}

fn slot_index_in_range(input: &str, prefix: &str, size: u8) -> bool {
    input
        .strip_prefix(prefix)
        .and_then(|index| index.parse::<u8>().ok())
        .is_some_and(|index| index < size)
}

pub(super) fn set_entity_items(
    state: &mut ServerCommandState,
    targets: Vec<EntityRef>,
    slot: &str,
    stack: Option<CommandItemStack>,
) -> Result<CommandResult, CommandError> {
    if targets.is_empty() {
        return Err(CommandError::ItemTargetNoChanges);
    }
    for target in &targets {
        upsert_entity_item(state, target.clone(), slot, stack.clone());
    }
    Ok(CommandResult {
        success_count: targets.len() as i32,
        feedback_key: if targets.len() == 1 {
            "commands.item.entity.set.success.single"
        } else {
            "commands.item.entity.set.success.multiple"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn set_block_item(
    state: &mut ServerCommandState,
    pos: BlockPos,
    slot: &str,
    stack: Option<CommandItemStack>,
) -> Result<CommandResult, CommandError> {
    if !is_block_container(state, &pos) {
        return Err(CommandError::ItemTargetNotContainer);
    }
    if !has_block_slot(state, &pos, slot) {
        return Err(CommandError::ItemTargetNoSuchSlot);
    }
    upsert_block_item(state, pos, slot, stack);
    Ok(CommandResult {
        success_count: 1,
        feedback_key: "commands.item.block.set.success",
        broadcast_to_admins: true,
    })
}

pub(super) fn get_entity_item(
    state: &ServerCommandState,
    entity: &EntityRef,
    slot: &str,
) -> Result<Option<CommandItemStack>, CommandError> {
    state
        .entity_item_slots
        .iter()
        .find(|entry| entry.entity.id == entity.id && entry.slot == slot)
        .map(|entry| entry.item.clone())
        .ok_or(CommandError::ItemSourceNoSuchSlot)
}

pub(super) fn get_block_item(
    state: &ServerCommandState,
    pos: &BlockPos,
    slot: &str,
) -> Result<Option<CommandItemStack>, CommandError> {
    if !is_block_container(state, pos) {
        return Err(CommandError::ItemSourceNotContainer);
    }
    state
        .block_item_slots
        .iter()
        .find(|entry| entry.pos == *pos && entry.slot == slot)
        .map(|entry| entry.item.clone())
        .ok_or(CommandError::ItemSourceNoSuchSlot)
}

fn is_block_container(state: &ServerCommandState, pos: &BlockPos) -> bool {
    state.block_item_slots.iter().any(|entry| entry.pos == *pos)
}

fn has_block_slot(state: &ServerCommandState, pos: &BlockPos, slot: &str) -> bool {
    state
        .block_item_slots
        .iter()
        .any(|entry| entry.pos == *pos && entry.slot == slot)
}

pub(super) fn upsert_entity_item(
    state: &mut ServerCommandState,
    entity: EntityRef,
    slot: &str,
    item: Option<CommandItemStack>,
) {
    if let Some(entry) = state
        .entity_item_slots
        .iter_mut()
        .find(|entry| entry.entity.id == entity.id && entry.slot == slot)
    {
        entry.item = item;
    } else {
        state.entity_item_slots.push(CommandEntityItemSlot {
            entity,
            slot: slot.to_string(),
            item,
        });
    }
}

pub(super) fn upsert_block_item(
    state: &mut ServerCommandState,
    pos: BlockPos,
    slot: &str,
    item: Option<CommandItemStack>,
) {
    if let Some(entry) = state
        .block_item_slots
        .iter_mut()
        .find(|entry| entry.pos == pos && entry.slot == slot)
    {
        entry.item = item;
    } else {
        state.block_item_slots.push(CommandBlockItemSlot {
            pos,
            slot: slot.to_string(),
            item,
        });
    }
}

pub(super) fn apply_item_modifier(
    state: &mut ServerCommandState,
    target: Option<CommandItemTarget>,
    modifier: &str,
    stack: Option<CommandItemStack>,
) -> Result<Option<CommandItemStack>, CommandError> {
    let modifier = parse_resource_identifier(modifier)?;
    let output = stack.as_ref().map(|stack| {
        let max_stack_size = item_max_stack_size(&stack.item);
        CommandItemStack {
            item: stack.item.clone(),
            count: stack.count.min(max_stack_size),
        }
    });
    if let Some(target) = target {
        state.item_modifier_events.push(CommandItemModifierEvent {
            target,
            modifier,
            input: stack,
            output: output.clone(),
        });
    }
    Ok(output)
}
