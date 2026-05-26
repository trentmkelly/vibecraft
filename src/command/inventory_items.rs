use super::*;

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
        ["clear", targets] => (parse_name_list(targets), None, -1),
        ["clear", targets, item] => (
            parse_name_list(targets),
            Some(parse_resource_identifier(item)?),
            -1,
        ),
        ["clear", targets, item, max_count] => {
            let max_count = parse_i32(max_count)?;
            if max_count < 0 {
                return Err(CommandError::InvalidSyntax);
            }
            (
                parse_name_list(targets),
                Some(parse_resource_identifier(item)?),
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
            parse_resource_identifier(item)?,
            1,
        ),
        ["give", targets, item, count] => (
            parse_name_list(targets),
            parse_resource_identifier(item)?,
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
        return Err(CommandError::GiveTooManyItems);
    }
    for target in &targets {
        command_inventory_mut(state, target).add_item_stacks(&item, count, max_stack_size);
    }
    Ok(CommandResult {
        success_count: targets.len() as i32,
        feedback_key: if targets.len() == 1 {
            "commands.give.success.single"
        } else {
            "commands.give.success.multiple"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn item_max_stack_size(item: &str) -> i32 {
    if item.ends_with("_sword")
        || item.ends_with("_pickaxe")
        || item.ends_with("_axe")
        || item.ends_with("_shovel")
        || item.ends_with("_hoe")
        || item.ends_with("_helmet")
        || item.ends_with("_chestplate")
        || item.ends_with("_leggings")
        || item.ends_with("_boots")
        || matches!(
            item,
            "minecraft:bow"
                | "minecraft:crossbow"
                | "minecraft:trident"
                | "minecraft:mace"
                | "minecraft:shield"
                | "minecraft:elytra"
                | "minecraft:written_book"
                | "minecraft:enchanted_book"
                | "minecraft:music_disc_13"
                | "minecraft:music_disc_cat"
        )
    {
        1
    } else if matches!(
        item,
        "minecraft:ender_pearl"
            | "minecraft:snowball"
            | "minecraft:egg"
            | "minecraft:honey_bottle"
            | "minecraft:bucket"
            | "minecraft:water_bucket"
            | "minecraft:lava_bucket"
            | "minecraft:milk_bucket"
            | "minecraft:oak_sign"
            | "minecraft:oak_hanging_sign"
    ) {
        16
    } else {
        64
    }
}

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
            let stack = CommandItemStack {
                item: parse_resource_identifier(item)?,
                count: 1,
            };
            set_entity_items(
                state,
                parse_entity_list(targets),
                &parse_item_slot(slot)?,
                stack,
            )
        }
        ["item", "replace", "entity", targets, slot, "with", item, count] => {
            let count = parse_item_count(count)?;
            let stack = CommandItemStack {
                item: parse_resource_identifier(item)?,
                count,
            };
            set_entity_items(
                state,
                parse_entity_list(targets),
                &parse_item_slot(slot)?,
                stack,
            )
        }
        ["item", "replace", "block", x, y, z, slot, "with", item] => {
            let pos = parse_block_pos(x, y, z)?;
            let stack = CommandItemStack {
                item: parse_resource_identifier(item)?,
                count: 1,
            };
            set_block_item(state, pos, &parse_item_slot(slot)?, stack)
        }
        ["item", "replace", "block", x, y, z, slot, "with", item, count] => {
            let pos = parse_block_pos(x, y, z)?;
            let count = parse_item_count(count)?;
            let stack = CommandItemStack {
                item: parse_resource_identifier(item)?,
                count,
            };
            set_block_item(state, pos, &parse_item_slot(slot)?, stack)
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
                upsert_entity_item(state, target, &slot, Some(modified));
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

pub(super) fn parse_item_count(input: &str) -> Result<i32, CommandError> {
    let count = parse_i32(input)?;
    if (1..=99).contains(&count) {
        Ok(count)
    } else {
        Err(CommandError::InvalidSyntax)
    }
}

pub(super) fn parse_item_slot(input: &str) -> Result<String, CommandError> {
    let valid_named_slot = input == "weapon"
        || input == "weapon.mainhand"
        || input == "weapon.offhand"
        || input == "armor.head"
        || input == "armor.chest"
        || input == "armor.legs"
        || input == "armor.feet"
        || input
            .strip_prefix("container.")
            .or_else(|| input.strip_prefix("hotbar."))
            .or_else(|| input.strip_prefix("inventory."))
            .and_then(|index| index.parse::<u8>().ok())
            .is_some();
    if valid_named_slot || input.parse::<i32>().is_ok_and(|slot| slot >= 0) {
        Ok(input.to_string())
    } else {
        Err(CommandError::InvalidSyntax)
    }
}

pub(super) fn set_entity_items(
    state: &mut ServerCommandState,
    targets: Vec<EntityRef>,
    slot: &str,
    stack: CommandItemStack,
) -> Result<CommandResult, CommandError> {
    if targets.is_empty() {
        return Err(CommandError::ItemTargetNoChanges);
    }
    for target in &targets {
        upsert_entity_item(state, target.clone(), slot, Some(stack.clone()));
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
    stack: CommandItemStack,
) -> Result<CommandResult, CommandError> {
    upsert_block_item(state, pos, slot, Some(stack));
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
) -> Result<CommandItemStack, CommandError> {
    state
        .entity_item_slots
        .iter()
        .find(|entry| entry.entity.id == entity.id && entry.slot == slot)
        .and_then(|entry| entry.item.clone())
        .ok_or(CommandError::ItemSourceNoSuchSlot)
}

pub(super) fn get_block_item(
    state: &ServerCommandState,
    pos: &BlockPos,
    slot: &str,
) -> Result<CommandItemStack, CommandError> {
    state
        .block_item_slots
        .iter()
        .find(|entry| entry.pos == *pos && entry.slot == slot)
        .and_then(|entry| entry.item.clone())
        .ok_or(CommandError::ItemSourceNoSuchSlot)
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
    stack: CommandItemStack,
) -> Result<CommandItemStack, CommandError> {
    let modifier = parse_resource_identifier(modifier)?;
    let max_stack_size = item_max_stack_size(&stack.item);
    let output = CommandItemStack {
        item: stack.item.clone(),
        count: stack.count.min(max_stack_size),
    };
    if let Some(target) = target {
        state.item_modifier_events.push(CommandItemModifierEvent {
            target,
            modifier,
            input: Some(stack),
            output: Some(output.clone()),
        });
    }
    Ok(output)
}
